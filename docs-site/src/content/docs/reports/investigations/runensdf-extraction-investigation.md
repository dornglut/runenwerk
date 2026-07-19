---
title: RunenSDF Extraction Investigation
description: Source, test, API, dependency, numerical, consumer, and cutover evidence for correcting and extracting domain/sdf.
status: active
owner: sdf
layer: investigation
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runensdf-002-boundary-correction.ron
  - ../../domain/sdf/README.md
  - ../../domain/sdf/api-notes.md
  - ../../domain/sdf/query-model.md
  - ../../domain/sdf/numerics.md
  - ../../domain/sdf/ownership-boundary.md
---

# RunenSDF Extraction Investigation

## Question

Can `domain/sdf` become an independently useful RunenSDF package, and what exact
boundary corrections are required before source transfer?

## Verdict

```text
EXTRACTION CANDIDATE                 yes
MOVE AS-IS                           no
SOURCE/API INVENTORY                 complete through connector inspection
CONSUMER/CARGO COMMAND VERIFICATION  pending local execution
BOUNDARY REPAIR SIZE                 bounded but API-breaking
INITIAL PACKAGE                      one runensdf crate
EXTRACTION ORDER                     first
SOURCE CHANGES AUTHORIZED HERE       no
NEXT PHASE                           PT-RUNENSDF-002 after activation gates
```

The package is small and host-neutral in intent. Its confirmed internal dependency
is Runenwerk `geometry`, exposed through bounds and ray queries. The implementation
also has weak validation, ambiguous query terminals, incorrect disjoint-bound
semantics, and no explicit contract separating exact distance, conservative
step-safe estimation, and sign-only fields.

## Baseline and evidence

Repository: `Crystonix/Runenwerk`

Reviewed published main:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

Historical origin inspected:

```text
e2fd2b9ebbcb4c3c78cc49714b24a2b11c7cc39a
feat(foundation): add sdf core crate with queries and tests
```

Evidence:

```text
E2 commit/package history
E3 current source, tests, manifests, docs, and selected consumer manifests
E4 Cargo.lock package facts
```

The GitHub connector cannot run Cargo or a reliable complete repository-wide
search. Consumer and executable baseline verification remain mandatory before
implementation activation.

## Package inventory

```text
path: domain/sdf
package: sdf
version: 0.1.0
edition: 2024
publish: false
dependencies: glam, geometry
features: none
build script: none
```

No package examples, benches, optional dependencies, or dev-dependencies are
declared.

## Source inventory

All current source modules were inspected.

### Core

```text
src/lib.rs
src/field.rs
src/sample.rs
src/bounds.rs
src/epsilon.rs
src/gradient.rs
src/normal.rs
```

### Primitives

```text
src/primitives/sphere.rs
src/primitives/box3.rs
src/primitives/capsule.rs
src/primitives/cylinder.rs
src/primitives/plane.rs
src/primitives/torus.rs
```

### Operations and composition

```text
src/ops/union.rs
src/ops/intersect.rs
src/ops/subtract.rs
src/ops/smooth_union.rs
src/ops/smooth_intersect.rs
src/ops/smooth_subtract.rs
src/combine/clamp.rs
src/combine/domain_warp.rs
src/combine/mirror.rs
src/combine/repeat.rs
```

### Transforms

```text
src/transform/translate.rs
src/transform/rotate.rs
src/transform/scale.rs
src/transform/affine.rs
```

### Queries and utilities

```text
src/queries/classify.rs
src/queries/closest_point.rs
src/queries/project.rs
src/queries/raymarch.rs
src/queries/sweep.rs
src/util/finite_difference.rs
```

## Test inventory

All nine integration-test files were inspected:

```text
tests/classification.rs
tests/gradient.rs
tests/normals.rs
tests/ops.rs
tests/primitives.rs
tests/project.rs
tests/raymarch.rs
tests/regression.rs
tests/transform.rs
```

Current tests cover ordinary primitive values, selected bounds, composition,
gradients/normals, projection, raymarch, sweep, transforms, and a small set of
degenerate cases.

Material gaps include:

- NaN/infinite inputs and samples;
- invalid constructors and public-field invariant bypass;
- singular transforms and invalid rotations/scales;
- disjoint intersection bounds;
- conservative smooth/warp bounds;
- exact-distance versus safe-step properties;
- no-overshoot sphere-tracing properties;
- unsupported tracing capability;
- distinct terminal outcomes;
- external downstream implementation;
- trait-object/`?Sized` use;
- property, allocation, MSRV, and benchmark evidence.

## Current public API

Crate-root spine:

```rust
pub use bounds::FieldBounds;
pub use field::SdfField3;
pub use sample::SdfSample;
```

Current contract:

```rust
pub trait SdfField3 {
    fn sample(&self, point: Vec3) -> SdfSample;
    fn bounds(&self) -> FieldBounds;
}
```

Current sample and bounds:

```text
SdfSample { pub distance: f32 }
FieldBounds::Unbounded
FieldBounds::Bounded(geometry::Aabb3)
```

Primitive, operation, composition, transform, and query-setting values expose
numerous public fields, allowing callers to bypass constructor policy.

Raymarch accepts `geometry::Ray3` plus positional policy parameters.

## Geometry coupling

Confirmed direct uses:

```text
FieldBounds stores Aabb3
primitive bounds construct Aabb3
operation bounds combine Aabb3
transform bounds map Aabb3 corners
DomainWarp and Mirror construct Aabb3
raymarch accepts Ray3
tests and docs use Aabb3/Ray3
```

The dependency is narrow enough to replace with RunenSDF-owned validated values.
Runenwerk owns conversion adapters after correction.

## Numerical findings

### Silent normalization

Current behavior silently normalizes or falls back for several invalid states,
including radii/extents/smoothness, clamp order, repeat periods, epsilon, sweep
steps/radius, zero scale, zero gradients, and affine distance scale.

These are not acceptable undocumented public semantics.

### Non-finite propagation

Sampling generally does not reject non-finite points or parameters. Classification
can treat NaN as on-surface because ordered comparisons are false. Gradients and
queries may propagate non-finite values.

### Bounds defect

Disjoint finite intersection currently returns the operands' union. That is broad
containment but incorrect intersection semantics and creates unnecessarily large
acceleration bounds.

The corrected model requires explicit `Empty`.

### Distance-quality defect

The current trait calls every sample a distance but does not state whether it is:

```text
exact Euclidean signed distance
conservative lower-bound distance estimator
arbitrary signed implicit value
sphere-tracing-safe step
```

Non-uniform affine transforms and domain warps do not automatically preserve
exact distance or safe sphere-tracing steps. Using `abs(distance)` for all fields
can overshoot and miss surfaces.

The corrected design therefore separates:

```text
signed_value  finite sign/value information
safe_step     optional proven non-negative lower bound to the zero set
```

Sphere tracing uses `safe_step` only.

### Query outcome collapse

Current `Option` results conflate ordinary miss, bounds exit, maximum distance,
step/convergence exhaustion, invalid settings/rays, non-finite evaluation, and
unsupported query capability.

The corrected design separates:

```text
Hit
Miss(terminal reason)
Error(invalid/evaluation/unsupported capability)
```

## Consumer evidence

No direct `sdf` dependency appears in the inspected manifests for:

```text
engine
world_sdf
procgen
material_graph
world_ops
```

Cargo.lock represents `sdf` as an isolated workspace package depending on
`geometry` and `glam`. The inspected SDF renderer example is engine/WGSL product
code rather than a consumer of the `sdf` package.

Current evidence supports:

```text
confirmed production manifest consumers  0
confirmed package test consumers          9 integration targets
confirmed documentation consumers         SDF domain docs
```

This conclusion remains provisional until local `cargo tree` and `rg` verification.
A newly found consumer is migrated through the corrected API; it does not change
repository ownership.

## Documentation findings

Current SDF docs correctly describe the intended engine-neutral direction and
sign convention but overstate maturity and do not define:

- validated construction;
- non-finite behavior;
- exact versus conservative sampling;
- sphere-tracing safe-step requirements;
- distinct query terminal outcomes;
- singular transform policy;
- independent release/API stability.

These documents must align with the corrected API before extraction and then move
to RunenSDF authority. Runenwerk retains integration and migration records only.

## Disposition matrix

| Current responsibility | Disposition |
|---|---|
| field trait and sample | redesign in place, then transfer |
| primitives | validate construction, prove sample/step behavior, then transfer |
| hard/smooth operations | prove sign/step/bounds semantics, then transfer |
| transforms | validate and prove exact/conservative factors, then transfer |
| repeat/mirror/clamp/warp | prove or remove safe-step capability, then transfer |
| gradients/normals | remove implicit fallback, then transfer |
| queries | structured settings/errors/terminal outcomes, then transfer |
| geometry AABB/ray dependency | replace with local values |
| Runenwerk geometry conversions | remain in Runenwerk adapter |
| world/render/material/product policy | remain outside RunenSDF |
| old package/source after cutover | delete |

## Mandatory local gate

Before activating `PT-RUNENSDF-002`, run:

```text
cargo metadata --format-version 1 --locked
cargo tree -p sdf
cargo tree -i sdf --workspace
rg -n '\bsdf\b|SdfField3|SdfSample|FieldBounds|SdfSphere|raymarch_first_hit' .
rg -n 'sdf\s*=|package\s*=\s*"sdf"' --glob Cargo.toml .
cargo +stable fmt --all --check
cargo test -p sdf --locked
cargo clippy -p sdf --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
git status --short --branch
```

Report every failure honestly. Do not infer command success from source inspection.

## Gate result

```text
source/test/API investigation        complete
architecture and numerical design    complete
consumer verification                pending
current executable baseline          pending
implementation authorization         blocked
external repository transfer         forbidden
```

## Next safe action

After the mandatory local gate and owner review pass, activate exactly one bounded
`PT-RUNENSDF-002` implementation phase to correct the API inside Runenwerk.
Do not create RunenSDF or delete `domain/sdf` during that phase.