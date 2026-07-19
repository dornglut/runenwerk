---
title: RunenSDF Extraction Design
description: Decision-complete public boundary, numerical policy, package shape, conformance, and clean-cutover design for extracting domain/sdf into RunenSDF.
status: active
owner: sdf
layer: domain/sdf
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../reports/investigations/runensdf-complete-extraction-investigation.md
  - ../../workspace/planning/roadmap.md
---

# RunenSDF Extraction Design

## Status

The extraction target and `PT-RUNENSDF-002` boundary-correction design are
decision-complete. Source changes remain blocked until the repository-family
charter and complete investigation are reviewed, local preflight commands pass,
and a bounded implementation phase is activated.

The complete evidence, public API inventory, test gaps, consumer findings, and
implementation contract live in the linked investigation report. This document
owns the durable target architecture.

## Goal

Create `Crystonix/RunenSDF` as a small, independently useful Rust library for
signed-distance-field mathematics and CPU reference queries without Runenwerk
geometry, ECS, world, renderer, material, or product dependencies.

## Initial repository and package

```text
repository: Crystonix/RunenSDF
package: runensdf
version: 0.1.0
edition: 2024
license: MIT OR Apache-2.0
publish: false until release gates are accepted
```

Initial shape:

```text
RunenSDF/
├── Cargo.toml
├── Cargo.lock
├── crates/runensdf/
├── examples/
├── tests/
├── benches/
├── docs/
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE-MIT
└── LICENSE-APACHE
```

Do not create core/query/GPU/shader/program/macro subcrates initially. A new
package requires independent dependency or release pressure.

## Ownership

RunenSDF owns:

- `SdfField3` and `SdfSample`;
- repository-local bounds and rays;
- analytic primitives;
- hard and smooth boolean operations;
- composition wrappers;
- translation, rotation, scale, and affine transforms;
- finite-difference gradient and normal estimation;
- classification, projection, closest-point, raymarch, and sphere-sweep queries;
- numerical defaults and validation;
- deterministic CPU reference conformance and benchmarks.

RunenSDF does not own:

- Runenwerk geometry;
- ECS components/resources;
- spatial indexes;
- chunks, streaming, scene, or world policy;
- materials or material payloads;
- renderer passes, WGPU, shader generation, or GPU residency;
- editor/game policy;
- stable persisted field/program formats.

## Core field contract

The initial contract remains deliberately small:

```rust
pub trait SdfField3 {
    fn sample(&self, point: glam::Vec3) -> SdfSample;

    fn bounds(&self) -> FieldBounds {
        FieldBounds::Unbounded
    }
}
```

Helpers and queries accept `F: SdfField3 + ?Sized` where no sized value is
required.

The base trait does not require `Send + Sync`, serialization, allocation,
mutation, rendering, or ECS semantics. Consumers may impose stronger bounds.

`SdfSample` initially contains signed distance only. Material IDs, channels,
gradients, provenance, and payloads are deferred until real consumers justify
them.

## Geometry boundary

The current public `geometry::Aabb3` and `geometry::Ray3` dependency is removed
before source transfer.

RunenSDF owns:

```rust
pub struct Bounds3 {
    min: glam::Vec3,
    max: glam::Vec3,
}

pub enum FieldBounds {
    Unbounded,
    Empty,
    Bounded(Bounds3),
}

pub struct Ray3 {
    origin: glam::Vec3,
    direction: glam::Vec3,
}
```

Exact accessor names remain implementation-level, but invariants are fixed.

### Bounds3

- all components are finite;
- `min <= max` component-wise;
- external construction is fallible;
- valid derived arithmetic cannot silently publish NaN/infinity;
- union/intersection are deterministic;
- disjoint finite intersection is `FieldBounds::Empty`;
- `Empty` is distinct from unknown/unbounded.

### FieldBounds algebra

```text
union(Unbounded, x)          = Unbounded
union(Empty, x)              = x
intersection(Empty, x)       = Empty
intersection(Unbounded, x)   = x
intersection(disjoint finite)= Empty
subtraction bounds           = left operand bounds
```

Smooth operations expand finite bounds where required. When a finite conservative
bound cannot be proven, return `Unbounded`.

The current disjoint-intersection fallback to the operands' union is rejected.

### Ray3

- origin and direction are finite;
- zero-length direction is rejected;
- direction normalization occurs once at validated construction or query
  admission;
- hit distance is world-space distance independent of original direction
  magnitude.

Runenwerk owns conversions between RunenSDF and Runenwerk geometry types. Those
conversions do not live in RunenSDF.

## Invariant-preserving values

Primitive, operation-policy, composition, transform, bounds, ray, and settings
fields become private where public mutation bypasses invariants.

Fallible constructors reject invalid external values, including:

- NaN/infinite positions and scalars;
- negative sizes where no explicit signed meaning exists;
- zero/invalid plane normals;
- invalid quaternions;
- singular/non-finite affine transforms;
- zero/non-finite scale;
- invalid clamp ranges;
- invalid repeat periods;
- non-positive/non-finite epsilon;
- invalid iteration/distance budgets.

Deliberate normalization requires an explicitly named constructor or method. It
must not occur invisibly inside `sample()`.

## Numerical policy

- scalar type is `f32` initially;
- sign convention is negative inside, zero on surface, positive outside;
- epsilon defaults are explicit immutable constants/settings defaults;
- there is no global mutable tolerance;
- public authored/query inputs are finite and validated;
- ordinary invalid input does not panic;
- field evaluation that returns non-finite distance is reported by query APIs;
- CPU behavior is deterministic for fixed inputs, settings, target, and expected
  `f32` precision;
- ordinary scalar query paths avoid allocation unless documented and benchmarked.

The package does not promise bit-identical floating-point results across all
architectures unless later conformance proves that stronger contract.

## Gradient and normal policy

Finite differences remain the CPU baseline.

A positive finite epsilon is required. Invalid/non-finite samples and unusable
zero gradients produce explicit failure in the primary API.

The primary normal API does not silently substitute `Vec3::Y`.

A deliberately named fallback helper may exist for debug visualization, but it
must expose that fallback policy and is not used by correctness-sensitive
queries.

## Query API

Policy-heavy queries use validated settings:

```text
RaymarchSettings
ProjectSettings
SweepSettings
```

Use an explicit epsilon value or a small validated settings type for gradient and
classification depending on final implementation ergonomics.

Query outcomes use the semantic shape:

```rust
Result<Option<Hit>, QueryError>
```

- `Err` means invalid admission or invalid field evaluation;
- `Ok(None)` means a valid query produced no hit/convergence within limits;
- `Ok(Some(hit))` means success.

Classification returns an error when the field cannot provide a valid finite
sample rather than treating NaN as on-surface.

Structured errors cover invalid scalar/vector/bounds/ray/transform/settings and
field-evaluation failures. Public API does not use `anyhow`.

## Transform policy

- translation requires finite offset;
- rotation requires a finite non-zero quaternion and stores a validated normalized
  form;
- uniform scale requires finite non-zero scale; negative scale behavior must be
  documented and tested if retained;
- affine transform requires finite invertible transform;
- transformed bounds conservatively map all finite corners;
- operations that cannot guarantee a conservative finite bound return
  `Unbounded`.

Zero scale and singular transforms are invalid rather than constant-field
fallbacks.

## Serialization policy

No stable serialization format is part of the initial extraction. Do not add
serde merely for potential future authoring.

A later persisted SDF program/field design must define schema identity, version,
validation, migration, and deterministic encoding separately from Rust API
versioning.

## Public API policy

- preserve concrete wrappers and ordinary Rust composition;
- no universal graph or AST in the initial package;
- keep preludes small or omit them until usage proves value;
- keep low-level utilities private unless external implementation requires them;
- generated/derived results expose read-only accessors;
- no compatibility aliases when package and API names change before 1.0;
- all public enums/traits receive deliberate pre-1.0 evolution review.

## Independent conformance

RunenSDF must prove:

- valid and invalid construction for every public value;
- primitive inside/surface/outside samples;
- conservative finite bounds and `Empty` algebra;
- hard/smooth operation semantics and bounds;
- composition/repeat/warp/mirror/clamp behavior;
- transform sampling, bounds, and invalid admission;
- finite-difference gradient/normal success and failure;
- classification success and invalid evaluation;
- raymarch hit, miss, inside start, zero direction, step/distance exhaustion, and
  invalid field output;
- projection/closest-point convergence, exhaustion, invalid gradient, and invalid
  field output;
- sweep hit/miss/settings/evaluation cases;
- deterministic repeated results;
- external downstream `SdfField3` implementation using public APIs only;
- trait-object/`?Sized` usage;
- stable and MSRV validation;
- representative benchmarks.

Property tests cover finite bounds, bound containment, algebra relationships, and
transform invariants where mathematically applicable.

## Runenwerk adapter

Runenwerk retains one narrow integration boundary for:

- geometry conversions;
- ECS/world/scene ownership;
- spatial indexing and chunk policy;
- renderer preparation and SDF representation selection;
- materials and product policy;
- integration diagnostics.

The adapter does not duplicate field or query algorithms.

## Cutover phases

### PT-RUNENSDF-001 — Complete investigation

Complete. The linked investigation records all source and test files, public API,
dependencies, docs, known consumer evidence, defects, decisions, and the next
implementation contract. Local verification commands remain required before
implementation activation.

### PT-RUNENSDF-002 — Boundary correction inside Runenwerk

- add local validated geometry/query vocabulary;
- remove `geometry` dependency;
- enforce constructors/settings invariants;
- correct bounds algebra and smooth bounds;
- add structured query outcomes;
- remove implicit primary normal fallback;
- add external/public negative/property conformance;
- migrate any consumers found by mandatory local grep.

No external repository is created in this phase.

### PT-RUNENSDF-003 — Repository creation and transfer

- create `Crystonix/RunenSDF`;
- add governance, licensing, validation, and provenance;
- transfer the corrected package and tests;
- validate independently.

### PT-RUNENSDF-004 — Runenwerk cutover

- pin exact RunenSDF revision;
- migrate all consumers;
- delete `domain/sdf`;
- remove old workspace/lockfile entries;
- retain no compatibility package or source mirror;
- run full integration/runtime validation.

### PT-RUNENSDF-005 — Closeout

Record SHAs, compatibility, validation, provenance, deleted paths, and final
ownership; remove temporary branches and migration authority.

## Stop conditions

Stop and return to design when:

- mandatory local search finds a persisted/public format using current types;
- a production consumer requires unclassified material/channel payloads;
- GPU/shader code shares direct source authority with CPU field code;
- conservative operation bounds cannot be defined without changing semantics;
- singular-transform fallback is intentional product behavior;
- current `main` is not green for unrelated reasons;
- removal of geometry would require a universal shared-core repository;
- final cutover would retain duplicate or compatibility source.

## Definition of done

RunenSDF is complete only when:

- the independent repository validates on stable and MSRV;
- a public downstream consumer passes;
- RunenSDF contains no Runenwerk dependency;
- Runenwerk pins an exact revision and uses explicit adapters;
- all original `domain/sdf` source is deleted;
- no compatibility package, mirror, or moving dependency remains;
- full Runenwerk integration validation passes;
- provenance and current documentation are complete.
