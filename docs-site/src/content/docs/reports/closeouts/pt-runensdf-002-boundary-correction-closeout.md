---
title: PT-RUNENSDF-002 Boundary Correction Closeout
description: Completion evidence for the in-workspace RunenSDF boundary correction.
status: completed
owner: sdf
layer: reports
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/specs/pt-runensdf-002-boundary-correction.ron
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
---

# PT-RUNENSDF-002 Boundary Correction Closeout

ID: `PT-RUNENSDF-002`

Title: `RunenSDF Boundary Correction`

Completed on: 2026-07-20

Implementation PR: `#116 — Correct RunenSDF boundaries and query contracts`

## Scope delivered

The phase corrected `domain/sdf` before external repository creation:

- removed the Runenwerk `geometry` dependency;
- added validated SDF-owned `Bounds3`, `FieldBounds`, and normalized `Ray3`;
- separated finite signed field value from optional conservative `safe_step`;
- added explicit exact-distance capability;
- made primitive, composition-policy, transform, ray, bounds, and query-setting
  construction invariant-preserving;
- introduced structured validation, sampling, gradient, and query errors;
- replaced ambiguous query `Option` results with explicit hit and terminal reasons;
- migrated all nine SDF package test modules;
- retained one source authority with no compatibility layer or source mirror.

Permanent implementation scope is limited to `domain/sdf`, the bounded `Cargo.lock`
entry, and phase planning/closeout truth. Repository creation, dependency cutover,
and deletion of `domain/sdf` remain later phases.

## Public contract

`SdfField3` now returns fallible samples, conservative bounds, and explicit field
capabilities. `SdfSample` carries:

```text
signed_value  finite inside/surface/outside value; not universally exact distance
safe_step     absent or a finite non-negative conservative tracing step
```

Sphere tracing uses only `safe_step`. Projection, closest-point, and sphere sweep
require exact-distance capability. Unsupported capability is an error rather than
an ordinary miss or silent fallback.

Finite bounds distinguish `Unbounded`, `Empty`, and `Bounded`. Disjoint finite
intersection is `Empty`. Primary normal estimation reports unusable gradients and
never fabricates `Vec3::Y`.

Capability propagation is conservative:

- analytic primitives are exact;
- translation and validated rotation preserve exactness;
- uniform scale preserves exactness while scaling value and step;
- affine transforms provide a conservative step and downgrade exactness;
- hard booleans preserve only a conservative step;
- smooth operations, clamps, warps, repetition, and mirrors remove unproven
  metric capabilities.

## Test evidence

Migrated package tests:

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

Coverage includes invalid construction, primitive metrics, `Empty` bounds algebra,
capability propagation/removal, affine conservative stepping, tracing
no-overshoot, sign-only rejection, structured query terminals, gradient failures,
downstream-style custom fields, and trait-object use.

## Validation

GitHub Actions run `29693976293` passed for the completed implementation source:

```text
cargo metadata --format-version 1 --locked --no-deps
cargo tree -p sdf --locked
cargo tree -i sdf --workspace --locked
cargo fmt --all -- --check
cargo test -p sdf --locked
cargo clippy -p sdf --all-targets --locked -- -D warnings
cargo check --workspace --all-targets --locked
pnpm --dir docs-site install --frozen-lockfile
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check and clean tracked-state verification
```

The final closeout/document state is validated again before the temporary workflow
is removed.

Not claimed as passed: MSRV, benchmarks, dedicated property-testing tooling, full
workspace tests, full workspace Clippy, or runtime/GPU evidence. Exploratory broad
runs encountered hosted-runner disk exhaustion during editor/Godot linking and
existing unrelated `ui_text` Clippy warnings.

## Dependency proof

The `sdf` lockfile entry changed only from `geometry + glam` to
`glam + thiserror`. No unrelated dependency refresh remains.

## Scope exclusions

No permanent renderer, world, material, ECS, scheduler, UI, RunenUI, GPU, shader,
serialization, or external-repository changes are included. The temporary
validation workflow is removed before merge.

## Next safe action

`PT-RUNENSDF-003` may create and independently prove the standalone RunenSDF
repository after separate planning and authorization. It may not cut Runenwerk
over or delete `domain/sdf`; those remain `PT-RUNENSDF-004` scope.
