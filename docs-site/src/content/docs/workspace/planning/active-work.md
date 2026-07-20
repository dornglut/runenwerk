---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../specs/pt-runensdf-003-standalone-transfer.ron
  - ./roadmap.md
  - ./production-tracks.md
  - ./decision-register.md
---

# Active Work

## Current primary implementation

ID: `PT-RUNENSDF-003`

Title: Standalone RunenSDF Repository and Corrected Source Transfer

Lifecycle state: `active-implementation`

External repository:

```text
Crystonix/runen-sdf
main skeleton commit: 6fb544856e42445e5be53107f61a14e5ccea211d
implementation branch: agent/pt-runensdf-003-bootstrap
bootstrap pull request: Crystonix/runen-sdf#1
```

Canonical identity:

```text
product: RunenSDF
repository: Crystonix/runen-sdf
package: runen-sdf
crate: runen_sdf
```

The former `Crystonix/RunenSDF` and `runensdf` planning spellings are superseded.
No compatibility package or crate alias preserves them.

## Completed prerequisite

`PT-RUNENSDF-002` completed the public and numerical boundary correction in
Runenwerk through PR #116 and merge commit
`8de096259eab30f8d67672010df9190970d0bfc4`.

That commit is the source-transfer baseline for `domain/sdf`.

## Current implementation order

1. establish one root package and repository governance;
2. establish provenance, downstream conformance, and validation authority;
3. transfer the corrected implementation without behavioral redesign;
4. migrate all nine package test modules and crate imports;
5. generate the independent lockfile;
6. pass stable and Rust 1.93.0 validation;
7. record the exact standalone revision intended for Runenwerk cutover.

Parity is established before optional module-vocabulary cleanup. The proposed
`operators`, `transforms`, and `differential` grouping must be a separately
reviewable behavior-preserving change after standalone parity is green.

## Allowed scope

```text
Crystonix/runen-sdf repository files
Runenwerk PT-RUNENSDF-003 planning and provenance records
package/test import migration required by sdf -> runen-sdf naming
independent conformance and validation tooling
```

## Forbidden scope

```text
Runenwerk Cargo dependency cutover
retirement of Runenwerk domain/sdf
compatibility or forwarding packages
permanent source mirrors
crates.io publication
GPU, shader, renderer, world, material, ECS, UI, or persisted-program ownership
speculative public package decomposition
```

## Completion gate

PT-RUNENSDF-003 completes only when:

- the corrected source and all package tests run independently;
- no Runenwerk dependency or private source inclusion exists;
- public downstream conformance passes;
- stable and Rust 1.93.0 validation pass through one maintained command;
- provenance names the exact source and accepted standalone commits;
- no Runenwerk cutover or source retirement enters the phase.

## Program allocation

```text
RunenSDF     PT-RUNENSDF-003 active implementation
RunenECS     R1 specified; no Rust implementation authorized
RunenRender  R1 specified; no Rust implementation authorized
RunenUI      independent workstream outside this program
```

`PT-RUNENSDF-004` remains blocked until the standalone revision is accepted.