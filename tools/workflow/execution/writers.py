from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from execution.contracts import ActionContract


@dataclass(frozen=True)
class AgentResult:
    returncode: int
    stdout: str
    stderr: str


class AgentBackend(Protocol):
    def run(self, *, workspace: Path, prompt: str) -> AgentResult:
        ...


class CodexExecBackend:
    def __init__(self, *, codex_bin: str = "codex", timeout_seconds: int = 1800) -> None:
        self.codex_bin = codex_bin
        self.timeout_seconds = timeout_seconds

    def run(self, *, workspace: Path, prompt: str) -> AgentResult:
        try:
            completed = subprocess.run(
                [
                    self.codex_bin,
                    "exec",
                    "--ephemeral",
                    "--sandbox",
                    "workspace-write",
                    "-C",
                    str(workspace),
                    "-",
                ],
                input=prompt,
                text=True,
                capture_output=True,
                check=False,
                timeout=self.timeout_seconds,
            )
        except subprocess.TimeoutExpired as error:
            return AgentResult(returncode=124, stdout=error.stdout or "", stderr=error.stderr or "codex exec timed out")
        return AgentResult(returncode=completed.returncode, stdout=completed.stdout, stderr=completed.stderr)


def action_prompt(action: ActionContract) -> str:
    allowed = "\n".join(f"- {path}" for path in [*action.allowed_outputs, *action.new_outputs])
    forbidden = "\n".join(f"- {path}" for path in action.forbidden_outputs)
    validations = "\n".join(f"- {' '.join(command.argv)}" for command in action.validation_commands)
    return "\n".join(
        [
            "You are running inside an isolated Track Execution Harness workspace.",
            "Modify only the allowed outputs. Do not run destructive git commands.",
            "",
            f"Action: {action.action_id}",
            f"Execution kind: {action.execution_kind}",
            "",
            "Allowed outputs:",
            allowed or "- none",
            "",
            "Forbidden outputs:",
            forbidden or "- none",
            "",
            "Validation commands after import:",
            validations or "- none",
        ]
    )


def ensure_path_allowed(action: ActionContract, path: str) -> None:
    allowed = set(action.allowed_outputs) | set(action.new_outputs)
    if path not in allowed:
        raise WorkflowError(f"{action.action_id}: writer output is not declared: {path}")


def run_template_writer(action: ActionContract, *, workspace_root: Path) -> list[Path]:
    if not action.template_outputs:
        raise WorkflowError(f"{action.action_id}: template_writer requires template_outputs")
    written: list[Path] = []
    for relative, content in action.template_outputs.items():
        ensure_path_allowed(action, relative)
        target = workspace_root / relative
        if not target.parent.exists():
            raise WorkflowError(f"{action.action_id}: output parent directory does not exist: {repo_path(target.parent)}")
        target.write_text(content, encoding="utf-8", newline="\n")
        written.append(target)
    return written


def run_patch_writer(action: ActionContract, *, workspace_root: Path) -> list[Path]:
    if not action.patches:
        raise WorkflowError(f"{action.action_id}: patch_writer requires patches")
    written: list[Path] = []
    for patch in action.patches:
        ensure_path_allowed(action, patch.path)
        target = workspace_root / patch.path
        if not target.exists():
            raise WorkflowError(f"{action.action_id}: patch target does not exist: {patch.path}")
        text = target.read_text(encoding="utf-8")
        if patch.find not in text:
            raise WorkflowError(f"{action.action_id}: patch find text not found in {patch.path}")
        target.write_text(text.replace(patch.find, patch.replace, 1), encoding="utf-8", newline="\n")
        written.append(target)
    return written


def run_agent_writer(
    action: ActionContract,
    *,
    backend: AgentBackend,
    lock_validated: bool,
    workspace_root: Path,
) -> list[Path]:
    if not lock_validated:
        raise WorkflowError(f"{action.action_id}: agent_writer requires a current execution lock")
    result = backend.run(workspace=workspace_root, prompt=action_prompt(action))
    if result.returncode != 0:
        raise WorkflowError(f"{action.action_id}: agent backend failed with exit {result.returncode}")
    return []


def run_writer_in_workspace(
    action: ActionContract,
    workspace_root: Path,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool = False,
) -> list[Path]:
    if action.writer_strategy == "no_writer":
        raise WorkflowError(f"{action.action_id}: no_writer cannot execute")
    if action.writer_strategy == "template_writer":
        return run_template_writer(action, workspace_root=workspace_root)
    if action.writer_strategy == "patch_writer":
        return run_patch_writer(action, workspace_root=workspace_root)
    if action.writer_strategy == "agent_writer":
        if backend is None:
            backend = CodexExecBackend()
        return run_agent_writer(action, backend=backend, lock_validated=lock_validated, workspace_root=workspace_root)
    if action.writer_strategy == "proof_aggregation_writer":
        return []
    raise WorkflowError(f"{action.action_id}: unsupported writer strategy {action.writer_strategy}")


def run_writer(
    action: ActionContract,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool = False,
    repo_root: Path = REPO_ROOT,
    run_validations: bool = True,
) -> list[Path]:
    from execution.workspace import create_full_snapshot, dispose_workspace, import_scoped_changes

    workspace = create_full_snapshot(action, repo_root=repo_root)
    try:
        run_writer_in_workspace(action, workspace.workspace, backend=backend, lock_validated=lock_validated)
        if run_validations:
            from execution.runner import run_validation_commands

            run_validation_commands(action.validation_commands, repo_root=workspace.workspace)
        return import_scoped_changes(action, workspace, repo_root=repo_root)
    finally:
        dispose_workspace(workspace)
