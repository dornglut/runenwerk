#!/usr/bin/env python3
"""
Machine-readable Track Execution Manifest contracts.

File: tools/workflow/track_execution_manifest.py
Module: track_execution_manifest
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from typing import Literal

import typer
import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator
from rich.console import Console

from production_plan import (
    ProductionPlanContext,
    classify_plan_action,
    default_contract_path,
    render_readiness_report,
    resolve_plan_context,
)
from production_state import PRODUCTION_SOURCE, ProductionMilestone, ProductionPlanningState, ProductionTrack, load_production_tracks
from roadmap_state import (
    REPO_ROOT,
    ROADMAP_SOURCE,
    RoadmapItem,
    RoadmapState,
    WorkflowError,
    combine_roadmap_data,
    document_frontmatter_status,
    load_yaml,
    load_roadmap,
    normalize_repo_path,
    normalized_write_scopes_with_generated_outputs,
    path_within_scope,
    repo_path,
    split_source_paths,
)


TRACK_EXECUTION_MANIFEST_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/track-execution-manifests"
UI_PROGRAM_ARCHITECTURE_PATH = REPO_ROOT / "docs-site/src/content/docs/design/active/ui-program-architecture.md"
UI_PROGRAM_CONTRACT_DESIGN_PATH = REPO_ROOT / "docs-site/src/content/docs/design/active/ui-program-contract-design.md"

ROADMAP_ID_PATTERN = re.compile(r"^WR-\d{3}$")
FUTURE_ROADMAP_ID_PATTERN = re.compile(r"^WR-TBD-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
GENERATED_SCOPE_PREFIXES = ("generated:", "derived:")

ManifestMilestoneType = Literal["docs_only", "design_only", "implementation", "hardening", "closeout"]
MANIFEST_RUNNER_PERMISSIONS = {
    "auto_safe",
    "agent_design",
    "agent_closeout",
    "product_code",
    "crate_creation",
    "foundation_extraction",
}

console = Console()
app = typer.Typer(no_args_is_help=True, help="Plan, inspect, and audit Track Execution Manifest sources.")

MANIFEST_AUDIT_CATEGORY_ORDER = (
    "alignment errors",
    "missing gates",
    "invalid blocked fields",
    "invalid closeout path",
    "WR scope mismatch",
    "missing WR authority",
    "other manifest audit blockers",
)


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


class IndentedSafeDumper(yaml.SafeDumper):
    def increase_indent(self, flow: bool = False, indentless: bool = False):
        return super().increase_indent(flow, False)


class ManifestDesignDependency(StrictModel):
    path: str
    required_status: str = "active"
    reason: str

    @field_validator("path")
    @classmethod
    def validate_path(cls, value: str) -> str:
        normalized = normalize_repo_path(value)
        if not normalized:
            raise ValueError("design dependency path must not be empty")
        return normalized

    @field_validator("required_status", "reason")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("design dependency text fields must not be empty")
        return cleaned


class ManifestAgentDesignContract(StrictModel):
    source_documents: list[str]
    required_sections: list[str]
    required_decisions: list[str]
    acceptance_checklist: list[str]

    @field_validator("source_documents")
    @classmethod
    def validate_source_documents(cls, value: list[str]) -> list[str]:
        cleaned = [normalize_repo_path(item) for item in value if item.strip()]
        if not cleaned:
            raise ValueError("agent_design source_documents must not be empty")
        return cleaned

    @field_validator("required_sections", "required_decisions", "acceptance_checklist")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("agent_design list fields must not be empty")
        return cleaned


class TrackExecutionManifestMilestone(StrictModel):
    milestone_id: str
    title: str
    stage: str
    authority_level: str
    milestone_type: ManifestMilestoneType
    owning_wr: str | None = None
    future_wr_candidate: str | None = None
    predecessor_dependencies: list[str] = Field(default_factory=list)
    write_scope: list[str]
    forbidden_scope: list[str]
    required_contracts: list[str]
    validation_commands: list[str]
    evidence_gates: list[str]
    expected_closeout_path: str
    stop_conditions: list[str]
    next_legal_action: str
    may_create_code: bool
    may_create_crates: bool
    may_modify_production_behavior: bool
    agent_design: ManifestAgentDesignContract | None = None

    @field_validator(
        "milestone_id",
        "title",
        "stage",
        "authority_level",
        "expected_closeout_path",
        "next_legal_action",
    )
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("manifest milestone text fields must not be empty")
        return cleaned

    @field_validator(
        "write_scope",
        "forbidden_scope",
        "required_contracts",
        "validation_commands",
        "evidence_gates",
        "stop_conditions",
    )
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("manifest milestone list fields must not be empty")
        return cleaned

    @field_validator("owning_wr")
    @classmethod
    def validate_owning_wr(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        if not ROADMAP_ID_PATTERN.fullmatch(cleaned):
            raise ValueError("owning_wr must match WR-000")
        return cleaned

    @field_validator("future_wr_candidate")
    @classmethod
    def validate_future_wr_candidate(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        if not FUTURE_ROADMAP_ID_PATTERN.fullmatch(cleaned):
            raise ValueError("future_wr_candidate must match WR-TBD-NAME")
        return cleaned

    @model_validator(mode="after")
    def validate_wr_authority(self) -> TrackExecutionManifestMilestone:
        if bool(self.owning_wr) == bool(self.future_wr_candidate):
            raise ValueError("manifest milestones must have exactly one owning_wr or future_wr_candidate")
        return self


class TrackExecutionManifest(StrictModel):
    version: int
    track_id: str
    authority_level: str
    accepted_design_dependencies: list[ManifestDesignDependency]
    global_forbidden_scope: list[str]
    global_validation_commands: list[str]
    global_stop_conditions: list[str]
    next_legal_action: str
    milestones: list[TrackExecutionManifestMilestone]

    @field_validator("track_id", "authority_level", "next_legal_action")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("manifest text fields must not be empty")
        return cleaned

    @field_validator("global_forbidden_scope", "global_validation_commands", "global_stop_conditions")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("manifest list fields must not be empty")
        return cleaned

    @model_validator(mode="after")
    def validate_unique_milestones(self) -> TrackExecutionManifest:
        milestone_ids = [milestone.milestone_id for milestone in self.milestones]
        duplicates = sorted({milestone_id for milestone_id in milestone_ids if milestone_ids.count(milestone_id) > 1})
        if duplicates:
            raise ValueError(f"duplicate manifest milestone ids: {', '.join(duplicates)}")
        return self

    @property
    def by_milestone_id(self) -> dict[str, TrackExecutionManifestMilestone]:
        return {milestone.milestone_id: milestone for milestone in self.milestones}


@dataclass(frozen=True)
class LoadedTrackExecutionManifest:
    manifest: TrackExecutionManifest
    path: Path


def manifest_source_path(track_id: str, root: Path = TRACK_EXECUTION_MANIFEST_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


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


def load_track_execution_manifest(
    track_id: str,
    *,
    root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> LoadedTrackExecutionManifest | None:
    path = manifest_source_path(track_id, root=root)
    if not path.exists():
        return None
    try:
        with path.open("r", encoding="utf-8") as source:
            data = yaml.safe_load(source)
    except yaml.YAMLError as error:
        raise WorkflowError(f"{repo_path(path)} contains malformed YAML: {error}") from error
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    try:
        manifest = TrackExecutionManifest.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid Track Execution Manifest: {error}") from error
    if manifest.track_id != track_id:
        raise WorkflowError(f"{repo_path(path)} declares track_id={manifest.track_id}, expected {track_id}")
    return LoadedTrackExecutionManifest(manifest=manifest, path=path)


def manifest_alignment_errors(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
    ordered_milestone_ids: list[str],
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    manifest = loaded.manifest
    errors: list[str] = []
    if manifest.track_id != track.id:
        errors.append(f"{repo_path(loaded.path)}: track_id {manifest.track_id} does not match production track {track.id}")

    manifest_milestone_ids = [entry.milestone_id for entry in manifest.milestones]
    if manifest_milestone_ids != ordered_milestone_ids:
        errors.append(
            f"{track.id}: manifest milestone order {manifest_milestone_ids} "
            f"does not match production dependency order {ordered_milestone_ids}"
        )

    errors.extend(manifest_design_dependency_errors(manifest, repo_root=repo_root))

    production_by_id = {milestone.id: milestone for milestone in track.milestones}
    for entry in manifest.milestones:
        milestone = production_by_id.get(entry.milestone_id)
        if milestone is None:
            errors.append(f"{entry.milestone_id}: manifest milestone is not present in production track {track.id}")
            continue
        if entry.title != milestone.title:
            errors.append(
                f"{entry.milestone_id}: manifest title {entry.title!r} "
                f"does not match production title {milestone.title!r}"
            )
        if entry.predecessor_dependencies != milestone.dependencies:
            errors.append(
                f"{entry.milestone_id}: manifest dependencies {entry.predecessor_dependencies} "
                f"do not match production dependencies {milestone.dependencies}"
            )
        expected_type_errors = manifest_type_errors(entry, milestone_kind=milestone.kind)
        errors.extend(f"{entry.milestone_id}: {error}" for error in expected_type_errors)
        if entry.owning_wr:
            if milestone.roadmap_links != [entry.owning_wr]:
                errors.append(
                    f"{entry.milestone_id}: manifest owning_wr {entry.owning_wr} "
                    f"does not match production roadmap_links {milestone.roadmap_links}"
                )
            roadmap_item = roadmap.by_id.get(entry.owning_wr)
            if roadmap_item is None:
                errors.append(f"{entry.milestone_id}: manifest owning_wr {entry.owning_wr} is not present in roadmap")
            else:
                errors.extend(manifest_write_scope_coverage_errors(entry, roadmap_item))
        if entry.future_wr_candidate and milestone.roadmap_links:
            errors.append(
                f"{entry.milestone_id}: manifest future_wr_candidate {entry.future_wr_candidate} "
                f"conflicts with production roadmap_links {milestone.roadmap_links}"
            )
    return errors


def manifest_write_scope_coverage_errors(
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
) -> list[str]:
    errors: list[str] = []
    wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
    for scope in entry.write_scope:
        if is_generated_or_derived_scope(scope):
            continue
        if mentions_generated_or_derived_scope(scope):
            errors.append(
                f"{entry.milestone_id}: manifest write_scope {scope!r} must use "
                "'generated:' or 'derived:' when it names generated/derived output"
            )
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            continue
        if not any(path_within_scope(normalized, wr_scope) for wr_scope in wr_scopes):
            errors.append(
                f"{entry.milestone_id}: manifest write_scope {normalized} is not covered by "
                f"owning WR {roadmap_item.id} write_scopes"
            )
    return errors


def is_generated_or_derived_scope(scope: str) -> bool:
    cleaned = scope.strip().lower()
    return cleaned.startswith(GENERATED_SCOPE_PREFIXES)


def mentions_generated_or_derived_scope(scope: str) -> bool:
    cleaned = scope.strip().lower()
    return "generated" in cleaned or "derived" in cleaned


def manifest_write_scope_path(scope: str) -> str | None:
    normalized = normalize_repo_path(scope)
    if not normalized or normalized.startswith("blocked:") or " " in normalized:
        return None
    if "/" not in normalized:
        return None
    return normalized


def manifest_design_dependency_errors(
    manifest: TrackExecutionManifest,
    *,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    errors: list[str] = []
    for dependency in manifest.accepted_design_dependencies:
        candidate = repo_root / dependency.path
        if not candidate.exists():
            errors.append(f"{manifest.track_id}: manifest design dependency missing {dependency.path} ({dependency.reason})")
            continue
        status = document_frontmatter_status(candidate)
        if status is None:
            errors.append(
                f"{manifest.track_id}: manifest design dependency {dependency.path} has no frontmatter status "
                f"({dependency.reason})"
            )
        elif status.lower() != dependency.required_status.lower():
            errors.append(
                f"{manifest.track_id}: manifest design dependency {dependency.path} status {status!r} "
                f"does not match required {dependency.required_status!r} ({dependency.reason})"
            )
    return errors


def manifest_type_errors(entry: TrackExecutionManifestMilestone, *, milestone_kind: str) -> list[str]:
    if milestone_kind == "design" and entry.milestone_type not in {"docs_only", "design_only"}:
        return [f"manifest milestone_type {entry.milestone_type!r} does not match design milestone kind"]
    if milestone_kind == "implementation" and entry.milestone_type != "implementation":
        return [f"manifest milestone_type {entry.milestone_type!r} does not match implementation milestone kind"]
    if milestone_kind == "hardening" and entry.milestone_type != "hardening":
        return [f"manifest milestone_type {entry.milestone_type!r} does not match hardening milestone kind"]
    if milestone_kind == "release" and entry.milestone_type != "closeout":
        return [f"manifest milestone_type {entry.milestone_type!r} does not match release milestone kind"]
    return []


def audit_manifest(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
) -> list[str]:
    errors = manifest_alignment_errors(
        loaded,
        track=track,
        roadmap=roadmap,
        ordered_milestone_ids=[milestone.id for milestone in ordered_track_milestones(track)],
    )
    manifest = loaded.manifest
    if not manifest.accepted_design_dependencies:
        errors.append(f"{manifest.track_id}: manifest must list accepted design dependencies")
    for value in manifest.global_forbidden_scope + manifest.global_validation_commands + manifest.global_stop_conditions:
        if value.startswith("blocked:"):
            errors.append(f"{manifest.track_id}: global manifest field remains blocked: {value}")
    for entry in manifest.milestones:
        errors.extend(audit_manifest_milestone(entry))
    return errors


def manifest_audit_error_category(error: str) -> str:
    if "manifest write_scope" in error:
        return "WR scope mismatch"
    if "remains blocked" in error:
        return "invalid blocked fields"
    if "expected_closeout_path" in error or "closeout/report" in error:
        return "invalid closeout path"
    if "design dependency" in error or "gate" in error:
        return "missing gates"
    if (
        "does not match" in error
        or "conflicts with production" in error
        or "not present in production track" in error
        or "manifest milestone order" in error
        or "manifest title" in error
        or "manifest dependencies" in error
    ):
        return "alignment errors"
    if "owning_wr" in error or "future_wr_candidate" in error or "owning WR" in error:
        return "missing WR authority"
    return "other manifest audit blockers"


def grouped_manifest_audit_errors(errors: list[str]) -> dict[str, list[str]]:
    grouped = {category: [] for category in MANIFEST_AUDIT_CATEGORY_ORDER}
    for error in errors:
        grouped[manifest_audit_error_category(error)].append(error)
    return {category: grouped[category] for category in MANIFEST_AUDIT_CATEGORY_ORDER if grouped[category]}


def manifest_audit_blocker_lines(errors: list[str]) -> list[str]:
    lines = ["Track Execution Manifest audit blockers:"]
    for category, category_errors in grouped_manifest_audit_errors(errors).items():
        lines.append(f"{category}:")
        lines.extend(f"- {error}" for error in category_errors)
    return lines


def audit_manifest_or_raise(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
) -> None:
    errors = audit_manifest(loaded, track=track, roadmap=roadmap)
    if errors:
        raise WorkflowError("\n".join(manifest_audit_blocker_lines(errors)))


def print_manifest_audit_blockers(errors: list[str]) -> None:
    lines = manifest_audit_blocker_lines(errors)
    console.print(f"[red]{lines[0]}[/red]")
    for line in lines[1:]:
        console.print(line)


def audit_manifest_milestone(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    for field_name in (
        "write_scope",
        "forbidden_scope",
        "required_contracts",
        "validation_commands",
        "evidence_gates",
        "stop_conditions",
    ):
        values = getattr(entry, field_name)
        for value in values:
            if value.startswith("blocked:"):
                errors.append(f"{entry.milestone_id}: {field_name} remains blocked: {value}")
    if entry.expected_closeout_path.startswith("blocked:"):
        errors.append(f"{entry.milestone_id}: expected_closeout_path remains blocked")
    if not entry.expected_closeout_path.endswith(".md"):
        errors.append(f"{entry.milestone_id}: expected_closeout_path must point at a Markdown closeout/report")
    return errors


def first_current_manifest_entry(
    manifest: TrackExecutionManifest,
    track: ProductionTrack,
) -> tuple[TrackExecutionManifestMilestone, ProductionMilestone]:
    manifest_by_id = manifest.by_milestone_id
    ordered = ordered_track_milestones(track)
    for milestone in ordered:
        entry = manifest_by_id[milestone.id]
        if milestone.state != "completed":
            return entry, milestone
    last_milestone = ordered[-1]
    return manifest_by_id[last_milestone.id], last_milestone


def next_action_blockers(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    planning: ProductionPlanningState,
    track: ProductionTrack,
    roadmap: RoadmapState,
) -> tuple[str, list[str]]:
    production_by_id = {candidate.id: candidate for candidate in track.milestones}
    blockers: list[str] = []
    for dependency in entry.predecessor_dependencies:
        dependency_state = production_by_id.get(dependency).state if dependency in production_by_id else "missing"
        if dependency_state != "completed":
            blockers.append(f"{entry.milestone_id}: dependency {dependency} is {dependency_state}, expected completed")
    if entry.future_wr_candidate:
        blockers.append(f"{entry.milestone_id}: Track Expansion must create or link {entry.future_wr_candidate}")
        return "track_expansion_required", blockers
    if not entry.owning_wr:
        blockers.append(f"{entry.milestone_id}: missing owning WR")
        return "missing_wr_authority", blockers
    item = roadmap.by_id.get(entry.owning_wr)
    if item is None:
        blockers.append(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
        return "missing_wr_authority", blockers
    action = classify_plan_action(
        ProductionPlanContext(
            planning=planning,
            roadmap=roadmap,
            track=track,
            milestone=milestone,
            roadmap_item=item,
        )
    )
    if entry.milestone_type in {"docs_only", "design_only"} and action == "design_first":
        return action, blockers
    if action != "write_implementation_contract":
        blockers.append(f"{item.id}: workflow action is {action} (state={item.planning_state}, blocker={item.blocker_label})")
    return action, blockers


def implementation_authorization_note(
    entry: TrackExecutionManifestMilestone,
    workflow_action: str,
    blockers: list[str],
) -> str:
    if not entry.may_create_code:
        return "no - manifest milestone does not allow code creation"
    if blockers:
        return "no - blockers must be cleared first"
    if workflow_action != "write_implementation_contract":
        return f"no - workflow action is {workflow_action}"
    return "no - task production:next is read-only; run task production:plan for the active WR contract before code"


def future_wr_candidate_for_milestone(milestone: ProductionMilestone) -> str:
    suffix = milestone.id.removeprefix("PM-")
    return f"WR-TBD-{suffix}"


def manifest_type_for_milestone(milestone: ProductionMilestone) -> ManifestMilestoneType:
    if milestone.kind == "implementation":
        return "implementation"
    if milestone.kind == "hardening":
        return "hardening"
    if milestone.kind == "release":
        return "closeout"
    return "design_only"


def build_manifest_scaffold(track: ProductionTrack, roadmap: RoadmapState) -> TrackExecutionManifest:
    design_dependencies: dict[str, ManifestDesignDependency] = {}
    for milestone in track.milestones:
        for gate in milestone.design_gates:
            design_dependencies[gate.path] = ManifestDesignDependency(
                path=gate.path,
                required_status=gate.required_status,
                reason=gate.reason,
            )
    if not design_dependencies:
        design_dependencies["blocked: define accepted design dependency"] = ManifestDesignDependency(
            path="blocked: define accepted design dependency",
            required_status="active",
            reason="Track manifest scaffold requires an accepted design dependency before goal execution.",
        )

    global_validation_commands = [
        "task production:render",
        "task production:validate",
        "task production:check",
        "task roadmap:render",
        "task roadmap:validate",
        "task roadmap:check",
        "task docs:validate",
        "task planning:validate",
    ]
    milestones: list[TrackExecutionManifestMilestone] = []
    for milestone in ordered_track_milestones(track):
        if len(milestone.roadmap_links) > 1:
            raise WorkflowError(f"{milestone.id}: cannot scaffold manifest for multiple roadmap links")
        owning_wr = milestone.roadmap_links[0] if milestone.roadmap_links else None
        future_wr = None if owning_wr else future_wr_candidate_for_milestone(milestone)
        roadmap_item: RoadmapItem | None = roadmap.by_id.get(owning_wr) if owning_wr else None
        write_scope = roadmap_item.write_scopes if roadmap_item and roadmap_item.write_scopes else [
            f"blocked: define exact write scope for {milestone.id}"
        ]
        validation_commands = roadmap_item.validations if roadmap_item and roadmap_item.validations else global_validation_commands
        milestones.append(
            TrackExecutionManifestMilestone(
                milestone_id=milestone.id,
                title=milestone.title,
                stage=f"blocked: assign stage for {milestone.id}",
                authority_level="planning_and_sequencing_only",
                milestone_type=manifest_type_for_milestone(milestone),
                owning_wr=owning_wr,
                future_wr_candidate=future_wr,
                predecessor_dependencies=milestone.dependencies,
                write_scope=write_scope,
                forbidden_scope=["no implementation from this manifest alone", "no crate creation without separate authority"],
                required_contracts=[f"blocked: define required contract for {milestone.id}"],
                validation_commands=validation_commands,
                evidence_gates=[f"blocked: define evidence gate for {milestone.id}"],
                expected_closeout_path=(
                    "docs-site/src/content/docs/reports/closeouts/"
                    f"{milestone.id.lower()}-{slugify(milestone.title)}/closeout.md"
                ),
                stop_conditions=[
                    "stop if validation fails",
                    "stop if WR authority is missing",
                    "stop before implementation unless a production plan authorizes a bounded slice",
                ],
                next_legal_action=(
                    f"Use owning WR {owning_wr} to plan the next bounded action."
                    if owning_wr
                    else f"Track Expansion must create or link {future_wr}."
                ),
                may_create_code=False,
                may_create_crates=False,
                may_modify_production_behavior=False,
            )
        )
    return TrackExecutionManifest(
        version=1,
        track_id=track.id,
        authority_level="planning_and_sequencing_only",
        accepted_design_dependencies=list(design_dependencies.values()),
        global_forbidden_scope=[
            "no product code from this manifest alone",
            "no new crates from this manifest alone",
            "no production behavior changes from this manifest alone",
        ],
        global_validation_commands=global_validation_commands,
        global_stop_conditions=[
            "stop if manifest data conflicts with production track or roadmap state",
            "stop if a milestone lacks WR authority",
            "stop after one legal action and rerun task ai:goal",
        ],
        next_legal_action="blocked: review generated scaffold and replace blocked fields before relying on full-track execution",
        milestones=milestones,
    )


def slugify(value: str) -> str:
    cleaned = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return cleaned or "track-execution-manifest"


def write_manifest(path: Path, manifest: TrackExecutionManifest) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data = manifest.model_dump(exclude_none=True, mode="json")
    write_yaml_mapping(path, data)


def write_yaml_mapping(path: Path, data: dict, *, indent_sequences: bool = True) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    dumper = IndentedSafeDumper if indent_sequences else yaml.SafeDumper
    path.write_text(
        yaml.dump(data, Dumper=dumper, sort_keys=False, allow_unicode=False, width=4096),
        encoding="utf-8",
        newline="\n",
    )


@dataclass(frozen=True)
class ManifestCommandContext:
    planning: ProductionPlanningState
    roadmap: RoadmapState
    track: ProductionTrack
    loaded: LoadedTrackExecutionManifest


@dataclass(frozen=True)
class AutoSafeExpansionResult:
    track_id: str
    milestone_id: str
    wr_id: str
    manifest_path: Path
    production_source: Path
    roadmap_deferred_source: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class AgentDesignResult:
    track_id: str
    milestone_id: str
    wr_id: str
    plan_path: Path
    design_paths: tuple[Path, ...]
    manifest_path: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class AgentCloseoutResult:
    track_id: str
    milestone_id: str
    wr_id: str
    closeout_path: Path
    manifest_path: Path
    production_source: Path
    roadmap_archive_source: Path
    roadmap_deferred_source: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class ProductCodeResult:
    track_id: str
    milestone_id: str
    wr_id: str
    plan_path: Path
    manifest_path: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


def resolve_manifest_command_context(
    track_id: str,
    *,
    production_source: Path,
    roadmap_source: Path,
    manifest_source_root: Path,
) -> ManifestCommandContext:
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    track = find_track(planning, track_id)
    loaded = load_track_execution_manifest(track_id, root=manifest_source_root)
    if loaded is None:
        raise WorkflowError(
            f"{track_id}: no Track Execution Manifest source at {repo_path(manifest_source_path(track_id, root=manifest_source_root))}"
        )
    return ManifestCommandContext(planning=planning, roadmap=roadmap, track=track, loaded=loaded)


def allocate_next_wr_id(roadmap: RoadmapState) -> str:
    existing_numbers = [int(item.id.split("-")[1]) for item in roadmap.items if ROADMAP_ID_PATTERN.fullmatch(item.id)]
    if not existing_numbers:
        return "WR-001"
    return f"WR-{max(existing_numbers) + 1:03d}"


def manifest_report_path(track_id: str) -> str:
    return f"docs-site/src/content/docs/reports/track-execution-manifests/{track_id.lower()}/manifest.md"


def production_generated_scopes() -> list[str]:
    return [
        "generated: production docs from task production:render",
        "generated: roadmap docs from task roadmap:render",
    ]


def implementation_plan_path(wr_id: str, milestone: ProductionMilestone) -> str:
    return (
        "docs-site/src/content/docs/reports/implementation-plans/"
        f"{wr_id.lower()}-{slugify(milestone.title)}/plan.md"
    )


def exact_auto_safe_write_scope(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track_id: str,
    wr_id: str,
    manifest_source: Path | None = None,
    deferred_source: Path | None = None,
) -> list[str]:
    scopes = [
        "docs-site/src/content/docs/workspace/production-tracks.yaml",
        "docs-site/src/content/docs/workspace/roadmap-deferred.yaml",
        f"docs-site/src/content/docs/workspace/track-execution-manifests/{track_id.lower()}.yaml",
        manifest_report_path(track_id),
        implementation_plan_path(wr_id, milestone),
        entry.expected_closeout_path,
        *production_generated_scopes(),
    ]
    if entry.milestone_type in {"docs_only", "design_only"}:
        scopes.extend(
            [
                repo_path(UI_PROGRAM_ARCHITECTURE_PATH),
                repo_path(UI_PROGRAM_CONTRACT_DESIGN_PATH),
            ]
        )
    if manifest_source is not None:
        scopes.append(repo_path(manifest_source))
    if deferred_source is not None:
        scopes.append(repo_path(deferred_source))
    return list(dict.fromkeys(scopes))


def predecessor_wr_dependencies(
    entry: TrackExecutionManifestMilestone,
    track: ProductionTrack,
) -> list[str]:
    by_milestone_id = {milestone.id: milestone for milestone in track.milestones}
    dependencies: list[str] = []
    for dependency in entry.predecessor_dependencies:
        dependency_milestone = by_milestone_id.get(dependency)
        if dependency_milestone is None:
            continue
        dependencies.extend(dependency_milestone.roadmap_links)
    return list(dict.fromkeys(dependencies))


def decision_gates_for_milestone(milestone: ProductionMilestone) -> list[dict]:
    return [
        {
            "kind": gate.kind,
            "path": gate.path,
            "required_status": gate.required_status,
            "applies_to": "discovery",
            "reason": gate.reason,
        }
        for gate in milestone.design_gates
    ]


def roadmap_row_for_auto_safe_expansion(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track: ProductionTrack,
    wr_id: str,
    manifest_source: Path | None = None,
    deferred_source: Path | None = None,
) -> dict:
    write_scope = exact_auto_safe_write_scope(
        entry,
        milestone,
        track_id=track.id,
        wr_id=wr_id,
        manifest_source=manifest_source,
        deferred_source=deferred_source,
    )
    return {
        "id": wr_id,
        "title": milestone.title,
        "diagram_title": milestone.title[:48],
        "alias": wr_id.replace("-", ""),
        "lane": "Product planning",
        "dependency_level": "L4",
        "gate": "Design-only Track Expansion gate" if entry.milestone_type in {"docs_only", "design_only"} else "Track Expansion gate",
        "planning_state": "blocked_deferred",
        "priority": "P2",
        "value": 4,
        "blocker": 4,
        "tc": 2,
        "rr_oe": 2,
        "du": 2,
        "effort": 5,
        "confidence": 0.5,
        "expected_score": 1.0,
        "rice": "N/A",
        "kano": "Basic",
        "dependencies": predecessor_wr_dependencies(entry, track),
        "write_scopes": write_scope,
        "validations": entry.validation_commands,
        "next_evidence": (
            f"{entry.milestone_id} requires a dedicated production plan and closeout evidence before completion."
        ),
        "current_decision": (
            f"Auto-safe Track Expansion created this deferred WR from {track.id} manifest data only; "
            "it authorizes planning metadata, not design authoring or implementation."
        ),
        "current_call": (
            f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
            "stop before design content or code until that contract is accepted."
        ),
        "first_move": f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}.",
        "main_blocker": "Dedicated production plan and closeout evidence are still missing.",
        "why_not_ready": "Track Expansion linked WR authority mechanically; the milestone work itself has not started.",
        "completion_quality": "not_applicable",
        "known_quality_gaps": [],
        "completion_audit": "",
        "diagram_call": ["track expansion", "no implementation"],
        "decision_gates": decision_gates_for_milestone(milestone),
        "ddd_owner": "domain/ui owns UiProgram contract design; workspace governance coordinates production sequencing.",
        "adr_requirement": "No new ADR unless the UiProgram contract changes accepted ownership, dependency direction, or extraction gates.",
        "fitness_function_requirement": "Docs, production, roadmap, and planning validation before closeout.",
        "ownership_mode": "Stream-aligned UI proving-domain planning with governance-owned sequencing.",
    }


def assert_auto_safe_expansion_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    roadmap: RoadmapState,
    allow: set[str],
) -> None:
    unsupported = sorted(allow - {"auto_safe"})
    if unsupported:
        raise WorkflowError(f"Manifest Runner V1 does not support permissions: {', '.join(unsupported)}")
    if "auto_safe" not in allow:
        raise WorkflowError("Manifest Runner requires --allow auto_safe for mechanical Track Expansion")
    if milestone.state == "completed":
        raise WorkflowError(f"{entry.milestone_id}: completed milestones must not be mutated by Track Expansion")
    if milestone.roadmap_links:
        raise WorkflowError(f"{entry.milestone_id}: production milestone already links WR rows {milestone.roadmap_links}")
    if not entry.future_wr_candidate:
        raise WorkflowError(f"{entry.milestone_id}: no future WR candidate is available for Track Expansion")
    if entry.may_create_code or entry.may_create_crates or entry.may_modify_production_behavior:
        raise WorkflowError(
            f"{entry.milestone_id}: auto_safe cannot expand milestones that allow code, crate creation, "
            "or production behavior changes"
        )
    if entry.milestone_type not in {"docs_only", "design_only"}:
        raise WorkflowError(f"{entry.milestone_id}: auto_safe expansion only supports docs-only or design-only milestones in V1")
    if roadmap.by_id.get(entry.future_wr_candidate):
        raise WorkflowError(f"{entry.future_wr_candidate}: future WR candidate unexpectedly exists as a concrete WR")


def updated_production_data_with_wr(
    production_source: Path,
    *,
    track_id: str,
    milestone_id: str,
    wr_id: str,
) -> dict:
    data = load_yaml(production_source)
    changed = False
    for track_data in data.get("tracks", []):
        if track_data.get("id") != track_id:
            continue
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") == milestone_id:
                milestone_data["roadmap_links"] = [wr_id]
                changed = True
                break
    if not changed:
        raise WorkflowError(f"{milestone_id}: not found in production source {repo_path(production_source)}")
    return data


def updated_manifest_data_with_wr(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    wr_id: str,
    deferred_source: Path | None = None,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    changed = False
    for milestone_data in data["milestones"]:
        if milestone_data["milestone_id"] != entry.milestone_id:
            continue
        milestone_data.pop("future_wr_candidate", None)
        milestone_data["owning_wr"] = wr_id
        milestone_data["write_scope"] = exact_auto_safe_write_scope(
            entry,
            milestone,
            track_id=loaded.manifest.track_id,
            wr_id=wr_id,
            manifest_source=loaded.path,
            deferred_source=deferred_source,
        )
        milestone_data["next_legal_action"] = (
            f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
            "stop before design authoring or implementation until that contract is accepted."
        )
        changed = True
        break
    if not changed:
        raise WorkflowError(f"{entry.milestone_id}: not found in manifest {repo_path(loaded.path)}")
    data["next_legal_action"] = (
        f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
        "do not skip to PM-UI-PROGRAM-007 / 6A."
    )
    return data


def updated_deferred_roadmap_data(
    roadmap_source: Path,
    *,
    roadmap: RoadmapState,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    track: ProductionTrack,
    wr_id: str,
    manifest_source: Path,
) -> tuple[Path, dict]:
    active_data = load_yaml(roadmap_source)
    _, deferred_source = split_source_paths(roadmap_source)
    if deferred_source.exists():
        deferred_data = load_yaml(deferred_source)
    else:
        deferred_data = empty_split_source_like(active_data)
    if any(item.get("id") == wr_id for item in deferred_data.get("items", [])):
        raise WorkflowError(f"{wr_id}: already present in deferred roadmap source")
    if roadmap.by_id.get(wr_id):
        raise WorkflowError(f"{wr_id}: already present in combined roadmap state")
    deferred_data.setdefault("items", []).append(
        roadmap_row_for_auto_safe_expansion(
            entry,
            milestone,
            track=track,
            wr_id=wr_id,
            manifest_source=manifest_source,
            deferred_source=deferred_source,
        )
    )
    return deferred_source, deferred_data


def empty_split_source_like(active_data: dict) -> dict:
    return {
        "version": active_data.get("version", 1),
        "roadmap": active_data.get("roadmap", {}),
        "items": [],
    }


def validate_auto_safe_expansion_data(
    *,
    production_data: dict,
    manifest_data: dict,
    active_roadmap_data: dict,
    archive_roadmap_data: dict | None,
    deferred_roadmap_data: dict,
    production_source: Path,
    roadmap_source: Path,
    manifest_path: Path,
    track_id: str,
) -> tuple[ProductionPlanningState, RoadmapState, LoadedTrackExecutionManifest]:
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            active_roadmap_data,
            roadmap_source,
            archive_data=archive_roadmap_data,
            deferred_data=deferred_roadmap_data,
        )
    )
    manifest = TrackExecutionManifest.model_validate(manifest_data)
    loaded = LoadedTrackExecutionManifest(manifest=manifest, path=manifest_path)
    track = find_track(planning, track_id)
    audit_manifest_or_raise(loaded, track=track, roadmap=roadmap)
    return planning, roadmap, loaded


def run_validation_commands(commands: list[str], *, cwd: Path = REPO_ROOT) -> tuple[str, ...]:
    outputs: list[str] = []
    for command in commands:
        completed = subprocess.run(command, cwd=cwd, shell=True, text=True, capture_output=True)
        combined = "\n".join(part for part in (completed.stdout.strip(), completed.stderr.strip()) if part)
        outputs.append(f"{command}: exit {completed.returncode}")
        if completed.returncode != 0:
            detail = f"\n{combined}" if combined else ""
            raise WorkflowError(f"validation command failed: {command}{detail}")
    return tuple(outputs)


def auto_safe_validation_commands() -> list[str]:
    return [
        "task production:render",
        "task roadmap:render",
        "task production:validate",
        "task roadmap:validate",
        "task production:check",
        "task roadmap:check",
        "task docs:validate",
        "task planning:validate",
    ]


def apply_auto_safe_track_expansion(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    run_validations: bool = True,
) -> AutoSafeExpansionResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if workflow_action != "track_expansion_required":
        raise WorkflowError(
            f"{entry.milestone_id}: next legal action is {workflow_action}, not auto_safe Track Expansion"
        )
    dependency_blockers = [blocker for blocker in blockers if "Track Expansion must create or link" not in blocker]
    if dependency_blockers:
        raise WorkflowError("\n".join(dependency_blockers))
    assert_auto_safe_expansion_allowed(entry, milestone, roadmap=context.roadmap, allow=allow)

    wr_id = allocate_next_wr_id(context.roadmap)
    production_data = updated_production_data_with_wr(
        production_source,
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=wr_id,
    )
    deferred_source, deferred_data = updated_deferred_roadmap_data(
        roadmap_source,
        roadmap=context.roadmap,
        entry=entry,
        milestone=milestone,
        track=context.track,
        wr_id=wr_id,
        manifest_source=context.loaded.path,
    )
    manifest_data = updated_manifest_data_with_wr(
        context.loaded,
        entry=entry,
        milestone=milestone,
        wr_id=wr_id,
        deferred_source=deferred_source,
    )
    active_data = load_yaml(roadmap_source)
    archive_source, _ = split_source_paths(roadmap_source)
    archive_data = load_yaml(archive_source) if archive_source.exists() else None
    validate_auto_safe_expansion_data(
        production_data=production_data,
        manifest_data=manifest_data,
        active_roadmap_data=active_data,
        archive_roadmap_data=archive_data,
        deferred_roadmap_data=deferred_data,
        production_source=production_source,
        roadmap_source=roadmap_source,
        manifest_path=context.loaded.path,
        track_id=context.track.id,
    )

    write_yaml_mapping(production_source, production_data)
    write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)
    write_yaml_mapping(context.loaded.path, manifest_data)
    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    next_legal_action = (
        f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
        "do not start design authoring until that plan is accepted."
    )
    return AutoSafeExpansionResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=wr_id,
        manifest_path=context.loaded.path,
        production_source=production_source,
        roadmap_deferred_source=deferred_source,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
    )


def manifest_write_scopes_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    return [
        normalized
        for normalized in (manifest_write_scope_path(scope) for scope in entry.write_scope)
        if normalized is not None
    ]


def path_is_covered_by_scope(path: str, scopes: list[str]) -> bool:
    normalized = normalize_repo_path(path)
    return any(path_within_scope(normalized, scope) for scope in scopes)


def assert_agent_design_write_scope(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    write_paths: list[str],
) -> None:
    assert_runner_write_scope(
        entry=entry,
        roadmap_item=roadmap_item,
        write_paths=write_paths,
        action_label="agent_design",
    )


def assert_runner_write_scope(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    write_paths: list[str],
    action_label: str,
) -> None:
    manifest_scopes = manifest_write_scopes_for_entry(entry)
    wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
    missing_manifest = [path for path in write_paths if not path_is_covered_by_scope(path, manifest_scopes)]
    missing_wr = [path for path in write_paths if not path_is_covered_by_scope(path, wr_scopes)]
    if missing_manifest:
        raise WorkflowError(
            f"{entry.milestone_id}: {action_label} write paths are not covered by manifest write_scope: "
            + ", ".join(missing_manifest)
        )
    if missing_wr:
        raise WorkflowError(
            f"{entry.milestone_id}: {action_label} write paths are not covered by owning WR {roadmap_item.id} write_scopes: "
            + ", ".join(missing_wr)
        )


PRODUCT_CODE_REQUIRED_PLAN_MARKERS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("exact files/modules allowed", ("expected implementation files", "files/modules expected to change", "exact files/modules")),
    ("exact methods/functions", ("expected methods/functions", "methods/functions", "functions/methods")),
    ("files/modules forbidden", ("forbidden", "non-goals", "non-goals")),
    ("tests to add/change", ("tests to add/change", "focused tests", "tests")),
    ("validation commands", ("## validation", "validation commands")),
    ("closeout evidence", ("## closeout requirements", "closeout evidence")),
    ("compatibility/rollback plan", ("rollback", "compatibility")),
    ("stop conditions", ("## stop conditions", "stop conditions")),
)


def exact_product_write_scope_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    broad_roots = {
        ".",
        "apps",
        "domain",
        "engine",
        "foundation",
        "src",
        "tools",
        "docs-site",
        "docs-site/src",
        "docs-site/src/content",
        "docs-site/src/content/docs",
    }
    for scope in entry.write_scope:
        if is_generated_or_derived_scope(scope):
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            errors.append(f"{entry.milestone_id}: product_code write_scope is ambiguous or non-path: {scope}")
            continue
        if "*" in normalized or "..." in normalized:
            errors.append(f"{entry.milestone_id}: product_code write_scope must not use wildcard or ellipsis scope: {scope}")
            continue
        parts = normalized.split("/")
        if normalized in broad_roots or len(parts) < 3:
            errors.append(f"{entry.milestone_id}: product_code write_scope is too broad: {scope}")
            continue
        if normalized.endswith("/"):
            errors.append(f"{entry.milestone_id}: product_code write_scope must be an exact file or module path: {scope}")
            continue
    return errors


def product_plan_contract_errors(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    plan_path: Path,
) -> list[str]:
    errors: list[str] = []
    if not plan_path.exists():
        return [f"{entry.milestone_id}: accepted production plan is missing: {repo_path(plan_path)}"]
    status = document_frontmatter_status(plan_path)
    if status is None:
        errors.append(f"{entry.milestone_id}: accepted production plan has no frontmatter status: {repo_path(plan_path)}")
    elif status.lower() not in {"active", "accepted", "completed"}:
        errors.append(
            f"{entry.milestone_id}: accepted production plan status {status!r} is not active, accepted, or completed"
        )
    plan_text = plan_path.read_text(encoding="utf-8").lower()
    for label, markers in PRODUCT_CODE_REQUIRED_PLAN_MARKERS:
        if not any(marker in plan_text for marker in markers):
            errors.append(f"{entry.milestone_id}: accepted production plan is missing {label}")
    for scope in entry.write_scope:
        normalized = manifest_write_scope_path(scope)
        if normalized is None or is_generated_or_derived_scope(scope):
            continue
        if normalized.lower() not in plan_text:
            errors.append(
                f"{entry.milestone_id}: accepted production plan does not name manifest write_scope {normalized}"
            )
    if roadmap_item.completion_quality == "runtime_proven":
        errors.append(
            f"{entry.milestone_id}: product_code cannot claim runtime_proven before closeout runtime/test evidence exists"
        )
    return errors


def assert_product_code_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    roadmap_item: RoadmapItem,
    *,
    plan_path: Path,
    allow: set[str],
) -> None:
    errors: list[str] = []
    if "product_code" not in allow:
        errors.append("Manifest Runner V4 requires --allow product_code for product/runtime code automation")
    if "foundation_extraction" in allow:
        errors.append("foundation_extraction automation is not implemented and remains blocked")
    if "crate_creation" in allow:
        errors.append("crate_creation automation is not implemented and remains blocked")
    if entry.milestone_type not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: product_code supports implementation/runtime-proof milestones only")
    if milestone.kind not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: production milestone kind {milestone.kind!r} cannot run product_code")
    if not entry.may_create_code:
        errors.append(f"{entry.milestone_id}: manifest does not authorize code creation")
    if entry.may_create_crates:
        errors.append(f"{entry.milestone_id}: crate_creation is required but Manifest Runner V4 does not implement crate creation")
    if not entry.may_modify_production_behavior:
        errors.append(f"{entry.milestone_id}: manifest does not authorize production behavior changes")
    if not entry.owning_wr:
        errors.append(f"{entry.milestone_id}: product_code requires an active owning WR")
    if entry.future_wr_candidate:
        errors.append(f"{entry.milestone_id}: product_code requires a concrete active WR, not a future WR candidate")
    if roadmap_item.planning_state != "current_candidate":
        errors.append(
            f"{entry.milestone_id}: product_code requires active current_candidate WR; "
            f"{roadmap_item.id} is {roadmap_item.planning_state}"
        )
    if roadmap_item.blocker > 2:
        errors.append(f"{entry.milestone_id}: product_code requires B2 or lower implementation blocker; {roadmap_item.id} is {roadmap_item.blocker_label}")
    if not entry.forbidden_scope:
        errors.append(f"{entry.milestone_id}: product_code requires explicit forbidden scope")
    if not entry.validation_commands:
        errors.append(f"{entry.milestone_id}: product_code requires validation commands")
    if not entry.expected_closeout_path or entry.expected_closeout_path.startswith("blocked:"):
        errors.append(f"{entry.milestone_id}: product_code requires an expected closeout path")
    if any("foundation/meta" in scope.lower() for scope in entry.write_scope):
        errors.append(f"{entry.milestone_id}: product_code cannot authorize shared foundation/meta extraction")
    errors.extend(exact_product_write_scope_errors(entry))
    errors.extend(product_plan_contract_errors(entry=entry, roadmap_item=roadmap_item, plan_path=plan_path))
    if errors:
        raise WorkflowError("\n".join(errors))


def apply_product_code(
    context: ManifestCommandContext,
    *,
    allow: set[str],
    run_validations: bool = True,
) -> ProductCodeResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if blockers:
        raise WorkflowError("\n".join(blockers))
    if workflow_action != "write_implementation_contract":
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not product_code")
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
    plan_path = default_contract_path(roadmap_item)
    assert_product_code_allowed(entry, milestone, roadmap_item, plan_path=plan_path, allow=allow)
    validation_results = run_validation_commands(entry.validation_commands) if run_validations else ()
    return ProductCodeResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        plan_path=plan_path,
        manifest_path=context.loaded.path,
        validation_commands=validation_results,
        next_legal_action=(
            f"{entry.milestone_id} product_code gate passed for {entry.owning_wr}; "
            "stop after this implementation WR and close out with runtime/test evidence before continuing."
        ),
    )


def assert_agent_design_source_documents(contract: ManifestAgentDesignContract) -> None:
    missing = [path for path in contract.source_documents if not (REPO_ROOT / path).exists()]
    if missing:
        raise WorkflowError("agent_design source documents are missing: " + ", ".join(missing))


def assert_agent_design_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    allow: set[str],
    deny: set[str],
) -> ManifestAgentDesignContract:
    if "agent_design" not in allow:
        raise WorkflowError("Manifest Runner V2 requires --allow agent_design for design/planning document mutation")
    if "product_code" in allow:
        raise WorkflowError("agent_design cannot run when product_code is allowed")
    if "product_code" not in deny:
        raise WorkflowError("agent_design requires --deny product_code")
    if entry.may_create_code or entry.may_create_crates or entry.may_modify_production_behavior:
        raise WorkflowError(
            f"{entry.milestone_id}: agent_design cannot run for milestones that allow product code, crates, or production behavior changes"
        )
    if milestone.state == "completed":
        raise WorkflowError(f"{entry.milestone_id}: agent_design cannot mutate completed milestones")
    if "agent_design completed design/planning writes" in entry.next_legal_action:
        raise WorkflowError(
            f"{entry.milestone_id}: agent_design output already exists; rerun with --allow agent_closeout for closeout"
        )
    if entry.milestone_type not in {"docs_only", "design_only"}:
        raise WorkflowError(f"{entry.milestone_id}: agent_design supports docs-only or design-only milestones only")
    if not entry.owning_wr:
        raise WorkflowError(f"{entry.milestone_id}: agent_design requires an owning WR")
    if entry.agent_design is None:
        raise WorkflowError(f"{entry.milestone_id}: manifest milestone is missing agent_design contract")
    assert_agent_design_source_documents(entry.agent_design)
    return entry.agent_design


def agent_design_plan_content(
    plan_context: ProductionPlanContext,
    *,
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
    plan_path: Path,
) -> str:
    readiness = render_readiness_report(plan_context, classify_plan_action(plan_context), plan_path)
    source_lines = "\n".join(f"- `{path}`" for path in contract.source_documents)
    section_lines = "\n".join(f"- {section}" for section in contract.required_sections)
    decision_lines = "\n".join(f"- {decision}" for decision in contract.required_decisions)
    acceptance_lines = "\n".join(f"- {item}" for item in contract.acceptance_checklist)
    write_scope_lines = "\n".join(f"- `{scope}`" for scope in entry.write_scope)
    forbidden_lines = "\n".join(f"- {scope}" for scope in entry.forbidden_scope)
    validation_lines = "\n".join(f"- `{command}`" for command in entry.validation_commands)
    stop_lines = "\n".join(f"- {condition}" for condition in entry.stop_conditions)
    return "\n".join(
        [
            "---",
            f"title: {plan_context.roadmap_item.id} UI Program Contract Design Plan",
            f"description: Design/planning contract for {entry.milestone_id} under {plan_context.roadmap_item.id}.",
            "status: active",
            "owner: ui",
            "layer: workspace / domain/ui",
            "canonical: false",
            f"last_reviewed: {plan_context.planning.production.last_reviewed}",
            "related:",
            "  - ../../../workspace/production-tracks.yaml",
            "  - ../../../workspace/roadmap-deferred.yaml",
            "  - ../../../workspace/track-execution-manifests/pt-ui-program.yaml",
            "related_designs:",
            "  - ../../../design/active/runenwerk-domain-workbench-north-star.md",
            "  - ../../../design/active/ui-program-architecture.md",
            "---",
            "",
            f"# {plan_context.roadmap_item.id} UI Program Contract Design Plan",
            "",
            "## Status And Authority",
            "",
            f"- Production milestone: `{entry.milestone_id}` - {entry.title}",
            f"- Roadmap item: `{plan_context.roadmap_item.id}` - {plan_context.roadmap_item.title}",
            "- Authority: design/planning only.",
            "- This plan does not authorize product/runtime code, crate creation, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, or shared foundation/meta extraction.",
            "- This plan does not close the milestone. Closeout requires separate manual evidence or an explicit `agent_closeout` automation run.",
            "",
            "## Production Planning Output",
            "",
            readiness,
            "",
            "## Source Documents",
            "",
            source_lines,
            "",
            "## Required Stage 1 Sections",
            "",
            section_lines,
            "",
            "## Required Decisions",
            "",
            decision_lines,
            "",
            "## Exact Write Scope",
            "",
            write_scope_lines,
            "",
            "## Forbidden Scope",
            "",
            forbidden_lines,
            "",
            "## Acceptance Checklist",
            "",
            acceptance_lines,
            "",
            "## Validation Commands",
            "",
            validation_lines,
            "",
            "## Stop Conditions",
            "",
            stop_lines,
            f"- Stop after writing the design/planning contract and rerun `task production:next -- --track {plan_context.track.id}`.",
            "- Stop before closeout unless rerun with `--allow agent_closeout` and the required evidence and validation pass.",
            "",
            "## Closeout Expectation",
            "",
            f"- Expected closeout path: `{entry.expected_closeout_path}`",
            "- Closeout must prove the Stage 1 design sections exist, answer or explicitly block open questions, and preserve all forbidden-scope constraints.",
            "",
        ]
    )


def pm002_contract_section() -> str:
    return """## PM-UI-PROGRAM-002 Stage 1 Contract

This section is the bounded Stage 1 contract produced for `PM-UI-PROGRAM-002`.
It tightens the design-level contracts only. It does not authorize code,
crates, placeholder folders, Stage 6 proof work, MaterialProgram work, or
shared `foundation/meta` extraction.

### Graph Ownership Contract

`UiProgram` owns the durable UI program contract. The graph list is closed for
Stage 1 and contains `ControlGraph`, `LayoutGraph`, `StateGraph`,
`StyleGraph`, `InteractionGraph`, `BindingGraph`, `VisualGraph`,
`AccessibilityGraph`, and `InspectionGraph`.

Each graph owns one class of UI truth and may reference other graphs only by
stable IDs, source-map spans, and explicit dependency edges. None of these
graphs are generic node soup:

- `ControlGraph` owns control identity, package/control kind, hierarchy, and
  capability requirements.
- `LayoutGraph` owns measurement, constraints, placement, and layout kernel
  dependencies.
- `StateGraph` owns structural state declarations and dependencies.
- `StyleGraph` owns UI-domain style intent and state variants.
- `InteractionGraph` owns focus, capture, gestures, route slots, and event
  packet emission.
- `BindingGraph` owns read/write host data contracts and binding dependency
  flow.
- `VisualGraph` owns UI visual intent before renderer-facing output exists.
- `AccessibilityGraph` owns semantic roles, names, focus order, and assistive
  metadata.
- `InspectionGraph` owns provenance, source-map links, diagnostics, and
  runtime inspection references.

### UiSchemaValue Contract

`UiSchemaValue` is UI-owned until the Second-Domain Extraction Gate. It supports
null, booleans, signed integers, unsigned integers, finite floats, strings,
stable ID references, route references, opaque host references, object values,
list values, and schema-declared optional values.

Opaque host references are handles to host-provided data that UI may compare,
route, inspect, and diagnose, but never dereference into hidden app behavior.
Route references identify stable `RouteId` contracts and never replace
`UiEventPacket`.

Every schema-bound value is validated against a schema ID and schema version.
Unknown fields are rejected unless the schema explicitly marks them as
preserved debug/fixture data. Breaking schema changes require a new version and
a migration report. Validation failures attach schema ID, schema version,
source-map span, diagnostic ID, and the graph/artifact location that observed
the failure.

### Stable ID Policy

Stable IDs are namespaced and versioned. Stage 1 applies the policy to control
kind IDs, package IDs, schema IDs, event payload schema IDs, kernel IDs,
capability IDs, route IDs, artifact IDs, fixture IDs, and diagnostic IDs.

IDs are never silently repurposed. Collisions fail validation. Deprecation
requires a replacement or explicit no-replacement reason plus migration
diagnostics. Breaking shape changes require a new version. Artifact manifests
record the exact IDs and versions used for compile, evaluation, diagnostics,
source maps, and fixtures.

### StateGraph And UiStateModel Contract

`StateGraph` owns structural state requirements: declarations, state class,
dependencies, package state schemas, persistence eligibility, source-map spans,
and artifact layout inputs.

`UiStateModel` owns runtime state contracts: transient state, preview state,
committed state, focus state, hover state, pressed/captured state, drag state,
animation state, host-fed state, and package-owned state. Compiled state tables
belong to `UiRuntimeArtifactTables`. Evaluation-time transitions belong to
`UiEvaluator`. Host-fed state enters only through explicit binding or host
contracts and cannot become hidden UI semantics.

### BindingGraph Contract

`BindingGraph` owns read model declarations, write model declarations, value
snapshots, collection diffs, dirty propagation, host data contracts,
authorization policy, capability checks, binding diagnostics, and source-map
attachments.

Reads consume host-provided snapshots. Writes produce route/event payloads or
domain-owned mutation requests; UI never mutates host/domain state directly.
Collection diffs identify insert, remove, move, replace, selection, and
expansion changes with stable item IDs when available. Binding failure is
diagnostic output, not silent fallback.

### VisualGraph And UiFrame Boundary

`VisualGraph` is UI-owned visual intent. It describes draw order, shapes, text
operators, image slots, clips, overlays, control visual kernels, invalidation
dependencies, and source maps.

`UiFrame` is derived renderer-facing structural output. It may contain resolved
visual commands, text layout requests/results, clipping, z-order, accessibility
links, source-map references, diagnostics, and render-handoff metadata. It is
not product truth. The renderer owns backend execution and resource residency,
not UI meaning.

### Text / Render Boundary

UI owns font/style intent, text value bindings, text layout requests, wrapping,
overflow policy, semantic role, accessibility labels, source maps, DPI/scale
intent, and fallback policy requirements.

The text backend owns shaping, fallback font resolution, glyph identity keys,
atlas preparation keys, glyph metrics, text layout metrics, cache preparation,
and invalidation policy for font, DPI, scale, locale, and text-policy changes.

The renderer owns GPU atlas residency, upload handles, eviction implementation,
backend resource lifetime, batching, and draw execution. Renderer handles are
not UI truth and must not be stored as semantic UI data.

### Event Packet And Payload Contract

`UiEventPacket` is semantic UI output. It carries `RouteId`,
`RouteSchemaVersion`, source control ID, interaction phase, payload schema ID,
`UiSchemaValue` payload, source-map attachment, capability context, and
diagnostic context.

There is no giant `UiSemanticEvent` enum. Event payloads are schema-based and
route-based. Unknown routes, unsupported route schema versions, missing
capabilities, invalid payloads, and missing host route-map entries produce
diagnostics and do not execute hidden behavior.

### Route / HostCommand / DomainCommand Boundary

`RouteId` is a stable namespaced ID. `RouteSchemaVersion` versions the route
payload and route contract shape. `RouteCapability` names the capability needed
to emit or consume the route. `HostRouteMapVersion` versions the host-owned map
from `UiEventPacket` to `HostCommand` and optional `DomainCommand`.

`HostCommand` is environment-specific. `DomainCommand` is domain-owned mutation
authority. Hosts map UI events into commands only through inspectable route-map
policy and capability checks. Routes must not become hidden app behavior or
free-form strings.

### Source-Map Attachment Points

Source-map attachments are required on control nodes, graph edges, schema
values, package properties, state declarations, bindings, routes, event
payloads, visual operators, text layout requests, artifact manifest entries,
artifact tables, diagnostics, fixtures, migrations, and host route-map entries.

Every runtime diagnostic and fixture assertion must be able to point back to
the authored source when source context exists. Generated or synthesized nodes
must record generated provenance and the source span or rule that produced
them.

### Diagnostics Attachment Points

Diagnostics attach to schema validation, ID collision, deprecated ID use,
package registration, missing kernels, capability denial, invalid bindings,
collection diff mismatch, route validation, host route-map mismatch, text
fallback failure, source-map loss, artifact invalidation, migration failure,
fixture mismatch, and renderer-boundary misuse.

Diagnostic IDs are stable IDs. Diagnostics must include severity, source-map
span when available, owning graph/artifact, schema or route context when
available, suggested migration or fix when known, and whether evaluation may
continue.

### Open Questions And Blocked Decisions

- Final graph serialization format remains deferred until Stage 6 evidence
  proves the minimum artifact and fixture shape.
- Exact future module paths remain target responsibilities, not immediate file
  creation instructions.
- Shared `foundation/meta` extraction remains blocked until UI and
  `MaterialProgram` prove the same primitive and a separate extraction design
  accepts it.
- Stage 6 proof work remains blocked until Stages 1 through 5 close with
  evidence.

"""


def upsert_markdown_section(text: str, *, heading: str, section: str, before_heading: str) -> str:
    if heading in text:
        start = text.index(heading)
        next_heading = text.find("\n## ", start + len(heading))
        end = next_heading if next_heading != -1 else len(text)
        return text[:start] + section.rstrip() + "\n\n" + text[end:].lstrip("\n")
    before = text.find(before_heading)
    if before == -1:
        return text.rstrip() + "\n\n" + section.rstrip() + "\n"
    return text[:before] + section.rstrip() + "\n\n" + text[before:]


def update_ui_program_architecture(path: Path) -> None:
    original = path.read_text(encoding="utf-8")
    updated = upsert_markdown_section(
        original,
        heading="## PM-UI-PROGRAM-002 Stage 1 Contract",
        section=pm002_contract_section(),
        before_heading="## 13. Staged Implementation Plan",
    )
    path.write_text(updated, encoding="utf-8", newline="\n")


def updated_manifest_data_after_agent_design(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    next_action = (
        f"{entry.milestone_id} agent_design completed design/planning writes; "
        "stop for closeout; rerun with --allow agent_closeout after evidence is valid."
    )
    data["next_legal_action"] = next_action
    for milestone_data in data["milestones"]:
        if milestone_data["milestone_id"] != entry.milestone_id:
            continue
        milestone_data["next_legal_action"] = next_action
        closeout_stop = "stop before closeout unless agent_closeout is explicitly allowed and evidence is valid"
        if closeout_stop not in milestone_data["stop_conditions"]:
            milestone_data["stop_conditions"].append(closeout_stop)
    return data


def updated_deferred_roadmap_data_after_agent_design(
    roadmap_source: Path,
    *,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
    plan_path: Path,
) -> tuple[Path, dict]:
    _, deferred_source = split_source_paths(roadmap_source)
    deferred_data = load_yaml(deferred_source)
    changed = False
    for item in deferred_data.get("items", []):
        if item.get("id") != wr_id:
            continue
        item["next_evidence"] = (
            f"{entry.milestone_id} design/planning output exists at {repo_path(plan_path)} and "
            f"docs-site/src/content/docs/design/active/ui-program-architecture.md; closeout evidence is still required."
        )
        item["current_decision"] = (
            "Manifest Runner V2 agent_design wrote the Stage 1 UI Program Contract Design plan and architecture sections. "
            "This remains design/planning evidence only."
        )
        item["current_call"] = (
            "Stop before closeout unless agent_closeout is explicitly allowed and evidence is valid. No product code, crates, 6A, MaterialProgram, "
            "or foundation/meta work is authorized."
        )
        item["main_blocker"] = "Closeout evidence is missing; rerun with agent_closeout only after evidence and validation are valid."
        item["why_not_ready"] = "Design/planning content exists, but PM-002 has not been closed with evidence."
        changed = True
        break
    if not changed:
        raise WorkflowError(f"{wr_id}: not present in deferred roadmap source")
    return deferred_source, deferred_data


def apply_agent_design(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    deny: set[str],
    run_validations: bool = True,
) -> AgentDesignResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if blockers:
        raise WorkflowError("\n".join(blockers))
    if workflow_action != "design_first":
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not agent_design")
    contract = assert_agent_design_allowed(entry, milestone, allow=allow, deny=deny)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")

    plan_context = resolve_plan_context(
        entry.milestone_id,
        entry.owning_wr,
        production_source=production_source,
        roadmap_source=roadmap_source,
    )
    plan_path = default_contract_path(plan_context.roadmap_item)
    design_paths = (UI_PROGRAM_ARCHITECTURE_PATH,)
    _, deferred_source = split_source_paths(roadmap_source)
    write_paths = [
        repo_path(plan_path),
        repo_path(context.loaded.path),
        repo_path(deferred_source),
        *(repo_path(path) for path in design_paths),
    ]
    assert_agent_design_write_scope(entry=entry, roadmap_item=roadmap_item, write_paths=write_paths)

    plan_path.parent.mkdir(parents=True, exist_ok=True)
    plan_path.write_text(
        agent_design_plan_content(plan_context, entry=entry, contract=contract, plan_path=plan_path),
        encoding="utf-8",
        newline="\n",
    )
    for design_path in design_paths:
        update_ui_program_architecture(design_path)

    manifest_data = updated_manifest_data_after_agent_design(context.loaded, entry=entry)
    write_yaml_mapping(context.loaded.path, manifest_data)
    deferred_source, deferred_data = updated_deferred_roadmap_data_after_agent_design(
        roadmap_source,
        wr_id=entry.owning_wr,
        entry=entry,
        plan_path=plan_path,
    )
    write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)

    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    next_legal_action = (
        f"{entry.milestone_id} design/planning output exists; stop for closeout and rerun with --allow agent_closeout after evidence is valid."
    )
    return AgentDesignResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        plan_path=plan_path,
        design_paths=design_paths,
        manifest_path=context.loaded.path,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
    )


def agent_closeout_pending(entry: TrackExecutionManifestMilestone) -> bool:
    return "agent_design completed design/planning writes" in entry.next_legal_action


def assert_agent_closeout_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    allow: set[str],
    deny: set[str],
) -> None:
    if "agent_closeout" not in allow:
        raise WorkflowError("Manifest Runner V3 requires --allow agent_closeout for closeout mutation")
    if "product_code" in allow:
        raise WorkflowError("agent_closeout cannot run when product_code is allowed")
    if "product_code" not in deny:
        raise WorkflowError("agent_closeout requires --deny product_code")
    if entry.may_create_code or entry.may_create_crates or entry.may_modify_production_behavior:
        raise WorkflowError(
            f"{entry.milestone_id}: agent_closeout supports docs, design, or governance milestones only; "
            "runtime/product milestones need a future runtime evidence closeout path"
        )
    if milestone.kind != "design" or entry.milestone_type not in {"docs_only", "design_only"}:
        raise WorkflowError(f"{entry.milestone_id}: agent_closeout supports docs, design, or governance milestones only")
    if milestone.state == "completed":
        raise WorkflowError(f"{entry.milestone_id}: milestone is already completed")
    if milestone.completion_quality == "runtime_proven":
        raise WorkflowError(f"{entry.milestone_id}: docs or design milestones cannot close as runtime_proven")
    if not entry.owning_wr:
        raise WorkflowError(f"{entry.milestone_id}: agent_closeout requires an owning WR")
    if not agent_closeout_pending(entry):
        raise WorkflowError(
            f"{entry.milestone_id}: closeout evidence is not ready; run the required design/governance evidence step first"
        )


def required_pm002_stage_headings() -> list[str]:
    return [
        "## PM-UI-PROGRAM-002 Stage 1 Contract",
        "### Graph Ownership Contract",
        "### UiSchemaValue Contract",
        "### Stable ID Policy",
        "### StateGraph And UiStateModel Contract",
        "### BindingGraph Contract",
        "### VisualGraph And UiFrame Boundary",
        "### Text / Render Boundary",
        "### Event Packet And Payload Contract",
        "### Route / HostCommand / DomainCommand Boundary",
        "### Source-Map Attachment Points",
        "### Diagnostics Attachment Points",
        "### Open Questions And Blocked Decisions",
    ]


def agent_closeout_evidence_paths(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    plan_path: Path,
) -> tuple[list[Path], list[str]]:
    evidence_paths = [plan_path]
    errors: list[str] = []
    if not plan_path.exists():
        errors.append(f"{entry.milestone_id}: required production plan evidence is missing: {repo_path(plan_path)}")

    architecture_scope = repo_path(UI_PROGRAM_ARCHITECTURE_PATH)
    if path_is_covered_by_scope(architecture_scope, manifest_write_scopes_for_entry(entry)) or path_is_covered_by_scope(
        architecture_scope,
        normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes),
    ):
        evidence_paths.append(UI_PROGRAM_ARCHITECTURE_PATH)
        if not UI_PROGRAM_ARCHITECTURE_PATH.exists():
            errors.append(f"{entry.milestone_id}: required design evidence is missing: {architecture_scope}")
        else:
            design_text = UI_PROGRAM_ARCHITECTURE_PATH.read_text(encoding="utf-8")
            missing_headings = [heading for heading in required_pm002_stage_headings() if heading not in design_text]
            if missing_headings:
                errors.append(
                    f"{entry.milestone_id}: UI Program Architecture is missing Stage 1 closeout headings: "
                    + ", ".join(missing_headings)
                )
    return evidence_paths, errors


def bounded_contract_known_gaps(entry: TrackExecutionManifestMilestone) -> list[str]:
    return [
        f"{entry.milestone_id} is a bounded design/governance closeout, not runtime_proven evidence.",
        "No product/runtime code, crates, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, or shared foundation/meta extraction was performed.",
        "Later PT-UI-PROGRAM milestones still require their own WRs, production plans, validation, and closeout evidence.",
    ]


def closeout_report_content(
    *,
    track_id: str,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    roadmap_item: RoadmapItem,
    closeout_path: Path,
    evidence_paths: list[Path],
) -> str:
    evidence_lines = "\n".join(f"- `{repo_path(path)}`" for path in evidence_paths)
    validation_lines = "\n".join(f"- `{command}`" for command in entry.validation_commands)
    forbidden_lines = "\n".join(f"- {scope}" for scope in entry.forbidden_scope)
    gap_lines = "\n".join(f"- {gap}" for gap in bounded_contract_known_gaps(entry))
    next_action = f"After this closeout, rerun `task ai:goal -- --track {track_id}` and continue only to the next manifest legal action."
    return "\n".join(
        [
            "---",
            f"title: {entry.milestone_id} {entry.title} Closeout",
            f"description: Bounded-contract closeout for {entry.milestone_id} / {roadmap_item.id}.",
            "status: completed",
            "owner: ui",
            "layer: workspace / domain-ui",
            "canonical: false",
            f"last_reviewed: {date.today().isoformat()}",
            "related_reports:",
            f"  - ../../implementation-plans/{roadmap_item.id.lower()}-{slugify(roadmap_item.title)}/plan.md",
            "  - ../../track-execution-manifests/pt-ui-program/manifest.md",
            "related_roadmaps:",
            "  - ../../../workspace/production-tracks.yaml",
            "  - ../../../workspace/roadmap-archive.yaml",
            "  - ../../../workspace/track-execution-manifests/pt-ui-program.yaml",
            "---",
            "",
            f"# {entry.milestone_id} {entry.title} Closeout",
            "",
            "## Summary",
            "",
            f"`{entry.milestone_id}` / `{roadmap_item.id}` is closed as `bounded_contract` design/governance evidence for `{track_id}`.",
            "",
            "This closeout does not authorize product/runtime code, crate creation, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, RenderPlan substitution, or shared `foundation/meta` extraction.",
            "",
            "## Authority",
            "",
            f"- Milestone id: `{entry.milestone_id}`",
            f"- WR id: `{roadmap_item.id}`",
            f"- Authority level: `{entry.authority_level}`",
            f"- Milestone type: `{entry.milestone_type}`",
            f"- Production milestone kind/state before closeout: `{milestone.kind}` / `{milestone.state}`",
            "- Completion quality: `bounded_contract`",
            "",
            "## Evidence Files",
            "",
            evidence_lines,
            f"- `{repo_path(closeout_path)}`",
            "",
            "## Validation Commands",
            "",
            validation_lines,
            "",
            "The Manifest Runner executes these commands after writing closeout, production, roadmap, and manifest state. Command output records the exit codes.",
            "",
            "## Forbidden Scope Preserved",
            "",
            forbidden_lines,
            "",
            "No product/runtime source files, crates, placeholder folders, 6A proof work, MaterialProgram implementation, or shared foundation/meta extraction were created or modified by this closeout.",
            "",
            "## Known Gaps",
            "",
            gap_lines,
            "",
            "## Next Legal Action",
            "",
            next_action,
            "",
            "The next milestone may not start design authoring or implementation inside this closeout action.",
            "",
        ]
    )


def updated_production_data_after_agent_closeout(
    production_source: Path,
    *,
    milestone_id: str,
    closeout_path: Path,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = load_yaml(production_source)
    changed = False
    closeout_repo_path = repo_path(closeout_path)
    for track_data in data.get("tracks", []):
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") != milestone_id:
                continue
            milestone_data["state"] = "completed"
            milestone_data["completion_quality"] = "bounded_contract"
            milestone_data["known_quality_gaps"] = bounded_contract_known_gaps(entry)
            milestone_data["completion_audit"] = closeout_repo_path
            evidence_gates = milestone_data.setdefault("evidence_gates", [])
            if not any(gate.get("path") == closeout_repo_path for gate in evidence_gates):
                evidence_gates.append(
                    {
                        "path": closeout_repo_path,
                        "required_status": "completed",
                        "reason": f"{milestone_id} requires completed bounded-contract closeout evidence.",
                    }
                )
            changed = True
            break
        if changed:
            break
    if not changed:
        raise WorkflowError(f"{milestone_id}: not found in production source")
    return data


def updated_roadmap_sources_after_agent_closeout(
    roadmap_source: Path,
    *,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
    closeout_path: Path,
) -> tuple[Path, dict, Path, dict, Path, dict | None]:
    active_data = load_yaml(roadmap_source)
    archive_source, deferred_source = split_source_paths(roadmap_source)
    archive_data = load_yaml(archive_source) if archive_source.exists() else empty_split_source_like(active_data)
    deferred_data = load_yaml(deferred_source) if deferred_source.exists() else empty_split_source_like(active_data)
    if any(item.get("id") == wr_id for item in archive_data.get("items", [])):
        raise WorkflowError(f"{wr_id}: already present in archive roadmap source")

    item_data: dict | None = None
    active_changed = False
    active_items = active_data.get("items", [])
    deferred_items = deferred_data.get("items", [])
    for source_name, source_items in (("deferred", deferred_items), ("active", active_items)):
        for index, candidate in enumerate(list(source_items)):
            if candidate.get("id") == wr_id:
                item_data = source_items.pop(index)
                active_changed = source_name == "active"
                break
        if item_data is not None:
            break
    if item_data is None:
        raise WorkflowError(f"{wr_id}: not present in active or deferred roadmap source")

    closeout_repo_path = repo_path(closeout_path)
    item_data["gate"] = "Completed"
    item_data["planning_state"] = "completed"
    item_data["next_evidence"] = f"Completed through {closeout_repo_path}."
    item_data["current_decision"] = (
        f"Completed at bounded_contract by Manifest Runner V3 agent_closeout for {entry.milestone_id}. "
        "This is design/governance evidence only."
    )
    item_data["current_call"] = (
        "Complete. Preserve as bounded-contract design evidence; continue PT-UI-PROGRAM only through the next manifest legal action."
    )
    item_data["first_move"] = "Completed; run task ai:goal -- --track PT-UI-PROGRAM for the next legal milestone."
    item_data["main_blocker"] = "Complete; later milestones require separate WRs, plans, validation, and closeout evidence."
    item_data["why_not_ready"] = ""
    item_data["completion_quality"] = "bounded_contract"
    item_data["known_quality_gaps"] = bounded_contract_known_gaps(entry)
    item_data["completion_audit"] = closeout_repo_path
    item_data["diagram_call"] = ["bounded contract", "no implementation"]
    archive_data.setdefault("items", []).append(item_data)
    return archive_source, archive_data, deferred_source, deferred_data, roadmap_source, active_data if active_changed else None


def updated_manifest_data_after_agent_closeout(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    milestones = data["milestones"]
    current_index = next(
        index for index, milestone_data in enumerate(milestones) if milestone_data["milestone_id"] == entry.milestone_id
    )
    next_milestone = milestones[current_index + 1] if current_index + 1 < len(milestones) else None
    if next_milestone is None:
        next_action = f"{entry.milestone_id} is complete; run track closeout if all milestones are complete."
    elif next_milestone.get("owning_wr"):
        next_action = (
            f"Continue to {next_milestone['milestone_id']} {next_milestone['title']} only through its owning WR "
            f"{next_milestone['owning_wr']} and bounded plan."
        )
    else:
        next_action = (
            f"Create or link the design WR for {next_milestone['milestone_id']} {next_milestone['title']}; "
            "stop before design authoring until that WR and plan exist."
        )
    data["next_legal_action"] = next_action
    for milestone_data in milestones:
        if milestone_data["milestone_id"] == entry.milestone_id:
            milestone_data["next_legal_action"] = (
                f"{entry.milestone_id} completed by agent_closeout as bounded_contract; "
                "continue only to the next manifest legal action."
            )
            milestone_data["stop_conditions"] = [
                condition
                for condition in milestone_data["stop_conditions"]
                if "agent_closeout is not implemented" not in condition
                and "stop before closeout unless agent_closeout" not in condition
            ]
            milestone_data["stop_conditions"].append("completed by agent_closeout as bounded_contract")
        elif next_milestone is not None and milestone_data["milestone_id"] == next_milestone["milestone_id"]:
            milestone_data["next_legal_action"] = next_action
    return data


def update_manifest_report_after_agent_closeout(path: Path) -> None:
    if not path.exists():
        return
    text = path.read_text(encoding="utf-8")
    replacements = {
        "Current blockers after Stage 1 design/planning output:": "Current blockers after Stage 1 closeout:",
        "- `PM-UI-PROGRAM-002` / Stage 1 is still the current legal milestone and is\n  blocked on design closeout evidence because `agent_closeout` is not\n  implemented.": "- `PM-UI-PROGRAM-002` / Stage 1 is completed as bounded-contract design evidence.\n- `PM-UI-PROGRAM-003` / Stage 2 is the next legal milestone and is blocked on Track Expansion creating or linking its owning WR.",
        "| Current next legal action | `PM-UI-PROGRAM-002` design/planning output exists; stop for closeout because `agent_closeout` is not implemented |": "| Current next legal action | Create or link the design WR for `PM-UI-PROGRAM-003` Control Package Proof Design; stop before Stage 2 design authoring until that WR and plan exist |",
        "| Current next legal action | `PM-UI-PROGRAM-002` design/planning output exists; stop for closeout; rerun with `--allow agent_closeout` after evidence is valid |": "| Current next legal action | Create or link the design WR for `PM-UI-PROGRAM-003` Control Package Proof Design; stop before Stage 2 design authoring until that WR and plan exist |",
        "| 2 | `PM-UI-PROGRAM-002` UI Program Contract Design | Stage 1 | `WR-136` | PM-001 | Stop for design closeout; `agent_closeout` is not implemented. |": "| 2 | `PM-UI-PROGRAM-002` UI Program Contract Design | Stage 1 | `WR-136` | PM-001 | Completed bounded-contract closeout. |",
        "| Stop conditions | design/planning output exists; stop before closeout because `agent_closeout` is not implemented |": "| Stop conditions | completed; no further PM-002 action is legal through this milestone |",
        "- `production:next`: reports `PM-UI-PROGRAM-002` / Stage 1 as the current legal\n  milestone and blocks on closeout because `agent_closeout` is not implemented.": "- `production:next`: reports `PM-UI-PROGRAM-003` / Stage 2 as the current legal\n  milestone and points to Track Expansion for the future WR candidate.",
        "- `PM-UI-PROGRAM-002` / Stage 1 now owns deferred WR `WR-136`, created by\n  Manifest Runner V1 `auto_safe` Track Expansion.": "- `WR-136` is archived as completed bounded-contract Stage 1 evidence after\n  Manifest Runner V3 `agent_closeout` closeout.",
        "- `production:run-track -- --allow auto_safe --allow agent_design --deny\n  product_code`: created the `WR-136` design/planning plan, updated the bounded\n  Stage 1 contract in the UI Program Architecture design, updated planning\n  metadata, and stopped before closeout.": "- `production:run-track -- --allow auto_safe --allow agent_design --deny\n  product_code`: created the `WR-136` design/planning plan, updated the bounded\n  Stage 1 contract in the UI Program Architecture design, updated planning\n  metadata, and stopped before closeout.\n- `production:run-track -- --allow auto_safe --allow agent_design --allow\n  agent_closeout --deny product_code`: closed `PM-UI-PROGRAM-002` / `WR-136`\n  as `bounded_contract`, archived the WR, and stopped before Stage 2 authoring.",
        "- `production:audit-track`: sees the full staged path and the PM-002 WR link,\n  but still blocks implementation until PM-002 closeout gates pass.": "- `production:audit-track`: sees the full staged path and completed PM-002\n  closeout evidence; it still blocks before Stage 2 authoring until Track\n  Expansion creates or links the PM-003 WR.",
        "PM-UI-PROGRAM-002 design/planning output exists. Stop for closeout because\n`agent_closeout` is not implemented.": "PM-UI-PROGRAM-002 is closed as bounded-contract design evidence. The next legal action is PM-UI-PROGRAM-003 Track Expansion; stop before Stage 2 design authoring.",
    }
    updated = text
    for old, new in replacements.items():
        updated = updated.replace(old, new)
    path.write_text(updated, encoding="utf-8", newline="\n")


def validate_agent_closeout_data(
    *,
    production_data: dict,
    manifest_data: dict,
    active_roadmap_data: dict,
    archive_roadmap_data: dict,
    deferred_roadmap_data: dict,
    roadmap_source: Path,
    manifest_path: Path,
    track_id: str,
) -> None:
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            active_roadmap_data,
            roadmap_source,
            archive_data=archive_roadmap_data,
            deferred_data=deferred_roadmap_data,
        )
    )
    manifest = TrackExecutionManifest.model_validate(manifest_data)
    loaded = LoadedTrackExecutionManifest(manifest=manifest, path=manifest_path)
    track = find_track(planning, track_id)
    audit_manifest_or_raise(loaded, track=track, roadmap=roadmap)


def apply_agent_closeout(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    deny: set[str],
    run_validations: bool = True,
) -> AgentCloseoutResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    closeout_blockers = [blocker for blocker in blockers if "Track Expansion must create or link" not in blocker]
    if closeout_blockers:
        raise WorkflowError("\n".join(closeout_blockers))
    if workflow_action != "design_first":
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not agent_closeout")
    assert_agent_closeout_allowed(entry, milestone, allow=allow, deny=deny)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")

    plan_path = default_contract_path(roadmap_item)
    closeout_path = REPO_ROOT / entry.expected_closeout_path
    archive_source, deferred_source = split_source_paths(roadmap_source)
    manifest_report = REPO_ROOT / manifest_report_path(context.track.id)
    write_paths = [
        repo_path(production_source),
        repo_path(context.loaded.path),
        repo_path(manifest_report),
        repo_path(archive_source),
        repo_path(deferred_source),
        repo_path(closeout_path),
    ]
    assert_runner_write_scope(entry=entry, roadmap_item=roadmap_item, write_paths=write_paths, action_label="agent_closeout")
    evidence_paths, evidence_errors = agent_closeout_evidence_paths(
        entry=entry,
        roadmap_item=roadmap_item,
        plan_path=plan_path,
    )
    if evidence_errors:
        raise WorkflowError("\n".join(evidence_errors))

    closeout_path.parent.mkdir(parents=True, exist_ok=True)
    closeout_path.write_text(
        closeout_report_content(
            track_id=context.track.id,
            entry=entry,
            milestone=milestone,
            roadmap_item=roadmap_item,
            closeout_path=closeout_path,
            evidence_paths=evidence_paths,
        ),
        encoding="utf-8",
        newline="\n",
    )
    production_data = updated_production_data_after_agent_closeout(
        production_source,
        milestone_id=entry.milestone_id,
        closeout_path=closeout_path,
        entry=entry,
    )
    archive_source, archive_data, deferred_source, deferred_data, active_source, active_data = (
        updated_roadmap_sources_after_agent_closeout(
            roadmap_source,
            wr_id=entry.owning_wr,
            entry=entry,
            closeout_path=closeout_path,
        )
    )
    manifest_data = updated_manifest_data_after_agent_closeout(context.loaded, entry=entry)
    validate_agent_closeout_data(
        production_data=production_data,
        manifest_data=manifest_data,
        active_roadmap_data=active_data if active_data is not None else load_yaml(roadmap_source),
        archive_roadmap_data=archive_data,
        deferred_roadmap_data=deferred_data,
        roadmap_source=roadmap_source,
        manifest_path=context.loaded.path,
        track_id=context.track.id,
    )

    write_yaml_mapping(production_source, production_data)
    if active_data is not None:
        write_yaml_mapping(active_source, active_data, indent_sequences=False)
    write_yaml_mapping(archive_source, archive_data, indent_sequences=False)
    write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)
    write_yaml_mapping(context.loaded.path, manifest_data)
    update_manifest_report_after_agent_closeout(manifest_report)
    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    next_legal_action = str(manifest_data["next_legal_action"])
    return AgentCloseoutResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        closeout_path=closeout_path,
        manifest_path=context.loaded.path,
        production_source=production_source,
        roadmap_archive_source=archive_source,
        roadmap_deferred_source=deferred_source,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
    )


def print_errors(title: str, errors: list[str]) -> None:
    console.print(f"[red]{title}[/red]")
    for error in errors:
        console.print(f"- {error}")


@app.command("plan-track")
def plan_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-UI-PROGRAM."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
    force: bool = typer.Option(False, "--force", help="Overwrite an existing manifest scaffold. Use only for manual repair."),
) -> None:
    try:
        planning = load_production_tracks(production_source)
        roadmap = load_roadmap(roadmap_source)
        production_track = find_track(planning, track)
        path = manifest_source_path(track, root=manifest_source_root)
        if path.exists() and not force:
            loaded = load_track_execution_manifest(track, root=manifest_source_root)
            assert loaded is not None
            errors = audit_manifest(loaded, track=production_track, roadmap=roadmap)
            console.print(f"Track Execution Manifest already exists: {repo_path(path)}")
            console.print("No implementation authority is created by this command.")
            if errors:
                print_manifest_audit_blockers(errors)
                raise typer.Exit(1)
            console.print("[green]manifest audit passed[/green]")
            return
        manifest = build_manifest_scaffold(production_track, roadmap)
        write_manifest(path, manifest)
        console.print(f"Wrote conservative Track Execution Manifest scaffold: {repo_path(path)}")
        console.print("The scaffold is planning-only and contains blocked fields until reviewed.")
        console.print("No implementation authority is created by this command.")
    except WorkflowError as error:
        console.print("[red]production:plan-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("expand-track")
def expand_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-UI-PROGRAM."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
) -> None:
    try:
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        errors = audit_manifest(context.loaded, track=context.track, roadmap=context.roadmap)
        if errors:
            print_manifest_audit_blockers(errors)
            raise typer.Exit(1)
        candidates = [entry for entry in context.loaded.manifest.milestones if entry.future_wr_candidate]
        console.print(f"Track Expansion candidates for {track}:")
        for entry in candidates:
            console.print(f"- {entry.future_wr_candidate}: {entry.milestone_id} - {entry.title}")
        console.print("production:expand-track is read-only; run production:run-track -- --allow auto_safe for the guarded V1 mutation path.")
        console.print("No implementation authority is created by this command.")
    except WorkflowError as error:
        console.print("[red]production:expand-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("run-track")
def run_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-UI-PROGRAM."),
    allow: list[str] = typer.Option(
        [],
        "--allow",
        help="Permission tier to allow. Supported: auto_safe, agent_design, agent_closeout, product_code.",
    ),
    deny: list[str] = typer.Option(
        [],
        "--deny",
        help="Permission tier to deny explicitly, for example product_code.",
    ),
    max_actions: int = typer.Option(1, "--max-actions", min=1, help="Maximum mechanical actions before stopping."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
) -> None:
    try:
        allow_set = set(allow)
        deny_set = set(deny)
        unknown_permissions = sorted((allow_set | deny_set) - MANIFEST_RUNNER_PERMISSIONS)
        if unknown_permissions:
            raise WorkflowError(f"unknown Manifest Runner permissions: {', '.join(unknown_permissions)}")
        if allow_set & deny_set:
            raise WorkflowError("the same Manifest Runner permission cannot be both allowed and denied")
        if not allow_set:
            raise WorkflowError("Manifest Runner requires at least one --allow permission")
        unsupported_allowed = sorted(allow_set - {"auto_safe", "agent_design", "agent_closeout", "product_code"})
        if unsupported_allowed:
            raise WorkflowError(
                "Manifest Runner does not implement allowed permissions: " + ", ".join(unsupported_allowed)
            )

        actions: list[AutoSafeExpansionResult | AgentDesignResult | AgentCloseoutResult | ProductCodeResult] = []
        while len(actions) < max_actions:
            context = resolve_manifest_command_context(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
            )
            audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
            entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
            workflow_action, blockers = next_action_blockers(
                entry,
                milestone,
                planning=context.planning,
                track=context.track,
                roadmap=context.roadmap,
            )
            if "product_code" in allow_set and entry.milestone_type in {"docs_only", "design_only", "closeout"}:
                raise WorkflowError(
                    f"{entry.milestone_id}: product_code cannot run for docs, design, or governance milestones"
                )
            if workflow_action == "track_expansion_required":
                result = apply_auto_safe_track_expansion(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow={"auto_safe"} if "auto_safe" in allow_set else set(),
                    run_validations=True,
                )
                actions.append(result)
                if "agent_design" not in allow_set:
                    break
                continue
            if workflow_action == "design_first" and agent_closeout_pending(entry):
                if "agent_closeout" not in allow_set:
                    raise WorkflowError(
                        f"{entry.milestone_id}: closeout is the next legal action; rerun with --allow agent_closeout"
                    )
                result = apply_agent_closeout(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=allow_set,
                    deny=deny_set,
                    run_validations=True,
                )
                actions.append(result)
                break
            if workflow_action == "design_first" and "agent_design" in allow_set:
                result = apply_agent_design(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=allow_set,
                    deny=deny_set,
                    run_validations=True,
                )
                actions.append(result)
                break
            if workflow_action == "write_implementation_contract":
                if "product_code" not in allow_set:
                    raise WorkflowError(f"{entry.milestone_id}: product_code is the next legal action; rerun with --allow product_code")
                result = apply_product_code(
                    context,
                    allow=allow_set,
                    run_validations=True,
                )
                actions.append(result)
                break
            if blockers:
                raise WorkflowError("\n".join(blockers))
            raise WorkflowError(f"{entry.milestone_id}: no permitted runner action for workflow action {workflow_action}")

        if not actions:
            raise WorkflowError("Manifest Runner did not apply any action")
        for result in actions:
            if isinstance(result, AutoSafeExpansionResult):
                console.print("[green]Manifest Runner V1 applied one auto_safe Track Expansion action.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Created/linked WR: {result.wr_id}")
                console.print(f"Production source: {repo_path(result.production_source)}")
                console.print(f"Roadmap deferred source: {repo_path(result.roadmap_deferred_source)}")
            elif isinstance(result, AgentDesignResult):
                console.print("[green]Manifest Runner V2 applied one agent_design action.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Owning WR: {result.wr_id}")
                console.print(f"Plan path: {repo_path(result.plan_path)}")
                console.print("Design docs:")
                for design_path in result.design_paths:
                    console.print(f"- {repo_path(design_path)}")
            elif isinstance(result, AgentCloseoutResult):
                console.print("[green]Manifest Runner V3 applied one agent_closeout action.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Closed WR: {result.wr_id}")
                console.print(f"Closeout path: {repo_path(result.closeout_path)}")
                console.print(f"Production source: {repo_path(result.production_source)}")
                console.print(f"Roadmap archive source: {repo_path(result.roadmap_archive_source)}")
                console.print(f"Roadmap deferred source: {repo_path(result.roadmap_deferred_source)}")
            else:
                console.print("[green]Manifest Runner V4 verified one product_code implementation gate.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Owning WR: {result.wr_id}")
                console.print(f"Plan path: {repo_path(result.plan_path)}")
            if result.validation_commands:
                console.print("Validation commands:")
                for command_result in result.validation_commands:
                    console.print(f"- {command_result}")
            console.print(f"Next legal action: {result.next_legal_action}")
        console.print("Must stop after this action: yes")
    except WorkflowError as error:
        console.print("[red]production:run-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("next")
def next_action(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-UI-PROGRAM."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
) -> None:
    try:
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        audit_errors = audit_manifest(context.loaded, track=context.track, roadmap=context.roadmap)
        if audit_errors:
            print_manifest_audit_blockers(audit_errors)
            raise typer.Exit(1)
        entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
        workflow_action, blockers = next_action_blockers(
            entry,
            milestone,
            planning=context.planning,
            track=context.track,
            roadmap=context.roadmap,
        )
        console.print(f"Manifest: {repo_path(context.loaded.path)}")
        console.print(f"Current milestone: {entry.milestone_id} - {entry.title}")
        console.print(f"Next legal action: {entry.next_legal_action}")
        console.print(f"Workflow action: {workflow_action}")
        console.print(f"Implementation authorized now: {implementation_authorization_note(entry, workflow_action, blockers)}")
        console.print("Must stop after this action: yes")
        if blockers:
            console.print("Unmet gates:")
            for blocker in blockers:
                console.print(f"- {blocker}")
            if workflow_action != "track_expansion_required" or any(
                "Track Expansion must create or link" not in blocker for blocker in blockers
            ):
                raise typer.Exit(1)
    except WorkflowError as error:
        console.print("[red]production:next failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("audit-track")
def audit_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-UI-PROGRAM."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
) -> None:
    try:
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        errors = audit_manifest(context.loaded, track=context.track, roadmap=context.roadmap)
        console.print(f"Manifest: {repo_path(context.loaded.path)}")
        if errors:
            print_manifest_audit_blockers(errors)
            raise typer.Exit(1)
        console.print("[green]manifest audit passed[/green]")
    except WorkflowError as error:
        console.print("[red]production:audit-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("_commands", hidden=True)
def commands() -> None:
    """Keep Typer in multi-command mode so public subcommands are stable."""
    console.print("plan-track expand-track run-track next audit-track")


if __name__ == "__main__":
    app()
