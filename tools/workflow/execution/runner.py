from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from execution.closeout_claims import closeout_claim_errors
from execution.compiler import first_action
from execution.contracts import ActionContract, ContractPack, ValidationCommand
from execution.evidence import EVIDENCE_ROOT, resolve_evidence_records
from execution.workspace import create_full_snapshot, dispose_workspace, import_scoped_changes
from execution.writers import AgentBackend, run_writer_in_workspace


@dataclass(frozen=True)
class CommandResult:
    command_id: str
    argv: tuple[str, ...]
    returncode: int

    def __str__(self) -> str:
        return f"{self.command_id} ({' '.join(self.argv)}) -> exit {self.returncode}"


@dataclass(frozen=True)
class HarnessRunResult:
    action_id: str
    written_paths: tuple[Path, ...]
    validation_results: tuple[CommandResult, ...]
    evidence_paths: tuple[Path, ...]
    next_action: str


def command_cwd(command: ValidationCommand, *, repo_root: Path) -> Path:
    cwd = repo_root / command.cwd
    try:
        cwd.resolve().relative_to(repo_root.resolve())
    except ValueError as error:
        raise WorkflowError(f"{command.command_id}: validation command cwd escapes repository: {command.cwd}") from error
    return cwd


def run_validation_commands(commands: list[ValidationCommand], *, repo_root: Path = REPO_ROOT) -> tuple[CommandResult, ...]:
    results: list[CommandResult] = []
    for command in commands:
        if command.blocked_reason or command.command_id == "blocked":
            label = command.raw or " ".join(command.argv)
            raise WorkflowError(f"validation command is blocked: {label}: {command.blocked_reason or 'blocked'}")
        completed = subprocess.run(
            command.argv,
            cwd=command_cwd(command, repo_root=repo_root),
            shell=False,
            text=True,
            capture_output=True,
            check=False,
            timeout=command.timeout_seconds,
        )
        results.append(CommandResult(command_id=command.command_id, argv=tuple(command.argv), returncode=completed.returncode))
        if completed.returncode != 0:
            combined = "\n".join(part for part in (completed.stdout.strip(), completed.stderr.strip()) if part)
            detail = f"\n{combined}" if combined else ""
            raise WorkflowError(f"validation failed: {' '.join(command.argv)} -> exit {completed.returncode}{detail}")
    return tuple(results)


def run_action(
    action: ActionContract,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool,
    repo_root: Path = REPO_ROOT,
    run_validations: bool = True,
    evidence_root: Path = EVIDENCE_ROOT,
) -> HarnessRunResult:
    if action.writer_strategy == "proof_aggregation_writer":
        claim_errors = closeout_claim_errors(action, evidence_root=evidence_root)
        if claim_errors:
            raise WorkflowError("\n".join(claim_errors))
        validation_results = run_validation_commands(action.validation_commands, repo_root=repo_root) if run_validations else ()
        evidence_paths = tuple(
            resolve_evidence_records(
                action,
                validation_results=validation_results,
                written_paths=[],
                evidence_root=evidence_root,
                repo_root=repo_root,
            )
        )
        return HarnessRunResult(
            action_id=action.action_id,
            written_paths=(),
            validation_results=validation_results,
            evidence_paths=evidence_paths,
            next_action=f"{action.action_id} completed; recompute the next legal ActionContract before continuing.",
        )

    workspace = create_full_snapshot(action, repo_root=repo_root)
    try:
        run_writer_in_workspace(action, workspace.workspace, backend=backend, lock_validated=lock_validated)
        validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace) if run_validations else ()
        written_paths = import_scoped_changes(action, workspace, repo_root=repo_root)
    finally:
        dispose_workspace(workspace)
    evidence_paths: list[Path] = []
    if run_validations:
        evidence_paths = resolve_evidence_records(
            action,
            validation_results=validation_results,
            written_paths=written_paths,
            evidence_root=evidence_root,
            repo_root=repo_root,
        )
    return HarnessRunResult(
        action_id=action.action_id,
        written_paths=tuple(written_paths),
        validation_results=validation_results,
        evidence_paths=tuple(evidence_paths),
        next_action=f"{action.action_id} completed; recompute the next legal ActionContract before continuing.",
    )


def run_next_action(
    pack: ContractPack,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool,
    repo_root: Path = REPO_ROOT,
    run_validations: bool = True,
    evidence_root: Path = EVIDENCE_ROOT,
) -> HarnessRunResult:
    action = first_action(pack)
    if action is None:
        raise WorkflowError(f"{pack.track_id}: no remaining ActionContracts")
    result = run_action(
        action,
        backend=backend,
        lock_validated=lock_validated,
        repo_root=repo_root,
        run_validations=run_validations,
        evidence_root=evidence_root,
    )
    if result.written_paths:
        changed = ", ".join(repo_path(path) for path in result.written_paths)
    else:
        changed = "no files"
    return HarnessRunResult(
        action_id=result.action_id,
        written_paths=result.written_paths,
        validation_results=result.validation_results,
        evidence_paths=result.evidence_paths,
        next_action=f"{result.next_action} Changed: {changed}",
    )
