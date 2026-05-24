---
title: Design Implementation Triage
description: Current workspace triage of active and deferred design work by implementation readiness, blocker weight, and value-weighted dependency level.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-15
related:
  - ./planning-methods.md
  - ./roadmap-items.yaml
  - ./roadmap-decision-register.md
  - ./repo-execution-priority-checklist.md
  - ./roadmap-index.md
  - ./sdf-first-execution-roadmap.md
  - ../apps/runenwerk-editor/roadmap.md
  - ../apps/runenwerk-draw/roadmap.md
  - ../net/multiplayer-replication-implementation-roadmap.md
  - ../engine/plugins/render/docs/roadmap.md
  - ../domain/ui/roadmap.md
  - ../design/active/README.md
  - ../design/deferred/README.md
  - ../design/accepted/diagrams/runenwerk-design-priority-map.puml
---

# Design Implementation Triage

## Purpose

This page answers a narrow question: which remaining designs can turn into
implementation now, which ones are only next after the current focus stabilizes,
and which ones are still blocked by missing contracts.

It is a workspace triage page, not a replacement for owning domain roadmaps.
When this page disagrees with an owning roadmap, the owning roadmap wins.

## Diagram

The current implementation planning diagram is a value-weighted layered PDM /
Activity-on-Node dependency roadmap:

- [value-weighted-dependency-roadmap.puml](diagrams/value-weighted-dependency-roadmap.puml)

Use it as the canonical workspace map for implementation planning:

- nodes on the same level are parallelizable;
- downward arrows are dependency or sequencing gates;
- node color is value weight;
- node text carries roadmap ID, score, gate, value, blocker, and the
  implementation call.

Full scorecard details live in
[roadmap-decision-register.md](./roadmap-decision-register.md).

There is also an older accepted SDF capability dependency diagram:

- [runenwerk-design-priority-map.puml](../design/accepted/diagrams/runenwerk-design-priority-map.puml)

That existing diagram is still useful as a long-term capability dependency map,
but it does not encode current implementation readiness, current blockers, or
parallelizable implementation levels.

## Weighting Model

Value weight:

| Weight | Meaning |
|---|---|
| V5 | Unlocks the current cross-repo focus or many downstream tracks. |
| V4 | High product or architecture value, ready soon, or strong enabler. |
| V3 | Useful medium-horizon work or important cleanup. |
| V2 | Valid but not currently central. |
| V1 | Deferred, exploratory, or policy-blocked. |

Priority weight:

| Weight | Meaning |
|---|---|
| P0 | Current focus. Start or continue now if selecting code work. |
| P1 | Ready next after the P0 path is stable. |
| P2 | Design-gated or medium-horizon work. |
| P3 | Deferred or long-horizon capability work. |

Priority weight answers when the work should be considered. Value weight answers
how much the work unlocks once its blockers are satisfied.

For roadmap ordering, use the scorecard policy in
[planning-methods.md](./planning-methods.md):

1. Gate and blocker state first.
2. Dependency level second.
3. Lane and A-WSJF score third.
4. RICE only for product-facing work with credible reach evidence.

Scores do not override owning roadmaps, dependency gates, or `B5` policy
deferral.

Blocker weight:

| Weight | Meaning |
|---|---|
| B1 | No material architecture blocker. Needs scoped implementation and validation. |
| B2 | Partial blocker. Some substrate exists, but one bounded contract or proof is still needed. |
| B3 | Product or runtime blocker. Needs another roadmap slice first. |
| B4 | Domain contract blocker. Missing owning domain contracts or formed-product path. |
| B5 | Deferred by policy. Do not implement until promoted by an active design, accepted design, ADR, or roadmap update. |

<!-- BEGIN GENERATED ROADMAP STATUS -->
## Current Candidate

| ID | Track | Priority | Value | Blocker | Score | Current call | First implementation move |
|---|---|---:|---:|---:|---:|---|---|
| WR-089 | Renderer Procedural Population Hardening Doctrine And Track Activation | P0 | V4 | B2 | 5.6 | Use WR-089 only to activate PT-RENDER-PROCEDURAL-POPULATION-HARDENING and split implementation into WR-090, WR-091, WR-092, WR-101, with WR-093 reserved for closeout. | Run task production:plan -- --milestone "PM-RENDER-POP-HARDEN-001" --roadmap "WR-089", then close WR-089 only after planning metadata and contracts validate. |
| WR-100 | UI Lab perfectionist governance and no-gap audit doctrine | P2 | V3 | B3 | 1.0 | Execute the PM-UI-LAB-PERF-001 governance/design contract, then stop before app or domain implementation until follow-on WRs are accepted. | Run task production:plan -- --milestone PM-UI-LAB-PERF-001 --roadmap WR-100, then finish the code-truth and evidence matrix closeout. |

## Support Only

| ID | Track | Priority | Value | Blocker | Score | Current call | Reactivation evidence |
|---|---|---:|---:|---:|---:|---|---|
| WR-002 | ECS/runtime convergence support for product jobs and diagnostics | P0 | V4 | B2 | 2.7 | Implement as supporting work only when it directly unblocks the SDF-first execution path. The 2026-05-14 pilot landed M5 consumer lag/backpressure diagnostics and F3 plan reporting; F2 remains open. | 2026-05-14 L0 substrate pilot added ECS-owned runtime plan reporting; M5 diagnostics are landed and F2 lifecycle/finalization remains. |
| WR-003 | Render contract follow-ups through product selection and derived residency | P0 | V4 | B2 | 2.4 | Implement only as contract-following FR-1 follow-up from the fully featured renderer roadmap, not as renderer-owned world truth. The 2026-05-14 pilot added prepared-view ownership guards plus selected-source residency derivation and invalidation. | 2026-05-14 L0 substrate pilot added view-ordered product selection snapshots, selected-source residency invalidation, and conflict rejection for selected source state. |
| WR-004 | UI/editor guard and sequencing maintenance | P0 | V4 | B1 | 7.5 | Keep active while any editor surface work lands. The 2026-05-14 parallel batch added entity-table and SDF operation routing/capability guards. | 2026-05-14 entity-table and SDF surface routing/capability guards landed. |
| WR-005 | Design lifecycle cleanup for implemented active designs | P1 | V3 | B1 | 4.0 | Use only for future lifecycle drift or source-of-truth cleanup. Current validation now rejects active designs that claim implementation without an explicit phase-evidence marker and rejects missing crate-doc coverage. | 2026-05-16 docs governance milestone moved implemented active designs into implemented lifecycle, aligned canonical ownership maps with workspace truth, indexed preserved batch artifacts, and hardened docs validation against lifecycle drift, missing crate coverage, stale prompt paths, and stale active-looking batches. |
| WR-040 | External Component Sandbox Design | P2 | V3 | B5 | 0.7 | Keep this as design-only future work. | Accepted sandbox/security design before any external dynamic component implementation. |
| WR-046 | UI Designer doctrine and target boundary ratification | P1 | V3 | B1 | 4.0 | Use only for UI Designer doctrine and target-boundary planning evidence. Do not use this row as permission for UI Designer product implementation. | PM-UI-DESIGN-001 closeout proving active UI Designer doctrine, target-profile boundaries, no runtime/code changes, and passing production, roadmap, docs, and planning validators. |

## Ready Next

| ID | Track | Priority | Value | Blocker | Score | Current call | Main blocker |
|---|---|---:|---:|---:|---:|---|---|
| WR-008 | Native tablet input backend arbitration and diagnostics | P1 | V3 | B3 | 1.0 | Code work can continue after the WR-006 dependency, but product acceptance remains blocked. | Hardware validation is still required for Windows Ink, Wacom Wintab, and macOS Wacom devices. |
| WR-022 | SDF Prefab V2 | P2 | V4 | B3 | 0.8 | Do not implement runtime prefab instances until rendered-world V1 and source-backed asset identity are stable; prefab renderer handoff must consume the fully featured renderer roadmap instead of adding a parallel scene path. | Runtime prefab instancing waits for rendered-world V1, source-backed prefab identity adapters, and accepted product ownership. |
| WR-023 | ECS Parallel Execution | P2 | V3 | B2 | 0.5 | Design now, implement later after diagnostics and deterministic merge policy are accepted. | Public parallel execution waits for accepted deterministic merge policy, blocked-parallelism diagnostics, and serial equivalence tests. |
| WR-024 | Editor Shell Polish | P0 | V4 | B1 | 4.0 | Ready-next only after the WR-025 doctrine repair is committed; consume landed popup, scroll, chrome, docking, and status overflow slices instead of defining local policy. | Interaction V2 contract/migration spine must lead; polish can only proceed as a retained-UI slice consuming those contracts or as explicitly bounded compatibility evidence. |
| WR-029 | Model Mesh Material Binding | P1 | V4 | B3 | 1.1 | Ready-next only. Do not claim WR-029 complete until WR-030 produces visible source-backed model/mesh pixels through a material-consuming pass, WR-028 SDF non-regression proof is refreshed, and closeout evidence names the consuming renderer module. | WR-030 must prove visible model/mesh pixels from PreparedModelMeshMaterialSelection and the scene material table before WR-029 can close. |
| WR-030 | Model Mesh Renderable Scene Contract | P0 | V4 | B2 | 1.0 | Implement the Mesh Preview material-consuming pass proof: source-backed model/mesh region selection must resolve through PreparedModelMeshMaterialSelection and the scene material table into visible pixels, with pass provenance and WR-028 SDF non-regression. | Needs visible Mesh Preview model/mesh pixels from the selected scene material table entry; descriptor/status rows and SDF pixels are not sufficient proof. |
| WR-090 | Indirect Draw Contract Hardening | P0 | V4 | B2 | 2.7 | Harden direct, indexed direct, indirect, and indexed indirect draw-source semantics without breaking direct `.draw(...)` authoring. | Waiting on WR-089 doctrine activation closeout. |
| WR-091 | Reusable GPU Primitive Shader Dispatch | P0 | V4 | B2 | 1.8 | Convert primitive contracts from descriptor/planning-level evidence into reusable renderer-dispatched kernels. | Waiting on WR-090 indirect draw hardening closeout. |
| WR-092 | Fixed Step Graph Catch Up Scheduling | P0 | V4 | B2 | 2.6 | Add fixed-step catch-up as render-flow graph scheduling using FixedTimeConfig, FixedTimeState, and CatchupBudget; do not add boids-local timing logic. | Waiting on WR-091 primitive shader dispatch closeout. |
| WR-101 | Procedural Camera And View Projection | P0 | V4 | B2 | 2.6 | Add reusable procedural camera projection and sprite sizing contracts; do not patch boids-only draw parameters or make PreparedViewFrame own camera truth. | Waiting on WR-092 graph catch-up scheduling closeout. |
| WR-093 | Procedural Population Hardening Evidence Benchmarks Docs And Closeout | P0 | V4 | B2 | 3.7 | Close the hardening track at runtime_proven only after earlier slice closeouts prove runtime behavior and camera projection evidence. | Waiting on WR-101 procedural camera projection closeout. |

## Archived And Deferred Registers

- Completed evidence: [docs-site/src/content/docs/workspace/roadmap-archive-register.md](./roadmap-archive-register.md)
- Deferred backlog: [docs-site/src/content/docs/workspace/roadmap-deferred-register.md](./roadmap-deferred-register.md)
<!-- END GENERATED ROADMAP STATUS -->
## Design Lifecycle Cleanup Findings

Several active designs now describe implemented foundations. They are not code
blockers, but they should be reviewed for promotion to `accepted/` or
`implemented/` after code truth and tests are rechecked:

- foundation ratification, schema, vocabulary, and commands;
- UI definition formation;
- workspace identity and viewport expression foundations;
- viewport dynamic product target allocation;
- render product surface foundation bundle;
- drawing domain contracts and the implemented drawing Phase 2 through Phase 5.1 foundation.

Do not move them mechanically. Each move needs link updates, validation, and a
clear record of any implementation drift.

## Operating Rule

The current code-facing answer is:

1. Treat `WR-018` rendered-world V1 and `WR-025` UI Runtime V2 interaction
   formation as the P0 editor current candidates.
2. Keep `WR-024` editor shell polish ready-next behind `WR-025`; polish may
   proceed only as a retained-UI slice consuming Interaction V2 contracts or as
   explicitly bounded compatibility evidence.
3. Treat `WR-007` as completed evidence; Phase 4 standard ECS extraction/apply
   is the next net phase, but only after a fresh roadmap slice selects it.
4. Treat `WR-001`, `WR-006`, and `WR-007` as completed evidence, not selectable
   implementation batches.
5. Keep `WR-002`, `WR-003`, `WR-004`, and `WR-005` as support-only tracks
   until roadmap evidence explicitly reactivates one of them.
6. Keep `WR-008` code-reachable but product-acceptance-blocked until hardware
   validation evidence exists.
7. Keep gameplay graph, particles, physics, animation, world processes,
   alternate UI execution, and deferred SDF capability detail drafts behind
   their owning contract gates.

After completing any phased implementation, run the phase completion drift-check
routine before starting the next phase.
