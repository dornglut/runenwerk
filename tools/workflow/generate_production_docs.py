#!/usr/bin/env python3
"""
Generate production track Markdown and PlantUML from canonical YAML state.

File: tools/workflow/generate_production_docs.py
Module: generate_production_docs
"""

from __future__ import annotations

import re
from pathlib import Path

import typer
from rich.console import Console

from production_state import (
    PRODUCTION_SOURCE,
    ProductionMilestone,
    ProductionPlanningState,
    ProductionTrack,
    load_production_tracks,
)
from roadmap_state import REPO_ROOT, repo_path


console = Console()
app = typer.Typer(no_args_is_help=True, help="Generate Runenwerk production track docs and PUML.")


def render_outputs(planning: ProductionPlanningState) -> dict[Path, str]:
    production_index = REPO_ROOT / planning.render.production_index
    milestone_register = REPO_ROOT / planning.render.milestone_register
    track_roadmap = REPO_ROOT / planning.render.track_roadmap
    return {
        production_index: render_production_index(planning),
        milestone_register: render_milestone_register(planning),
        track_roadmap: render_track_diagram(planning),
    }


def render_production_index(planning: ProductionPlanningState) -> str:
    lines = [
        "---",
        "title: Production Track Index",
        "description: Generated index of long-term production tracks and their milestone states.",
        "status: active",
        "owner: workspace",
        "layer: workspace",
        "canonical: true",
        f"last_reviewed: {planning.production.last_reviewed}",
        "related:",
        "  - ./production-track-planning-model.md",
        "  - ./production-tracks.yaml",
        "  - ./production-milestone-register.md",
        "  - ./roadmap-items.yaml",
        "  - ./roadmap-decision-register.md",
        "  - ./schemas/production-tracks.schema.json",
        "  - ./diagrams/production-track-roadmap.puml",
        "---",
        "",
        "# Production Track Index",
        "",
        "This page is generated from [production-tracks.yaml](./production-tracks.yaml).",
        "Do not edit it directly; update the YAML source and run `task production:render`.",
        "",
        "Production tracks guide long-term sequencing. The WR roadmap remains the",
        "dependency-checked execution graph.",
        "",
        "## Tracks",
        "",
        "| ID | Track | State | Owner | Strategic goal | Success criteria |",
        "|---|---|---|---|---|---|",
    ]
    for track in planning.tracks:
        lines.append(
            f"| {track.id} | {track.title} | {track.state} | {track.owner} | "
            f"{track.strategic_goal} | {'<br>'.join(track.success_criteria)} |"
        )
    lines.extend(["", "## Current Milestone States", ""])
    for track in planning.tracks:
        lines.extend(
            [
                f"### {track.id} - {track.title}",
                "",
                "| ID | Milestone | Kind | State | Roadmap links | Outcome |",
                "|---|---|---|---|---|---|",
            ]
        )
        for milestone in track.milestones:
            lines.append(
                f"| {milestone.id} | {milestone.title} | {milestone.kind} | {milestone.state} | "
                f"{', '.join(milestone.roadmap_links) or 'N/A'} | {milestone.outcome} |"
            )
        lines.append("")
    return "\n".join(lines)


def render_milestone_register(planning: ProductionPlanningState) -> str:
    lines = [
        "---",
        "title: Production Milestone Register",
        "description: Generated production milestone register with gates, WR links, and acceptance criteria.",
        "status: active",
        "owner: workspace",
        "layer: workspace",
        "canonical: true",
        f"last_reviewed: {planning.production.last_reviewed}",
        "related:",
        "  - ./production-track-planning-model.md",
        "  - ./production-track-index.md",
        "  - ./production-tracks.yaml",
        "  - ./roadmap-items.yaml",
        "---",
        "",
        "# Production Milestone Register",
        "",
        "This register is generated from [production-tracks.yaml](./production-tracks.yaml).",
        "Do not edit it directly.",
        "",
        "| Track | Milestone | Kind | State | Dependencies | WR links | Design gates | Evidence gates | Acceptance criteria |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for track in planning.tracks:
        for milestone in track.milestones:
            lines.append(
                "| "
                + " | ".join(
                    [
                        track.id,
                        f"{milestone.id} {milestone.title}",
                        milestone.kind,
                        milestone.state,
                        ", ".join(milestone.dependencies) or "N/A",
                        ", ".join(milestone.roadmap_links) or "N/A",
                        format_design_gates(milestone),
                        format_evidence_gates(milestone),
                        "<br>".join(milestone.acceptance_criteria) or "N/A",
                    ]
                )
                + " |"
            )
    lines.append("")
    return "\n".join(lines)


def render_track_diagram(planning: ProductionPlanningState) -> str:
    lines = [
        "@startuml",
        f"title {planning.production.title}",
        "",
        "top to bottom direction",
        "skinparam componentStyle rectangle",
        "skinparam shadowing false",
        "skinparam packageStyle rectangle",
        "skinparam defaultFontName Monospaced",
        "",
        "skinparam component {",
        "  BackgroundColor<<active>> #FFF2CC",
        "  BorderColor<<active>> #AA7700",
        "  BackgroundColor<<ready_next>> #DDEEFF",
        "  BorderColor<<ready_next>> #004488",
        "  BackgroundColor<<designing>> #EFE6FF",
        "  BorderColor<<designing>> #6741A3",
        "  BackgroundColor<<blocked>> #F4F4F4",
        "  BorderColor<<blocked>> #777777",
        "  BackgroundColor<<deferred>> #EEEEEE",
        "  BorderColor<<deferred>> #666666",
        "  FontColor<<deferred>> #666666",
        "  BackgroundColor<<completed>> #DFF3DF",
        "  BorderColor<<completed>> #267326",
        "}",
        "",
        "legend right",
        "  <b>Production Tracks</b>",
        "  Milestones express coherent long-term outcomes.",
        "  WR links remain the legal implementation graph.",
        "  Designing milestones resolve architecture before implementation.",
        "endlegend",
        "",
    ]
    all_milestones = {milestone.id: milestone for track in planning.tracks for milestone in track.milestones}
    for track in planning.tracks:
        lines.append(f'package "{track.id} {track.title}" {{')
        for milestone in track.milestones:
            lines.append(render_milestone_component(milestone))
            lines.append("")
        lines.append("}")
        lines.append("")
    for milestone in all_milestones.values():
        for dependency in milestone.dependencies:
            lines.append(f"{component_alias(dependency)} -down-> {component_alias(milestone.id)} : production prerequisite")
    lines.extend(["", "@enduml", ""])
    return "\n".join(lines)


def render_milestone_component(milestone: ProductionMilestone) -> str:
    label_lines = [
        f"{milestone.id} {milestone.title}",
        f"kind={milestone.kind}",
        f"state={milestone.state}",
        "WR=" + (", ".join(milestone.roadmap_links) or "N/A"),
    ]
    label = "\\n".join(label_lines).replace('"', '\\"')
    return f'  component "{label}" as {component_alias(milestone.id)} <<{milestone.state}>>'


def format_design_gates(milestone: ProductionMilestone) -> str:
    if not milestone.design_gates:
        return "N/A"
    return "<br>".join(f"{gate.kind}:{gate.path} requires {gate.required_status}" for gate in milestone.design_gates)


def format_evidence_gates(milestone: ProductionMilestone) -> str:
    if not milestone.evidence_gates:
        return "N/A"
    return "<br>".join(f"{gate.path} requires {gate.required_status}" for gate in milestone.evidence_gates)


def component_alias(identifier: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", identifier)


def write_outputs(outputs: dict[Path, str]) -> None:
    for path, content in outputs.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8", newline="\n")


def stale_outputs(outputs: dict[Path, str]) -> list[str]:
    stale: list[str] = []
    for path, expected in outputs.items():
        if not path.exists() or path.read_text(encoding="utf-8") != expected:
            stale.append(repo_path(path))
    return stale


@app.command()
def render(source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source.")) -> None:
    planning = load_production_tracks(source)
    write_outputs(render_outputs(planning))
    console.print("[green]production docs rendered[/green]")


@app.command()
def check(source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source.")) -> None:
    planning = load_production_tracks(source)
    stale = stale_outputs(render_outputs(planning))
    if stale:
        console.print("[red]production render check failed[/red]")
        for path in stale:
            console.print(f"- stale generated file: {path}")
        raise typer.Exit(1)
    console.print("[green]production render check passed[/green]")


if __name__ == "__main__":
    app()
