#!/usr/bin/env python3
"""
Coordinate approved parallel roadmap batches.

File: tools/workflow/parallel_batch.py
Module: parallel_batch
"""

from __future__ import annotations

import datetime as dt
import re
import subprocess
from pathlib import Path

import typer
from rich.console import Console

from roadmap_state import (
    REPO_ROOT,
    ROADMAP_SOURCE,
    BatchItem,
    BatchManifest,
    RoadmapItem,
    WorkflowError,
    git_output,
    load_batch_manifest,
    load_roadmap,
    parse_scope_selector,
    render_batch_manifest,
    repo_path,
    select_batch_candidates,
    slash_path,
    validate_batch_against_roadmap,
    validate_write_scopes,
)


DEFAULT_BATCH_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/batches"
DEFAULT_WORKTREE_ROOT = REPO_ROOT.parent / "Runenwerk-worktrees"

console = Console()
app = typer.Typer(no_args_is_help=True, help="Propose, approve, prepare, and close out roadmap batches.")


@app.command()
def propose(
    goal: str = typer.Option(..., help="Batch goal."),
    scope: str = typer.Option("<level/items>", help="Dependency level or comma-separated WR IDs."),
    out: Path | None = typer.Option(None, help="Batch manifest path."),
    batch_id: str | None = typer.Option(None, help="Stable batch id."),
    level: str | None = typer.Option(None, help="Dependency level such as L0."),
    include_discovery: bool = typer.Option(False, help="Include B3-B4 discovery work."),
    allow_scope_conflicts: bool = typer.Option(False, help="Allow overlapping write scopes."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
) -> None:
    roadmap = load_roadmap(source)
    level_from_scope, item_ids = parse_scope_selector(scope)
    selected = select_batch_candidates(
        roadmap,
        level=level or level_from_scope,
        item_ids=item_ids,
        include_discovery=include_discovery,
    )
    if not selected:
        raise WorkflowError("no eligible roadmap items matched the requested batch scope")

    conflicts = validate_write_scopes(selected)
    if conflicts and not allow_scope_conflicts:
        for conflict in conflicts:
            console.print(f"[red]write-scope conflict:[/red] {conflict}")
        raise typer.Exit(1)

    resolved_id = batch_id or default_batch_id(goal)
    batch_path = out or DEFAULT_BATCH_ROOT / resolved_id / "batch.toml"
    manifest = build_manifest(resolved_id, goal, selected, batch_path.parent)
    batch_path.parent.mkdir(parents=True, exist_ok=True)
    batch_path.write_text(render_batch_manifest(manifest), encoding="utf-8", newline="\n")
    console.print(f"[green]wrote batch proposal:[/green] {repo_path(batch_path)}")


@app.command()
def approve(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
) -> None:
    manifest = load_batch_manifest(batch)
    roadmap = load_roadmap(source)
    validation_errors = validate_batch_against_roadmap(manifest, roadmap)
    if validation_errors:
        console.print("[red]batch approval failed[/red]")
        for error in validation_errors:
            console.print(f"- {error}")
        raise typer.Exit(1)
    updated = manifest.model_copy(update={"approval_state": "approved"})
    batch.write_text(render_batch_manifest(updated), encoding="utf-8", newline="\n")
    console.print(f"[green]approved batch:[/green] {repo_path(batch)}")


@app.command("worker-prompt")
def worker_prompt(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    item: str = typer.Option(..., help="Roadmap item ID."),
    write: bool = typer.Option(False, help="Write to the manifest prompt_path instead of stdout."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
) -> None:
    roadmap = load_roadmap(source)
    manifest = load_batch_manifest(batch)
    batch_item = next((candidate for candidate in manifest.items if candidate.id == item), None)
    if batch_item is None:
        raise WorkflowError(f"{item} is not part of batch {manifest.id}")
    roadmap_item = roadmap.by_id.get(item)
    if roadmap_item is None:
        raise WorkflowError(f"{item} is not present in roadmap source")
    prompt = render_worker_prompt(manifest, roadmap_item, batch_item)
    if write:
        target = REPO_ROOT / batch_item.prompt_path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(prompt, encoding="utf-8", newline="\n")
        console.print(f"[green]wrote worker prompt:[/green] {repo_path(target)}")
    else:
        console.print(prompt)


@app.command()
def prepare(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    root: Path = typer.Option(DEFAULT_WORKTREE_ROOT, help="Worktree root directory."),
    allow_unapproved: bool = typer.Option(False, help="Allow preparing a proposed batch."),
    dry_run: bool = typer.Option(False, help="Print worktree actions without executing git."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
) -> None:
    manifest = load_batch_manifest(batch)
    if manifest.approval_state != "approved" and not allow_unapproved:
        raise WorkflowError("batch must be approved before preparing worktrees")

    roadmap = load_roadmap(source)
    validation_errors = validate_batch_against_roadmap(manifest, roadmap)
    if validation_errors:
        console.print("[red]batch prepare failed[/red]")
        for error in validation_errors:
            console.print(f"- {error}")
        raise typer.Exit(1)
    updated_items: list[BatchItem] = []
    for item in manifest.items:
        prompt_target = REPO_ROOT / item.prompt_path
        if not prompt_target.exists():
            roadmap_item = roadmap.by_id.get(item.id)
            if roadmap_item is None:
                raise WorkflowError(f"{item.id} is not present in roadmap source")
            if dry_run:
                console.print(f"would write worker prompt: {repo_path(prompt_target)}")
            else:
                prompt_target.parent.mkdir(parents=True, exist_ok=True)
                prompt_target.write_text(render_worker_prompt(manifest, roadmap_item, item), encoding="utf-8", newline="\n")
                console.print(f"[green]wrote worker prompt:[/green] {repo_path(prompt_target)}")

        worktree = root / manifest.id / item.id
        if dry_run:
            console.print(" ".join(worktree_add_command(item.branch, worktree, manifest.base_sha)))
        elif worktree.exists() and any(worktree.iterdir()):
            console.print(f"[yellow]worktree exists:[/yellow] {worktree}")
        else:
            subprocess.run(worktree_add_command(item.branch, worktree, manifest.base_sha), cwd=REPO_ROOT, check=True)
        updated_items.append(item.model_copy(update={"worktree": slash_path(worktree), "status": "approved"}))

    updated = manifest.model_copy(
        update={
            "execution_mode": "worktree",
            "integration_risk": "isolated worktrees",
            "items": updated_items,
        }
    )
    if not dry_run:
        batch.write_text(render_batch_manifest(updated), encoding="utf-8", newline="\n")
    console.print(f"[green]prepared batch:[/green] {manifest.id}")


def worktree_add_command(branch: str, worktree: Path, base_sha: str) -> list[str]:
    existing_branch_sha = git_output(["git", "rev-parse", "--verify", branch])
    command = ["git", "-c", "core.longpaths=true", "worktree", "add"]
    if existing_branch_sha:
        if existing_branch_sha != base_sha:
            raise WorkflowError(
                f"branch {branch} already exists at {existing_branch_sha}, expected batch base {base_sha}"
            )
        return [*command, str(worktree), branch]
    return [*command, "-b", branch, str(worktree), base_sha]


@app.command()
def closeout(batch: Path = typer.Option(..., help="Batch manifest path."), write: bool = typer.Option(False, help="Write batch.md report.")) -> None:
    manifest = load_batch_manifest(batch)
    missing_prompts = [item.prompt_path for item in manifest.items if item.prompt_path and not (REPO_ROOT / item.prompt_path).exists()]
    if missing_prompts:
        console.print("[red]batch closeout is incomplete[/red]")
        for prompt in missing_prompts:
            console.print(f"- missing worker prompt artifact: {prompt}")
        raise typer.Exit(1)

    report = render_batch_report(manifest)
    if write:
        report_path = batch.parent / "batch.md"
        report_path.write_text(report, encoding="utf-8", newline="\n")
        console.print(f"[green]wrote batch report:[/green] {repo_path(report_path)}")
    else:
        console.print(report)


def build_manifest(batch_id: str, goal: str, items: list[RoadmapItem], batch_dir: Path) -> BatchManifest:
    base_branch = git_output(["git", "branch", "--show-current"]) or "unknown"
    base_sha = git_output(["git", "rev-parse", "HEAD"]) or "unknown"
    batch_items: list[BatchItem] = []
    for item in items:
        batch_items.append(
            BatchItem(
                id=item.id,
                title=item.title,
                lane=item.lane,
                dependency_level=item.dependency_level,
                gate=item.gate,
                score=item.score,
                branch=f"codex/{batch_id}-{item.id.lower()}",
                prompt_path=slash_path(batch_dir / "prompts" / f"{item.id.lower()}.md"),
                write_scopes=item.write_scopes,
                validations=item.validations,
            )
        )
    return BatchManifest(
        id=batch_id,
        goal=goal,
        approval_state="proposed",
        base_branch=base_branch,
        base_sha=base_sha,
        execution_mode="worktree",
        integration_risk="isolated worktrees pending",
        items=batch_items,
    )


def render_worker_prompt(manifest: BatchManifest, item: RoadmapItem, batch_item: BatchItem) -> str:
    lines = [
        "---",
        f"title: Worker Prompt {item.id}",
        f"description: Generated worker prompt for batch {manifest.id}.",
        "status: active",
        "owner: workspace",
        "layer: workspace",
        "canonical: false",
        f"last_reviewed: {dt.date.today().isoformat()}",
        "---",
        "",
        f"# Runenwerk Parallel Worker: {item.id}",
        "",
        f"Batch: {manifest.id}",
        f"Goal: {manifest.goal}",
        "",
        "## Task",
        "",
        item.title,
        "",
        "## Required Reading",
        "",
        "- AGENTS.md",
        "- AI_GUIDE.md",
        "- docs-site/src/content/docs/workspace/parallel-roadmap-batch-automation.md",
        "- docs-site/src/content/docs/workspace/architecture-governance-review.md",
        "- docs-site/src/content/docs/workspace/roadmap-items.yaml",
        "",
        "## Governance",
        "",
        f"- DDD owner: {item.ddd_owner}",
        f"- ADR requirement: {item.adr_requirement}",
        f"- Fitness function requirement: {item.fitness_function_requirement}",
        f"- Ownership mode: {item.ownership_mode}",
        "",
        "## Write Scope",
        "",
    ]
    lines.extend(f"- {scope}" for scope in batch_item.write_scopes)
    lines.extend(["", "## Validation", ""])
    lines.extend(f"- {validation}" for validation in batch_item.validations)
    lines.extend(
        [
            "",
            "## Stop Conditions",
            "",
            "- Ownership or dependency direction contradicts the roadmap source.",
            "- The implementation needs a write scope outside the batch manifest.",
            "- The work requires an unaccepted ADR or design update.",
            "- Validation fails for a reason that changes the roadmap decision.",
            "",
        ]
    )
    return "\n".join(lines)


def render_batch_report(manifest: BatchManifest) -> str:
    lines = [
        "---",
        f"title: Batch {manifest.id}",
        "description: Parallel roadmap batch closeout report.",
        "status: active",
        "owner: workspace",
        "layer: workspace",
        "canonical: true",
        f"last_reviewed: {dt.date.today().isoformat()}",
        "---",
        "",
        f"# Batch {manifest.id}",
        "",
        f"Goal: {manifest.goal}",
        f"Approval state: {manifest.approval_state}",
        f"Integration status: {manifest.integration_status}",
        f"Closeout status: {manifest.closeout_status}",
        "",
        "## Validation Results",
        "",
    ]
    if manifest.validation_results:
        lines.extend(f"- {result}" for result in manifest.validation_results)
    else:
        lines.append("- Not recorded.")
    lines.extend(
        [
            "",
            "## Roadmap Evidence Updates",
            "",
        ]
    )
    if manifest.roadmap_evidence_updates:
        lines.extend(f"- {update}" for update in manifest.roadmap_evidence_updates)
    else:
        lines.append("- Not recorded.")
    lines.extend(
        [
            "",
            "## Tooling Hardening",
            "",
        ]
    )
    if manifest.tooling_hardening:
        lines.extend(f"- {item}" for item in manifest.tooling_hardening)
    else:
        lines.append("- Not recorded.")
    lines.extend(
        [
            "",
            "## Items",
            "",
        ]
    )
    for item in manifest.items:
        lines.extend(
            [
                f"### {item.id} {item.title}",
                "",
                f"- Branch: `{item.branch}`",
                f"- Worktree: `{item.worktree or 'not prepared'}`",
                f"- Status: `{item.status}`",
                f"- Write scopes: {', '.join(f'`{scope}`' for scope in item.write_scopes)}",
                "",
            ]
        )
    return "\n".join(lines)


def default_batch_id(goal: str) -> str:
    today = dt.date.today().isoformat()
    slug = re.sub(r"[^a-z0-9]+", "-", goal.lower()).strip("-")[:40] or "roadmap-batch"
    return f"{today}-{slug}"


if __name__ == "__main__":
    app()
