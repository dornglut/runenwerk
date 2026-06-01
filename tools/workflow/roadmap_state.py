#!/usr/bin/env python3
"""
Structured roadmap and batch state for Runenwerk workflow automation.

File: tools/workflow/roadmap_state.py
Module: roadmap_state
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Annotated, Literal

import typer
import yaml
from jsonschema import ValidationError, validate as validate_json_schema
from pydantic import BaseModel, ConfigDict, Field, computed_field, field_validator, model_validator
from rich.console import Console
from rich.table import Table


REPO_ROOT = Path(__file__).resolve().parents[2]
ROADMAP_SOURCE = REPO_ROOT / "docs-site/src/content/docs/workspace/roadmap-items.yaml"
ROADMAP_ARCHIVE_SOURCE = REPO_ROOT / "docs-site/src/content/docs/workspace/roadmap-archive.yaml"
ROADMAP_DEFERRED_SOURCE = REPO_ROOT / "docs-site/src/content/docs/workspace/roadmap-deferred.yaml"
SCHEMA_DIR = REPO_ROOT / "docs-site/src/content/docs/workspace/schemas"
ROADMAP_SCHEMA = SCHEMA_DIR / "roadmap-items.schema.json"
ROADMAP_ITEM_SOURCE_SCHEMA = SCHEMA_DIR / "roadmap-item-source.schema.json"
BATCH_SCHEMA = SCHEMA_DIR / "batch-manifest.schema.json"

ALLOWED_EFFORTS = {1, 2, 3, 5, 8, 13}
ALLOWED_CONFIDENCE = {1.0, 0.8, 0.5, 0.3}
ID_PATTERN = re.compile(r"^WR-\d{3}$")
COMPLETION_EVIDENCE_PATTERN = re.compile(
    r"docs-site/src/content/docs/reports/(?:closeouts|batches)/[^\s`\"'<>)\]]+"
)
COMPLETION_EVIDENCE_ROOTS = (
    "docs-site/src/content/docs/reports/closeouts/",
    "docs-site/src/content/docs/reports/batches/",
)
COMPLETED_BATCH_INTEGRATION_STATUSES = {"merged", "integrated"}
NEW_WRITE_SCOPE_PREFIXES = ("new:", "create:")

Level = Literal["L0", "L1", "L2", "L3", "L4"]
PlanningState = Literal["current_candidate", "support_only", "ready_next", "completed", "blocked_deferred"]
PromotionPreflightStatus = Literal["promotable", "needs_switch", "metadata_blocked", "hard_blocked"]
Priority = Literal["P0", "P1", "P2", "P3"]
ApprovalState = Literal["proposed", "approved", "rejected"]
BatchItemStatus = Literal["proposed", "approved", "running", "slice_completed", "integrated", "roadmap_closed", "rejected"]
RoadmapOutcome = Literal["unknown", "roadmap_completed", "slice_landed_item_still_current", "deferred_followup_required"]
DecisionGateKind = Literal["adr", "design", "roadmap", "doc"]
DecisionGateAppliesTo = Literal["implementation", "discovery"]
CompletionQuality = Literal[
    "not_applicable",
    "bounded_contract",
    "runtime_proven",
    "proof_slice_runtime_proven",
    "architecture_runtime_proven",
    "perfectionist_verified",
]


class WorkflowError(ValueError):
    """Raised when workflow state is structurally invalid."""


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


@dataclass(frozen=True)
class PromotionPreflightResult:
    item_id: str
    target_state: PlanningState
    status: PromotionPreflightStatus
    reasons: tuple[str, ...] = ()
    suggested_command: str = ""
    blocking_current_candidates: tuple[str, ...] = ()


class RoadmapMeta(StrictModel):
    title: str
    last_reviewed: str
    owner: str


class RenderTargets(StrictModel):
    decision_register: str
    dependency_roadmap: str
    current_candidates_roadmap: str
    triage: str
    archive_register: str = "docs-site/src/content/docs/workspace/roadmap-archive-register.md"
    deferred_register: str = "docs-site/src/content/docs/workspace/roadmap-deferred-register.md"


class RoadmapEdge(StrictModel):
    source: str
    target: str
    label: str

    @field_validator("source", "target")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not ID_PATTERN.fullmatch(value):
            raise ValueError("roadmap edge endpoint must match WR-000")
        return value


class DecisionGate(StrictModel):
    kind: DecisionGateKind
    path: str
    required_status: str
    applies_to: DecisionGateAppliesTo = "implementation"
    reason: str

    @field_validator("path")
    @classmethod
    def validate_path(cls, value: str) -> str:
        normalized = value.replace("\\", "/").strip().strip("/")
        if not normalized:
            raise ValueError("decision gate path must not be empty")
        return normalized

    @field_validator("required_status", "reason")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("decision gate text fields must not be empty")
        return value.strip()


class RoadmapItem(StrictModel):
    id: str
    title: str
    diagram_title: str
    alias: str
    lane: str
    dependency_level: Level
    gate: str
    planning_state: PlanningState
    priority: Priority
    value: Annotated[int, Field(ge=1, le=5)]
    blocker: Annotated[int, Field(ge=1, le=5)]
    tc: Annotated[int, Field(ge=1, le=5)]
    rr_oe: Annotated[int, Field(ge=1, le=5)]
    du: Annotated[int, Field(ge=1, le=5)]
    effort: int
    confidence: float
    expected_score: float
    rice: str
    kano: str
    dependencies: list[str] = Field(default_factory=list)
    write_scopes: list[str] = Field(default_factory=list)
    validations: list[str] = Field(default_factory=list)
    next_evidence: str
    current_decision: str
    current_call: str = ""
    first_move: str = ""
    main_blocker: str = ""
    why_not_ready: str = ""
    completion_quality: CompletionQuality = "not_applicable"
    known_quality_gaps: list[str] = Field(default_factory=list)
    completion_audit: str = ""
    diagram_call: list[str] = Field(default_factory=list)
    decision_gates: list[DecisionGate] = Field(default_factory=list)
    ddd_owner: str
    adr_requirement: str
    fitness_function_requirement: str
    ownership_mode: str

    @field_validator("id")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not ID_PATTERN.fullmatch(value):
            raise ValueError("roadmap item id must match WR-000")
        return value

    @field_validator("dependencies")
    @classmethod
    def validate_dependencies(cls, values: list[str]) -> list[str]:
        for value in values:
            if not ID_PATTERN.fullmatch(value):
                raise ValueError("dependency ids must match WR-000")
        return values

    @field_validator("effort")
    @classmethod
    def validate_effort(cls, value: int) -> int:
        if value not in ALLOWED_EFFORTS:
            raise ValueError("effort must be one of 1, 2, 3, 5, 8, 13")
        return value

    @field_validator("confidence")
    @classmethod
    def validate_confidence(cls, value: float) -> float:
        if value not in ALLOWED_CONFIDENCE:
            raise ValueError("confidence must be one of 1.0, 0.8, 0.5, 0.3")
        return value

    @model_validator(mode="after")
    def validate_expected_score(self) -> RoadmapItem:
        if abs(self.score - self.expected_score) > 0.05:
            raise ValueError(f"{self.id} expected_score={self.expected_score:.1f} does not match computed {self.score:.1f}")
        if self.planning_state == "ready_next" and not self.main_blocker.strip():
            raise ValueError(f"{self.id} ready_next items must set main_blocker")
        return self

    @computed_field
    @property
    def score(self) -> float:
        return round(((self.value + self.tc + self.rr_oe + self.du) * self.confidence) / self.effort, 1)

    @property
    def level_number(self) -> int:
        return int(self.dependency_level[1:])

    @property
    def is_policy_deferred(self) -> bool:
        return self.blocker == 5 or "policy deferred" in self.gate.lower()

    @property
    def can_enter_implementation_batch(self) -> bool:
        return (
            self.planning_state == "current_candidate"
            and self.blocker <= 2
            and not self.is_policy_deferred
            and not decision_gate_errors(self, applies_to="implementation")
        )

    @property
    def can_enter_discovery_batch(self) -> bool:
        return self.planning_state in {"current_candidate", "ready_next"} and self.blocker <= 4 and not self.is_policy_deferred

    @property
    def value_label(self) -> str:
        return f"V{self.value}"

    @property
    def blocker_label(self) -> str:
        return f"B{self.blocker}"


class RoadmapState(StrictModel):
    version: int
    roadmap: RoadmapMeta
    render: RenderTargets
    items: list[RoadmapItem]
    edges: list[RoadmapEdge]
    archived_item_ids: list[str] = Field(default_factory=list)
    deferred_item_ids: list[str] = Field(default_factory=list)
    split_sources_enabled: bool = False

    @model_validator(mode="after")
    def validate_graph(self) -> RoadmapState:
        seen: set[str] = set()
        duplicate_ids: list[str] = []
        for item in self.items:
            if item.id in seen:
                duplicate_ids.append(item.id)
            seen.add(item.id)
        if duplicate_ids:
            raise ValueError(f"duplicate roadmap item ids: {', '.join(sorted(set(duplicate_ids)))}")

        item_ids = {item.id for item in self.items}
        archived_ids = set(self.archived_item_ids)
        deferred_ids = set(self.deferred_item_ids)
        source_errors: list[str] = []
        unknown_archived = sorted(archived_ids - item_ids)
        unknown_deferred = sorted(deferred_ids - item_ids)
        overlap = sorted(archived_ids & deferred_ids)
        if unknown_archived:
            source_errors.append(f"archived item ids are not present: {', '.join(unknown_archived)}")
        if unknown_deferred:
            source_errors.append(f"deferred item ids are not present: {', '.join(unknown_deferred)}")
        if overlap:
            source_errors.append(f"item ids appear in both archive and deferred sources: {', '.join(overlap)}")
        if self.split_sources_enabled:
            for item in self.items:
                if item.id in archived_ids and item.planning_state != "completed":
                    source_errors.append(f"{item.id}: archive source items must be completed")
                elif item.id in deferred_ids and item.planning_state != "blocked_deferred":
                    source_errors.append(f"{item.id}: deferred source items must be blocked_deferred")
                elif item.id not in archived_ids and item.id not in deferred_ids and item.planning_state in {
                    "completed",
                    "blocked_deferred",
                }:
                    source_errors.append(
                        f"{item.id}: active roadmap source must not contain {item.planning_state} items"
                    )
        if source_errors:
            raise ValueError("; ".join(source_errors))

        dependency_map = {item.id: set(item.dependencies) for item in self.items}
        errors: list[str] = []
        for item in self.items:
            for dependency in item.dependencies:
                if dependency not in item_ids:
                    errors.append(f"{item.id}: unknown dependency {dependency}")
        for edge in self.edges:
            if edge.source not in item_ids:
                errors.append(f"edge source {edge.source} does not exist")
            if edge.target not in item_ids:
                errors.append(f"edge target {edge.target} does not exist")
            if edge.target in dependency_map and edge.source not in dependency_map[edge.target]:
                errors.append(f"edge {edge.source}->{edge.target} is missing from {edge.target}.dependencies")
        if errors:
            raise ValueError("; ".join(errors))
        return self

    @property
    def by_id(self) -> dict[str, RoadmapItem]:
        return {item.id: item for item in self.items}

    @property
    def active_items(self) -> list[RoadmapItem]:
        archived_ids = set(self.archived_item_ids)
        deferred_ids = set(self.deferred_item_ids)
        return [item for item in self.items if item.id not in archived_ids and item.id not in deferred_ids]

    @property
    def archived_items(self) -> list[RoadmapItem]:
        archived_ids = set(self.archived_item_ids)
        return [item for item in self.items if item.id in archived_ids]

    @property
    def deferred_items(self) -> list[RoadmapItem]:
        deferred_ids = set(self.deferred_item_ids)
        return [item for item in self.items if item.id in deferred_ids]


class RoadmapItemSource(StrictModel):
    version: int
    roadmap: RoadmapMeta
    items: list[RoadmapItem]


class BatchItem(StrictModel):
    id: str
    title: str
    lane: str
    dependency_level: Level
    gate: str
    score: float
    branch: str
    worktree: str = ""
    worktree_cleanup: str = ""
    prompt_path: str
    status: BatchItemStatus = "proposed"
    roadmap_outcome: RoadmapOutcome = "unknown"
    write_scopes: list[str] = Field(default_factory=list)
    validations: list[str] = Field(default_factory=list)

    @field_validator("id")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not ID_PATTERN.fullmatch(value):
            raise ValueError("batch item id must match WR-000")
        return value

    @field_validator("status", mode="before")
    @classmethod
    def migrate_completed_status(cls, value: str) -> str:
        if value == "completed":
            return "slice_completed"
        return value


class BatchManifest(StrictModel):
    id: str
    goal: str
    approval_state: ApprovalState = "proposed"
    base_branch: str
    base_sha: str
    execution_mode: Literal["worktree", "shared-workspace"] = "worktree"
    integration_risk: str
    integration_status: str = "not_started"
    closeout_status: str = "not_started"
    integrated_target: str = ""
    integrated_sha: str = ""
    validation_results: list[str] = Field(default_factory=list)
    roadmap_evidence_updates: list[str] = Field(default_factory=list)
    tooling_hardening: list[str] = Field(default_factory=list)
    items: list[BatchItem]

    @model_validator(mode="after")
    def validate_batch_ids(self) -> BatchManifest:
        ids = [item.id for item in self.items]
        duplicates = sorted({item_id for item_id in ids if ids.count(item_id) > 1})
        if duplicates:
            raise ValueError(f"duplicate batch item ids: {', '.join(duplicates)}")
        return self


console = Console()
app = typer.Typer(no_args_is_help=True, help="Validate and inspect Runenwerk roadmap state.")


def load_yaml(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as source:
        data = yaml.safe_load(source)
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    return data


def load_roadmap(path: Path = ROADMAP_SOURCE) -> RoadmapState:
    data = load_yaml(path)
    return RoadmapState.model_validate(combine_roadmap_data(data, path))


def combine_roadmap_data(
    active_data: dict,
    source: Path = ROADMAP_SOURCE,
    *,
    archive_data: dict | None = None,
    deferred_data: dict | None = None,
) -> dict:
    archive_path, deferred_path = split_source_paths(source)
    split_sources_enabled = (
        archive_data is not None
        or deferred_data is not None
        or archive_path.exists()
        or deferred_path.exists()
    )
    if not split_sources_enabled:
        return active_data

    if archive_data is None:
        archive_data = load_split_item_source(archive_path) if archive_path.exists() else empty_split_source(active_data)
    if deferred_data is None:
        deferred_data = load_split_item_source(deferred_path) if deferred_path.exists() else empty_split_source(active_data)
    archived_items = list(archive_data.get("items", []))
    deferred_items = list(deferred_data.get("items", []))
    combined = dict(active_data)
    combined["items"] = [*list(active_data.get("items", [])), *archived_items, *deferred_items]
    combined["archived_item_ids"] = [item.get("id", "") for item in archived_items]
    combined["deferred_item_ids"] = [item.get("id", "") for item in deferred_items]
    combined["split_sources_enabled"] = True
    return combined


def split_source_paths(source: Path = ROADMAP_SOURCE) -> tuple[Path, Path]:
    return source.with_name("roadmap-archive.yaml"), source.with_name("roadmap-deferred.yaml")


def load_split_item_source(path: Path) -> dict:
    data = load_yaml(path)
    try:
        RoadmapItemSource.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid roadmap item source: {error}") from error
    return data


def empty_split_source(active_data: dict) -> dict:
    return {
        "version": active_data.get("version", 1),
        "roadmap": active_data.get("roadmap", {}),
        "items": [],
    }


def validate_roadmap_with_json_schema(path: Path = ROADMAP_SOURCE) -> None:
    data = load_yaml(path)
    try:
        validate_json_schema(instance=data, schema=RoadmapState.model_json_schema())
        archive_path, deferred_path = split_source_paths(path)
        for split_path in (archive_path, deferred_path):
            if split_path.exists():
                validate_json_schema(instance=load_yaml(split_path), schema=RoadmapItemSource.model_json_schema())
        load_roadmap(path)
    except ValidationError as error:
        raise WorkflowError(f"JSON Schema validation failed for {repo_path(path)}: {error.message}") from error
    except ValueError as error:
        raise WorkflowError(str(error)) from error


def load_batch_manifest(path: Path) -> BatchManifest:
    with path.open("rb") as source:
        data = tomllib.load(source)
    raw_items = data.pop("items", [])
    data["items"] = raw_items
    return BatchManifest.model_validate(data)


def render_batch_manifest(manifest: BatchManifest) -> str:
    data = manifest.model_dump(mode="json")
    items = data.pop("items")
    lines = [toml_line(key, value) for key, value in data.items()]
    lines.append("")
    for item in items:
        lines.append("[[items]]")
        lines.extend(toml_line(key, value) for key, value in item.items())
        lines.append("")
    return "\n".join(lines)


def select_batch_candidates(
    roadmap: RoadmapState,
    *,
    level: str | None = None,
    item_ids: tuple[str, ...] = (),
    include_discovery: bool = False,
) -> list[RoadmapItem]:
    by_id = roadmap.by_id
    if item_ids:
        missing = [item_id for item_id in item_ids if item_id not in by_id]
        if missing:
            raise WorkflowError(f"unknown roadmap item ids: {', '.join(missing)}")
        candidates = [by_id[item_id] for item_id in item_ids]
    else:
        candidates = list(roadmap.active_items)
        if level:
            candidates = [item for item in candidates if item.dependency_level == level]

    selected: list[RoadmapItem] = []
    for item in candidates:
        reason = batch_ineligibility_reason(item, include_discovery=include_discovery)
        if reason:
            continue
        selected.append(item)

    if item_ids and len(selected) != len(candidates):
        rejected = [
            f"{item.id}: {batch_ineligibility_reason(item, include_discovery=include_discovery)}"
            for item in candidates
            if batch_ineligibility_reason(item, include_discovery=include_discovery)
        ]
        raise WorkflowError("\n".join(rejected))

    return sorted(selected, key=lambda item: (item.level_number, item.lane, -item.score, item.id))


def batch_ineligibility_reason(item: RoadmapItem, *, include_discovery: bool = False) -> str | None:
    if include_discovery:
        if item.planning_state not in {"current_candidate", "ready_next"}:
            return f"planning_state {item.planning_state!r} is not discovery-ready"
        if item.blocker > 4:
            return f"{item.blocker_label} is above the B4 discovery gate"
        if item.is_policy_deferred:
            return f"planning_state {item.planning_state!r} is policy-deferred"
        return None
    if item.planning_state != "current_candidate":
        return f"planning_state {item.planning_state!r} is not current_candidate"
    if item.blocker > 2:
        return f"{item.blocker_label} is above the B2 implementation gate"
    if item.is_policy_deferred:
        return f"planning_state {item.planning_state!r} is policy-deferred"
    gate_errors = decision_gate_errors(item, applies_to="implementation")
    if gate_errors:
        return f"decision gate unmet: {gate_errors[0]}"
    return None


def promotion_preflight(
    roadmap: RoadmapState,
    item_id: str,
    target_state: PlanningState,
    *,
    evidence: str = "<accepted evidence>",
) -> PromotionPreflightResult:
    item = roadmap.by_id.get(item_id)
    if item is None:
        return PromotionPreflightResult(
            item_id=item_id,
            target_state=target_state,
            status="hard_blocked",
            reasons=(f"{item_id}: not present in combined roadmap sources",),
        )

    metadata_errors = promotion_metadata_errors(roadmap, item, target_state)
    if metadata_errors:
        return PromotionPreflightResult(
            item_id=item_id,
            target_state=target_state,
            status="metadata_blocked",
            reasons=tuple(metadata_errors),
        )

    post_items = items_with_planning_state(roadmap.items, item_id, target_state)
    implementation_items = [candidate for candidate in post_items if candidate.can_enter_implementation_batch]
    missing_scopes = validate_existing_write_scope_paths(implementation_items)
    if missing_scopes:
        status: PromotionPreflightStatus = (
            "metadata_blocked" if all(missing.startswith(f"{item_id}:") for missing in missing_scopes) else "hard_blocked"
        )
        return PromotionPreflightResult(
            item_id=item_id,
            target_state=target_state,
            status=status,
            reasons=tuple(f"write-scope path missing: {missing}" for missing in missing_scopes),
        )

    conflicts = validate_write_scopes(implementation_items)
    if conflicts:
        blockers = blocking_current_candidates_for_conflicts(roadmap, item_id, conflicts)
        if blockers and len(blockers) == 1 and all(conflict_involves_item(conflict, item_id) for conflict in conflicts):
            blocker = blockers[0]
            return PromotionPreflightResult(
                item_id=item_id,
                target_state=target_state,
                status="needs_switch",
                reasons=tuple(f"write-scope conflict: {conflict}" for conflict in conflicts),
                suggested_command=switch_current_command(blocker, item_id, evidence),
                blocking_current_candidates=tuple(blockers),
            )
        return PromotionPreflightResult(
            item_id=item_id,
            target_state=target_state,
            status="hard_blocked",
            reasons=tuple(f"write-scope conflict: {conflict}" for conflict in conflicts),
        )

    return PromotionPreflightResult(
        item_id=item_id,
        target_state=target_state,
        status="promotable",
        suggested_command=promotion_command(item_id, target_state, evidence),
    )


def promotion_metadata_errors(roadmap: RoadmapState, item: RoadmapItem, target_state: PlanningState) -> list[str]:
    if target_state != "current_candidate":
        return []
    errors: list[str] = []
    if item.blocker > 2:
        errors.append(f"{item.id}: {item.blocker_label} is above the B2 implementation gate")
    invalid_dependencies = [
        dependency
        for dependency in item.dependencies
        if (roadmap.by_id.get(dependency).planning_state if roadmap.by_id.get(dependency) is not None else None)
        not in {"completed", "support_only"}
    ]
    if invalid_dependencies:
        errors.append(f"{item.id}: dependencies are not completed/support context: {', '.join(invalid_dependencies)}")
    errors.extend(decision_gate_errors(item, applies_to="implementation"))
    return errors


def items_with_planning_state(items: list[RoadmapItem], item_id: str, target_state: PlanningState) -> list[RoadmapItem]:
    return [
        item.model_copy(update={"planning_state": target_state}) if item.id == item_id else item
        for item in items
    ]


def conflict_item_ids(conflict: str) -> tuple[str, str] | None:
    match = re.match(r"^(WR-\d{3}):.+ overlaps (WR-\d{3}):", conflict)
    if not match:
        return None
    return match.group(1), match.group(2)


def conflict_involves_item(conflict: str, item_id: str) -> bool:
    ids = conflict_item_ids(conflict)
    return ids is not None and item_id in ids


def blocking_current_candidates_for_conflicts(
    roadmap: RoadmapState,
    item_id: str,
    conflicts: list[str],
) -> list[str]:
    blockers: set[str] = set()
    for conflict in conflicts:
        ids = conflict_item_ids(conflict)
        if ids is None or item_id not in ids:
            continue
        other_id = ids[1] if ids[0] == item_id else ids[0]
        other = roadmap.by_id.get(other_id)
        if other is not None and other.planning_state == "current_candidate":
            blockers.add(other_id)
    return sorted(blockers)


def promotion_command(item_id: str, target_state: PlanningState, evidence: str) -> str:
    return f'task roadmap:promote -- --id {item_id} --state {target_state} --evidence "{command_arg(evidence)}"'


def switch_current_command(from_id: str, to_id: str, evidence: str) -> str:
    return f'task roadmap:switch-current -- --from {from_id} --to {to_id} --evidence "{command_arg(evidence)}"'


def command_arg(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def decision_gate_errors(item: RoadmapItem, *, applies_to: DecisionGateAppliesTo) -> list[str]:
    errors: list[str] = []
    for gate in item.decision_gates:
        if gate.applies_to != applies_to:
            continue
        errors.extend(decision_gate_status_errors(item, gate))
    return errors


def decision_gate_status_errors(item: RoadmapItem, gate: DecisionGate) -> list[str]:
    path = REPO_ROOT / gate.path
    if not path.exists():
        return [f"{item.id}: {gate.kind} gate missing {gate.path} ({gate.reason})"]
    status = document_frontmatter_status(path)
    if status is None:
        return [f"{item.id}: {gate.kind} gate {gate.path} has no frontmatter status ({gate.reason})"]
    if status.lower() != gate.required_status.lower():
        return [
            f"{item.id}: {gate.kind} gate {gate.path} status {status!r} "
            f"does not match required {gate.required_status!r} ({gate.reason})"
        ]
    return []


def document_frontmatter_status(path: Path) -> str | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = next(index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---")
    except StopIteration:
        return None
    frontmatter = yaml.safe_load("\n".join(lines[1:end])) or {}
    if not isinstance(frontmatter, dict):
        return None
    status = frontmatter.get("status")
    return str(status).strip() if status is not None else None


def validate_write_scopes(items: list[RoadmapItem] | list[BatchItem]) -> list[str]:
    conflicts: list[str] = []
    seen: list[tuple[str, str]] = []
    for item in items:
        for scope in item.write_scopes:
            normalized = normalize_write_scope_path(scope)
            for other_item_id, other_scope in seen:
                if scope_overlaps(normalized, other_scope):
                    conflicts.append(f"{item.id}:{normalized} overlaps {other_item_id}:{other_scope}")
            seen.append((item.id, normalized))
    return conflicts


def validate_existing_write_scope_paths(items: list[RoadmapItem] | list[BatchItem]) -> list[str]:
    errors: list[str] = []
    for item in items:
        for scope in item.write_scopes:
            normalized = normalize_write_scope_path(scope)
            if is_new_write_scope(scope):
                parent = (REPO_ROOT / normalized).parent
                if not parent.exists():
                    errors.append(f"{item.id}:{normalized} parent does not exist for new write scope")
                continue
            if not (REPO_ROOT / normalized).exists():
                errors.append(f"{item.id}:{normalized} does not exist")
    return errors


def validate_batch_against_roadmap(manifest: BatchManifest, roadmap: RoadmapState) -> list[str]:
    errors: list[str] = []
    roadmap_by_id = roadmap.by_id
    for batch_item in manifest.items:
        roadmap_item = roadmap_by_id.get(batch_item.id)
        if roadmap_item is None:
            errors.append(f"{batch_item.id}: not present in roadmap source")
            continue
        reason = batch_ineligibility_reason(roadmap_item)
        if reason:
            errors.append(f"{batch_item.id}: {reason}")
        if batch_item.title != roadmap_item.title:
            errors.append(f"{batch_item.id}: title is stale")
        if batch_item.lane != roadmap_item.lane:
            errors.append(f"{batch_item.id}: lane is stale")
        if batch_item.dependency_level != roadmap_item.dependency_level:
            errors.append(f"{batch_item.id}: dependency_level is stale")
        if batch_item.gate != roadmap_item.gate:
            errors.append(f"{batch_item.id}: gate is stale")
        if abs(batch_item.score - roadmap_item.score) > 0.05:
            errors.append(f"{batch_item.id}: score is stale")
        if [normalize_repo_path(scope) for scope in batch_item.write_scopes] != [
            normalize_repo_path(scope) for scope in roadmap_item.write_scopes
        ]:
            errors.append(f"{batch_item.id}: write_scopes are stale")
        if batch_item.validations != roadmap_item.validations:
            errors.append(f"{batch_item.id}: validations are stale")
    errors.extend(f"write-scope conflict: {conflict}" for conflict in validate_write_scopes(manifest.items))
    errors.extend(f"write-scope path missing: {error}" for error in validate_existing_write_scope_paths(manifest.items))
    return errors


def parse_scope_selector(scope: str) -> tuple[str | None, tuple[str, ...]]:
    cleaned = scope.strip()
    if not cleaned or cleaned == "<level/items>":
        return None, ()
    if re.fullmatch(r"L[0-4]", cleaned):
        return cleaned, ()
    return None, tuple(part.strip() for part in re.split(r"[, ]+", cleaned) if part.strip())


def write_schema_files(check: bool = False) -> list[str]:
    schemas = {
        ROADMAP_SCHEMA: RoadmapState.model_json_schema(),
        ROADMAP_ITEM_SOURCE_SCHEMA: RoadmapItemSource.model_json_schema(),
        BATCH_SCHEMA: BatchManifest.model_json_schema(),
    }
    stale: list[str] = []
    for path, schema in schemas.items():
        expected = json.dumps(schema, indent=2, sort_keys=True) + "\n"
        if check:
            if not path.exists() or path.read_text(encoding="utf-8") != expected:
                stale.append(repo_path(path))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(expected, encoding="utf-8", newline="\n")
    return stale


def validate_changed_paths(paths: list[str], scopes: list[str]) -> list[str]:
    normalized_scopes = normalized_write_scopes_with_generated_outputs(scopes)
    violations: list[str] = []
    for path in paths:
        normalized = normalize_repo_path(path)
        if not any(path_within_scope(normalized, scope) for scope in normalized_scopes):
            violations.append(normalized)
    return violations


def validate_completion_evidence(items: list[RoadmapItem], repo_root: Path = REPO_ROOT) -> list[str]:
    errors: list[str] = []
    for item in items:
        if item.planning_state != "completed":
            continue
        evidence_paths = completion_evidence_paths(item)
        if not evidence_paths:
            errors.append(
                f"{item.id}: completed items must reference an existing completed closeout or batch evidence path"
            )
            continue
        accepted_paths: list[str] = []
        for path in evidence_paths:
            evidence_error = completion_evidence_status_error(item, path, repo_root)
            if evidence_error:
                errors.append(evidence_error)
            else:
                accepted_paths.append(path)
        write_scope_paths = {normalize_write_scope_path(scope) for scope in item.write_scopes}
        if accepted_paths and not any(path in write_scope_paths for path in accepted_paths):
            errors.append(
                f"{item.id}: completed items must include a completed closeout or batch evidence path in write_scopes"
            )
    return errors


def completion_evidence_paths(item: RoadmapItem) -> list[str]:
    paths: list[str] = []
    evidence_text = " ".join(
        [
            item.next_evidence,
            item.current_decision,
            item.current_call,
            item.first_move,
        ]
    )
    for match in COMPLETION_EVIDENCE_PATTERN.finditer(evidence_text):
        append_completion_evidence_path(paths, match.group(0))
    for scope in item.write_scopes:
        append_completion_evidence_path(paths, scope)
    return paths


def append_completion_evidence_path(paths: list[str], path: str) -> None:
    normalized = normalize_completion_evidence_path(path)
    if not is_completion_evidence_path(normalized):
        return
    if normalized not in paths:
        paths.append(normalized)


def normalize_completion_evidence_path(path: str) -> str:
    return normalize_repo_path(path).rstrip(".,;:")


def is_completion_evidence_path(path: str) -> bool:
    return any(path.startswith(root) for root in COMPLETION_EVIDENCE_ROOTS)


def completion_evidence_status_error(item: RoadmapItem, path: str, repo_root: Path) -> str | None:
    evidence_path = repo_root / path
    if not evidence_path.exists():
        return f"{item.id}: completion evidence path does not exist: {path}"
    if path.startswith("docs-site/src/content/docs/reports/closeouts/"):
        status = document_frontmatter_status(evidence_path)
        if status is None:
            return f"{item.id}: completion closeout evidence has no frontmatter status: {path}"
        if status.lower() != "completed":
            return f"{item.id}: completion closeout evidence status {status!r} is not 'completed': {path}"
        return None
    manifest_path = evidence_path if evidence_path.name == "batch.toml" else evidence_path.parent / "batch.toml"
    if not manifest_path.exists():
        return f"{item.id}: completion batch evidence has no batch.toml manifest: {path}"
    try:
        manifest = load_batch_manifest(manifest_path)
    except (OSError, tomllib.TOMLDecodeError, ValueError) as error:
        return f"{item.id}: completion batch evidence is not a valid batch manifest: {repo_path(manifest_path)} ({error})"
    if manifest.integration_status not in COMPLETED_BATCH_INTEGRATION_STATUSES or manifest.closeout_status != "completed":
        return (
            f"{item.id}: completion batch evidence is not finalized: {repo_path(manifest_path)} "
            f"(integration_status={manifest.integration_status!r}, closeout_status={manifest.closeout_status!r})"
        )
    if item.id not in {batch_item.id for batch_item in manifest.items}:
        return f"{item.id}: completion batch evidence does not include this roadmap item: {repo_path(manifest_path)}"
    return None


def validate_completion_quality(items: list[RoadmapItem], repo_root: Path = REPO_ROOT) -> list[str]:
    errors: list[str] = []
    for item in items:
        if item.planning_state == "completed" and item.completion_quality == "not_applicable":
            errors.append(f"{item.id}: completed items must set completion_quality")
        if item.completion_quality == "perfectionist_verified":
            if item.known_quality_gaps:
                errors.append(f"{item.id}: perfectionist_verified items must not list known_quality_gaps")
            if not item.completion_audit.strip():
                errors.append(f"{item.id}: perfectionist_verified items must reference a completed audit")
            else:
                audit_path = repo_root / normalize_repo_path(item.completion_audit)
                if not audit_path.exists():
                    errors.append(f"{item.id}: completion_audit path does not exist: {item.completion_audit}")
                else:
                    status = document_frontmatter_status(audit_path)
                    if status is None:
                        errors.append(f"{item.id}: completion_audit has no frontmatter status: {item.completion_audit}")
                    elif status.lower() != "completed":
                        errors.append(
                            f"{item.id}: completion_audit status {status!r} is not 'completed': {item.completion_audit}"
                        )
    return errors


def validate_current_candidate_decision_gates(items: list[RoadmapItem]) -> list[str]:
    errors: list[str] = []
    for item in items:
        if item.planning_state == "current_candidate":
            errors.extend(decision_gate_errors(item, applies_to="implementation"))
    return errors


def validate_completed_items_not_current_in_docs(
    items: list[RoadmapItem],
    doc_paths: list[Path] | None = None,
) -> list[str]:
    completed_ids = [item.id for item in items if item.planning_state == "completed"]
    if not completed_ids:
        return []
    paths = doc_paths or [
        REPO_ROOT / "docs-site/src/content/docs/workspace/roadmap-index.md",
        REPO_ROOT / "docs-site/src/content/docs/workspace/repo-execution-priority-checklist.md",
        REPO_ROOT / "docs-site/src/content/docs/workspace/design-implementation-triage.md",
    ]
    errors: list[str] = []
    for path in paths:
        if not path.exists():
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            lowered = line.lower()
            if "current" not in lowered:
                continue
            for item_id in completed_ids:
                if item_id.lower() in lowered:
                    errors.append(
                        f"{repo_path(path)}:{lineno}: completed item {item_id} is still described as current work"
                    )
    return errors


def normalized_write_scopes_with_generated_outputs(scopes: list[str]) -> list[str]:
    normalized_scopes = [normalize_write_scope_path(scope) for scope in scopes]
    roadmap_source_scope = normalize_repo_path(str(ROADMAP_SOURCE.relative_to(REPO_ROOT)))
    if roadmap_source_scope in normalized_scopes:
        normalized_scopes.extend(
            path
            for path in roadmap_generated_output_paths()
            if path not in normalized_scopes
        )
    return normalized_scopes


def roadmap_generated_output_paths() -> list[str]:
    try:
        data = yaml.safe_load(ROADMAP_SOURCE.read_text(encoding="utf-8")) or {}
    except OSError:
        return []
    render = data.get("render")
    if not isinstance(render, dict):
        return []
    outputs: list[str] = []
    for key in (
        "decision_register",
        "dependency_roadmap",
        "current_candidates_roadmap",
        "triage",
        "archive_register",
        "deferred_register",
    ):
        value = render.get(key)
        if isinstance(value, str):
            outputs.append(normalize_repo_path(value))
    return outputs


def changed_files_for_worktree(worktree: Path, base_sha: str) -> list[str]:
    status_commands = [
        ["git", "-C", str(worktree), "status", "--porcelain=v1", "--untracked-files=all"],
        ["git", "-C", str(worktree), "-c", "core.longpaths=true", "status", "--porcelain=v1", "--untracked-files=all"],
    ]
    commands = [
        ["diff", "--name-only", f"{base_sha}...HEAD"],
        ["diff", "--name-only", "--cached"],
        ["diff", "--name-only"],
        ["ls-files", "--others", "--exclude-standard"],
    ]
    changed: list[str] = []
    seen: set[str] = set()
    status_entries: list[tuple[str, str]] = []

    for status_command in status_commands:
        status_completed = subprocess.run(
            status_command,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        for line in status_completed.stdout.splitlines():
            for status_code, raw_path in porcelain_status_entries(line):
                path = normalize_repo_path(raw_path)
                if path:
                    status_entries.append((status_code, path))

    for command in commands:
        completed = subprocess.run(
            ["git", "-C", str(worktree), "-c", "core.longpaths=true", *command],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        for line in completed.stdout.splitlines():
            path = normalize_repo_path(line)
            if path and path not in seen:
                changed.append(path)
                seen.add(path)
    for status_code, path in status_entries:
        if path and path not in seen and status_needs_status_only_fallback(status_code):
            changed.append(path)
            seen.add(path)
    return changed


def porcelain_status_entries(line: str) -> list[tuple[str, str]]:
    if len(line) < 4:
        return []
    status_code = line[:2]
    return [(status_code, path) for path in porcelain_status_paths(line)]


def porcelain_status_paths(line: str) -> list[str]:
    if len(line) < 4:
        return []
    raw_path = line[3:].strip()
    if not raw_path:
        return []
    if " -> " in raw_path:
        return [part.strip().strip('"') for part in raw_path.split(" -> ", maxsplit=1)]
    return [raw_path.strip('"')]


def status_needs_status_only_fallback(status_code: str) -> bool:
    return status_code == "??" or any(marker in status_code for marker in ("A", "D", "R", "C", "U"))


def validate_puml_files() -> int:
    diagrams = sorted(REPO_ROOT.rglob("*.puml"))
    if not diagrams:
        console.print("[red]no PlantUML diagrams found[/red]")
        return 1
    completed = subprocess.run(
        ["plantuml", "-checkonly", *[str(path.relative_to(REPO_ROOT)) for path in diagrams]],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if completed.stdout:
        console.print(completed.stdout, end="")
    if completed.returncode != 0 or "Some diagram description contains errors" in completed.stdout or "Error line" in completed.stdout:
        return completed.returncode or 1
    console.print(f"[green]PlantUML validation passed:[/green] {len(diagrams)} diagrams")
    return 0


def git_output(command: list[str]) -> str:
    completed = subprocess.run(command, cwd=REPO_ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    if completed.returncode != 0:
        return ""
    return completed.stdout.strip()


def repo_path(path: Path) -> str:
    try:
        return slash_path(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return slash_path(path)


def normalize_repo_path(path: str) -> str:
    return path.replace("\\", "/").strip().strip("/")


def is_new_write_scope(scope: str) -> bool:
    return scope.strip().lower().startswith(NEW_WRITE_SCOPE_PREFIXES)


def normalize_write_scope_path(scope: str) -> str:
    cleaned = scope.strip()
    lowered = cleaned.lower()
    for prefix in NEW_WRITE_SCOPE_PREFIXES:
        if lowered.startswith(prefix):
            return normalize_repo_path(cleaned[len(prefix) :])
    return normalize_repo_path(cleaned)


def slash_path(path: Path) -> str:
    return str(path).replace("\\", "/")


def path_within_scope(path: str, scope: str) -> bool:
    return path == scope or path.startswith(f"{scope}/")


def scope_overlaps(left: str, right: str) -> bool:
    return left == right or left.startswith(f"{right}/") or right.startswith(f"{left}/")


def toml_line(key: str, value: object) -> str:
    if isinstance(value, str):
        return f'{key} = "{toml_escape(value)}"'
    if isinstance(value, bool):
        return f"{key} = {str(value).lower()}"
    if isinstance(value, int | float):
        return f"{key} = {value}"
    if isinstance(value, list):
        return f"{key} = [" + ", ".join(f'"{toml_escape(str(entry))}"' for entry in value) + "]"
    raise TypeError(f"unsupported TOML value for {key}: {type(value).__name__}")


def toml_escape(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def print_items_table(items: list[RoadmapItem]) -> None:
    table = Table(title="Eligible Roadmap Items")
    table.add_column("ID")
    table.add_column("Level")
    table.add_column("Lane")
    table.add_column("Score", justify="right")
    table.add_column("Gate")
    table.add_column("Title")
    for item in items:
        table.add_row(item.id, item.dependency_level, item.lane, f"{item.score:.1f}", item.gate, item.title)
    console.print(table)


@app.command()
def validate(source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source.")) -> None:
    try:
        validate_roadmap_with_json_schema(source)
    except WorkflowError as error:
        console.print(f"[red]{error}[/red]")
        raise typer.Exit(1) from error
    roadmap = load_roadmap(source)
    conflicts = validate_write_scopes([item for item in roadmap.items if item.can_enter_implementation_batch])
    missing_scope_paths = validate_existing_write_scope_paths([item for item in roadmap.items if item.can_enter_implementation_batch])
    completion_errors = validate_completion_evidence(roadmap.items)
    quality_errors = validate_completion_quality(roadmap.items)
    gate_errors = validate_current_candidate_decision_gates(roadmap.items)
    current_doc_errors = validate_completed_items_not_current_in_docs(roadmap.items)
    if conflicts or missing_scope_paths or completion_errors or quality_errors or gate_errors or current_doc_errors:
        console.print("[red]roadmap validation failed[/red]")
        for conflict in conflicts:
            console.print(f"- write-scope conflict: {conflict}")
        for missing_scope_path in missing_scope_paths:
            console.print(f"- write-scope path missing: {missing_scope_path}")
        for completion_error in completion_errors:
            console.print(f"- completion evidence missing: {completion_error}")
        for quality_error in quality_errors:
            console.print(f"- completion quality invalid: {quality_error}")
        for gate_error in gate_errors:
            console.print(f"- decision gate unmet: {gate_error}")
        for current_doc_error in current_doc_errors:
            console.print(f"- stale current-work doc: {current_doc_error}")
        raise typer.Exit(1)
    console.print(f"[green]roadmap validation passed:[/green] {len(roadmap.items)} items, {len(roadmap.edges)} edges")


@app.command()
def schema(check: bool = typer.Option(False, "--check", help="Fail if generated schema files are stale.")) -> None:
    stale = write_schema_files(check=check)
    if stale:
        console.print("[red]schema check failed[/red]")
        for path in stale:
            console.print(f"- stale schema: {path}")
        raise typer.Exit(1)
    console.print("[green]schema check passed[/green]" if check else "[green]schema files rendered[/green]")


@app.command()
def puml() -> None:
    raise typer.Exit(validate_puml_files())


if __name__ == "__main__":
    app()
