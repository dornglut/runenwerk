---
title: Renderer Procedural Population Hardening Platform
description: Active design for fail-closed indirect draw contracts, reusable GPU primitive dispatch, and graph-level fixed-step catch-up after the procedural population runtime proof.
status: active
owner: engine
layer: engine-runtime / renderer / procedural
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - renderer-procedural-population-platform-design.md
  - ../accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_reports:
  - ../../reports/closeouts/pt-render-procedural-population-runtime-proven/closeout.md
---

# Renderer Procedural Population Hardening Platform

## Decision

`PT-RENDER-PROCEDURAL-POPULATION-HARDENING` closes the direct technical gaps
left visible by the `PT-RENDER-PROCEDURAL-POPULATION` runtime-proven closeout.
This is a focused hardening track, not a cleanup bucket and not a replacement
for `PT-RENDER-PERFECTION`.

The renderer must harden three production contracts before later population
expansion:

- indirect draw submission must be typed, bounds-checked, indexed-aware, and
  fail-closed before runtime submission;
- GPU primitives must lower into reusable renderer-owned shader dispatches,
  including hierarchical prefix scan for arbitrary counts;
- fixed-step catch-up must become graph scheduling with bounded repeated pass
  execution, not example-local timing logic.

Spatial hash and chunked unbounded populations are intentionally outside this
track. They require a separate intake and design because they add collision
policy, chunk residency, world-coordinate windows, and product ownership
questions that are not direct hardening of the bounded population platform.

## Scope

This track covers:

- indirect draw contract hardening for direct, indexed direct, indirect, and
  indexed indirect draw sources;
- validation of indirect argument buffer type, element count, byte size, byte
  offset alignment, byte offset bounds, and indexed versus non-indexed
  compatibility;
- removal or redesign of indirect APIs whose CPU-side vertex or instance
  offsets cannot affect WGPU indirect submission;
- renderer-owned GPU primitive kernels for counter reset, u32 prefix scan,
  scatter/compaction, and indirect draw argument generation;
- primitive plan lowering into normal render-flow compute passes with stable
  labels, typed diagnostics, and no hidden CPU fallback;
- graph-level fixed-step scheduling for bounded repeated pass execution,
  including accumulated time, fixed dt, submitted substeps, max substeps, and
  dropped or deferred time diagnostics;
- benchmarks, public renderer docs, and closeout evidence sufficient for
  `runtime_proven`.

## Non-Scope

This track does not implement:

- spatial hash grids;
- chunked unbounded population residency;
- world-coordinate streaming windows;
- hash collision policy;
- renderer-owned gameplay or product truth;
- final no-gap renderer verification.

Those remain separate design or audit work. `perfectionist_verified` remains
owned by `PT-RENDER-PERFECTION`.

## Ownership

The bounded context owner is `engine/src/plugins/render`.

Renderer owns:

- render-flow draw-source semantics, validation, execution-plan compilation,
  and WGPU submission selection;
- GPU primitive kernels, primitive dispatch lowering, primitive diagnostics,
  and primitive benchmark evidence;
- graph scheduling contracts for repeated pass execution and renderer-visible
  fixed-step evidence;
- public renderer usage docs and API reference updates.

Renderer does not own:

- gameplay or world source truth;
- semantic population identity;
- product selection;
- authored simulation meaning;
- streaming authority;
- fallback legality outside renderer capability diagnostics.

## Current Gaps

The completed procedural population track left these explicit gaps:

- `engine/src/plugins/render/graph/pass_graph.rs::RenderDrawSource` represents
  indirect drawing, but generic `IndirectDrawArgsBuffer` typing is erased before
  validation and execution, so indexed and non-indexed argument compatibility is
  not yet a fully enforced runtime contract.
- `engine/src/plugins/render/graph/validation.rs::validate_graphics_draw_source`
  checks declaration and 4-byte byte-offset alignment, but does not prove byte
  offset bounds against the typed indirect argument element size.
- `engine/src/plugins/render/api/passes.rs::GraphicsPassBuilder::draw_indirect_with_offsets`
  exposes CPU-side offsets that WGPU indirect submission cannot consume; that
  API shape can mislead users into believing offsets are applied outside the
  indirect argument buffer.
- `engine/src/plugins/render/gpu_primitives` provides reusable descriptors,
  validation, and explicit primitive execution plans, but not renderer-owned
  shader dispatch kernels.
- `engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState`
  exposes fixed-step evidence for one submitted simulation step, but multi-step
  catch-up is not graph scheduling and must not be added locally to boids.

## Architecture Rules

- Direct draw remains the simple public path through `.draw(...)` and
  `.draw_with_offsets(...)`.
- Indirect draw authoring must use explicit indirect methods with typed
  argument-buffer expectations.
- Indexed indirect and non-indexed indirect sources must be distinct validation
  and execution paths.
- Indirect offsets must be expressed by the indirect argument buffer and the
  indirect byte offset. CPU-side vertex, base-vertex, or instance offsets must
  not be accepted on indirect APIs unless a backend-supported contract consumes
  them.
- Primitive dispatch must use renderer-owned shaders and render-flow compute
  passes, not descriptor-only metadata or boids-local shader patches.
- Prefix scan must support arbitrary total counts through a hierarchical plan:
  block scan, block-sum scan, and block-offset propagation.
- Unsupported backend capability for required runtime proof must produce a
  typed diagnostic and fail closed. A CPU fallback is allowed only if a future
  design explicitly names it as an offline/debug path, not as runtime proof.
- Fixed-step catch-up must be graph-level bounded repeated pass execution that
  preserves resource sequencing across substeps.

## Production Slices

- `WR-089`: doctrine and track activation.
- `WR-090`: indirect draw contract hardening.
- `WR-091`: reusable GPU primitive shader dispatch.
- `WR-092`: fixed-step graph catch-up scheduling.
- `WR-093`: evidence, benchmarks, docs, and runtime-proven closeout.

`WR-089` is the activation slice only. It must not absorb product code from
later slices.

## Evidence

Closeout evidence must prove:

- invalid indexed versus non-indexed indirect argument buffers fail graph
  validation before submit;
- indirect byte offsets are alignment-checked and bounds-checked against typed
  argument buffer element sizes;
- indirect APIs do not expose ignored CPU offsets;
- renderer-owned primitive shader dispatch is exercised outside the boids
  example;
- u32 prefix scan supports arbitrary counts through hierarchical dispatch;
- primitive execution has typed diagnostics and no hidden CPU fallback in
  runtime proof paths;
- graph fixed-step catch-up submits `0..N` bounded substeps deterministically;
- ping-pong and other pass resource sequencing remain stable across substeps;
- docs and benchmarks match the public API and runtime behavior.

