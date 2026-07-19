---
title: RunenECS Extraction Investigation
description: Connector-backed source, API, safety, scheduler, spatial, messaging, reflection, ownership, networking, and extraction-readiness evidence for RunenECS.
status: active
owner: ecs
layer: investigation
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runenecs-r1-entity-errors.ron
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
COMPLETE FILE/CONSUMER INVENTORY     pending local verification
SAFETY DESIGN                        incomplete until unsafe inventory is command-verified
FIRST EXECUTABLE REPAIR              R1 entity identity and structured core errors
SOURCE MOVEMENT                      forbidden
```

The current `ecs` package is not a narrow ECS core. `World` and the crate root
aggregate entity/storage/query/runtime behavior with reflection, multiple messaging
families, ownership routing, geometry-based spatial indexing, change extraction,
and engine tick/frame concepts. The separate scheduler is context-generic in
shape but exposes Runenwerk lifecycle and render/network barriers.

This report establishes the durable owner split and the ordered repair program.
It does not claim complete command-verified inventory and does not authorize code
changes.

## Baseline and evidence

Repository: `Crystonix/Runenwerk`

Reviewed published main:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

Evidence:

```text
E2 GitHub commit/package/PR metadata
E3 connector-backed source, manifest, test, and consumer inspection
E4 Cargo.lock/package facts where available
```

The connector cannot run Cargo, Miri, sanitizers, benchmarks, or reliable complete
repository-wide searches. Local verification remains mandatory.

## Current package candidates

```text
domain/ecs          package ecs
domain/ecs_macros   package ecs_macros
domain/scheduler    package scheduler
```

The intended repository remains:

```text
Crystonix/RunenECS
  runenecs
  runenecs_macros
  runen_schedule
```

`runen_schedule` remains separately usable without `runenecs`. A fourth repository
is not created during this program.

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
runen_schedule
  neutral labels
  access/conflict declarations
  dependency graph
  deterministic stages/waves
  generic reports
  serial reference execution

runenecs
  system access extraction
  world/resource borrowing
  deferred command application at ECS-owned boundaries

Runenwerk
  frame/tick/startup/shutdown/render/network/replay/product phases
```

Parallel execution remains deferred until sound access, deterministic barriers,
panic/poison policy, cancellation, worker ownership, bounded queues, and serial
equivalence are proven.

## Messaging findings

Current public families have different semantics and owners.

Provisional classification:

| Facility | Target disposition |
|---|---|
| typed events/broadcast | likely RunenECS; verify retention, overflow, cursors, terminal behavior |
| FIFO world queues | candidate; retain only with independent non-network ECS consumer proof |
| tick buffers/provenance | Runenwerk simulation/runtime |
| external ingress/transport | Runenwerk |
| work claims/retry/ack | not automatically ECS; requires separate ownership proof |

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

The final disposition follows local consumer inventory.

## Macro findings

`ecs_macros` must be reviewed as a public downstream package. Generated code must:

- use only public `runenecs` APIs;
- preserve generics and where clauses;
- emit stable compile errors;
- avoid Runenwerk paths and hidden global registration;
- pass external compile-pass and compile-fail tests.

## Durable target decisions

Fixed:

- one RunenECS repository with `runenecs`, `runenecs_macros`, and
  independently usable `runen_schedule`;
- no Runenwerk geometry in ECS core;
- no ECS-owned general spatial index;
- opaque world-local generational entities;
- explicit reflection registry;
- serial execution as normative initial behavior;
- structured framework errors;
- no process-global reflection or telemetry authority;
- no source movement before boundary repair and downstream conformance.

Provisional until local evidence:

- exact event/queue/change-journal public families;
- exact scheduler public API and labels;
- public low-level query/SystemParam extensibility;
- retained parallel executor;
- exact package/file move matrix.

## Repair program

```text
R1 entity identity and structured core errors
R2 atomic bundle/spawn/command invariants
R3 query and SystemParam unsafe-boundary hardening
R4 explicit reflection registry and macro migration
R5 remove ECS spatial and geometry ownership
R6 messaging split and tick-buffer migration
R7 change journal and ownership/network separation
R8 neutralize runen_schedule and lifecycle integration
R9 standalone downstream conformance and benchmark baseline
```

The sequence is dependency-ordered. Later steps may be investigated in parallel,
but only the next executable repair receives a concrete phase specification.

## Mandatory local gate

Before activating R1, run:

```text
cargo metadata --format-version 1 --locked
cargo tree -p ecs
cargo tree -i ecs --workspace
cargo tree -p scheduler
cargo tree -i scheduler --workspace
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
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
git status --short --branch
```

Also identify repository-authoritative Miri and sanitizer commands. Do not infer
success when a command is unavailable.

## Gate result

```text
repository ownership direction   established
connector source investigation   substantial but not complete by workflow gate
local consumer/unsafe inventory  pending
complete design gate             pending
implementation authorization     blocked
external extraction              forbidden
```

## Next safe action

After the local gate and owner review, activate exactly R1. Do not implement R2–R9,
rename packages, create RunenECS, or move source externally in the R1 phase.