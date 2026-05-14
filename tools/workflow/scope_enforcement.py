#!/usr/bin/env python3
"""
Validate batch worker diffs against approved write scopes.

File: tools/workflow/scope_enforcement.py
Module: scope_enforcement
"""

from __future__ import annotations

from pathlib import Path

import typer
from rich.console import Console

from roadmap_state import (
    REPO_ROOT,
    BatchItem,
    changed_files_for_worktree,
    load_batch_manifest,
    validate_changed_paths,
)


console = Console()
app = typer.Typer(no_args_is_help=True, help="Enforce batch write scopes.")


@app.command()
def check(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    item: str | None = typer.Option(None, help="Limit validation to one WR item."),
    changed_file: list[str] = typer.Option(None, help="Explicit changed file for dry/test validation."),
) -> None:
    manifest = load_batch_manifest(batch)
    selected = [candidate for candidate in manifest.items if item is None or candidate.id == item]
    if not selected:
        console.print("[red]no matching batch items[/red]")
        raise typer.Exit(1)

    violations: list[str] = []
    for candidate in selected:
        paths = list(changed_file or changed_paths(candidate, manifest.base_sha))
        item_violations = validate_changed_paths(paths, candidate.write_scopes)
        violations.extend(f"{candidate.id}: {path}" for path in item_violations)

    if violations:
        console.print("[red]scope enforcement failed[/red]")
        for violation in violations:
            console.print(f"- {violation}")
        raise typer.Exit(1)
    console.print("[green]scope enforcement passed[/green]")


@app.command("noop", hidden=True)
def noop() -> None:
    """Hidden command that keeps Typer in multi-command mode."""


def changed_paths(item: BatchItem, base_sha: str) -> list[str]:
    if not item.worktree:
        return []
    worktree = Path(item.worktree)
    if not worktree.is_absolute():
        worktree = REPO_ROOT / worktree
    return changed_files_for_worktree(worktree, base_sha)


if __name__ == "__main__":
    app()
