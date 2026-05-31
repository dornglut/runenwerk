#!/usr/bin/env python3
"""
Structured production track state for Runenwerk workflow automation.

File: tools/workflow/production_state.py
Module: production_state
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Literal

import typer
import yaml
from jsonschema import ValidationError, validate as validate_json_schema
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator
from rich.console import Console

from roadmap_state import (
    REPO_ROOT,
    ROADMAP_SOURCE,
    SCHEMA_DIR,
    WorkflowError,
    document_frontmatter_status,
    load_roadmap,
    normalize_repo_path,
    repo_path,
)


PRODUCTION_SOURCE = REPO_ROOT / "docs-site/src/content/docs/workspace/production-tracks.yaml"
PRODUCTION_SCHEMA = SCHEMA_DIR / "production-tracks.schema.json"
TRACK_EXECUTION_MANIFEST_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/track-execution-manifests"

TRACK_ID_PATTERN = re.compile(r"^PT-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
MILESTONE_ID_PATTERN = re.compile(r"^PM-[A-Z0-9]+(?:-[A-Z0-9]+)*-\d{3}$")
ROADMAP_ID_PATTERN = re.compile(r"^WR-\d{3}$")

ProductionMilestoneState = Literal["designing", "ready_next", "active", "completed", "blocked", "deferred"]
ProductionMilestoneKind = Literal["design", "implementation", "hardening", "release"]
ProductionTrackState = Literal["active", "paused", "completed", "deferred"]
ProductionGateKind = Literal["adr", "design", "roadmap", "doc"]
ProductionCompletionQuality = Literal["not_applicable", "bounded_contract", "runtime_proven", "perfectionist_verified"]


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


class ProductionPlanningMeta(StrictModel):
    title: str
    last_reviewed: str
    owner: str


class ProductionRenderTargets(StrictModel):
    production_index: str
    milestone_register: str
    track_roadmap: str
    full_track_roadmap: str


class ProductionDesignGate(StrictModel):
    kind: ProductionGateKind
    path: str
    required_status: str
    reason: str

    @field_validator("path")
    @classmethod
    def validate_path(cls, value: str) -> str:
        normalized = normalize_repo_path(value)
        if not normalized:
            raise ValueError("gate path must not be empty")
        return normalized

    @field_validator("required_status", "reason")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("gate text fields must not be empty")
        return cleaned


class ProductionEvidenceGate(StrictModel):
    path: str
    required_status: str = "completed"
    reason: str

    @field_validator("path")
    @classmethod
    def validate_path(cls, value: str) -> str:
        normalized = normalize_repo_path(value)
        if not normalized:
            raise ValueError("evidence gate path must not be empty")
        return normalized

    @field_validator("required_status", "reason")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("evidence gate text fields must not be empty")
        return cleaned


class ProductionMilestone(StrictModel):
    id: str
    title: str
    kind: ProductionMilestoneKind
    state: ProductionMilestoneState
    goal: str
    outcome: str
    dependencies: list[str] = Field(default_factory=list)
    roadmap_links: list[str] = Field(default_factory=list)
    design_gates: list[ProductionDesignGate] = Field(default_factory=list)
    evidence_gates: list[ProductionEvidenceGate] = Field(default_factory=list)
    acceptance_criteria: list[str] = Field(default_factory=list)
    completion_quality: ProductionCompletionQuality = "not_applicable"
    known_quality_gaps: list[str] = Field(default_factory=list)
    completion_audit: str = ""

    @field_validator("id")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not MILESTONE_ID_PATTERN.fullmatch(value):
            raise ValueError("production milestone id must match PM-TRACK-000")
        return value

    @field_validator("dependencies")
    @classmethod
    def validate_dependencies(cls, values: list[str]) -> list[str]:
        for value in values:
            if not MILESTONE_ID_PATTERN.fullmatch(value):
                raise ValueError("production milestone dependency ids must match PM-TRACK-000")
        return values

    @field_validator("roadmap_links")
    @classmethod
    def validate_roadmap_links(cls, values: list[str]) -> list[str]:
        for value in values:
            if not ROADMAP_ID_PATTERN.fullmatch(value):
                raise ValueError("roadmap links must match WR-000")
        return values

    @field_validator("goal", "outcome")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("milestone goal and outcome must not be empty")
        return cleaned

    @property
    def requires_roadmap_links(self) -> bool:
        return self.kind in {"implementation", "hardening", "release"} and self.state in {
            "ready_next",
            "active",
            "completed",
        }

    @property
    def requires_design_gates_to_pass(self) -> bool:
        return self.kind in {"implementation", "hardening", "release"} and self.state in {
            "ready_next",
            "active",
            "completed",
        }


class ProductionTrack(StrictModel):
    id: str
    title: str
    state: ProductionTrackState
    owner: str
    target_completion_quality: ProductionCompletionQuality = "not_applicable"
    strategic_goal: str
    success_criteria: list[str]
    milestones: list[ProductionMilestone]

    @field_validator("id")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not TRACK_ID_PATTERN.fullmatch(value):
            raise ValueError("production track id must match PT-NAME")
        return value

    @field_validator("title", "owner", "strategic_goal")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("production track text fields must not be empty")
        return cleaned


class ProductionPlanningState(StrictModel):
    version: int
    production: ProductionPlanningMeta
    render: ProductionRenderTargets
    tracks: list[ProductionTrack]

    @model_validator(mode="after")
    def validate_graph(self) -> ProductionPlanningState:
        duplicate_tracks = sorted({track.id for track in self.tracks if [item.id for item in self.tracks].count(track.id) > 1})
        if duplicate_tracks:
            raise ValueError(f"duplicate production track ids: {', '.join(duplicate_tracks)}")

        milestone_ids: list[str] = []
        for track in self.tracks:
            milestone_ids.extend(milestone.id for milestone in track.milestones)
        duplicate_milestones = sorted({milestone_id for milestone_id in milestone_ids if milestone_ids.count(milestone_id) > 1})
        if duplicate_milestones:
            raise ValueError(f"duplicate production milestone ids: {', '.join(duplicate_milestones)}")

        graph_errors = validate_milestone_dependency_graph(self)
        if graph_errors:
            raise ValueError("; ".join(graph_errors))
        return self

    @property
    def by_milestone_id(self) -> dict[str, ProductionMilestone]:
        return {milestone.id: milestone for track in self.tracks for milestone in track.milestones}


console = Console()
app = typer.Typer(no_args_is_help=True, help="Validate and inspect Runenwerk production track state.")


def load_yaml(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as source:
        data = yaml.safe_load(source)
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    return data


def load_production_tracks(path: Path = PRODUCTION_SOURCE) -> ProductionPlanningState:
    data = load_yaml(path)
    return ProductionPlanningState.model_validate(data)


def validate_production_tracks_with_json_schema(path: Path = PRODUCTION_SOURCE) -> None:
    data = load_yaml(path)
    try:
        validate_json_schema(instance=data, schema=ProductionPlanningState.model_json_schema())
    except ValidationError as error:
        raise WorkflowError(f"JSON Schema validation failed for {repo_path(path)}: {error.message}") from error


def validate_milestone_dependency_graph(state: ProductionPlanningState) -> list[str]:
    milestone_ids = set(state.by_milestone_id)
    errors: list[str] = []
    dependency_map = {milestone.id: milestone.dependencies for milestone in state.by_milestone_id.values()}
    for milestone in state.by_milestone_id.values():
        for dependency in milestone.dependencies:
            if dependency not in milestone_ids:
                errors.append(f"{milestone.id}: unknown milestone dependency {dependency}")
    if errors:
        return errors

    visiting: set[str] = set()
    visited: set[str] = set()
    stack: list[str] = []

    def visit(milestone_id: str) -> None:
        if milestone_id in visited:
            return
        if milestone_id in visiting:
            cycle_start = stack.index(milestone_id) if milestone_id in stack else 0
            cycle = stack[cycle_start:] + [milestone_id]
            errors.append(f"production milestone dependency cycle: {' -> '.join(cycle)}")
            return
        visiting.add(milestone_id)
        stack.append(milestone_id)
        for dependency in dependency_map[milestone_id]:
            visit(dependency)
        stack.pop()
        visiting.remove(milestone_id)
        visited.add(milestone_id)

    for milestone_id in sorted(milestone_ids):
        visit(milestone_id)
    return errors


def validate_design_gates(state: ProductionPlanningState) -> list[str]:
    errors: list[str] = []
    for milestone in state.by_milestone_id.values():
        if not milestone.requires_design_gates_to_pass:
            continue
        for gate in milestone.design_gates:
            errors.extend(gate_status_errors(milestone.id, gate.kind, gate.path, gate.required_status, gate.reason))
    return errors


def validate_roadmap_links(state: ProductionPlanningState, roadmap_path: Path = ROADMAP_SOURCE) -> list[str]:
    # load_roadmap resolves the active, archive, and deferred WR sources together.
    roadmap = load_roadmap(roadmap_path)
    roadmap_ids = set(roadmap.by_id)
    errors: list[str] = []
    for milestone in state.by_milestone_id.values():
        if milestone.requires_roadmap_links and not milestone.roadmap_links:
            errors.append(f"{milestone.id}: execution-ready production milestones must link at least one WR roadmap item")
        for roadmap_id in milestone.roadmap_links:
            if roadmap_id not in roadmap_ids:
                errors.append(f"{milestone.id}: unknown roadmap link {roadmap_id}")
    return errors


def validate_evidence_gates(state: ProductionPlanningState) -> list[str]:
    errors: list[str] = []
    for milestone in state.by_milestone_id.values():
        if milestone.state != "completed":
            continue
        if not milestone.evidence_gates:
            errors.append(f"{milestone.id}: completed production milestones must include evidence gates")
            continue
        for gate in milestone.evidence_gates:
            errors.extend(gate_status_errors(milestone.id, "doc", gate.path, gate.required_status, gate.reason))
    return errors


def validate_completion_quality(
    state: ProductionPlanningState,
    roadmap_path: Path = ROADMAP_SOURCE,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    # Completed production evidence can link archived WR rows after roadmap cleanup.
    roadmap = load_roadmap(roadmap_path)
    errors: list[str] = []
    for milestone in state.by_milestone_id.values():
        if milestone.state == "completed" and milestone.completion_quality == "not_applicable":
            errors.append(f"{milestone.id}: completed production milestones must set completion_quality")
        if milestone.completion_quality == "perfectionist_verified":
            if milestone.known_quality_gaps:
                errors.append(f"{milestone.id}: perfectionist_verified milestones must not list known_quality_gaps")
            if not milestone.completion_audit.strip():
                errors.append(f"{milestone.id}: perfectionist_verified milestones must reference a completed audit")
            else:
                errors.extend(
                    gate_status_errors(
                        milestone.id,
                        "doc",
                        milestone.completion_audit,
                        "completed",
                        "perfectionist production completion requires a completed audit",
                        repo_root=repo_root,
                    )
                )
            for roadmap_id in milestone.roadmap_links:
                roadmap_item = roadmap.by_id.get(roadmap_id)
                if roadmap_item is None:
                    continue
                if roadmap_item.completion_quality != "perfectionist_verified":
                    errors.append(
                        f"{milestone.id}: perfectionist_verified milestone links {roadmap_id} "
                        f"with completion_quality={roadmap_item.completion_quality!r}"
                    )
    return errors


def validate_manifest_backed_tracks(
    state: ProductionPlanningState,
    *,
    roadmap_path: Path = ROADMAP_SOURCE,
    manifest_root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> list[str]:
    # Import locally to avoid a module import cycle: the manifest workflow also
    # imports production state models for its standalone CLI.
    from track_execution_manifest import audit_manifest, load_track_execution_manifest

    roadmap = load_roadmap(roadmap_path)
    errors: list[str] = []
    for track in state.tracks:
        manifest_path = manifest_root / f"{track.id.lower()}.yaml"
        if not manifest_path.exists():
            continue
        try:
            loaded = load_track_execution_manifest(track.id, root=manifest_root)
        except WorkflowError as error:
            errors.append(str(error))
            continue
        if loaded is None:
            continue
        errors.extend(audit_manifest(loaded, track=track, roadmap=roadmap))
        manifest_by_milestone_id = loaded.manifest.by_milestone_id
        for milestone in track.milestones:
            entry = manifest_by_milestone_id.get(milestone.id)
            if entry is None:
                errors.append(f"{track.id}: manifest missing milestone {milestone.id}")
                continue
            errors.extend(validate_manifest_backed_milestone(track.id, milestone, entry, roadmap))
        if track.state == "completed":
            for entry in loaded.manifest.milestones:
                if entry.future_wr_candidate:
                    errors.append(
                        f"{track.id}: completed manifest-backed tracks must not retain future WR candidate "
                        f"{entry.future_wr_candidate} for {entry.milestone_id}"
                    )
    return errors


def validate_manifest_backed_milestone(
    track_id: str,
    milestone: ProductionMilestone,
    entry: object,
    roadmap,
) -> list[str]:
    errors: list[str] = []
    milestone_type = getattr(entry, "milestone_type")
    if milestone_type == "docs_only" and (
        getattr(entry, "may_create_code")
        or getattr(entry, "may_create_crates")
        or getattr(entry, "may_modify_production_behavior")
    ):
        errors.append(f"{milestone.id}: docs-only manifest milestones cannot authorize code, crates, or production behavior")
    if milestone.completion_quality == "runtime_proven" and milestone_type == "docs_only":
        errors.append(f"{milestone.id}: runtime_proven milestones cannot be docs-only in a Track Execution Manifest")
    if milestone.state == "completed":
        expected_closeout_path = getattr(entry, "expected_closeout_path")
        evidence_paths = {gate.path for gate in milestone.evidence_gates}
        if expected_closeout_path not in evidence_paths and milestone.completion_audit != expected_closeout_path:
            errors.append(
                f"{milestone.id}: completed manifest-backed milestone must reference expected closeout "
                f"{expected_closeout_path}"
            )
        if milestone.completion_quality == "not_applicable":
            errors.append(f"{milestone.id}: completed manifest-backed milestone must set completion_quality")
        owning_wr = getattr(entry, "owning_wr")
        if owning_wr:
            roadmap_item = roadmap.by_id.get(owning_wr)
            if roadmap_item is None:
                errors.append(f"{milestone.id}: owning WR {owning_wr} is missing from roadmap state")
            elif roadmap_item.planning_state != "completed":
                errors.append(
                    f"{milestone.id}: completed manifest-backed milestone owns {owning_wr} "
                    f"with planning_state={roadmap_item.planning_state!r}, expected 'completed'"
                )
    if milestone.state in {"ready_next", "active", "completed"} and milestone.kind in {
        "implementation",
        "hardening",
        "release",
    }:
        if getattr(entry, "future_wr_candidate"):
            errors.append(
                f"{milestone.id}: execution-ready manifest-backed milestone must link an owning WR, "
                f"not future candidate {getattr(entry, 'future_wr_candidate')}"
            )
    if track_id and getattr(entry, "evidence_gates", None) == []:
        errors.append(f"{milestone.id}: manifest-backed milestones must declare evidence gates")
    return errors


def gate_status_errors(
    owner_id: str,
    kind: str,
    path: str,
    required_status: str,
    reason: str,
    *,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    candidate = repo_root / path
    if not candidate.exists():
        return [f"{owner_id}: {kind} gate missing {path} ({reason})"]
    status = document_frontmatter_status(candidate)
    if status is None:
        return [f"{owner_id}: {kind} gate {path} has no frontmatter status ({reason})"]
    if status.lower() != required_status.lower():
        return [
            f"{owner_id}: {kind} gate {path} status {status!r} "
            f"does not match required {required_status!r} ({reason})"
        ]
    return []


def write_production_schema_files(check: bool = False) -> list[str]:
    schema = ProductionPlanningState.model_json_schema()
    expected = json.dumps(schema, indent=2, sort_keys=True) + "\n"
    if check:
        if not PRODUCTION_SCHEMA.exists() or PRODUCTION_SCHEMA.read_text(encoding="utf-8") != expected:
            return [repo_path(PRODUCTION_SCHEMA)]
        return []
    PRODUCTION_SCHEMA.parent.mkdir(parents=True, exist_ok=True)
    PRODUCTION_SCHEMA.write_text(expected, encoding="utf-8", newline="\n")
    return []


@app.command()
def validate(
    source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
    manifest_source_root: Path = typer.Option(
        TRACK_EXECUTION_MANIFEST_ROOT,
        help="Track Execution Manifest source root.",
    ),
) -> None:
    try:
        validate_production_tracks_with_json_schema(source)
        state = load_production_tracks(source)
        errors = []
        errors.extend(validate_milestone_dependency_graph(state))
        errors.extend(validate_roadmap_links(state, roadmap_path=roadmap_source))
        errors.extend(validate_design_gates(state))
        errors.extend(validate_evidence_gates(state))
        errors.extend(validate_completion_quality(state, roadmap_path=roadmap_source))
        errors.extend(
            validate_manifest_backed_tracks(
                state,
                roadmap_path=roadmap_source,
                manifest_root=manifest_source_root,
            )
        )
    except (WorkflowError, ValueError) as error:
        console.print(f"[red]{error}[/red]")
        raise typer.Exit(1) from error
    if errors:
        console.print("[red]production track validation failed[/red]")
        for error in errors:
            console.print(f"- {error}")
        raise typer.Exit(1)
    milestone_count = sum(len(track.milestones) for track in state.tracks)
    console.print(f"[green]production track validation passed:[/green] {len(state.tracks)} tracks, {milestone_count} milestones")


@app.command()
def schema(check: bool = typer.Option(False, "--check", help="Fail if generated schema file is stale.")) -> None:
    stale = write_production_schema_files(check=check)
    if stale:
        console.print("[red]production schema check failed[/red]")
        for path in stale:
            console.print(f"- stale schema: {path}")
        raise typer.Exit(1)
    console.print("[green]production schema check passed[/green]" if check else "[green]production schema rendered[/green]")


if __name__ == "__main__":
    app()
