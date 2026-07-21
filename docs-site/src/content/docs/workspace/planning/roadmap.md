---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../specs/pt-runensdf-003-standalone-transfer.ron
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runensdf-repository-identity-decision.md
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
RunenSDF transfer authority active in Runenwerk PR #118
RunenSDF repository         created at Crystonix/runen-sdf
RunenSDF bootstrap          active in Crystonix/runen-sdf PR #1
```

## PT-RUNENSDF-003 — Standalone repository and parity transfer

State: `active-implementation`

```text
product: RunenSDF
repository: Crystonix/runen-sdf
package: runen-sdf
crate: runen_sdf
version: 0.1.0
source baseline: Runenwerk 8de096259eab30f8d67672010df9190970d0bfc4
```

The former `Crystonix/RunenSDF` and `runensdf` spellings are superseded. No
compatibility package or crate alias preserves them.

### Repository target

The repository contains one root public package plus only:

```text
conformance/downstream  public downstream-consumer proof
xtask                   maintained repository validation authority
```

It does not use `crates/runen-sdf`, a façade, a source submodule, private source
inclusion, or speculative public subpackages.

### Ordered delivery

1. merge the decision-complete Runenwerk authority and durable docs validation;
2. complete the standalone foundation: licenses, security policy, lockfile,
   validation authority, durable CI, truthful status, and repository-policy checks;
3. transfer `domain/sdf/src/**` without behavioral redesign;
4. transfer all nine integration-test modules and migrate `sdf` imports to
   `runen_sdf`;
5. migrate or rewrite framework-owned API, numerical, query, and ownership docs;
6. replace the downstream stub with a real public-API consumer;
7. pass `cargo validate` on stable and Rust 1.93.0 in GitHub Actions;
8. review source parity against the exact baseline;
9. record the exact standalone commit intended for PT-RUNENSDF-004;
10. close PT-RUNENSDF-003 without changing Runenwerk dependencies or deleting
    `domain/sdf`.

Parity precedes module-vocabulary cleanup. Combining `combine` and `ops`, renaming
`transform`, or introducing a `differential` group requires a later independently
reviewed behavior-preserving change.

### Required automated evidence

GitHub Actions is the merge gate and invokes the repository-local `cargo validate`
command. The command covers:

- committed locked metadata and dependency trees;
- formatting, workspace tests, downstream conformance, and all-target Clippy;
- rustdoc with denied warnings;
- Rust 1.93.0 tests;
- dependency-direction and external-path checks;
- rejection of Runenwerk references, source includes, forwarding packages, and
  stale package identities;
- licenses, security policy, provenance, document links, and diff hygiene.

CI must not generate or mutate `Cargo.lock`.

### Forbidden scope

```text
Runenwerk dependency cutover or domain/sdf retirement
compatibility or forwarding package
permanent source mirror, submodule, or private include
behavioral or numerical redesign during parity transfer
GPU, shader, renderer, world, material, ECS, UI, or persisted-program ownership
crates.io publication
speculative public package decomposition
```

### Exit gate

PT-RUNENSDF-003 completes only when one exact standalone revision:

- contains the corrected source and all nine package-test modules;
- passes public downstream conformance;
- has no Runenwerk or external-path dependency;
- passes stable and Rust 1.93.0 validation through `cargo validate` in GitHub
  Actions;
- records exact provenance and source-parity evidence;
- is ready to be pinned by a separately authorized PT-RUNENSDF-004 cutover.

## PT-RUNENSDF-004 — Runenwerk clean cutover

State: blocked by PT-RUNENSDF-003.

The cutover phase must:

1. repeat the complete Runenwerk consumer audit;
2. pin every real consumer to the exact accepted standalone commit;
3. migrate imports and Runenwerk-owned adapters;
4. remove `domain/sdf` from workspace membership and the old lockfile authority;
5. delete internal source, tests, and stale framework-owned documentation;
6. prove no forwarding package, alias, source include, submodule, duplicate source,
   or old package identity remains;
7. validate the complete Runenwerk workspace and integration.

If the consumer audit still finds no production consumer, remove the isolated
internal package without adding an unused external dependency.

Temporary duplicate source may exist only between unmerged coordinated transfer
and cutover branches. A moving branch dependency is forbidden.

## PT-RUNENSDF-005 — Final closeout and release readiness

State: blocked by PT-RUNENSDF-004.

Close provenance, compatibility, deleted-path, branch, release-policy, ownership,
and adoption evidence. Publication remains a separate decision.

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
RunenUI output into generic RunenRender contributions after both public boundaries
are accepted.

## Parallelism policy

Read-only investigation, consumer classification, benchmark/safety command
discovery, and unrelated documentation cleanup may proceed in parallel.
Structural implementation remains phase-authorized and serialized where branches
share workspace, identity, dependency, or planning authority.