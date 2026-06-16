from __future__ import annotations

import shutil
import tempfile
import re
import subprocess
import tomllib
from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path
from typing import Any

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
    workspace = temp_root / repo_root.parent.name / repo_root.name
    workspace.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(repo_root, workspace, ignore=ignore_snapshot_entries)
    mirror_external_cargo_path_workspaces(repo_root=repo_root, workspace=workspace, temp_root=temp_root)
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


def mirror_external_cargo_path_workspaces(*, repo_root: Path, workspace: Path, temp_root: Path) -> None:
    """Mirror external Cargo path dependencies so snapshots keep workspace-relative paths valid."""

    repo_root = repo_root.resolve()
    workspace = workspace.resolve()
    source_roots: list[tuple[Path, Path]] = [(repo_root, workspace)]
    copied_roots: set[tuple[Path, Path]] = set()
    index = 0
    while index < len(source_roots):
        source_root, snapshot_source_root = source_roots[index]
        index += 1
        for manifest in cargo_manifests(source_root):
            relative_manifest = manifest.relative_to(source_root)
            for source_dep in cargo_path_dependencies(manifest):
                if any(path_is_within(source_dep, known_source_root) for known_source_root, _ in source_roots):
                    continue
                external_root = cargo_workspace_root_for(source_dep)
                relative_dep = source_dep.relative_to(external_root)
                snapshot_dep = (
                    snapshot_source_root / relative_manifest.parent / manifest_path_for(manifest, source_dep)
                ).resolve()
                snapshot_root = ancestor_for_relative(snapshot_dep, relative_dep)
                if not path_is_within(snapshot_root, temp_root.resolve()):
                    raise WorkflowError(
                        f"external Cargo path dependency would copy outside snapshot root: {source_dep}"
                    )
                copy_key = (external_root, snapshot_root)
                if copy_key in copied_roots:
                    continue
                if not snapshot_root.exists():
                    snapshot_root.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copytree(external_root, snapshot_root, ignore=ignore_snapshot_entries)
                copied_roots.add(copy_key)
                source_roots.append((external_root, snapshot_root))


def cargo_manifests(root: Path) -> list[Path]:
    return [
        path
        for path in root.rglob("Cargo.toml")
        if not any(part in SNAPSHOT_EXCLUDES for part in path.relative_to(root).parts)
    ]


def cargo_path_dependencies(manifest: Path) -> list[Path]:
    try:
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        raise WorkflowError(f"failed to parse Cargo manifest {repo_path(manifest)}: {error}") from error
    paths: list[Path] = []
    for value in cargo_dependency_tables(data):
        dependency_path = value.get("path")
        if not isinstance(dependency_path, str):
            continue
        source_dep = (manifest.parent / dependency_path).resolve()
        if source_dep.exists():
            paths.append(source_dep)
    return paths


def cargo_dependency_tables(data: dict[str, Any]) -> list[dict[str, Any]]:
    tables: list[dict[str, Any]] = []
    for key in ("dependencies", "dev-dependencies", "build-dependencies"):
        tables.extend(dependency_values(data.get(key)))
    target = data.get("target")
    if isinstance(target, dict):
        for target_data in target.values():
            if not isinstance(target_data, dict):
                continue
            for key in ("dependencies", "dev-dependencies", "build-dependencies"):
                tables.extend(dependency_values(target_data.get(key)))
    return tables


def dependency_values(table: Any) -> list[dict[str, Any]]:
    if not isinstance(table, dict):
        return []
    return [value for value in table.values() if isinstance(value, dict)]


def manifest_path_for(manifest: Path, dependency: Path) -> Path:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    for value in cargo_dependency_tables(data):
        dependency_path = value.get("path")
        if isinstance(dependency_path, str) and (manifest.parent / dependency_path).resolve() == dependency:
            return Path(dependency_path)
    raise WorkflowError(f"missing Cargo dependency path for {repo_path(manifest)} -> {dependency}")


def cargo_workspace_root_for(path_dependency: Path) -> Path:
    for candidate in [path_dependency, *path_dependency.parents]:
        manifest = candidate / "Cargo.toml"
        if not manifest.exists():
            continue
        try:
            data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError:
            continue
        if isinstance(data.get("workspace"), dict):
            return candidate
    return path_dependency


def ancestor_for_relative(path: Path, relative: Path) -> Path:
    result = path
    for _part in relative.parts:
        result = result.parent
    return result


def path_is_within(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
    except ValueError:
        return False
    return True


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
