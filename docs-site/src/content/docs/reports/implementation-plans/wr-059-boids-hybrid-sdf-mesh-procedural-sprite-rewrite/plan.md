---
title: WR-059 Boids Hybrid SDF/Mesh Procedural Sprite Rewrite Implementation Contract
description: Design-first implementation contract for rewriting the canonical boids example through the renderer procedural API.
status: active
owner: engine
layer: engine-runtime / renderer examples
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-059 Boids Hybrid SDF/Mesh Procedural Sprite Rewrite Implementation Contract

## Goal

Define the bounded implementation contract for `PM-RENDER-GPU-005` and
`WR-059`. The slice rewrites the canonical boids example so users copy the
public procedural instance API added by `WR-058`, not fullscreen-per-instance
or fragment-loop-over-all-boids patterns.

This is a design-first contract. It records readiness, ownership, and the exact
rewrite decisions only. Product code changes remain unauthorized until the
stack or single-track coordinator selects the WR for implementation.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-GPU-005`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml::WR-059`.
- Accepted GPU evidence and procedural visuals doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`.
- Procedural API closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md`.

`task production:plan -- --milestone PM-RENDER-GPU-005 --roadmap WR-059`
reported:

- milestone state: `designing`;
- WR state: `blocked_deferred`;
- dependency: `WR-058:completed`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/plan.md`.

## Readiness

The original blockers are cleared:

- the renderer GPU evidence and procedural visuals doctrine is accepted;
- `WR-057` completed pass-shape guards;
- `WR-058` completed the public procedural instance API;
- architecture-governance kickoff was run for this bounded scope.

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Plan canonical boids hybrid procedural sprite rewrite for WR-059" --scope "engine/examples/boids_render_flow assets/shaders/boids_compose.wgsl assets/shaders/boids_compute.wgsl docs-site/src/content/docs/engine/reference/plugins/render docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md"
```

Governance conclusion for this contract:

- bounded context owner: `engine/examples/boids_render_flow` as a canonical
  renderer example consuming `engine/src/plugins/render` public APIs;
- no ADR is required while the rewrite remains an example and shader-asset
  consumer of renderer public APIs;
- stop for ADR/design update if boids needs renderer-private handles, changes
  renderer API ownership, introduces product truth/freshness policy, or turns
  boids-specific behavior into renderer core policy.

## Promotion Readiness

The design-first action promotes `WR-059` from deferred planning to
`ready_next` only. It does not authorize code implementation until the stack or
single-track coordinator selects the row for execution.

Promotion evidence:

- accepted renderer GPU evidence and procedural visuals design;
- completed `WR-058` procedural instance API closeout;
- this implementation contract;
- architecture-governance kickoff for the bounded boids example and shader
  scope.

Resolved planning questions:

- Runtime smoke uses the existing default boids configuration for the example;
  broader duration, stress-count, and cross-machine benchmark targets remain
  `WR-060` production evidence.
- The canonical visual defaults to a local 2D SDF impostor through
  `ProceduralPassDescriptor::local_sdf_2d_impostors(...)`. Mesh or quad sprite
  comparison paths are out of scope unless they are incidental tests of the
  same public API and do not replace the canonical proof.
- `WR-059` has no fixed GPU timing failure threshold because local adapters and
  timestamp-query support vary. The row fails on missing runtime timing or
  explicit unsupported diagnostics, missing compute/render/present evidence,
  any fullscreen-per-boid draw shape, or any fragment loop over all boids.

`task production:plan -- --milestone PM-RENDER-GPU-005 --roadmap WR-059`
reported after the row moved to `ready_next`:

- milestone state: `active`;
- WR state: `ready_next`;
- blocker: `B2`;
- dependency: `WR-058:completed`;
- promotion preflight: `promotable`.

Accepted promotion evidence:

```text
Accepted renderer GPU evidence design, completed WR-058 procedural API closeout, architecture-governance kickoff, and WR-059 implementation contract at docs-site/src/content/docs/reports/implementation-plans/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/plan.md.
```

The row may be promoted when the stack or single-track coordinator selects the
promotion action with:

```text
task roadmap:promote -- --id WR-059 --state current_candidate --evidence "Accepted renderer GPU evidence design, completed WR-058 procedural API closeout, architecture-governance kickoff, and WR-059 implementation contract at docs-site/src/content/docs/reports/implementation-plans/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/plan.md."
```

Implementation may still start only when the stack or single-track coordinator
selects `execute_next_wr_implementation_contract`.

## Promotion History

The row was promoted after the promotion preflight reported `promotable`:

```text
task roadmap:promote -- --id WR-059 --state current_candidate --evidence "Accepted renderer GPU evidence design, completed WR-058 procedural API closeout, architecture-governance kickoff, and WR-059 implementation contract at docs-site/src/content/docs/reports/implementation-plans/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/plan.md."
```

Promotion result:

- `WR-059` planning state: `current_candidate`;
- blocker: `B2`;
- dependency: `WR-058:completed`;
- roadmap, production, docs, and planning validation passed after promotion.

`task production:plan -- --milestone PM-RENDER-GPU-005 --roadmap WR-059`
reported after promotion:

- milestone state: `active`;
- WR state: `current_candidate`;
- next action: `write_implementation_contract`;
- current candidate is eligible for a bounded implementation slice.

This section completes the current-candidate implementation-readiness contract.
It does not expand write scopes or change the closeout requirements below.

## Ownership And Boundaries

Renderer example owns:

- the boids render-flow declaration;
- the boids compute and compose shader assets;
- example-local runtime smoke and tests proving the public procedural API usage.

Renderer core owns:

- the public procedural API and pass-shape guards consumed by the example;
- GPU timing and runtime inspection DTOs.

Not owned by this row:

- new renderer API design beyond consuming `WR-058`;
- gameplay/VFX product truth;
- product freshness, authority, fallback, rebuild, or residency policy;
- production benchmarking and final runtime_proven evidence beyond this boids
  proof row.

## Critical Review Decisions

Current boids risk:

- The existing compose shader draws fullscreen generated geometry with an
  instance count and loops over the full boid storage array in the fragment
  shader.
- The current flow also keeps a history copy even though no trail/history effect
  consumes it.
- `WR-057` pass-shape metadata cannot prove shader-source loop behavior, so the
  canonical example must remove that pattern explicitly.

Implementation decisions for the future code slice:

- Boids must use `RenderFlow::procedural_pass(...)` with
  `ProceduralPassDescriptor::local_sdf_2d_impostors(...)` as the canonical
  default path. A mesh or quad sprite comparison path is allowed only as
  incidental public-API coverage and must not become the accepted proof.
- Compute simulation remains storage-backed and continues to write the double
  buffered boid state.
- Rendering consumes the current boid storage buffer as an instance buffer with
  explicit `RenderVertexBufferLayout` attributes for position and velocity.
- The vertex stage must place local per-boid quad/impostor geometry. The
  fragment stage may shade one local boid sprite/impostor, but it must not loop
  over all boids.
- History copy must be removed unless a real trail/history effect consumes it in
  the same row. If trails are added, the history read/write must be explicit and
  covered by tests and docs.
- Runtime evidence should report compute, draw, and present timing. GPU timing
  may be measured when supported or reported through explicit unsupported
  diagnostics when not.
- Numeric GPU timing budgets, stress thresholds, and production benchmark
  acceptance belong to `WR-060`, not this canonical rewrite row.

## Implementation Scope

Allowed write scopes after roadmap promotion:

```text
engine/examples/boids_render_flow
assets/shaders/boids_compose.wgsl
assets/shaders/boids_compute.wgsl
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-boids-hybrid-sdf-mesh-procedural-sprite-rewrite
docs-site/src/content/docs/reports/implementation-plans/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/plan.md
docs-site/src/content/docs/reports/closeouts/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected owning files:

- `engine/examples/boids_render_flow/rendering/graph.rs`: consume
  `RenderFlow::procedural_pass(...)`, remove direct ad hoc graphics-pass shape,
  and remove the history copy unless a real trail effect is implemented.
- `assets/shaders/boids_compose.wgsl`: move from fullscreen fragment iteration
  to local per-boid vertex/impostor shading.
- `assets/shaders/boids_compute.wgsl`: preserve compute simulation ownership;
  change only if storage layout or current-buffer selection requires it.
- renderer public docs: show boids as the canonical procedural API consumer.

## Acceptance Criteria

- Boids rendering uses the public procedural API from `WR-058`.
- No fullscreen draw is multiplied by boid count.
- The compose fragment shader no longer loops over every boid.
- Compute simulation remains storage-backed.
- Runtime/inspection evidence can identify compute, draw, and present passes
  and GPU timing status.
- The example does not keep history copies unless a real trail/history effect
  consumes them.

## Required Validation

Implementation validation must include:

```text
cargo test -p engine --example boids_render_flow
cargo run -p engine --example boids_render_flow
cargo test -p engine render_runtime_inspect
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

Focused tests must prove:

- the boids flow declares compute, procedural draw, and present passes;
- the draw pass has local instance geometry through the procedural API;
- no history copy remains unless tests prove a consuming trail/history effect;
- the shader no longer contains the fullscreen fragment loop over all boids;
- runtime inspection/timing DTOs remain backend-handle-free.

## Stop Conditions

Stop before implementation if:

- the rewrite requires renderer-private handles or changes the `WR-058` public
  API contract;
- the shader cannot be rewritten without reintroducing a fragment loop over all
  boids;
- runtime smoke evidence cannot distinguish measured GPU timing from explicit
  unsupported diagnostics;
- a trail/history effect is introduced without a real consumer and tests;
- product truth, freshness, fallback, rebuild, or residency policy becomes part
  of the example.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/closeout.md
```

The closeout must record changed files, runtime smoke evidence, validation
output, pass-shape proof, shader-loop removal evidence, and remaining quality
gaps for `WR-060`.

## Perfectionist Closeout Audit

Expected completion quality for this row is `runtime_proven` if the example run
captures runtime timing or explicit unsupported GPU timing diagnostics. If the
runtime smoke cannot run in the local environment, completion must stay
`bounded_contract` with that gap recorded.

`perfectionist_verified` is not available for this row because final production
evidence and renderer-stack audit work remain in later milestones.
