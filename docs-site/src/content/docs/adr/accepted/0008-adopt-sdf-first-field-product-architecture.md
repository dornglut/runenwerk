---
title: Adopt SDF-First Field Product Architecture
description: Decision to make Runenwerk SDF-first and product-driven without making it SDF-only.
status: accepted
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-12
related_designs:
  - ../../design/accepted/sdf-first-field-world-platform-design.md
  - ../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
---

# ADR: Adopt SDF-First Field Product Architecture

## Status

Accepted.

## Context

Runenwerk is evolving toward editable field worlds, large spatial scopes,
product-based rendering, strict collision/query products, diagnostics, and
future simulation domains. A mesh-first world model would make editable field
truth, lineage, changed-region rebuilds, collision/query formation, and
streaming product selection harder to reason about.

The repository already separates authoritative descriptions from execution,
treats projections as derived state, and requires generated or imported state
to pass through owning-domain validation. The world architecture needs the same
kind of explicit source-of-truth rule.

## Decision

Runenwerk is SDF-first and field-product-driven.

SDF and field products are the primary production substrate for world geometry,
editable spatial truth, material and world fields, product lineage, rendering
inputs, collision/query formation, diagnostics, and long-term simulation
handoffs.

SDF-first does not mean SDF-only. Meshes remain valid as:

- imported source assets;
- derived render, debug, export, preview, or fallback products;
- interoperability artifacts;
- specialized representations where an owning design explicitly accepts them.

Meshes must not silently become the authoritative world truth for SDF-first
world systems.

Consumers receive formed products with explicit identity, scope, lineage,
freshness, residency, authority, diagnostics, and fallback policy. Renderer
caches, GPU resources, previews, and UI expression outputs are derived state.

## Rejected Alternatives

### Mesh-First World Truth

Rejected because it makes editable field-world operations, field diagnostics,
strict collision/query formation, and multiscale product streaming secondary
instead of foundational.

### SDF-Only Doctrine

Rejected because it would turn an architectural preference into a constraint
that blocks import, export, debug, fallback, and specialized runtime products.
Runenwerk should be strict about source-of-truth ownership, not dogmatic about
every temporary representation.

### Renderer-Owned World Truth

Rejected because GPU caches and render products are derived execution data.
The renderer may choose, upload, and present products, but it must not own
world, simulation, gameplay, or authoring truth.

### One Universal Solver Or Product Graph

Rejected because terrain, collision, fluids, AI influence, animation, VFX,
lighting, and diagnostics need different invariants and update models. Shared
product contracts are useful; a universal owner is not.

## Consequences

- Domain crates own product contracts, ratifiers, query rules, and invariants.
- Runtime executes product jobs, manages caches, schedules work, and submits
  renderer work without becoming the source of product truth.
- Strict consumers such as collision, gameplay authority, and server
  validation cannot use visual-only products unless an owning design certifies
  the product for that strict use.
- Future designs for characters, prefabs, vegetation, water, AI, procgen, VFX,
  fluids, snow, erosion, and day/night must enter through explicit product
  contracts rather than renderer shortcuts or global mutable state.
- Documentation and implementation should preserve SDF-first direction while
  still allowing derived mesh products where they improve interoperability,
  tooling, or runtime efficiency.
