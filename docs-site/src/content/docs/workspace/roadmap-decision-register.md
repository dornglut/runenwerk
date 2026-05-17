---
title: Roadmap Decision Register
description: Workspace scorecard and decision register for roadmap sequencing.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-16
related:
  - ./planning-methods.md
  - ./design-implementation-triage.md
  - ./repo-execution-priority-checklist.md
  - ./roadmap-index.md
  - ./roadmap-items.yaml
  - ./schemas/roadmap-items.schema.json
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
[roadmap-items.yaml](./roadmap-items.yaml). Do not edit the table directly;
update the YAML source and run `task roadmap:render`.

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

| ID | Track | Lane | Planning state | Dependency level | Gate | V | B | TC | RR/OE | DU | E | C | A-WSJF | RICE | Kano | Next evidence | Current decision |
|---|---|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|
| WR-001 | Post-Phase 6D product-job and Draw cache follow-up | Core delivery | completed | L0 | Continue next slice | 5 | 2 | 4 | 4 | 5 | 3 | 0.8 | 4.8 | Candidate after Draw reach exists | Performance | 2026-05-15 DRF3 preview/final product-surface bridge landed through docs-site/src/content/docs/reports/batches/2026-05-14-wr-001-product-job-draw-bridge/batch.toml and WR-006 completed DRF4/DRF5 GPU proof and promotion; remaining WR-001 work needs fresh runtime product-job evidence before reactivation. | DRF3 landed through the WR-001 batch; keep any remaining WR-001 work as a bounded follow-up and do not reopen procgen bake/rollback. |
| WR-002 | ECS/runtime convergence support for product jobs and diagnostics | Core support | support_only | L0 | Supporting now | 4 | 2 | 3 | 5 | 5 | 5 | 0.8 | 2.7 | N/A | Neutral | 2026-05-14 L0 substrate pilot added ECS-owned runtime plan reporting; M5 diagnostics are landed and F2 lifecycle/finalization remains. | Continue only where lifecycle/finalization unblocks product jobs or diagnostics. |
| WR-003 | Render contract follow-ups through product selection and derived residency | Core support | support_only | L0 | Supporting now | 4 | 2 | 3 | 4 | 4 | 5 | 0.8 | 2.4 | N/A | Neutral | 2026-05-14 L0 substrate pilot added view-ordered product selection snapshots, selected-source residency invalidation, and conflict rejection for selected source state. | Continue as contract-following support, not renderer-owned world truth. |
| WR-004 | UI/editor guard and sequencing maintenance | Guardrail | support_only | L0 | Always-on guard | 4 | 1 | 4 | 4 | 3 | 2 | 1.0 | 7.5 | N/A | Basic | 2026-05-14 entity-table and SDF surface routing/capability guards landed. | Keep active in parallel; score does not make it the main roadmap. |
| WR-005 | Design lifecycle cleanup for implemented active designs | Docs | support_only | L0 | Docs now | 3 | 1 | 2 | 3 | 2 | 2 | 0.8 | 4.0 | N/A | Neutral | 2026-05-16 docs governance milestone moved implemented active designs into implemented lifecycle, aligned canonical ownership maps with workspace truth, indexed preserved batch artifacts, and hardened docs validation against lifecycle drift, missing crate coverage, stale prompt paths, and stale active-looking batches. | Keep as an always-available docs governance support lane; the current no-shortcuts cleanup milestone is complete and future drift should fail through task docs:validate. |
| WR-006 | Runenwerk Draw DRF4 through DRF5 | Core delivery | completed | L1 | Completed | 4 | 2 | 3 | 3 | 4 | 5 | 0.8 | 2.2 | Candidate after workflow reach exists | Performance | 2026-05-15 WR-006 landed DRF4 GPU ink proof and DRF5 GPU promotion/fallback through public render-flow/product-surface APIs; closeout evidence is recorded in docs-site/src/content/docs/reports/closeouts/wr-006-draw-drf4-drf5/closeout.md. | DRF4 and DRF5 are complete; keep CPU tile formation canonical while future Draw work builds on validated GPU promotion/fallback. |
| WR-007 | Multiplayer replication Phase 1 to Phase 3 | Core delivery | completed | L1 | Completed | 4 | 2 | 2 | 4 | 4 | 8 | 0.8 | 1.4 | N/A | Neutral | 2026-05-15 Phase 1-3 landed ACK/baseline hardening, delta lifecycle normalization, and engine bridge baseline convergence; closeout evidence is recorded in docs-site/src/content/docs/reports/closeouts/wr-007-multiplayer-replication-phase-1-3/closeout.md. | Phase 1-3 is complete; future replication work should build on the accepted ACK/baseline outcome contract and engine bridge checkpoint behavior instead of reopening ad hoc baseline mutation. |
| WR-008 | Native tablet input backend arbitration and diagnostics | Product discovery | ready_next | L1 | Product evidence gate | 3 | 3 | 2 | 3 | 2 | 5 | 0.5 | 1.0 | Candidate after hardware target exists | Basic | Hardware validation across Windows Ink, Wacom Wintab, and macOS Wacom. | WR-006 dependency is complete; code can continue, but product acceptance is hardware-blocked. |
| WR-009 | Native multi-window editor presentation | Productization | ready_next | L2 | Runtime gate | 3 | 3 | 2 | 3 | 3 | 8 | 0.5 | 0.7 | Candidate after workflow reach exists | Performance | Window-scoped runtime, input, UI frame, and swapchain ownership proof. | Keep gated behind runtime/window-scope contracts. |
| WR-010 | Render fragment and data-driven maturity R10 | Productization | ready_next | L2 | Product priority gate | 3 | 3 | 2 | 3 | 3 | 8 | 0.5 | 0.7 | N/A | Neutral | Stable aliases, prepared flow invocation, hot reload, diagnostics, and inspection evidence. | Queue as FR-7 after render surface, ergonomics, persistence, and inspection priorities. |
| WR-011 | Gameplay Graph ATR IR and ECS lowering | Contract | blocked_deferred | L2 | Domain contract gate | 4 | 4 | 2 | 4 | 4 | 13 | 0.3 | 0.3 | N/A | Neutral | domain/gameplay event, action, state, quest, authority, and lowering contracts. | Blocked until narrower gameplay contracts exist. |
| WR-012 | Neutral graph substrate boundary | Core support | completed | L0 | Completed | 3 | 2 | 2 | 3 | 3 | 5 | 0.8 | 1.8 | N/A | Neutral | 2026-05-16 ADR 0010 accepted the graph substrate/canvas boundary and preserved the former draft rationale; closeout evidence is recorded in docs-site/src/content/docs/reports/closeouts/wr-012-neutral-graph-substrate-boundary/closeout.md. | Graph substrate policy is complete; future semantic graph work must start from one concrete owning domain and one formed product target, not a broad graph platform. |
| WR-013 | Scripting and runtime gameplay bridge | Contract | blocked_deferred | L3 | Domain contract gate | 3 | 4 | 2 | 3 | 3 | 13 | 0.3 | 0.3 | N/A | Neutral | Language-neutral runtime boundary and formed gameplay products. | Rhai can be first adapter only after the domain contract exists. |
| WR-014 | Particles, physics, animation, and world-process product tracks | Contract | blocked_deferred | L3 | Product contract gate | 3 | 4 | 1 | 3 | 3 | 13 | 0.3 | 0.2 | N/A | Neutral | Owning domain docs and formed product contracts. | Follow product-job, query snapshot, and publication substrate maturity. |
| WR-015 | SDF character, vegetation, atmosphere, water, VFX, and influence AI drafts | Deferred | blocked_deferred | L4 | Policy deferred | 2 | 5 | 1 | 2 | 2 | 13 | 0.3 | 0.2 | N/A | Exciter | Active design, ADR, or roadmap promotion. SDF prefab composition has moved to WR-022 and is no longer part of this deferred bucket. | Keep remaining deferred drafts until product ownership and handoff contracts are promoted. |
| WR-016 | Compiled-reactive UI and ECS-driven UI execution strategies | Deferred | blocked_deferred | L4 | Policy deferred | 2 | 5 | 1 | 2 | 1 | 13 | 0.3 | 0.1 | N/A | Neutral | Separate accepted ADR or active design after Interaction V2 names a concrete alternate target, formation product, invalidation/debug model, command boundary, and guard suite. | Keep deferred; retained UI remains the active implementation path and Interaction V2 is a contract layer for retained UI first, not permission to implement alternate execution. |
| WR-017 | Gameplay actions, powers, runtime, and power editor | Deferred | blocked_deferred | L4 | Policy deferred | 2 | 5 | 1 | 2 | 2 | 13 | 0.3 | 0.2 | N/A | Exciter | Narrower gameplay graph and domain contract sequence. | Keep deferred behind gameplay contract work. |
| WR-018 | Rendered World V1 | Core delivery | completed | L1 | Completed | 5 | 2 | 4 | 5 | 5 | 5 | 0.8 | 3.0 | SDF-first editor world reach | Basic | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-018-rendered-world-v1/closeout.md. Multi-entity SDF primitive extraction, scene-product shader packet use, CPU picking from the render-state packet, picking shader packet decode, and viewport render-job uniform override tests are implemented and validated. | All editor SDF primitive entities now render and pick through one app-runtime extracted viewport scene packet before material, prefab, terrain, or field visualization production work expands. |
| WR-019 | Field Visualizer product workflow | Product workflow | completed | L2 | After rendered-world V1 | 4 | 2 | 3 | 4 | 4 | 5 | 0.8 | 2.4 | Product debug workflow after viewport packet stability | Performance | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md. Viewport-owned field visualizer settings, granular patch actions, product-aware slice bounds, routed controls, persisted hydration, stable product target identity, and strict diagnostics are implemented and validated. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md. Viewport-owned field visualizer settings, granular patch actions, product-aware slice bounds, routed controls, persisted hydration, stable product target identity, and strict diagnostics are implemented and validated. |
| WR-020 | Source-backed Asset Core Contracts | Core support | completed | L1 | Completed | 5 | 2 | 4 | 5 | 5 | 8 | 0.8 | 1.9 | Stable asset identity before material and prefab implementation | Basic | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-020-source-backed-asset-core-contracts/closeout.md. Domain-owned project catalog descriptors, importer-aware source descriptors, deterministic import plans, dependency graph contracts, diagnostics, ratification, and prior-valid artifact preservation are implemented and validated in domain/asset. WR-026 editor adapters remain downstream. | Source-backed asset semantic truth now lives in domain/asset for SDF graph, field product, material graph/material, UI definition, and prefab descriptor families before broad external import work or editor adapter expansion. |
| WR-026 | Source-backed Asset Editor Adapters | Core support | completed | L2 | After asset core contracts | 4 | 2 | 3 | 4 | 4 | 5 | 0.8 | 2.4 | Editor catalog workflow after domain asset identity stabilizes | Basic | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md after implementation and validation. | 2026-05-16 closeout evidence landed in docs-site/src/content/docs/reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md after implementation and validation. |
| WR-021 | Material Lab and material preview products | Product workflow | completed | L3 | After rendered-world V1 and Field Visualizer | 4 | 2 | 3 | 4 | 4 | 8 | 0.5 | 0.9 | Material authoring after render/product handoff stabilizes | Performance | Completed after the WR-021 perfectionist repair. Evidence now covers source-owned V2 material graph layout, direct source-document Material Lab projection, ui_graph_editor shell contracts, immediate source edit ratification refresh, topologically ordered executable IR, generated preview/scene shader artifacts, KTX2-backed typed texture descriptors, exact generated shader load gating, generated default scene material bootstrap, PreparedSceneMaterialBundle consumption, group-1 GPU-resident material texture/sampler bind groups, runtime scene packet material slot indices, dedicated material-preview producer output, selected-primary viewport presentation, prior-valid material handoff preservation, and mandatory GPU proof for generated scene and preview shaders sampling Texture2D and Texture3D KTX2 resources. | WR-021 is completed by the superseding closeout at docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md. The accepted product spine is source MaterialGraph V2 -> ratification -> typed executable IR -> generated WGSL -> validated shader artifacts -> KTX2 texture residency -> scene material slots -> material preview product -> viewport selection -> GPU pixel evidence. Dynamic plugins, multiple material catalogs, and richer visual polish remain later slices. |
| WR-022 | SDF Prefab V2 | Product workflow | ready_next | L3 | V2 gated | 4 | 3 | 2 | 4 | 3 | 8 | 0.5 | 0.8 | Runtime prefab instancing after rendered-world and asset identity stabilize | Exciter | Source/catalog prefab identity, descriptor diagnostics, and approved runtime instancing sequence. | Keep prefab composition active as design and catalog identity now; runtime instancing waits for V2 gates. |
| WR-023 | ECS Parallel Execution | Core support | ready_next | L3 | Design first | 3 | 2 | 3 | 5 | 3 | 13 | 0.5 | 0.5 | Parallel execution only after deterministic system contract acceptance | Performance | Accepted design for Send + Sync system constraints, access sharding, per-wave command queues, deterministic merges, and blocked-parallelism diagnostics. | Keep ECS execution serial during rendered-world V1; harden product jobs as the active multithreaded path. |
| WR-024 | Editor Shell Polish | Guardrail | ready_next | L1 | After Interaction V2 contract spine | 4 | 1 | 4 | 3 | 4 | 3 | 0.8 | 4.0 | Immediate viewport usability and menu clarity | Basic | Retained-UI slice evidence after the relevant named Interaction V2 contracts exist for IV2-menu-stack, IV2-scroll-ownership, IV2-menu-sizing, IV2-chrome-slots, IV2-dock-drop-zones, and IV2-status-and-viewport-arbitration. | Treat editor shell polish as a retained-UI implementation slice under the broader Interaction V2 contract layer, not as a pile of isolated cosmetic fixes. |
| WR-025 | UI Runtime V2 and interaction formation | Architecture guardrail | completed | L1 | Accepted architecture gate | 5 | 2 | 4 | 5 | 4 | 3 | 0.8 | 4.8 | Prevent repeated editor UI contract regressions | Basic | 2026-05-15 WR-025 batches landed named retained UI migration slices for IV2-menu-stack, IV2-scroll-ownership, IV2-menu-sizing, IV2-chrome-slots, IV2-dock-drop-zones, and IV2-status-and-viewport-arbitration. The 2026-05-15 WR-025 doctrine repair adds explicit invalid dock/drop target semantics, non-dismissible viewport chrome/status popup policy, and behavior guards for viewport status dispatch. Next retained UI polish should consume those contracts rather than add local interaction policy. | Accepted ADR 0009 inserts an execution-neutral interaction formation layer before retained UI products while keeping retained UI as the first execution target and renderer output as derived product data. |

## Review Rules

- Re-score after a closeout report changes the evidence for a track.
- Re-score when a blocker moves between `B1` through `B5`.
- Keep RICE blank as `N/A` until there is a credible reach estimate.
- Never promote `B5` work by score alone; require an accepted design, ADR, or
  owning roadmap update.
