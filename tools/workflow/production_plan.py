#!/usr/bin/env python3
"""
Bridge production milestones to WR implementation contract planning.

File: tools/workflow/production_plan.py
Module: production_plan
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import typer
from rich.console import Console

from production_state import (
    PRODUCTION_SOURCE,
    ProductionMilestone,
    ProductionPlanningState,
    ProductionTrack,
    gate_status_errors,
    load_production_tracks,
)
from roadmap_state import (
    REPO_ROOT,
    ROADMAP_SOURCE,
    RoadmapItem,
    RoadmapState,
    WorkflowError,
    batch_ineligibility_reason,
    decision_gate_errors,
    load_roadmap,
    promotion_preflight,
    repo_path,
)


PROMPT_TEMPLATE = REPO_ROOT / "docs-site/src/content/docs/workspace/prompt-templates/production-implementation-contract.md"
DEFAULT_IMPLEMENTATION_PLAN_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/implementation-plans"

PlanAction = Literal[
    "write_implementation_contract",
    "write_promotion_contract",
    "switch_current_candidate",
    "repair_wr_promotion_metadata",
    "stop_on_promotion_blocker",
    "design_first",
    "already_completed",
    "invalid",
]

console = Console()
app = typer.Typer(no_args_is_help=True, help="Plan a production milestone WR implementation contract.")


@dataclass(frozen=True)
class ProductionPlanContext:
    planning: ProductionPlanningState
    roadmap: RoadmapState
    track: ProductionTrack
    milestone: ProductionMilestone
    roadmap_item: RoadmapItem


def resolve_plan_context(
    milestone_id: str,
    roadmap_id: str,
    *,
    production_source: Path = PRODUCTION_SOURCE,
    roadmap_source: Path = ROADMAP_SOURCE,
) -> ProductionPlanContext:
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    track, milestone = find_milestone(planning, milestone_id)
    roadmap_item = roadmap.by_id.get(roadmap_id)
    if roadmap_item is None:
        raise WorkflowError(f"{roadmap_id}: not present in combined roadmap sources")
    if roadmap_id not in milestone.roadmap_links:
        raise WorkflowError(f"{roadmap_id}: not linked by production milestone {milestone_id}")
    return ProductionPlanContext(
        planning=planning,
        roadmap=roadmap,
        track=track,
        milestone=milestone,
        roadmap_item=roadmap_item,
    )


def find_milestone(planning: ProductionPlanningState, milestone_id: str) -> tuple[ProductionTrack, ProductionMilestone]:
    for track in planning.tracks:
        for milestone in track.milestones:
            if milestone.id == milestone_id:
                return track, milestone
    raise WorkflowError(f"{milestone_id}: not present in production tracks source")


def classify_plan_action(context: ProductionPlanContext) -> PlanAction:
    item = context.roadmap_item
    if item.planning_state == "completed":
        return "already_completed"
    if production_gate_errors(context.milestone) or decision_gate_errors(item, applies_to="implementation"):
        return "design_first"
    if item.planning_state == "current_candidate":
        return "write_implementation_contract" if batch_ineligibility_reason(item) is None else "design_first"
    if item.planning_state == "ready_next":
        preflight = promotion_preflight(context.roadmap, item.id, "current_candidate")
        if preflight.status == "needs_switch":
            return "switch_current_candidate"
        if preflight.status == "metadata_blocked":
            return "repair_wr_promotion_metadata"
        if preflight.status == "hard_blocked":
            return "stop_on_promotion_blocker"
        return "write_promotion_contract"
    if item.planning_state == "blocked_deferred":
        return "design_first"
    return "invalid"


def production_gate_errors(milestone: ProductionMilestone) -> list[str]:
    if not milestone.requires_design_gates_to_pass:
        return []
    errors: list[str] = []
    for gate in milestone.design_gates:
        errors.extend(gate_status_errors(milestone.id, gate.kind, gate.path, gate.required_status, gate.reason))
    return errors


def default_contract_path(item: RoadmapItem, root: Path = DEFAULT_IMPLEMENTATION_PLAN_ROOT) -> Path:
    return root / f"{item.id.lower()}-{slugify(item.title)}" / "plan.md"


def slugify(value: str) -> str:
    cleaned = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return cleaned or "implementation-contract"


def render_readiness_report(context: ProductionPlanContext, action: PlanAction, contract_path: Path) -> str:
    item = context.roadmap_item
    milestone = context.milestone
    dependency_states = []
    for dependency in item.dependencies:
        dependency_item = context.roadmap.by_id.get(dependency)
        state = dependency_item.planning_state if dependency_item else "missing"
        dependency_states.append(f"{dependency}:{state}")
    gate_errors = production_gate_errors(milestone) + decision_gate_errors(item, applies_to="implementation")
    batch_reason = batch_ineligibility_reason(item) if item.planning_state == "current_candidate" else ""
    lines = [
        "# Production Plan Readiness",
        "",
        f"Production track: {context.track.id} - {context.track.title}",
        f"Production milestone: {milestone.id} - {milestone.title}",
        f"Production milestone state: {milestone.state}",
        f"Roadmap item: {item.id} - {item.title}",
        f"Roadmap planning_state: {item.planning_state}",
        f"Roadmap blocker: {item.blocker_label}",
        f"Roadmap dependencies: {', '.join(dependency_states) or 'none'}",
        f"Milestone links WR item: {'yes' if item.id in milestone.roadmap_links else 'no'}",
        f"Contract target: {repo_path(contract_path)}",
        f"Next action: {action}",
        "",
        "## Readiness Notes",
        "",
    ]
    if gate_errors:
        lines.extend(f"- Design gate unmet: {error}" for error in gate_errors)
    if batch_reason:
        lines.append(f"- Current candidate is not batch eligible: {batch_reason}")
    preflight = promotion_preflight(context.roadmap, item.id, "current_candidate") if item.planning_state == "ready_next" else None
    if action == "write_promotion_contract":
        lines.append("- This is ready_next planning work, not direct implementation. The contract should verify promotion evidence first.")
    elif action == "switch_current_candidate":
        lines.append("- Promotion preflight found an overlapping current candidate. Switch current candidate before planning code.")
    elif action == "repair_wr_promotion_metadata":
        lines.append("- Promotion preflight found roadmap metadata that must be repaired before promotion.")
    elif action == "stop_on_promotion_blocker":
        lines.append("- Promotion preflight found a hard blocker. Stop and report before doing adjacent WR work.")
    elif action == "write_implementation_contract":
        lines.append("- This WR row is current-candidate eligible. The contract may plan implementation and closeout.")
    elif action == "design_first":
        lines.append("- Design or gate repair must happen before implementation planning.")
    elif action == "already_completed":
        lines.append("- The WR row is completed. Check closeout evidence before doing more work.")
    elif action == "invalid":
        lines.append("- The current state does not map to a production implementation contract.")
    if preflight is not None:
        lines.extend(["", "## Promotion Preflight", ""])
        lines.append(f"- Status: {preflight.status}")
        if preflight.blocking_current_candidates:
            lines.append(f"- Blocking current candidate: {', '.join(preflight.blocking_current_candidates)}")
        if preflight.reasons:
            lines.append("- Reasons:")
            lines.extend(f"  - {reason}" for reason in preflight.reasons)
        if preflight.suggested_command:
            lines.append(f"- Suggested command: `{preflight.suggested_command}`")
    lines.extend(["", "## Prompt Template", "", f"- {repo_path(PROMPT_TEMPLATE)}", ""])
    return "\n".join(lines)


def render_contract_prompt(context: ProductionPlanContext, action: PlanAction, contract_path: Path) -> str:
    item = context.roadmap_item
    milestone = context.milestone
    if action == "write_implementation_contract":
        task_line = "Create a decision-complete implementation contract, then stop. No product code changes."
    elif action == "write_promotion_contract":
        task_line = (
            "Create a decision-complete promotion and implementation-readiness contract, then stop. "
            "Verify whether the WR row can honestly be promoted before planning code changes."
        )
    elif action == "switch_current_candidate":
        task_line = (
            "Switch the current candidate through task roadmap:switch-current, validate, rerun task ai:goal, then stop. "
            "Do not inspect or repair adjacent WR evidence."
        )
    elif action == "repair_wr_promotion_metadata":
        task_line = (
            "Repair the exact roadmap metadata named by promotion preflight, validate, rerun task ai:goal, then stop. "
            "Do not inspect or repair adjacent WR evidence."
        )
    elif action == "stop_on_promotion_blocker":
        task_line = "Stop and report the hard promotion blocker. Do not inspect or repair adjacent WR evidence."
    elif action == "design_first":
        task_line = "Create the design-first planning contract needed to clear gates before implementation, then stop."
    elif action == "already_completed":
        task_line = "Review completion evidence and decide whether production milestone evidence should be updated, then stop."
    else:
        task_line = "Investigate why this milestone and WR row cannot produce a valid implementation contract, then stop."
    return "\n".join(
        [
            "Prepare the next production-slice implementation contract.",
            "",
            f"Production milestone: {milestone.id}",
            f"Candidate WR item: {item.id}",
            f"Next action classification: {action}",
            f"Contract path: {repo_path(contract_path)}",
            "",
            task_line,
            "",
            "Use the current production tracks, roadmap items, design docs, and task workflow.",
            "First verify whether the WR item is actually ready to promote or implement.",
            "If it is not ready, write the design/planning work needed instead.",
            "",
            "The contract must be decision-complete: scope, exact owners/modules, non-goals, gates, implementation steps, validation, stop conditions, and closeout requirements.",
            "",
            "No product code changes.",
            "",
        ]
    )


def render_contract_scaffold(context: ProductionPlanContext, action: PlanAction) -> str:
    item = context.roadmap_item
    milestone = context.milestone
    return "\n".join(
        [
            "---",
            f"title: {item.id} Implementation Contract",
            f"description: Implementation contract for {item.title} under {milestone.id}.",
            "status: draft",
            "owner: workspace",
            "layer: workspace",
            "canonical: false",
            f"last_reviewed: {context.planning.production.last_reviewed}",
            "related:",
            "  - ../../../workspace/production-tracks.yaml",
            "  - ../../../workspace/roadmap-items.yaml",
            "  - ../../../workspace/production-track-planning-model.md",
            "---",
            "",
            f"# {item.id} Implementation Contract",
            "",
            "## Goal",
            "",
            f"Production milestone: `{milestone.id}`",
            f"Roadmap item: `{item.id}`",
            f"Next action classification: `{action}`",
            "",
            "## Source Of Truth",
            "",
            "- Production track source: `docs-site/src/content/docs/workspace/production-tracks.yaml`",
            "- Roadmap source: `docs-site/src/content/docs/workspace/roadmap-items.yaml`",
            "- Prompt template: `docs-site/src/content/docs/workspace/prompt-templates/production-implementation-contract.md`",
            "",
            "## Readiness",
            "",
            "- Required design gates:",
            "- Required dependencies:",
            "- Known blockers:",
            "- Explicit non-goals:",
            "",
            "## Implementation Scope",
            "",
            "- Files/modules expected to change:",
            "- Public APIs/types affected:",
            "- Data flow:",
            "- Diagnostics/errors:",
            "- Persistence or migration impact:",
            "",
            "## Acceptance Criteria",
            "",
            "- Observable behavior:",
            "- Tests:",
            "- Generated docs/schema/roadmap updates:",
            "",
            "## Stop Conditions",
            "",
            "- Stop if an unaccepted design or ADR is required.",
            "- Stop if the WR row cannot be honestly promoted or selected.",
            "- Stop if implementation would exceed the stated production milestone.",
            "",
            "## Closeout Requirements",
            "",
            "- Validation commands:",
            "- Roadmap evidence update:",
            "- Production milestone evidence impact:",
            "",
        ]
    )


@app.command()
def plan(
    milestone: str = typer.Option(..., help="Production milestone id, for example PM-SDF-OW-001."),
    roadmap: str = typer.Option(..., help="Roadmap item id, for example WR-019."),
    write_scaffold: bool = typer.Option(False, "--write-scaffold", help="Write a draft contract scaffold."),
    force: bool = typer.Option(False, "--force", help="Overwrite an existing scaffold path."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    out: Path | None = typer.Option(None, help="Optional implementation contract path."),
) -> None:
    try:
        context = resolve_plan_context(
            milestone,
            roadmap,
            production_source=production_source,
            roadmap_source=roadmap_source,
        )
        contract_path = out or default_contract_path(context.roadmap_item)
        action = classify_plan_action(context)
        if action == "invalid":
            raise WorkflowError(f"{milestone} + {roadmap}: no valid production planning action")
        console.print(render_readiness_report(context, action, contract_path), soft_wrap=True)
        console.print("## Ready-to-use prompt\n", soft_wrap=True)
        console.print(render_contract_prompt(context, action, contract_path), soft_wrap=True)
        if write_scaffold:
            write_contract_scaffold(context, action, contract_path, force=force)
            console.print(f"[green]wrote implementation contract scaffold:[/green] {repo_path(contract_path)}")
    except WorkflowError as error:
        console.print("[red]production plan failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("_commands", hidden=True)
def commands() -> None:
    """Keep Typer in multi-command mode so the public `plan` subcommand is stable."""
    console.print("plan")


def write_contract_scaffold(context: ProductionPlanContext, action: PlanAction, path: Path, *, force: bool = False) -> None:
    if path.exists() and not force:
        raise WorkflowError(f"contract scaffold already exists: {repo_path(path)}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(render_contract_scaffold(context, action), encoding="utf-8", newline="\n")


if __name__ == "__main__":
    app()
