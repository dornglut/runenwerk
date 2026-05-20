#!/usr/bin/env python3
"""
Generate roadmap Markdown and PlantUML from canonical YAML state.

File: tools/workflow/generate_roadmap_docs.py
Module: generate_roadmap_docs
"""

from __future__ import annotations

from pathlib import Path
import re

import typer
from rich.console import Console

from roadmap_state import (
    REPO_ROOT,
    ROADMAP_SOURCE,
    RoadmapItem,
    RoadmapState,
    WorkflowError,
    load_roadmap,
    repo_path,
    select_batch_candidates,
)


TRIAGE_BEGIN = "<!-- BEGIN GENERATED ROADMAP STATUS -->"
TRIAGE_END = "<!-- END GENERATED ROADMAP STATUS -->"

console = Console()
app = typer.Typer(no_args_is_help=True, help="Generate Runenwerk roadmap docs and PUML.")


def render_outputs(roadmap: RoadmapState) -> dict[Path, str]:
    decision_register = REPO_ROOT / roadmap.render.decision_register
    dependency_roadmap = REPO_ROOT / roadmap.render.dependency_roadmap
    current_candidates_roadmap = REPO_ROOT / roadmap.render.current_candidates_roadmap
    triage = REPO_ROOT / roadmap.render.triage
    archive_register = REPO_ROOT / roadmap.render.archive_register
    deferred_register = REPO_ROOT / roadmap.render.deferred_register
    return {
        decision_register: render_decision_register(roadmap),
        dependency_roadmap: render_dependency_roadmap(roadmap),
        current_candidates_roadmap: render_current_candidates_roadmap(roadmap),
        triage: render_triage_document(triage, roadmap),
        archive_register: render_item_register(
            roadmap,
            title="Roadmap Archive Register",
            description="Completed WR roadmap rows archived out of the active execution source.",
            items=roadmap.archived_items,
        ),
        deferred_register: render_item_register(
            roadmap,
            title="Roadmap Deferred Register",
            description="Blocked or policy-deferred WR roadmap rows archived out of active execution.",
            items=roadmap.deferred_items,
        ),
    }


def render_decision_register(roadmap: RoadmapState) -> str:
    lines = [
        "---",
        "title: Roadmap Decision Register",
        "description: Workspace scorecard and decision register for roadmap sequencing.",
        "status: active",
        "owner: workspace",
        "layer: workspace",
        "canonical: true",
        f"last_reviewed: {roadmap.roadmap.last_reviewed}",
        "related:",
        "  - ./planning-methods.md",
        "  - ./design-implementation-triage.md",
        "  - ./repo-execution-priority-checklist.md",
        "  - ./roadmap-index.md",
        "  - ./roadmap-items.yaml",
        "  - ./roadmap-archive.yaml",
        "  - ./roadmap-deferred.yaml",
        "  - ./roadmap-archive-register.md",
        "  - ./roadmap-deferred-register.md",
        "  - ./schemas/roadmap-items.schema.json",
        "  - ./schemas/roadmap-item-source.schema.json",
        "  - ./diagrams/value-weighted-dependency-roadmap.puml",
        "  - ./diagrams/current-roadmap-candidates.puml",
        "---",
        "",
        "# Roadmap Decision Register",
        "",
        "## Purpose",
        "",
        "This register records the current workspace-level roadmap scoring. It supports",
        "implementation triage, but it does not replace owning domain or app roadmaps.",
        "",
        "Scores are first-pass relative estimates. Update them when code evidence,",
        "closeout reports, product evidence, or owning roadmaps change.",
        "",
        "The scorecard table below is generated from",
        "[roadmap-items.yaml](./roadmap-items.yaml), the active execution source.",
        "Completed and deferred rows live in",
        "[roadmap-archive.yaml](./roadmap-archive.yaml) and",
        "[roadmap-deferred.yaml](./roadmap-deferred.yaml). Do not edit generated",
        "tables directly; update the YAML sources and run `task roadmap:render`.",
        "",
        "## Score Model",
        "",
        "The score model is defined in [planning-methods.md](./planning-methods.md).",
        "",
        "```text",
        "A-WSJF = ((V + TC + RR/OE + DU) * C) / E",
        "```",
        "",
        "Priority resolution order:",
        "",
        "1. Gate and blocker state.",
        "2. Dependency level.",
        "3. Lane.",
        "4. A-WSJF score.",
        "5. RICE only for product-facing items with credible reach.",
        "",
        "## Scorecard",
        "",
        "| ID | Track | Lane | Planning state | Completion quality | Dependency level | Gate | V | B | TC | RR/OE | DU | E | C | A-WSJF | RICE | Kano | Next evidence | Current decision |",
        "|---|---|---|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|",
    ]
    for item in active_register_items(roadmap):
        lines.append(
            "| "
            + " | ".join(
                [
                    item.id,
                    item.title,
                    item.lane,
                    item.planning_state,
                    item.completion_quality,
                    item.dependency_level,
                    item.gate,
                    str(item.value),
                    str(item.blocker),
                    str(item.tc),
                    str(item.rr_oe),
                    str(item.du),
                    str(item.effort),
                    f"{item.confidence:.1f}",
                    f"{item.score:.1f}",
                    item.rice,
                    item.kano,
                    item.next_evidence,
                    item.current_decision,
                ]
            )
            + " |"
        )
    lines.extend(
        [
            "",
            "Active views omit completed and deferred rows. Use",
            "[roadmap-archive-register.md](./roadmap-archive-register.md) for",
            "completed evidence and",
            "[roadmap-deferred-register.md](./roadmap-deferred-register.md) for",
            "blocked or deferred backlog.",
            "",
            "",
            "## Review Rules",
            "",
            "- Re-score after a closeout report changes the evidence for a track.",
            "- Re-score when a blocker moves between `B1` through `B5`.",
            "- Keep RICE blank as `N/A` until there is a credible reach estimate.",
            "- Never promote `B5` work by score alone; require an accepted design, ADR, or",
            "  owning roadmap update.",
            "",
        ]
    )
    return "\n".join(lines)


def active_register_items(roadmap: RoadmapState) -> list[RoadmapItem]:
    return list(roadmap.active_items)


def render_dependency_roadmap(roadmap: RoadmapState) -> str:
    view_items = list(roadmap.active_items)
    view_ids = {item.id for item in view_items}
    levels = [
        ("L0", "Level 0 - Support Substrate"),
        ("L1", "Level 1 - Near-Term Productization"),
        ("L2", "Level 2 - Contract-Gated Productization"),
        ("L3", "Level 3 - Active Domain Tracks"),
        ("L4", "Level 4 - Active Discovery Context"),
    ]
    lines = [
        "@startuml",
        f"title {roadmap.roadmap.title}",
        "",
        "top to bottom direction",
        "skinparam componentStyle rectangle",
        "skinparam shadowing false",
        "skinparam packageStyle rectangle",
        "skinparam defaultFontName Monospaced",
        "",
        "skinparam component {",
        "  BackgroundColor<<V5>> #FFDCDC",
        "  BorderColor<<V5>> #AA0000",
        "  BackgroundColor<<V4>> #FFF2CC",
        "  BorderColor<<V4>> #AA7700",
        "  BackgroundColor<<V3>> #DDEEFF",
        "  BorderColor<<V3>> #004488",
        "  BackgroundColor<<V2>> #EEEEEE",
        "  BorderColor<<V2>> #666666",
        "  BackgroundColor<<V1>> #F5F5F5",
        "  BorderColor<<V1>> #999999",
        "  BackgroundColor<<support_only>> #F4F4F4",
        "  BorderColor<<support_only>> #777777",
        "  FontColor<<support_only>> #666666",
        "}",
        "",
        "legend right",
        "  <b>Diagram Type</b>",
        "  Value-weighted layered PDM / Activity-on-Node roadmap",
        "",
        "  <b>Layout</b>",
        "  Same level = dependency tier, not necessarily selectable work",
        "  Downward edge = dependency or sequencing gate",
        "  Scores rank comparable work only.",
        "  Gate and dependency level win before score.",
        "  Archive and deferred rows live in separate registers.",
        "  Support-only nodes remain active context, not batch candidates.",
        "",
        "  <b>Value Weight</b>",
        "  <#FFDCDC> V5 = unlocks current focus or many downstream tracks",
        "  <#FFF2CC> V4 = high product / architecture value",
        "  <#DDEEFF> V3 = useful medium-horizon work or cleanup",
        "  <#EEEEEE> V2 = valid but not currently central",
        "  <#F5F5F5> V1 = exploratory or policy-blocked",
        "",
        "  <b>Blocker Weight</b>",
        "  B1 = scoped implementation",
        "  B2 = bounded proof or partial substrate blocker",
        "  B3 = product/runtime blocker",
        "  B4 = missing domain contract blocker",
        "  B5 = policy-deferred blocker",
        "endlegend",
        "",
    ]
    for level, title in levels:
        level_items = [item for item in view_items if item.dependency_level == level]
        if not level_items:
            continue
        lines.append(f'package "{title}" {{')
        for item in level_items:
            lines.append(render_component(item))
            lines.append("")
        lines.append("}")
        lines.append("")

    by_id = roadmap.by_id
    for edge in roadmap.edges:
        if edge.source not in view_ids or edge.target not in view_ids:
            continue
        source = by_id[edge.source].alias
        target = by_id[edge.target].alias
        lines.append(f"{source} -down-> {target} : {edge.label}")
    lines.extend(["", "@enduml", ""])
    return "\n".join(lines)


def render_component(item: RoadmapItem) -> str:
    label_lines = [
        f"{item.id} {item.diagram_title}",
        f"state={item.planning_state}",
        f"score={item.score:.1f} gate={item.gate.lower()}",
        f"{item.value_label} {item.blocker_label}",
        "call=" + "\\n".join(item.diagram_call),
    ]
    label = "\\n".join(label_lines).replace('"', '\\"')
    return f'  component "{label}" as {item.alias} <<{component_stereotype(item)}>>'


def component_stereotype(item: RoadmapItem) -> str:
    if item.planning_state in {"completed", "support_only"}:
        return item.planning_state
    return item.value_label


def render_current_candidates_roadmap(roadmap: RoadmapState) -> str:
    candidates = select_batch_candidates(roadmap)
    candidate_ids = {item.id for item in candidates}
    dependency_ids = {dependency for item in candidates for dependency in item.dependencies}
    dependencies = [item for item in roadmap.active_items if item.id in dependency_ids and item.id not in candidate_ids]
    active_dependency_ids = {item.id for item in dependencies}
    by_id = roadmap.by_id
    lines = [
        "@startuml",
        "title Runenwerk Current Roadmap Candidates",
        "",
        "top to bottom direction",
        "skinparam componentStyle rectangle",
        "skinparam shadowing false",
        "skinparam packageStyle rectangle",
        "skinparam defaultFontName Monospaced",
        "",
        "skinparam component {",
        "  BackgroundColor<<candidate>> #FFF2CC",
        "  BorderColor<<candidate>> #AA7700",
        "  BackgroundColor<<context>> #EEEEEE",
        "  BorderColor<<context>> #666666",
        "  FontColor<<context>> #666666",
        "}",
        "",
        "legend right",
        "  <b>Selectable Work</b>",
        "  Only planning_state=current_candidate nodes may enter implementation batches.",
        "  Context nodes are direct dependencies and are not selectable here.",
        "endlegend",
        "",
    ]
    if dependencies:
        lines.append('package "Immediate Dependency Context" {')
        for item in dependencies:
            lines.append(render_candidate_context_component(item))
            lines.append("")
        lines.append("}")
        lines.append("")
    lines.append('package "Current Implementation Candidates" {')
    for item in candidates:
        lines.append(render_candidate_component(item))
        lines.append("")
    if not candidates:
        lines.append('  component "No current_candidate items" as NO_CANDIDATES <<context>>')
        lines.append("")
    lines.append("}")
    lines.append("")
    for item in candidates:
        for dependency in item.dependencies:
            if dependency in active_dependency_ids:
                lines.append(f"{by_id[dependency].alias} -down-> {item.alias} : dependency context")
    lines.extend(["", "@enduml", ""])
    return "\n".join(lines)


def render_candidate_component(item: RoadmapItem) -> str:
    label_lines = [
        f"{item.id} {item.diagram_title}",
        f"state={item.planning_state}",
        f"score={item.score:.1f} {item.value_label} {item.blocker_label}",
        "call=" + "\\n".join(item.diagram_call),
    ]
    label = "\\n".join(label_lines).replace('"', '\\"')
    return f'  component "{label}" as {item.alias} <<candidate>>'


def render_candidate_context_component(item: RoadmapItem) -> str:
    label_lines = [
        f"{item.id} {item.diagram_title}",
        f"state={item.planning_state}",
        f"{item.value_label} {item.blocker_label}",
    ]
    label = "\\n".join(label_lines).replace('"', '\\"')
    return f'  component "{label}" as {item.alias} <<context>>'


def render_triage_document(path: Path, roadmap: RoadmapState) -> str:
    current = path.read_text(encoding="utf-8")
    block = f"{TRIAGE_BEGIN}\n{render_triage_status(roadmap)}{TRIAGE_END}"
    if TRIAGE_BEGIN in current and TRIAGE_END in current:
        before, rest = current.split(TRIAGE_BEGIN, 1)
        _, after = rest.split(TRIAGE_END, 1)
        return before + block + after
    marker = "\n## Design Lifecycle Cleanup Findings\n"
    if marker not in current:
        raise WorkflowError(f"{repo_path(path)} is missing generated roadmap marker or Design Lifecycle Cleanup Findings section")
    before, after = current.split(marker, 1)
    before = re.sub(r"\n## Implement Now\n.*\Z", "\n", before, flags=re.S)
    return before.rstrip() + "\n\n" + block + marker + after


def render_triage_status(roadmap: RoadmapState) -> str:
    active_items = roadmap.active_items
    groups = {
        "current_candidate": [item for item in active_items if item.planning_state == "current_candidate"],
        "support_only": [item for item in active_items if item.planning_state == "support_only"],
        "ready_next": [item for item in active_items if item.planning_state == "ready_next"],
    }
    lines = [
        "## Current Candidate",
        "",
        "| ID | Track | Priority | Value | Blocker | Score | Current call | First implementation move |",
        "|---|---|---:|---:|---:|---:|---|---|",
    ]
    for item in groups["current_candidate"]:
        lines.append(f"| {item.id} | {item.title} | {item.priority} | {item.value_label} | {item.blocker_label} | {item.score:.1f} | {item.current_call} | {item.first_move} |")
    lines.extend(
        [
            "",
            "## Support Only",
            "",
            "| ID | Track | Priority | Value | Blocker | Score | Current call | Reactivation evidence |",
            "|---|---|---:|---:|---:|---:|---|---|",
        ]
    )
    for item in groups["support_only"]:
        lines.append(f"| {item.id} | {item.title} | {item.priority} | {item.value_label} | {item.blocker_label} | {item.score:.1f} | {item.current_call} | {item.next_evidence} |")
    lines.extend(
        [
            "",
            "## Ready Next",
            "",
            "| ID | Track | Priority | Value | Blocker | Score | Current call | Main blocker |",
            "|---|---|---:|---:|---:|---:|---|---|",
        ]
    )
    for item in groups["ready_next"]:
        lines.append(f"| {item.id} | {item.title} | {item.priority} | {item.value_label} | {item.blocker_label} | {item.score:.1f} | {item.current_call} | {item.main_blocker} |")
    lines.extend(
        [
            "",
            "## Archived And Deferred Registers",
            "",
            f"- Completed evidence: [{repo_path(REPO_ROOT / roadmap.render.archive_register)}]({relative_workspace_link(roadmap.render.archive_register)})",
            f"- Deferred backlog: [{repo_path(REPO_ROOT / roadmap.render.deferred_register)}]({relative_workspace_link(roadmap.render.deferred_register)})",
        ]
    )
    lines.append("")
    return "\n".join(lines)


def relative_workspace_link(path: str) -> str:
    prefix = "docs-site/src/content/docs/workspace/"
    return f"./{path.removeprefix(prefix)}"


def render_item_register(
    roadmap: RoadmapState,
    *,
    title: str,
    description: str,
    items: list[RoadmapItem],
) -> str:
    lines = [
        "---",
        f"title: {title}",
        f"description: {description}",
        "status: active",
        "owner: workspace",
        "layer: workspace",
        "canonical: true",
        f"last_reviewed: {roadmap.roadmap.last_reviewed}",
        "related:",
        "  - ./roadmap-items.yaml",
        "  - ./roadmap-archive.yaml",
        "  - ./roadmap-deferred.yaml",
        "  - ./roadmap-decision-register.md",
        "---",
        "",
        f"# {title}",
        "",
        description,
        "",
        "| ID | Track | Lane | Planning state | Completion quality | Dependency level | Gate | V | B | Score | Current decision | Evidence / blocker |",
        "|---|---|---|---|---|---:|---|---:|---:|---:|---|---|",
    ]
    for item in items:
        evidence_or_blocker = item.next_evidence if item.planning_state == "completed" else item.why_not_ready
        lines.append(
            "| "
            + " | ".join(
                [
                    item.id,
                    item.title,
                    item.lane,
                    item.planning_state,
                    item.completion_quality,
                    item.dependency_level,
                    item.gate,
                    str(item.value),
                    str(item.blocker),
                    f"{item.score:.1f}",
                    item.current_decision,
                    evidence_or_blocker,
                ]
            )
            + " |"
        )
    lines.append("")
    return "\n".join(lines)


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
def render(source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source.")) -> None:
    roadmap = load_roadmap(source)
    write_outputs(render_outputs(roadmap))
    console.print("[green]roadmap docs rendered[/green]")


@app.command()
def check(source: Path = typer.Option(ROADMAP_SOURCE, help="Roadmap YAML source.")) -> None:
    roadmap = load_roadmap(source)
    stale = stale_outputs(render_outputs(roadmap))
    if stale:
        console.print("[red]roadmap render check failed[/red]")
        for path in stale:
            console.print(f"- stale generated file: {path}")
        raise typer.Exit(1)
    console.print("[green]roadmap render check passed[/green]")


if __name__ == "__main__":
    app()
