---
title: Roadmap Deferred Register
description: Blocked or policy-deferred WR roadmap rows archived out of active execution.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-25
related:
  - ./roadmap-items.yaml
  - ./roadmap-archive.yaml
  - ./roadmap-deferred.yaml
  - ./roadmap-decision-register.md
---

# Roadmap Deferred Register

Blocked or policy-deferred WR roadmap rows archived out of active execution.

| ID | Track | Lane | Planning state | Completion quality | Dependency level | Gate | V | B | Score | Current decision | Evidence / blocker |
|---|---|---|---|---|---:|---|---:|---:|---:|---|---|
| WR-011 | Gameplay Graph ATR IR and ECS lowering | Contract | blocked_deferred | not_applicable | L2 | Domain contract gate | 4 | 4 | 0.3 | Blocked until narrower gameplay contracts exist. | Missing domain/gameplay/events, domain/gameplay/actions, domain/gameplay/state, and domain/gameplay/quests; SDF physics relation readiness and authority diagnostics also need owning contracts. |
| WR-013 | Scripting and runtime gameplay bridge | Contract | blocked_deferred | not_applicable | L3 | Domain contract gate | 3 | 4 | 0.3 | Rhai can be first adapter only after the domain contract exists. | It depends on M6 formed procedural/gameplay product contracts and a language-neutral runtime boundary. Rhai can be the first adapter only after the domain contract exists. |
| WR-014 | Particles, physics, animation, and world-process product tracks | Contract | blocked_deferred | not_applicable | L3 | Product contract gate | 3 | 4 | 0.2 | Follow product-job, query snapshot, and publication substrate maturity. | Their domain docs and formed product contracts are still missing or deferred. They follow the product-job/query snapshot/publication barrier substrate. |
| WR-015 | SDF character, vegetation, atmosphere, water, VFX, and influence AI drafts | Deferred | blocked_deferred | not_applicable | L4 | Policy deferred | 2 | 5 | 0.2 | Keep remaining deferred drafts until product ownership and handoff contracts are promoted. | These remaining detail drafts are explicitly deferred. Reactivate only after the relevant product ownership, renderer/runtime handoff, query policy, and authority contracts are promoted. |
| WR-016 | Compiled-reactive UI and ECS-driven UI execution strategies | Deferred | blocked_deferred | not_applicable | L4 | Policy deferred | 2 | 5 | 0.1 | Keep deferred; retained UI remains the active implementation path and Interaction V2 is a contract layer for retained UI first, not permission to implement alternate execution. | The current retained UI path is the active implementation. Alternative execution strategies require a new active design or accepted ADR before code, and must consume normalized definitions plus formed interaction contracts. |
| WR-017 | Gameplay actions, powers, runtime, and power editor | Deferred | blocked_deferred | not_applicable | L4 | Policy deferred | 2 | 5 | 0.2 | Keep deferred behind gameplay contract work. | These deferred designs still need remaining decisions and must not precede the narrower gameplay graph and domain contract sequence. |
| WR-079 | Renderer Cross Track Evidence Matrix | Discovery | blocked_deferred | not_applicable | L4 | Policy deferred pending intake approval | 2 | 5 | 0.2 | Intake proposal only; do not implement until applied and promoted by roadmap review. | New idea has not passed roadmap intake, architecture governance, and dependency review. |
| WR-080 | Renderer Gap Closure And Consistency Audit | Discovery | blocked_deferred | not_applicable | L4 | Policy deferred pending intake approval | 2 | 5 | 0.2 | Intake proposal only; do not implement until applied and promoted by roadmap review. | New idea has not passed roadmap intake, architecture governance, and dependency review. |
| WR-081 | Renderer Perfectionist Verification Closeout | Discovery | blocked_deferred | not_applicable | L4 | Policy deferred pending intake approval | 2 | 5 | 0.2 | Intake proposal only; do not implement until applied and promoted by roadmap review. | New idea has not passed roadmap intake, architecture governance, and dependency review. |
| WR-104 | Game Runtime UI Projection Governance And Track Activation | Product planning | blocked_deferred | not_applicable | L4 | Governance before game UI implementation | 4 | 4 | 1.3 | Intake proposal only; keep as blocked_deferred until accepted and explicitly promoted. | New game-runtime UI product track has not passed roadmap intake, architecture governance closeout, or dependency review. |
| WR-106 | Viewport And Procedural Projection Contract Consolidation | Discovery | blocked_deferred | not_applicable | L4 | Policy deferred pending intake approval | 2 | 5 | 0.2 | Deferred intake proposal only. Preserve producer-owned camera truth, camera-free PreparedViewFrame, and camera-free UI primitives while deciding the long-term contract split. | New cross-boundary idea has not passed intake approval, accepted design review, ADR triage, and follow-on WR decomposition. |
| WR-112 | Native Editor UX Story Lab And Evidence Harness | Editor quality | blocked_deferred | not_applicable | L4 | After editor UX governance | 5 | 4 | 1.2 | Deferred follow-on row for PM-EDITOR-UX-002. Do not implement until WR-112 has a fresh production plan and exact write scopes. | Follow-on product code is blocked until PM001 governance is consumed and WR-112 is promoted with a bounded implementation contract. |
| WR-113 | Layered Editor Design System Migration | Editor quality | blocked_deferred | not_applicable | L4 | After native Story Lab | 4 | 4 | 1.7 | Deferred follow-on row for PM-EDITOR-UX-003. | Design-system migration without Story Lab coverage would repeat app-only polish. |
| WR-114 | Standalone UI Designer Workbench | Editor quality | blocked_deferred | not_applicable | L4 | After layered design system migration | 5 | 4 | 1.2 | Deferred follow-on row for PM-EDITOR-UX-004. | UI Designer workbench must wait for proof and design-system substrate. |
| WR-115 | Graph Canvas And Node Editor Productization | Editor quality | blocked_deferred | not_applicable | L4 | After Story Lab and design-system layers | 5 | 4 | 1.1 | Deferred follow-on row for PM-EDITOR-UX-005. | Registered graph surfaces need proof policy before product code changes. |
| WR-116 | Shell And Product Pattern Polish | Editor quality | blocked_deferred | not_applicable | L4 | After Designer and graph productization | 5 | 4 | 1.1 | Deferred follow-on row for PM-EDITOR-UX-006. | Shell polish needs reusable patterns and evidence harness first. |
| WR-117 | All Registered Visible Surface Wave | Editor quality | blocked_deferred | not_applicable | L4 | After product patterns | 5 | 4 | 1.2 | Deferred follow-on row for PM-EDITOR-UX-007. | All-surface certification must not start before Story Lab and major product patterns can certify surfaces. |
| WR-118 | Game UI Readiness Seam | Editor quality | blocked_deferred | not_applicable | L4 | After all-surface wave | 4 | 4 | 1.7 | Deferred follow-on row for PM-EDITOR-UX-008. | Game UI readiness must wait until editor surfaces are certified and ownership seams are enforceable. |
| WR-119 | Final Local Native Editor UX No Gap Certification | Editor quality | blocked_deferred | not_applicable | L4 | After game UI readiness seam | 5 | 4 | 1.2 | Deferred follow-on row for PM-EDITOR-UX-009. | Final certification is illegal before completed native Story Lab, design system, UI Designer, graph, shell, all-surface, and game-readiness evidence. |
