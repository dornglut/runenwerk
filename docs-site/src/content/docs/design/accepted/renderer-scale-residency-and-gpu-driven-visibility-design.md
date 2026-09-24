---
title: Renderer Scale Residency And GPU Driven Visibility Platform
description: Accepted design for finite renderer working sets, GPU residency budgets, culling, LOD, indirect drawing, and millions-scale evidence.
status: accepted
owner: engine
layer: engine-runtime / renderer
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ./render-product-graph-platform-design.md
  - ./sdf-first-field-world-platform-design.md
  - ./sdf-product-renderer-and-gpu-residency-design.md
  - ./renderer-gpu-evidence-and-procedural-visuals-design.md
---

# Renderer Scale Residency And GPU Driven Visibility Platform

## Decision

Renderer scale is a finite-working-set problem. Unbounded world and product
space may exist in owning domains, but renderer submission consumes bounded
resident, visible, and submitted GPU working sets per frame.

The renderer owns derived execution structures: GPU buffers, page or cluster
registries, visibility buffers, indirect command buffers, timing, memory
pressure diagnostics, and inspection DTOs. Product truth, product selection,
freshness, authority, fallback legality, rebuild policy, and residency intent
remain outside the renderer.

## Scope

This track covers:

- renderer-facing chunk, page, cluster, and instance registries;
- finite resident and visible working-set contracts;
- GPU memory class budgets and upload-byte budgets;
- visibility compaction, frustum culling, screen-size LOD, and later occlusion;
- indirect draw and dispatch command generation;
- scale evidence for large addressable populations without per-entity CPU
  submission.

It does not define authored world truth, gameplay entity semantics, product
freshness policy, or domain-owned streaming decisions.

## Ownership

The bounded context owner is `engine/src/plugins/render`, specifically the
renderer execution and inspection surface. The renderer owns derived execution
state only:

- resident GPU buffers, textures, pages, clusters, and instance ranges;
- renderer-facing working-set registry records and stable debug identities;
- visibility, LOD, compaction, and indirect command buffers;
- memory, upload, visibility, culling, LOD, indirect, timing, and unsupported
  capability diagnostics;
- public inspection DTOs, examples, benchmarks, and closeout evidence.

Product, world, domain, editor, asset, gameplay, field, SDF, material, model,
and streaming owners retain source truth, product identity, selection,
freshness, authority, fallback legality, rebuild policy, residency intent,
semantic LOD policy, and authoring workflows.

## Scale Doctrine

The renderer must not claim infinity by attempting to render infinite things.
The accepted model is:

```text
unbounded product/world space
  -> bounded selected products
  -> bounded resident GPU resources
  -> bounded visible candidates
  -> bounded submitted draw/dispatch work
  -> measured frame budget
```

Millions-scale evidence must distinguish:

- addressable product or instance records;
- resident GPU records;
- visible candidates after culling and LOD;
- submitted draw, dispatch, and indirect command counts.

## Vocabulary

- Addressable records: product-owned instances or resources that could be
  rendered if selected and resident.
- Selected products: prepared render products and feature contributions already
  handed to the renderer through accepted product/render contracts.
- Resident records: renderer-owned GPU-backed records available this frame.
- Visible candidates: resident records surviving renderer-owned visibility and
  LOD tests for a view.
- Submitted work: draw, dispatch, and indirect command counts actually encoded
  for the frame.
- Scale band: renderer diagnostic bucket for population size, memory pressure,
  LOD band, or submitted command volume.
- Degraded mode: explicit renderer diagnostic state when a backend capability,
  memory budget, upload budget, or timing source cannot satisfy the preferred
  path.

## Invariants

- Renderer working sets are derived from prepared product selections or feature
  contributions. They do not create, repair, or silently substitute product
  truth.
- Every scale claim must expose addressable, resident, visible, submitted, and
  measured counts separately.
- Memory and upload budgets report pressure, over-budget state, and degraded or
  unsupported capability diagnostics. They do not decide product fallback,
  freshness, rebuild, authority, or streaming policy.
- Visibility and LOD reduce submitted renderer work before command encoding.
  Per-entity CPU submission is not an acceptable millions-scale path.
- GPU-driven paths must be capability-gated. Missing indirect, storage,
  timestamp, or readback support produces typed unsupported diagnostics instead
  of success-shaped no-op behavior.
- Inspection DTOs must not expose WGPU handles, mutable backend caches, or
  product-domain source objects.
- Benchmarks and examples must keep runner code, raw artifacts, and
  human-readable closeout reports in their owning locations.

## Translation Boundaries

Renderer scale records translate product-side selections into execution-side
working-set records. The translation must preserve product lineage keys for
inspection, but product lineage remains descriptive metadata, not renderer
authority to choose product state.

Allowed inbound data:

- prepared render products and feature contributions;
- product lineage keys, bounds, estimated memory class, and renderable instance
  descriptors;
- product-provided policy facts that are already part of accepted product
  contracts.

Forbidden renderer decisions:

- choosing product fallback assets;
- deciding whether product data is fresh or authoritative;
- mutating product residency intent;
- inventing gameplay, field, material, model, or SDF semantics;
- replacing product streaming policy with renderer cache pressure.

## Implementation Sequence

1. Working-set registry and residency budgets:
   establish renderer-owned registry DTOs, memory/upload budgets, product
   lineage fields, pressure diagnostics, and tests proving no product truth
   leaks into renderer state.
2. GPU-driven culling, LOD, and indirect submission:
   add capability-gated visibility compaction and indirect command generation
   so large addressable populations produce bounded submitted work.
3. Scale production evidence:
   add examples, benchmarks, hardware profile reports, docs, and closeout
   evidence that distinguish addressable, resident, visible, submitted, and
   measured frame cost.

Each implementation row requires its own roadmap promotion, implementation
contract, focused validation, and closeout. This doctrine does not authorize
code changes by itself.

## Evidence

Runtime evidence must expose memory pressure, upload pressure, visible counts,
culled counts, LOD bands, indirect command counts, GPU timings, and fallback or
over-budget diagnostics. Any hardware-dependent target must include unsupported
or degraded-mode diagnostics instead of silent success.

## Fitness Functions

Required implementation fitness functions before runtime claims:

- compile-time or unit tests proving renderer registry records carry product
  lineage without product-domain source ownership;
- inspection tests for addressable, resident, visible, submitted, budget, and
  degraded-mode DTO fields;
- validation or preflight tests rejecting unbounded per-entity CPU submission
  paths for scale examples;
- capability tests for unsupported indirect, storage, timestamp, or readback
  paths;
- benchmarks for registry planning, culling/LOD compaction, indirect command
  generation, and evidence reporting;
- docs, roadmap, production, and planning validators after every closeout.

## Architecture Governance

Clean Architecture direction is preserved when product domains feed prepared
facts into renderer execution contracts and renderer diagnostics report derived
execution state back outward. An ADR is not required for this doctrine because
it extends the accepted render product graph and GPU evidence boundaries without
changing ownership direction.

Write or update an ADR before implementation if a later slice introduces a
persisted cross-domain ABI, moves residency or fallback authority into the
renderer, changes product dependency direction, or makes renderer scale records
the canonical product model.

ATAM-lite tradeoff: the design favors bounded, inspectable submitted work and
honest degraded diagnostics over maximum single-backend throughput. Backend
specialization is allowed only behind capability-gated renderer contracts and
portable inspection evidence.

Team Topologies ownership: the renderer is a complicated subsystem team surface.
Product and editor teams remain stream-aligned producers that consume explicit
renderer scale evidence instead of sharing backend-owned mutable state.
