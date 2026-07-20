---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../workflow-lifecycle.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../specs/pt-runensdf-003-standalone-transfer.ron
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ./active-work.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Roadmap

## Program destination

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk integration/adapters --> applications
RunenRender ----+
RunenUI --------+   independent external workstream
```

Framework repositories do not depend on Runenwerk.

## Current state

```text
Repository-family charter   complete through PR #109
RunenSDF investigation      complete through PR #110
RunenECS investigation      recorded through PR #111; R1 not active
RunenRender investigation   recorded through PR #112; R1 not active
RunenSDF boundary repair    complete through PR #116
RunenSDF standalone phase   active in Crystonix/runen-sdf PR #1
```

## PT-RUNENSDF-003 — Standalone repository

State: `active-implementation`

```text
product: RunenSDF
repository: Crystonix/runen-sdf
package: runen-sdf
crate: runen_sdf
version: 0.1.0
```

The former `Crystonix/RunenSDF` and `runensdf` spellings are superseded.

The repository uses one root public package with `conformance/downstream` and
`xtask` as its only support packages. It does not use `crates/runen-sdf`,
`domain/sdf`, a docs site, a façade, or a compatibility package.

Implementation order:

1. identity, governance, provenance, and validation foundation;
2. corrected source transfer from Runenwerk commit
   `8de096259eab30f8d67672010df9190970d0bfc4`;
3. crate-import and nine-test-module migration;
4. independent lockfile and public downstream conformance;
5. stable and Rust 1.93.0 validation;
6. exact accepted standalone revision closeout.

Parity precedes the proposed module-vocabulary cleanup. `operators`, `transforms`,
and `differential` are reviewed separately after the transferred implementation is
green.

PT-RUNENSDF-003 does not change Runenwerk dependencies, retire `domain/sdf`,
publish a crate, retain a permanent mirror, introduce compatibility forwarding, or
add GPU/renderer/world/material/ECS/UI/persisted-program ownership.

Completion requires independent validation, public downstream proof, exact
provenance, and the standalone revision intended for PT-RUNENSDF-004.

## Later RunenSDF phases

| Phase | Purpose | State |
|---|---|---|
| `PT-RUNENSDF-004` | Consume the accepted standalone revision, migrate Runenwerk, retire the in-workspace package, and validate integration | Blocked by PT-003 |
| `PT-RUNENSDF-005` | Close provenance, compatibility, release, branch, and ownership evidence | Blocked by PT-004 |

## Other tracks

RunenECS retains its accepted R1-R9 sequence; R1 is specified but not authorized.
RunenRender retains its accepted R1-R10 sequence; R1 is specified but not
authorized. RunenUI remains a separate workstream.

Read-only investigation may overlap. Structural implementation remains bounded by
accepted phase authority.