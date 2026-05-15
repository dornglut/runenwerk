---
title: Roadmap Decision Register
description: Workspace scorecard and decision register for roadmap sequencing.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-15
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
| WR-001 | Post-Phase 6D product-job and Draw cache follow-up | Core delivery | completed | L0 | Continue next slice | 5 | 2 | 4 | 4 | 5 | 3 | 0.8 | 4.8 | Candidate after Draw reach exists | Performance | 2026-05-15 DRF3 preview/final product-surface bridge landed and WR-006 completed DRF4/DRF5 GPU proof and promotion; remaining WR-001 work needs fresh runtime product-job evidence before reactivation. | DRF3 landed through the WR-001 batch; keep any remaining WR-001 work as a bounded follow-up and do not reopen procgen bake/rollback. |
| WR-002 | ECS/runtime convergence support for product jobs and diagnostics | Core support | support_only | L0 | Supporting now | 4 | 2 | 3 | 5 | 5 | 5 | 0.8 | 2.7 | N/A | Neutral | 2026-05-14 L0 substrate pilot added ECS-owned runtime plan reporting; M5 diagnostics are landed and F2 lifecycle/finalization remains. | Continue only where lifecycle/finalization unblocks product jobs or diagnostics. |
| WR-003 | Render contract follow-ups through product selection and derived residency | Core support | support_only | L0 | Supporting now | 4 | 2 | 3 | 4 | 4 | 5 | 0.8 | 2.4 | N/A | Neutral | 2026-05-14 L0 substrate pilot added view-ordered product selection snapshots, selected-source residency invalidation, and conflict rejection for selected source state. | Continue as contract-following support, not renderer-owned world truth. |
| WR-004 | UI/editor guard and sequencing maintenance | Guardrail | support_only | L0 | Always-on guard | 4 | 1 | 4 | 4 | 3 | 2 | 1.0 | 7.5 | N/A | Basic | 2026-05-14 entity-table and SDF surface routing/capability guards landed. | Keep active in parallel; score does not make it the main roadmap. |
| WR-005 | Design lifecycle cleanup for implemented active designs | Docs | support_only | L0 | Docs now | 3 | 1 | 2 | 3 | 2 | 2 | 0.8 | 4.0 | N/A | Neutral | 2026-05-14 surface workflow design moved to implemented lifecycle; more candidates remain. | Useful now when documentation capacity is available. |
| WR-006 | Runenwerk Draw DRF4 through DRF5 | Core delivery | completed | L1 | Completed | 4 | 2 | 3 | 3 | 4 | 5 | 0.8 | 2.2 | Candidate after workflow reach exists | Performance | 2026-05-15 WR-006 landed DRF4 GPU ink proof and DRF5 GPU promotion/fallback through public render-flow/product-surface APIs; validation passed with cargo test -p runenwerk_draw and cargo test -p engine. | DRF4 and DRF5 are complete; keep CPU tile formation canonical while future Draw work builds on validated GPU promotion/fallback. |
| WR-007 | Multiplayer replication Phase 1 to Phase 3 | Core delivery | current_candidate | L1 | Ready next | 4 | 2 | 2 | 4 | 4 | 8 | 0.8 | 1.4 | N/A | Neutral | ACK/baseline hardening and delta lifecycle tests. | Current L1 implementation candidate after WR-006 completed the Draw GPU proof and promotion slice. |
| WR-008 | Native tablet input backend arbitration and diagnostics | Product discovery | ready_next | L1 | Product evidence gate | 3 | 3 | 2 | 3 | 2 | 5 | 0.5 | 1.0 | Candidate after hardware target exists | Basic | Hardware validation across Windows Ink, Wacom Wintab, and macOS Wacom. | WR-006 dependency is complete; code can continue, but product acceptance is hardware-blocked. |
| WR-009 | Native multi-window editor presentation | Productization | ready_next | L2 | Runtime gate | 3 | 3 | 2 | 3 | 3 | 8 | 0.5 | 0.7 | Candidate after workflow reach exists | Performance | Window-scoped runtime, input, UI frame, and swapchain ownership proof. | Keep gated behind runtime/window-scope contracts. |
| WR-010 | Render fragment and data-driven maturity R10 | Productization | ready_next | L2 | Product priority gate | 3 | 3 | 2 | 3 | 3 | 8 | 0.5 | 0.7 | N/A | Neutral | Stable aliases, prepared flow invocation, hot reload, diagnostics, and inspection evidence. | Queue after render surface, ergonomics, persistence, and inspection priorities. |
| WR-011 | Gameplay Graph ATR IR and ECS lowering | Contract | blocked_deferred | L2 | Domain contract gate | 4 | 4 | 2 | 4 | 4 | 13 | 0.3 | 0.3 | N/A | Neutral | domain/gameplay event, action, state, quest, authority, and lowering contracts. | Blocked until narrower gameplay contracts exist. |
| WR-012 | General semantic graph implementation | Contract | blocked_deferred | L2 | Domain contract gate | 3 | 4 | 1 | 3 | 3 | 13 | 0.3 | 0.2 | N/A | Neutral | One owning domain and one formed product target. | Do not build a broad graph platform first. |
| WR-013 | Scripting and runtime gameplay bridge | Contract | blocked_deferred | L3 | Domain contract gate | 3 | 4 | 2 | 3 | 3 | 13 | 0.3 | 0.3 | N/A | Neutral | Language-neutral runtime boundary and formed gameplay products. | Rhai can be first adapter only after the domain contract exists. |
| WR-014 | Particles, physics, animation, and world-process product tracks | Contract | blocked_deferred | L3 | Product contract gate | 3 | 4 | 1 | 3 | 3 | 13 | 0.3 | 0.2 | N/A | Neutral | Owning domain docs and formed product contracts. | Follow product-job, query snapshot, and publication substrate maturity. |
| WR-015 | SDF prefab, character, vegetation, atmosphere, water, VFX, and influence AI drafts | Deferred | blocked_deferred | L4 | Policy deferred | 2 | 5 | 1 | 2 | 2 | 13 | 0.3 | 0.2 | N/A | Exciter | Active design, ADR, or roadmap promotion. | Keep deferred until product ownership and handoff contracts are promoted. |
| WR-016 | Compiled-reactive UI and ECS-driven UI execution strategies | Deferred | blocked_deferred | L4 | Policy deferred | 2 | 5 | 1 | 2 | 1 | 13 | 0.3 | 0.1 | N/A | Neutral | Accepted ADR or active design changing the retained UI default. | Keep deferred; retained UI remains the active implementation path. |
| WR-017 | Gameplay actions, powers, runtime, and power editor | Deferred | blocked_deferred | L4 | Policy deferred | 2 | 5 | 1 | 2 | 2 | 13 | 0.3 | 0.2 | N/A | Exciter | Narrower gameplay graph and domain contract sequence. | Keep deferred behind gameplay contract work. |

## Review Rules

- Re-score after a closeout report changes the evidence for a track.
- Re-score when a blocker moves between `B1` through `B5`.
- Keep RICE blank as `N/A` until there is a credible reach estimate.
- Never promote `B5` work by score alone; require an accepted design, ADR, or
  owning roadmap update.
