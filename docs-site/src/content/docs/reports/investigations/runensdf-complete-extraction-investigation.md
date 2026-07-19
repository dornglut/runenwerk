---
title: RunenSDF Complete Extraction Investigation
description: Complete source, test, public API, dependency, numerical, consumer, and cutover investigation for extracting domain/sdf into RunenSDF.
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
  - ../../domain/sdf/README.md
  - ../../domain/sdf/api-notes.md
  - ../../domain/sdf/query-model.md
  - ../../domain/sdf/numerics.md
  - ../../domain/sdf/ownership-boundary.md
---

# RunenSDF Complete Extraction Investigation

## Question

Can `domain/sdf` be extracted into an independently useful RunenSDF repository,
and what exact API and behavior corrections are required before source transfer?

## Verdict

```text
EXTRACTION CANDIDATE: yes
DIRECTORY MOVE AS-IS: no
BOUNDARY REPAIR SIZE: bounded
REPOSITORY SHAPE: one initial runensdf crate
EXTRACTION ORDER: first
SOURCE CHANGES AUTHORIZED BY THIS REPORT: no
NEXT PHASE: PT-RUNENSDF-002 boundary correction
```

The package is small, host-neutral in intent, and has no confirmed active
production manifest consumer. Its only package dependencies are `glam` and
Runenwerk `geometry`. The geometry dependency is public and concentrated in
bounds and ray-query vocabulary.

The algorithms are reusable, but the current public API is not suitable for an
independent pre-1.0 framework without repair. It exposes unchecked fields,
silently normalizes invalid parameters, conflates invalid input and ordinary
misses, and contains incorrect disjoint-intersection bounds behavior.

## Reviewed baseline

Repository: `Crystonix/Runenwerk`

Published main head reviewed:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

Repository-family charter branch used as planning base:

```text
docs/repository-family-charter
d14fc0e07ace3c2123ff70fc748b0694114cb6e1
```

Historical origin commit inspected:

```text
e2fd2b9ebbcb4c3c78cc49714b24a2b11c7cc39a
feat(foundation): add sdf core crate with queries and tests
```

## Evidence classes

```text
E2 Git commit and package-history metadata
E3 current source, manifests, tests, documentation, and likely-consumer manifests
E4 current lockfile package graph evidence
```

No local Cargo commands were available through the GitHub connector. Command
validation remains a mandatory activation gate for the implementation phase.

## Package inventory

Current manifest:

```text
path: domain/sdf/Cargo.toml
package: sdf
version: 0.1.0
edition: 2024
publish: false
```

Dependencies:

```text
glam = 0.31.0 in the manifest, resolved as 0.31.1 in Cargo.lock
geometry = path ../geometry
```

No features, build script, optional dependencies, dev-dependencies, examples, or
bench targets are declared in the package manifest.

## Complete source inventory

### Crate root and core

```text
domain/sdf/src/lib.rs
domain/sdf/src/field.rs
domain/sdf/src/sample.rs
domain/sdf/src/bounds.rs
domain/sdf/src/epsilon.rs
domain/sdf/src/gradient.rs
domain/sdf/src/normal.rs
```

### Primitives

```text
domain/sdf/src/primitives/mod.rs
domain/sdf/src/primitives/sphere.rs
domain/sdf/src/primitives/box3.rs
domain/sdf/src/primitives/capsule.rs
domain/sdf/src/primitives/cylinder.rs
domain/sdf/src/primitives/plane.rs
domain/sdf/src/primitives/torus.rs
```

### Hard and smooth operations

```text
domain/sdf/src/ops/mod.rs
domain/sdf/src/ops/union.rs
domain/sdf/src/ops/intersect.rs
domain/sdf/src/ops/subtract.rs
domain/sdf/src/ops/smooth_union.rs
domain/sdf/src/ops/smooth_intersect.rs
domain/sdf/src/ops/smooth_subtract.rs
```

### Composition wrappers

```text
domain/sdf/src/combine/mod.rs
domain/sdf/src/combine/clamp.rs
domain/sdf/src/combine/domain_warp.rs
domain/sdf/src/combine/mirror.rs
domain/sdf/src/combine/repeat.rs
```

### Transforms

```text
domain/sdf/src/transform/mod.rs
domain/sdf/src/transform/translate.rs
domain/sdf/src/transform/rotate.rs
domain/sdf/src/transform/scale.rs
domain/sdf/src/transform/affine.rs
```

### Queries

```text
domain/sdf/src/queries/mod.rs
domain/sdf/src/queries/classify.rs
domain/sdf/src/queries/closest_point.rs
domain/sdf/src/queries/project.rs
domain/sdf/src/queries/raymarch.rs
domain/sdf/src/queries/sweep.rs
```

### Utilities

```text
domain/sdf/src/util/mod.rs
domain/sdf/src/util/finite_difference.rs
```

All current source modules were inspected.

## Complete test inventory

```text
domain/sdf/tests/classification.rs
domain/sdf/tests/gradient.rs
domain/sdf/tests/normals.rs
domain/sdf/tests/ops.rs
domain/sdf/tests/primitives.rs
domain/sdf/tests/project.rs
domain/sdf/tests/raymarch.rs
domain/sdf/tests/regression.rs
domain/sdf/tests/transform.rs
```

Coverage exists for:

- ordinary primitive signed distances and selected bounds;
- hard and smooth operation samples;
- bounded/unbounded propagation;
- gradient and normal happy paths;
- projection convergence and one iteration-exhaustion case;
- raymarch hit/miss/max-distance/max-step/inside-start cases;
- transforms and selected bounds;
- degenerate capsule;
- zero-direction ray returning `None`;
- one sphere sweep;
- basic repeat/mirror/domain-warp finite sample behavior.

Coverage is missing or materially incomplete for:

- invalid and non-finite constructors;
- public-field invariant bypass;
- NaN/infinite point and parameter inputs;
- non-finite field samples;
- invalid settings versus valid miss/result exhaustion;
- zero/invalid plane normal admission;
- invalid quaternion and singular affine transforms;
- zero/non-finite scale admission;
- negative or non-finite primitive sizes;
- disjoint intersection bounds;
- smooth-operation conservative-bound expansion;
- domain-warp bound correctness under invalid amplitudes/frequencies;
- repeat period validation;
- clamp interval validation;
- property-based bound containment;
- external downstream field implementation;
- trait-object or `?Sized` use;
- allocation and benchmark evidence;
- MSRV and package-independent validation.

Several tests use `panic!` in branch assertions. The later denied-warning Clippy
policy must account for test-code policy deliberately rather than inheriting this
shape accidentally.

## Documentation inventory

Canonical current SDF documentation:

```text
docs-site/src/content/docs/domain/sdf/README.md
docs-site/src/content/docs/domain/sdf/index.md
docs-site/src/content/docs/domain/sdf/api-notes.md
docs-site/src/content/docs/domain/sdf/query-model.md
docs-site/src/content/docs/domain/sdf/numerics.md
docs-site/src/content/docs/domain/sdf/ownership-boundary.md
docs-site/src/content/docs/domain/sdf/implementation-roadmap.md
```

The documentation correctly states the intended engine-neutral boundary and sign
convention, but it overstates maturity:

- it calls the vocabulary stable without a release/API-stability policy;
- the implementation roadmap claims a “mature reusable SDF foundation”;
- the quick example exposes the geometry dependency as normal API;
- it documents defensive gradient fallbacks but not their semantic cost;
- it does not distinguish invalid query input from no result;
- the roadmap structure lists a crate README that is no longer present at
  `domain/sdf/README.md` after documentation migration;
- no current document records constructor, non-finite, singular-transform, or
  error semantics.

These documents require migration to RunenSDF current authority during source
transfer. Runenwerk should retain only integration and historical migration
records afterward.

## Public API inventory

### Crate-root re-exports

```rust
pub use bounds::FieldBounds;
pub use field::SdfField3;
pub use sample::SdfSample;
```

Modules are all public, including `util` and low-level finite-difference helpers.

### Core contract

```rust
pub trait SdfField3 {
    fn sample(&self, point: Vec3) -> SdfSample;
    fn bounds(&self) -> FieldBounds;
}
```

Current default bounds are unbounded.

### Core values

```text
SdfSample { pub distance: f32 }
FieldBounds::Unbounded
FieldBounds::Bounded(geometry::Aabb3)
```

`FieldBounds` exposes geometry through enum payload and accessors.

### Primitive values

```text
SdfSphere { pub center, pub radius }
SdfBox3 { pub center, pub half_extents }
SdfCapsule { pub start, pub end, pub radius }
SdfCylinder { pub center, pub radius, pub half_height }
SdfPlane { pub normal, pub distance }
SdfTorus { pub center, pub major_radius, pub minor_radius }
```

Every primitive has an infallible constructor and public fields that bypass
constructor policy.

### Operations

```text
Union<A, B> { pub left, pub right }
Intersect<A, B> { pub left, pub right }
Subtract<A, B> { pub left, pub right }
SmoothUnion<A, B> { pub left, pub right, pub smoothness }
SmoothIntersect<A, B> { pub left, pub right, pub smoothness }
SmoothSubtract<A, B> { pub left, pub right, pub smoothness }
```

### Composition

```text
ClampDistance<F> { pub field, pub min_distance, pub max_distance }
DomainWarp<F> { pub field, pub amplitude, pub frequency, pub phase }
MirrorAxes { pub x, pub y, pub z }
Mirror<F> { pub field, pub axes }
Repeat<F> { pub field, pub period }
```

### Transforms

```text
Translate<F> { pub field, pub offset }
Rotate<F> { pub field, pub rotation, private inverse }
Scale<F> { pub field, pub scale }
Affine<F> { pub field, pub transform, private inverse/distance_scale }
```

### Query settings and results

```text
PointClassification
ClosestPointHit
ProjectSettings { public fields }
ProjectHit
RayHit
SweepSettings { public fields }
SweepHit
```

Raymarch uses positional `max_steps`, `max_distance`, and `epsilon` parameters
plus `geometry::Ray3`.

### Public constants

```text
DEFAULT_SAMPLE_EPSILON
DEFAULT_NORMAL_EPSILON
DEFAULT_RAY_HIT_EPSILON
DEFAULT_PROJECT_EPSILON
DEFAULT_CLASSIFY_EPSILON
DEFAULT_MAX_RAYMARCH_STEPS
DEFAULT_MAX_PROJECT_STEPS
```

`DEFAULT_SAMPLE_EPSILON` is public but no inspected source path uses it.

## Dependency and consumer findings

### Direct dependency

The only direct internal dependency is Runenwerk `geometry`.

Confirmed uses:

```text
FieldBounds stores Aabb3
primitive bounds construct Aabb3
operation bounds combine Aabb3 through FieldBounds
transform bounds map Aabb3 corners
DomainWarp and Mirror construct Aabb3
raymarch accepts Ray3
SDF tests use Aabb3 and Ray3
SDF docs use geometry::Ray3
```

### Production consumers

No direct `sdf` dependency appears in the inspected current manifests for:

```text
engine
domain/world_sdf
domain/procgen
domain/material_graph
domain/world_ops
```

The root workspace does not expose `sdf` as a workspace dependency. Cargo.lock
contains `sdf` as an isolated workspace package with only `geometry` and `glam`
dependencies.

The SDF-named renderer example inspected is an engine/WGSL render product and
uses engine renderer contracts rather than importing the `sdf` package.

Current evidence therefore supports:

```text
CONFIRMED PRODUCTION MANIFEST CONSUMERS: 0
CONFIRMED IN-PACKAGE TEST CONSUMERS: 9 integration test targets
CONFIRMED DOCUMENTATION CONSUMERS: SDF domain docs/quick example
```

Because GitHub code search repeatedly timed out and the connector cannot run a
repository-wide local grep, the implementation activation must still run exact
source searches before changing APIs:

```text
rg -n '\bsdf\b|SdfField3|SdfSample|FieldBounds|SdfSphere|raymarch_first_hit' .
rg -n 'sdf\s*=|package\s*=\s*"sdf"' --glob Cargo.toml .
cargo tree -i sdf --workspace
```

This is a verification gate, not an unresolved architecture decision. Any newly
found consumer is migrated through the same corrected public API and does not
change repository ownership.

## Numerical and semantic findings

### Unchecked values

Public fields and infallible constructors admit NaN, infinity, negative extents,
zero directions, zero/singular transforms, and invalid settings.

### Silent normalization

Current behavior silently:

- clamps radii/extents/smoothness to zero;
- swaps clamp interval endpoints;
- treats zero repeat period as no repeat on that axis;
- clamps invalid epsilon to defaults or `f32::EPSILON`;
- converts zero sweep steps to one;
- converts negative sweep radius to zero;
- treats zero scale as a constant sample of the source at the origin;
- substitutes `Vec3::Y` for zero gradient;
- substitutes distance-scale `1.0` for non-finite affine scale analysis.

These are implementation fallbacks, not acceptable undocumented authored-input
semantics.

### Non-finite propagation

Primitive and wrapper samples generally do not reject non-finite points or
parameters. Gradients may propagate non-finite differences. Classification may
classify NaN as `OnSurface` because both comparisons are false.

### Query outcome collapse

Queries commonly return `Option`, collapsing:

- valid miss;
- valid convergence/step exhaustion;
- invalid ray/settings;
- non-finite field result;
- invalid or unusable gradient.

The corrected API must distinguish invalid/evaluation failure from ordinary
valid no-result outcomes.

### Bounds defect

For two disjoint bounded operands, `FieldBounds::intersection` currently returns
their union. This is conservative in the broad containment sense but semantically
misrepresents intersection and causes an unnecessarily maximal bound.

The corrected model needs an explicit empty bound or an accepted precise
alternative. Returning the union is rejected.

### Smooth bounds

Smooth union/intersection currently reuse hard-operation bounds without reviewing
whether the smoothing radius can expand the represented surface. This must be
proved operation by operation. If a finite conservative bound cannot be proven,
return unbounded or expand correctly.

### Transform admission

Quaternion normalization and affine inversion are unchecked. Singular/non-finite
transforms must be rejected before a transform wrapper becomes valid.

## Design decisions

The investigation accepts these decisions for `PT-RUNENSDF-002`.

### Package

```text
one package: runensdf
no initial macro/GPU/shader/program/query subcrates
```

### Geometry

RunenSDF owns validated repository-local:

```text
Bounds3
FieldBounds
Ray3
```

Runenwerk owns conversions to/from its geometry domain.

### Bounds state

Use an explicit finite-bound state capable of representing empty/disjoint
results. Accepted target:

```rust
pub enum FieldBounds {
    Unbounded,
    Empty,
    Bounded(Bounds3),
}
```

`Empty` means the field has no possible surface/interior under the represented
operation. It is distinct from unknown/unbounded.

Rules:

- union with unbounded is unbounded;
- union with empty returns the other operand;
- intersection with empty is empty;
- intersection of disjoint finite bounds is empty;
- intersection with unbounded returns the finite/empty operand;
- operations lacking a conservative finite proof return unbounded;
- subtraction retains the left bound;
- smoothing expands bounds when mathematically required.

### Invariant-preserving construction

Primitive, transform, wrapper, ray, bounds, and settings fields become private
where direct mutation bypasses invariants.

Fallible constructors reject invalid external values. Deliberate normalization
requires an explicitly named constructor or method; it is not hidden in
`sample()`.

### Sampling

`SdfSample` remains signed distance only for the initial release. No material,
channel, or provenance payload is added.

The normal sampling path remains lightweight. Field implementations are expected
to return finite distance for finite valid points. Query/evaluation APIs detect
and report violation rather than silently reinterpret it.

### Query outcomes

Use validated settings and structured query errors. The exact type design should
follow this semantic shape:

```rust
Result<Option<Hit>, QueryError>
```

where:

- `Err` means invalid input/settings or invalid field evaluation;
- `Ok(None)` means a valid query produced no hit/convergence within limits;
- `Ok(Some(hit))` means success.

Classification similarly returns a result when evaluation can fail.

### Normal estimation

Zero/invalid gradient does not silently become world-up in the primary API.
Return an explicit failure or optional normal. A named fallback helper may exist
for visualization-only consumers if clearly documented.

### Trait flexibility

Query/helpers should accept `F: SdfField3 + ?Sized` where no sized value is
required, allowing trait-object and borrowed custom-field use.

### Serialization

No stable serialization is promised. No serde dependency is added in this
extraction.

### Error vocabulary

Use one small structured error family for invalid scalar/vector/transform/settings
construction and query evaluation. Do not expose `anyhow` in the package API.

## Move/stay/redesign/delete matrix

| Current responsibility | Disposition | Final owner |
|---|---|---|
| Field/sample trait and sign convention | Move after API repair | RunenSDF |
| Primitive algorithms | Move after validated constructors | RunenSDF |
| Hard/smooth operations | Move after bounds/parameter review | RunenSDF |
| Composition wrappers | Move after validation/bounds review | RunenSDF |
| Transforms | Move after finite/invertible admission | RunenSDF |
| Gradients/normals | Move after explicit failure policy | RunenSDF |
| Point/ray/project/sweep queries | Move after settings/error redesign | RunenSDF |
| Epsilon defaults | Move after typed/settings review | RunenSDF |
| `geometry::Aabb3`/`Ray3` public use | Delete from framework API | Runenwerk adapter conversions only |
| World/chunk/residency policy | Stay | Runenwerk |
| Renderer/WGSL SDF product paths | Stay | Runenwerk render adapter |
| ECS integration | Stay | Runenwerk adapter |
| Existing SDF domain docs | Migrate/rewrite | RunenSDF; Runenwerk keeps migration history |
| Existing tests | Migrate and expand | RunenSDF |
| Original `domain/sdf` source after cutover | Delete | none |
| Compatibility package `sdf` | Do not create | none |

## Principle review

### KISS

One initial package and a small explicit bounds/ray/settings vocabulary.

### DRY

Remove dependency on Runenwerk geometry and keep conversion in one adapter. Do
not duplicate CPU field algorithms in renderer or world code.

### YAGNI

No GPU, shader, material payload, graph, serialization, macro, or multi-scalar
abstraction without consumer evidence.

### SOLID and separation of concerns

RunenSDF owns implicit field mathematics. Runenwerk owns world, ECS, rendering,
and product integration.

### Avoid premature optimization

CPU reference algorithms and benchmarks precede specialized acceleration or SIMD
contracts.

### Law of Demeter

Consumers depend on RunenSDF public values rather than Runenwerk geometry or
internal modules.

## PT-RUNENSDF-002 implementation contract

The next phase is a boundary correction inside Runenwerk. It must not create the
external repository yet.

Allowed scope:

```text
domain/sdf/Cargo.toml
domain/sdf/src/**
domain/sdf/tests/**
Runenwerk consumers discovered by mandatory grep/cargo tree
one explicit Runenwerk geometry adapter module and focused tests
domain/sdf current docs required to align corrected API
Cargo.lock only for dependency removal if Cargo updates it
phase-specific proof/report/planning files
```

Required implementation outcomes:

1. Rename package imports/API toward `runensdf` only if the in-workspace package
   rename is accepted as part of the mechanical future cutover; otherwise defer
   package rename until transfer. No compatibility alias.
2. Add validated `Bounds3`, `FieldBounds::Empty`, and repository-local `Ray3`.
3. Remove `geometry` from `domain/sdf/Cargo.toml`.
4. Make externally authored scalar/vector/transform/settings values invariant
   preserving.
5. Correct disjoint intersection and review every operation/wrapper bound.
6. Replace raymarch positional policy with validated settings.
7. Make invalid input/evaluation distinct from valid miss/exhaustion.
8. Remove primary normal fallback to `Vec3::Y`.
9. Support `?Sized` field references where valid.
10. Preserve the simple `SdfField3` and signed-distance model.
11. Add public downstream conformance and comprehensive negative/property tests.
12. Migrate all discovered consumers through explicit Runenwerk conversions.
13. Keep renderer/world/ECS/GPU work out of the framework package.

Forbidden scope:

```text
external repository creation
RunenSDF Git dependency
source deletion from Runenwerk
GPU/shader implementation
renderer changes unrelated to API migration
world_sdf/procgen redesign
ECS extraction
RunenRender extraction
RunenUI work
new shared foundation/meta crates
serialization format
material/channel payload expansion
```

Mandatory local preflight:

```text
git status --short --branch
git rev-parse HEAD
cargo metadata --format-version 1 --locked
cargo tree -p sdf
cargo tree -i sdf --workspace
rg -n '\bsdf\b|SdfField3|SdfSample|FieldBounds|SdfSphere|raymarch_first_hit' .
rg -n 'sdf\s*=|package\s*=\s*"sdf"' --glob Cargo.toml .
cargo test -p sdf --locked
cargo clippy -p sdf --all-targets --locked -- -D warnings
```

Required final validation:

```text
cargo +stable fmt --all --check
cargo test -p sdf --locked
cargo clippy -p sdf --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
git diff --check main...HEAD
git status --short --branch
```

Use the repository-authoritative MSRV command if one exists at activation time.

## Stop conditions

Stop implementation and return to design if:

- mandatory grep finds a persisted or public external format using current SDF
  types;
- a production consumer requires material/channel payloads in `SdfSample`;
- GPU/shader source is generated from or shares direct code authority with this
  package;
- operation bounds cannot be made conservative without changing field semantics;
- singular transform behavior is relied upon as intentional product behavior;
- local `main` is not green for unrelated reasons;
- removing geometry requires a new shared geometry repository;
- a compatibility package would be required after the final cutover.

## Gate status

```text
Complete source inspection: complete
Complete test inspection: complete
Public API inventory: complete
Manifest/dependency inspection: complete
Documentation inspection: complete
Consumer inspection: complete at manifest/lockfile level; mandatory local grep verification remains
Move/stay/redesign/delete map: complete
Target architecture: complete
Implementation contract: complete
Local command validation: not run; connector limitation
Complete investigation gate: complete subject to local verification commands
Complete design gate: complete for PT-RUNENSDF-002 subject to local baseline
External extraction authorization: blocked until PT-RUNENSDF-002 passes
```

## Next action

Run the mandatory local verification and documentation validation. After owner
review and merge of the repository-family charter and this investigation, open
one bounded `PT-RUNENSDF-002` boundary-correction implementation PR from current
`main`.
