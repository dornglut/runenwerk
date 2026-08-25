---
title: RunenECS Extraction Investigation
description: Connector-backed source, API, safety, scheduler, spatial, messaging, reflection, ownership, networking, and extraction-readiness evidence for RunenECS.
status: active
owner: ecs
layer: investigation
canonical: true
last_reviewed: 2026-08-25
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runenecs-r1-entity-errors.ron
  - ./runenecs-issue-198-current-main-census.md
---

# RunenECS Extraction Investigation

## Question

Which parts of the current ECS, macro, scheduler, spatial, messaging, change,
ownership, and networking surface belong in RunenECS, and what must be repaired
before extraction?

## Verdict

```text
EXTRACTION CANDIDATE                 yes
MOVE CURRENT PACKAGES AS-IS          forbidden
ARCHITECTURAL OWNERSHIP DIRECTION    established
COMPLETE FILE/CONSUMER INVENTORY     command-verified in the Issue 198 census
SAFETY DESIGN                        repair remains required; unsafe inventory is command-verified
FIRST EXECUTABLE REPAIR              R1 entity identity and structured core errors
SOURCE MOVEMENT                      forbidden
```

The current `ecs` package is not a narrow ECS core. `World` and the crate root
aggregate entity/storage/query/runtime behavior with reflection, multiple messaging
families, ownership routing, geometry-based spatial indexing, change extraction,
and engine tick/frame concepts. The separate scheduler is context-generic in
shape but exposes Runenwerk lifecycle and render/network barriers.

This report establishes the durable owner split and the ordered repair program.
The complete current-main evidence and current decisions are now recorded in
[RunenECS Issue 198 Current-Main Census](./runenecs-issue-198-current-main-census.md),
which supersedes conflicting historical statements in this report. This report
does not authorize code changes.

## Baseline and evidence

Repository: `dornglut/runenwerk`

Reviewed published main:

```text
25c20a8b7643dc391ec49d870b24458767dd6033
```

Evidence:

```text
E2 GitHub commit/package/PR metadata
E3 connector-backed source, manifest, test, and consumer inspection
E4 Cargo.lock/package facts where available
```

The historical connector evidence did not run Cargo, Miri, sanitizers, benchmarks,
or reliable complete repository-wide searches. The current-main census performs
the local command-verified gate and records the remaining Miri, sanitizer, and
MSRV gaps explicitly.

## Current package candidates

```text
domain/ecs          package ecs
domain/ecs_macros   package ecs_macros
domain/scheduler    package scheduler
```

The intended repository remains:

```text
dornglut/runen-ecs
  runenecs
  runenecs_macros
```

ECS-native scheduling is part of `runenecs`; no external scheduler repository or
generic scheduler dependency is required by #198.

## Current ECS aggregation

The inspected ECS root and `World` combine:

- entity allocation and generations;
- archetype/dense component storage implementation;
- resources, bundles, queries, filters, and deferred commands;
- system params and schedule integration;
- explicit and process-global reflection;
- broadcast/event streams;
- work/FIFO queues;
- tick-local message buffers;
- ownership/authority routing;
- component indexes and geometry-based spatial hash indexes;
- structural/component/resource change extraction;
- engine-shaped tick/frame windows and counters;
- telemetry and mixed error/terminal policy.

This aggregation is current implementation fact, not the target repository
boundary.

## Safety and correctness findings

### Entity identity

- `Entity` is forgeable through public fields.
- allocator generations saturate rather than proving safe exhaustion behavior;
- stale, double-free, and cross-world identity behavior requires explicit proof;
- raw entity values are used near product/network ownership concepts but are not
  suitable stable persistence/network identities.

Target direction:

- private entity representation;
- checked accessors only where justified;
- stale and double-free rejection;
- exhausted slots retired permanently rather than reused through saturation;
- explicit Runenwerk mapping to product/network IDs.

### Atomic structural mutation

Inspected bundle insertion/removal, spawn, and commands can partially mutate or
panic in ordinary failure paths.

Required invariant:

- each safe structural operation is all-or-nothing for its documented scope;
- preflight/registration occurs before mutation;
- spawn failure leaves no live partial entity;
- command failure cannot silently replay or leave undocumented partial state;
- batch naming does not imply transactions unless rollback/atomicity is real.

### Query extension boundary

Safe query APIs rely on low-level implementors declaring access metadata while
using raw storage pointers. Externally implementable metadata participates in
aliasing and lifetime safety.

The first extracted release should seal low-level query implementation and expose
supported read/write/entity/optional/tuple/filter forms. A future custom-query API
requires an explicit unsafe contract and independent conformance.

### SystemParam boundary

`SystemParam` extraction uses cached state and raw world/command pointers.
Generated and manual extension rules must define access, state lifetime, pointer
scope, escape prevention, and nested parameter behavior.

Preferred initial direction:

- public derive-based composition;
- sealed/doc-hidden or explicitly unsafe implementation internals;
- complete safety comments;
- Miri/sanitizer proof for query/resource/command combinations.

### Errors and telemetry

Current public/runtime behavior mixes structured errors, `anyhow`, `expect`,
assertions, panic overflow, tracing, process-global telemetry, and wall-clock
facts.

Framework public APIs need structured errors and deterministic reports. Global
logging/telemetry switches are not framework authority.

## Reflection findings

The current implementation exposes both explicit registries and process-global
registration.

Target ownership:

```text
Rust TypeId       process-local concrete Rust identity
registry ID       explicit registry-local identity
stable type key   persisted/schema identity only when separately governed
```

Requirements:

- explicit registry instance and lifetime;
- no hidden `OnceLock`/global mutation authority;
- deterministic duplicate policy;
- test isolation;
- macros generate descriptors rather than register globally;
- serialization/versioning remains separate from Rust reflection identity.

## Spatial and geometry findings

ECS owns a geometry-based spatial hash while the workspace already has separate
`spatial` and `spatial_index` domains.

This is duplicate ownership and forces Runenwerk geometry into ECS core.

Target:

```text
RunenECS
  stores entities and component data
  exposes generic change observation

Runenwerk spatial adapter
  maps selected entity/component changes to accepted spatial indexes
```

RunenECS does not understand AABBs, coordinates, cells, or spatial query policy.
No RunenSpatial repository is authorized here.

## Scheduler findings

The scheduler is generic over execution context but exposes Runenwerk-shaped
concepts such as update/render phases, render submission, publication,
generation finalization, replay/network capture, and product barriers.

Target split:

```text
runenecs
  system identity and access facts
  explicit semantic order and sets
  schedule validation and deferred-command boundaries
  deterministic serial reference execution

Runenwerk
  frame/tick/startup/shutdown/render/network/replay/product phases
  application host execution and product/publication barriers
```

Semantic ordering is distinct from access incompatibility. The `ecs -> scheduler`
edge is removed in the target topology; no replacement generic scheduler crate is
introduced. Generic DAG/demo/DOT/telemetry residue without a consumer is deleted.
Parallel execution remains deferred until sound access, deterministic barriers,
panic/poison policy, cancellation, worker ownership, bounded queues, and serial
equivalence are proven.

## Messaging findings

Current public families have different semantics and owners.

Current-main classification:

| Facility | Target disposition |
|---|---|
| typed events/broadcast | RunenECS when local retention, overflow, cursors, and terminal behavior are proven |
| FIFO world queues | RunenECS only for proven local semantics |
| tick buffers/provenance | Runenwerk simulation/runtime |
| external ingress/transport | Runenwerk |
| work claims/retry/ack | delete unless an ECS-local consumer proves the semantics |

No facility is retained solely because it currently lives in `World`.

## Change, ownership, and networking findings

The inspected change extraction and ownership APIs use engine tick/frame windows,
owner routing, interest filters, process-local sequences, and networking/editor
consumers.

Target classification:

```text
local component/resource change observation   RunenECS
optional generic local journal                 candidate; needs non-network proof
tick/window lifecycle and provenance           Runenwerk
owner/authority/interest routing                Runenwerk
replication, prediction, rollback, transport   Runenwerk
replay formats and retention                    Runenwerk
```

The exact current-main disposition is recorded in the Issue 198 census.

## Macro findings

`ecs_macros` must be reviewed as a public downstream package. Generated code must:

- use only public `runenecs` APIs;
- preserve generics and where clauses;
- emit stable compile errors;
- avoid Runenwerk paths and hidden global registration;
- pass external compile-pass and compile-fail tests.

## Durable target decisions

Current-main decisions:

- one eventual RunenECS repository with `runenecs` and `runenecs_macros`;
- no `runen_schedule` package and no generic scheduler dependency;
- no Runenwerk geometry in ECS core;
- no ECS-owned general spatial index;
- opaque world-local generational entities;
- explicit reflection registry;
- deterministic serial execution as normative initial behavior;
- structured framework errors;
- no process-global reflection or telemetry authority;
- no source movement before boundary repair and downstream conformance;
- current-main census C0 is complete; C1/R1 remains the first implementation
  slice after #198 acceptance.

Remaining redesign gates:

- event/queue/change-journal retention and terminal semantics;
- sealed low-level query/SystemParam boundary and Miri/sanitizer proof;
- final package/file move matrix after C1-C8 evidence;
- retained parallel executor, which is not required for the serial reference.

## Repair program

```text
C0 current-main census and authority binding
C1 entity identity and structured core errors
C2 atomic bundle/spawn/command invariants
C3 query and SystemParam unsafe-boundary hardening
C4 explicit reflection registry and macro migration
C5 ECS-native schedule/access/order/deferred semantics; remove ecs -> scheduler
C6 remove ECS spatial and geometry ownership
C7 messaging split and ownership/network/lifecycle separation
C8 standalone downstream conformance and benchmark baseline
C9 accepted repository transfer and Runenwerk cutover
```

The sequence is dependency-ordered. Later steps may be investigated in parallel,
but only the next executable repair receives a concrete phase specification.

## Mandatory local gate

Before activating C1/R1, the current-main census ran:

```text
cargo metadata --format-version 1 --locked
cargo tree -p ecs --locked
cargo tree -i ecs --workspace --locked
cargo tree -p scheduler --locked
cargo tree -i scheduler --workspace --locked
find domain/ecs domain/ecs_macros domain/scheduler -type f | sort
rg -n '^\s*(pub\s+)?(unsafe\s+)?trait|unsafe\s*\{' domain/ecs domain/ecs_macros domain/scheduler
rg -n '\becs\b|ecs::|scheduler::' --glob Cargo.toml --glob '*.rs' .
rg -n 'SpatialIndex|SpatialHashIndex|SpatialHashConfig|geometry::Aabb3' .
rg -n 'OnceLock|global_type_registry|register_global' domain/ecs domain/ecs_macros .
rg -n 'ExecutionPhaseKind|BarrierKind|set_slow_node_logging_enabled' .
rg -n 'OwnerId|OwnerRole|tick_buffer|change_extraction|interest' domain engine net apps adapters
cargo test -p ecs --all-features --locked
cargo test -p scheduler --all-features --locked
cargo clippy -p ecs -p scheduler -p ecs_macros --all-targets --all-features --locked -- -D warnings
cargo validate
CI=true pnpm --dir docs-site build
git diff --check
git status --short --branch
```

The census found no repository-authoritative Miri, sanitizer, or MSRV command;
those gaps remain explicit and are not inferred as success.

## Gate result

```text
repository ownership direction   established
accepted base                    exact 25c20a8b7643dc391ec49d870b24458767dd6033
local consumer/unsafe inventory  command-verified for #198 census
focused/workspace/docs gate      passed
Miri/sanitizer/MSRV              no repository-authoritative lane found
implementation authorization     blocked until #198 acceptance
external extraction              forbidden
```

## Next safe action

After #198 is accepted and owner review authorizes C1, activate exactly R1. Do
not implement C2–C9, rename packages, create RunenECS, or move source externally
in the R1 phase.
