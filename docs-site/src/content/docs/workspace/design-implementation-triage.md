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

## Support Only

| ID | Track | Priority | Value | Blocker | Score | Current call | Reactivation evidence |
|---|---|---:|---:|---:|---:|---|---|
| WR-002 | ECS/runtime convergence support for product jobs and diagnostics | P0 | V4 | B2 | 2.7 | Implement as supporting work only when it directly unblocks the SDF-first execution path. The 2026-05-14 pilot landed M5 consumer lag/backpressure diagnostics and F3 plan reporting; F2 remains open. | 2026-05-14 L0 substrate pilot added ECS-owned runtime plan reporting; M5 diagnostics are landed and F2 lifecycle/finalization remains. |
| WR-003 | Render contract follow-ups through product selection and derived residency | P0 | V4 | B2 | 2.4 | Implement only as contract-following FR-1 follow-up from the fully featured renderer roadmap, not as renderer-owned world truth. The 2026-05-14 pilot added prepared-view ownership guards plus selected-source residency derivation and invalidation. | 2026-05-14 L0 substrate pilot added view-ordered product selection snapshots, selected-source residency invalidation, and conflict rejection for selected source state. |
| WR-004 | UI/editor guard and sequencing maintenance | P0 | V4 | B1 | 7.5 | Keep active while any editor surface work lands. The 2026-05-14 parallel batch added entity-table and SDF operation routing/capability guards. | 2026-05-14 entity-table and SDF surface routing/capability guards landed. |
| WR-005 | Design lifecycle cleanup for implemented active designs | P1 | V3 | B1 | 4.0 | Use only for future lifecycle drift or source-of-truth cleanup. Current validation now rejects active designs that claim implementation without an explicit phase-evidence marker and rejects missing crate-doc coverage. | 2026-05-16 docs governance milestone moved implemented active designs into implemented lifecycle, aligned canonical ownership maps with workspace truth, indexed preserved batch artifacts, and hardened docs validation against lifecycle drift, missing crate coverage, stale prompt paths, and stale active-looking batches. |

## Ready Next

| ID | Track | Priority | Value | Blocker | Score | Current call | Main blocker |
|---|---|---:|---:|---:|---:|---|---|
| WR-008 | Native tablet input backend arbitration and diagnostics | P1 | V3 | B3 | 1.0 | Code work can continue after the WR-006 dependency, but product acceptance remains blocked. | Hardware validation is still required for Windows Ink, Wacom Wintab, and macOS Wacom devices. |
| WR-009 | Native multi-window editor presentation | P2 | V3 | B3 | 0.7 | Design is active, but it should not preempt current product-surface and post-6D work. | Runtime window state and render surface handling are still singleton-shaped; second-window productization needs window-scoped runtime, input, UI frame, and swapchain ownership. |
| WR-010 | Render fragment and data-driven maturity R10 | P2 | V3 | B3 | 0.7 | Keep queued after render surface, ergonomics, persistence, inspection, and product-surface follow-ups in engine/roadmaps/fully-featured-renderer-roadmap.md. | Fragment assets and hot reload need stable target aliases, prepared flow invocations, diagnostics, and a product priority. |
| WR-022 | SDF Prefab V2 | P2 | V4 | B3 | 0.8 | Do not implement runtime prefab instances until rendered-world V1 and source-backed asset identity are stable; prefab renderer handoff must consume the fully featured renderer roadmap instead of adding a parallel scene path. | Runtime prefab instancing waits for rendered-world V1, source-backed prefab identity adapters, and accepted product ownership. |
| WR-023 | ECS Parallel Execution | P2 | V3 | B2 | 0.5 | Design now, implement later after diagnostics and deterministic merge policy are accepted. | Public parallel execution waits for accepted deterministic merge policy, blocked-parallelism diagnostics, and serial equivalence tests. |
| WR-024 | Editor Shell Polish | P0 | V4 | B1 | 4.0 | Ready-next only after the WR-025 doctrine repair is committed; consume landed popup, scroll, chrome, docking, and status overflow slices instead of defining local policy. | Interaction V2 contract/migration spine must lead; polish can only proceed as a retained-UI slice consuming those contracts or as explicitly bounded compatibility evidence. |

## Completed Evidence

| ID | Track | Priority | Value | Blocker | Score | Completion quality | Quality gaps | Current decision | Evidence |
|---|---|---:|---:|---:|---:|---|---|---|---|
| WR-001 | Post-Phase 6D product-job and Draw cache follow-up | P0 | V5 | B2 | 4.8 | bounded_contract | Remaining WR-001 work needs fresh runtime product-job evidence before reactivation. | DRF3 landed through the WR-001 batch; keep any remaining WR-001 work as a bounded follow-up and do not reopen procgen bake/rollback. | 2026-05-15 DRF3 preview/final product-surface bridge landed through docs-site/src/content/docs/reports/batches/2026-05-14-wr-001-product-job-draw-bridge/batch.toml and WR-006 completed DRF4/DRF5 GPU proof and promotion; remaining WR-001 work needs fresh runtime product-job evidence before reactivation. |
| WR-006 | Runenwerk Draw DRF4 through DRF5 | P1 | V4 | B2 | 2.2 | runtime_proven | None recorded | DRF4 and DRF5 are complete; keep CPU tile formation canonical while future Draw work builds on validated GPU promotion/fallback. | 2026-05-15 WR-006 landed DRF4 GPU ink proof and DRF5 GPU promotion/fallback through public render-flow/product-surface APIs; closeout evidence is recorded in docs-site/src/content/docs/reports/closeouts/wr-006-draw-drf4-drf5/closeout.md. |
| WR-007 | Multiplayer replication Phase 1 to Phase 3 | P1 | V4 | B2 | 1.4 | bounded_contract | Future replication rows still need declarative replication and ECS component extraction evidence before broader completion claims. | Phase 1-3 is complete; future replication work should build on the accepted ACK/baseline outcome contract and engine bridge checkpoint behavior instead of reopening ad hoc baseline mutation. | 2026-05-15 Phase 1-3 landed ACK/baseline hardening, delta lifecycle normalization, and engine bridge baseline convergence; closeout evidence is recorded in docs-site/src/content/docs/reports/closeouts/wr-007-multiplayer-replication-phase-1-3/closeout.md. |
| WR-012 | Neutral graph substrate boundary | P2 | V3 | B2 | 1.8 | bounded_contract | This is a policy and substrate boundary closeout; future semantic graph products still need owning-domain implementation evidence. | Graph substrate policy is complete; future semantic graph work must start from one concrete owning domain and one formed product target, not a broad graph platform. | 2026-05-16 ADR 0010 accepted the graph substrate/canvas boundary and preserved the former draft rationale; closeout evidence is recorded in docs-site/src/content/docs/reports/closeouts/wr-012-neutral-graph-substrate-boundary/closeout.md. |
| WR-018 | Rendered World V1 | P0 | V5 | B2 | 3.0 | bounded_contract | Storage-buffer scene packet expansion and later product producers remain future bounded render-flow work.<br>Windowed GPU truth smoke remains environment-gated and does not make this row perfectionist verified. | All editor SDF primitive entities now render and pick through one app-runtime extracted viewport scene packet before material, prefab, terrain, or field visualization production work expands. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-018-rendered-world-v1/closeout.md. Multi-entity SDF primitive extraction, scene-product shader packet use, CPU picking from the render-state packet, picking shader packet decode, and viewport render-job uniform override tests are implemented and validated. |
| WR-019 | Field Visualizer product workflow | P1 | V4 | B2 | 2.4 | bounded_contract | Field producers remain product-debug workflow evidence, not a full field-authoring or simulation product family.<br>Windowed GPU truth smoke remains environment-gated and does not make this row perfectionist verified. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md. Viewport-owned field visualizer settings, granular patch actions, product-aware slice bounds, routed controls, persisted hydration, stable product target identity, and strict diagnostics are implemented and validated. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md. Viewport-owned field visualizer settings, granular patch actions, product-aware slice bounds, routed controls, persisted hydration, stable product target identity, and strict diagnostics are implemented and validated. |
| WR-020 | Source-backed Asset Core Contracts | P0 | V5 | B2 | 1.9 | bounded_contract | Editor adapter execution, external importer execution, cache garbage collection, and runtime hot reload remain downstream adapter/product work. | Source-backed asset semantic truth now lives in domain/asset for SDF graph, field product, material graph/material, UI definition, and prefab descriptor families before broad external import work or editor adapter expansion. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-020-source-backed-asset-core-contracts/closeout.md. Domain-owned project catalog descriptors, importer-aware source descriptors, deterministic import plans, dependency graph contracts, diagnostics, ratification, and prior-valid artifact preservation are implemented and validated in domain/asset. WR-026 editor adapters remain downstream. |
| WR-026 | Source-backed Asset Editor Adapters | P1 | V4 | B2 | 2.4 | bounded_contract | Later material, prefab, external importer, cache-GC, and runtime hot-reload consumers still need product-specific evidence. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md after implementation and validation. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md after implementation and validation. |
| WR-021 | Material Lab and material preview products | P1 | V4 | B2 | 0.9 | runtime_proven | WR-028 supersedes the inherited rich Material Lab graph, live texture view, SDF assignment persistence, and SDF per-hit material-selection gaps for the SDF primitive path.<br>Model/mesh material binding is not closed by WR-021 or WR-028; WR-029 owns model/mesh source identity, submesh/material-region assignment, and renderable_index ABI semantics. | WR-021 is completed by the superseding closeout at docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md. The accepted product spine is source MaterialGraph V2 -> ratification -> typed executable IR -> generated WGSL -> validated shader artifacts -> KTX2 texture residency -> scene material slots -> material preview product -> viewport selection -> GPU pixel evidence. Dynamic plugins, multiple material catalogs, and richer visual polish remain later slices. | Completed after the WR-021 perfectionist repair. Evidence now covers source-owned V2 material graph layout, direct source-document Material Lab projection, ui_graph_editor shell contracts, immediate source edit ratification refresh, topologically ordered executable IR, generated preview/scene shader artifacts, KTX2-backed typed texture descriptors, exact generated shader load gating, generated default scene material bootstrap, PreparedSceneMaterialBundle consumption, group-1 GPU-resident material texture/sampler bind groups, runtime scene packet material slot indices, dedicated material-preview producer output, selected-primary viewport presentation, prior-valid material handoff preservation, and mandatory GPU proof for generated scene and preview shaders sampling Texture2D and Texture3D KTX2 resources. |
| WR-025 | UI Runtime V2 and interaction formation | P0 | V5 | B2 | 4.8 | bounded_contract | WR-024 retained UI polish remains the next product-facing shell polish slice.<br>Alternate UI execution strategies remain deferred behind separate ADR or accepted design evidence. | Accepted ADR 0009 inserts an execution-neutral interaction formation layer before retained UI products while keeping retained UI as the first execution target and renderer output as derived product data. | 2026-05-15 WR-025 batches landed named retained UI migration slices for IV2-menu-stack, IV2-scroll-ownership, IV2-menu-sizing, IV2-chrome-slots, IV2-dock-drop-zones, and IV2-status-and-viewport-arbitration. The 2026-05-15 WR-025 doctrine repair adds explicit invalid dock/drop target semantics, non-dismissible viewport chrome/status popup policy, and behavior guards for viewport status dispatch. Next retained UI polish should consume those contracts rather than add local interaction policy. |
| WR-027 | Completion Quality And Perfectionist Audit Gate | P0 | V5 | B1 | 3.0 | bounded_contract | Existing completed rows were classified honestly, but most have not been individually audited to perfectionist_verified.<br>WR-028 now owns the explicit Material Lab perfectionist follow-up for rich visual graph editing, live texture views, and per-object scene material binding. | Completed roadmap rows now carry an explicit completion quality tier, known quality gaps, and optional audit reference so completed history remains honest without implying perfectionist verification. | 2026-05-17 completion quality metadata, validation, generated docs, perfectionist audit report, workflow gate docs, and material compiler module-structure repair landed with closeout evidence in docs-site/src/content/docs/reports/closeouts/wr-027-completion-quality-and-perfectionist-audit-gate/closeout.md. |
| WR-028 | Perfectionist Material Lab Texture Views And Scene Material Binding | P0 | V5 | B1 | 1.2 | perfectionist_verified | None recorded | WR-028 is SDF-scoped and perfectionist-verified after repairing the source-to-GPU material graph proof and graph canvas UX evidence. WR-021 remains runtime-proven history; WR-029 carries model/mesh material binding forward. | WR-028 closeout and proof manifest now record source-backed MaterialGraphSourceFileV2 texture edits flowing through lowering, product generation, shader/material-table identity changes, and preview/scene GPU captures. WR-029 still owns model/mesh material binding. |

## Blocked Or Deferred

| ID | Track | Priority | Value | Blocker | Score | Why it is not ready now |
|---|---|---:|---:|---:|---:|---|
| WR-011 | Gameplay Graph ATR IR and ECS lowering | P2 | V4 | B4 | 0.3 | Missing domain/gameplay/events, domain/gameplay/actions, domain/gameplay/state, and domain/gameplay/quests; SDF physics relation readiness and authority diagnostics also need owning contracts. |
| WR-013 | Scripting and runtime gameplay bridge | P2 | V3 | B4 | 0.3 | It depends on M6 formed procedural/gameplay product contracts and a language-neutral runtime boundary. Rhai can be the first adapter only after the domain contract exists. |
| WR-014 | Particles, physics, animation, and world-process product tracks | P3 | V3 | B4 | 0.2 | Their domain docs and formed product contracts are still missing or deferred. They follow the product-job/query snapshot/publication barrier substrate. |
| WR-015 | SDF character, vegetation, atmosphere, water, VFX, and influence AI drafts | P3 | V2 | B5 | 0.2 | These remaining detail drafts are explicitly deferred. Reactivate only after the relevant product ownership, renderer/runtime handoff, query policy, and authority contracts are promoted. |
| WR-016 | Compiled-reactive UI and ECS-driven UI execution strategies | P3 | V2 | B5 | 0.1 | The current retained UI path is the active implementation. Alternative execution strategies require a new active design or accepted ADR before code, and must consume normalized definitions plus formed interaction contracts. |
| WR-017 | Gameplay actions, powers, runtime, and power editor | P3 | V2 | B5 | 0.2 | These deferred designs still need remaining decisions and must not precede the narrower gameplay graph and domain contract sequence. |
| WR-029 | Model Mesh Material Binding | P1 | V4 | B3 | 1.1 | Model/mesh source identity, submesh/material-region assignment, and renderable_index ABI semantics are not yet accepted. |
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
