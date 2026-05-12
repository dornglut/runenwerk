---
title: Field Product Diagnostics System Design
description: Superseded draft for field product diagnostics and inspection.
status: superseded
owner: workspace
layer: cross-domain
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
---

# Field Product Diagnostics System Design

## Status

Superseded draft.

Replaced by `../accepted/field-product-contracts-diagnostics-and-residency-design.md`.

This document defines diagnostics and inspection for the Adaptive Field Product System.

Diagnostics are not optional polish. They are part of the product contract.

---

# Purpose

Runenwerk needs a way to explain:

- what products exist
- where they apply
- which product is selected
- which product is resident
- which product is stale
- which fallback is active
- which ghost summary is active
- what failed to rebuild
- which source changed
- which consumer uses which product
- what product generations are active
- whether a product is authoritative or derived

Without this, field products, streaming, renderer selection, and simulation handoffs become opaque and fragile.

---

# Design Goals

1. Make product state inspectable.
2. Make lineage and freshness visible.
3. Make residency and fallback visible.
4. Make missing/stale/failed products actionable.
5. Expose consumer/product usage.
6. Support editor overlays and inspector panels.
7. Support validation and regression checks.
8. Avoid exposing mutable internals.
9. Keep diagnostics domain-owned where possible.
10. Make ghost/non-authoritative data obvious.

---

# Non-Goals

This design is not:

- a logging-only system
- an editor panel implementation
- a replacement for domain ratifiers
- a telemetry backend
- a renderer debug HUD only
- a generic ECS inspector

It defines product diagnostics contracts and expected inspection surfaces.

---

# Diagnostic Principles

## Diagnostics explain product truth

Every product should be able to explain:

- who produced it
- what it depends on
- what generation it represents
- whether it is current
- whether it is resident
- whether it is fallback
- why it is stale or missing
- who is consuming it
- whether it is authoritative or derived

## Diagnostics are typed

Avoid unstructured string-only diagnostics.

Diagnostics should have:

- stable code
- severity
- product identity
- scope
- consumer class
- message
- suggested action where applicable
- source lineage references where applicable

## Diagnostics are read-only

Inspection APIs expose read-only DTOs.

Tools must not mutate product internals directly.

---

# Diagnostic Categories

## Product Existence

Answers:

- Does this product exist?
- Was it declared but never formed?
- Is the product retired?
- Does a replacement exist?

Example issues:

| Code | Meaning |
|---|---|
| `field.product.missing` | Product is required but unavailable. |
| `field.product.declared_not_formed` | Product is declared but has no payload. |
| `field.product.retired_selected` | Consumer selected a retired product. |

## Freshness

Answers:

- Is the product current?
- What made it stale?
- Is stale use allowed?
- What upstream generation changed?

Example issues:

| Code | Meaning |
|---|---|
| `field.product.stale` | Product is outdated. |
| `field.product.potentially_stale` | Product may be outdated but usable by policy. |
| `field.product.generation_mismatch` | Product generation does not match expected lineage. |

## Residency

Answers:

- Is the product loaded?
- Is it pending load/unload?
- Is it fallback resident?
- Is it using a ghost summary?

Example issues:

| Code | Meaning |
|---|---|
| `field.product.non_resident` | Product payload is not resident. |
| `field.product.pending_load` | Product is requested but not ready. |
| `field.product.ghost_summary_active` | Consumer is using non-authoritative summary. |
| `field.product.fallback_active` | Consumer is using lower-quality fallback. |

## Formation/Rebuild

Answers:

- Did product formation succeed?
- What failed?
- Was prior product preserved?
- What rebuild policy applies?

Example issues:

| Code | Meaning |
|---|---|
| `field.product.formation_failed` | Product formation failed. |
| `field.product.failed_preserved` | Prior product preserved after failed rebuild. |
| `field.product.rebuild_budget_exhausted` | Rebuild skipped due to budget. |

## Dependency/Lineage

Answers:

- What produced this product?
- What inputs does it depend on?
- Which source changed?
- Which downstream products are affected?

Example issues:

| Code | Meaning |
|---|---|
| `field.product.missing_dependency` | Required upstream product/source is missing. |
| `field.product.lineage_ambiguous` | Source lineage is incomplete. |
| `field.product.untracked_dependency` | Product depends on undeclared input. |

## Consumer Usage

Answers:

- Which consumer selected this product?
- Why was this product selected?
- Was fallback allowed?
- Was stale use allowed?

Example issues:

| Code | Meaning |
|---|---|
| `field.consumer.unsupported_product` | Consumer requested unsupported product kind. |
| `field.consumer.invalid_scale_band` | Consumer requested invalid band. |
| `field.consumer.strict_query_fallback_rejected` | Strict consumer rejected fallback. |

## Authority

Answers:

- Is the product authoritative?
- Is it derived?
- Is it visual-only?
- Is it safe for gameplay?

Example issues:

| Code | Meaning |
|---|---|
| `field.product.visual_only_used_for_strict_query` | Visual product was used where strict product is required. |
| `field.product.derived_used_as_authority` | Derived product treated as authoritative. |
| `field.product.ghost_used_for_authority` | Ghost summary used for authoritative result. |

---

# Inspection Surfaces

## Product Inspector

Shows one product:

- identity
- family/kind
- scope
- scale band
- lineage
- generation
- freshness
- residency
- rebuild policy
- retention policy
- consumers
- diagnostics

## Region Product View

Shows all products in a region/chunk/sector:

- terrain products
- material fields
- collision products
- vegetation products
- water products
- influence products
- renderer-selected products
- stale/fallback/missing state

## Consumer View

Shows what a consumer is using:

- renderer selected products
- physics selected products
- AI selected products
- editor selected products
- network-relevant products later

## Lineage View

Shows upstream/downstream graph:

```text
source asset / operation log / simulation state
  -> formed product
  -> downstream render/collision/AI products
  -> runtime cache
```

## Residency View

Shows load state:

- resident
- pending load
- pending unload
- fallback
- ghost
- missing
- failed preserved

## Overlay Views

Viewport overlays:

- product scope boundaries
- LOD/scale bands
- residency state
- stale/fallback products
- ghost summaries
- collision vs visual product mismatch
- influence field heatmaps
- water/wetness fields
- grass density/trample fields

---

# Diagnostic Payload Model

A diagnostic record should include:

| Field | Meaning |
|---|---|
| Code | Stable diagnostic code. |
| Severity | Info, warning, error, blocking. |
| Product ID | Product involved, if applicable. |
| Product family/kind | Classification. |
| Scope | Chunk/region/sector/view/etc. |
| Consumer | Consumer involved, if applicable. |
| Generation | Product/source generation. |
| Message | Human-readable explanation. |
| Cause | Source/invalidation/rebuild reason. |
| Suggested action | Optional recovery/action. |
| Related products | Upstream/downstream products. |

---

# Severity Model

| Severity | Meaning |
|---|---|
| Info | Expected state or debug information. |
| Warning | Usable but degraded or suspicious. |
| Error | Product failed or consumer cannot use requested product. |
| Blocking | Product/state must not be accepted by owning domain. |

Blocking diagnostics should be domain-owned and ratifier-backed where appropriate.

---

# Renderer Diagnostics

Renderer diagnostics should expose:

- selected render products
- product generation
- residency state
- GPU upload state
- fallback/ghost state
- missing texture/field/target
- stale product usage
- target alias resolution
- history target validity
- dynamic product surface state

Renderer diagnostics should not expose mutable backend resources.

---

# Streaming Diagnostics

Streaming diagnostics should expose:

- active scopes
- requested products
- pending loads
- unload candidates
- memory pressure
- upload budget pressure
- rebuild budget pressure
- fallback use
- ghost summaries
- high-priority misses

---

# SDF World Diagnostics

SDF diagnostics should expose:

- terrain field products
- material fields
- chunk/region lineage
- dirty operation source
- field preview availability
- collision product availability
- SDF query product state
- render product state

---

# Character and Prefab Diagnostics

Character diagnostics should expose:

- SDF body graph product
- pose product generation
- animation product state
- collision/query product state
- interaction emitters
- render product selection

Prefab diagnostics should expose:

- placement product
- SDF composition product
- material field rules
- collision/render product state
- LOD/fallback state

---

# Vegetation Diagnostics

Vegetation diagnostics should expose:

- density fields
- species rules
- wind inputs
- trample/bend fields
- LOD band selection
- recovery/decay state
- product fallback

---

# Water Diagnostics

Water diagnostics should expose:

- water masks
- surface/level-set fields
- flow fields
- wetness products
- foam/mist fields
- buoyancy/query products
- future solver state
- fallback/ghost status

---

# Day/Night Diagnostics

Atmosphere diagnostics should expose:

- time-of-day
- sun/moon direction
- sky/fog/exposure state
- material response state
- lighting invalidation state
- enemy schedule hooks where relevant

---

# Validation and Tests

Diagnostics should support tests for:

- missing product produces diagnostic
- stale product is reported
- fallback use is explicit
- strict consumer rejects visual fallback
- ghost summary cannot satisfy authority query
- product generation mismatch is detected
- failed rebuild preserves prior valid product when policy allows
- renderer target alias failures are diagnosable
- streaming budget exhaustion is diagnosable

---

# Open Questions

1. What diagnostic code namespace should be used for field products?
2. Which product diagnostics belong in `world_sdf` versus `world_ops` versus engine render?
3. How should editor panels subscribe to product diagnostics?
4. Should diagnostics be stored as products themselves?
5. Which diagnostics are persisted across sessions?
6. How should diagnostics integrate with existing ratification reports?
7. What is the minimum overlay set for the first production slice?
8. Should product lineage be visualized as a graph, table, or both?
9. How do diagnostics behave in multiplayer clients versus server authority?
10. How much backend renderer state may be exposed in inspection DTOs?

---

# Design Decisions

1. Diagnostics are part of the product contract.
2. Diagnostics use stable codes.
3. Inspection APIs expose read-only DTOs.
4. Ghost and fallback use must be visible.
5. Strict consumer failures must be diagnostics, not silent fallbacks.
6. Product lineage must be inspectable.
7. Renderer diagnostics must not leak mutable backend internals.
8. Editor overlays are consumers of diagnostic products.
9. Diagnostics support tests and validation.
10. Authority mismatches are high-severity issues.

---

# Implementation Phases

## Phase 1: Diagnostic Vocabulary

Deliver:

- field product diagnostic code namespace
- severity model
- product diagnostic DTO
- basic report aggregation

## Phase 2: Product Inspector

Deliver:

- product identity/family/scope/freshness/residency view
- lineage fields
- related diagnostics

## Phase 3: Region Product View

Deliver:

- products by chunk/region/scope
- residency/freshness/fallback state
- missing products

## Phase 4: Consumer View

Deliver:

- renderer selected products
- physics selected products
- editor selected products
- strict/fallback rules

## Phase 5: Renderer/Streaming Overlays

Deliver:

- loaded product overlays
- stale/fallback/ghost overlays
- target/history diagnostics

## Phase 6: Domain-Specific Views

Deliver:

- SDF world diagnostics
- vegetation diagnostics
- water diagnostics
- character/prefab diagnostics
- day/night diagnostics

## Phase 7: Validation Integration

Deliver:

- tests for diagnostic invariants
- docs validation hooks where useful
- regression captures for product state

---

# Acceptance Criteria

This design is accepted when:

1. Every field product can produce an inspection record.
2. Stale/fallback/missing/ghost states are visible.
3. Product lineage is inspectable.
4. Consumers can explain product selection.
5. Strict consumer fallback rejection is diagnosable.
6. Renderer product diagnostics are available without backend leakage.
7. Streaming diagnostics expose residency and budget state.
8. The first production slice can be debugged through product diagnostics.
