#!/usr/bin/env python3
"""Legacy Track Execution Manifest compatibility shim.

File: tools/workflow/track_execution_manifest.py
Module: track_execution_manifest

Executable production-track authority now lives in:

- tools/workflow/track_sources/
- tools/workflow/execution/
- tools/workflow/production_track_cli.py

This module intentionally does not expose the old runner implementation. Keep it
small so accidental imports or direct script execution fail closed instead of
reviving loose manifest execution.
"""

from __future__ import annotations

import typer
from rich.console import Console


console = Console()
app = typer.Typer(no_args_is_help=True, help="Legacy compatibility shim; execution is disabled.")

COMMANDS = (
    "plan-track",
    "expand-track",
    "complete-track-contracts",
    "lock-track",
    "run-track",
    "next",
    "audit-track",
)


def legacy_error() -> None:
    raise typer.BadParameter(
        "track_execution_manifest.py is legacy compatibility only. "
        "Use tools/workflow/production_track_cli.py, task production:*, "
        "or task execution:* commands."
    )


@app.command("plan-track")
def plan_track() -> None:
    legacy_error()


@app.command("expand-track")
def expand_track() -> None:
    legacy_error()


@app.command("complete-track-contracts")
def complete_track_contracts() -> None:
    legacy_error()


@app.command("lock-track")
def lock_track() -> None:
    legacy_error()


@app.command("run-track")
def run_track() -> None:
    legacy_error()


@app.command("next")
def next_action() -> None:
    legacy_error()


@app.command("audit-track")
def audit_track() -> None:
    legacy_error()


@app.command("_commands", hidden=True)
def commands() -> None:
    console.print(" ".join(COMMANDS))


if __name__ == "__main__":
    raise SystemExit(
        "track_execution_manifest.py is legacy compatibility only; "
        "use tools/workflow/production_track_cli.py, task production:*, "
        "or task execution:* commands."
    )
