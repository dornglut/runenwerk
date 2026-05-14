---
title: Design Implementation Triage
description: Current workspace triage of active and deferred design work by implementation readiness, blocker weight, and execution priority.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related:
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

The current readiness and blocker map is:

- [design-implementation-readiness-map.puml](diagrams/design-implementation-readiness-map.puml)

There is also an older accepted SDF capability dependency diagram:

- [runenwerk-design-priority-map.puml](../design/accepted/diagrams/runenwerk-design-priority-map.puml)

That existing diagram is still useful as a long-term capability dependency map,
but it does not encode current implementation readiness or current blockers.

## Weighting Model

Priority weight:

| Weight | Meaning |
|---|---|
| P0 | Current focus. Start or continue now if selecting code work. |
| P1 | Ready next after the P0 path is stable. |
| P2 | Design-gated or medium-horizon work. |
| P3 | Deferred or long-horizon capability work. |

Blocker weight:

| Weight | Meaning |
|---|---|
| B1 | No material architecture blocker. Needs scoped implementation and validation. |
| B2 | Partial blocker. Some substrate exists, but one bounded contract or proof is still needed. |
| B3 | Product or runtime blocker. Needs another roadmap slice first. |
| B4 | Domain contract blocker. Missing owning domain contracts or formed-product path. |
| B5 | Deferred by policy. Do not implement until promoted by an active design, accepted design, ADR, or roadmap update. |

## Implement Now

| Track | Priority | Blocker | Current call | First implementation move |
|---|---:|---:|---|---|
| SDF-first Phase 6D / M6.2C procgen bake, rollback, persistence, and runtime preview reload classification | P0 | B1 | Implement now. Phase 6A, 6B, and 6C are complete, and this is the current cross-repo focus. | Continue from [sdf-first-execution-roadmap.md](./sdf-first-execution-roadmap.md) and keep work inside `domain/procgen`, `apps/runenwerk_editor`, and the existing product publication, query snapshot, render selection, and derived residency path. |
| ECS/runtime convergence support for product jobs and diagnostics | P0 | B2 | Implement as supporting work only when it directly unblocks the SDF-first execution path. | Close remaining lifecycle finalization, deterministic registration/plan reporting, and consumer lag/backpressure diagnostics from [net/ecs-runtime-prioritized-roadmap.md](../net/ecs-runtime-prioritized-roadmap.md). |
| Render immediate follow-ups through product-selection and derived-residency contracts | P0 | B2 | Implement only as contract-following follow-up, not as renderer-owned world truth. | Audit current R4/R6/R7 code truth first, then use [engine/plugins/render/docs/roadmap.md](../engine/plugins/render/docs/roadmap.md) and [render-product-surface-foundation-bundle-design.md](../design/active/render-product-surface-foundation-bundle-design.md). |
| UI/editor guard and sequencing maintenance | P0 | B1 | Keep active while any editor surface work lands. | Preserve structural routing, capability gating, and surface ownership guard coverage from [domain/ui/roadmap.md](../domain/ui/roadmap.md). |
| Design lifecycle cleanup for implemented active designs | P1 | B1 | Can be done now as documentation work. It reduces drift but should not interrupt P0 code if code capacity is scarce. | Review active designs that already say implemented, then promote or move them according to [design/README.md](../design/README.md). |

## Ready Next

| Track | Priority | Blocker | Current call | Main blocker |
|---|---:|---:|---|---|
| Runenwerk Draw rendering foundation DRF2 through DRF5 | P1 | B2 | Ready next after the current SDF-first Now work is stable. | DRF2 app-derived cache is only partially complete; DRF3 to DRF5 must keep CPU tile formation canonical and use public render-flow/product-surface APIs. |
| Multiplayer replication Phase 1 to Phase 3 | P1 | B2 | Ready next for net hardening. | ACK/baseline hardening and delta lifecycle rules need tests and code evidence before broader declarative replication. |
| Native tablet input backend arbitration and diagnostics | P1 | B3 | Code work can continue, but product acceptance remains blocked. | Hardware validation is still required for Windows Ink, Wacom Wintab, and macOS Wacom devices. |
| Native multi-window editor presentation | P2 | B3 | Design is active, but it should not preempt current product-surface and SDF-first work. | Runtime window state and render surface handling are still singleton-shaped; second-window productization needs window-scoped runtime, input, UI frame, and swapchain ownership. |
| Render fragment/data-driven maturity R10 | P2 | B3 | Keep queued after render surface, ergonomics, persistence, and inspection follow-ups. | Fragment assets and hot reload need stable target aliases, prepared flow invocations, diagnostics, and a product priority. |

## Blocked Or Deferred

| Track | Priority | Blocker | Why it is not ready now |
|---|---:|---:|---|
| Gameplay Graph ATR IR and ECS lowering | P2 | B4 | Missing `domain/gameplay/events`, `domain/gameplay/actions`, `domain/gameplay/state`, and `domain/gameplay/quests`; SDF physics relation readiness and authority diagnostics also need owning contracts. |
| General semantic graph implementation | P2 | B4 | The policy is active, but implementation must start from one concrete owning domain and one formed product target, not a broad graph platform. |
| Scripting and runtime gameplay bridge | P2 | B4 | It depends on M6 formed procedural/gameplay product contracts and a language-neutral runtime boundary. Rhai can be the first adapter only after the domain contract exists. |
| Particles, physics, animation, and world-process product tracks | P3 | B4 | Their domain docs and formed product contracts are still missing or deferred. They follow the product-job/query snapshot/publication barrier substrate. |
| SDF prefab, character animation, vegetation, atmosphere, water/wetness, influence AI, VFX, and larger world-process drafts | P3 | B5 | These are explicitly deferred detail drafts. Reactivate only after the relevant product ownership, renderer/runtime handoff, query policy, and authority contracts are promoted. |
| Compiled-reactive UI and ECS-driven UI execution strategies | P3 | B5 | The current retained UI path is the active implementation. Alternative execution strategies require a new active design or accepted ADR before code. |
| Gameplay actions, powers, power runtime, action runtime, and power editor | P3 | B5 | These deferred designs still need remaining decisions and must not precede the narrower gameplay graph and domain contract sequence. |

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

1. Do SDF-first Phase 6D first.
2. Do only directly supporting ECS/runtime, render, UI guard, and docs alignment work in parallel.
3. After that stabilizes, choose between Draw DRF follow-up and net Phase 1 hardening based on product need.
4. Keep gameplay graph, particles, physics, animation, world processes, alternate UI execution, and deferred SDF capability detail drafts behind their owning contract gates.

After completing any phased implementation, run the phase completion drift-check
routine before starting the next phase.
