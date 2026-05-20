#!/usr/bin/env python3
"""
Check repository branch, worktree, and batch-manifest hygiene.

File: tools/workflow/repo_hygiene.py
Module: repo_hygiene
"""

from __future__ import annotations

import subprocess
from pathlib import Path

import typer
from rich.console import Console

from roadmap_state import REPO_ROOT, BatchManifest, load_batch_manifest, repo_path, slash_path


DEFAULT_BATCH_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/batches"

console = Console()
app = typer.Typer(no_args_is_help=True, help="Check local repository hygiene.")


@app.callback()
def main() -> None:
    """Repository hygiene commands."""


@app.command()
def check(batch_root: Path = typer.Option(DEFAULT_BATCH_ROOT, help="Batch manifest root.")) -> None:
    errors, warnings = repo_hygiene_findings(batch_root)
    for warning in warnings:
        console.print(f"[yellow]warning:[/yellow] {warning}")
    if errors:
        console.print("[red]repository hygiene failed[/red]")
        for error in errors:
            console.print(f"- {error}")
        raise typer.Exit(1)
    console.print("[green]repository hygiene passed[/green]")


def repo_hygiene_findings(batch_root: Path = DEFAULT_BATCH_ROOT) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []

    status = git_stdout(["status", "--porcelain=v1"])
    if status.strip():
        errors.append("working tree is dirty")

    registered_worktrees = worktree_paths()
    expected_extra_worktrees = expected_active_batch_worktrees(batch_root)
    for worktree in registered_worktrees:
        if same_path(worktree, REPO_ROOT):
            continue
        if worktree not in expected_extra_worktrees:
            errors.append(f"unexpected registered worktree: {slash_path(worktree)}")

    current_branch = git_stdout(["branch", "--show-current"]).strip()
    for branch in local_branches(merged_only=True):
        if branch == current_branch or branch == "main":
            continue
        if branch.startswith("codex/"):
            errors.append(f"merged codex branch should be deleted: {branch}")

    for branch in local_branches():
        upstream = git_stdout(["rev-parse", "--abbrev-ref", f"{branch}@{{upstream}}"], check=False).strip()
        if not upstream:
            continue
        ahead_behind = git_stdout(["rev-list", "--left-right", "--count", f"{upstream}...{branch}"], check=False).split()
        if len(ahead_behind) != 2:
            continue
        behind, ahead = (int(ahead_behind[0]), int(ahead_behind[1]))
        if behind:
            errors.append(f"{branch} is behind {upstream} by {behind} commit(s)")
        if ahead:
            warnings.append(f"{branch} is ahead of {upstream} by {ahead} commit(s)")

    errors.extend(batch_manifest_hygiene_errors(batch_root))
    return errors, warnings


def batch_manifest_hygiene_errors(batch_root: Path = DEFAULT_BATCH_ROOT) -> list[str]:
    errors: list[str] = []
    for path in sorted(batch_root.glob("*/batch.toml")):
        manifest = load_batch_manifest(path)
        errors.extend(batch_manifest_errors(path, manifest))
    return errors


def batch_manifest_errors(path: Path, manifest: BatchManifest) -> list[str]:
    errors: list[str] = []
    finalized = manifest.integration_status in {"merged", "integrated"} and manifest.closeout_status == "completed"
    for item in manifest.items:
        if item.prompt_path and not (REPO_ROOT / item.prompt_path).exists():
            errors.append(f"{repo_path(path)}:{item.id}: prompt_path does not exist: {item.prompt_path}")
        if item.worktree and item.worktree_cleanup:
            errors.append(f"{repo_path(path)}:{item.id}: records both worktree and cleanup state")
        if item.worktree:
            worktree = Path(item.worktree)
            if not worktree.is_absolute():
                worktree = REPO_ROOT / worktree
            if finalized:
                errors.append(f"{repo_path(path)}:{item.id}: finalized batch still records active worktree {slash_path(worktree)}")
            elif not worktree.exists():
                errors.append(f"{repo_path(path)}:{item.id}: recorded worktree does not exist: {slash_path(worktree)}")
        if finalized and not item.worktree and not item.worktree_cleanup:
            errors.append(f"{repo_path(path)}:{item.id}: finalized batch does not record worktree cleanup state")
    return errors


def expected_active_batch_worktrees(batch_root: Path) -> set[Path]:
    expected: set[Path] = set()
    for path in sorted(batch_root.glob("*/batch.toml")):
        manifest = load_batch_manifest(path)
        if manifest.integration_status in {"merged", "integrated"} and manifest.closeout_status == "completed":
            continue
        for item in manifest.items:
            if item.worktree:
                worktree = Path(item.worktree)
                if not worktree.is_absolute():
                    worktree = REPO_ROOT / worktree
                expected.add(worktree.resolve())
    return expected


def worktree_paths() -> list[Path]:
    paths: list[Path] = []
    for line in git_stdout(["worktree", "list", "--porcelain"]).splitlines():
        if line.startswith("worktree "):
            paths.append(Path(line.removeprefix("worktree ")).resolve())
    return paths


def local_branches(*, merged_only: bool = False) -> list[str]:
    command = ["branch", "--format=%(refname:short)"]
    if merged_only:
        command.append("--merged")
    return [line.strip() for line in git_stdout(command).splitlines() if line.strip()]


def git_stdout(command: list[str], *, check: bool = True) -> str:
    completed = subprocess.run(
        ["git", *command],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=check,
    )
    if completed.returncode != 0:
        return ""
    return completed.stdout


def same_path(left: Path, right: Path) -> bool:
    try:
        return left.resolve() == right.resolve()
    except OSError:
        return left == right


if __name__ == "__main__":
    app()
