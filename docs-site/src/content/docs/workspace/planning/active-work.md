---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../specs/pt-runensdf-003-standalone-transfer.ron
  - ../../guidelines/programming-principles.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runensdf-repository-identity-decision.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

## Current primary implementation

ID: `PT-RUNENSDF-003`

Title: Standalone RunenSDF Repository and Corrected Source Transfer

Lifecycle state: `active-implementation`

Implementation authorization: `active-implementation-authorized`

External delivery:

```text
repository: Crystonix/runen-sdf
main skeleton commit: 6fb544856e42445e5be53107f61a14e5ccea211d
implementation branch: agent/pt-runensdf-003-bootstrap
pull request: Crystonix/runen-sdf#1
```

Canonical identity:

```text
product: RunenSDF
repository: Crystonix/runen-sdf
package: runen-sdf
crate: runen_sdf
version: 0.1.0
```

The former `Crystonix/RunenSDF` and `runensdf` planning spellings are superseded.
No compatibility package or crate alias preserves them.

## Completed prerequisite

`PT-RUNENSDF-002` completed the public and numerical boundary correction through
Runenwerk PR #116 and merge commit
`8de096259eab30f8d67672010df9190970d0bfc4`.

That exact commit and `domain/sdf` are the only source-transfer authority.

## Immediate execution order

1. merge Runenwerk PR #118 after its durable GitHub Actions documentation gate is
   green;
2. finish the standalone repository foundation in `runen-sdf` PR #1;
3. transfer the corrected source without behavioral redesign;
4. transfer all nine integration-test modules and migrate imports to `runen_sdf`;
5. replace the downstream conformance stub with real public-API proof;
6. migrate framework-owned API, numerical, query, and ownership documentation;
7. commit the independent lockfile and complete repository-policy checks;
8. pass stable and Rust 1.93.0 validation through `cargo validate` in GitHub
   Actions;
9. review exact source parity and record the accepted standalone revision;
10. close PT-RUNENSDF-003 and activate a separately bounded PT-RUNENSDF-004.

No owner-operated local validation is required. GitHub Actions is the merge gate
and must invoke the same repository-local validation command used by contributors.

## Allowed scope

```text
Crystonix/runen-sdf root package, source, tests, docs, conformance, xtask, CI,
licenses, security policy, lockfile, status, roadmap, and provenance
Runenwerk PT-RUNENSDF-003 design, identity, planning, specification, provenance,
and durable documentation validation
identity/import migration required by sdf -> runen-sdf / runen_sdf
```

## Forbidden scope

```text
Runenwerk Cargo dependency cutover
removal of domain/sdf or its workspace membership
compatibility or forwarding packages
permanent source mirrors, source includes, or submodules
behavioral or numerical redesign during parity transfer
module regrouping before parity is independently green
crates.io publication
GPU, shader, renderer, world, material, ECS, scheduler, UI, or persisted-program ownership
```

## Automated completion gate

PT-RUNENSDF-003 completes only when one exact standalone revision proves through
GitHub Actions that:

- the corrected source and all nine package tests run independently;
- public downstream code implements and consumes the framework through public APIs;
- no Runenwerk reference, external path dependency, private include, compatibility
  package, forwarding namespace, or stale package identity exists;
- the committed lockfile, licenses, security policy, provenance, and documentation
  are valid;
- stable and Rust 1.93.0 metadata, tests, Clippy, and rustdoc pass through one
  maintained `cargo validate` command;
- the exact standalone commit intended for PT-RUNENSDF-004 is recorded;
- Runenwerk source, dependencies, workspace membership, and lockfile remain
  unchanged during this phase.

## Program allocation

```text
RunenSDF     PT-RUNENSDF-003 active implementation
RunenECS     R1 specified; no Rust implementation authorized
RunenRender  R1 specified; no Rust implementation authorized
RunenUI      independent workstream outside this program
```

`PT-RUNENSDF-004` remains blocked until the standalone revision is accepted and
its GitHub Actions evidence is recorded.