from __future__ import annotations

import shutil
import tempfile
import re
import subprocess
from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path

from roadmap_state import REPO_ROOT, WorkflowError, normalize_repo_path, repo_path, path_within_scope

from execution.contracts import ActionContract


SNAPSHOT_EXCLUDES = {
    ".git",
    ".venv",
    "target",
    "node_modules",
    ".cache",
    ".pytest_cache",
    "__pycache__",
}


def optional_digest(path: Path) -> str | None:
    if not path.exists():
        return None
    if path.is_dir():
        raise WorkflowError(f"cannot digest directory as action output: {repo_path(path)}")
    return sha256(path.read_bytes()).hexdigest()


def file_digests(root: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if any(part in SNAPSHOT_EXCLUDES for part in path.relative_to(root).parts):
            continue
        if path.suffix == ".pyc":
            continue
        relative = normalize_repo_path(str(path.relative_to(root)))
        result[relative] = sha256(path.read_bytes()).hexdigest()
    return result


def changed_files(before: dict[str, str], after: dict[str, str]) -> list[str]:
    keys = sorted(set(before) | set(after))
    return [key for key in keys if before.get(key) != after.get(key)]


def ignore_snapshot_entries(_directory: str, names: list[str]) -> set[str]:
    return {name for name in names if name in SNAPSHOT_EXCLUDES}


@dataclass(frozen=True)
class ActionWorkspace:
    temp_root: Path
    workspace: Path
    baseline: dict[str, str | None]
    baseline_content: dict[str, bytes | None]
    tree_baseline: dict[str, str]


def create_full_snapshot(action: ActionContract, *, repo_root: Path = REPO_ROOT) -> ActionWorkspace:
    temp_root = Path(tempfile.mkdtemp(prefix=f"execution-{action.milestone_id.lower()}-"))
    workspace = temp_root / "workspace"
    shutil.copytree(repo_root, workspace, ignore=ignore_snapshot_entries)
    initialize_snapshot_git_index(workspace)
    outputs = importable_outputs(action)
    baseline = {output: optional_digest(repo_root / output) for output in outputs}
    baseline_content = {
        output: (repo_root / output).read_bytes() if (repo_root / output).exists() and (repo_root / output).is_file() else None
        for output in outputs
    }
    return ActionWorkspace(
        temp_root=temp_root,
        workspace=workspace,
        baseline=baseline,
        baseline_content=baseline_content,
        tree_baseline=file_digests(workspace),
    )


def initialize_snapshot_git_index(workspace: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=workspace, check=True)
    subprocess.run(["git", "add", "-A"], cwd=workspace, check=True)


def dispose_workspace(workspace: ActionWorkspace) -> None:
    shutil.rmtree(workspace.temp_root, ignore_errors=True)


def allowed_output_set(action: ActionContract) -> set[str]:
    return {normalize_repo_path(path) for path in [*action.allowed_outputs, *action.new_outputs]}


def validation_output_set(action: ActionContract) -> set[str]:
    return {
        normalize_repo_path(path)
        for command in action.validation_commands
        for path in command.allowed_outputs
    }


def evidence_output_set(action: ActionContract) -> set[str]:
    return {
        normalize_repo_path(path)
        for requirement in [*action.evidence_required, *action.closeout_contract.evidence_required]
        for path in requirement.paths
    }


def importable_outputs(action: ActionContract) -> list[str]:
    return list(
        dict.fromkeys(
            [
                *[normalize_repo_path(path) for path in action.allowed_outputs],
                *[normalize_repo_path(path) for path in action.new_outputs],
                *sorted(validation_output_set(action)),
                *sorted(evidence_output_set(action)),
            ]
        )
    )


def forbidden_output_set(action: ActionContract) -> set[str]:
    return {normalize_repo_path(path) for path in action.forbidden_outputs}


def validate_changed_files(
    action: ActionContract,
    changed: list[str],
    *,
    allowed_paths: set[str] | None = None,
    phase: str = "action",
) -> list[str]:
    errors: list[str] = []
    allowed = allowed_paths if allowed_paths is not None else allowed_output_set(action)
    forbidden = forbidden_output_set(action)
    for path in changed:
        normalized = normalize_repo_path(path)
        if normalized not in allowed:
            if phase == "validation":
                errors.append(
                    f"{action.action_id}: validation command changed undeclared file {normalized}; "
                    "declare it in validation_commands[].allowed_outputs if the tool is allowed to produce it"
                )
            elif phase == "evidence":
                errors.append(f"{action.action_id}: evidence writer changed undeclared file {normalized}")
            else:
                errors.append(f"{action.action_id}: changed undeclared file {normalized}")
        for forbidden_path in forbidden:
            if path_within_scope(normalized, forbidden_path) or path_within_scope(forbidden_path, normalized):
                errors.append(f"{action.action_id}: changed forbidden file {normalized}")
        for pattern in action.forbidden_patterns:
            if re.search(pattern, normalized):
                errors.append(f"{action.action_id}: changed file matches forbidden pattern {pattern!r}: {normalized}")
    return errors


def validate_agent_changed_files(action: ActionContract, changed: list[str]) -> list[str]:
    return validate_changed_files(action, changed, allowed_paths=allowed_output_set(action), phase="agent")


def reset_validation_only_outputs(action: ActionContract, workspace: ActionWorkspace) -> tuple[str, ...]:
    phase_only_outputs = sorted(validation_output_set(action) - allowed_output_set(action) - evidence_output_set(action))
    reset_paths: list[str] = []
    for relative in phase_only_outputs:
        target = workspace.workspace / relative
        baseline_content = workspace.baseline_content.get(relative)
        if baseline_content is None:
            if target.exists():
                if target.is_dir():
                    shutil.rmtree(target)
                else:
                    target.unlink()
                reset_paths.append(relative)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists() or target.read_bytes() != baseline_content:
            target.write_bytes(baseline_content)
            reset_paths.append(relative)
    return tuple(reset_paths)


def validate_validation_changed_files(action: ActionContract, command, changed: list[str]) -> list[str]:
    return validate_changed_files(
        action,
        changed,
        allowed_paths={normalize_repo_path(path) for path in command.allowed_outputs},
        phase="validation",
    )


def validate_evidence_changed_files(action: ActionContract, changed: list[str]) -> list[str]:
    return validate_changed_files(action, changed, allowed_paths=evidence_output_set(action), phase="evidence")


def import_scoped_changes(
    action: ActionContract,
    workspace: ActionWorkspace,
    *,
    repo_root: Path = REPO_ROOT,
) -> list[Path]:
    after = file_digests(workspace.workspace)
    changed = changed_files(workspace.tree_baseline, after)
    errors = validate_changed_files(
        action,
        changed,
        allowed_paths=set(importable_outputs(action)),
        phase="action",
    )
    if errors:
        raise WorkflowError("\n".join(errors))
    imported: list[Path] = []
    new_outputs = {normalize_repo_path(path) for path in action.new_outputs}
    for relative in changed:
        target = repo_root / relative
        baseline = workspace.baseline.get(relative)
        current = optional_digest(target)
        if baseline is None and relative not in new_outputs:
            raise WorkflowError(f"{action.action_id}: created undeclared new file {relative}")
        if current != baseline:
            raise WorkflowError(f"{action.action_id}: target digest drifted before import: {relative}")
        source = workspace.workspace / relative
        if not target.parent.exists():
            if relative not in new_outputs:
                raise WorkflowError(f"{action.action_id}: output parent directory does not exist: {repo_path(target.parent)}")
            target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(source.read_bytes())
        imported.append(target)
    return imported
