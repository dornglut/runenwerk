---
title: WR-058 Hybrid Mesh/SDF Procedural Instance Rendering API Implementation Contract
description: Design-first implementation contract for renderer-owned procedural mesh sprite and local 2D SDF impostor APIs.
status: active
owner: engine
layer: engine-runtime / renderer public API
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-058 Hybrid Mesh/SDF Procedural Instance Rendering API Implementation Contract

## Goal

Define the bounded implementation contract for `PM-RENDER-GPU-004` and
`WR-058`. The slice adds discoverable renderer-owned procedural instance APIs
for mesh/quad sprites and local 2D SDF impostors. The API must produce normal
render-flow graphics passes with explicit buffers, explicit render-state
policy, and pass-shape-safe local geometry.

This is a design-first contract. It records readiness, ownership, and API
decisions only. Product code changes remain unauthorized until the stack or
single-track coordinator selects the WR for implementation.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-GPU-004`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml::WR-058`.
- Accepted GPU evidence and procedural visuals doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`.
- Accepted product/render ownership boundary:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- Accepted SDF product and GPU residency boundary:
  `docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md`.
- Pass-shape dependency closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-057-render-flow-pass-shape-and-instance-contract-guards/closeout.md`.

`task production:plan -- --milestone PM-RENDER-GPU-004 --roadmap WR-058`
reported:

- milestone state: `designing`;
- WR state: `blocked_deferred`;
- dependency: `WR-057:completed`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/plan.md`.

## Readiness

The original blockers are cleared:

- the renderer GPU evidence and procedural visuals doctrine is accepted;
- the render product graph platform design is accepted;
- the SDF product renderer and GPU residency design is accepted;
- `WR-057` completed the renderer pass-shape guard dependency;
- architecture-governance kickoff was run for this bounded scope.

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Plan hybrid mesh and local SDF procedural instance rendering API for WR-058" --scope "engine/src/plugins/render engine/examples engine/tests docs-site/src/content/docs/engine/reference/plugins/render docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md"
```

Governance conclusion for this contract:

- bounded context owner: `engine/src/plugins/render` public procedural
  execution APIs and derived GPU resources;
- no ADR is required while the API remains renderer-owned execution and
  descriptor help over existing render-flow contracts;
- stop for ADR/design update if implementation changes durable render-flow
  ownership, dependency direction, persisted cross-domain ABI, or makes
  renderer procedural APIs authoritative for product truth, freshness,
  fallback, rebuild, or residency policy.

## Promotion History

`task production:plan -- --milestone PM-RENDER-GPU-004 --roadmap WR-058`
reported after the row moved to `ready_next`:

- milestone state: `active`;
- WR state: `ready_next`;
- blocker: `B2`;
- dependency: `WR-057:completed`;
- promotion preflight: `promotable`.

Promotion evidence:

```text
Accepted renderer GPU evidence design, accepted product/render and SDF boundary designs, completed WR-057 pass-shape guard closeout, architecture-governance kickoff, and WR-058 promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/plan.md.
```

The row was promoted with that evidence:

```text
task roadmap:promote -- --id WR-058 --state current_candidate --evidence "Accepted renderer GPU evidence design, accepted product/render and SDF boundary designs, completed WR-057 pass-shape guard closeout, architecture-governance kickoff, and WR-058 promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/plan.md."
```

Implementation may still start only when the stack or single-track coordinator
selects `execute_next_wr_implementation_contract`.

## Ownership And Boundaries

Renderer-owned:

- public procedural instance descriptors and builder helpers;
- derived render-flow pass construction for mesh sprites, quad sprites, and
  local 2D SDF impostors;
- explicit render-state policy types for primitive, blend, depth, cull, and
  target handling;
- validation diagnostics for missing buffers, incompatible layouts, ambiguous
  shapes, unsupported policy combinations, and pass-shape guard integration;
- examples and docs that show normal procedural usage without renderer-private
  backend handles.

Not renderer-owned:

- gameplay, particle, VFX, material, model, or product truth;
- source-product freshness, authority, fallback legality, rebuild policy, or
  residency policy;
- 3D SDF raymarch, sparse SDF residency, or product-owned SDF semantics;
- boids-specific shortcuts.

## Critical Review Decisions

Source truth:

- Product domains own semantic truth and decide what visual work to submit.
- `RenderFlow`, storage array handles, vertex/instance layouts, draw
  descriptors, and pass-shape diagnostics remain source truth for renderer
  execution shape.
- Procedural descriptors are a public renderer API for deriving render-flow
  graphics passes; they are not product entities.

Resolved design questions:

- The first public SDF impostor API is local 2D only. It may describe a
  screen-facing or locally transformed 2D impostor shape and shader payload,
  but it must not expose 3D SDF raymarch, sparse brick residency, or product
  SDF authority hooks.
- v1 render-state policy is explicit through typed values for primitive
  topology, blend mode, depth policy, cull mode, target output, local mesh or
  generated quad primitive, and instance buffer layout.
- Normal usage must produce pass-shape-safe graphics passes with local vertex
  and/or instance buffers. Authors should not need
  `GraphicsPassBuilder::allow_instanced_fullscreen(...)` for ordinary mesh or
  impostor sprites.
- Shared instance buffers use existing `StorageArrayHandle<T>` and
  `RenderVertexBufferLayout` contracts. The API may wrap those contracts for
  ergonomics, but it must preserve typed handles and explicit shader locations.
- Backend `wgpu` handles stay private. Public APIs expose descriptors and
  validation results, not mutable runtime resources.

Expected source-to-runtime chain:

1. `engine/src/plugins/render/procedural/mod.rs` owns the procedural API
   boundary, keeping descriptor and validation modules under a focused
   subdomain folder.
2. `engine/src/plugins/render/procedural/descriptors.rs` defines mesh sprite,
   quad sprite, local 2D SDF impostor, instance-buffer, target, and render-state
   policy descriptors.
3. `engine/src/plugins/render/procedural/validation.rs` validates descriptor
   completeness, buffer layout compatibility, supported local SDF scope, and
   pass-shape safety before a pass is added to a flow.
4. `engine/src/plugins/render/procedural/builders.rs` or a similarly focused
   module translates validated descriptors into existing
   `RenderFlow::graphics_pass(...)`, `StorageArrayHandle<T>`,
   `RenderVertexBufferLayout`, and `RenderDrawDescriptor` calls.
5. `engine/src/plugins/render/api/flow.rs` and
   `engine/src/plugins/render/api/passes.rs` expose only the small public
   extension points needed to keep the procedural path discoverable.
6. `engine/src/plugins/render/graph/pass_shape.rs` remains the enforcement
   owner for dangerous pass shapes; procedural validation must satisfy that
   guard instead of bypassing it.
7. `engine/src/plugins/render/renderer/render_flow/execute_passes.rs` may gain
   render-state execution support only for policies carried through compiled
   render-flow plans.

Typed contracts replace ad hoc strings for:

- procedural primitive kind;
- local 2D SDF impostor kind;
- blend, depth, cull, primitive topology, and target policy;
- instance layout declaration and draw count;
- validation diagnostic kind.

Forbidden fallbacks:

- Do not infer product semantics from shader names, labels, or resource labels.
- Do not infer freshness, authority, fallback legality, rebuild, or residency
  policy inside renderer procedural APIs.
- Do not make a fullscreen pass multiplied by instance count the normal path.
- Do not expose renderer-private backend resources as public handles.
- Do not implement 3D SDF or raymarch hooks in this row.

## Implementation Scope

Allowed write scopes after roadmap promotion:

```text
engine/src/plugins/render
engine/examples
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-hybrid-mesh-sdf-procedural-instance-rendering-api
docs-site/src/content/docs/reports/implementation-plans/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/plan.md
docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected owning modules:

- `engine/src/plugins/render/procedural/mod.rs`: public procedural subsystem
  boundary and re-exports.
- `engine/src/plugins/render/procedural/descriptors.rs`: typed descriptors for
  mesh sprites, quad sprites, local 2D SDF impostors, instance buffers, and
  render-state policy.
- `engine/src/plugins/render/procedural/validation.rs`: descriptor validation
  and typed diagnostics.
- `engine/src/plugins/render/procedural/builders.rs`: translation from
  validated descriptors into existing render-flow builder calls.
- `engine/src/plugins/render/api/flow.rs` and
  `engine/src/plugins/render/api/passes.rs`: narrow discoverability hooks only
  where the existing public builder surface needs them.
- `engine/src/plugins/render/graph/pass_graph.rs`,
  `engine/src/plugins/render/graph/execution_plan.rs`, and
  `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`: compiled
  render-state transport and backend execution if explicit policy support
  requires it.
- `engine/src/plugins/render/graph/diagnostics.rs` and
  `engine/src/plugins/render/inspect`: public diagnostic/readiness projection
  when procedural validation needs inspectable summaries.
- `engine/tests/procedural_instance.rs` or focused tests under an existing
  render test module: descriptor validation, flow generation, pass-shape guard
  compatibility, and inspection coverage.
- `engine/examples`: one minimal procedural example if API discoverability
  cannot be proven by tests and docs alone.

## Acceptance Criteria

- Public renderer docs and examples show how to build mesh sprites, quad
  sprites, and local 2D SDF impostors through the procedural API.
- Shared storage-backed instance buffers are accepted through typed handles and
  explicit vertex/instance layouts.
- Blend, depth, cull, primitive topology, and target policy are explicit and
  validated.
- Generated render-flow passes use local geometry or explicit instance buffers
  so the `WR-057` pass-shape guard remains satisfied by default.
- Validation rejects missing instance buffers, incompatible layout/attribute
  declarations, unsupported local SDF scope, ambiguous pass shapes, and
  unsupported render-state policy combinations.
- The API derives renderer execution resources only and does not infer product
  semantics, freshness, authority, fallback, rebuild, or residency policy.
- Backend handles remain private.

## Implementation Steps

Implement the slice in this order once the coordinator allows product code:

1. Add `engine/src/plugins/render/procedural/` with a `mod.rs` boundary and
   focused descriptor, validation, and builder modules. Re-export the public
   surface through `engine/src/plugins/render/mod.rs` via the existing render
   module export pattern.
2. Define typed descriptors for mesh sprites, generated quad sprites, local 2D
   SDF impostors, instance buffer declarations, target policy, primitive
   topology, blend policy, depth policy, and cull policy.
3. Add procedural validation diagnostics that reject missing instance buffers,
   empty or mismatched layouts, unsupported local SDF scope, ambiguous pass
   shape, and unsupported render-state combinations before a render-flow pass is
   emitted.
4. Add a public builder path that translates validated procedural descriptors
   into `RenderFlow::graphics_pass(...)` with `vertex_buffer(...)`,
   `instance_buffer(...)`, `draw(...)`, color/depth targets, and dependency
   wiring. Generated quad paths must still declare local geometry or a typed
   generated-quad primitive that the pass-shape guard can distinguish from
   fullscreen multiplied work.
5. Extend compiled render-flow execution only for explicit render-state policy
   that cannot be represented by current defaults. Preserve existing defaults
   for old graphics passes unless the new procedural descriptor opts into a
   policy.
6. Add focused `procedural_instance` tests for descriptor validation, generated
   render-flow shape, pass-shape guard compatibility, local 2D SDF scope, and
   backend-handle-free inspection/readiness projection.
7. Update renderer public API docs and usage guides so normal procedural usage
   is discoverable without relying on boids or renderer-private handles.

Architecture guard tests must prevent:

- descriptor-only APIs that never emit a render-flow pass;
- render-flow-only APIs that bypass typed procedural validation;
- fullscreen-per-instance generated work as the normal sprite/impostor path;
- status-panel-only or inspection-only evidence without execution contracts;
- backend-resource handles leaking into public descriptors.

## Required Validation

Implementation validation must include:

```text
cargo test -p engine render_flow
cargo test -p engine procedural_instance
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

- mesh/quad sprite descriptors compile into graphics passes with local geometry;
- local 2D SDF impostor descriptors are accepted while 3D SDF hooks are
  rejected or absent;
- explicit blend/depth/cull/primitive policies survive validation and, when
  implemented, compiled execution;
- shared `StorageArrayHandle<T>` instance buffers bind with the declared
  `RenderVertexBufferLayout`;
- pass-shape guard diagnostics are not triggered for normal procedural paths
  and still trigger for unsafe fullscreen-instanced work;
- inspection/readiness DTOs remain backend-handle-free.

## Stop Conditions

Stop before implementation if:

- the desired API requires renderer ownership of product truth, source
  freshness, fallback legality, rebuild policy, or residency policy;
- local 2D SDF impostors cannot be separated from 3D SDF raymarch or sparse
  residency decisions;
- a public API change would reject a documented supported render-flow shape
  without accepted design or ADR evidence;
- pass-shape safety depends on shader-source inspection or label conventions;
- validation would rely only on the boids example instead of reusable renderer
  API tests.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md
```

The closeout must record:

- exact changed modules and public APIs;
- examples/docs that demonstrate mesh sprites, quad sprites, and local 2D SDF
  impostors;
- validation output for `cargo test -p engine render_flow`,
  `cargo test -p engine procedural_instance`,
  `cargo test -p engine render_runtime_inspect`, docs, roadmap, production, and
  planning gates;
- evidence that normal procedural passes satisfy the `WR-057` pass-shape guard;
- confirmation that renderer APIs derive execution resources only and do not
  own product truth, freshness, authority, fallback, rebuild, or residency
  policy;
- remaining quality gaps that must stay visible for `WR-059` and `WR-060`.

Roadmap closeout must move `WR-058` to completed, preserve known gaps in the
archive, and update `PM-RENDER-GPU-004` with completion quality and closeout
evidence. Production render, roadmap render, docs validation, and planning
validation must pass after those source updates.

## Perfectionist Closeout Audit

Expected completion quality for this row is `bounded_contract`.

`runtime_proven` is not expected for `WR-058` alone because the canonical boids
runtime proof and broader procedural visuals production evidence remain in
`WR-059` and `WR-060`. `perfectionist_verified` is not available until the full
renderer production audit track proves there are no known quality gaps.

Known quality gaps to preserve unless the implementation closes them:

- canonical boids still needs to consume the procedural API in `WR-059`;
- runtime GPU evidence for the complete procedural visual chain remains a
  `WR-060` requirement;
- local 2D SDF impostors do not include 3D SDF raymarch, sparse residency, or
  product-owned SDF authority decisions.
