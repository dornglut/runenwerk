---
title: RunenECS Extraction Investigation
description: Historical source, API, safety, scheduler, spatial, messaging, reflection, ownership, networking, and extraction-readiness evidence for RunenECS.
status: completed
owner: ecs
layer: investigation
last_reviewed: 2026-09-15
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runenecs-r1-entity-errors.ron
  - ./runenecs-issue-198-current-main-census.md
---

# RunenECS Extraction Investigation

## Question

Which parts of the then-current ECS, macro, scheduler, spatial, messaging, change,
ownership, and networking surface belonged in RunenECS, and what had to be repaired
before extraction?

## Role of this report

This is point-in-time supporting investigation evidence for the completed RunenECS
repair/extraction program. It does not own current reusable ECS architecture, phase
sequencing, activation, or implementation state.

Current reusable RunenECS authority belongs to
[standalone RunenECS](https://github.com/dornglut/runen-ecs/blob/main/ARCHITECTURE.md).
Runenwerk's durable downstream framework/integration boundary is owned by
[ADR 0014](../../adr/accepted/0014-repository-family-extraction-boundaries.md) and the
[framework integration architecture](../../architecture/repository-family-architecture.md).
The [Issue 198 Current-Main Census](./runenecs-issue-198-current-main-census.md)
records the command-verified predecessor evidence that updated this earlier investigation.
The completed extraction boundary and C0-C9 repair-plan documents remain available in
Git history.

## Verdict

The following was the accepted investigation verdict at the 2026-08-25 planning point:

```text
EXTRACTION CANDIDATE                 yes
MOVE CURRENT PACKAGES AS-IS          forbidden
ARCHITECTURAL OWNERSHIP DIRECTION    established in accepted design
COMPLETE FILE/CONSUMER INVENTORY     command-verified in Issue 198 census
SAFETY DESIGN                        repair required; unsafe inventory verified
FIRST IMPLEMENTATION CANDIDATE       C1/R1 entity identity and core errors
SOURCE MOVEMENT                      forbidden
```

The predecessor `ecs` package was not a narrow ECS core. `World` and the crate root
aggregated entity/storage/query/runtime behavior with reflection, multiple messaging
families, ownership routing, geometry-based spatial indexing, change extraction, and
engine tick/frame concepts. The predecessor `scheduler` package was generic in some
implementation shape but also exposed Runenwerk lifecycle and render/network/product
barriers.

## Baseline and evidence

Repository: `dornglut/runenwerk`

Current-main census base at the time:

```text
25c20a8b7643dc391ec49d870b24458767dd6033
```

The historical connector investigation established structure but could not run
Cargo, Miri, sanitizers, benchmarks, or reliable complete repository-wide
searches. Issue #198 subsequently ran the required checked-out census. Remaining
Miri, sanitizer, and MSRV gaps were recorded rather than inferred as success.

## Current and target package shape at the investigation point

Predecessor packages:

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

The proc-macro companion was expected to remain separate while technically required by
Rust proc-macro packaging and the existing derive boundary. ECS-native scheduling was
targeted for `runen_ecs`; no `runen_schedule` or external RunenScheduler dependency was
required.

## Predecessor ECS aggregation

The inspected ECS root and `World` combined:

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

This aggregation is historical implementation fact, not current standalone RunenECS
architecture.

## Safety and correctness findings

### Entity identity

The investigation found:

- `Entity` was forgeable through public `id` and `generation` fields;
- allocator generations saturated rather than proving safe exhaustion behavior;
- two fresh worlds could produce equal entity bits;
- world membership used the entity value itself, so index+generation alone could not
  guarantee cross-world rejection;
- raw entity values occurred near product/network ownership concepts but were not
  suitable stable persistence or network identities.

The accepted C1/R1 target mechanism was:

```text
Entity = opaque WorldScopeId + slot/index + generation
World  = owns exactly one matching WorldScopeId
```

Required semantics recorded by the investigation:

- the allocator emits only entities carrying its world's scope;
- every world operation validates scope before slot/generation;
- a foreign-world entity is rejected even when slot and generation coincide with a
  live local entity;
- world scopes are checked, non-reusing process-local runtime identities;
- world-scope exhaustion fails world creation rather than wrapping or reusing;
- slots retire permanently on generation exhaustion;
- stale, unknown/cross-world, double-free, index-exhaustion, and generation-exhaustion
  operations are structured and non-mutating on failure;
- there is no public forgeable entity constructor;
- diagnostic accessors do not create persistence or wire contracts;
- WorldScopeId, slot, and generation are never stable network/persistence IDs.

### Atomic structural mutation

Inspected bundle insertion/removal, spawn, and commands could partially mutate or panic
in ordinary failure paths.

The recorded target invariant was:

- each safe structural operation is all-or-nothing for its documented scope;
- preflight/registration occurs before mutation;
- spawn failure leaves no live partial entity;
- command failure cannot silently replay or leave undocumented partial state;
- batch naming does not imply transactions unless rollback/atomicity is real.

### Query extension boundary

Safe query APIs relied on low-level implementors declaring access metadata while using
raw storage pointers. Externally implementable metadata participated in aliasing and
lifetime safety.

The first extracted release was expected to seal low-level query implementation and
expose supported read/write/entity/optional/tuple/filter forms. A future custom-query
API required an explicit unsafe contract and independent conformance.

### SystemParam boundary

`SystemParam` extraction used cached state and raw world/command pointers. Generated and
manual extension rules needed to define access, state lifetime, pointer scope, escape
prevention, and nested parameter behavior.

Preferred initial direction was:

- public derive-based composition;
- sealed/doc-hidden or explicitly unsafe implementation internals;
- complete safety comments;
- Miri/sanitizer proof for query/resource/command combinations.

### Errors and telemetry

The predecessor public/runtime behavior mixed structured errors, `anyhow`, `expect`,
assertions, panic overflow, tracing, process-global telemetry, and wall-clock facts.
Framework public APIs were expected to use structured errors and deterministic reports;
global logging/telemetry switches were not framework authority.

## Reflection findings

The predecessor implementation exposed both explicit registries and process-global
registration.

Target ownership:

```text
Rust TypeId       process-local concrete Rust identity
registry ID       explicit registry-local identity
stable type key   persisted/schema identity only when separately governed
```

Requirements recorded then:

- explicit registry instance and lifetime;
- no hidden `OnceLock`/global mutable registration authority;
- deterministic duplicate policy;
- test isolation;
- macros generate descriptors rather than register globally;
- serialization/versioning remains separate from Rust reflection identity.

## Spatial and geometry findings

ECS owned a geometry-based spatial hash while the workspace already had separate spatial
domains and the repository family had accepted RunenSpatial ownership.

Target direction:

```text
RunenECS
  stores entities and component data
  exposes generic local change observation

Runenwerk spatial integration
  maps selected ECS changes into accepted RunenSpatial facilities
```

RunenECS was not to understand AABBs, coordinates, cells, or world-query policy. Issue
#198 did not introduce a new RunenSpatial dependency.

## Scheduler findings

The predecessor scheduler source mixed several owners:

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

Semantic ordering was distinct from access incompatibility. The `ecs -> scheduler` edge
was to be removed in the target topology; no replacement generic scheduler crate was to
be introduced. Parallel execution was deferred until its safety and equivalence proof
was separately accepted. Current execution semantics are now owned by standalone
RunenECS, not by this historical report.

## Messaging findings

The predecessor public families had different semantics and owners.

| Facility | Historical target disposition |
|---|---|
| typed events/broadcast | RunenECS only when retention, overflow, cursors, and terminal behavior are ECS-local |
| FIFO world queues | RunenECS only for independently proven local semantics |
| ECS-local change observation | RunenECS, with no implied network meaning |
| tick/frame provenance | Runenwerk simulation/runtime |
| host/external ingress | owning integration or framework adapter |
| work claims/retry/ack | delete unless an ECS-local consumer proves the semantics |

No facility was to be retained solely because it lived in `World`.

## Change, ownership, networking, and replay findings

The inspected change extraction and ownership APIs mixed engine tick/frame windows,
owner routing, interest filters, process-local sequences, networking, and editor
consumers.

Historical target classification:

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

`ecs_macros` was reviewed as a public downstream proc-macro package. Generated code was
required to:

- use only public `runen_ecs` APIs;
- preserve generics and where clauses;
- emit stable compile errors;
- avoid Runenwerk paths and hidden global registration;
- pass external compile-pass and compile-fail tests.

## Evidence-backed target implications

The investigation supported these then-current decisions:

- one eventual RunenECS repository with `runen-ecs` and `runen-ecs-macros` packages;
- no generic scheduler package or dependency;
- no Runenwerk geometry in ECS core and no ECS-owned general spatial index;
- opaque, explicitly world-scoped generational entities;
- explicit reflection registry;
- deterministic serial ECS execution as correctness/reference behavior;
- structured framework errors;
- no process-global reflection or telemetry authority;
- no source movement before internal repair and standalone conformance.

Remaining redesign/proof gates at that point included event/queue/change-journal
retention, sealed low-level query/SystemParam safety, Miri/sanitizer/MSRV support, and
future parallel execution.

These are historical program findings; current reusable RunenECS behavior is defined by
the standalone repository.

## Historical repair program

The C0-C9 sequence recorded by the completed repair program is retained in Git history.
At the time, this evidence supported C1/R1 as the first implementation slice, spatial
and messaging/lifecycle cleanup before scheduler decontamination, and standalone
conformance after all internal repairs.

## Mandatory checked-out gate used by #198

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

The Issue #198 census records the actual results. No repository-authoritative Miri,
sanitizer, or ECS MSRV command was found at that review point; those were explicit gaps.

## Historical handoff

The safe next action recorded on 2026-08-25 was to create one bounded C1/R1
implementation issue from the accepted Entity/world-scope contract and not implement
later repair phases or move source during C1. That handoff is now completed historical
provenance; it does not activate current work. Current RunenECS work is governed by the
standalone repository and its owning GitHub issues.
