---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-19
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

## Current primary implementation

ID: `PT-RUNENSDF-002`

Title: RunenSDF Boundary Correction

Lifecycle state: `active-implementation`

Implementation branch:

```text
impl/runensdf-boundary-correction
```

Owner authorization:

```text
The repository owner explicitly authorized continued connector-side execution on
2026-07-19 and waived manual/local validation as an activation prerequisite.
```

The waiver permits implementation to proceed. It does not establish that any
Cargo, Clippy, docs, property, benchmark, or runtime command passed. Skipped
commands remain unavailable evidence and must be reported as such.

## Program state

Repository-family planning is merged through:

```text
PR #109  repository-family authority
PR #110  RunenSDF investigation, design, and PT-RUNENSDF-002 specification
PR #111  RunenECS investigation, R1-R9 roadmap, and R1 specification
PR #112  RunenRender investigation, R1-R10 roadmap, and R1 specification
```

Current allocation:

```text
RunenSDF     active implementation: PT-RUNENSDF-002
RunenECS     parallel read-only investigation/specification; R1 not code-active
RunenRender  parallel read-only investigation/specification; R1 not code-active
RunenUI      independent workstream outside this program
```

## PT-RUNENSDF-002 goal

Correct `domain/sdf` in place so its public contract is suitable for later clean
transfer into RunenSDF:

- remove the public and manifest dependency on Runenwerk `geometry`;
- add repository-local validated bounds and ray values;
- distinguish signed field value from a proven conservative tracing step;
- expose exact-distance capability explicitly where queries require it;
- replace unchecked authored state and hidden normalization with validated
  construction;
- replace ambiguous `Option` query terminals with structured outcomes/errors;
- make gradient/normal failure explicit;
- migrate all package tests and any discovered direct consumers;
- keep repository creation and source deletion out of this phase.

## Fixed public invariants

```text
SdfSample.signed_value
    finite sign/value information; not universally exact Euclidean distance

SdfSample.safe_step
    absent or a finite non-negative conservative lower bound to the nearest zero set

Sphere tracing
    advances only by safe_step and rejects unsupported fields structurally

Exact-distance queries
    require an explicit exact-distance capability

FieldBounds
    Unbounded | Empty | Bounded(Bounds3)

Disjoint finite intersection
    Empty
```

## Allowed scope

```text
domain/sdf/Cargo.toml
domain/sdf/src/**
domain/sdf/tests/**
focused directly discovered consumer migrations
Cargo.lock only when dependency correction changes it
current SDF documentation/proof/closeout truth
this active-work record and PT-RUNENSDF-002 status
```

## Forbidden scope

```text
external RunenSDF repository creation
deleting domain/sdf
compatibility packages, forwarding APIs, or source mirrors
renderer, world, material, ECS, scheduler, UI, or RunenUI redesign
stable serialization or material/channel payload expansion
GPU/shader/program authoring expansion
universal geometry/core/meta repository
RunenECS or RunenRender Rust implementation in this branch
```

## Evidence status

Available:

```text
E2 GitHub commit, branch, and pull-request metadata
E3 connector-backed inspection of all SDF source modules and all nine test files
E8 merged repository-family, investigation, design, and phase-spec authority
```

Not available unless later supplied by CI or a local executor:

```text
Cargo metadata/tree
Rust compilation and tests
Clippy and formatting
MSRV
property/benchmark execution
docs-site build and docs validator
runtime/GPU evidence
```

No GitHub status checks are configured for the current planning heads.

## Stop conditions

Stop and revise authority before continuing if connector inspection finds:

- a persisted or production consumer requiring an unclassified SDF contract;
- a built-in composition whose sign or conservative-step behavior cannot be
  specified correctly;
- shared CPU/shader source authority;
- a need for material/channel payloads in the core sample;
- a need to create the external repository or delete original source in this phase;
- a dependency on a universal shared-core repository;
- required changes outside the explicitly owned SDF boundary.

## Next phase rule

`PT-RUNENSDF-003` repository-creation planning is not active. It may be considered
only after this implementation is reviewed and its delivered API, source scope,
consumer migrations, tests-as-written, unavailable validations, and remaining
risks are recorded truthfully.
