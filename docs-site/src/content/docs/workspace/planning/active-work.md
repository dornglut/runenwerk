---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../design/active/runensdf-extraction-design.md
  - ../specs/pt-runensdf-002-boundary-correction.ron
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../reports/investigations/runenecs-extraction-investigation.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../specs/pt-runenecs-r1-entity-errors.ron
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../specs/pt-runenrender-r1-identities-errors.ron
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

## Current primary track

ID: `PT-RUNENSDF-003`

Title: Standalone RunenSDF Repository Creation and Corrected Source Transfer

Lifecycle state: `queued-planning`

Implementation authorization: **none**

No external repository creation, source transfer, Runenwerk dependency cutover, or
`domain/sdf` deletion is authorized by this record.

## Completed prerequisite

`PT-RUNENSDF-002 — RunenSDF Boundary Correction` is complete through PR #116.
The phase corrected the public and numerical boundary inside Runenwerk before any
source movement:

- removed the `geometry` package dependency from `domain/sdf`;
- introduced validated SDF-owned bounds and ray values;
- separated signed field value from a proven conservative tracing step;
- made exact-distance capability explicit;
- replaced unchecked authored state and hidden normalization with validated
  construction;
- replaced ambiguous query `Option` results with structured outcomes and errors;
- made gradient and normal failures explicit;
- migrated all nine SDF package test modules;
- retained one implementation authority with no compatibility layer or source
  mirror.

Detailed evidence is recorded in
`../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md`.

## Verified merge gate

GitHub Actions validation passed for the completed source and documentation state:

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

MSRV, dedicated property-test tooling, benchmarks, full workspace tests, and full
workspace Clippy remain separate evidence. They are not represented as passed by
this phase. Earlier exploratory broad runs encountered hosted-runner disk
exhaustion while linking editor/Godot tests and existing unrelated `ui_text`
Clippy warnings.

## Program allocation

```text
RunenSDF     PT-RUNENSDF-002 complete; PT-RUNENSDF-003 queued planning only
RunenECS     R1 specified; no Rust implementation authorized
RunenRender  R1 specified; no Rust implementation authorized
RunenUI      independent workstream outside this program
```

## PT-RUNENSDF-003 planning gate

Before implementation authorization, the next phase must close:

1. external repository creation responsibility and owner action;
2. exact repository/package naming and license/toolchain baseline;
3. provenance-preserving source-transfer method;
4. independent downstream-consumer and conformance layout;
5. exact Runenwerk pinning strategy for the later cutover;
6. move/stay/delete matrix for every current SDF file;
7. validation that the external package has no Runenwerk dependency;
8. a bounded phase specification written against merged PT-RUNENSDF-002 source.

The next phase creates and proves the standalone repository. It does **not** delete
`domain/sdf` or cut Runenwerk over; those remain `PT-RUNENSDF-004` scope.

## Parallel work

Allowed:

- read-only RunenECS and RunenRender investigation;
- documentation cleanup that does not alter extraction authority;
- preparation of repository-creation and provenance decisions.

Forbidden until separately authorized:

- RunenECS or RunenRender structural implementation;
- RunenSDF source duplication without a bounded transfer phase;
- Runenwerk dependency cutover;
- deletion of `domain/sdf`;
- compatibility packages, source mirrors, or universal shared-core repositories.
