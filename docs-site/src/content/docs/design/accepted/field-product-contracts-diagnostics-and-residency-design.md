---
title: Field Product Contracts, Diagnostics, and Residency Design
description: Accepted target contracts for field products, diagnostics, residency, authority, and query policy.
status: accepted
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-12
related_adrs:
  - ../../adr/accepted/0003-ratification-is-domain-specific.md
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ./sdf-first-field-world-platform-design.md
  - ./sdf-product-renderer-and-gpu-residency-design.md
  - ./execution-fabric-and-product-jobs-design.md
supersedes:
  - ../superseded/adaptive-field-product-system-design.md
  - ../superseded/field-product-diagnostics-system-design.md
  - ../superseded/open-world-product-streaming-system-design.md
---

# Field Product Contracts, Diagnostics, and Residency Design

## Status

Accepted target contract design.

Existing `domain/world_sdf` product types remain the current SDF-world
specialization until code migration creates broader field/product contracts.

## Purpose

Field products are formed, typed, versioned world data used by renderers,
physics, AI, tools, simulations, streaming, diagnostics, and editor workflows.
The contract makes products inspectable and safe to consume without exposing
private owner state.

## Product Descriptor Contract

Every accepted field product descriptor must include:

| Field | Meaning |
|---|---|
| Identity | Stable product identity. |
| Family and kind | Product family such as surface, material, water, influence, collision, expression, diagnostic, plus specific kind. |
| Scope | Chunk, region, clipmap window, view, sector, portal, basin, river segment, or non-spatial scope. |
| Scale band | Near, mid, far, summary, preview, collision/strict-query, offline, or family-specific band. |
| Lineage | Source assets, operations, generated inputs, simulation generations, upstream products, producer version. |
| Freshness | Current, potentially stale, stale, fallback, missing, failed-preserved, retired, or rebuilding. |
| Residency | Resident, non-resident, pending load, pending unload, fallback resident, ghost summary, or not applicable. |
| Consumer class | Renderer, physics, AI, simulation, editor, network, tooling, diagnostics, or family-specific class. |
| Authority class | Authoritative, server-validated, deterministic derived, visual-only, diagnostic-only, or local-only. |
| Retention policy | Frame-local, session-local, cacheable, persisted, retained while referenced, or rebuild-on-demand. |
| Rebuild policy | Immediate, budgeted, lazy, idle, manual, offline, or never. |
| Query policy | Strict current-only, stale allowed, fallback allowed, visual-only, diagnostic-only, or custom owner policy. |
| Diagnostics | Stable issues, severity, cause, suggested action, and related products. |

Product storage may differ by family. Dense grids, sparse bricks, clipmaps,
graphs, summaries, particles, mesh proxies, or GPU-packed caches are storage
choices. Consumers depend on product contracts and query policies, not storage
internals.

## Product Families

Accepted top-level families:

- surface and SDF products;
- material and substance products;
- flow, water, wetness, liquid, and process products;
- influence, perception, navigation, and gameplay products;
- radiance, atmosphere, and time/celestial products;
- collision, movement, interaction, and strict query products;
- expression products such as scene color, picking ids, overlays, previews, and
  remote frames;
- provenance and diagnostic products.

Family-specific products may add fields, but they must not remove the common
contract needed for lineage, freshness, diagnostics, and safe consumption.

## Query Policy

Strict consumers must declare their requirements.

Accepted query policy classes:

- `strict_current_only`: rejects stale, fallback, visual-only, missing, and
  ghost products.
- `certified_fallback_allowed`: allows fallback only when the owning domain
  certifies it for the consumer.
- `visual_fallback_allowed`: allows stale or fallback products for visual
  continuity when diagnostics remain visible.
- `diagnostic_only`: exposes product state but must not drive gameplay or
  authoritative decisions.
- `local_visual_only`: may vary by client or viewport and is never authority.

Visual products cannot satisfy strict collision, authoritative gameplay, or
server validation unless an owning domain explicitly certifies the product and
records that certification in its descriptor or ratifier.

## Residency And Streaming

Residency is product-specific. Render distance is not one global number; it is
policy over selected products.

Accepted residency states:

- resident;
- non-resident;
- pending load;
- pending unload;
- rebuilding;
- stale or potentially stale;
- fallback resident;
- ghost summary;
- missing;
- failed-preserved.

Ghost summaries are non-authoritative continuity products. They may support
distant visuals, rough editor streaming views, or explicitly non-authoritative
planning aids. They must not satisfy precise collision, authoritative gameplay,
server validation, or exact simulation correction.

Budgets must be diagnosable:

- resident memory;
- GPU upload;
- CPU formation;
- rebuild jobs;
- unload work;
- high-priority misses;
- background work;
- editor diagnostic overrides.

## Diagnostics Contract

Product diagnostics are typed, read-only, and stable.

`FieldProductDiagnostic` target fields:

- stable code;
- severity: info, warning, error, blocking;
- product identity and family/kind where applicable;
- scope and consumer where applicable;
- generation or lineage reference;
- human-readable message;
- cause;
- suggested action;
- related upstream or downstream products.

Blocking diagnostics should be domain-owned and ratifier-backed when they
control acceptance of products or mutations.

Required diagnostic categories:

- missing, declared-not-formed, retired-selected products;
- stale, potentially stale, generation mismatch;
- non-resident, pending, fallback, ghost summary;
- formation failure, failed-preserved output, rebuild budget exhausted;
- missing dependency, ambiguous lineage, undeclared dependency;
- unsupported consumer request, invalid scale band, strict fallback rejection;
- visual-only product used for strict query;
- derived product used as authority;
- ghost summary used for authority.

## Ownership Answers

- Generic product contracts belong in a domain-level product boundary, not
  foundation and not engine runtime.
- Domain-specific diagnostic codes belong in the owning product domain.
- Foundation diagnostics and ratification provide vocabulary, not field-product
  policy.
- Renderer diagnostics expose selected products, GPU residency, dynamic targets,
  history, and fallback state without mutable backend handles.
- Editor panels subscribe to read-only product inspection DTOs or projections.

## Validation Expectations

Future implementation work should include tests that prove:

- missing products produce diagnostics;
- stale products are reported and rejected by strict consumers;
- fallback use is explicit;
- ghost summaries cannot satisfy authority queries;
- product generation mismatch is detected;
- failed rebuilds preserve prior products only when policy allows;
- streaming budget exhaustion is diagnosable;
- renderer/backend inspection DTOs do not leak mutable backend internals.
