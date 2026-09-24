---
title: Renderer Hardware Ray Query And Hybrid Tracing Platform
description: Accepted design for optional hardware ray-query capability, derived acceleration resources, hybrid tracing paths, and mandatory non-RT fallback.
status: accepted
owner: engine
layer: engine-runtime / renderer / backend-capabilities
canonical: true
last_reviewed: 2026-05-23
related_designs:
  - ./render-product-graph-platform-design.md
  - ./renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ./sdf-world-rendering-and-raymarch-acceleration-design.md
  - ./renderer-temporal-reconstruction-and-dynamic-resolution-design.md
---

# Renderer Hardware Ray Query And Hybrid Tracing Platform

## Decision

Hardware ray-query and raytracing support is optional, capability-gated renderer
execution work. It must not become the baseline renderer path and must not move
scene, material, asset, product, temporal, or SDF truth into backend
acceleration structures.

The portable renderer baseline remains raster, prepared product surfaces, SDF
raymarch evidence, temporal reconstruction evidence, and explicit non-RT
fallback. Hardware ray-query paths may add quality or performance when the
backend supports them, but unsupported hardware is a normal diagnosed state.

## Ownership

Renderer owns:

- backend capability reports for ray-query, raytracing pipeline, acceleration
  structure build/update, shader table, timestamp, and readback support;
- derived BLAS/TLAS-style acceleration-resource evidence and build diagnostics;
- hybrid pass scheduling, timing labels, debug capture labels, fallback labels,
  and runtime inspection DTOs;
- ray-query invocation records, unsupported diagnostics, and production
  evidence packets.

Producer domains own:

- scene identity, transforms, visibility intent, and product freshness;
- mesh, material, texture, shader-source, SDF, camera, exposure, temporal, and
  fallback-legality truth;
- semantic decisions about when an RT result is authoritative for gameplay,
  editor tools, or product presentation.

The renderer may cache derived backend resources, but those resources are
invalidated by producer lineage and are never the source of scene, material, or
SDF meaning.

## Capability Policy

Ray-query capability must be feature-detected before use. Unsupported,
partially supported, disabled, or readback-pending states are explicit
diagnostics and must not be collapsed into silent native rendering.

Vendor-specific or backend-specific support is allowed only behind typed
capability evidence. Baseline production evidence must remain valid on machines
without RT hardware by proving the non-RT fallback path.

## Derived Acceleration Resources

Acceleration resources are renderer-owned derived execution artifacts. They
must be built from prepared products, mesh/material handoff records, SDF
residency evidence, and product lineage keys. Public inspection may expose
stable debug identity, source lineage, build/update status, memory estimate,
diagnostic counts, and invalidation reason; it must not expose mutable backend
handles as public authority.

Acceleration builds must fail closed when required source evidence is missing,
stale, over budget, or semantically incompatible. Fallback behavior remains a
diagnosed renderer state, not an invented product-authority decision.

## Hybrid Rendering Path

Hybrid rendering composes existing renderer evidence rather than replacing it:

- raster and material passes remain prepared renderer execution;
- SDF raymarch passes keep conservative acceleration and residency diagnostics;
- temporal reconstruction consumes motion/depth/history and ray-query input
  evidence when available;
- optional ray-query passes add capability-gated work with distinct timing,
  visual, and fallback diagnostics.

Production proof must compare supported and unsupported states where possible,
record visual parity or documented differences against fallback paths, and keep
timing costs separated for raster, SDF, ray-query, reconstruction, and fallback
work.

## Evidence

Production evidence must include:

- ray-query capability matrix with supported, unsupported, disabled, and
  fallback states;
- derived acceleration-resource inspection and source lineage;
- unsupported hardware/backend diagnostics;
- non-RT fallback visual evidence;
- hybrid path timing and pass provenance;
- quality or parity notes for differences between RT and fallback paths;
- benchmark/report artifacts that do not become runtime dependencies.

## Downstream Sequence

Implementation must remain sequenced through the roadmap:

1. `WR-073` adds capability and acceleration-resource inspection.
2. `WR-074` proves hybrid raster/SDF/ray-query runtime behavior and fallback.
3. `WR-075` closes production evidence with docs, examples, hardware matrix,
   benchmark evidence, and visible quality gaps.

Each implementation row needs its own roadmap state, production-plan contract,
write scopes, validation, and closeout evidence before the RT track can claim
`runtime_proven`.

## ADR Boundary

No ADR is required for this doctrine acceptance because the design preserves
the existing renderer ownership boundary and records optional capability
policy. An ADR is required before implementation introduces a durable
cross-domain acceleration-resource ABI, moves product or producer truth into
the renderer, changes fallback authority, or makes RT hardware a baseline
renderer requirement.
