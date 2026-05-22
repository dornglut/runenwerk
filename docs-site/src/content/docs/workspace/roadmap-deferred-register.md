---
title: Roadmap Deferred Register
description: Blocked or policy-deferred WR roadmap rows archived out of active execution.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-22
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
| WR-056 | Renderer GPU Pass Timing And Workload Evidence | Render platform | blocked_deferred | not_applicable | L2 | Accepted GPU evidence design and architecture governance | 4 | 4 | 0.9 | Deferred foundation row for GPU pass timing and workload evidence; do not implement until the active design is accepted and this row is promoted. | GPU timing changes submit-adjacent renderer evidence policy and requires accepted design plus architecture governance before code. |
| WR-057 | Render Flow Pass Shape And Instance Contract Guards | Render platform | blocked_deferred | not_applicable | L2 | Accepted GPU evidence design and GPU timing foundation | 4 | 4 | 0.9 | Deferred render-flow validation row for pass-shape and instance-count hazards. | Pass-shape enforcement needs the accepted evidence doctrine and timing foundation before implementation. |
| WR-058 | Hybrid Mesh/SDF Procedural Instance Rendering API | Render platform | blocked_deferred | not_applicable | L3 | Accepted GPU evidence design and pass-shape guards | 4 | 4 | 0.6 | Deferred API row for mesh/quad sprites, local 2D SDF impostors, shared instance buffers, and explicit render-state policy. | Procedural APIs need pass-shape guard doctrine and accepted ownership boundaries before implementation. |
| WR-059 | Boids Hybrid SDF/Mesh Procedural Sprite Rewrite | Render example | blocked_deferred | not_applicable | L3 | Procedural instance API and GPU timing evidence | 3 | 4 | 1.0 | Deferred canonical example rewrite; do not patch boids locally before the procedural API and guard rows exist. | The canonical boids rewrite must consume the new API instead of inventing a temporary example-only path. |
| WR-060 | Renderer Procedural Visuals Production Evidence | Render hardening | blocked_deferred | not_applicable | L3 | Completed GPU timing, guards, procedural API, and boids proof | 4 | 4 | 0.9 | Deferred production hardening row for docs, benchmarks, examples, runtime evidence, and closeout. | Production evidence cannot exist until the foundation, guards, API, and canonical boids proof are complete. |
