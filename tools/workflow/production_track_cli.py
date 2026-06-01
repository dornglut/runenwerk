#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

import typer
from rich.console import Console

sys.path.insert(0, str(Path(__file__).resolve().parent))

from execution.cli import (
    compile_command as execution_compile_command,
    lock_command as execution_lock_command,
    next_command as execution_next_command,
    preflight_command as execution_preflight_command,
    run_command as execution_run_command,
)
from execution.compiler import CONTRACT_PACK_ROOT, compile_contract_pack, contract_pack_path, load_contract_pack, write_contract_pack
from execution.evidence import EVIDENCE_ROOT
from execution.ledger import RUN_LEDGER_ROOT
from execution.locks import EXECUTION_LOCK_ROOT, contract_pack_freshness_errors, execution_lock_errors, load_execution_lock
from production_state import PRODUCTION_SOURCE, load_production_tracks
from roadmap_state import ROADMAP_SOURCE, REPO_ROOT, WorkflowError, load_roadmap, repo_path
from track_sources.audit import audit_manifest, full_automation_preflight_errors, manifest_audit_blocker_lines
from track_sources.manifest import TRACK_EXECUTION_MANIFEST_ROOT, load_track_execution_manifest, truth_claim_summary_lines
from track_sources.scaffold import create_manifest_scaffold


FULL_TRACK_PERMISSION_SET = {
    "auto_safe",
    "agent_design",
    "agent_closeout",
    "product_code",
    "product_implementation",
}


console = Console()
app = typer.Typer(no_args_is_help=True, help="Public production-track workflow adapter.")


def load_track_or_raise(track_id: str, *, production_source: Path = PRODUCTION_SOURCE):
    planning = load_production_tracks(production_source)
    track = next((candidate for candidate in planning.tracks if candidate.id == track_id), None)
    if track is None:
        raise WorkflowError(f"{track_id}: not present in production tracks source")
    return track


def load_manifest_or_raise(track_id: str, *, manifest_source_root: Path):
    loaded = load_track_execution_manifest(track_id, root=manifest_source_root)
    if loaded is None:
        raise WorkflowError(f"{track_id}: missing Track Execution Manifest")
    return loaded


def executable_without_pack_error(track_id: str, *, contract_pack_root: Path) -> WorkflowError:
    return WorkflowError(
        f"{track_id}: executable/full-automation track requires Execution Contract Pack at "
        f"{repo_path(contract_pack_path(track_id, root=contract_pack_root))}; legacy execution fallback is forbidden"
    )


def lock_errors_allow_agent_refresh(errors: list[str]) -> bool:
    if not errors:
        return False
    disallowed_fragments = (
        "requested permissions exceed execution lock grants",
        "requested permissions are denied by execution lock",
    )
    return not any(fragment in error for error in errors for fragment in disallowed_fragments)


def run_source_full_automation_preflight(
    track_id: str,
    *,
    loaded,
    allow: list[str],
    production_source: Path,
    roadmap_source: Path,
) -> None:
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    track_model = next(candidate for candidate in planning.tracks if candidate.id == track_id)
    errors = full_automation_preflight_errors(
        loaded,
        track=track_model,
        roadmap=roadmap,
        allow={permission.strip() for permission in allow if permission.strip()},
    )
    if errors:
        raise WorkflowError("Track Execution Manifest full-automation blockers:\n" + "\n".join(errors))


@app.command("plan-track")
def plan_track(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
) -> None:
    try:
        load_track_or_raise(track, production_source=production_source)
        loaded = load_track_execution_manifest(track, root=manifest_source_root)
        if loaded is None:
            path = create_manifest_scaffold(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_root=manifest_source_root,
            )
            console.print("[green]Track Execution Manifest scaffold written.[/green]")
            console.print(f"Manifest: {repo_path(path)}")
            console.print("No implementation authority is created by this command.")
            return
        planning = load_production_tracks(production_source)
        roadmap = load_roadmap(roadmap_source)
        track_model = next(candidate for candidate in planning.tracks if candidate.id == track)
        console.print(f"Track Execution Manifest already exists: {repo_path(loaded.path)}")
        errors = audit_manifest(loaded, track=track_model, roadmap=roadmap)
        if errors:
            console.print("[yellow]Existing manifest has audit blockers:[/yellow]")
            for line in manifest_audit_blocker_lines(errors):
                console.print(line)
        else:
            console.print("[green]Existing manifest audit passed.[/green]")
        console.print("No implementation authority is created by this command.")
    except WorkflowError as error:
        console.print("[red]production:plan-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("expand-track")
def expand_track(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
) -> None:
    try:
        load_track_or_raise(track, production_source=production_source)
        load_roadmap(roadmap_source)
        loaded = load_manifest_or_raise(track, manifest_source_root=manifest_source_root)
        candidates = [entry for entry in loaded.manifest.milestones if entry.future_wr_candidate]
        console.print(f"Track Expansion candidates for {track}:")
        for entry in candidates:
            console.print(f"- {entry.future_wr_candidate}: {entry.milestone_id} - {entry.title}")
        console.print("production:expand-track is read-only.")
    except WorkflowError as error:
        console.print("[red]production:expand-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("complete-track-contracts")
def complete_track_contracts(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
) -> None:
    try:
        execution_compile_command(
            track=track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
            contract_pack_root=contract_pack_root,
        )
    except WorkflowError as error:
        console.print("[red]production:complete-track-contracts failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("lock-track")
def lock_track(
    track: str = typer.Option(..., "--track"),
    locked_by: str = typer.Option("human", "--locked-by"),
    allow: list[str] = typer.Option(list(FULL_TRACK_PERMISSION_SET), "--allow"),
    deny: list[str] = typer.Option(["crate_creation", "foundation_extraction"], "--deny"),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
    lock_source_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
) -> None:
    try:
        execution_lock_command(
            track=track,
            locked_by=locked_by,
            mode="full-track",
            allow=allow,
            deny=deny,
            contract_pack_root=contract_pack_root,
            lock_root=lock_source_root,
        )
    except WorkflowError as error:
        console.print("[red]production:lock-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("run-track")
def run_track(
    track: str = typer.Option(..., "--track"),
    allow: list[str] = typer.Option([], "--allow"),
    deny: list[str] = typer.Option([], "--deny"),
    max_actions: int = typer.Option(1, "--max-actions", min=1),
    mode: str = typer.Option("bounded-segment", "--mode"),
    preflight_only: bool = typer.Option(False, "--preflight-only"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    lock_source_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
    run_ledger_root: Path = typer.Option(RUN_LEDGER_ROOT),
    evidence_root: Path | None = typer.Option(None, "--evidence-root"),
) -> None:
    try:
        if mode not in {"single-step", "bounded-segment", "full-track", "agent-track"}:
            raise WorkflowError("--mode must be one of single-step, bounded-segment, full-track, agent-track")
        loaded = load_manifest_or_raise(track, manifest_source_root=manifest_source_root)
        pack = load_contract_pack(track, root=contract_pack_root)
        if mode == "agent-track":
            if not loaded.manifest.ai_executable and not loaded.manifest.full_automation_target:
                raise WorkflowError(f"{track}: agent-track mode requires ai_executable or full_automation_target manifest authority")
            if pack is None or contract_pack_freshness_errors(pack):
                pack = compile_contract_pack(
                    track,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    manifest_root=manifest_source_root,
                    contract_pack_root=contract_pack_root,
                )
                write_contract_pack(pack, root=contract_pack_root)
                console.print("[green]Execution Contract Pack prepared.[/green]")
            execution_mode = "single-action" if max_actions == 1 else "full-track"
            execution_preflight_command(track=track, mode=execution_mode, allow=allow, contract_pack_root=contract_pack_root)
            lock_errors = execution_lock_errors(
                track,
                contract_pack_root=contract_pack_root,
                lock_root=lock_source_root,
                requested_permissions=set(allow),
                run_mode=execution_mode,
            )
            if lock_errors:
                if load_execution_lock(track, root=lock_source_root) is not None and not lock_errors_allow_agent_refresh(lock_errors):
                    raise WorkflowError("\n".join(lock_errors))
                execution_lock_command(
                    track=track,
                    locked_by="agent-track",
                    mode=execution_mode,
                    allow=allow,
                    deny=deny,
                    contract_pack_root=contract_pack_root,
                    lock_root=lock_source_root,
                )
            execution_run_command(
                track=track,
                mode=execution_mode,
                allow=allow,
                deny=deny,
                max_actions=max_actions,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
                contract_pack_root=contract_pack_root,
                lock_root=lock_source_root,
                run_ledger_root=run_ledger_root,
                evidence_root=evidence_root or EVIDENCE_ROOT,
                repo_root=REPO_ROOT,
            )
            return
        if mode == "full-track":
            if pack is None:
                if loaded.manifest.ai_executable or loaded.manifest.full_automation_target:
                    raise executable_without_pack_error(track, contract_pack_root=contract_pack_root)
                raise WorkflowError(f"{track}: full-track mode requires an Execution Contract Pack")
            run_source_full_automation_preflight(
                track,
                loaded=loaded,
                allow=allow,
                production_source=production_source,
                roadmap_source=roadmap_source,
            )
            if preflight_only:
                execution_preflight_command(track=track, mode="full-track", allow=allow, contract_pack_root=contract_pack_root)
            else:
                execution_run_command(
                    track=track,
                    mode="full-track",
                    allow=allow,
                    deny=deny,
                    max_actions=max_actions,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    manifest_source_root=manifest_source_root,
                    contract_pack_root=contract_pack_root,
                    lock_root=lock_source_root,
                    run_ledger_root=run_ledger_root,
                    evidence_root=evidence_root or EVIDENCE_ROOT,
                    repo_root=REPO_ROOT,
                )
            return
        if loaded.manifest.ai_executable or loaded.manifest.full_automation_target:
            raise WorkflowError(f"{track}: executable/full-automation tracks must use --mode full-track")
        raise WorkflowError(f"{track}: non-executable track mutation is not implemented in the clean adapter")
    except WorkflowError as error:
        console.print("[red]production:run-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("next")
def next_action(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
) -> None:
    try:
        pack = load_contract_pack(track, root=contract_pack_root)
        if pack is not None:
            execution_next_command(track=track, contract_pack_root=contract_pack_root)
            return
        loaded = load_manifest_or_raise(track, manifest_source_root=manifest_source_root)
        if loaded.manifest.ai_executable or loaded.manifest.full_automation_target:
            raise executable_without_pack_error(track, contract_pack_root=contract_pack_root)
        track_model = load_track_or_raise(track, production_source=production_source)
        roadmap = load_roadmap(roadmap_source)
        errors = audit_manifest(loaded, track=track_model, roadmap=roadmap)
        if errors:
            raise WorkflowError("Track Execution Manifest audit blockers:\n" + "\n".join(manifest_audit_blocker_lines(errors)))
        completed_milestones = {milestone.id for milestone in track_model.milestones if milestone.state == "completed"}
        current = next(
            (
                entry
                for entry in loaded.manifest.milestones
                if (entry.owning_wr or entry.future_wr_candidate) and entry.milestone_id not in completed_milestones
            ),
            None,
        )
        console.print(f"Manifest: {repo_path(loaded.path)}")
        if current is None:
            console.print(f"[green]{track}: manifest has no milestones.[/green]")
            return
        console.print(f"Current milestone: {current.milestone_id} - {current.title}")
        console.print(f"Next legal action: {current.next_legal_action}")
        truth_lines = truth_claim_summary_lines(loaded.manifest)
        if truth_lines:
            for line in truth_lines:
                console.print(line)
        console.print("Implementation authorized now: no - task production:next is read-only; Contract Pack is required for execution")
    except WorkflowError as error:
        console.print("[red]production:next failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("audit-track")
def audit_track(
    track: str = typer.Option(..., "--track"),
    full_automation: bool = typer.Option(False, "--full-automation"),
    require_lock: bool = typer.Option(False, "--require-lock"),
    allow: list[str] = typer.Option(list(FULL_TRACK_PERMISSION_SET), "--allow"),
    deny: list[str] = typer.Option(["crate_creation", "foundation_extraction"], "--deny"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    lock_source_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
) -> None:
    try:
        load_track_or_raise(track, production_source=production_source)
        load_roadmap(roadmap_source)
        loaded = load_manifest_or_raise(track, manifest_source_root=manifest_source_root)
        if full_automation:
            if set(allow) & set(deny):
                raise WorkflowError("the same permission cannot be both allowed and denied")
            if load_contract_pack(track, root=contract_pack_root) is None:
                raise WorkflowError(
                    f"{track}: full automation audit requires an Execution Contract Pack; "
                    "run task execution:compile explicitly"
                )
            execution_preflight_command(track=track, mode="full-track", allow=allow, contract_pack_root=contract_pack_root)
            if require_lock:
                lock_errors = execution_lock_errors(
                    track,
                    contract_pack_root=contract_pack_root,
                    lock_root=lock_source_root,
                    requested_permissions=set(allow),
                    run_mode="full-track",
                )
                if lock_errors:
                    raise WorkflowError("\n".join(lock_errors))
                console.print("[green]Execution Harness lock passed[/green]")
            return
        console.print(f"Manifest: {repo_path(loaded.path)}")
        planning = load_production_tracks(production_source)
        roadmap = load_roadmap(roadmap_source)
        track_model = next(candidate for candidate in planning.tracks if candidate.id == track)
        errors = audit_manifest(loaded, track=track_model, roadmap=roadmap)
        if errors:
            console.print("[yellow]Track Execution Manifest audit blockers:[/yellow]")
            for line in manifest_audit_blocker_lines(errors):
                console.print(line)
            raise WorkflowError("Track Execution Manifest audit blockers")
        console.print("[green]manifest audit passed[/green]")
    except WorkflowError as error:
        console.print("[red]production:audit-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("_commands", hidden=True)
def commands() -> None:
    console.print("plan-track expand-track complete-track-contracts lock-track run-track next audit-track")


if __name__ == "__main__":
    app()
