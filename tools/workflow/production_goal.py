#!/usr/bin/env python3
"""
Generate a production-track scoped Codex /goal coordinator prompt.

File: tools/workflow/production_goal.py
Module: production_goal
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from pathlib import Path

import typer
from rich.console import Console

from production_plan import ProductionPlanContext, classify_plan_action, production_gate_errors
from production_state import (
    PRODUCTION_SOURCE,
    ProductionMilestone,
    ProductionPlanningState,
    ProductionTrack,
    load_production_tracks,
    validate_completion_quality as validate_production_completion_quality,
    validate_evidence_gates,
)
from roadmap_state import ROADMAP_SOURCE, RoadmapItem, RoadmapState, WorkflowError, load_roadmap


console = Console()
app = typer.Typer(no_args_is_help=True, help="Generate a production-track /goal coordinator prompt.")


class GoalScope(str, Enum):
    full = "full"
    non_deferred = "non-deferred"


@dataclass(frozen=True)
class RoadmapAction:
    item: RoadmapItem | None
    roadmap_id: str
    action: str


@dataclass(frozen=True)
class MilestoneGoalStep:
    milestone: ProductionMilestone
    dependency_states: tuple[str, ...]
    gate_errors: tuple[str, ...]
    evidence_errors: tuple[str, ...]
    roadmap_actions: tuple[RoadmapAction, ...]
    next_action: str


def find_track(planning: ProductionPlanningState, track_id: str) -> ProductionTrack:
    for track in planning.tracks:
        if track.id == track_id:
            return track
    raise WorkflowError(f"{track_id}: not present in production tracks source")


def ordered_track_milestones(track: ProductionTrack) -> list[ProductionMilestone]:
    by_id = {milestone.id: milestone for milestone in track.milestones}
    ordered: list[ProductionMilestone] = []
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(milestone: ProductionMilestone) -> None:
        if milestone.id in visited:
            return
        if milestone.id in visiting:
            raise WorkflowError(f"{track.id}: milestone dependency cycle includes {milestone.id}")
        visiting.add(milestone.id)
        for dependency in milestone.dependencies:
            dependency_milestone = by_id.get(dependency)
            if dependency_milestone is not None:
                visit(dependency_milestone)
        visiting.remove(milestone.id)
        visited.add(milestone.id)
        ordered.append(milestone)

    for milestone in track.milestones:
        visit(milestone)
    return ordered


def build_goal_steps(
    planning: ProductionPlanningState,
    roadmap: RoadmapState,
    track: ProductionTrack,
    *,
    roadmap_source: Path = ROADMAP_SOURCE,
) -> list[MilestoneGoalStep]:
    milestone_by_id = planning.by_milestone_id
    evidence_errors = validate_evidence_gates(planning)
    quality_errors = validate_production_completion_quality(planning, roadmap_path=roadmap_source)
    steps: list[MilestoneGoalStep] = []
    for milestone in ordered_track_milestones(track):
        dependency_states = tuple(dependency_state(dependency, milestone_by_id) for dependency in milestone.dependencies)
        gate_errors = tuple(production_gate_errors(milestone))
        milestone_evidence_errors = tuple(
            error for error in evidence_errors + quality_errors if error.startswith(f"{milestone.id}:")
        )
        roadmap_actions = tuple(roadmap_actions_for_milestone(planning, roadmap, track, milestone))
        steps.append(
            MilestoneGoalStep(
                milestone=milestone,
                dependency_states=dependency_states,
                gate_errors=gate_errors,
                evidence_errors=milestone_evidence_errors,
                roadmap_actions=roadmap_actions,
                next_action=classify_milestone_next_action(milestone, dependency_states, gate_errors, roadmap_actions),
            )
        )
    return steps


def dependency_state(dependency: str, milestone_by_id: dict[str, ProductionMilestone]) -> str:
    dependency_milestone = milestone_by_id.get(dependency)
    state = dependency_milestone.state if dependency_milestone else "missing"
    return f"{dependency}:{state}"


def roadmap_actions_for_milestone(
    planning: ProductionPlanningState,
    roadmap: RoadmapState,
    track: ProductionTrack,
    milestone: ProductionMilestone,
) -> list[RoadmapAction]:
    actions: list[RoadmapAction] = []
    for roadmap_id in milestone.roadmap_links:
        item = roadmap.by_id.get(roadmap_id)
        if item is None:
            actions.append(RoadmapAction(item=None, roadmap_id=roadmap_id, action="missing_roadmap_link"))
            continue
        if milestone.state not in {"active", "ready_next"}:
            actions.append(RoadmapAction(item=item, roadmap_id=roadmap_id, action=f"linked_row_{item.planning_state}"))
            continue
        context = ProductionPlanContext(
            planning=planning,
            roadmap=roadmap,
            track=track,
            milestone=milestone,
            roadmap_item=item,
        )
        actions.append(RoadmapAction(item=item, roadmap_id=roadmap_id, action=classify_plan_action(context)))
    return actions


def classify_milestone_next_action(
    milestone: ProductionMilestone,
    dependency_states: tuple[str, ...],
    gate_errors: tuple[str, ...],
    roadmap_actions: tuple[RoadmapAction, ...],
) -> str:
    incomplete_dependencies = [state for state in dependency_states if not state.endswith(":completed")]
    if milestone.state == "completed":
        return "verify_completed_evidence"
    if incomplete_dependencies:
        return "wait_for_dependency_completion"
    if gate_errors:
        return "clear_design_or_adr_gates"
    if milestone.state == "designing":
        return "write_or_accept_design_before_implementation"
    if milestone.state == "blocked":
        return "repair_blocker_before_implementation"
    if milestone.state == "deferred":
        return "keep_deferred_until_reprioritized"
    if milestone.state in {"active", "ready_next"}:
        if not roadmap_actions:
            return "add_or_select_wr_roadmap_link"
        action_names = {action.action for action in roadmap_actions}
        if "write_implementation_contract" in action_names:
            return "execute_next_wr_implementation_contract"
        if "switch_current_candidate" in action_names:
            return "switch_current_candidate"
        if "repair_wr_promotion_metadata" in action_names:
            return "repair_wr_promotion_metadata"
        if "write_promotion_contract" in action_names:
            return "prepare_wr_promotion_contract"
        if "stop_on_promotion_blocker" in action_names:
            return "stop_on_promotion_blocker"
        if "design_first" in action_names:
            return "clear_wr_design_or_gate_blocker"
        if "already_completed" in action_names:
            return "verify_wr_completion_evidence"
        return "investigate_wr_link_state"
    return "investigate_milestone_state"


def render_track_goal(
    planning: ProductionPlanningState,
    roadmap: RoadmapState,
    track: ProductionTrack,
    *,
    roadmap_source: Path = ROADMAP_SOURCE,
    scope: GoalScope = GoalScope.full,
) -> str:
    steps = build_goal_steps(planning, roadmap, track, roadmap_source=roadmap_source)
    completion_errors = track_completion_errors(planning, roadmap_source=roadmap_source, track=track, scope=scope)
    out_of_scope = out_of_scope_milestones(track, scope)
    lines = [
        f"# Production Track /goal Kickoff: {track.id}",
        "",
        f"Track: {track.title}",
        f"State: {track.state}",
        f"Owner: {track.owner}",
        f"Strategic goal: {track.strategic_goal}",
        "",
        "## Success Criteria",
        "",
    ]
    lines.extend(f"- {criterion}" for criterion in track.success_criteria)
    lines.extend(["", "## Ordered Milestone Plan", ""])
    for step in steps:
        lines.extend(render_milestone_step(step, scope=scope))
    lines.extend(render_completion_gate(completion_errors, out_of_scope=out_of_scope, scope=scope))
    lines.extend(render_goal_prompt(track, steps, scope=scope))
    return "\n".join(lines)


def render_milestone_step(step: MilestoneGoalStep, *, scope: GoalScope = GoalScope.full) -> list[str]:
    milestone = step.milestone
    lines = [
        f"### {milestone.id} - {milestone.title}",
        "",
        f"- Kind/state: {milestone.kind}/{milestone.state}",
        f"- Goal: {milestone.goal}",
        f"- Outcome: {milestone.outcome}",
        f"- Dependencies: {', '.join(step.dependency_states) or 'none'}",
        f"- Roadmap links: {', '.join(milestone.roadmap_links) or 'none'}",
        f"- Next legal action: {step.next_action}",
    ]
    if is_out_of_scope_milestone(milestone, scope):
        lines.append("- Bounded goal scope: preserved out of scope; do not implement for `--scope non-deferred`.")
    if milestone.design_gates:
        lines.append("- Design gates:")
        lines.extend(
            f"  - {gate.kind} {gate.path} requires {gate.required_status}: {gate.reason}"
            for gate in milestone.design_gates
        )
    if step.gate_errors:
        lines.append("- Gate blockers:")
        lines.extend(f"  - {error}" for error in step.gate_errors)
    if milestone.evidence_gates:
        lines.append("- Evidence gates:")
        lines.extend(
            f"  - {gate.path} requires {gate.required_status}: {gate.reason}"
            for gate in milestone.evidence_gates
        )
    if step.evidence_errors:
        lines.append("- Completion evidence issues:")
        lines.extend(f"  - {error}" for error in step.evidence_errors)
    if step.roadmap_actions:
        lines.append("- Linked WR actions:")
        for action in step.roadmap_actions:
            if action.item is None:
                lines.append(f"  - {action.roadmap_id}: {action.action}")
            else:
                lines.append(
                    f"  - {action.item.id} {action.item.title}: {action.action} "
                    f"(state={action.item.planning_state}, blocker={action.item.blocker_label})"
                )
                if action.item.write_scopes:
                    lines.append("    write scopes:")
                    lines.extend(f"      - {scope}" for scope in action.item.write_scopes)
                if action.item.validations:
                    lines.append("    validations:")
                    lines.extend(f"      - {validation}" for validation in action.item.validations)
    lines.append("")
    return lines


def track_completion_errors(
    planning: ProductionPlanningState,
    *,
    roadmap_source: Path,
    track: ProductionTrack,
    scope: GoalScope = GoalScope.full,
) -> list[str]:
    target_milestone_ids = {
        milestone.id for milestone in track.milestones if not is_out_of_scope_milestone(milestone, scope)
    }
    errors = [
        f"{milestone.id}: state is {milestone.state!r}, expected 'completed'"
        for milestone in track.milestones
        if milestone.id in target_milestone_ids and milestone.state != "completed"
    ]
    evidence_errors = validate_evidence_gates(planning)
    quality_errors = validate_production_completion_quality(planning, roadmap_path=roadmap_source)
    errors.extend(
        error
        for error in evidence_errors + quality_errors
        if error.split(":", 1)[0] in target_milestone_ids
    )
    return errors


def out_of_scope_milestones(track: ProductionTrack, scope: GoalScope) -> list[ProductionMilestone]:
    return [milestone for milestone in track.milestones if is_out_of_scope_milestone(milestone, scope)]


def is_out_of_scope_milestone(milestone: ProductionMilestone, scope: GoalScope) -> bool:
    return scope == GoalScope.non_deferred and milestone.state in {"blocked", "deferred"}


def render_completion_gate(
    errors: list[str],
    *,
    out_of_scope: list[ProductionMilestone],
    scope: GoalScope,
) -> list[str]:
    lines = ["## Track Completion Gate", ""]
    if errors:
        if scope == GoalScope.non_deferred:
            lines.append("Non-deferred scope completion is blocked until:")
        else:
            lines.append("Track completion is blocked until:")
        lines.extend(f"- {error}" for error in errors)
    else:
        if scope == GoalScope.non_deferred:
            lines.append(
                "Non-deferred scope completion metadata may be considered after final roadmap and production gates pass."
            )
        else:
            lines.append("Track completion metadata may be considered after final roadmap and production gates pass.")
    if out_of_scope:
        lines.extend(["", "Preserved out-of-scope milestones:"])
        lines.extend(f"- {milestone.id}: {milestone.state} - {milestone.title}" for milestone in out_of_scope)
    lines.extend(
        [
            "",
            "Required final validation before claiming track completion:",
            "- task production:render",
            "- task production:validate",
            "- task production:check",
            "- task roadmap:render",
            "- task roadmap:validate",
            "- task roadmap:check",
            "",
        ]
    )
    return lines


def render_goal_prompt(track: ProductionTrack, steps: list[MilestoneGoalStep], *, scope: GoalScope) -> list[str]:
    if scope == GoalScope.non_deferred:
        goal_line = f"/goal Complete the non-deferred scope of production track {track.id} - {track.title}."
        scope_rules = [
            "Do not implement blocked or deferred milestones; preserve them as explicit deferred gaps.",
            "",
        ]
        completion_rule = (
            "- The bounded non-deferred scope is complete only when every in-scope milestone is completed with valid "
            "evidence gates and the final production and roadmap checks pass."
        )
    else:
        goal_line = f"/goal Complete production track {track.id} - {track.title}."
        scope_rules = []
        completion_rule = (
            "- The production track is complete only when every milestone is completed with valid evidence gates and "
            "the final production and roadmap checks pass."
        )
    lines = [
        "## Ready-to-paste /goal Prompt",
        "",
        "```text",
        goal_line,
        "",
        "Use the current production tracks, roadmap items, design docs, and task workflow.",
        "Work through the finite milestone list in dependency order.",
        "For each milestone, perform exactly one legal next action, validate it, close it out, then rerun task ai:goal before continuing.",
        "Do not bypass design gates, ADR gates, WR roadmap state, write scopes, validation, closeout evidence, or completion-quality rules.",
        "",
    ]
    lines.extend(scope_rules)
    lines.append("Milestone order and current next actions:")
    for step in steps:
        suffix = ""
        if is_out_of_scope_milestone(step.milestone, scope):
            suffix = f" (preserved out of scope: {step.milestone.state})"
        lines.append(f"- {step.milestone.id}: {step.next_action}{suffix}")
    lines.extend(
        [
            "",
            "Coordinator rules:",
            "- Completed milestones: verify evidence gates and completion-quality claims before relying on them.",
            "- Designing, blocked, or deferred milestones: do design, ADR, or unblock work only; do not implement product code.",
            "- Active or ready_next milestones: use linked WR rows and task production:plan before code changes.",
            "- After a failed roadmap:promote or gate command, only repair exact metadata, run task roadmap:switch-current, or stop/report; do not investigate adjacent WR evidence.",
            "- Implement only one bounded WR slice or design-gate repair at a time.",
            "- After implementation, run focused validation, closeout or drift-check routines, roadmap render/validate/check, and production render/validate/check as applicable.",
            completion_rule,
            "",
            "Stop immediately if ownership is unclear, a dependency is incomplete, a gate is unmet, a WR row is not ready for its required action, validation fails, closeout evidence is missing, or source files changed enough that this command must be rerun.",
            "```",
            "",
        ]
    )
    return lines


@app.command()
def goal(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-SDF-OW."),
    scope: GoalScope = typer.Option(
        GoalScope.full,
        "--scope",
        help="Goal completion scope. Use non-deferred to preserve blocked/deferred milestones out of scope.",
    ),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
) -> None:
    try:
        planning = load_production_tracks(production_source)
        roadmap = load_roadmap(roadmap_source)
        production_track = find_track(planning, track)
        console.print(
            render_track_goal(planning, roadmap, production_track, roadmap_source=roadmap_source, scope=scope),
            soft_wrap=True,
        )
    except WorkflowError as error:
        console.print("[red]production goal failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("_commands", hidden=True)
def commands() -> None:
    """Keep Typer in multi-command mode so the public `goal` subcommand is stable."""
    console.print("goal")


if __name__ == "__main__":
    app()
