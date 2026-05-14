---
title: Design Implementation Triage
description: Current workspace triage of active and deferred design work by implementation readiness, blocker weight, and value-weighted dependency level.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related:
  - ./planning-methods.md
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

## Implement Now

| ID | Track | Priority | Value | Blocker | Score | Current call | First implementation move |
|---|---|---:|---:|---:|---:|---|---|
| WR-001 | Post-Phase 6D product-job and Draw cache follow-up | P0 | V5 | B2 | 4.8 | Implement next. Phase 6D / M6.2C is complete; the next code track should resume from the runtime product job and Draw roadmaps without reopening procgen bake/rollback. | Continue from [runtime-product-job-executor-roadmap.md](../engine/roadmaps/runtime-product-job-executor-roadmap.md), [runenwerk-draw/roadmap.md](../apps/runenwerk-draw/roadmap.md), and [sdf-first-execution-roadmap.md](./sdf-first-execution-roadmap.md). |
| WR-002 | ECS/runtime convergence support for product jobs and diagnostics | P0 | V4 | B2 | 2.7 | Implement as supporting work only when it directly unblocks the SDF-first execution path. | Close remaining lifecycle finalization, deterministic registration/plan reporting, and consumer lag/backpressure diagnostics from [net/ecs-runtime-prioritized-roadmap.md](../net/ecs-runtime-prioritized-roadmap.md). |
| WR-003 | Render immediate follow-ups through product-selection and derived-residency contracts | P0 | V4 | B2 | 2.4 | Implement only as contract-following follow-up, not as renderer-owned world truth. | Audit current R4/R6/R7 code truth first, then use [engine/plugins/render/docs/roadmap.md](../engine/plugins/render/docs/roadmap.md) and [render-product-surface-foundation-bundle-design.md](../design/active/render-product-surface-foundation-bundle-design.md). |
| WR-004 | UI/editor guard and sequencing maintenance | P0 | V4 | B1 | 7.5 | Keep active while any editor surface work lands. Its score reflects low effort and high risk reduction, not replacement of the main roadmap. | Preserve structural routing, capability gating, and surface ownership guard coverage from [domain/ui/roadmap.md](../domain/ui/roadmap.md). |
| WR-005 | Design lifecycle cleanup for implemented active designs | P1 | V3 | B1 | 4.0 | Can be done now as documentation work. It reduces drift but should not interrupt P0 code if code capacity is scarce. | Review active designs that already say implemented, then promote or move them according to [design/README.md](../design/README.md). |

## Ready Next

| ID | Track | Priority | Value | Blocker | Score | Current call | Main blocker |
|---|---|---:|---:|---:|---:|---|---|
| WR-006 | Runenwerk Draw rendering foundation DRF2 through DRF5 | P1 | V4 | B2 | 1.4 | Ready next after the current post-6D product-job and Draw cache work is stable. | DRF2 app-derived cache is only partially complete; DRF3 to DRF5 must keep CPU tile formation canonical and use public render-flow/product-surface APIs. |
| WR-007 | Multiplayer replication Phase 1 to Phase 3 | P1 | V4 | B2 | 1.4 | Ready next for net hardening. | ACK/baseline hardening and delta lifecycle rules need tests and code evidence before broader declarative replication. |
| WR-008 | Native tablet input backend arbitration and diagnostics | P1 | V3 | B3 | 1.0 | Code work can continue, but product acceptance remains blocked. | Hardware validation is still required for Windows Ink, Wacom Wintab, and macOS Wacom devices. |
| WR-009 | Native multi-window editor presentation | P2 | V3 | B3 | 0.7 | Design is active, but it should not preempt current product-surface and post-6D work. | Runtime window state and render surface handling are still singleton-shaped; second-window productization needs window-scoped runtime, input, UI frame, and swapchain ownership. |
| WR-010 | Render fragment/data-driven maturity R10 | P2 | V3 | B3 | 0.7 | Keep queued after render surface, ergonomics, persistence, and inspection follow-ups. | Fragment assets and hot reload need stable target aliases, prepared flow invocations, diagnostics, and a product priority. |

## Blocked Or Deferred

| ID | Track | Priority | Value | Blocker | Score | Why it is not ready now |
|---|---|---:|---:|---:|---:|---|
| WR-011 | Gameplay Graph ATR IR and ECS lowering | P2 | V4 | B4 | 0.3 | Missing `domain/gameplay/events`, `domain/gameplay/actions`, `domain/gameplay/state`, and `domain/gameplay/quests`; SDF physics relation readiness and authority diagnostics also need owning contracts. |
| WR-012 | General semantic graph implementation | P2 | V3 | B4 | 0.2 | The policy is active, but implementation must start from one concrete owning domain and one formed product target, not a broad graph platform. |
| WR-013 | Scripting and runtime gameplay bridge | P2 | V3 | B4 | 0.3 | It depends on M6 formed procedural/gameplay product contracts and a language-neutral runtime boundary. Rhai can be the first adapter only after the domain contract exists. |
| WR-014 | Particles, physics, animation, and world-process product tracks | P3 | V3 | B4 | 0.2 | Their domain docs and formed product contracts are still missing or deferred. They follow the product-job/query snapshot/publication barrier substrate. |
| WR-015 | SDF prefab, character animation, vegetation, atmosphere, water/wetness, influence AI, VFX, and larger world-process drafts | P3 | V2 | B5 | 0.2 | These are explicitly deferred detail drafts. Reactivate only after the relevant product ownership, renderer/runtime handoff, query policy, and authority contracts are promoted. |
| WR-016 | Compiled-reactive UI and ECS-driven UI execution strategies | P3 | V2 | B5 | 0.1 | The current retained UI path is the active implementation. Alternative execution strategies require a new active design or accepted ADR before code. |
| WR-017 | Gameplay actions, powers, power runtime, action runtime, and power editor | P3 | V2 | B5 | 0.2 | These deferred designs still need remaining decisions and must not precede the narrower gameplay graph and domain contract sequence. |

## Design Lifecycle Cleanup Findings

Several active designs now describe implemented foundations. They are not code
blockers, but they should be reviewed for promotion to `accepted/` or
`implemented/` after code truth and tests are rechecked:

- foundation ratification, schema, vocabulary, and commands;
- UI definition formation and surface workflow redesign;
- workspace identity and viewport expression foundations;
- viewport dynamic product target allocation;
- render product surface foundation bundle;
- drawing domain contracts and the implemented drawing Phase 2 through Phase 5.1 foundation.

Do not move them mechanically. Each move needs link updates, validation, and a
clear record of any implementation drift.

## Operating Rule

The current code-facing answer is:

1. Treat SDF-first Phase 6D as complete unless validation regresses.
2. Resume `WR-001` post-Phase 6D product-job and Draw cache follow-up first.
3. Do only directly supporting `WR-002`, `WR-003`, `WR-004`, and `WR-005`
   work in parallel.
4. After that stabilizes, choose between later Draw DRF work and net Phase 1
   hardening using gate state, dependency level, lane, and score.
5. Keep gameplay graph, particles, physics, animation, world processes, alternate UI execution, and deferred SDF capability detail drafts behind their owning contract gates.

After completing any phased implementation, run the phase completion drift-check
routine before starting the next phase.
