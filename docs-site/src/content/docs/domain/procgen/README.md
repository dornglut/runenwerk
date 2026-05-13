---
title: Procgen Domain
description: Accepted ownership boundary for deterministic procedural generation documents, lineage, lowering contracts, and product outputs.
status: accepted
owner: procgen
layer: domain
canonical: true
last_reviewed: 2026-05-13
related_docs:
  - ../../workspace/sdf-first-execution-roadmap.md
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../graph/README.md
  - ../product/README.md
  - ../world-ops/README.md
  - ../world-sdf/README.md
---

# Procgen Domain

`domain/procgen` owns deterministic procedural generation contracts. Phase 6A
creates the domain crate for graph-backed documents, terrain/material node
semantics, ratification, deterministic lowering to `world_ops` operation
windows, and product job/publication descriptors.

## Pipeline

Every procgen slice follows this invariant:

```text
authored procgen document
  -> graph structural validation
  -> procgen semantic ratification
  -> deterministic lowering plan
  -> product job outputs
  -> publication/query/render/residency substrate
  -> editor diagnostics/surfaces
```

Runtime and preview systems consume formed products. They must not consume
editor graph canvas state, unratified documents, renderer caches, or private
generator scratch data as world truth.

## Planning Lifecycle

Procgen is the first concrete owner for content-planning vocabulary. The
initial lifecycle is:

```text
Prototype
  -> Candidate
  -> Reservation
  -> Instance Plan
  -> Realization
  -> Runtime State
```

Lifecycle ownership:

- prototype: reusable authored generator or content intent inside a procgen
  document;
- candidate: possible scoped placement or generated output with rule matches,
  score, constraints, and explanation data;
- reservation: provisional spatial, product, or gameplay claim used to prevent
  invalid overlap before realization;
- instance plan: accepted, ratified procgen plan with stable identity, scope,
  lineage, reservation, write targets, and realization requests;
- realization: lowering into governed `world_ops::OperationRecord` windows,
  formed field-product candidates, prefab/entity markers, or later owning
  domain products;
- runtime state: changes produced by play, simulation, or authority. Runtime
  state is owned by the relevant world, gameplay, simulation, or network domain,
  not by procgen.

The lifecycle is not a generic content-platform contract yet. It is incubated
inside procgen until repeated non-procgen use proves which parts are shared.

## Ownership

`domain/procgen` owns:

- `ProcgenDocument` source descriptors for deterministic generation intent;
- procgen-owned semantic node catalog rules over neutral `domain/graph`
  structure;
- seed, scope, version, input-product, write-target, and output-product
  descriptor policy;
- deterministic lowering contracts into bounded operation windows and formed
  product candidates;
- procgen issue codes, ratification rules, diagnostics, source maps, cache
  lineage, and failed-product preservation semantics.

This domain does not own:

- editor graph canvas layout, selection, pan, zoom, or provider routing;
- runtime scheduling, worker pools, product job execution, or publication
  barriers;
- renderer GPU resources, residency handles, upload policy, or render cache
  ownership;
- asset catalog authority or app-local cache folders;
- networking transport or server implementation;
- concrete terrain, cave, stamp, material, scatter, or biome algorithms.

## Accepted Document Contract

The first procgen source format is a typed `ProcgenDocument` over
`domain/graph::GraphDefinition`. `domain/graph` owns only graph structure.
`domain/procgen` owns the node catalog, node semantics, ratification, and
lowering rules.

Every accepted descriptor must carry:

- stable document identity and schema version;
- generator identity and generator version;
- world seed reference;
- bounded region and chunk scope;
- input product identities and generations;
- authored overlay generation;
- explicit write targets;
- declared output product descriptors;
- lowering policy;
- runtime/offline budget and retention class;
- diagnostics policy;
- cache lineage inputs.

The first implemented product track is bounded region terrain/material
generation that lowers to deterministic `world_ops::OperationRecord` windows
plus changed-region diagnostics. Phase 6A exposes product job and publication
descriptor builders for operation-window and field-product candidate outputs,
but does not generate field payload bytes or execute runtime publication.
Cave mask/stamp, scatter/distribution, concrete algorithms, editor providers,
preview execution, and bake commands remain deferred.

## Phase 6A Public Surface

The initial crate surface is intentionally domain-only:

- `ProcgenDocument` wraps `domain/graph::GraphDefinition` and stores
  procgen-owned node parameters, seed/version/scope/input/write-target/output,
  lowering, diagnostics, and cache-lineage fields.
- `ProcgenNodeCatalog::first_slice()` admits only terrain/material nodes:
  height/noise, material rule, world-operation output, field-product output,
  and diagnostics.
- `ratify_procgen_document` rejects unbounded scopes, unsupported nodes,
  missing output nodes/products, invalid deterministic inputs, duplicate or
  invalid write targets, stale cache lineage, and reservation conflicts.
- `lower_procgen_to_world_ops` lowers ratified documents to deterministic
  `world_ops::OperationRecord` windows using `DensityFieldDeform` and
  `MaterialFieldEdit` metadata payloads.
- `build_procgen_product_contracts` creates ratifiable `ProductJobDescriptor`
  and `ProductDescriptorCore` values for operation-window and field-product
  candidate outputs.

## Determinism And Lineage

The procgen determinism key is:

```text
world seed
  + generator identity/version
  + document source revision
  + schema version
  + bounded scope
  + parameter hash
  + upstream product generations
  + authored overlay generation
  + lowering version
```

The same key must produce identical lowering plans, operation windows, product
descriptors, and diagnostics. Any nondeterministic input must be ratified as
visual-only or rejected for strict/product-authoritative output.

## Authored Edit Layering

Generated bases are separate from authored overlays.

Authoritative authored edits remain `domain/world_ops` operations or authored
layers. Regeneration must not silently rewrite those edits. If a regenerated
base conflicts with authored overlays, the procgen product reports explicit
diagnostics and preserves the prior accepted product or candidate until a
governed command accepts a merge, rebake, or rollback.

## Product And Runtime Integration

Procgen candidates become product-job outputs. They must publish only through
product publication barriers, expose query snapshots, obey strict consumer
policy, feed render selection as backend-neutral product descriptors, and
request GPU residency only through derived renderer residency contracts.

Runtime preview and offline bake use the same future contract. They differ only
by budget, retention, and command boundary:

- runtime preview is bounded, diagnosable, and non-authoritative until accepted;
- offline bake can produce retained candidates but still needs ratification and
  governed publication;
- bake-to-`world_ops` and bake-to-field-product paths preserve source lineage,
  changed regions, diagnostics, and rollback evidence.

## Shared Contract Promotion

Do not extract a shared `domain/content_planning` contract during the first
procgen product track.

Shared candidate, reservation, or instance-plan contracts may be promoted only
after at least two non-procgen domains need the same model. Plausible future
consumers include prefab placement, vegetation/scatter products, water or
world-process planning, gameplay influence or encounter planning, and editor
explanation surfaces. Until then, each domain should consume procgen products
or define its own narrower owner-specific contract rather than depending on a
premature universal planner.

## Multiplayer Authority

The accepted multiplayer policy is server-validated deterministic generation.

Clients may reproduce generation from shared seeds, scopes, versions, input
generations, and authored overlays. Authoritative sessions validate and publish
accepted product generations or operation windows. Derived render caches and
local previews are never replicated as authority.

## Gates

Phase 6A has implemented the domain crate. Future procgen work must still keep
these gates:

- this domain contract is the implementation source of truth;
- product jobs, publication barriers, query snapshots, strict consumers, render
  product selection, and derived GPU residency remain the only runtime product
  path;
- first-slice tests continue proving deterministic operation-window lowering
  from the same seed, scope, version, inputs, and overlay generation.

Do not implement editor procgen providers, preview execution, bake execution,
worker pools, or concrete generator algorithms until the next accepted
Phase 6B implementation plan.
