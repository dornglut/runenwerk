---
title: Resource Ref
description: Current public API and boundaries for the foundation resource_ref crate.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-16
---

# Resource Ref

`foundation/resource_ref` owns portable external resource references for
domain-neutral data that needs to point at another authored or derived resource
without depending on an app catalog, renderer, filesystem, or runtime lookup
service.

It describes identity only. It does not resolve resources, check catalog
existence, load files, mutate assets, choose importer policy, or validate
domain-specific compatibility.

## Public API

Core entry points from `resource_ref::`:

- `ResourceRef`
- `ResourceRefKind`
- `ResourceStableId`
- `ResourceRevisionRef`
- `ResourceArtifactRef`
- `ResourceRefError`

## Invariants

- Resource kinds, stable IDs, revisions, and artifact selectors are non-empty.
- Identifier components are trimmed at construction and cannot contain empty
  values.
- `ResourceRef::canonical_component()` uses length-prefixed canonical encoding
  so delimiter-like user data cannot collide.
- Optional revision and artifact selectors are explicit identity components
  when present.

## Boundary

Owning domains decide what a reference means and which resource kinds are
compatible. Apps and adapters resolve references against catalogs, project
files, or host systems. `foundation/resource_ref` remains a portable identity
vocabulary shared by those layers.
