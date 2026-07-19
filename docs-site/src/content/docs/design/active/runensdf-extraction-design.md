---
title: RunenSDF Extraction Design
description: Decision-complete public field, numerical, query, package, conformance, and clean-cutover design for extracting domain/sdf into RunenSDF.
status: active
owner: sdf
layer: domain/sdf
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runensdf-002-boundary-correction.ron
---

# RunenSDF Extraction Design

## Status

The target repository, ownership, field-sample semantics, query terminal model,
spatial bounds, package shape, and boundary-correction sequence are
**decision-complete**.

Implementation remains unauthorized until the local consumer/Cargo baseline and
documentation validation pass and `PT-RUNENSDF-002` is explicitly activated.

## Goal

Create `Crystonix/RunenSDF` as one independently useful CPU signed-field package
without Runenwerk geometry, world, ECS, renderer, material, or product
dependencies.

## Initial package

```text
repository: Crystonix/RunenSDF
package: runensdf
version: 0.1.0
edition: 2024
license: MIT OR Apache-2.0
publish: false until release gates pass
```

Do not create core, query, GPU, shader, program, or macro subpackages initially.

## Ownership

RunenSDF owns:

- signed-field evaluation and samples;
- repository-local bounds and rays;
- analytic primitives;
- hard and smooth composition;
- transforms and domain composition;
- gradients and normals;
- classification, projection, closest-point, ray, and sweep queries;
- validation and numerical policy;
- deterministic CPU conformance and benchmarks.

RunenSDF does not own:

- Runenwerk geometry;
- ECS components/resources;
- world chunks, streaming, scene, or product policy;
- material channels or renderer payloads;
- WGPU, shaders, renderer graphs, or GPU residency;
- stable persisted field/program formats.

## Core field contract

The package retains a small object-model-neutral field trait. The semantic shape
is fixed even if final accessor names are refined:

```rust
pub trait SdfField3 {
    fn sample(&self, point: glam::Vec3) -> Result<SdfSample, SampleError>;

    fn bounds(&self) -> FieldBounds {
        FieldBounds::Unbounded
    }
}

pub struct SdfSample {
    signed_value: f32,
    safe_step: Option<f32>,
}
```

### `signed_value`

`signed_value` is finite and preserves the sign convention:

```text
negative  inside
zero      on the zero set/surface
positive  outside
```

It is not universally promised to equal exact Euclidean distance. Exact
primitives and transformations that preserve exactness may document that stronger
property.

### `safe_step`

`safe_step` is either:

- `Some(s)`, where `s` is finite, non-negative, and no greater than the distance
  to the nearest zero-set crossing along any direction; or
- `None`, meaning the field supplies sign/value information but no proven
  sphere-tracing step bound at that point.

Exact SDF primitives normally return `abs(signed_value)` as the safe step.
Conservative estimators may return a smaller bound. A field must never return a
step larger than its proven lower bound to the surface.

Sphere tracing and any other no-overshoot algorithm consume `safe_step`, not
`signed_value` magnitude. Encountering `None` produces a structured unsupported
capability error; it is never treated as an ordinary miss and never falls back to
`abs(signed_value)`.

This model allows exact SDFs, conservative distance estimators, and sign-only
implicit fields to coexist without misrepresenting their query safety.

## Composition safety

Every built-in wrapper must define both sign/value and safe-step propagation.

Required rules:

- translation and validated rotation preserve exactness and safe steps;
- uniform scale multiplies both value and safe step by `abs(scale)`;
- affine/non-uniform transforms reduce safe steps using a proven conservative
  lower-bound factor derived from the inverse linear transform;
- hard boolean operations preserve a safe step only through a documented proof;
- smooth operations must prove their safe-step propagation or return `None`;
- domain warps require a proven Lipschitz/contraction bound to return `Some`;
- repeat and mirror preserve a safe step only where their mapping proof applies;
- clamp-like value operations that destroy metric meaning return `None` unless a
  safe bound is proven.

A built-in operation with no proof remains usable for sign-based operations but
is not sphere-tracing-capable.

## Geometry boundary

The current `geometry::Aabb3` and `geometry::Ray3` dependency is removed.

RunenSDF owns validated local values:

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

Invariants:

- all components are finite;
- `Bounds3::min <= max` component-wise;
- ray direction is finite and non-zero;
- ray direction is normalized once by validated construction or query admission;
- query distances are world-space distances;
- disjoint finite intersection is `Empty`;
- `Empty` and `Unbounded` are distinct.

Runenwerk owns conversions to and from its geometry types.

## Bounds algebra

```text
union(Unbounded, x)           = Unbounded
union(Empty, x)               = x
intersection(Empty, x)        = Empty
intersection(Unbounded, x)    = x
intersection(disjoint finite) = Empty
subtraction bounds            = left operand bounds
```

Finite bounds are conservative spatial acceleration facts. They do not imply
exact signed distance or safe-step capability.

Smooth operations expand bounds where required. Warps, repetition, or transforms
return `Unbounded` when finite containment cannot be proven.

## Validated construction

Primitive, operation-policy, composition, transform, bounds, ray, and settings
fields are private where public mutation would bypass invariants.

External construction rejects:

- NaN and infinity;
- negative dimensions where no explicit signed meaning exists;
- zero/invalid plane normals;
- invalid quaternions;
- zero/non-finite uniform scale;
- singular/non-finite affine transforms;
- invalid repeat periods or clamp ranges;
- non-positive/non-finite epsilon;
- invalid distance or iteration budgets.

Deliberate normalization requires an explicitly named constructor or method. It
must not occur invisibly during sampling.

## Sampling errors

`SampleError` is structured and covers at least:

```text
non-finite point
non-finite evaluation
invalid internal field state detected at evaluation
unsupported evaluation capability where applicable
```

Built-in values should make invalid internal state unrepresentable. Queries still
validate returned samples because downstream `SdfField3` implementations are
external code.

## Query model

Policy-heavy queries use validated settings values, including:

```text
RaymarchSettings
ProjectSettings
SweepSettings
```

The semantic result shape is:

```rust
pub enum QueryOutcome<T> {
    Hit(T),
    Miss(QueryTermination),
}

pub enum QueryTermination {
    OutsideBounds,
    SurfaceRuledOut,
    MaxDistanceReached,
    StepBudgetExhausted,
    ConvergenceBudgetExhausted,
}

pub enum QueryError {
    InvalidInput(...),
    Sample(SampleError),
    UnsupportedCapability(...),
}
```

Exact names and payload factoring may be refined, but these distinctions are
fixed:

- invalid admission/evaluation is an error;
- unsupported safe-step capability is an error;
- ordinary ruled-out/no-surface completion is a miss;
- leaving finite bounds is distinct;
- maximum-distance termination is distinct;
- step/convergence budget exhaustion is distinct;
- a hit is explicit.

Queries do not collapse all valid terminal states into `None`.

### Raymarching

Raymarching:

- uses `safe_step` only;
- validates every sample;
- never overshoots under the documented safe-step contract;
- reports unsupported capability immediately;
- records steps, terminal reason, and travelled world distance;
- handles inside-start policy explicitly in settings or documented behavior.

### Projection and closest point

Projection may use signed value and gradient information, but reports unusable
gradient and convergence exhaustion separately. It does not silently return a
fabricated result.

### Sweep

Sweep must prove how the radius modifies the safe step and bounds. Negative or
non-finite radius is invalid.

## Gradient and normal policy

Finite differences remain the initial CPU reference.

A positive finite epsilon is required. Non-finite samples and unusable zero
gradients are explicit failures in the primary API.

The primary normal API does not substitute `Vec3::Y`. A separately named debug
fallback helper may exist, but correctness-sensitive queries do not use it.

## Threading and allocation

RunenSDF owns no scheduler or executor. The base trait does not require
`Send + Sync`; consumers may impose stronger bounds.

Ordinary scalar sampling and query paths avoid allocation unless explicitly
documented and benchmarked.

## Serialization

No stable serialization format is included. Do not add serde for hypothetical
future authoring. A future persisted field/program design requires separate
schema, version, validation, deterministic encoding, and migration authority.

## Independent conformance

Before extraction, prove:

- valid and invalid construction;
- primitive sign and exact-distance behavior;
- finite/non-finite sampling;
- safe-step lower-bound properties;
- no-overshoot raymarch properties;
- explicit unsupported behavior for sign-only fields;
- spatial bound containment and `Empty` algebra;
- transform, smooth-operation, repeat, mirror, clamp, and warp safety;
- all query terminal outcomes;
- gradient/normal failures;
- downstream public field implementation;
- trait-object/`?Sized` use where accepted;
- use without Runenwerk;
- stable, MSRV, Clippy, property, and benchmark evidence.

## Cutover sequence

```text
PT-RUNENSDF-001 verify investigation and consumer baseline
PT-RUNENSDF-002 correct public/numerical boundary inside Runenwerk
PT-RUNENSDF-003 create RunenSDF and transfer corrected source
PT-RUNENSDF-004 cut Runenwerk over and delete domain/sdf
PT-RUNENSDF-005 close provenance, compatibility, and temporary authority
```

## Stop conditions

Stop when:

- local search finds an unclassified public or persisted consumer;
- a composition cannot provide correct sign or safe-step semantics;
- CPU and shader implementations share unclassified source authority;
- current main fails for unrelated reasons;
- geometry removal requires a universal shared-core repository;
- implementation needs external repository creation or source deletion during
  PT-RUNENSDF-002;
- implementation authorization remains planning-only.

## Definition of done

RunenSDF is complete only when it validates independently, has a downstream public
consumer, contains no Runenwerk dependency, Runenwerk consumes an exact revision,
original `domain/sdf` source is removed, no compatibility authority remains, and
full integration validation is green.