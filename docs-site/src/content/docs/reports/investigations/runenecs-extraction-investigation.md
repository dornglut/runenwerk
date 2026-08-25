---
title: RunenECS Extraction Investigation
description: Source, API, safety, scheduler, spatial, messaging, reflection, ownership, networking, and extraction-readiness evidence for RunenECS.
status: active
owner: ecs
layer: investigation
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

## Role of this report

This is supporting investigation evidence. It does not own durable architecture,
phase sequencing, activation, or implementation state.

Durable ownership lives in the
[RunenECS Extraction Boundary Design](../../design/active/runenecs-extraction-boundary-design.md).
The one canonical repair sequence lives in the
[RunenECS Boundary Repair Execution Plan](../../design/active/runenecs-boundary-repair-execution-plan.md).
GitHub issue `#198` owns the current reconciliation work. The
[Issue 198 Current-Main Census](./runenecs-issue-198-current-main-census.md)
records the command-verified current-main evidence that updates this earlier
investigation.

## Verdict

```text
EXTRACTION CANDIDATE                 yes
MOVE CURRENT PACKAGES AS-IS          forbidden
ARCHITECTURAL OWNERSHIP DIRECTION    established in active design
COMPLETE FILE/CONSUMER INVENTORY     command-verified in Issue 198 census
SAFETY DESIGN                        repair required; unsafe inventory verified
FIRST IMPLEMENTATION CANDIDATE       C1/R1 entity identity and core errors
SOURCE MOVEMENT                      forbidden
```

The current `ecs` package is not a narrow ECS core. `World` and the crate root
aggregate entity/storage/query/runtime behavior with reflection, multiple
messaging families, ownership routing, geometry-based spatial indexing, change
extraction, and engine tick/frame concepts. The current `scheduler` package is
generic in some implementation shape but also exposes Runenwerk lifecycle and
render/network/product barriers.

## Baseline and evidence

Repository: `dornglut/runenwerk`

Current-main census base:

```text
25c20a8b7643dc391ec49d870b24458767dd6033
```

The historical connector investigation established structure but could not run
Cargo, Miri, sanitizers, benchmarks, or reliable complete repository-wide
searches. Issue #198 subsequently ran the required checked-out census. Remaining
Miri, sanitizer, and MSRV gaps are recorded rather than inferred as success.

## Current and target package shape

Current packages:

```text
domain/ecs          package ecs
domain/ecs_macros   package ecs_macros
domain/scheduler    package scheduler
```

Target repository and packages:

```text
dornglut/runen-ecs
  Cargo package runen-ecs          -> Rust crate runen_ecs
  Cargo package runen-ecs-macros   -> Rust crate runen_ecs_macros
```

The proc-macro companion remains separate while technically required by Rust
proc-macro packaging and the existing derive boundary. ECS-native scheduling is
part of `runen_ecs`; no `runen_schedule` or external RunenScheduler dependency
is required.

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

Current evidence:

- `Entity` is forgeable through public `id` and `generation` fields;
- allocator generations saturate rather than proving safe exhaustion behavior;
- two fresh worlds can produce equal entity bits;
- current world membership uses the entity value itself, so index+generation
  alone cannot guarantee cross-world rejection;
- raw entity values occur near product/network ownership concepts but are not
  suitable stable persistence or network identities.

Required target mechanism:

```text
Entity = opaque WorldScopeId + slot/index + generation
World  = owns exactly one matching WorldScopeId
```

The allocator emits only entities carrying its world's scope. World operations
validate scope before slot/generation. Scope allocation is checked and non-reusing
for the process lifetime; exhaustion cannot wrap into another live scope. The
scope is runtime-only and is never serialized or treated as product/network
identity.

Slots with exhausted generations retire permanently. Stale, unknown/cross-world,
double-free, index-exhaustion, generation-exhaustion, and rejected operations
have structured behavior and do not mutate state on failure.

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
- no hidden `OnceLock`/global mutable registration authority;
- deterministic duplicate policy;
- test isolation;
- macros generate descriptors rather than register globally;
- serialization/versioning remains separate from Rust reflection identity.

## Spatial and geometry findings

ECS owns a geometry-based spatial hash while the workspace already has separate
spatial domains and the repository family has accepted RunenSpatial ownership.

Target:

```text
RunenECS
  stores entities and component data
  exposes generic local change observation

Runenwerk spatial integration
  maps selected ECS changes into accepted RunenSpatial facilities
```

RunenECS does not understand AABBs, coordinates, cells, or world-query policy.
Issue #198 does not introduce a new RunenSpatial dependency.

## Scheduler findings

Current scheduler source mixes several owners:

```text
RunenECS
  system identity and ECS access facts
  explicit semantic order and sets
  schedule validation and deferred-command boundaries
  deterministic serial reference execution

Runenwerk
  frame/tick/startup/shutdown/render lifecycle
  host execution and product/publication barriers

Delete after consumer migration
  unsupported generic DAG/demo/DOT/filesystem/global-telemetry residue
```

Semantic ordering is distinct from access incompatibility. The `ecs -> scheduler`
edge is removed in the target topology; no replacement generic scheduler crate is
introduced. Parallel execution remains deferred until sound access, deterministic
boundaries, failure policy, worker ownership, bounded queues, and serial
equivalence are proven.

## Messaging findings

Current public families have different semantics and owners.

| Facility | Target disposition |
|---|---|
| typed events/broadcast | RunenECS only when retention, overflow, cursors, and terminal behavior are ECS-local |
| FIFO world queues | RunenECS only for independently proven local semantics |
| ECS-local change observation | RunenECS, with no implied network meaning |
| tick/frame provenance | Runenwerk simulation/runtime |
| host/external ingress | owning integration or framework adapter |
| work claims/retry/ack | delete unless an ECS-local consumer proves the semantics |

No facility is retained solely because it currently lives in `World`.

## Change, ownership, networking, and replay findings

The inspected change extraction and ownership APIs mix engine tick/frame windows,
owner routing, interest filters, process-local sequences, networking, and editor
consumers.

Target classification:

```text
local component/resource change observation   RunenECS
optional generic local journal                 candidate; needs non-network proof
tick/window lifecycle and provenance           Runenwerk
game ownership/relevancy policy                Runenwerk/application
ECS <-> network identity/state mapping         Runenwerk/application integration
protocol/schema/replication consistency        RunenNet
session/delivery/recovery/transport semantics  RunenNet
prediction/interest semantics                  RunenNet when separately accepted
archival/editor replay formats and retention   Runenwerk/application
```

Runenwerk integration may adapt ECS state into RunenNet contracts; it does not
duplicate RunenNet semantic authority.

## Macro findings

`ecs_macros` must be reviewed as a public downstream proc-macro package. Generated
code must:

- use only public `runen_ecs` APIs;
- preserve generics and where clauses;
- emit stable compile errors;
- avoid Runenwerk paths and hidden global registration;
- pass external compile-pass and compile-fail tests.

## Evidence-backed target implications

The investigation supports these durable decisions, which are owned by the active
boundary design rather than this report:

- one eventual RunenECS repository with `runen-ecs` and
  `runen-ecs-macros` packages;
- no generic scheduler package or dependency;
- no Runenwerk geometry in ECS core and no ECS-owned general spatial index;
- opaque, explicitly world-scoped generational entities;
- explicit reflection registry;
- deterministic serial ECS execution as correctness/reference behavior;
- structured framework errors;
- no process-global reflection or telemetry authority;
- no source movement before internal repair and standalone conformance.

Remaining redesign/proof gates include event/queue/change-journal retention,
sealed low-level query/SystemParam safety, Miri/sanitizer/MSRV support, and any
future parallel executor.

## Repair program

The canonical C0-C9 order is not duplicated here. It is owned by the
[Boundary Repair Execution Plan](../../design/active/runenecs-boundary-repair-execution-plan.md).
The evidence supports C1/R1 as the first implementation slice, spatial and
messaging/lifecycle cleanup before scheduler decontamination, and standalone
conformance after all internal repairs.

## Mandatory checked-out gate

Issue #198 required the checked-out executor to run at least:

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

The current-main census records the actual results. No repository-authoritative
Miri, sanitizer, or ECS MSRV command was found; those remain explicit gaps.

## Next safe action

After #198 is accepted, re-establish current main and create exactly one bounded
C1/R1 implementation issue from the accepted Entity/world-scope contract. A
retained RON spec may provide subordinate handoff detail if still useful; it does
not activate the phase. Do not implement C2-C9, rename packages, populate the
external repository, or move source during C1.
