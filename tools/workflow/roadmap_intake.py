#!/usr/bin/env python3
"""
Create and apply roadmap intake proposals.

File: tools/workflow/roadmap_intake.py
Module: roadmap_intake
"""

from __future__ import annotations

import datetime as dt
import re
from pathlib import Path

import typer
import yaml
from pydantic import Field
from rich.console import Console

from generate_roadmap_docs import render_outputs, stale_outputs, write_outputs
from roadmap_state import (
    ID_PATTERN,
    REPO_ROOT,
    ROADMAP_SOURCE,
    PlanningState,
    RoadmapEdge,
    RoadmapItem,
    RoadmapState,
    StrictModel,
    WorkflowError,
    decision_gate_errors,
    load_roadmap,
    load_yaml,
    normalize_repo_path,
    repo_path,
    validate_existing_write_scope_paths,
    validate_puml_files,
    validate_write_scopes,
    write_schema_files,
)


DEFAULT_INTAKE_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/roadmap-intake"

console = Console()
app = typer.Typer(no_args_is_help=True, help="Create and apply Runenwerk roadmap intake proposals.")


class RoadmapIntakeProposal(StrictModel):
    version: int = 1
    idea: str
    source_path: str = ""
    owner: str = ""
    created_at: str
    governance_notes: list[str]
    open_questions: list[str]
    item: RoadmapItem
    edges: list[RoadmapEdge] = Field(default_factory=list)


@app.command()
def intake(
    idea: str = typer.Option(..., help="Design or change idea to prepare for roadmap review."),
    from_path: Path | None = typer.Option(None, "--from", help="Optional source design/doc path."),
    owner: str = typer.Option("", help="Optional owning domain, module, or path."),
    out: Path | None = typer.Option(None, help="Proposal output directory."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
) -> None:
    roadmap = load_roadmap(source)
    proposal = build_intake_proposal(
        roadmap,
        idea=idea,
        source_path=repo_path(from_path) if from_path else "",
        owner=owner,
    )
    output_dir = out or default_intake_dir(idea)
    write_intake_proposal(proposal, output_dir)
    console.print(f"[green]wrote roadmap intake proposal:[/green] {repo_path(output_dir / 'proposal.yaml')}")
    console.print("")
    console.print("[bold]Next commands[/bold]")
    console.print(f"task roadmap:apply-intake -- --proposal {repo_path(output_dir / 'proposal.yaml')}", soft_wrap=True)
    console.print("task roadmap:check", soft_wrap=True)
    console.print("task puml:validate", soft_wrap=True)


@app.command("apply-intake")
def apply_intake(
    proposal: Path = typer.Option(..., help="Roadmap intake proposal YAML."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
    skip_checks: bool = typer.Option(False, help="Skip roadmap and PUML checks after writing."),
) -> None:
    try:
        apply_intake_proposal(proposal, source=source, skip_checks=skip_checks)
    except WorkflowError as error:
        console.print("[red]roadmap intake apply failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error
    console.print(f"[green]applied roadmap intake:[/green] {repo_path(proposal)}")


@app.command()
def promote(
    id: str = typer.Option(..., help="Roadmap item ID to promote."),
    state: PlanningState = typer.Option(..., help="New planning_state."),
    evidence: str = typer.Option(..., help="Evidence justifying the state change."),
    source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source."),
    skip_checks: bool = typer.Option(False, help="Skip roadmap and PUML checks after writing."),
) -> None:
    try:
        promote_roadmap_item(id, state=state, evidence=evidence, source=source, skip_checks=skip_checks)
    except WorkflowError as error:
        console.print("[red]roadmap promotion failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error
    console.print(f"[green]promoted roadmap item:[/green] {id} -> {state}")


def build_intake_proposal(
    roadmap: RoadmapState,
    *,
    idea: str,
    source_path: str = "",
    owner: str = "",
) -> RoadmapIntakeProposal:
    item_id = next_roadmap_id(roadmap)
    title = title_from_idea(idea)
    write_scopes = [normalize_repo_path(owner)] if owner and looks_like_repo_path(owner) else []
    item = RoadmapItem(
        id=item_id,
        title=title,
        diagram_title=title[:48],
        alias=item_id.replace("-", ""),
        lane="Discovery",
        dependency_level="L4",
        gate="Policy deferred pending intake approval",
        planning_state="blocked_deferred",
        priority="P3",
        value=2,
        blocker=5,
        tc=1,
        rr_oe=2,
        du=1,
        effort=8,
        confidence=0.3,
        expected_score=0.2,
        rice="N/A",
        kano="Unknown",
        dependencies=[],
        write_scopes=write_scopes,
        validations=[],
        next_evidence="Review intake proposal, ownership, dependency gates, and accepted design evidence.",
        current_decision="Intake proposal only; do not implement until applied and promoted by roadmap review.",
        current_call="Keep as roadmap intake until ownership, dependencies, and validation evidence are clear.",
        first_move="Run architecture governance review, then edit and apply this intake proposal if accepted.",
        main_blocker="Missing accepted roadmap/design evidence.",
        why_not_ready="New idea has not passed roadmap intake, architecture governance, and dependency review.",
        diagram_call=["intake proposal", "await approval"],
        ddd_owner=owner or "TBD",
        adr_requirement="Assess during architecture governance review.",
        fitness_function_requirement="Define before promotion to current_candidate.",
        ownership_mode="TBD",
    )
    return RoadmapIntakeProposal(
        idea=idea,
        source_path=source_path,
        owner=owner,
        created_at=dt.date.today().isoformat(),
        governance_notes=[
            "Run architecture governance review before implementation.",
            "Confirm Clean Architecture dependency direction and DDD owner.",
            "Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.",
        ],
        open_questions=[
            "What accepted design, ADR, or closeout evidence justifies promotion?",
            "Which existing WR items does this depend on?",
            "Which exact write scopes and validation commands will bound implementation?",
        ],
        item=item,
        edges=[],
    )


def write_intake_proposal(proposal: RoadmapIntakeProposal, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "proposal.yaml").write_text(
        yaml.safe_dump(proposal_to_yaml_data(proposal), sort_keys=False, allow_unicode=False, width=120),
        encoding="utf-8",
        newline="\n",
    )
    (output_dir / "proposal.md").write_text(render_intake_markdown(proposal), encoding="utf-8", newline="\n")


def proposal_to_yaml_data(proposal: RoadmapIntakeProposal) -> dict:
    data = proposal.model_dump(mode="json")
    if isinstance(data.get("item"), dict):
        data["item"].pop("score", None)
    return data


def render_intake_markdown(proposal: RoadmapIntakeProposal) -> str:
    item = proposal.item
    lines = [
        "---",
        f"title: Roadmap Intake {item.id}",
        "description: Roadmap intake proposal generated from a new idea.",
        "status: draft",
        "owner: workspace",
        "layer: workspace",
        "canonical: false",
        f"last_reviewed: {proposal.created_at}",
        "---",
        "",
        f"# Roadmap Intake {item.id}",
        "",
        f"Idea: {proposal.idea}",
        f"Suggested title: {item.title}",
        f"Initial planning state: `{item.planning_state}`",
        "",
        "## Governance Notes",
        "",
    ]
    lines.extend(f"- {note}" for note in proposal.governance_notes)
    lines.extend(["", "## Open Questions", ""])
    lines.extend(f"- {question}" for question in proposal.open_questions)
    lines.extend(
        [
            "",
            "## Apply Command",
            "",
            "```text",
            "task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def load_intake_proposal(path: Path) -> RoadmapIntakeProposal:
    data = load_yaml(path)
    try:
        return RoadmapIntakeProposal.model_validate(data)
    except ValueError as error:
        raise WorkflowError(str(error)) from error


def apply_intake_proposal(proposal_path: Path, *, source: Path = ROADMAP_SOURCE, skip_checks: bool = False) -> RoadmapState:
    proposal = load_intake_proposal(proposal_path)
    data = load_yaml(source)
    updated_data = roadmap_data_with_proposal(data, proposal)
    try:
        roadmap = RoadmapState.model_validate(updated_data)
    except ValueError as error:
        raise WorkflowError(str(error)) from error
    errors = validate_intake_item_scopes(proposal.item)
    if errors:
        raise WorkflowError("\n".join(errors))
    write_roadmap_data(source, updated_data)
    render_and_check(roadmap, skip_checks=skip_checks)
    return roadmap


def roadmap_data_with_proposal(data: dict, proposal: RoadmapIntakeProposal) -> dict:
    item_data = proposal.item.model_dump(mode="json", exclude={"score"})
    items = list(data.get("items", []))
    replaced = False
    for index, existing in enumerate(items):
        if existing.get("id") == proposal.item.id:
            items[index] = item_data
            replaced = True
            break
    if not replaced:
        items.append(item_data)

    edges = list(data.get("edges", []))
    existing_edges = {(edge.get("source"), edge.get("target")) for edge in edges}
    for edge in proposal.edges:
        key = (edge.source, edge.target)
        if key not in existing_edges:
            edges.append(edge.model_dump(mode="json"))
            existing_edges.add(key)

    updated = dict(data)
    updated["items"] = items
    updated["edges"] = edges
    updated["roadmap"] = {**updated.get("roadmap", {}), "last_reviewed": dt.date.today().isoformat()}
    return updated


def validate_intake_item_scopes(item: RoadmapItem) -> list[str]:
    return [f"write-scope path missing: {error}" for error in validate_existing_write_scope_paths([item])]


def promote_roadmap_item(
    item_id: str,
    *,
    state: PlanningState,
    evidence: str,
    source: Path = ROADMAP_SOURCE,
    skip_checks: bool = False,
) -> RoadmapState:
    if not ID_PATTERN.fullmatch(item_id):
        raise WorkflowError("roadmap item id must match WR-000")
    if not evidence.strip():
        raise WorkflowError("promotion evidence is required")
    data = load_yaml(source)
    updated_data = roadmap_data_with_promotion(data, item_id=item_id, state=state, evidence=evidence.strip())
    try:
        roadmap = RoadmapState.model_validate(updated_data)
    except ValueError as error:
        raise WorkflowError(str(error)) from error
    conflicts = validate_write_scopes([item for item in roadmap.items if item.can_enter_implementation_batch])
    missing = validate_existing_write_scope_paths([item for item in roadmap.items if item.can_enter_implementation_batch])
    if conflicts or missing:
        errors = [f"write-scope conflict: {conflict}" for conflict in conflicts]
        errors.extend(f"write-scope path missing: {error}" for error in missing)
        raise WorkflowError("\n".join(errors))
    write_roadmap_data(source, updated_data)
    render_and_check(roadmap, skip_checks=skip_checks)
    return roadmap


def roadmap_data_with_promotion(data: dict, *, item_id: str, state: PlanningState, evidence: str) -> dict:
    items = list(data.get("items", []))
    by_id = {item.get("id"): item for item in items}
    if item_id not in by_id:
        raise WorkflowError(f"{item_id}: not present in roadmap source")
    target = dict(by_id[item_id])
    if state == "current_candidate":
        blocker = int(target.get("blocker", 5))
        if blocker > 2:
            raise WorkflowError(f"{item_id}: B{blocker} is above the B2 implementation gate")
        invalid_dependencies = [
            dependency
            for dependency in target.get("dependencies", [])
            if by_id.get(dependency, {}).get("planning_state") not in {"completed", "support_only"}
        ]
        if invalid_dependencies:
            raise WorkflowError(
                f"{item_id}: dependencies are not completed/support context: {', '.join(invalid_dependencies)}"
            )
        target_item = RoadmapItem.model_validate(target)
        gate_errors = decision_gate_errors(target_item, applies_to="implementation")
        if gate_errors:
            raise WorkflowError("\n".join(gate_errors))

    updated_items: list[dict] = []
    for item in items:
        if item.get("id") == item_id:
            promoted = dict(item)
            promoted["planning_state"] = state
            promoted["current_decision"] = evidence
            promoted["next_evidence"] = evidence
            updated_items.append(promoted)
        else:
            updated_items.append(item)
    updated = dict(data)
    updated["items"] = updated_items
    updated["roadmap"] = {**updated.get("roadmap", {}), "last_reviewed": dt.date.today().isoformat()}
    return updated


def write_roadmap_data(source: Path, data: dict) -> None:
    source.write_text(yaml.safe_dump(data, sort_keys=False, allow_unicode=False, width=120), encoding="utf-8", newline="\n")


def render_and_check(roadmap: RoadmapState, *, skip_checks: bool = False) -> None:
    write_schema_files(check=False)
    outputs = render_outputs(roadmap)
    write_outputs(outputs)
    if skip_checks:
        return
    stale = write_schema_files(check=True)
    if stale:
        raise WorkflowError("stale schema files: " + ", ".join(stale))
    stale_rendered = stale_outputs(outputs)
    if stale_rendered:
        raise WorkflowError("stale generated roadmap files: " + ", ".join(stale_rendered))
    if validate_puml_files() != 0:
        raise WorkflowError("PlantUML validation failed")


def next_roadmap_id(roadmap: RoadmapState) -> str:
    max_id = max(int(item.id.split("-")[1]) for item in roadmap.items)
    return f"WR-{max_id + 1:03d}"


def title_from_idea(idea: str) -> str:
    cleaned = " ".join(idea.strip().split())
    if not cleaned:
        raise WorkflowError("idea must not be empty")
    return cleaned[:96]


def default_intake_dir(idea: str) -> Path:
    return DEFAULT_INTAKE_ROOT / f"{dt.date.today().isoformat()}-{slugify(idea)}"


def slugify(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")[:40] or "roadmap-intake"


def looks_like_repo_path(value: str) -> bool:
    return "/" in value or "\\" in value


if __name__ == "__main__":
    app()
