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
import sys
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
    changed_files_for_worktree,
    git_output,
    load_batch_manifest,
    load_roadmap,
    normalize_repo_path,
    path_within_scope,
    parse_scope_selector,
    render_batch_manifest,
    repo_path,
    select_batch_candidates,
    slash_path,
    validate_batch_against_roadmap,
    validate_changed_paths,
    validate_write_scopes,
)
from prompt_doctrine import quality_doctrine_block


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
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    roadmap = load_roadmap(source)
    level_from_scope, item_ids = parse_scope_selector(scope)
    try:
        selected = select_batch_candidates(
            roadmap,
            level=level or level_from_scope,
            item_ids=item_ids,
            include_discovery=include_discovery,
        )
    except WorkflowError as error:
        console.print("[red]batch proposal failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error
    if not selected:
        console.print("[red]batch proposal failed[/red]")
        console.print("- no eligible roadmap items matched the requested batch scope")
        raise typer.Exit(1)

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
def kickoff(
    next: bool = typer.Option(False, "--next", help="Select all current roadmap candidates."),
    scope: str = typer.Option("<level/items>", help="Dependency level or comma-separated WR IDs."),
    goal: str | None = typer.Option(None, help="Batch goal. Defaults to selected roadmap IDs."),
    out: Path | None = typer.Option(None, help="Batch manifest path."),
    batch_id: str | None = typer.Option(None, help="Stable batch id."),
    approve: bool = typer.Option(False, help="Approve the generated batch immediately."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    if not next and scope == "<level/items>":
        console.print("[red]batch kickoff failed[/red]")
        console.print("- use --next or provide --scope")
        raise typer.Exit(1)

    roadmap = load_roadmap(source)
    level_from_scope, item_ids = parse_scope_selector(scope)
    try:
        selected = select_batch_candidates(
            roadmap,
            level=None if next else level_from_scope,
            item_ids=() if next else item_ids,
        )
    except WorkflowError as error:
        console.print("[red]batch kickoff failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error
    if not selected:
        console.print("[red]batch kickoff failed[/red]")
        console.print("- no current_candidate items are eligible for implementation")
        raise typer.Exit(1)

    conflicts = validate_write_scopes(selected)
    if conflicts:
        console.print("[red]batch kickoff failed[/red]")
        for conflict in conflicts:
            console.print(f"- write-scope conflict: {conflict}")
        raise typer.Exit(1)

    resolved_goal = goal or default_kickoff_goal(selected)
    resolved_id = batch_id or default_batch_id(resolved_goal)
    batch_path = out or DEFAULT_BATCH_ROOT / resolved_id / "batch.toml"
    manifest = build_manifest(resolved_id, resolved_goal, selected, batch_path.parent)
    if approve:
        manifest = manifest.model_copy(update={"approval_state": "approved"})
    batch_path.parent.mkdir(parents=True, exist_ok=True)
    batch_path.write_text(render_batch_manifest(manifest), encoding="utf-8", newline="\n")

    console.print(f"[green]wrote batch kickoff:[/green] {repo_path(batch_path)}")
    print_kickoff_next_steps(batch_path, manifest)


@app.command("continue")
def continue_batch(
    batch: Path = typer.Option(..., help="Finalized batch manifest path."),
    out: Path | None = typer.Option(None, help="Continuation batch manifest path."),
    batch_id: str | None = typer.Option(None, help="Stable continuation batch id."),
    goal: str | None = typer.Option(None, help="Continuation batch goal."),
    include_all_current: bool = typer.Option(
        False,
        help="Continue with all currently eligible roadmap candidates, not only still-current items from the source batch.",
    ),
    force: bool = typer.Option(False, help="Allow overwriting an existing --out manifest."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    manifest = load_batch_manifest(batch)
    roadmap = load_roadmap(source)
    try:
        selected = continuation_items_for_manifest(manifest, roadmap, include_all_current=include_all_current)
    except WorkflowError as error:
        console.print("[red]batch continuation failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error

    conflicts = validate_write_scopes(selected)
    if conflicts:
        console.print("[red]batch continuation failed[/red]")
        for conflict in conflicts:
            console.print(f"- write-scope conflict: {conflict}")
        raise typer.Exit(1)

    resolved_goal = goal or default_continuation_goal(manifest, selected)
    resolved_id = batch_id or default_batch_id(resolved_goal)
    if out is None:
        resolved_id, batch_path = next_available_batch_path(resolved_id)
    else:
        batch_path = out
    if batch_path.exists() and not force:
        console.print("[red]batch continuation failed[/red]")
        console.print(f"- output manifest already exists: {repo_path(batch_path)}")
        raise typer.Exit(1)

    continuation = build_manifest(resolved_id, resolved_goal, selected, batch_path.parent)
    batch_path.parent.mkdir(parents=True, exist_ok=True)
    batch_path.write_text(render_batch_manifest(continuation), encoding="utf-8", newline="\n")

    console.print(f"[green]wrote batch continuation:[/green] {repo_path(batch_path)}")
    print_kickoff_next_steps(batch_path, continuation)


@app.command()
def approve(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
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
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    roadmap = load_roadmap(source)
    manifest = load_batch_manifest(batch)
    batch_item = next((candidate for candidate in manifest.items if candidate.id == item), None)
    if batch_item is None:
        raise WorkflowError(f"{item} is not part of batch {manifest.id}")
    roadmap_item = roadmap.by_id.get(item)
    if roadmap_item is None:
        raise WorkflowError(f"{item} is not present in combined roadmap sources")
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
    flat_worktrees: bool = typer.Option(False, help="Place worktrees directly under --root by item id."),
    allow_unapproved: bool = typer.Option(False, help="Allow preparing a proposed batch."),
    dry_run: bool = typer.Option(False, help="Print worktree actions without executing git."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
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
                raise WorkflowError(f"{item.id} is not present in combined roadmap sources")
            if dry_run:
                console.print(f"would write worker prompt: {repo_path(prompt_target)}")
            else:
                prompt_target.parent.mkdir(parents=True, exist_ok=True)
                prompt_target.write_text(render_worker_prompt(manifest, roadmap_item, item), encoding="utf-8", newline="\n")
                console.print(f"[green]wrote worker prompt:[/green] {repo_path(prompt_target)}")

        worktree = worktree_path_for_item(root, manifest, item, flat_worktrees=flat_worktrees)
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


def worktree_path_for_item(root: Path, manifest: BatchManifest, item: BatchItem, *, flat_worktrees: bool) -> Path:
    if flat_worktrees:
        return root / item.id
    return root / manifest.id / item.id


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
def validate(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    item: str | None = typer.Option(None, help="Limit worker validation to one WR item."),
    write: bool = typer.Option(False, help="Record the host validation result in batch.toml."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    manifest = load_batch_manifest(batch)
    roadmap = load_roadmap(source)
    selected: list[BatchItem] = []
    try:
        selected, output = run_official_batch_validation(manifest, roadmap, item)
    except WorkflowError as error:
        console.print("[red]batch validation failed before host validation[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1)
    except subprocess.CalledProcessError as error:
        if error.stdout:
            print(error.stdout, end="")
        if write and selected:
            write_validation_result(batch, manifest, "failed", validation_commands_for_items(selected))
        raise typer.Exit(error.returncode or 1) from error

    commands = validation_commands_for_items(selected)
    if output:
        emit_tool_output(output)
    if write:
        write_validation_result(batch, manifest, "passed", commands)
    console.print("[green]batch validation passed[/green]")


@app.command("refresh-base")
def refresh_base(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    base: str = typer.Option("main", help="Base branch or ref to refresh from."),
    clear_worktrees: bool = typer.Option(True, help="Clear recorded worker worktree paths in the manifest."),
    recreate_worktrees: bool = typer.Option(False, help="Remove existing worker worktrees and branches after safety checks."),
    discard_stale_worktrees: bool = typer.Option(
        False,
        "--discard-stale-worktrees",
        help="Allow removing dirty out-of-scope worker worktrees when recreating stale worktrees.",
    ),
) -> None:
    manifest = load_batch_manifest(batch)
    try:
        updated = refresh_base_manifest(
            manifest,
            base=base,
            clear_worktrees=clear_worktrees,
            recreate_worktrees=recreate_worktrees,
            discard_stale_worktrees=discard_stale_worktrees,
        )
    except WorkflowError as error:
        console.print("[red]batch base refresh is blocked[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error

    batch.write_text(render_batch_manifest(updated), encoding="utf-8", newline="\n")
    console.print(f"[green]refreshed batch base:[/green] {updated.base_branch}@{updated.base_sha}")


def refresh_base_manifest(
    manifest: BatchManifest,
    *,
    base: str,
    clear_worktrees: bool = True,
    recreate_worktrees: bool = False,
    discard_stale_worktrees: bool = False,
) -> BatchManifest:
    if manifest.integration_status != "not_started":
        raise WorkflowError(f"integration_status is {manifest.integration_status!r}, expected 'not_started'")

    if discard_stale_worktrees and not recreate_worktrees:
        raise WorkflowError("--discard-stale-worktrees requires --recreate-worktrees")

    all_changes, in_scope_changes, inspection_errors = worker_change_groups_for_manifest(manifest)
    if inspection_errors:
        raise WorkflowError("\n".join(inspection_errors))
    if in_scope_changes:
        raise WorkflowError("dirty in-scope worker changes block base refresh:\n" + "\n".join(in_scope_changes))
    if all_changes and not discard_stale_worktrees:
        raise WorkflowError(
            "dirty worker worktree changes block base refresh; rerun with "
            "--discard-stale-worktrees only for stale out-of-scope dirt:\n" + "\n".join(all_changes)
        )

    base_sha = git_output(["git", "rev-parse", base])
    if not base_sha:
        raise WorkflowError(f"could not resolve base ref {base!r}")

    if recreate_worktrees:
        remove_worker_worktrees_and_branches(manifest)
        clear_worktrees = True

    items = manifest.items
    if clear_worktrees:
        items = [candidate.model_copy(update={"worktree": "", "status": "approved"}) for candidate in manifest.items]

    return manifest.model_copy(
        update={
            "base_branch": base,
            "base_sha": base_sha,
            "items": items,
            "integration_risk": "isolated worktrees; base refreshed, prepare required",
        }
    )

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


@app.command()
def finalize(
    batch: Path = typer.Option(..., help="Batch manifest path."),
    target: str = typer.Option("main", help="Integration target branch or ref."),
    write: bool = typer.Option(False, help="Write batch.md after finalization."),
    cleanup: bool = typer.Option(True, "--cleanup/--no-cleanup", help="Remove integrated worker worktrees and branches."),
    keep_branches: bool = typer.Option(False, help="Preserve worker branches after worktree cleanup."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    manifest = load_batch_manifest(batch)
    roadmap = load_roadmap(source)
    before = repo_refs_snapshot()
    try:
        updated = finalize_batch_manifest(
            manifest,
            roadmap,
            target=target,
            cleanup=cleanup,
            keep_branches=keep_branches,
        )
    except WorkflowError as error:
        console.print("[red]batch finalization failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error

    batch.write_text(render_batch_manifest(updated), encoding="utf-8", newline="\n")
    if write:
        report_path = batch.parent / "batch.md"
        report_path.write_text(render_batch_report(updated), encoding="utf-8", newline="\n")
        console.print(f"[green]wrote batch report:[/green] {repo_path(report_path)}")
    console.print("[green]batch finalized[/green]")
    console.print("[bold]Before cleanup[/bold]")
    console.print(before or "no refs reported")
    console.print("[bold]After cleanup[/bold]")
    console.print(repo_refs_snapshot() or "no refs reported")


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
                prompt_path=repo_path(batch_dir / "prompts" / f"{item.id.lower()}.md"),
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
        "## Quality Doctrine",
        "",
        quality_doctrine_block(),
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
    ]
    if item.decision_gates:
        lines.extend(["Decision gates:", ""])
        lines.extend(
            f"- {gate.kind} `{gate.path}` must have status `{gate.required_status}` before {gate.applies_to}: {gate.reason}"
            for gate in item.decision_gates
        )
        lines.append("")
    lines.extend(["## Write Scope", ""])
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
        f"Integrated into: {integrated_into_label(manifest)}",
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
                f"- Worktree: `{worktree_report_value(item)}`",
                f"- Status: `{item.status}`",
                f"- Roadmap outcome: `{item.roadmap_outcome}`",
                f"- Write scopes: {', '.join(f'`{scope}`' for scope in item.write_scopes)}",
                "",
            ]
        )
    return "\n".join(lines)


def integrated_into_label(manifest: BatchManifest) -> str:
    if manifest.integrated_target and manifest.integrated_sha:
        return f"{manifest.integrated_target}@{manifest.integrated_sha}"
    if manifest.integrated_target:
        return manifest.integrated_target
    return "not recorded"


def worktree_report_value(item: BatchItem) -> str:
    if item.worktree_cleanup:
        return item.worktree_cleanup
    return item.worktree or "not prepared"


def default_batch_id(goal: str) -> str:
    today = dt.date.today().isoformat()
    base_slug = re.sub(r"[^a-z0-9]+", "-", goal.lower()).strip("-") or "roadmap-batch"
    item_suffix = "-".join(dict.fromkeys(re.findall(r"\bwr-\d+\b", goal.lower())))
    slug = base_slug[:40].strip("-") or "roadmap-batch"
    if item_suffix and item_suffix not in slug:
        if slug.endswith("wr"):
            slug = slug[:-2].strip("-") or "roadmap-batch"
        slug = f"{slug}-{item_suffix}"
    return f"{today}-{slug}"


def default_kickoff_goal(items: list[RoadmapItem]) -> str:
    ids = ", ".join(item.id for item in items)
    if len(items) == 1:
        return f"Next current-candidate roadmap batch: {items[0].id} {items[0].title}"
    return f"Next current-candidate roadmap batch: {ids}"


def default_continuation_goal(manifest: BatchManifest, items: list[RoadmapItem]) -> str:
    ids = ", ".join(item.id for item in items)
    return f"Continue roadmap batch after {manifest.id}: {ids}"


def next_available_batch_path(batch_id: str) -> tuple[str, Path]:
    candidate_id = batch_id
    candidate_path = DEFAULT_BATCH_ROOT / candidate_id / "batch.toml"
    if not candidate_path.exists():
        return candidate_id, candidate_path
    for suffix in range(2, 100):
        candidate_id = f"{batch_id}-{suffix}"
        candidate_path = DEFAULT_BATCH_ROOT / candidate_id / "batch.toml"
        if not candidate_path.exists():
            return candidate_id, candidate_path
    raise WorkflowError(f"could not find available batch path for id {batch_id!r}")


def continuation_items_for_manifest(
    manifest: BatchManifest,
    roadmap,
    *,
    include_all_current: bool = False,
) -> list[RoadmapItem]:
    if manifest.integration_status != "merged" or manifest.closeout_status != "completed":
        raise WorkflowError("batch must be finalized before a continuation can be proposed")
    if include_all_current:
        selected = select_batch_candidates(roadmap)
    else:
        continued_ids = tuple(
            item.id
            for item in manifest.items
            if item.roadmap_outcome == "slice_landed_item_still_current"
        )
        if not continued_ids:
            raise WorkflowError("no integrated still-current roadmap items are available for continuation")
        selected = select_batch_candidates(roadmap, item_ids=continued_ids)
    if not selected:
        raise WorkflowError("no eligible roadmap items are available for continuation")
    return selected


def print_kickoff_next_steps(batch_path: Path, manifest: BatchManifest) -> None:
    console.print("")
    console.print("[bold]Next commands[/bold]")
    for line in kickoff_next_step_lines(batch_path, manifest):
        console.print(line, soft_wrap=True)


def kickoff_next_step_lines(batch_path: Path, manifest: BatchManifest) -> list[str]:
    batch_arg = repo_path(batch_path)
    lines: list[str] = []
    if manifest.approval_state != "approved":
        lines.append(f"task batch:approve -- --batch {batch_arg}")
    lines.append(f"task batch:prepare -- --batch {batch_arg}")
    lines.append(f"task batch:validate -- --batch {batch_arg}")
    for item in manifest.items:
        lines.append(f"task batch:worker-prompt -- --batch {batch_arg} --item {item.id}")
    lines.append(f"task batch:scope-check -- --batch {batch_arg}")
    lines.append(f"task batch:finalize -- --batch {batch_arg} --target main --write --cleanup")
    return lines


def batch_execution_state(
    manifest: BatchManifest,
    roadmap,
    item_id: str | None = None,
) -> tuple[list[BatchItem], list[str]]:
    errors = validate_batch_against_roadmap(manifest, roadmap)
    if manifest.approval_state != "approved":
        errors.append("batch must be approved before validation")

    selected = selected_manifest_items(manifest, item_id)
    if item_id and not selected:
        errors.append(f"{item_id}: not present in batch {manifest.id}")

    for candidate in selected:
        try:
            paths = changed_paths_for_item(candidate, manifest.base_sha)
        except (OSError, subprocess.CalledProcessError) as error:
            errors.append(f"{candidate.id}: cannot inspect worker changes: {error}")
            continue
        for path in validate_changed_paths(paths, candidate.write_scopes):
            errors.append(f"{candidate.id}: changed path outside approved scope: {path}")
    return selected, errors


def run_official_batch_validation(
    manifest: BatchManifest,
    roadmap,
    item_id: str | None = None,
    command_runner=None,
) -> tuple[list[BatchItem], str]:
    selected, errors = batch_execution_state(manifest, roadmap, item_id)
    if errors:
        raise WorkflowError("\n".join(errors))
    return selected, run_worker_batch_validation(selected, command_runner=command_runner)


def selected_manifest_items(manifest: BatchManifest, item_id: str | None = None) -> list[BatchItem]:
    return [candidate for candidate in manifest.items if item_id is None or candidate.id == item_id]


def changed_paths_for_item(item: BatchItem, base_sha: str) -> list[str]:
    if not item.worktree:
        return []
    worktree = Path(item.worktree)
    if not worktree.is_absolute():
        worktree = REPO_ROOT / worktree
    return changed_files_for_worktree(worktree, base_sha)


def validation_commands_for_items(items: list[BatchItem]) -> list[str]:
    commands: list[str] = []
    seen: set[str] = set()
    for item in items:
        for command in item.validations:
            if command not in seen:
                commands.append(command)
                seen.add(command)
    return commands


def run_worker_batch_validation(items: list[BatchItem], command_runner=None) -> str:
    runner = command_runner or run_host_validation_command
    output: list[str] = []
    for item in items:
        if not item.validations:
            continue
        cwd = worktree_path_for_validation(item)
        output.append(f"[{item.id}] {slash_path(cwd)}\n")
        for command in item.validations:
            output.append(f"> {command}\n")
            command_output = runner(command, cwd)
            if command_output:
                output.append(command_output)
                if not command_output.endswith(("\n", "\r")):
                    output.append("\n")
    return "".join(output) if output else "no worker validation commands\n"


def worktree_path_for_validation(item: BatchItem) -> Path:
    if not item.worktree:
        return REPO_ROOT
    worktree = Path(item.worktree)
    if not worktree.is_absolute():
        worktree = REPO_ROOT / worktree
    return worktree


def run_host_batch_validation(validation_commands: list[str], command_runner=None) -> str:
    if not validation_commands:
        return "no worker validation commands\n"

    runner = command_runner or run_host_validation_command
    output: list[str] = []
    for command in validation_commands:
        output.append(f"> {command}\n")
        command_output = runner(command)
        if command_output:
            output.append(command_output)
            if not command_output.endswith(("\n", "\r")):
                output.append("\n")
    return "".join(output)


def run_host_validation_command(command: str, cwd: Path = REPO_ROOT) -> str:
    completed = subprocess.run(
        command,
        cwd=cwd,
        shell=True,
        check=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return completed.stdout


def emit_tool_output(output: str) -> None:
    text = output if output.endswith(("\n", "\r")) else f"{output}\n"
    stream = getattr(sys.stdout, "buffer", None)
    if stream is not None:
        stream.write(text.encode("utf-8", errors="replace"))
        stream.flush()
    else:
        print(text, end="")


def write_validation_result(batch: Path, manifest: BatchManifest, status: str, commands: list[str]) -> None:
    timestamp = dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat()
    command_summary = ", ".join(commands) if commands else "no worker commands"
    result = f"{timestamp} batch validate {status}: host batch validation; {command_summary}"
    updated = manifest.model_copy(update={"validation_results": [*manifest.validation_results, result]})
    batch.write_text(render_batch_manifest(updated), encoding="utf-8", newline="\n")


def finalize_batch_manifest(
    manifest: BatchManifest,
    roadmap,
    *,
    target: str,
    cleanup: bool = True,
    keep_branches: bool = False,
) -> BatchManifest:
    errors = batch_finalization_errors(manifest, target)
    if errors:
        raise WorkflowError("\n".join(errors))

    target_sha = git_output(["git", "rev-parse", target])
    updated_items: list[BatchItem] = []
    for item in manifest.items:
        roadmap_item = roadmap.by_id.get(item.id)
        outcome = roadmap_outcome_for_item(roadmap_item)
        status = "roadmap_closed" if outcome == "roadmap_completed" else "integrated"
        worktree = item.worktree
        worktree_cleanup = item.worktree_cleanup
        if cleanup and worktree:
            remove_worker_worktree_if_present(Path(worktree))
            worktree = ""
            worktree_cleanup = "cleaned after integration"
        if cleanup and not keep_branches and branch_exists(item.branch):
            delete_worker_branch(item.branch)
        updated_items.append(
            item.model_copy(
                update={
                    "status": status,
                    "roadmap_outcome": outcome,
                    "worktree": worktree,
                    "worktree_cleanup": worktree_cleanup,
                }
            )
        )

    return manifest.model_copy(
        update={
            "integration_status": "merged",
            "closeout_status": "completed",
            "integrated_target": target,
            "integrated_sha": target_sha,
            "items": updated_items,
            "integration_risk": "integrated; worker worktrees cleaned" if cleanup else "integrated; cleanup skipped",
        }
    )


def batch_finalization_errors(manifest: BatchManifest, target: str) -> list[str]:
    errors: list[str] = []
    if not git_output(["git", "rev-parse", "--verify", target]):
        return [f"target ref {target!r} does not exist"]
    for item in manifest.items:
        if branch_exists(item.branch) and not branch_is_ancestor(item.branch, target):
            errors.append(f"{item.id}: worker branch {item.branch} has commits not integrated into {target}")
        if not item.worktree:
            continue
        worktree = Path(item.worktree)
        if not worktree.is_absolute():
            worktree = REPO_ROOT / worktree
        if not worktree.exists():
            continue
        try:
            paths = changed_files_for_worktree(worktree, manifest.base_sha)
        except (OSError, subprocess.CalledProcessError) as error:
            errors.append(f"{item.id}: cannot inspect worker worktree {slash_path(worktree)}: {error}")
            continue
        normalized_scopes = [normalize_repo_path(scope) for scope in item.write_scopes]
        for path in paths:
            normalized = normalize_repo_path(path)
            in_scope = any(path_within_scope(normalized, scope) for scope in normalized_scopes)
            if path_matches_ref(worktree, target, normalized):
                continue
            if in_scope:
                errors.append(f"{item.id}: dirty in-scope worker change is not integrated into {target}: {normalized}")
            else:
                errors.append(f"{item.id}: dirty out-of-scope worker change blocks finalization: {normalized}")
    return errors


def roadmap_outcome_for_item(item: RoadmapItem | None) -> str:
    if item is None:
        return "deferred_followup_required"
    if item.planning_state == "completed":
        return "roadmap_completed"
    if item.planning_state == "current_candidate":
        return "slice_landed_item_still_current"
    return "deferred_followup_required"


def branch_exists(branch: str) -> bool:
    return bool(git_output(["git", "rev-parse", "--verify", branch]))


def branch_is_ancestor(branch: str, target: str) -> bool:
    return subprocess.run(
        ["git", "merge-base", "--is-ancestor", branch, target],
        cwd=REPO_ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


def path_matches_ref(worktree: Path, ref: str, path: str) -> bool:
    ref_blob = subprocess.run(
        ["git", "-C", str(worktree), "rev-parse", "--verify", f"{ref}:{path}"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    ref_has_path = ref_blob.returncode == 0
    worktree_path = worktree / path
    if not worktree_path.exists():
        return not ref_has_path
    if not worktree_path.is_file() or not ref_has_path:
        return False
    worktree_blob = subprocess.run(
        ["git", "-C", str(worktree), "hash-object", f"--path={path}", str(worktree_path)],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=True,
    ).stdout.strip()
    return worktree_blob == ref_blob.stdout.strip()


def remove_worker_worktree_if_present(worktree: Path) -> None:
    if not worktree.is_absolute():
        worktree = REPO_ROOT / worktree
    if not worktree.exists():
        return
    subprocess.run(
        ["git", "-c", "core.longpaths=true", "worktree", "remove", "--force", str(worktree)],
        cwd=REPO_ROOT,
        check=True,
    )


def delete_worker_branch(branch: str) -> None:
    subprocess.run(["git", "branch", "-D", branch], cwd=REPO_ROOT, check=True)


def repo_refs_snapshot() -> str:
    branches = subprocess.run(
        ["git", "branch", "--list"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    ).stdout.strip()
    worktrees = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    ).stdout.strip()
    return f"branches:\n{branches}\n\nworktrees:\n{worktrees}".strip()


def in_scope_changes_for_manifest(manifest: BatchManifest) -> list[str]:
    _all_changes, in_scope_changes, inspection_errors = worker_change_groups_for_manifest(manifest)
    return [*inspection_errors, *in_scope_changes]


def worker_change_groups_for_manifest(manifest: BatchManifest) -> tuple[list[str], list[str], list[str]]:
    changes: list[str] = []
    in_scope_changes: list[str] = []
    inspection_errors: list[str] = []
    for item in manifest.items:
        try:
            paths = changed_paths_for_item(item, manifest.base_sha)
        except (OSError, subprocess.CalledProcessError) as error:
            inspection_errors.append(f"{item.id}: cannot inspect worker changes: {error}")
            continue
        for path in paths:
            normalized = normalize_repo_path(path)
            changes.append(f"{item.id}: {normalized}")
            scopes = [normalize_repo_path(scope) for scope in item.write_scopes]
            if any(path_within_scope(normalized, scope) for scope in scopes):
                in_scope_changes.append(f"{item.id}: {normalized}")
    return changes, in_scope_changes, inspection_errors


def remove_worker_worktrees_and_branches(manifest: BatchManifest) -> None:
    for item in manifest.items:
        if item.worktree:
            worktree = Path(item.worktree)
            if not worktree.is_absolute():
                worktree = REPO_ROOT / worktree
            if worktree.exists():
                subprocess.run(
                    ["git", "-c", "core.longpaths=true", "worktree", "remove", "--force", str(worktree)],
                    cwd=REPO_ROOT,
                    check=True,
                )
        if git_output(["git", "rev-parse", "--verify", item.branch]):
            subprocess.run(["git", "branch", "-D", item.branch], cwd=REPO_ROOT, check=True)


if __name__ == "__main__":
    app()
