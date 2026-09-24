---
title: Product Domain
description: Engine-agnostic contract crate for formed products, product jobs, query snapshots, render product selection, diagnostics, and ratification.
status: active
owner: product
layer: domain
canonical: true
last_reviewed: 2026-05-13
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
- `ProductConsumptionRequest`, `ProductConsumptionDecision`, and
  `evaluate_product_consumption` for strict runtime consumer enforcement.
- `ProductJobDescriptor` and deterministic job metadata.
- `ProductPublicationOutcome`, `ProductPublicationStatus`, and
  `ProductPublicationReport` for barrier-published product job outcomes.
- `QuerySnapshotProductDescriptor`, `QuerySnapshotPublicationStatus`, and
  `QuerySnapshotPublicationReport`.
- `RenderProductSelection`, typed `RenderSelectedProduct`,
  `RenderTargetDescriptor`, and `RenderResidencyRequest`.
- Ratifiers for descriptors, jobs, publication outcomes, snapshots, and render
  selections.

## Invariants

- Strict current-only consumers reject stale, fallback, ghost, missing,
  visual-only, and diagnostic-only products through the product-domain
  consumption API, not copied app-local policy.
- Failed-preserved products require diagnostics.
- Query snapshots mirror descriptor scope, freshness, consumer class, and query
  policy, and preserved or invalidated snapshots surface diagnostics.
- Renderer selection is backend-neutral, cannot contain GPU handles, and
  carries typed freshness, residency, authority, and query-policy state instead
  of string markers.
- Render selection ratification rejects duplicate selected products, invalid
  targets, invalid residency requests, and strict selections that do not satisfy
  their product query policy.
- Product jobs describe work and publication targets; they do not execute work
  or mutate authoritative domain truth by themselves.
- Publication outcomes must match declared product-job outputs and are ratified
  before runtime publication.
