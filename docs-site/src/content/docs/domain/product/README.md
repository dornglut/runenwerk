---
title: Product Domain
description: Engine-agnostic contract crate for formed products, product jobs, query snapshots, render product selection, diagnostics, and ratification.
status: active
owner: product
layer: domain
canonical: true
last_reviewed: 2026-05-12
related_designs:
  - ../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../workspace/sdf-first-execution-roadmap.md
---

# Product Domain

`domain/product` owns shared engine-agnostic product contracts. It provides
descriptor, policy, diagnostic, product-job, query-snapshot, render-selection,
and ratification vocabulary for SDF-first formed products.

## Ownership Boundary

This crate owns contract vocabulary only. It does not own product storage,
runtime execution, renderer GPU resources, asset catalogs, editor surfaces, app
workflow policy, or a global product registry.

Product-family truth remains in the owning domain, such as `world_sdf`,
`material_graph`, or `texture`. Engine runtime executes jobs and renderer code
consumes prepared selections as derived state.

The SDF-first open-world substrate roadmap uses this crate as the shared
vocabulary for products, jobs, query snapshots, render selections, diagnostics,
and ratification. It must not become the product registry, catalog authority, or
world truth owner.

## Public Surface

- `ProductDescriptorCore`, `ProductFamily`, `ProductKind`, `ProductScope`,
  `ProductScaleBand`, and `ProductLineage`.
- `ProductFreshness`, `ProductResidency`, `ProductConsumerClass`,
  `ProductAuthorityClass`, `ProductRetentionPolicy`, `ProductRebuildPolicy`,
  and `ProductQueryPolicy`.
- `FieldProductDiagnostic` and stable diagnostic issue categories.
- `ProductJobDescriptor` and deterministic job metadata.
- `QuerySnapshotProductDescriptor`.
- `RenderProductSelection`.
- Ratifiers for descriptors, jobs, snapshots, and render selections.

## Invariants

- Strict current-only consumers reject stale, fallback, ghost, missing,
  visual-only, and diagnostic-only products.
- Failed-preserved products require diagnostics.
- Renderer selection is backend-neutral and cannot contain GPU handles.
- Product jobs describe work and publication targets; they do not execute work
  or mutate authoritative domain truth by themselves.
