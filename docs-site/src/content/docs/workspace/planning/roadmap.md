---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
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

Framework repositories do not depend on Runenwerk. Runenwerk owns product and
cross-framework integration. RunenUI and RunenRender remain independent peers.

## Current state

```text
Repository-family charter   complete through PR #109
RunenSDF investigation      complete through PR #110
RunenECS investigation      recorded through PR #111; R1 specified, not active
RunenRender investigation   recorded through PR #112; R1 specified, not active
RunenSDF boundary repair    complete through PR #116
RunenSDF repository phase   queued planning; no implementation authorization
```

## Next phase — PT-RUNENSDF-003

Title: Standalone RunenSDF Repository Creation and Corrected Source Transfer

State: `queued-planning`

Owner: SDF and repository governance

Implementation authorization: none until a new bounded phase specification is
accepted.

### Mission

Create `Crystonix/RunenSDF`, transfer the corrected SDF package and tests with
provenance, and prove the repository independently without cutting Runenwerk over
or deleting `domain/sdf`.

### Required decisions before activation

1. repository creation mechanism and owner action;
2. repository/package naming and initial version;
3. license, edition, MSRV, lockfile, and release policy;
4. provenance-preserving transfer method;
5. exact source/test/document move matrix;
6. initial repository layout and validation workflow;
7. downstream public-consumer proof;
8. dependency pinning strategy reserved for PT-RUNENSDF-004;
9. explicit stop and rollback conditions;
10. one exact implementation spec written against merged PT-RUNENSDF-002 source.

### Intended repository shape

```text
Crystonix/RunenSDF
├── Cargo.toml
├── Cargo.lock
├── LICENSE-APACHE
├── LICENSE-MIT
├── README.md
├── crates/
│   └── runensdf/
├── tests or downstream-conformance/
├── docs/
└── .github/workflows/ only for durable repository validation
```

The exact top-level layout is fixed during PT-RUNENSDF-003 planning. The initial
framework remains one package; no core/query/GPU/shader/program/macro split is
introduced without independent pressure.

### Required implementation outcomes

- corrected source and all package tests exist in RunenSDF;
- package name and public imports are finalized without compatibility aliases;
- no Runenwerk path, package, feature, type, document, or build dependency exists;
- independent downstream code can implement and consume `SdfField3` publicly;
- provenance identifies the Runenwerk source commit and transfer mapping;
- repository validation covers formatting, tests, Clippy, docs, stable/MSRV,
  dependency direction, and diff hygiene;
- benchmarks and property testing are included only when their owned scope and
  maintenance policy are explicit;
- Runenwerk remains unchanged except planning/provenance references required to
  prepare the later cutover.

### Forbidden scope

```text
Runenwerk dependency cutover
deleting domain/sdf
source mirror maintained after the later cutover
compatibility or forwarding package
RunenSDF dependency on Runenwerk
GPU, WGPU, shader, material, world, ECS, renderer, or UI ownership
stable persisted field/program format
speculative multi-package split
```

### Exit gate

PT-RUNENSDF-003 completes only when the standalone repository validates
independently, public downstream use is proven, provenance is recorded, and the
exact revision intended for PT-RUNENSDF-004 is identified. Completion authorizes
cutover planning, not automatic Runenwerk deletion.

---

## Later RunenSDF phases

| Phase | Purpose | State |
|---|---|---|
| `PT-RUNENSDF-004` | Pin exact RunenSDF revision, migrate Runenwerk consumers, delete `domain/sdf`, remove old workspace/lockfile authority, validate integration | Blocked by PT-003 |
| `PT-RUNENSDF-005` | Close provenance, compatibility, deleted-path, release, branch, and final ownership evidence | Blocked by PT-004 |

## RunenECS track

The accepted internal sequence remains:

```text
R1 entity identity and structured errors
R2 atomic structural mutation
R3 query and SystemParam unsafe boundaries
R4 explicit reflection and macro migration
R5 remove spatial and geometry ownership
R6 messaging split
R7 change, ownership, and networking separation
R8 neutralize runen_schedule
R9 standalone conformance and performance baseline
```

R1 has a planning specification but no Rust implementation authorization.
Repository creation remains blocked through R9.

## RunenRender track

The accepted internal sequence remains:

```text
R1 neutral identities, errors, and dependency guards
R2 neutral graph and resource descriptors
R3 prepared frame inputs and generic producers
R4 GPU parameter and optional macro ABI
R5 shader and hot-reload boundary
R6 headless WGPU executor
R7 generic surfaces and device loss
R8 diagnostics, capture, and provenance split
R9 Runenwerk adapter migration
R10 internal conformance and performance proof
```

R1 has a planning specification but no Rust implementation authorization.
External extraction remains blocked through R10.

## RunenUI relationship

RunenUI is governed elsewhere. This program does not design or implement RunenUI
APIs. A future Runenwerk-owned adapter may translate accepted renderer-neutral
RunenUI output into generic RunenRender contributions after both sides expose
stable public seams.

## Parallelism policy

Read-only investigation, consumer classification, benchmark/safety command
discovery, and unrelated documentation cleanup may proceed in parallel.
Structural implementation remains phase-authorized and serialized where branches
share workspace, identity, dependency, or planning authority.
