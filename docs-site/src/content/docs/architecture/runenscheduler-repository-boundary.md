---
title: RunenScheduler Repository Boundary
description: Canonical repository, dependency, adapter, proof, transfer, and clean-cutover boundary for the independent RunenScheduler framework.
status: active
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ./repository-family-architecture.md
  - ../adr/accepted/0016-independent-runenscheduler-framework.md
  - ../design/active/runenscheduler-design-canvas.md
  - ../design/active/runenscheduler-core-semantics.md
  - ../design/active/runenecs-extraction-boundary-design.md
  - ../reports/investigations/runenscheduler-ownership-investigation.md
  - ../workspace/planning/active-work.md
  - ../workspace/planning/roadmap.md
---

# RunenScheduler Repository Boundary

## Purpose

This document owns the repository-level RunenScheduler boundary accepted by ADR 0016.
The design canvas owns the concise product contract. The core-semantics document owns
planning behavior. RunenECS and Runenwerk designs own their adapters and execution
policy.

## Repository identity

Target external authority:

```text
product       RunenScheduler
repository    dornglut/runen-scheduler
package       runen-scheduler
crate         runen_scheduler
profile       rust-framework
```

The repository is created or populated only after the internal two-consumer proof.
Until clean cutover, Runenwerk remains the current source and migration owner.

## Dependency direction

```text
                      +--> RunenECS adapter --> RunenECS execution
RunenScheduler -------+
                      +--> Runenwerk adapters --> host executors/products

RunenECS ---------------> Runenwerk integration
other frameworks --------> Runenwerk integration
```

Required invariants:

- RunenScheduler does not depend on RunenECS or Runenwerk;
- RunenECS does not own or mirror the neutral planner;
- Runenwerk owns cross-framework composition and product policy;
- consumers use exact accepted RunenScheduler revisions before stable publication;
- no moving branch dependency is allowed;
- no dependency cycle is allowed.

## Public package boundary

RunenScheduler begins with one public package.

Internal modules may separate:

- identities;
- definitions;
- normalization;
- graph preparation;
- diagnostics and inspection;
- serial conformance.

These modules do not justify independent packages.

Additional packages require a proven external dependency subset, release/versioning
unit, platform or MSRV boundary, backend, proc macro, or independently consumed
conformance surface.

The initial program does not create:

```text
runen-scheduler-core
runen-scheduler-executor
runen-scheduler-async
runen-scheduler-gpu
runen-scheduler-physics
runen-scheduler-macros
runen-scheduler-testing
facade or compatibility packages
```

## Framework ownership

RunenScheduler owns only neutral deterministic planning:

- schedule and task identities;
- explicit dependencies;
- opaque access and placement claims;
- shared/exclusive compatibility;
- ambiguity policy;
- readiness plans;
- deterministic serial interpretation;
- provenance, structured errors, and inspection.

RunenScheduler does not own callback execution, thread pools, lifecycle, ECS safety,
product publication, async I/O, physics simulation graphs, or GPU hazards.

## RunenECS adapter

RunenECS owns one explicit adapter that:

- derives access from queries and `SystemParam`;
- maps ECS identities to opaque neutral keys;
- expands ECS sets and ordering into neutral relationships;
- maps systems to neutral task identities;
- retains command buffers and command application inside ECS;
- binds prepared task IDs back to ECS-owned callbacks;
- translates planning diagnostics without flattening them.

The adapter must use only public RunenScheduler contracts after extraction.

RunenECS does not re-export broad scheduler internals as ECS authority. Ergonomic ECS
configuration may wrap narrow neutral APIs while preserving ownership and diagnostics.

## Runenwerk adapters

Runenwerk owns adapters for non-ECS work such as product derivation, procedural or
asset generation, editor jobs, and offline processing.

A Runenwerk adapter may:

- create task and access keys from product identities;
- prepare plans;
- map ready tasks to the existing runtime job executor;
- retain generations, stale-result rejection, and publication;
- add lifecycle and product context to diagnostics.

It may not:

- duplicate neutral graph algorithms;
- move Runenwerk lifecycle into the framework;
- expose private scheduler storage;
- hide branch or dependency cycles;
- preserve a second writable planner authority.

## Proof program

### Planning proof

Accept:

- complete source and consumer census;
- alternatives review;
- concise design canvas;
- exact core semantics;
- migration/deletion map;
- implementation-backend evaluation plan;
- exact ECS and non-ECS proof specifications.

### Internal neutral-core proof

Implement one planner authority internally with:

- checked identities;
- deterministic normalization;
- explicit dependencies;
- shared/exclusive claims;
- ambiguity policy;
- readiness DAG;
- provenance and structured errors;
- serial conformance.

Delete or isolate the legacy callback DAG before treating the new core as authoritative.

### RunenECS proof

Prove adapter-derived access, deterministic readiness, explicit command application,
serial reference behavior, and no ECS types in the neutral public contract.

### Non-ECS proof

Prove a real Runenwerk product or derivation graph above the existing executor.

The proof must exercise branching dependencies, access compatibility, serial and
concurrent execution, deterministic accepted output/publication, and explanation of
why tasks are ordered or excluded.

### Standalone proof

Before transfer, prove that the package can validate without Runenwerk source and that
at least two downstream adapters compile and pass public conformance.

## Transfer gate

External repository work begins only after:

- internal implementation is accepted on main;
- RunenECS and non-ECS proofs pass;
- public API and error ownership are accepted;
- private graph machinery is replaceable and not exposed;
- exact dependency direction is documented;
- source provenance and licensing are identified;
- standalone toolchain, MSRV, validation, and release policy are specified;
- all current consumers and deletion targets are known;
- current exact-head CI is green.

Repository bootstrap and source transfer may be separate bounded slices. A skeleton
repository must not imply completed extraction.

## Clean cutover

The external transfer sequence is:

1. bootstrap repository governance and validation;
2. transfer the accepted neutral implementation and history/provenance evidence;
3. validate standalone tests and public documentation;
4. publish an exact accepted revision;
5. update RunenECS and Runenwerk adapters to that revision;
6. run downstream and integration conformance;
7. delete Runenwerk `domain/scheduler` source and workspace membership;
8. delete broad `engine` scheduler re-exports;
9. remove temporary seams and direct private reach-through;
10. update repository-family authority and closeout evidence.

Temporary duplication may exist only on an unmerged transfer branch.

The completed cutover retains no:

- forwarding package;
- compatibility alias;
- source mirror;
- include or submodule;
- branch dependency;
- duplicated executor path;
- parallel planning authority.

## Serialized changes

Under Dornglut's conflict-based concurrency policy, independent research and source
work may proceed in parallel. These RunenScheduler changes remain serialized:

- root workspace manifest and lockfile changes;
- transfer and source deletion;
- consumer dependency cutover;
- repository-family architecture and ADR acceptance;
- edits to the same scheduler/ECS files;
- public-contract changes that require coordinated downstream migration.

No track consumes an unmerged branch as accepted authority.

## Release and compatibility

The extracted repository defines:

- edition and MSRV;
- locked format, test, strict Clippy, rustdoc, docs, metadata, license, and provenance
  validation;
- public API stability state;
- dependency and feature policy;
- downstream conformance;
- diagnostic namespace `runenscheduler.*`;
- persistence policy stating that runtime IDs and plan internals are not stable formats.

Before stable publication, Runenwerk and RunenECS use exact commits or exact
pre-release versions.

## Closure evidence

The program closes only when:

- `dornglut/runen-scheduler` is the sole maintained neutral planner authority;
- two independent consumers pass public conformance;
- original Runenwerk source and re-exports are deleted;
- no compatibility or private path remains;
- accepted-main validation succeeds in every affected repository;
- provenance, licensing, release, roadmap, and closeout documentation are complete.
