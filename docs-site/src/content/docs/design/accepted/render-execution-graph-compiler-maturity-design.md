---
title: Render Execution Graph Compiler Maturity Design
description: Accepted design for PM-RENDER-PG-004 render execution graph compiler validation over prepared render selections without renderer-owned product truth.
status: accepted
owner: engine
layer: engine-runtime / render graph compiler
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ./render-product-graph-platform-design.md
  - ./feature-owned-render-contributions-design.md
  - ./render-contract-ergonomics-design.md
  - ./product-surface-platform-hardening-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./render-fragment-data-driven-maturity-design.md
related_roadmaps:
  - ../../engine/roadmaps/fully-featured-renderer-roadmap.md
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# Render Execution Graph Compiler Maturity Design

## Status

This is the accepted design contract for `PM-RENDER-PG-004`.

It accepts the render execution graph compiler maturity direction before any
implementation work starts. It does not authorize product code changes by
itself, does not mark `PM-RENDER-PG-004` complete, and does not assign
`completion_quality`. Implementation still requires a legal bounded WR row,
`task production:plan`, roadmap promotion or current-candidate selection,
focused validation, closeout evidence, and a rerun of
`task ai:goal -- --track PT-RENDER-PG`.

## Goal

Mature the render execution graph compiler so it can reject invalid execution
plans before backend submission while preserving the Product Graph boundary:

```text
Product Jobs and owning producers
  -> prepared render product selections, views, invocations, aliases, history signatures, diagnostics
  -> feature-owned prepared contributions
  -> authored RenderFlow or later merged fragment flow description
  -> Render Execution Graph Compiler
  -> typed compile/preflight diagnostics and compiled execution plan
  -> RenderSubmit executes backend-derived state only
```

The compiler owns render execution validation. It does not own product truth,
product dependency truth, product selection, freshness, authority, fallback
legality, rebuild policy, or residency policy.

## Current Baseline

The renderer already has an execution planning shape:

- `engine/src/plugins/render/graph/validation.rs::validate_flow_graph`
  validates static `RenderFlowGraph` resources, pass shape, pass order,
  resource references, import semantics, and raster/copy/present constraints.
- `engine/src/plugins/render/graph/planning.rs::compile_flow_plan` builds
  `CompiledRenderFlowPlan` from a validated `RenderFlow`.
- `engine/src/plugins/render/graph/execution_plan.rs::compile_execution_plan`
  lowers passes into binding, target, dispatch, draw, and resource reference
  plans.
- `engine/src/plugins/render/renderer/render_flow/runtime_resources/*`
  realizes and resolves flow-owned, surface, dynamic, invocation-uniform, and
  invocation-history resources during backend execution.
- `PreparedRenderFrame` carries views, prepared flow invocations, dynamic target
  descriptors, target alias bindings, history signatures, projected uniforms,
  and feature contributions.

The remaining maturity gap is not a new renderer ownership model. The gap is
that some execution compatibility failures still emerge too late or too
unstructured at runtime. PM-004 should move those failures into typed
compiler/preflight diagnostics without moving product policy into the renderer.

## Ownership

DDD bounded context owner:

- `engine/src/plugins/render` owns the render execution graph compiler,
  backend-neutral execution validation, resource-role diagnostics,
  prepared-frame execution preflight, and compiled execution inspection.

Translation boundaries:

- Product domains and Product Jobs own product graph semantics, product
  freshness, lineage, authority, fallback legality, rebuild policy, residency
  intent, and product diagnostics.
- App/domain producers translate product decisions into prepared render product
  selections, prepared views, flow invocations, alias bindings, dynamic target
  descriptors, history signatures, and producer diagnostics.
- Feature-owned contribution collectors translate prepared resources into
  render feature contribution packets.
- The render execution graph compiler translates those prepared render inputs
  and static flow descriptions into backend-neutral execution plans plus typed
  diagnostics.
- The backend runtime translates compiled execution plans into WGPU allocation,
  binding, command encoding, submission, timing, and presentation.

Team Topologies ownership:

- Complicated-subsystem render owner enabling stream-aligned product and editor
  producers.

No new `foundation` or `domain` crate is required for PM-004. Create a
cross-domain render contract crate only if a later accepted design needs the
compiler contract outside `engine/src/plugins/render`.

## Vocabulary

PM-004 uses this vocabulary consistently:

- `RenderFlow`: authored backend-neutral render flow description.
- `RenderFlowGraph`: static graph of resources and passes.
- `CompiledRenderFlowPlan`: validated flow plus compiled pass/resource
  execution metadata.
- `CompiledFlowExecutionPlan`: pass execution details consumed by backend
  runtime.
- `PreparedRenderFrame`: frame-scoped prepared packet consumed by submit.
- `PreparedFlowInvocation`: a prepared execution of one compiled flow for one
  prepared view/product scope.
- `PreparedTargetBinding`: prepared binding from a target alias to a dynamic,
  surface, or flow-owned target.
- `history scope`: view/invocation/resource identity plus history signature.
- `resource lifetime`: flow-owned, persistent, transient, imported, dynamic, or
  history lifetime from the render execution perspective.
- `backend capability profile`: backend-neutral limits and feature flags the
  compiler can validate before command encoding.

## Locked Decisions

PM-004 accepts these decisions:

- Keep the compiler as an execution-planning layer over existing `RenderFlow`,
  prepared-frame, dynamic target, target alias, history, and feature
  contribution contracts. This is not an RDG-first rewrite.
- Split compiler maturity into two validation moments:
  - static flow validation and planning for resource/pass topology;
  - prepared-frame execution preflight for alias bindings, dynamic target
    descriptors, invocation scopes, history scopes, and backend capability
    compatibility.
- Replace late unstructured runtime execution failures with typed diagnostics
  where the failure can be known before command encoding.
- Preserve `RenderSubmit` as a prepared-frame and compiled-plan consumer. Submit
  must not perform live ECS extraction to discover product, target, fallback,
  feature, residency, or graph decisions.
- Target alias declarations remain static flow authoring placeholders. Prepared
  invocations bind aliases explicitly. The compiler may validate alias kind,
  binding presence, target descriptor compatibility, and sample/write legality,
  but it must not choose product targets.
- History validation is scoped to view id, invocation id, resource id, and
  history signature. The compiler may reject incompatible or ambiguous history
  scopes, but it must not decide product rebuild or freshness policy.
- Resource lifetime validation should derive read/write windows and allocation
  requirements from the compiled plan. It must not infer product residency
  truth from backend cache state.
- Backend capability validation uses explicit backend-neutral capability
  profiles and limits. It must not leak `wgpu` handles into flow, prepared
  frame, product, app, or domain contracts.
- Render fragments remain description inputs that must merge into normal
  `RenderFlow` or equivalent backend-neutral flow descriptions before compiler
  validation. PM-004 does not implement fragment assets, fragment registries,
  hot reload, merge provenance, or last-good fragment promotion.
- `WR-010` remains the render fragment and hot-reload row for
  `PM-RENDER-PG-007`. PM-004 implementation should use a new bounded compiler
  maturity WR row or an explicitly narrowed legal row created by roadmap
  workflow; it must not repurpose `WR-010` for fragment implementation.

## Public Contract Shape

Implementation should add or refine contracts close to the existing graph
vocabulary:

```text
engine/src/plugins/render/graph/diagnostics.rs
engine/src/plugins/render/graph/capabilities.rs
engine/src/plugins/render/graph/prepared_validation.rs
engine/src/plugins/render/graph/resource_lifetimes.rs
engine/src/plugins/render/graph/planning.rs
engine/src/plugins/render/graph/validation.rs
engine/src/plugins/render/graph/execution_plan.rs
engine/src/plugins/render/inspect/graph_dump.rs
engine/src/plugins/render/inspect/plan.rs
```

Expected concepts:

- `RenderExecutionGraphDiagnostic` or equivalent typed diagnostic DTO;
- `RenderExecutionGraphCompileError` or equivalent error over typed
  diagnostics;
- `RenderExecutionGraphDiagnosticKind` with stable machine-readable categories;
- `RenderExecutionGraphPreparedReport` for prepared-frame execution preflight;
- `RenderBackendCapabilityProfile` for backend-neutral limits and support
  flags;
- `CompiledResourceLifetimeWindow` for read/write/last-use windows;
- prepared-frame compiler inspection fields for alias bindings, history scopes,
  dynamic target compatibility, resource windows, and backend capability
  checks.

The exact Rust names may follow local conventions during implementation, but
the contract must stay typed, inspectable, and backend-neutral.

## Static Flow Validation Requirements

Static validation should continue to reject existing invalid shapes and mature
toward these compiler-owned checks:

- duplicate resource and pass ids;
- unknown resource and pass references;
- invalid pass order, dependency cycles, and non-terminal present passes;
- invalid pass kind fields for compute, fullscreen, graphics, copy, present,
  and built-in UI composite passes;
- invalid texture format, usage, sample mode, and color/depth class
  combinations;
- invalid buffer roles, vertex/instance/index/indirect declarations, and
  graphics layout shape;
- invalid raster color/depth outputs;
- invalid copy resource classes;
- unsupported imported resource semantics for active runtime flows;
- target alias kind mismatch when the alias role is statically knowable;
- resource lifetime declarations that cannot satisfy the static pass graph.

Static validation must not inspect live ECS resources, product jobs, product
graphs, or backend caches.

## Prepared-Frame Execution Preflight

Prepared preflight validates the static compiled plan against the prepared
frame before backend encoding:

- every required target alias has an explicit prepared binding for each
  invocation that executes a pass needing that alias;
- prepared alias binding kind matches the compiled alias kind and pass role;
- dynamic target descriptors are present, valid, and compatible with how the
  flow uses the target;
- non-sampleable dynamic targets are not sampled by a later pass or UI binding;
- invocation-local uniform buffers match the compiled uniform resource ids;
- dispatch-from-state values were prepared for compute passes that require
  them;
- history resources have an unambiguous view/invocation/resource scope and
  compatible history signature;
- prepared views and pass view masks agree about main-surface versus offscreen
  product execution;
- feature-gated passes have a prepared feature contribution status and fallback
  policy before submit decides whether to skip or fail them;
- backend capability profile supports requested texture formats, storage
  access, pass kind, bind group shape, draw buffer layout, and copy path.

Prepared preflight may reject a frame, skip feature-gated passes according to
prepared fallback policy, or produce diagnostics. It must not compute whether a
product is fresh, authoritative, rebuildable, or legally fall back.

## Diagnostics Contract

Diagnostics must be structured enough for tools and closeout evidence:

- flow id and flow label;
- pass id and pass label when relevant;
- resource id and resource label when known;
- invocation id and prepared view id for prepared-frame failures;
- alias label and alias kind for target alias failures;
- dynamic target key and descriptor summary for dynamic target failures;
- history scope and signature for history failures;
- backend capability key or limit for capability failures;
- severity;
- stable diagnostic kind;
- human-readable message.

Diagnostics must cover:

- invalid render resources;
- invalid pass order;
- missing or mismatched target aliases;
- incompatible dynamic target usage;
- invalid or ambiguous history scopes;
- resource lifetime escape or use-after-last-write windows;
- unsupported imported resources in active runtime flows;
- backend capability mismatches;
- feature-gated pass execution without prepared contribution/fallback status.

Diagnostics are render execution diagnostics. They do not replace product-domain
diagnostics for stale products, failed rebuilds, illegal fallback, authority
violations, product dependency failure, or residency budget decisions.

## Resource Lifetime Model

The compiler should make lifetime behavior explicit:

- flow-owned persistent resources live for the active compiled flow unless the
  descriptor signature changes;
- transient resources have first-use and last-use windows derived from the pass
  order;
- imported surface resources are borrowed execution inputs or outputs with no
  renderer-owned lifetime;
- dynamic targets are renderer-owned backend resources requested by prepared
  producers and retained according to explicit dynamic target retention policy;
- invocation-history resources are keyed by invocation/view scope and history
  signature;
- flow-owned history textures are persistent execution resources but their
  reuse is scoped by prepared invocation history signatures.

Resource lifetime checks must produce diagnostics before backend command
encoding when a resource cannot satisfy the compiled plan.

## Backend Capability Profile

Backend capability checks should be backend-neutral:

- texture format capabilities;
- sample mode support;
- storage texture support;
- depth target support;
- copy compatibility;
- maximum color attachments currently supported by the runtime;
- bind group entry limits;
- vertex/instance layout limits;
- storage buffer and uniform buffer size limits;
- pass kind support.

The active WGPU backend may populate the profile, but the compiler contract must
not expose WGPU handles or WGPU-only types in prepared product data, fragments,
domain contracts, or app-facing APIs.

## Relationship To Render Fragments

PM-004 is a compiler maturity milestone, not fragment implementation.

Fragments later owned by `PM-RENDER-PG-007` must merge into normal `RenderFlow`
or equivalent backend-neutral flow descriptions before compiler validation.
PM-004 may define the validation seam that merged fragments use, and may add
provenance slots to diagnostics if they are optional and backend-neutral, but it
must not implement:

- fragment asset schemas;
- fragment registry or package metadata;
- fragment merge algorithms;
- hot reload;
- last-good fragment promotion;
- editor fragment authoring tools.

## Relationship To Product Surface Hardening

PM-004 may validate alias, history, lifetime, and capability correctness for
prepared product surfaces. `PM-RENDER-PG-005` still owns broad product-surface
platform hardening across viewport, material preview, field/debug, drawing, and
future preview producers.

PM-004 must not migrate additional producers or invent new product selection
policy.

## Architecture Governance Result

Architecture governance review for this design resolves as:

- DDD owner: `engine/src/plugins/render`.
- Dependency direction: unchanged. Compiler maturity remains in engine runtime
  and consumes existing engine render contracts plus prepared domain/app
  products. It does not move renderer contracts into foundation/domain.
- ADR need: no new ADR while the compiler remains engine-local and preserves
  the accepted Render Product Graph Platform boundary. Add an ADR only if the
  compiler becomes a cross-domain ABI, persisted graph format, external mod
  contract, or changes source-of-truth ownership.
- ATAM-lite tradeoff: earlier typed validation adds compiler surface area and
  diagnostic boilerplate, but reduces late runtime failure, improves
  inspection, and keeps product policy outside backend command encoding. That
  tradeoff is accepted.
- Strangler migration: replace late `anyhow` runtime failures only where
  equivalent typed preflight diagnostics can run beside existing runtime
  guards. Do not delete runtime guards until tests prove typed preflight covers
  the same failure.
- Fitness functions: static validation tests, prepared-frame preflight tests,
  alias/dynamic target compatibility tests, history/lifetime tests, backend
  capability mismatch tests, inspection tests, and submit-boundary tests.
- Ownership mode: complicated-subsystem render owner.

## Implementation Gates

Before code changes:

1. Rerun `task ai:goal -- --track PT-RENDER-PG`.
2. Use roadmap intake/apply or an existing legal row to create/select a bounded
   PM-004 compiler maturity WR row.
3. Do not repurpose `WR-010` for PM-004 fragment or hot-reload implementation.
   `WR-010` remains the render fragment and data-driven maturity row for
   `PM-RENDER-PG-007`.
4. Run `task production:plan -- --milestone PM-RENDER-PG-004 --roadmap <WR-ID>`
   before promotion or implementation.
5. Promote or switch WR state only through the roadmap workflow.
6. Implement one bounded compiler-maturity slice, validate it, create closeout
   evidence, and rerun `task ai:goal -- --track PT-RENDER-PG` before PM-005.

## Validation Required For Implementation

The implementation contract must include focused tests for:

- static resource and pass diagnostics;
- target alias required/missing/mismatched prepared bindings;
- dynamic target descriptor compatibility with compiled target alias usage;
- non-sampleable targets rejected when sampled;
- history signature and invocation/view scope conflicts;
- resource lifetime windows and use-after-last-write diagnostics;
- backend capability profile mismatch diagnostics;
- prepared-frame inspection of compiled/preflight diagnostics;
- render submit consuming only prepared frame and compiled/preflight output.

Expected command families:

```text
cargo test -p engine render_flow
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Non-Goals

PM-004 must not:

- implement render fragments, fragment asset schemas, fragment registry, merge
  provenance, hot reload, or last-good fragment promotion;
- broaden product-surface producer migration beyond compiler validation;
- add native multi-window or multi-surface presentation;
- add production-readiness inspection, capture/replay policy, or performance
  budgets beyond compiler diagnostics needed for this milestone;
- move product truth, product freshness, fallback legality, authority, rebuild
  policy, product dependency truth, or residency policy into the compiler;
- expose renderer-private backend handles to apps, domains, fragments, or
  product producers;
- replace Product Jobs, Product Graph, prepared-frame ownership, or feature
  contribution ownership.

## Acceptance Bar

This design is accepted when:

- PM-004 compiler ownership is clear and separate from Product Graph/Product
  Jobs;
- compiler maturity is split into static validation and prepared-frame
  execution preflight;
- diagnostics, lifetimes, target aliases, history scopes, and backend
  capability checks are typed and inspectable;
- render fragments and hot reload remain PM-007 scope;
- product-surface hardening remains PM-005 scope;
- implementation cannot start until a legal bounded WR row and production plan
  exist.
