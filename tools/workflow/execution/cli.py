#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

import typer
from rich.console import Console

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from roadmap_state import ROADMAP_SOURCE, REPO_ROOT, WorkflowError, repo_path
from production_state import PRODUCTION_SOURCE
from track_execution_manifest import TRACK_EXECUTION_MANIFEST_ROOT

from execution.compiler import CONTRACT_PACK_ROOT, compile_contract_pack, contract_pack_path, load_contract_pack, write_contract_pack
from execution.ledger import RUN_LEDGER_ROOT, append_run_action, append_run_failure, new_run_id
from execution.locks import (
    EXECUTION_LOCK_ROOT,
    build_execution_lock,
    contract_pack_freshness_errors,
    execution_lock_errors,
    load_execution_lock,
    write_execution_lock,
)
from execution.preflight import preflight_pack
from execution.runner import run_next_action


console = Console()
app = typer.Typer(no_args_is_help=True, help="Run the clean Track Execution Harness.")


def permission_set(values: list[str]) -> set[str]:
    return {value.strip() for value in values if value.strip()}


@app.command("compile")
def compile_command(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
) -> None:
    run_id = ""
    current_action = None
    current_pre_action_digests: dict[str, str] = {}
    try:
        pack = compile_contract_pack(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_root=manifest_source_root,
        )
        path = write_contract_pack(pack, root=contract_pack_root)
        console.print("[green]Execution Contract Pack written.[/green]")
        console.print(f"Contract Pack: {repo_path(path)}")
        console.print(f"Actions: {len(pack.actions)}")
    except WorkflowError as error:
        console.print("[red]execution:compile failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("preflight")
def preflight_command(
    track: str = typer.Option(..., "--track"),
    allow: list[str] = typer.Option([], "--allow"),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
) -> None:
    try:
        pack = load_contract_pack(track, root=contract_pack_root)
        if pack is None:
            raise WorkflowError(f"{track}: missing Execution Contract Pack at {repo_path(contract_pack_path(track, root=contract_pack_root))}")
        freshness_errors = contract_pack_freshness_errors(pack)
        if freshness_errors:
            raise WorkflowError("\n".join(freshness_errors))
        errors = preflight_pack(pack, allow=permission_set(allow) if allow else None)
        if errors:
            raise WorkflowError("\n".join(errors))
        console.print("[green]execution preflight passed[/green]")
        console.print(f"Contract Pack: {repo_path(contract_pack_path(track, root=contract_pack_root))}")
        console.print(f"Actions inspected: {len(pack.actions)}")
    except WorkflowError as error:
        console.print("[red]execution:preflight failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("lock")
def lock_command(
    track: str = typer.Option(..., "--track"),
    locked_by: str = typer.Option(..., "--locked-by"),
    allow: list[str] = typer.Option([], "--allow"),
    deny: list[str] = typer.Option([], "--deny"),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
    lock_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
) -> None:
    try:
        pack = load_contract_pack(track, root=contract_pack_root)
        if pack is None:
            raise WorkflowError(f"{track}: missing Execution Contract Pack at {repo_path(contract_pack_path(track, root=contract_pack_root))}")
        freshness_errors = contract_pack_freshness_errors(pack)
        if freshness_errors:
            raise WorkflowError("\n".join(freshness_errors))
        errors = preflight_pack(pack, allow=permission_set(allow) if allow else None)
        if errors:
            raise WorkflowError("\n".join(errors))
        lock = build_execution_lock(
            track,
            locked_by=locked_by,
            contract_pack_root=contract_pack_root,
            granted_permissions=sorted(permission_set(allow)),
            denied_permissions=sorted(permission_set(deny)),
        )
        path = write_execution_lock(lock, root=lock_root)
        console.print("[green]Execution Lock written.[/green]")
        console.print(f"Execution Lock: {repo_path(path)}")
    except WorkflowError as error:
        console.print("[red]execution:lock failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("next")
def next_command(
    track: str = typer.Option(..., "--track"),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
) -> None:
    try:
        pack = load_contract_pack(track, root=contract_pack_root)
        if pack is None:
            raise WorkflowError(f"{track}: missing Execution Contract Pack at {repo_path(contract_pack_path(track, root=contract_pack_root))}")
        freshness_errors = contract_pack_freshness_errors(pack)
        if freshness_errors:
            raise WorkflowError("\n".join(freshness_errors))
        console.print(f"Contract Pack: {repo_path(contract_pack_path(track, root=contract_pack_root))}")
        if not pack.actions:
            console.print(f"[green]{track}: no remaining ActionContracts.[/green]")
            return
        action = pack.actions[0]
        console.print(f"Next action: {action.action_id}")
        console.print(f"Execution kind: {action.execution_kind}")
        console.print(f"Writer strategy: {action.writer_strategy}")
        console.print("Validation commands:")
        for command in action.validation_commands:
            console.print(f"- {command.command_id}: {' '.join(command.argv)}")
    except WorkflowError as error:
        console.print("[red]execution:next failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("run")
def run_command(
    track: str = typer.Option(..., "--track"),
    mode: str = typer.Option("single-action", "--mode"),
    allow: list[str] = typer.Option([], "--allow"),
    deny: list[str] = typer.Option([], "--deny"),
    max_actions: int = typer.Option(1, "--max-actions", min=1),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
    lock_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
    run_ledger_root: Path = typer.Option(RUN_LEDGER_ROOT),
    repo_root: Path = typer.Option(REPO_ROOT, "--repo-root"),
) -> None:
    run_id = ""
    current_action = None
    current_pre_action_digests: dict[str, str] = {}
    try:
        if mode not in {"single-action", "full-track"}:
            raise WorkflowError("--mode must be one of single-action, full-track")
        allow_set = permission_set(allow)
        deny_set = permission_set(deny)
        if not allow_set:
            raise WorkflowError("execution:run requires at least one --allow permission")
        if allow_set & deny_set:
            raise WorkflowError("the same permission cannot be both allowed and denied")

        actions_run = 0
        run_id = new_run_id(track)
        while actions_run < max_actions:
            pack = load_contract_pack(track, root=contract_pack_root)
            if pack is None:
                raise WorkflowError(f"{track}: missing Execution Contract Pack at {repo_path(contract_pack_path(track, root=contract_pack_root))}")
            freshness_errors = contract_pack_freshness_errors(pack)
            if freshness_errors:
                raise WorkflowError("\n".join(freshness_errors))
            if not pack.actions:
                console.print(f"[green]{track}: no remaining ActionContracts.[/green]")
                return
            errors = preflight_pack(pack, allow=allow_set)
            if errors:
                raise WorkflowError("\n".join(errors))
            lock_errors = execution_lock_errors(
                track,
                contract_pack_root=contract_pack_root,
                lock_root=lock_root,
                requested_permissions=allow_set,
            )
            if lock_errors:
                raise WorkflowError("\n".join(lock_errors))
            before_action = pack.actions[0].action_id
            action = pack.actions[0]
            current_action = action
            pre_action_digests = dict(pack.source_digests)
            current_pre_action_digests = pre_action_digests
            result = run_next_action(pack, lock_validated=True, repo_root=repo_root)
            actions_run += 1
            console.print("[green]Execution Harness ran one ActionContract.[/green]")
            console.print(f"Action: {result.action_id}")
            for path in result.written_paths:
                console.print(f"- {repo_path(path)}")
            for path in result.evidence_paths:
                console.print(f"Evidence: {repo_path(path)}")
            for validation in result.validation_results:
                console.print(f"- {validation}")
            console.print(f"Next: {result.next_action}")
            if mode == "single-action":
                ledger_path = append_run_action(
                    track_id=track,
                    run_id=run_id,
                    action=action,
                    result=result,
                    pre_action_digests=pre_action_digests,
                    post_action_digests=dict(pack.source_digests),
                    root=run_ledger_root,
                    stop_reason="single-action complete",
                )
                console.print(f"Run ledger: {repo_path(ledger_path)}")
                return
            next_pack = compile_contract_pack(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_root=manifest_source_root,
            )
            write_contract_pack(next_pack, root=contract_pack_root)
            ledger_path = append_run_action(
                track_id=track,
                run_id=run_id,
                action=action,
                result=result,
                pre_action_digests=pre_action_digests,
                post_action_digests=dict(next_pack.source_digests),
                root=run_ledger_root,
                stop_reason="continuing full-track run" if next_pack.actions else "track complete",
            )
            console.print(f"Run ledger: {repo_path(ledger_path)}")
            existing_lock = load_execution_lock(track, root=lock_root)
            if existing_lock is None:
                raise WorkflowError(f"{track}: execution lock disappeared during run")
            write_execution_lock(
                build_execution_lock(
                    track,
                    locked_by=existing_lock.locked_by,
                    contract_pack_root=contract_pack_root,
                    granted_permissions=list(existing_lock.granted_permissions),
                    denied_permissions=list(existing_lock.denied_permissions),
                ),
                root=lock_root,
            )
            if not next_pack.actions:
                console.print(f"[green]{track}: track actions completed.[/green]")
                return
            if next_pack.actions[0].action_id == before_action:
                raise WorkflowError(
                    f"{track}: action {before_action} completed but did not advance Contract Pack state"
                )
        raise WorkflowError(f"{track}: max actions reached before track completion")
    except WorkflowError as error:
        if run_id:
            try:
                post_digests: dict[str, str] = {}
                pack = load_contract_pack(track, root=contract_pack_root)
                if pack is not None:
                    post_digests = dict(pack.source_digests)
                ledger_path = append_run_failure(
                    track_id=track,
                    run_id=run_id,
                    action=current_action,
                    error=str(error),
                    pre_action_digests=current_pre_action_digests,
                    post_action_digests=post_digests,
                    root=run_ledger_root,
                    stop_reason="workflow error",
                )
                console.print(f"Run ledger: {repo_path(ledger_path)}")
            except Exception as ledger_error:  # noqa: BLE001 - ledger failure must not hide the original blocker.
                console.print(f"[yellow]failed to write execution failure ledger: {ledger_error}[/yellow]")
        console.print("[red]execution:run failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


if __name__ == "__main__":
    app()
