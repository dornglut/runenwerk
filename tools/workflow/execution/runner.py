from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from execution.closeout_claims import closeout_claim_errors
from execution.compiler import first_action
from execution.closeouts import run_runtime_closeout
from execution.contracts import ActionContract, ContractPack, ValidationCommand
from execution.evidence import EVIDENCE_ROOT, write_resolver_evidence_records
from execution.planning import refresh_existing_contract_packs, run_planning_expansion
from execution.compiler import CONTRACT_PACK_ROOT
from execution.workspace import (
    changed_files,
    create_full_snapshot,
    dispose_workspace,
    file_digests,
    import_scoped_changes,
    reset_validation_only_outputs,
    validate_agent_changed_files,
    validate_evidence_changed_files,
    validate_validation_changed_files,
)
from execution.writers import AgentBackend, run_writer_in_workspace


LOCAL_AGENT_TRANSCRIPT_ROOT = REPO_ROOT / ".runenwerk/track-execution-transcripts"


@dataclass(frozen=True)
class CommandResult:
    command_id: str
    argv: tuple[str, ...]
    returncode: int
    files_changed: tuple[str, ...] = ()

    def __str__(self) -> str:
        return f"{self.command_id} ({' '.join(self.argv)}) -> exit {self.returncode}"


@dataclass(frozen=True)
class HarnessRunResult:
    action_id: str
    written_paths: tuple[Path, ...]
    validation_results: tuple[CommandResult, ...]
    evidence_paths: tuple[Path, ...]
    transcript_paths: tuple[Path, ...]
    next_action: str
    agent_files_changed: tuple[Path, ...] = ()
    validation_files_changed: tuple[Path, ...] = ()
    evidence_files_changed: tuple[Path, ...] = ()


def command_cwd(command: ValidationCommand, *, repo_root: Path) -> Path:
    cwd = repo_root / command.cwd
    try:
        cwd.resolve().relative_to(repo_root.resolve())
    except ValueError as error:
        raise WorkflowError(f"{command.command_id}: validation command cwd escapes repository: {command.cwd}") from error
    return cwd


def paths_for_changed(repo_root: Path, changed: list[str]) -> tuple[Path, ...]:
    return tuple(repo_root / path for path in changed)


def validate_agent_phase(action: ActionContract, workspace_root: Path, before: dict[str, str]) -> tuple[dict[str, str], tuple[Path, ...]]:
    after = file_digests(workspace_root)
    changed = changed_files(before, after)
    errors = validate_agent_changed_files(action, changed)
    if errors:
        raise WorkflowError("\n".join(errors))
    return after, paths_for_changed(workspace_root, changed)


def validation_files_from_results(results: tuple[CommandResult, ...], *, workspace_root: Path) -> tuple[Path, ...]:
    paths: list[Path] = []
    for result in results:
        paths.extend(workspace_root / changed for changed in result.files_changed)
    return tuple(dict.fromkeys(paths))


def write_evidence_phase(
    action: ActionContract,
    *,
    validation_results: tuple[CommandResult, ...],
    workspace_root: Path,
    run_id: str | None,
) -> tuple[list[Path], tuple[Path, ...]]:
    before = file_digests(workspace_root)
    workspace_evidence_paths = write_resolver_evidence_records(
        action,
        validation_results=validation_results,
        workspace_root=workspace_root,
        run_id=run_id,
    )
    after = file_digests(workspace_root)
    changed = changed_files(before, after)
    errors = validate_evidence_changed_files(action, changed)
    if errors:
        raise WorkflowError("\n".join(errors))
    return workspace_evidence_paths, paths_for_changed(workspace_root, changed)


def run_validation_commands(
    commands: list[ValidationCommand],
    *,
    repo_root: Path = REPO_ROOT,
    action: ActionContract | None = None,
) -> tuple[CommandResult, ...]:
    results: list[CommandResult] = []
    for command in commands:
        if command.blocked_reason or command.command_id == "blocked":
            label = command.raw or " ".join(command.argv)
            raise WorkflowError(f"validation command is blocked: {label}: {command.blocked_reason or 'blocked'}")
        before = file_digests(repo_root) if action is not None else {}
        completed = subprocess.run(
            command.argv,
            cwd=command_cwd(command, repo_root=repo_root),
            shell=False,
            text=True,
            capture_output=True,
            check=False,
            timeout=command.timeout_seconds,
        )
        after = file_digests(repo_root) if action is not None else {}
        command_changed = changed_files(before, after) if action is not None else []
        if action is not None:
            errors = validate_validation_changed_files(action, command, command_changed)
            if errors:
                raise WorkflowError("\n".join(errors))
        results.append(
            CommandResult(
                command_id=command.command_id,
                argv=tuple(command.argv),
                returncode=completed.returncode,
                files_changed=tuple(command_changed),
            )
        )
        if completed.returncode != 0:
            combined = "\n".join(part for part in (completed.stdout.strip(), completed.stderr.strip()) if part)
            detail = f"\n{combined}" if combined else ""
            raise WorkflowError(f"validation failed: {' '.join(command.argv)} -> exit {completed.returncode}{detail}")
    return tuple(results)


def refresh_derived_contract_packs_for_action(
    action: ActionContract,
    *,
    workspace_root: Path,
    repo_root: Path,
    contract_pack_root: Path,
) -> None:
    if not action.manifest_source_path:
        return
    production_source = workspace_root / action.production_source_path
    roadmap_source = workspace_root / action.roadmap_source_path
    manifest_path = workspace_root / action.manifest_source_path
    if not production_source.exists() or not roadmap_source.exists() or not manifest_path.exists():
        return
    refresh_existing_contract_packs(
        action,
        workspace_root=workspace_root,
        repo_root=repo_root,
        contract_pack_root=contract_pack_root,
        production_source=production_source,
        roadmap_source=roadmap_source,
        manifest_root=manifest_path.parent,
    )


def run_action(
    action: ActionContract,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool,
    repo_root: Path = REPO_ROOT,
    run_validations: bool = True,
    evidence_root: Path = EVIDENCE_ROOT,
    contract_pack_root: Path = CONTRACT_PACK_ROOT,
    run_id: str | None = None,
) -> HarnessRunResult:
    if action.executor_kind == "planning_expansion":
        workspace = create_full_snapshot(action, repo_root=repo_root)
        try:
            phase_baseline = workspace.tree_baseline
            run_planning_expansion(
                action,
                workspace_root=workspace.workspace,
                repo_root=repo_root,
                contract_pack_root=contract_pack_root,
            )
            phase_baseline, agent_files = validate_agent_phase(action, workspace.workspace, phase_baseline)
            validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action) if run_validations else ()
            validation_files = validation_files_from_results(validation_results, workspace_root=workspace.workspace)
            workspace_evidence_paths, evidence_files = (
                write_evidence_phase(action, validation_results=validation_results, workspace_root=workspace.workspace, run_id=run_id)
                if run_validations and action.evidence_required
                else ([], ())
            )
            written_paths = import_scoped_changes(action, workspace, repo_root=repo_root)
        finally:
            dispose_workspace(workspace)
        evidence_paths = [repo_root / path.relative_to(workspace.workspace) for path in workspace_evidence_paths]
        return HarnessRunResult(
            action_id=action.action_id,
            written_paths=tuple(written_paths),
            validation_results=validation_results,
            evidence_paths=tuple(evidence_paths),
            transcript_paths=(),
            next_action=f"{action.action_id} completed; recompute the next legal ActionContract before continuing.",
            agent_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in agent_files),
            validation_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in validation_files),
            evidence_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in evidence_files),
        )

    if action.executor_kind == "runtime_closeout":
        workspace = create_full_snapshot(action, repo_root=repo_root)
        try:
            phase_baseline = workspace.tree_baseline
            workspace_evidence_root = workspace.workspace / "docs-site/src/content/docs/reports/execution-evidence"
            claim_errors = closeout_claim_errors(action, evidence_root=workspace_evidence_root)
            if claim_errors:
                raise WorkflowError("\n".join(claim_errors))
            run_runtime_closeout(action, workspace_root=workspace.workspace)
            refresh_derived_contract_packs_for_action(
                action,
                workspace_root=workspace.workspace,
                repo_root=repo_root,
                contract_pack_root=contract_pack_root,
            )
            _phase_baseline, agent_files = validate_agent_phase(action, workspace.workspace, phase_baseline)
            validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action) if run_validations else ()
            validation_files = validation_files_from_results(validation_results, workspace_root=workspace.workspace)
            written_paths = import_scoped_changes(action, workspace, repo_root=repo_root)
        finally:
            dispose_workspace(workspace)
        return HarnessRunResult(
            action_id=action.action_id,
            written_paths=tuple(written_paths),
            validation_results=validation_results,
            evidence_paths=(),
            transcript_paths=(),
            next_action=f"{action.action_id} closed; recompute the next legal ActionContract before continuing.",
            agent_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in agent_files),
            validation_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in validation_files),
        )

    if action.executor_kind == "handoff_closeout":
        workspace = create_full_snapshot(action, repo_root=repo_root)
        try:
            phase_baseline = workspace.tree_baseline
            transcript_paths: list[Path] = []
            run_writer_in_workspace(
                action,
                workspace.workspace,
                backend=backend,
                lock_validated=lock_validated,
                transcript_root=agent_transcript_root(action, repo_root=repo_root, run_id=run_id),
                transcript_paths_out=transcript_paths,
            )
            reset_validation_only_outputs(action, workspace)
            phase_baseline, agent_files = validate_agent_phase(action, workspace.workspace, phase_baseline)
            validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action) if run_validations else ()
            validation_files = list(validation_files_from_results(validation_results, workspace_root=workspace.workspace))
            workspace_evidence_paths, evidence_files = (
                write_evidence_phase(action, validation_results=validation_results, workspace_root=workspace.workspace, run_id=run_id)
                if run_validations and action.evidence_required
                else ([], ())
            )
            phase_baseline = file_digests(workspace.workspace)
            workspace_evidence_root = workspace.workspace / "docs-site/src/content/docs/reports/execution-evidence"
            claim_errors = closeout_claim_errors(action, evidence_root=workspace_evidence_root)
            if claim_errors:
                raise WorkflowError("\n".join(claim_errors))
            run_runtime_closeout(action, workspace_root=workspace.workspace)
            refresh_derived_contract_packs_for_action(
                action,
                workspace_root=workspace.workspace,
                repo_root=repo_root,
                contract_pack_root=contract_pack_root,
            )
            _phase_baseline, closeout_files = validate_agent_phase(action, workspace.workspace, phase_baseline)
            validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action) if run_validations else ()
            validation_files.extend(validation_files_from_results(validation_results, workspace_root=workspace.workspace))
            written_paths = import_scoped_changes(action, workspace, repo_root=repo_root)
        finally:
            dispose_workspace(workspace)
        evidence_paths = tuple(repo_root / path.relative_to(workspace.workspace) for path in workspace_evidence_paths)
        agent_phase_files = tuple(agent_files) + tuple(closeout_files)
        return HarnessRunResult(
            action_id=action.action_id,
            written_paths=tuple(written_paths),
            validation_results=validation_results,
            evidence_paths=evidence_paths,
            transcript_paths=tuple(transcript_paths),
            next_action=f"{action.action_id} closed; recompute the next legal ActionContract before continuing.",
            agent_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in agent_phase_files),
            validation_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in validation_files),
            evidence_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in evidence_files),
        )

    if action.writer_strategy == "proof_aggregation_writer":
        workspace = create_full_snapshot(action, repo_root=repo_root)
        try:
            phase_baseline = workspace.tree_baseline
            claim_errors = closeout_claim_errors(action, evidence_root=evidence_root)
            if claim_errors:
                raise WorkflowError("\n".join(claim_errors))
            refresh_derived_contract_packs_for_action(
                action,
                workspace_root=workspace.workspace,
                repo_root=repo_root,
                contract_pack_root=contract_pack_root,
            )
            phase_baseline, agent_files = validate_agent_phase(action, workspace.workspace, phase_baseline)
            validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action) if run_validations else ()
            validation_files = validation_files_from_results(validation_results, workspace_root=workspace.workspace)
            workspace_evidence_paths, evidence_files = (
                write_evidence_phase(action, validation_results=validation_results, workspace_root=workspace.workspace, run_id=run_id)
                if run_validations and action.evidence_required
                else ([], ())
            )
            written_paths = import_scoped_changes(action, workspace, repo_root=repo_root)
        finally:
            dispose_workspace(workspace)
        evidence_paths = tuple(repo_root / path.relative_to(workspace.workspace) for path in workspace_evidence_paths)
        return HarnessRunResult(
            action_id=action.action_id,
            written_paths=tuple(written_paths),
            validation_results=validation_results,
            evidence_paths=evidence_paths,
            transcript_paths=(),
            next_action=f"{action.action_id} completed; recompute the next legal ActionContract before continuing.",
            agent_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in agent_files),
            validation_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in validation_files),
            evidence_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in evidence_files),
        )

    workspace = create_full_snapshot(action, repo_root=repo_root)
    try:
        phase_baseline = workspace.tree_baseline
        transcript_paths: list[Path] = []
        run_writer_in_workspace(
            action,
            workspace.workspace,
            backend=backend,
            lock_validated=lock_validated,
            transcript_root=agent_transcript_root(action, repo_root=repo_root, run_id=run_id),
            transcript_paths_out=transcript_paths,
        )
        reset_validation_only_outputs(action, workspace)
        refresh_derived_contract_packs_for_action(
            action,
            workspace_root=workspace.workspace,
            repo_root=repo_root,
            contract_pack_root=contract_pack_root,
        )
        phase_baseline, agent_files = validate_agent_phase(action, workspace.workspace, phase_baseline)
        validation_results = run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action) if run_validations else ()
        validation_files = validation_files_from_results(validation_results, workspace_root=workspace.workspace)
        workspace_evidence_paths, evidence_files = (
            write_evidence_phase(action, validation_results=validation_results, workspace_root=workspace.workspace, run_id=run_id)
            if run_validations and action.evidence_required
            else ([], ())
        )
        written_paths = import_scoped_changes(action, workspace, repo_root=repo_root)
    finally:
        dispose_workspace(workspace)
    evidence_paths = [repo_root / path.relative_to(workspace.workspace) for path in workspace_evidence_paths]
    return HarnessRunResult(
        action_id=action.action_id,
        written_paths=tuple(written_paths),
        validation_results=validation_results,
        evidence_paths=tuple(evidence_paths),
        transcript_paths=tuple(transcript_paths),
        next_action=f"{action.action_id} completed; recompute the next legal ActionContract before continuing.",
        agent_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in agent_files),
        validation_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in validation_files),
        evidence_files_changed=tuple(repo_root / path.relative_to(workspace.workspace) for path in evidence_files),
    )


def agent_transcript_root(action: ActionContract, *, repo_root: Path, run_id: str | None) -> Path | None:
    if action.writer_strategy != "agent_writer":
        return None
    resolved_run_id = run_id or "manual"
    return (
        repo_root
        / ".runenwerk/track-execution-transcripts"
        / action.track_id.lower()
        / resolved_run_id
    )


def run_next_action(
    pack: ContractPack,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool,
    repo_root: Path = REPO_ROOT,
    run_validations: bool = True,
    evidence_root: Path = EVIDENCE_ROOT,
    contract_pack_root: Path = CONTRACT_PACK_ROOT,
    run_id: str | None = None,
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
        contract_pack_root=contract_pack_root,
        run_id=run_id,
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
        transcript_paths=result.transcript_paths,
        next_action=f"{result.next_action} Changed: {changed}",
        agent_files_changed=result.agent_files_changed,
        validation_files_changed=result.validation_files_changed,
        evidence_files_changed=result.evidence_files_changed,
    )
