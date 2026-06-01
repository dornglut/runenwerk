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
from roadmap_state import ROADMAP_SOURCE, RoadmapItem, RoadmapState, WorkflowError, load_roadmap, repo_path
from track_execution_manifest import (
    FULL_TRACK_PERMISSION_SET,
    TRACK_EXECUTION_LOCK_ROOT,
    TRACK_EXECUTION_MANIFEST_ROOT,
    LoadedTrackExecutionManifest,
    TrackExecutionManifestMilestone,
    audit_manifest_or_raise,
    full_automation_preflight_errors,
    implementation_authorization_note,
    load_track_execution_lock,
    load_track_execution_manifest,
    next_action_blockers,
    track_execution_lock_errors,
    truth_claim_summary_lines,
)


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


@dataclass(frozen=True)
class TrackGoalSummary:
    track: ProductionTrack
    steps: tuple[MilestoneGoalStep, ...]
    completion_errors: tuple[str, ...]


@dataclass(frozen=True)
class ManifestGoalContext:
    loaded: LoadedTrackExecutionManifest
    current_entry: TrackExecutionManifestMilestone
    current_step: MilestoneGoalStep
    manifest_workflow_action: str
    unmet_gates: tuple[str, ...]
    implementation_authorized: bool
    implementation_authorization_note: str
    must_stop: bool
    full_automation_target: bool
    full_automation_ready: bool
    full_automation_blockers: tuple[str, ...]
    execution_lock_path: Path | None
    execution_lock_ready: bool
    execution_lock_blockers: tuple[str, ...]


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


def build_manifest_goal_context(
    planning: ProductionPlanningState,
    roadmap: RoadmapState,
    track: ProductionTrack,
    steps: list[MilestoneGoalStep],
    loaded_manifest: LoadedTrackExecutionManifest,
    *,
    production_source: Path = PRODUCTION_SOURCE,
    roadmap_source: Path = ROADMAP_SOURCE,
    lock_source_root: Path = TRACK_EXECUTION_LOCK_ROOT,
) -> ManifestGoalContext:
    audit_manifest_or_raise(loaded_manifest, track=track, roadmap=roadmap)
    current_step = first_incomplete_or_evidence_step(tuple(steps)) or steps[-1]
    current_entry = loaded_manifest.manifest.by_milestone_id[current_step.milestone.id]
    track_completed = all(milestone.state == "completed" for milestone in track.milestones)
    if track_completed:
        manifest_workflow_action = "already_completed"
        manifest_blockers: list[str] = []
    else:
        manifest_workflow_action, manifest_blockers = next_action_blockers(
            current_entry,
            current_step.milestone,
            planning=planning,
            track=track,
            roadmap=roadmap,
        )
    if track_completed:
        unmet_gates = ()
    else:
        unmet_gates = tuple(manifest_unmet_gates(current_step, current_entry) + manifest_blockers)
    raw_implementation_note = implementation_authorization_note(
        current_entry,
        manifest_workflow_action,
        manifest_blockers,
    )
    implementation_authorized = raw_implementation_note.startswith("yes -")
    implementation_note = raw_implementation_note.removeprefix("yes - ").removeprefix("no - ")
    full_automation_blockers: tuple[str, ...] = ()
    execution_lock_path: Path | None = None
    execution_lock_blockers: tuple[str, ...] = ()
    if loaded_manifest.manifest.full_automation_target:
        harness_context = harness_manifest_goal_gate(track.id)
        if harness_context is not None:
            full_automation_blockers = tuple(harness_context["preflight_errors"])
            execution_lock_path = harness_context["lock_path"]
            execution_lock_blockers = tuple(harness_context["lock_errors"])
        else:
            full_automation_blockers = tuple(
                full_automation_preflight_errors(
                    loaded_manifest,
                    track=track,
                    roadmap=roadmap,
                    allow=FULL_TRACK_PERMISSION_SET,
                )
            )
            loaded_lock = load_track_execution_lock(track.id, root=lock_source_root)
            execution_lock_path = loaded_lock.path if loaded_lock is not None else None
            execution_lock_blockers = tuple(
                track_execution_lock_errors(
                    loaded_manifest,
                    loaded_lock,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=FULL_TRACK_PERMISSION_SET,
                    deny={"crate_creation", "foundation_extraction"},
                    track=track,
                )
            )
        if full_automation_blockers:
            unmet_gates = (*unmet_gates, *full_automation_blockers)
        if execution_lock_blockers:
            unmet_gates = (*unmet_gates, *execution_lock_blockers)
    return ManifestGoalContext(
        loaded=loaded_manifest,
        current_entry=current_entry,
        current_step=current_step,
        manifest_workflow_action=manifest_workflow_action,
        unmet_gates=unmet_gates,
        implementation_authorized=implementation_authorized,
        implementation_authorization_note=implementation_note,
        must_stop=True,
        full_automation_target=loaded_manifest.manifest.full_automation_target,
        full_automation_ready=loaded_manifest.manifest.full_automation_target and not full_automation_blockers,
        full_automation_blockers=full_automation_blockers,
        execution_lock_path=execution_lock_path,
        execution_lock_ready=loaded_manifest.manifest.full_automation_target and not execution_lock_blockers,
        execution_lock_blockers=execution_lock_blockers,
    )


def harness_manifest_goal_gate(track_id: str) -> dict | None:
    try:
        from execution.compiler import load_contract_pack
        from execution.locks import contract_pack_freshness_errors, execution_lock_errors, load_execution_lock
        from execution.preflight import preflight_pack
    except Exception:
        return None
    pack = load_contract_pack(track_id)
    if pack is None:
        return None
    lock = load_execution_lock(track_id)
    preflight_errors = [
        *contract_pack_freshness_errors(pack),
        *preflight_pack(pack, allow=FULL_TRACK_PERMISSION_SET),
    ]
    return {
        "preflight_errors": preflight_errors,
        "lock_path": lock.path if lock is not None else None,
        "lock_errors": execution_lock_errors(
            track_id,
            requested_permissions=FULL_TRACK_PERMISSION_SET,
        ),
    }


def manifest_unmet_gates(
    step: MilestoneGoalStep,
    entry: TrackExecutionManifestMilestone,
) -> list[str]:
    unmet: list[str] = []
    for dependency_state in step.dependency_states:
        if not dependency_state.endswith(":completed"):
            unmet.append(f"{entry.milestone_id}: dependency not complete: {dependency_state}")
    unmet.extend(step.gate_errors)
    unmet.extend(step.evidence_errors)
    if entry.future_wr_candidate:
        unmet.append(f"{entry.milestone_id}: Track Expansion must create or link {entry.future_wr_candidate}")
    for action in step.roadmap_actions:
        if action.item is None:
            unmet.append(f"{entry.milestone_id}: roadmap link {action.roadmap_id} is missing")
            continue
        if action.action != "write_implementation_contract":
            unmet.append(
                f"{action.item.id}: workflow action is {action.action} "
                f"(state={action.item.planning_state}, blocker={action.item.blocker_label})"
            )
    return unmet


def manifest_implementation_authorization(
    step: MilestoneGoalStep,
    entry: TrackExecutionManifestMilestone,
) -> tuple[bool, str]:
    if not entry.may_create_code:
        return False, "manifest milestone does not allow code creation"
    if not entry.owning_wr:
        return False, "manifest milestone has only a future WR candidate; run Track Expansion first"
    if step.next_action != "execute_next_wr_implementation_contract":
        return False, f"workflow next action is {step.next_action}, not implementation-contract execution"
    if step.gate_errors or step.evidence_errors:
        return False, "milestone gates or completion evidence must be repaired first"
    return (
        False,
        "manifest permits code only after task production:plan confirms the active WR contract; task ai:goal alone does not authorize implementation",
    )


def production_stack_track_order(planning: ProductionPlanningState, root_track: ProductionTrack) -> list[ProductionTrack]:
    milestone_to_track_id = {
        milestone.id: track.id for track in planning.tracks for milestone in track.milestones
    }
    tracks_by_id = {track.id: track for track in planning.tracks}
    production_order = {track.id: index for index, track in enumerate(planning.tracks)}
    milestone_by_id = planning.by_milestone_id
    relevant_track_ids: set[str] = set()
    visited_milestones: set[str] = set()
    visiting_milestones: set[str] = set()

    def collect_milestone(milestone_id: str) -> None:
        if milestone_id in visited_milestones:
            return
        if milestone_id in visiting_milestones:
            raise WorkflowError(f"{root_track.id}: milestone dependency cycle includes {milestone_id}")
        milestone = milestone_by_id.get(milestone_id)
        if milestone is None:
            raise WorkflowError(f"{root_track.id}: missing milestone dependency {milestone_id}")
        visiting_milestones.add(milestone_id)
        relevant_track_ids.add(milestone_to_track_id[milestone_id])
        for dependency in milestone.dependencies:
            collect_milestone(dependency)
        visiting_milestones.remove(milestone_id)
        visited_milestones.add(milestone_id)

    for milestone in root_track.milestones:
        collect_milestone(milestone.id)

    track_dependencies: dict[str, set[str]] = {track_id: set() for track_id in relevant_track_ids}
    for track_id in relevant_track_ids:
        for milestone in tracks_by_id[track_id].milestones:
            for dependency in milestone.dependencies:
                dependency_track_id = milestone_to_track_id.get(dependency)
                if dependency_track_id and dependency_track_id != track_id and dependency_track_id in relevant_track_ids:
                    track_dependencies[track_id].add(dependency_track_id)

    ordered_track_ids: list[str] = []
    visited_tracks: set[str] = set()
    visiting_tracks: set[str] = set()

    def visit_track(track_id: str) -> None:
        if track_id in visited_tracks:
            return
        if track_id in visiting_tracks:
            raise WorkflowError(f"{root_track.id}: track dependency cycle includes {track_id}")
        visiting_tracks.add(track_id)
        for dependency_track_id in sorted(track_dependencies[track_id], key=production_order.__getitem__):
            visit_track(dependency_track_id)
        visiting_tracks.remove(track_id)
        visited_tracks.add(track_id)
        ordered_track_ids.append(track_id)

    for track_id in sorted(relevant_track_ids, key=production_order.__getitem__):
        visit_track(track_id)

    return [tracks_by_id[track_id] for track_id in ordered_track_ids]


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
    if milestone.kind == "design" and milestone.state in {"active", "ready_next"} and not roadmap_actions:
        return "accept_design_or_record_design_evidence"
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
    production_source: Path = PRODUCTION_SOURCE,
    lock_source_root: Path = TRACK_EXECUTION_LOCK_ROOT,
    scope: GoalScope = GoalScope.full,
    manifest: LoadedTrackExecutionManifest | None = None,
) -> str:
    steps = build_goal_steps(planning, roadmap, track, roadmap_source=roadmap_source)
    manifest_context = (
        build_manifest_goal_context(
            planning,
            roadmap,
            track,
            steps,
            manifest,
            production_source=production_source,
            roadmap_source=roadmap_source,
            lock_source_root=lock_source_root,
        )
        if manifest is not None
        else None
    )
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
    lines.extend(render_manifest_gate(track, manifest_context))
    lines.extend(["", "## Ordered Milestone Plan", ""])
    for step in steps:
        manifest_entry = manifest_context.loaded.manifest.by_milestone_id.get(step.milestone.id) if manifest_context else None
        lines.extend(render_milestone_step(step, scope=scope, manifest_entry=manifest_entry))
    lines.extend(render_completion_gate(completion_errors, out_of_scope=out_of_scope, scope=scope))
    lines.extend(render_goal_prompt(track, steps, scope=scope, manifest_context=manifest_context))
    return "\n".join(lines)


def render_stack_goal(
    planning: ProductionPlanningState,
    roadmap: RoadmapState,
    root_track: ProductionTrack,
    *,
    roadmap_source: Path = ROADMAP_SOURCE,
    scope: GoalScope = GoalScope.full,
) -> str:
    stack_tracks = production_stack_track_order(planning, root_track)
    summaries = [
        TrackGoalSummary(
            track=track,
            steps=tuple(build_goal_steps(planning, roadmap, track, roadmap_source=roadmap_source)),
            completion_errors=tuple(
                track_completion_errors(planning, roadmap_source=roadmap_source, track=track, scope=scope)
            ),
        )
        for track in stack_tracks
    ]
    current_summary = next((summary for summary in summaries if summary.completion_errors), None)
    lines = [
        f"# Production Stack /goal Kickoff: {root_track.id}",
        "",
        f"Overall end goal: {root_track.title}",
        f"State: {root_track.state}",
        f"Owner: {root_track.owner}",
        f"Strategic goal: {root_track.strategic_goal}",
        "",
        "## Dependency Track Order",
        "",
    ]
    for summary in summaries:
        step = first_incomplete_or_evidence_step(summary.steps)
        if summary.completion_errors and step is not None:
            lines.append(
                f"- {summary.track.id} - {summary.track.title}: {step.milestone.id} -> {step.next_action}"
            )
        elif summary.completion_errors:
            lines.append(f"- {summary.track.id} - {summary.track.title}: completion evidence repair required")
        else:
            lines.append(f"- {summary.track.id} - {summary.track.title}: completed")
    lines.extend(["", "## Current Stack Driver", ""])
    if current_summary is None:
        lines.append("All stack tracks currently satisfy completion gates. Run the final production, roadmap, and planning checks before any completion-quality claim.")
    else:
        current_step = first_incomplete_or_evidence_step(current_summary.steps)
        lines.append(f"Track: {current_summary.track.id} - {current_summary.track.title}")
        if current_step is not None:
            lines.append(f"Milestone: {current_step.milestone.id} - {current_step.milestone.title}")
            lines.append(f"Next legal action: {current_step.next_action}")
        else:
            lines.append("Milestone: completion evidence repair required")
        lines.append(f"Single-track command: task ai:goal -- --track {current_summary.track.id}")
    lines.extend(["", "## Stack Completion Gate", ""])
    stack_completion_errors = [
        f"{summary.track.id}: {error}"
        for summary in summaries
        for error in summary.completion_errors
    ]
    if stack_completion_errors:
        lines.append("Stack completion is blocked until:")
        lines.extend(f"- {error}" for error in stack_completion_errors)
    else:
        lines.append("Stack completion metadata may be considered after final roadmap, production, docs, and planning gates pass.")
    lines.extend(render_stack_goal_prompt(root_track, summaries, current_summary=current_summary, scope=scope))
    return "\n".join(lines)


def first_incomplete_or_evidence_step(steps: tuple[MilestoneGoalStep, ...]) -> MilestoneGoalStep | None:
    return next(
        (
            step
            for step in steps
            if step.milestone.state != "completed" or step.evidence_errors or step.gate_errors
        ),
        None,
    )


def render_manifest_gate(
    track: ProductionTrack,
    context: ManifestGoalContext | None,
) -> list[str]:
    lines = ["", "## Track Execution Manifest Gate", ""]
    if context is None:
        if len(track.milestones) >= 6:
            expected_path = TRACK_EXECUTION_MANIFEST_ROOT / f"{track.id.lower()}.yaml"
            lines.extend(
                [
                    f"- Manifest source: not found at `{repo_path(expected_path)}`",
                    "- Warning: this is a long production track. Goal generation is falling back to production and roadmap metadata only.",
                    "- Stop condition: create a machine-readable Track Execution Manifest before relying on full-track `/goal` persistence.",
                    "",
                ]
            )
        else:
            lines.extend(["- Manifest source: none; fallback production/roadmap goal mode.", ""])
        return lines

    entry = context.current_entry
    manifest = context.loaded.manifest
    contract_pack_line = execution_contract_pack_line(track.id)
    lines.extend(
        [
            f"- Manifest source: `{repo_path(context.loaded.path)}`",
            contract_pack_line,
            f"- Authority level: {manifest.authority_level}",
            f"- Current milestone: `{entry.milestone_id}` - {entry.title}",
            f"- Manifest next legal action: {entry.next_legal_action}",
            f"- Workflow next action: {context.manifest_workflow_action}",
            f"- Implementation authorized now: {'yes' if context.implementation_authorized else 'no'} - {context.implementation_authorization_note}",
            f"- Must stop after this action: {'yes' if context.must_stop else 'no'}",
            "- Agent-track preparation-ready: yes",
            f"- AI executable declared: {'yes' if manifest.ai_executable else 'no'}",
            f"- Full automation target: {'yes' if context.full_automation_target else 'no'}",
            f"- Full automation readiness: {'ready' if context.full_automation_ready else 'blocked' if context.full_automation_target else 'not requested'}",
            f"- Execution lock: {repo_path(context.execution_lock_path) if context.execution_lock_path else 'missing'}",
            f"- Execution lock readiness: {'ready' if context.execution_lock_ready else 'blocked' if context.full_automation_target else 'not requested'}",
            f"- `--mode full-track` can run now: {'yes' if context.full_automation_ready and context.execution_lock_ready else 'no'}",
            f"- Locked-and-executable: {'yes' if context.full_automation_ready and context.execution_lock_ready and manifest.ai_executable else 'no'}",
        ]
    )
    truth_lines = truth_claim_summary_lines(manifest)
    lines.append("- Truth claims:")
    if len(truth_lines) == 1:
        lines.append(f"  - {truth_lines[0]}")
    else:
        lines.extend(f"  {line}" for line in truth_lines[1:])
    if context.full_automation_blockers:
        lines.append("- Full automation blockers:")
        lines.extend(f"  - {blocker}" for blocker in context.full_automation_blockers)
    if context.execution_lock_blockers:
        lines.append("- Execution lock blockers:")
        lines.extend(f"  - {blocker}" for blocker in context.execution_lock_blockers)
    if context.unmet_gates:
        lines.append("- Unmet gates:")
        lines.extend(f"  - {gate}" for gate in context.unmet_gates)
    else:
        lines.append("- Unmet gates: none detected by manifest-aware workflow checks.")
    lines.append("- Current milestone stop conditions:")
    lines.extend(f"  - {condition}" for condition in entry.stop_conditions)
    lines.append("- Global manifest stop conditions:")
    lines.extend(f"  - {condition}" for condition in manifest.global_stop_conditions)
    lines.append("")
    return lines


def execution_contract_pack_line(track_id: str) -> str:
    try:
        from execution.compiler import contract_pack_path, load_contract_pack
        from execution.locks import contract_pack_freshness_errors

        pack_path = contract_pack_path(track_id)
        pack = load_contract_pack(track_id)
    except WorkflowError as error:
        return f"- Execution Contract Pack: invalid ({error})"
    if pack is None:
        return "- Execution Contract Pack: missing; executable locked tracks cannot use legacy execution."
    freshness_errors = contract_pack_freshness_errors(pack)
    if freshness_errors:
        return f"- Execution Contract Pack: `{repo_path(pack_path)}` stale ({len(freshness_errors)} source digest blocker(s))"
    return f"- Execution Contract Pack: `{repo_path(pack_path)}` ({len(pack.actions)} remaining actions)"


def render_milestone_step(
    step: MilestoneGoalStep,
    *,
    scope: GoalScope = GoalScope.full,
    manifest_entry: TrackExecutionManifestMilestone | None = None,
) -> list[str]:
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
    if manifest_entry is not None:
        wr_authority = manifest_entry.owning_wr or f"future {manifest_entry.future_wr_candidate}"
        lines.extend(
            [
                f"- Manifest authority: {manifest_entry.authority_level}",
                f"- Manifest next legal action: {manifest_entry.next_legal_action}",
                f"- Manifest WR authority: {wr_authority}",
                f"- Manifest permissions: code={'yes' if manifest_entry.may_create_code else 'no'}, crates={'yes' if manifest_entry.may_create_crates else 'no'}, production behavior={'yes' if manifest_entry.may_modify_production_behavior else 'no'}",
                f"- Expected closeout path: {manifest_entry.expected_closeout_path}",
            ]
        )
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


def render_stack_goal_prompt(
    root_track: ProductionTrack,
    summaries: list[TrackGoalSummary],
    *,
    current_summary: TrackGoalSummary | None,
    scope: GoalScope,
) -> list[str]:
    if scope == GoalScope.non_deferred:
        scope_rule = "Preserve blocked or deferred milestones as explicit out-of-scope gaps for each stack track."
        completion_rule = (
            "- The bounded renderer stack is complete only when every in-scope milestone in every dependency track is completed with valid evidence gates and final production, roadmap, docs, and planning checks pass."
        )
    else:
        scope_rule = "Complete every milestone in every dependency track; do not preserve blocked or deferred gaps unless the source track says so."
        completion_rule = (
            "- The renderer stack is complete only when every milestone in every dependency track is completed with valid evidence gates and final production, roadmap, docs, and planning checks pass."
        )
    current_command = (
        f"task ai:goal -- --track {current_summary.track.id}"
        if current_summary is not None
        else f"task ai:goal -- --track {root_track.id} --stack"
    )
    lines = [
        "",
        "## Ready-to-paste /goal Prompt",
        "",
        "```text",
        f"/goal Complete the production stack ending in {root_track.id} - {root_track.title}.",
        "",
        "Use the current production tracks, roadmap items, design docs, and task workflow.",
        f"Use task ai:goal -- --track {root_track.id} --stack as the stack coordinator after every bounded action.",
        "Do not stop merely because the target track is waiting for dependency completion; resolve the first incomplete dependency track named by the stack coordinator.",
        "For each iteration, run the selected single-track command, perform exactly one legal next action, validate it, close it out, then rerun the stack coordinator.",
        "Do not bypass design gates, ADR gates, WR roadmap state, write scopes, validation, closeout evidence, or completion-quality rules.",
        scope_rule,
        "",
        "Dependency track order and current next actions:",
    ]
    for summary in summaries:
        step = first_incomplete_or_evidence_step(summary.steps)
        if summary.completion_errors and step is not None:
            lines.append(f"- {summary.track.id}: {step.milestone.id} -> {step.next_action}")
        elif summary.completion_errors:
            lines.append(f"- {summary.track.id}: completion_evidence_repair")
        else:
            lines.append(f"- {summary.track.id}: completed")
    lines.extend(
        [
            "",
            f"Current single-track command: {current_command}",
            "",
            "Coordinator rules:",
            "- Completed milestones: verify evidence gates and completion-quality claims before relying on them.",
            "- Designing, blocked, or deferred milestones: do design, ADR, or unblock work only; do not implement product code.",
            "- Active or ready_next milestones: use linked WR rows and task production:plan before code changes.",
            "- Cross-track dependency waits are routing signals in stack mode; switch to the named prerequisite track instead of treating the overall goal as blocked.",
            "- After a failed roadmap:promote or gate command, only repair exact metadata, run task roadmap:switch-current, or stop/report; do not investigate adjacent WR evidence.",
            "- Implement only one bounded WR slice or design-gate repair at a time.",
            "- After implementation, run focused validation, closeout or drift-check routines, roadmap render/validate/check, production render/validate/check, docs validation, and planning validation as applicable.",
            completion_rule,
            "",
            "Stop immediately if ownership is unclear, a gate is unmet, a WR row is not ready for its required action, validation fails, closeout evidence is missing, or source files changed enough that this command must be rerun.",
            "```",
            "",
        ]
    )
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


def render_goal_prompt(
    track: ProductionTrack,
    steps: list[MilestoneGoalStep],
    *,
    scope: GoalScope,
    manifest_context: ManifestGoalContext | None,
) -> list[str]:
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
        "Use the current production tracks, roadmap items, machine-readable Track Execution Manifest when present, design docs, and task workflow.",
        "Work through the finite milestone list in dependency order.",
        "For each milestone, perform exactly one legal next action, validate it, close it out, then rerun task ai:goal before continuing.",
        "Do not bypass design gates, ADR gates, WR roadmap state, write scopes, validation, closeout evidence, or completion-quality rules.",
        "",
    ]
    if manifest_context is not None:
        lines.extend(
            [
                f"Manifest source: {repo_path(manifest_context.loaded.path)}",
                f"Current manifest milestone: {manifest_context.current_entry.milestone_id}",
                f"Current manifest next legal action: {manifest_context.current_entry.next_legal_action}",
                f"Implementation authorized now: {'yes' if manifest_context.implementation_authorized else 'no'} - {manifest_context.implementation_authorization_note}",
                f"Agent-track preparation-ready: yes; full-automation-ready: {'yes' if manifest_context.full_automation_ready else 'no'}; locked-and-executable: {'yes' if manifest_context.full_automation_ready and manifest_context.execution_lock_ready and manifest_context.loaded.manifest.ai_executable else 'no'}",
                "Stop after the current legal action and rerun this command before crossing another milestone boundary.",
                "",
            ]
        )
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
    stack: bool = typer.Option(
        False,
        "--stack",
        help="Render a dependency-stack coordinator prompt that works prerequisite production tracks before the target track.",
    ),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(
        TRACK_EXECUTION_MANIFEST_ROOT,
        help="Machine-readable Track Execution Manifest source root.",
    ),
    lock_source_root: Path = typer.Option(
        TRACK_EXECUTION_LOCK_ROOT,
        help="Machine-readable Track Execution Lock source root.",
    ),
) -> None:
    try:
        planning = load_production_tracks(production_source)
        roadmap = load_roadmap(roadmap_source)
        production_track = find_track(planning, track)
        if stack:
            console.print(
                render_stack_goal(planning, roadmap, production_track, roadmap_source=roadmap_source, scope=scope),
                soft_wrap=True,
            )
            return
        manifest = load_track_execution_manifest(track, root=manifest_source_root)
        console.print(
            render_track_goal(
                planning,
                roadmap,
                production_track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                lock_source_root=lock_source_root,
                scope=scope,
                manifest=manifest,
            ),
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
