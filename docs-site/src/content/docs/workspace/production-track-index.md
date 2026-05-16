---
title: Production Track Index
description: Generated index of long-term production tracks and their milestone states.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-16
related:
  - ./production-track-planning-model.md
  - ./production-tracks.yaml
  - ./production-milestone-register.md
  - ./roadmap-items.yaml
  - ./roadmap-decision-register.md
  - ./schemas/production-tracks.schema.json
  - ./diagrams/production-track-roadmap.puml
---

# Production Track Index

This page is generated from [production-tracks.yaml](./production-tracks.yaml).
Do not edit it directly; update the YAML source and run `task production:render`.

Production tracks guide long-term sequencing. The WR roadmap remains the
dependency-checked execution graph.

## Tracks

| ID | Track | State | Owner | Strategic goal | Success criteria |
|---|---|---|---|---|---|
| PT-SDF-OW | SDF-first open-world playable vertical | active | workspace | Prove the SDF-first field-world architecture through a playable, visible, inspectable open world. | Player movement, world rendering, strict query products, diagnostics, and content products are integrated through production contracts.<br>Deferred world capabilities move through design gates before implementation, not through one-off prototype shortcuts.<br>The track remains extensible for caves, multiplayer, richer simulation, advanced VFX, and gameplay systems. |

## Current Milestone States

### PT-SDF-OW - SDF-first open-world playable vertical

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-SDF-OW-001 | Production product spine | implementation | active | WR-019, WR-026, WR-021 | Field visualization, source-backed asset adapters, material preview products, and renderer handoff are planned and executed through WR roadmap rows. |
| PM-SDF-OW-002 | Open world substrate design | design | active | WR-001, WR-014, WR-015 | Owning designs identify product formats, mutation paths, residency, strict query behavior, and diagnostics before implementation. |
| PM-SDF-OW-003 | Playable SDF character | design | designing | WR-014, WR-015, WR-022 | Character body, pose, motion, collision, render, interaction emitters, and diagnostics have accepted ownership and product boundaries. |
| PM-SDF-OW-004 | Atmosphere and material response | design | designing | WR-014, WR-015 | Day/night becomes a world/render product family, not a renderer color shortcut. |
| PM-SDF-OW-005 | Vegetation field interaction | design | designing | WR-014, WR-015 | Vegetation is planned as deterministic field products rather than per-blade authored state. |
| PM-SDF-OW-006 | SDF prefab production set | design | designing | WR-022 | Prefabs become reusable field compositions with product outputs rather than mesh-centric object bundles. |
| PM-SDF-OW-007 | Water and wetness fields | design | designing | WR-014, WR-015 | Water is a field product family with explicit interaction and render handoff. |
| PM-SDF-OW-008 | Enemy and influence AI proof | design | designing | WR-011, WR-014, WR-015, WR-022 | Enemy behavior uses explicit gameplay/influence contracts without requiring the full gameplay graph first. |
| PM-SDF-OW-009 | Production hardening and playable evidence | hardening | designing | WR-018, WR-019, WR-021, WR-022 | The first production vertical is complete only when the combined acceptance criteria are observed and documented. |
