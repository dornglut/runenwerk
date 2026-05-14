#!/usr/bin/env python3
"""
Structured roadmap and batch state for Runenwerk workflow automation.

File: tools/workflow/roadmap_state.py
Module: roadmap_state
"""

from __future__ import annotations

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
SCHEMA_DIR = REPO_ROOT / "docs-site/src/content/docs/workspace/schemas"
ROADMAP_SCHEMA = SCHEMA_DIR / "roadmap-items.schema.json"
BATCH_SCHEMA = SCHEMA_DIR / "batch-manifest.schema.json"

ALLOWED_EFFORTS = {1, 2, 3, 5, 8, 13}
ALLOWED_CONFIDENCE = {1.0, 0.8, 0.5, 0.3}
ID_PATTERN = re.compile(r"^WR-\d{3}$")

Level = Literal["L0", "L1", "L2", "L3", "L4"]
Status = Literal["implement_now", "ready_next", "blocked_deferred"]
Priority = Literal["P0", "P1", "P2", "P3"]
ApprovalState = Literal["proposed", "approved", "rejected"]


class WorkflowError(ValueError):
    """Raised when workflow state is structurally invalid."""


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


class RoadmapMeta(StrictModel):
    title: str
    last_reviewed: str
    owner: str


class RenderTargets(StrictModel):
    decision_register: str
    dependency_roadmap: str
    triage: str


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


class RoadmapItem(StrictModel):
    id: str
    title: str
    diagram_title: str
    alias: str
    lane: str
    dependency_level: Level
    gate: str
    status: Status
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
    diagram_call: list[str] = Field(default_factory=list)
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
        return self.status == "implement_now" and self.blocker <= 2 and not self.is_policy_deferred

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


class BatchItem(StrictModel):
    id: str
    title: str
    lane: str
    dependency_level: Level
    gate: str
    score: float
    branch: str
    worktree: str = ""
    prompt_path: str
    status: Literal["proposed", "approved", "running", "completed", "rejected"] = "proposed"
    write_scopes: list[str] = Field(default_factory=list)
    validations: list[str] = Field(default_factory=list)

    @field_validator("id")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not ID_PATTERN.fullmatch(value):
            raise ValueError("batch item id must match WR-000")
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
    return RoadmapState.model_validate(data)


def validate_roadmap_with_json_schema(path: Path = ROADMAP_SOURCE) -> None:
    data = load_yaml(path)
    try:
        validate_json_schema(instance=data, schema=RoadmapState.model_json_schema())
    except ValidationError as error:
        raise WorkflowError(f"JSON Schema validation failed for {repo_path(path)}: {error.message}") from error


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
        candidates = list(roadmap.items)
        if level:
            candidates = [item for item in candidates if item.dependency_level == level]

    selected: list[RoadmapItem] = []
    for item in candidates:
        if item.is_policy_deferred:
            continue
        if include_discovery:
            if item.blocker <= 4 and item.status != "blocked_deferred":
                selected.append(item)
        elif item.can_enter_implementation_batch:
            selected.append(item)

    return sorted(selected, key=lambda item: (item.level_number, item.lane, -item.score, item.id))


def validate_write_scopes(items: list[RoadmapItem] | list[BatchItem]) -> list[str]:
    conflicts: list[str] = []
    seen: list[tuple[str, str]] = []
    for item in items:
        for scope in item.write_scopes:
            normalized = normalize_repo_path(scope)
            for other_item_id, other_scope in seen:
                if scope_overlaps(normalized, other_scope):
                    conflicts.append(f"{item.id}:{normalized} overlaps {other_item_id}:{other_scope}")
            seen.append((item.id, normalized))
    return conflicts


def validate_existing_write_scope_paths(items: list[RoadmapItem] | list[BatchItem]) -> list[str]:
    errors: list[str] = []
    for item in items:
        for scope in item.write_scopes:
            normalized = normalize_repo_path(scope)
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
        if not roadmap_item.can_enter_implementation_batch:
            errors.append(f"{batch_item.id}: roadmap gate is not implementation-ready")
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
    normalized_scopes = [normalize_repo_path(scope) for scope in scopes]
    violations: list[str] = []
    for path in paths:
        normalized = normalize_repo_path(path)
        if not any(path_within_scope(normalized, scope) for scope in normalized_scopes):
            violations.append(normalized)
    return violations


def changed_files_for_worktree(worktree: Path, base_sha: str) -> list[str]:
    status_completed = subprocess.run(
        ["git", "-C", str(worktree), "-c", "core.longpaths=true", "status", "--porcelain=v1", "--untracked-files=all"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    commands = [
        ["diff", "--name-only", f"{base_sha}...HEAD"],
        ["diff", "--name-only", "--cached"],
        ["diff", "--name-only"],
        ["ls-files", "--others", "--exclude-standard"],
    ]
    changed: list[str] = []
    seen: set[str] = set()

    for line in status_completed.stdout.splitlines():
        for raw_path in porcelain_status_paths(line):
            path = normalize_repo_path(raw_path)
            if path and path not in seen:
                changed.append(path)
                seen.add(path)

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
    return changed


def porcelain_status_paths(line: str) -> list[str]:
    if len(line) < 4:
        return []
    raw_path = line[3:].strip()
    if not raw_path:
        return []
    if " -> " in raw_path:
        return [part.strip().strip('"') for part in raw_path.split(" -> ", maxsplit=1)]
    return [raw_path.strip('"')]


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
    if conflicts or missing_scope_paths:
        console.print("[red]roadmap validation failed[/red]")
        for conflict in conflicts:
            console.print(f"- write-scope conflict: {conflict}")
        for missing_scope_path in missing_scope_paths:
            console.print(f"- write-scope path missing: {missing_scope_path}")
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
