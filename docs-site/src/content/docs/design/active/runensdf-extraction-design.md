---
title: RunenSDF Extraction Design
description: Decision-complete public field, numerical, repository, conformance, provenance, and clean-cutover design for extracting domain/sdf into runen-sdf.
status: active
owner: sdf
layer: domain/sdf
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ./runensdf-repository-identity-decision.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runensdf-003-standalone-transfer.ron
---

# RunenSDF Extraction Design

## Status

The repository identity, ownership boundary, public API, numerical semantics,
source-transfer baseline, standalone conformance, validation authority, and clean
cutover sequence are decision-complete.

`PT-RUNENSDF-002` completed the boundary correction in Runenwerk through merge
commit `8de096259eab30f8d67672010df9190970d0bfc4`.

`PT-RUNENSDF-003` is authorized to create and prove the standalone repository.
It may not change Runenwerk dependencies, remove `domain/sdf`, or introduce a
compatibility package. Those operations belong to `PT-RUNENSDF-004`.

## Canonical identity

```text
product: RunenSDF
repository: Crystonix/runen-sdf
package: runen-sdf
crate: runen_sdf
version: 0.1.0
edition: 2024
minimum supported Rust version: 1.93.0
license: MIT OR Apache-2.0
publication: disabled until a separate release gate
```

The former `Crystonix/RunenSDF` and `runensdf` spellings are superseded. No
compatibility alias preserves them.

## Repository shape

```text
runen-sdf/
├── Cargo.toml                 root public package
├── Cargo.lock                 committed independent lockfile
├── LICENSE-MIT
├── LICENSE-APACHE
├── SECURITY.md
├── README.md
├── src/                       transferred public implementation
├── tests/                     all nine transferred integration-test modules
├── conformance/downstream/    public downstream consumer proof
├── docs/                      framework-owned design and numerical authority
├── xtask/                     maintained repository validation authority
└── .github/workflows/         durable CI invoking cargo validate
```

There is no `crates/runen-sdf`, façade package, compatibility package, source
submodule, private source include, or speculative public package split.

## Ownership

RunenSDF owns:

- signed-field evaluation, samples, and field capabilities;
- validated local bounds and rays;
- analytic primitives;
- hard and smooth composition;
- transforms and domain composition;
- gradients and normals;
- classification, projection, closest-point, ray, and sweep queries;
- validation, numerical policy, deterministic CPU conformance, and benchmarks
  when separately governed.

RunenSDF does not own:

- Runenwerk adapters or product policy;
- ECS components, resources, scheduling, or world storage;
- scene, chunk, streaming, or procgen orchestration;
- material channels or renderer payloads;
- GPU resources, WGPU, shaders, render graphs, or UI integration;
- stable persisted field/program formats.

Runenwerk owns explicit adapters and cross-domain integration. An adapter may
depend on RunenSDF and Runenwerk contracts, but RunenSDF never depends on the
adapter or Runenwerk.

## Public field contract

```rust
pub trait SdfField3 {
    fn sample(&self, point: glam::Vec3) -> Result<SdfSample, SampleError>;

    fn bounds(&self) -> FieldBounds {
        FieldBounds::Unbounded
    }

    fn capabilities(&self) -> FieldCapabilities;
}
```

`SdfSample` contains finite signed-value information and an optional proven
conservative tracing step:

```text
signed_value  finite; negative inside, zero on the zero set, positive outside
safe_step     absent or finite, non-negative, and no larger than a proven lower
              bound to the nearest zero-set crossing
```

`signed_value` is not universally Euclidean distance. Sphere tracing consumes
`safe_step` only. Projection, closest-point, and sphere-sweep operations that
interpret value magnitude metrically require exact-distance capability.
Unsupported capabilities are structured errors, not ordinary misses or implicit
fallbacks.

## Capability propagation

- analytic primitives expose exact-distance and conservative-step capability;
- translation and validated rotation preserve both;
- uniform scale preserves exactness while scaling value and step by `abs(scale)`;
- affine and non-uniform transforms expose only a proven conservative step and
  remove exact-distance capability;
- hard boolean operations preserve only capability supported by an explicit proof;
- smooth operations, clamps, warps, repetition, and mirrors remove unproven metric
  capability;
- no wrapper may infer a tracing step from `abs(signed_value)` without proof.

The standalone transfer preserves these accepted PT-RUNENSDF-002 rules exactly.

## Validated geometry boundary

RunenSDF owns:

```rust
pub struct Bounds3 { /* private validated state */ }
pub enum FieldBounds { Unbounded, Empty, Bounded(Bounds3) }
pub struct Ray3 { /* finite origin and normalized non-zero direction */ }
```

Required invariants:

- all authored components are finite;
- `Bounds3::min <= max` component-wise;
- disjoint finite intersection is `Empty`;
- `Empty` and `Unbounded` remain distinct;
- ray direction is finite, normalized, and non-zero;
- query distance is world-space distance;
- invalid or singular authored state is rejected rather than silently normalized.

Runenwerk conversions remain adapter code outside RunenSDF.

## Query and failure model

Policy-heavy queries use validated settings values. Query completion distinguishes:

```text
Hit(value)
Miss(OutsideBounds | SurfaceRuledOut | MaxDistanceReached |
     StepBudgetExhausted | ConvergenceBudgetExhausted)
Error(InvalidInput | Sample | Gradient | UnsupportedCapability)
```

Queries never collapse invalid evaluation, unsupported capability, ordinary miss,
bounds exit, distance limits, and budget exhaustion into `Option`.

Primary normal estimation reports unusable gradients and never fabricates
`Vec3::Y`.

## Transfer baseline and parity rule

The only source baseline for PT-RUNENSDF-003 is:

```text
repository: Crystonix/runenwerk
commit: 8de096259eab30f8d67672010df9190970d0bfc4
source path: domain/sdf/src/**
test path: domain/sdf/tests/**
package manifest: domain/sdf/Cargo.toml
```

The transfer may change only:

- package identity `sdf` to `runen-sdf`;
- crate imports `sdf` to `runen_sdf`;
- repository metadata and documentation links;
- framework documentation ownership and provenance;
- downstream conformance and validation integration.

The transfer must not change numerical behavior, public semantics, module grouping,
query policy, capability propagation, error meaning, or test intent.

The accepted module layout remains in place until standalone parity is green.
Combining `combine` and `ops`, renaming `transform`, or introducing a differential
module is a later behavior-preserving review.

## Source and document disposition

| Current Runenwerk material | PT-003 disposition | PT-004 disposition |
|---|---|---|
| `domain/sdf/src/**` | transfer unchanged except accepted identity/import edits | delete original source |
| `domain/sdf/tests/**` | transfer all nine modules | delete original tests |
| public API and numerical docs | rewrite or migrate to standalone authority | delete stale internal copies |
| extraction investigation | remain in Runenwerk | retain as historical evidence |
| PT-002 closeout | remain in Runenwerk | retain as historical evidence |
| Runenwerk conversion/integration docs | remain in Runenwerk | update for external package |
| product, ECS, world, material, renderer code | never transfer | remain Runenwerk-owned |

The transferred package test modules are:

```text
classification
gradient
normals
ops
primitives
project
raymarch
regression
transform
```

## Independent downstream conformance

The downstream package must use only public APIs and prove at least:

- an external `SdfField3` implementation;
- valid and invalid construction;
- public sampling and field capabilities;
- trait-object and accepted `?Sized` use;
- tracing rejection for a sign-only field;
- one successful public query path;
- use without Runenwerk, private source inclusion, or external path dependencies.

## Validation authority

`cargo validate` is the single maintained validation command. GitHub Actions calls
that command rather than duplicating the validation sequence.

The validation envelope includes:

- locked metadata and dependency trees;
- formatting;
- all workspace tests and downstream conformance;
- Clippy for all targets with denied warnings;
- rustdoc with denied warnings;
- Rust 1.93.0 tests;
- committed-lockfile enforcement;
- dependency-direction and external-path checks;
- rejection of Runenwerk references, source includes, forwarding packages, and
  stale package identities;
- license, security, provenance, and documentation-link checks;
- diff hygiene and clean generated state.

CI must not generate or mutate `Cargo.lock`.

## Coordinated cutover

Temporary duplicate source may exist only while the standalone and Runenwerk
cutover branches are unmerged and coordinated.

The sequence is:

```text
PT-RUNENSDF-003
  1. transfer and validate the standalone implementation;
  2. record the exact accepted standalone commit;
  3. leave Runenwerk source and dependencies unchanged.

PT-RUNENSDF-004
  4. repeat the Runenwerk consumer audit;
  5. pin every real consumer to the exact standalone commit;
  6. migrate imports and adapters;
  7. remove domain/sdf from the workspace and lockfile;
  8. delete domain/sdf source, tests, and stale framework docs;
  9. prove no forwarding package, alias, source include, submodule, or duplicate
     implementation remains;
 10. validate the complete Runenwerk integration.
```

If the final consumer audit still finds no production consumer, PT-RUNENSDF-004
removes the isolated internal package without adding an unused external dependency.

No moving branch dependency is allowed. If the accepted standalone commit changes,
the Runenwerk pin and all integration evidence must be updated and rerun.

## Stop and rollback conditions

Stop the transfer when:

- a behavior or numerical-contract change becomes necessary;
- an unclassified consumer, persisted format, shader authority, dependency cycle,
  or private reach-through is found;
- standalone validation requires Runenwerk or an external path dependency;
- CI cannot allocate a runner or produce command evidence;
- source parity cannot be explained by the declared transfer edits.

Rollback before PT-RUNENSDF-004 is simple: close or revert the unmerged standalone
transfer branch. PT-RUNENSDF-003 never mutates Runenwerk source authority.

## Completion

PT-RUNENSDF-003 completes when one exact standalone commit passes GitHub Actions,
all nine package tests and downstream conformance pass, repository policy proves
independence, provenance is closed, and PT-RUNENSDF-004 can consume that exact
revision.

RunenSDF extraction completes only after PT-RUNENSDF-004 deletes the original
implementation and proves no compatibility or duplicate source authority remains.