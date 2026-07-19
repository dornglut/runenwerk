---
title: RunenSDF Extraction Design
description: Provisional ownership, numerical gates, package shape, conformance, and clean-cutover sequence for extracting domain/sdf into RunenSDF.
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

The repository mission and extraction order are fixed. The public numerical and
query contracts are not yet implementation-authorized.

Implementation remains blocked until:

- every current source and test file is inspected;
- all consumers are verified through local search and Cargo metadata;
- the field-distance and safe-step contract is decided;
- query terminal outcomes are decided;
- current package and workspace validation pass;
- a bounded implementation phase is activated.

## Goal

Extract reusable signed-field mathematics into an independent host-neutral
repository without carrying Runenwerk geometry, world, ECS, renderer, material,
or product ownership into the new package.

## Initial package shape

Start with one package:

```text
repository: Crystonix/RunenSDF
package: runensdf
initial version: 0.1.0
edition: 2024
publish: false until release gates pass
```

Do not create core, query, GPU, shader, program, or macro subpackages without
independent dependency or release pressure.

## Ownership

RunenSDF is expected to own:

- field evaluation vocabulary;
- spatial bounds and rays required by field queries;
- analytic primitives;
- hard and smooth composition;
- transforms and domain composition;
- gradients and normals;
- classification, projection, closest-point, ray, and sweep queries;
- numerical policy and CPU reference conformance.

It does not own:

- Runenwerk geometry;
- ECS components/resources;
- world chunks or streaming;
- scene nodes or product state;
- materials or material payloads;
- renderer graphs, WGPU, shaders, or GPU residency;
- stable persisted field/program formats.

## Geometry boundary

The public `geometry::Aabb3` and `geometry::Ray3` dependency is removed before
source transfer.

RunenSDF owns validated local bounds and ray values. The bounds model must
distinguish:

```text
Unbounded
Empty
Bounded(Bounds3)
```

Disjoint finite intersection is `Empty`, not the operands' union.

Runenwerk owns conversions between RunenSDF and Runenwerk geometry. Those
conversions contain no SDF algorithms.

## Required numerical decision

The current `SdfField3::sample -> distance` shape does not state whether every
field returns:

- exact Euclidean signed distance;
- a conservative signed distance estimate;
- an arbitrary implicit field value;
- a value with a separate safe sphere-tracing step bound.

This must be decided before implementation. Non-uniform affine transforms, domain
warps, smooth operations, and downstream custom fields cannot automatically be
assumed to preserve exact signed-distance magnitude or a safe raymarch step.

The accepted design must provide one of these capabilities explicitly:

```text
exact signed distance
conservative safe step bound
query capability declaration
unsupported-query outcome
```

A field that cannot prove a safe step must not silently participate in sphere
tracing as though it were exact.

The final API may use a richer sample, separate sampling methods, capability
metadata, or another reviewed shape. This charter does not freeze the method
names.

## Admission and value invariants

Public authored values must preserve invariants through validated construction.
The design must explicitly decide handling for:

- NaN and infinity;
- negative or zero dimensions;
- invalid plane normals;
- invalid quaternions;
- zero/non-finite uniform scale;
- singular/non-finite affine transforms;
- invalid repeat periods and clamp ranges;
- invalid epsilon and query budgets.

Silent normalization inside `sample()` is not a public contract.

## Query outcomes

Invalid admission and invalid/non-finite field evaluation are errors.

Valid query termination must not collapse all non-hit cases into one `None`.
The final model must distinguish at least where relevant:

```text
surface hit
surface ruled out / ordinary miss
left finite bounds
maximum distance reached
step budget exhausted
convergence budget exhausted
query unsupported by field capability
```

Exact type names are decided by the complete RunenSDF design. Query APIs must
preserve enough terminal information for diagnostics and deterministic tests.

## Gradient and normal policy

Finite differences remain the initial CPU reference unless investigation proves a
better owned contract.

The primary API must not silently substitute a world-up vector for an unusable
zero or non-finite gradient. Any visualization fallback is explicitly named and
must not be used by correctness-sensitive queries.

## Bounds policy

Bounds are conservative acceleration facts. They do not prove exact distance or
query capability.

Required properties include:

- every finite bound contains the represented surface under documented domain
  assumptions;
- `Empty` and `Unbounded` are distinct;
- smooth operations account for any surface expansion;
- transforms map bounds conservatively;
- warps/repetition return `Unbounded` when finite containment cannot be proven;
- no-overshoot/safe-step properties are tested separately from spatial bounds.

## Serialization and threading

No stable serialization format is promised initially. Rust layout is not a
persistence format.

RunenSDF owns no scheduler or executor. The base field contract does not acquire
`Send + Sync` merely for hypothetical consumers.

## Conformance requirements

Before extraction, RunenSDF must prove:

- valid and invalid construction;
- primitive sign behavior;
- finite/non-finite evaluation handling;
- conservative spatial bounds;
- exact-distance or safe-step properties for supported compositions;
- no-overshoot behavior for sphere-tracing-capable fields;
- explicit unsupported behavior for fields lacking that capability;
- distinct query terminal outcomes;
- transform and warp edge cases;
- downstream public field implementation;
- trait-object or `?Sized` use where accepted;
- use without Runenwerk;
- stable, MSRV, Clippy, property, and benchmark evidence.

## Sequence

```text
SDF-001 verify complete source/test/consumer investigation and close numerical design
SDF-002 correct the boundary inside Runenwerk
SDF-003 create RunenSDF and transfer corrected source
SDF-004 cut Runenwerk over and delete domain/sdf
SDF-005 close provenance, compatibility, and temporary authority
```

Only SDF-001 is active after the repository-family charter. Later phases require
separate bounded authorization.

## Stop conditions

Stop before implementation when:

- consumer inventory is incomplete;
- the safe-step/distance contract remains unresolved;
- a public or persisted consumer requires an unclassified capability;
- GPU/shader code shares direct source authority with the CPU package;
- conservative bounds or query safety cannot be specified;
- current main fails for unrelated reasons;
- extraction would require a shared-core repository or long-lived duplicate
  package.

## Definition of done

RunenSDF is extracted only when it validates independently, contains no Runenwerk
dependency, has a public downstream consumer, Runenwerk consumes an exact revision,
all original `domain/sdf` implementation is removed, and the complete Runenwerk
integration validation is green.