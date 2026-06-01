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
    tree_baseline: dict[str, str]


def create_full_snapshot(action: ActionContract, *, repo_root: Path = REPO_ROOT) -> ActionWorkspace:
    temp_root = Path(tempfile.mkdtemp(prefix=f"execution-{action.milestone_id.lower()}-"))
    workspace = temp_root / "workspace"
    shutil.copytree(repo_root, workspace, ignore=ignore_snapshot_entries)
    initialize_snapshot_git_index(workspace)
    outputs = [*action.allowed_outputs, *action.new_outputs]
    baseline = {output: optional_digest(repo_root / output) for output in outputs}
    return ActionWorkspace(
        temp_root=temp_root,
        workspace=workspace,
        baseline=baseline,
        tree_baseline=file_digests(workspace),
    )


def initialize_snapshot_git_index(workspace: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=workspace, check=True)
    subprocess.run(["git", "add", "-A"], cwd=workspace, check=True)


def dispose_workspace(workspace: ActionWorkspace) -> None:
    shutil.rmtree(workspace.temp_root, ignore_errors=True)


def allowed_output_set(action: ActionContract) -> set[str]:
    return {normalize_repo_path(path) for path in [*action.allowed_outputs, *action.new_outputs]}


def forbidden_output_set(action: ActionContract) -> set[str]:
    return {normalize_repo_path(path) for path in action.forbidden_outputs}


def validate_changed_files(action: ActionContract, changed: list[str]) -> list[str]:
    errors: list[str] = []
    allowed = allowed_output_set(action)
    forbidden = forbidden_output_set(action)
    for path in changed:
        normalized = normalize_repo_path(path)
        if normalized not in allowed:
            errors.append(f"{action.action_id}: changed undeclared file {normalized}")
        for forbidden_path in forbidden:
            if path_within_scope(normalized, forbidden_path) or path_within_scope(forbidden_path, normalized):
                errors.append(f"{action.action_id}: changed forbidden file {normalized}")
        for pattern in action.forbidden_patterns:
            if re.search(pattern, normalized):
                errors.append(f"{action.action_id}: changed file matches forbidden pattern {pattern!r}: {normalized}")
    return errors


def import_scoped_changes(
    action: ActionContract,
    workspace: ActionWorkspace,
    *,
    repo_root: Path = REPO_ROOT,
) -> list[Path]:
    after = file_digests(workspace.workspace)
    changed = changed_files(workspace.tree_baseline, after)
    errors = validate_changed_files(action, changed)
    if errors:
        raise WorkflowError("\n".join(errors))
    imported: list[Path] = []
    for relative in changed:
        target = repo_root / relative
        baseline = workspace.baseline.get(relative)
        current = optional_digest(target)
        if baseline is None and relative not in {normalize_repo_path(path) for path in action.new_outputs}:
            raise WorkflowError(f"{action.action_id}: created undeclared new file {relative}")
        if current != baseline:
            raise WorkflowError(f"{action.action_id}: target digest drifted before import: {relative}")
        source = workspace.workspace / relative
        if not target.parent.exists():
            raise WorkflowError(f"{action.action_id}: output parent directory does not exist: {repo_path(target.parent)}")
        target.write_bytes(source.read_bytes())
        imported.append(target)
    return imported
