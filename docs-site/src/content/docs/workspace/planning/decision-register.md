---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../workflow-lifecycle.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ./active-work.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
---

# Decision Register

## Repository-family ownership

Date: 2026-07-19

Decision:

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk integration --> applications
RunenRender ----+
RunenUI --------+   independent peer workstream
```

Runenwerk remains the product/integration repository. Framework repositories do
not depend on Runenwerk. Direct framework dependencies require separate accepted
evidence. Rejected: submodules, source mirrors, universal shared core, unchanged
directory extraction, and indefinite compatibility packages.

## Parallel maturity model

Date: 2026-07-19

Decision: RunenSDF extracts first. RunenECS performs ordered internal repairs.
RunenRender performs internal decomposition and public-boundary proof. RunenUI is
managed separately. Read-only investigation may overlap; structural work requires
one bounded authorized phase.

## PT-RUNENSDF-002 completion

Date: 2026-07-20

Decision: accept the in-workspace RunenSDF boundary correction through PR #116.

Accepted outcomes:

- no Runenwerk `geometry` dependency in `domain/sdf`;
- validated SDF-owned bounds and ray values;
- distinct signed value and conservative tracing step;
- explicit exact-distance capability;
- capability-sensitive metric and tracing queries;
- validated authored state and structured failures;
- explicit query terminals and fallible normals;
- all nine package tests migrated;
- no compatibility layer, source mirror, external repository, or deletion.

Accepted command evidence:

```text
Cargo metadata and dependency trees
formatting
focused SDF tests and Clippy
workspace all-target API check
docs frozen install, validator, and build
diff and clean tracked-state checks
```

Not claimed: MSRV, benchmarks, dedicated property tooling, full workspace tests,
full workspace Clippy, runtime, or GPU evidence.

Closeout:
`../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md`.

## PT-RUNENSDF-003 queue

Date: 2026-07-20

Decision: queue standalone repository creation and corrected source transfer for
planning only. No implementation is authorized by this transition.

PT-003 may define governance, create `Crystonix/RunenSDF` after owner/tool
availability is resolved, transfer corrected source/tests with provenance, and
establish independent conformance. It may not cut Runenwerk over, delete
`domain/sdf`, introduce forwarding compatibility, retain a permanent source
mirror, or add renderer/GPU/material/ECS/world/UI/persisted-program ownership.

Runenwerk cutover and deletion remain PT-RUNENSDF-004.

## RunenECS boundary

Date: 2026-07-19

Decision: do not extract current ECS directories unchanged. Candidate packages are
`runenecs`, `runenecs_macros`, and independently usable `runen_schedule`. ECS core
does not own Runenwerk geometry/spatial policy. Reflection is explicit. Runenwerk
owns lifecycle and network/replay/product policy. Only R1 is specified; no ECS
Rust implementation is authorized.

## RunenRender decomposition

Date: 2026-07-19

Decision: prove renderer boundaries inside Runenwerk before extraction. Required
candidates are `runenrender_core` and `runenrender_wgpu`; macros remain optional.
Neutral core contains no WGPU/Winit/ECS/SDF/UI/scene/material/Runenwerk policy.
Runenwerk retains lifecycle, native-window mapping, domain adapters, editor policy,
shader filesystem/hot reload, and future UI integration. Only R1 is specified; no
renderer Rust implementation is authorized.

## RunenUI separation

Date: 2026-07-19

Decision: RunenUI implementation remains outside this program. A future adapter is
Runenwerk-owned after both RunenUI and RunenRender expose stable public seams.

## Historical supersession

PR #107 is closed unmerged. Commit
`b5e9624c594c9f1e3f2a0929bf84028f13fde860` is a rejected incomplete extraction
attempt and is not an implementation base.
