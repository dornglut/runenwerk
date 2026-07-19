---
title: RunenSDF Extraction Design
description: Target public boundary, numerical policy, package shape, conformance, and clean-cutover sequence for extracting domain/sdf into RunenSDF.
status: active
owner: sdf
layer: domain/sdf
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenSDF Extraction Design

## Status

Target direction is fixed. Implementation remains blocked until the complete
source, test, and consumer inventory is recorded and the existing Runenwerk
baseline is validated locally.

## Goal

Extract reusable signed-distance-field mathematics into an independent,
host-neutral repository without carrying Runenwerk geometry, world, ECS,
renderer, material, or product ownership into the new package.

## Initial repository shape

Start with one crate:

```text
RunenSDF/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   └── runensdf/
├── examples/
├── tests/
├── docs/
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
└── CHANGELOG.md
```

Do not create `runensdf_core`, `runensdf_queries`, `runensdf_gpu`,
`runensdf_shader`, or a macro crate initially. Add a crate only when a real
independent consumer or dependency boundary proves the need.

## Package identity

```text
repository: Crystonix/RunenSDF
package: runensdf
initial version: 0.1.0
edition: 2024
MSRV: align with the accepted repository-family toolchain baseline
publish: false until release and API gates are accepted
license: MIT OR Apache-2.0
```

Runenwerk may rename its dependency from `sdf` to `runensdf` during cutover. No
compatibility package named `sdf` remains afterward.

## Owned capabilities

RunenSDF owns:

- `SdfField3` or its reviewed replacement;
- `SdfSample`;
- repository-local finite bounds and unbounded-field representation;
- primitives: sphere, box, capsule, cylinder, plane, torus;
- boolean operations: union, intersection, subtraction;
- smooth boolean operations;
- translation, rotation, scale, and affine transforms;
- repeat, mirror, clamp, and domain warp composition;
- finite-difference gradient and normal estimation;
- point classification;
- closest-point and projection queries;
- raymarch queries;
- sphere sweep queries;
- epsilon/default-query policy;
- deterministic CPU reference tests and benchmarks.

## Explicit non-ownership

RunenSDF does not own:

- Runenwerk `geometry`;
- ECS components or resources;
- spatial indexes;
- world chunks or streaming;
- scene nodes;
- materials;
- renderer graph or pipelines;
- WGPU or shader compilation;
- collision-world policy;
- gameplay or editor policy;
- persistence formats unless later accepted explicitly.

## Geometry boundary

The current public dependency on `geometry::Aabb3` and `geometry::Ray3` is
removed before extraction.

RunenSDF owns a minimal validated vocabulary:

```rust
pub struct Bounds3 {
    min: glam::Vec3,
    max: glam::Vec3,
}

pub enum FieldBounds {
    Unbounded,
    Bounded(Bounds3),
}

pub struct Ray3 {
    origin: glam::Vec3,
    direction: glam::Vec3,
}
```

Exact constructor and accessor names may be refined during implementation, but
the following semantics are fixed.

### Bounds3 invariants

- every component is finite;
- `min <= max` component-wise;
- construction is fallible for invalid authored/external values;
- internally derived valid finite arithmetic must not silently create NaN;
- union and intersection behavior is deterministic;
- disjoint intersection must not return a union while claiming intersection;
- an empty/disjoint result must be represented explicitly or conservatively as
  an accepted documented alternative.

The current disjoint-intersection fallback that returns the union is not retained
as final semantics.

### Ray3 invariants

- origin components are finite;
- direction components are finite;
- zero-length direction is rejected at construction or query admission;
- normalized direction is derived once per query;
- query distance remains world-space distance independent of input direction
  magnitude.

Runenwerk owns conversions between `runensdf::Bounds3`/`Ray3` and its own
geometry types. RunenSDF does not implement conversions that require depending on
Runenwerk.

## Field contract

The initial public field contract remains object-model-neutral and CPU-oriented:

```rust
pub trait SdfField3 {
    fn sample(&self, point: glam::Vec3) -> SdfSample;

    fn bounds(&self) -> FieldBounds {
        FieldBounds::Unbounded
    }
}
```

Before implementation, review whether query functions require `?Sized` generic
support so trait objects and borrowed composites work without unnecessary
restrictions.

The trait does not expose renderer, ECS, allocation, mutation, serialization, or
threading policy.

## Sample and numerical policy

`SdfSample` initially owns signed distance only. Material IDs, gradients,
provenance, channels, or payloads are not added until a second real consumer
requires them.

Public constructors and query settings reject non-finite authored inputs.
Negative primitive extents or radii are either rejected or deliberately
normalized through a documented constructor policy; public fields must not allow
callers to bypass that policy.

The design adopts:

- `f32` as the initial scalar;
- explicit finite-value validation at public boundaries;
- deterministic epsilon defaults;
- no hidden global mutable tolerance;
- no panic for ordinary invalid query input;
- structured errors for invalid settings where recovery matters;
- `Option` only for ordinary no-hit/no-result outcomes after valid admission.

## Query API

Current long positional parameter lists are replaced by settings values where
multiple policy inputs exist.

Target families:

```text
RaymarchSettings
ProjectSettings
SweepSettings
GradientSettings or explicit epsilon where one scalar is sufficient
```

Settings validate:

- positive finite epsilon;
- finite non-negative maximum distance;
- non-zero iteration/step budget where required;
- finite radii and offsets;
- any convergence-specific limits.

A query distinguishes:

- invalid settings/input;
- no result within valid limits;
- successful result.

The exact error enum is fixed during the complete API inventory; silent fallback
from invalid user input to defaults is not the final public contract.

## Bounds semantics

Bounds are conservative acceleration facts, not proof that a field is finite.

Rules:

- unbounded combined with union remains unbounded;
- bounded intersection with unbounded may retain the bounded operand;
- transforms conservatively map all corners where that is valid;
- operations that cannot guarantee a finite conservative bound return
  `Unbounded`;
- domain warps and repeats must not claim finite bounds without a proof;
- negative expansion is rejected or clamped through explicit named behavior, not
  silently reinterpreted.

## Threading and allocation

RunenSDF does not own a scheduler or allocator policy. Field values may be used
from multiple threads when their Rust trait bounds permit it. The base trait does
not impose `Send + Sync` globally unless consumer evidence proves that as a
necessary public invariant.

Queries must not allocate in ordinary scalar paths unless documented and
benchmarked.

## Serialization

No stable serialization format is promised in the initial extraction.

Primitive and composition values may later derive or implement serialization
only after an accepted persisted-field/program design names schema identity,
versioning, validation, and migration. Rust layout is never a persistence format.

## Public API review requirements

Before code changes, inventory and decide every public:

- module;
- type;
- trait;
- constructor;
- public field;
- error/result shape;
- re-export;
- default constant;
- generic bound;
- serialization or debug promise.

Public struct fields should become private where they bypass invariants.

## Independent conformance

RunenSDF must prove:

- primitive signed-distance behavior at inside/surface/outside points;
- finite and invalid-input handling;
- conservative bounds for every bounded primitive and operation;
- transform bounds;
- union/intersection/subtraction and smooth-operation edge cases;
- gradient and normal behavior;
- raymarch hit/miss/zero-direction/max-step/max-distance cases;
- projection and closest-point convergence/failure cases;
- sweep cases;
- deterministic repeated outcomes;
- external downstream implementation of `SdfField3`;
- use without Runenwerk;
- stable and MSRV validation;
- benchmark baselines for representative primitive/composite/query workloads.

Property tests should cover finite geometry, bound containment, and operation
relationships where mathematically valid.

## Runenwerk adapter

Runenwerk retains a narrow integration module for:

- conversion to/from Runenwerk geometry;
- world and scene ownership;
- ECS components/resources;
- renderer preparation;
- material and product policy;
- application diagnostics context.

The adapter must not duplicate SDF algorithms.

## Cutover sequence

### SDF-001 — Complete investigation

- read every source and test file;
- inventory all consumers, examples, benchmarks, docs, persisted assets, and
  shader duplication;
- produce the exact public API inventory;
- produce move/stay/redesign/delete classification;
- run the current local baseline.

### SDF-002 — Boundary correction

Inside Runenwerk:

- add repository-local bounds/ray vocabulary to the SDF package;
- remove public and manifest dependency on `geometry`;
- correct validation, disjoint intersection, and query setting semantics;
- migrate consumers through Runenwerk-owned conversions;
- add independent downstream conformance.

### SDF-003 — Repository creation and source transfer

- create `Crystonix/RunenSDF`;
- establish governance, license, validation, and provenance;
- transfer the corrected package and history evidence;
- validate independently.

### SDF-004 — Runenwerk cutover

- pin Runenwerk to the exact RunenSDF revision;
- migrate all consumers;
- delete `domain/sdf`;
- remove old workspace entries and lockfile packages;
- retain no compatibility package or mirror;
- run full Runenwerk validation and runtime examples.

### SDF-005 — Closeout

- record repository SHAs and compatibility;
- update architecture/domain maps;
- record provenance and deleted paths;
- close temporary branches and migration authority.

## Stop conditions

Stop implementation when:

- a public consumer requires an unclassified capability;
- geometry coupling is broader than the recorded boundary;
- shader/GPU code is found to share source authority with CPU SDF code;
- current main is not green for unrelated reasons;
- repository creation or dependency pinning is unavailable;
- the final cutover would require a long-lived duplicate package.

## Definition of done

RunenSDF is complete only when it validates independently, has a public downstream
consumer, contains no Runenwerk dependency, Runenwerk consumes an exact revision,
all original `domain/sdf` source is removed, and the full Runenwerk integration
validation is green.
