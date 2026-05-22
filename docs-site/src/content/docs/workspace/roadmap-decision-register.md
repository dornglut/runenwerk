---
title: Roadmap Decision Register
description: Workspace scorecard and decision register for roadmap sequencing.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-22
related:
  - ./planning-methods.md
  - ./design-implementation-triage.md
  - ./repo-execution-priority-checklist.md
  - ./roadmap-index.md
  - ./roadmap-items.yaml
  - ./roadmap-archive.yaml
  - ./roadmap-deferred.yaml
  - ./roadmap-archive-register.md
  - ./roadmap-deferred-register.md
  - ./schemas/roadmap-items.schema.json
  - ./schemas/roadmap-item-source.schema.json
  - ./diagrams/value-weighted-dependency-roadmap.puml
  - ./diagrams/current-roadmap-candidates.puml
---

# Roadmap Decision Register

## Purpose

This register records the current workspace-level roadmap scoring. It supports
implementation triage, but it does not replace owning domain or app roadmaps.

Scores are first-pass relative estimates. Update them when code evidence,
closeout reports, product evidence, or owning roadmaps change.

The scorecard table below is generated from
[roadmap-items.yaml](./roadmap-items.yaml), the active execution source.
Completed and deferred rows live in
[roadmap-archive.yaml](./roadmap-archive.yaml) and
[roadmap-deferred.yaml](./roadmap-deferred.yaml). Do not edit generated
tables directly; update the YAML sources and run `task roadmap:render`.

## Score Model

The score model is defined in [planning-methods.md](./planning-methods.md).

```text
A-WSJF = ((V + TC + RR/OE + DU) * C) / E
```

Priority resolution order:

1. Gate and blocker state.
2. Dependency level.
3. Lane.
4. A-WSJF score.
5. RICE only for product-facing items with credible reach.

## Scorecard

| ID | Track | Lane | Planning state | Completion quality | Dependency level | Gate | V | B | TC | RR/OE | DU | E | C | A-WSJF | RICE | Kano | Next evidence | Current decision |
|---|---|---|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|
| WR-002 | ECS/runtime convergence support for product jobs and diagnostics | Core support | support_only | not_applicable | L0 | Supporting now | 4 | 2 | 3 | 5 | 5 | 5 | 0.8 | 2.7 | N/A | Neutral | 2026-05-14 L0 substrate pilot added ECS-owned runtime plan reporting; M5 diagnostics are landed and F2 lifecycle/finalization remains. | Continue only where lifecycle/finalization unblocks product jobs or diagnostics. |
| WR-003 | Render contract follow-ups through product selection and derived residency | Core support | support_only | not_applicable | L0 | Supporting now | 4 | 2 | 3 | 4 | 4 | 5 | 0.8 | 2.4 | N/A | Neutral | 2026-05-14 L0 substrate pilot added view-ordered product selection snapshots, selected-source residency invalidation, and conflict rejection for selected source state. | Continue as contract-following support, not renderer-owned world truth. |
| WR-004 | UI/editor guard and sequencing maintenance | Guardrail | support_only | not_applicable | L0 | Always-on guard | 4 | 1 | 4 | 4 | 3 | 2 | 1.0 | 7.5 | N/A | Basic | 2026-05-14 entity-table and SDF surface routing/capability guards landed. | Keep active in parallel; score does not make it the main roadmap. |
| WR-005 | Design lifecycle cleanup for implemented active designs | Docs | support_only | not_applicable | L0 | Docs now | 3 | 1 | 2 | 3 | 2 | 2 | 0.8 | 4.0 | N/A | Neutral | 2026-05-16 docs governance milestone moved implemented active designs into implemented lifecycle, aligned canonical ownership maps with workspace truth, indexed preserved batch artifacts, and hardened docs validation against lifecycle drift, missing crate coverage, stale prompt paths, and stale active-looking batches. | Keep as an always-available docs governance support lane; the current no-shortcuts cleanup milestone is complete and future drift should fail through task docs:validate. |
| WR-008 | Native tablet input backend arbitration and diagnostics | Product discovery | ready_next | not_applicable | L1 | Product evidence gate | 3 | 3 | 2 | 3 | 2 | 5 | 0.5 | 1.0 | Candidate after hardware target exists | Basic | Hardware validation across Windows Ink, Wacom Wintab, and macOS Wacom. | WR-006 dependency is complete; code can continue, but product acceptance is hardware-blocked. |
| WR-022 | SDF Prefab V2 | Product workflow | ready_next | not_applicable | L3 | V2 gated | 4 | 3 | 2 | 4 | 3 | 8 | 0.5 | 0.8 | Runtime prefab instancing after rendered-world and asset identity stabilize | Exciter | Source/catalog prefab identity, descriptor diagnostics, and approved runtime instancing sequence. | Keep prefab composition active as design and catalog identity now; runtime instancing waits for V2 gates. |
| WR-023 | ECS Parallel Execution | Core support | ready_next | not_applicable | L3 | Design first | 3 | 2 | 3 | 5 | 3 | 13 | 0.5 | 0.5 | Parallel execution only after deterministic system contract acceptance | Performance | Accepted design for Send + Sync system constraints, access sharding, per-wave command queues, deterministic merges, and blocked-parallelism diagnostics. | Keep ECS execution serial during rendered-world V1; harden product jobs as the active multithreaded path. |
| WR-024 | Editor Shell Polish | Guardrail | ready_next | not_applicable | L1 | After Interaction V2 contract spine | 4 | 1 | 4 | 3 | 4 | 3 | 0.8 | 4.0 | Immediate viewport usability and menu clarity | Basic | Retained-UI slice evidence after the relevant named Interaction V2 contracts exist for IV2-menu-stack, IV2-scroll-ownership, IV2-menu-sizing, IV2-chrome-slots, IV2-dock-drop-zones, and IV2-status-and-viewport-arbitration. | Treat editor shell polish as a retained-UI implementation slice under the broader Interaction V2 contract layer, not as a pile of isolated cosmetic fixes. |
| WR-029 | Model Mesh Material Binding | Product workflow | ready_next | not_applicable | L3 | After WR-030 model/mesh pixel proof | 4 | 3 | 4 | 5 | 4 | 8 | 0.5 | 1.1 | Close model/mesh material binding without weakening the proven SDF material path. | Basic | Phase 1-3 code evidence is accepted for model/mesh identity, persistence, workflow projection, prepared transport, shader lane, and pass provenance. Next evidence must come from WR-030 visible Mesh Preview model/mesh pixel proof plus refreshed WR-028 SDF non-regression. | WR-029 is ready-next, not current, because Phase 4 depends on WR-030. Keep the accepted implementation contract active and preserve the landed Phase 1-3 code evidence, but do not close model/mesh material binding without source-backed visible pixel proof. |
| WR-030 | Model Mesh Renderable Scene Contract | Product workflow | ready_next | not_applicable | L3 | Current Mesh Preview pixel proof | 4 | 2 | 4 | 4 | 4 | 8 | 0.5 | 1.0 | Provide the source-backed model/mesh preview surface required before WR-029 can honestly claim model/mesh material pixels. | Basic | Deferring product-workflow model-mesh pixel proof so PT-WB-CAP can run WR-032; WR-030 remains valid ready-next work for WR-029 and is not a typed Workbench handle prerequisite. | Deferring product-workflow model-mesh pixel proof so PT-WB-CAP can run WR-032; WR-030 remains valid ready-next work for WR-029 and is not a typed Workbench handle prerequisite. |
| WR-040 | External Component Sandbox Design | Platform security | support_only | not_applicable | L3 | Policy deferred until sandbox/security design acceptance | 3 | 5 | 5 | 5 | 5 | 8 | 0.3 | 0.7 | Design-only row for future external components. | Basic | Accepted sandbox/security design before any external dynamic component implementation. | External components stay blocked until sandbox, package trust, permissions, unload/reload, and diagnostics are accepted. |
| WR-046 | UI Designer doctrine and target boundary ratification | Product planning | support_only | not_applicable | L0 | Support-only doctrine evidence | 3 | 1 | 2 | 3 | 2 | 2 | 0.8 | 4.0 | N/A | Basic | PM-UI-DESIGN-001 closeout proving active UI Designer doctrine, target-profile boundaries, no runtime/code changes, and passing production, roadmap, docs, and planning validators. | Support-only planning row for PM-UI-DESIGN-001; link doctrine closeout to the production track without changing WR execution state. |

Active views omit completed and deferred rows. Use
[roadmap-archive-register.md](./roadmap-archive-register.md) for
completed evidence and
[roadmap-deferred-register.md](./roadmap-deferred-register.md) for
blocked or deferred backlog.


## Review Rules

- Re-score after a closeout report changes the evidence for a track.
- Re-score when a blocker moves between `B1` through `B5`.
- Keep RICE blank as `N/A` until there is a credible reach estimate.
- Never promote `B5` work by score alone; require an accepted design, ADR, or
  owning roadmap update.
