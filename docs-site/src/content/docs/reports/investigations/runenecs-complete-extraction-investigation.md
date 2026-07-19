---
title: RunenECS Complete Extraction Investigation
description: Source, public API, safety, scheduler, spatial, reflection, messaging, change, ownership, consumer, and cutover investigation for RunenECS.
status: active
owner: ecs
layer: investigation
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../workspace/planning/roadmap.md
  - ../../domain/ecs/README.md
  - ../../domain/ecs/architecture.md
  - ../../domain/ecs/features.md
  - ../../net/ecs-runtime-feature-inventory.md
---

# RunenECS Complete Extraction Investigation

## Question

Can the current ECS, macros, and scheduler packages become an independently
usable RunenECS repository, and what must be repaired before source transfer?

## Verdict

```text
EXTRACTION CANDIDATE: yes
MOVE CURRENT PACKAGES AS-IS: no
TARGET REPOSITORY: Crystonix/RunenECS
TARGET PACKAGES: runenecs, runenecs_macros, runen_schedule
SOURCE MOVEMENT AUTHORIZED: no
NEXT PHASE: PT-RUNENECS-002 decision closure / implementation contract
```

The storage, query, command, system, macro, and context-generic scheduling
foundations are substantial. The current package boundary is nevertheless not
safe or repository-neutral enough to transfer unchanged.

The blocking concerns are:

- public/forgeable entity identity and generation exhaustion;
- non-atomic bundle/spawn/command mutation;
- public extension traits that participate in unsafe alias/lifetime contracts;
- `anyhow` and panic-based failure in public/runtime paths;
- process-global reflection and telemetry authorities;
- geometry-dependent ECS-owned spatial indexing despite separate spatial domains;
- scheduler plans hard-coded to Runenwerk phases and product barriers;
- engine frame/tick, ownership routing, interest filtering, and replication-shaped
  state inside `World`;
- three messaging families whose final ownership must be narrowed;
- incomplete external macro and unsafe-extension conformance.

## Reviewed baseline

Repository: `Crystonix/Runenwerk`

Published main head:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

Repository-family charter base:

```text
docs/repository-family-charter
d14fc0e07ace3c2123ff70fc748b0694114cb6e1
```

## Evidence limits

This investigation used GitHub source, manifest, documentation, commit, lockfile,
and selected consumer inspection. GitHub code search repeatedly timed out, and
the connector cannot execute local Cargo, grep, Miri, sanitizer, benchmark, or
runtime commands.

The ownership and design conclusions are decision-ready. Exact file/test/consumer
counts and the current command baseline must be verified locally before source
changes.

## Package inventory

### ECS

```text
path: domain/ecs
package: ecs
version: 0.1.0
edition: 2024
publish: false
features: telemetry -> scheduler/telemetry
dependencies: anyhow, ecs_macros, geometry, scheduler, thiserror
dev dependencies: criterion, glam
benches: phase35, phase4, phase5b, phase6
```

### Macros

```text
path: domain/ecs_macros
package: ecs_macros
proc-macro package
```

The macro implementation resolves the package name `ecs` and currently derives:

```text
Component
StatefulComponent
Resource
SystemParam
Bundle
Reflect
ReflectComponent
ReflectResource
```

### Scheduler

```text
path: domain/scheduler
package: scheduler
context-generic execution package
```

Its module surface includes access declarations, graph/DAG planning, labels,
registered systems, plans, telemetry, and utility support.

## ECS source ownership inventory

Current ECS module families:

```text
bundle
commands
component
entity
errors
indexing
query
reflect
storage
system
telemetry
world
```

`World` currently owns or coordinates:

- entity allocator and alive set;
- component and archetype storage;
- resources;
- reflection registrations;
- broadcast streams and observers;
- destructive work queues;
- tick-indexed buffers and finalization;
- current tick and current frame indices;
- owner/target routing and transfer logs;
- component secondary indexes;
- geometry-based spatial indexes;
- component/resource change logs;
- removed-component stage windows;
- structural change extraction and interest/ownership filtering.

This is broader than the final framework boundary.

## Core entity and storage findings

### Entity identity

Current `Entity` exposes public `id` and `generation` fields. `EntityAllocator`
reuses IDs and increments generations with saturating arithmetic.

Risks:

- consumers can forge arbitrary identities;
- generation saturation may eventually reuse the maximum generation;
- stale/double free behavior is not a clearly structured public contract;
- raw entity identity is already converted to `u64` in network mapping code.

Target:

- private entity fields with read-only accessors;
- world/allocator-local opaque generation identity;
- checked free/despawn;
- a slot whose generation is exhausted is retired rather than reused;
- no raw entity ID is a stable persisted/network identity;
- Runenwerk networking maps entities to explicit `NetEntityId` values.

### Storage

The crate uses dense/archetype storage and records row-level added/changed ticks.
Unsafe query paths read and mutate archetype storage through raw world pointers.

The storage algorithms may transfer after the unsafe contract is reviewed and
proved. Storage internals must remain private, with any public inspection exposed
through read-only reports.

### Bundle atomicity

Tuple and derived bundles register/insert/remove members sequentially. Tuple
support is currently bounded, and derived bundle operations can partially mutate
the world when a later member fails.

`World::spawn` currently expects bundle insertion to succeed and may panic after
entity allocation. Command batches also apply sequentially and stop on first
error without rollback.

Target:

- bundle registration and mutation have preflight/commit semantics;
- insertion either commits the complete bundle or leaves the entity unchanged;
- removal preflights all required members before mutation;
- spawn failure leaves no live partial entity and returns a structured error;
- command queue ordering remains deterministic, but batch atomicity is explicit:
  either a transactional batch or clearly named ordered non-atomic commands;
- no safe public operation relies on panic for ordinary invalid input.

## Query and system safety findings

### QueryData

The public `QueryData` contract exposes unsafe fetch/world methods and access
metadata. Safe query APIs rely on implementors declaring complete access and
respecting aliasing/lifetime requirements.

A safe trait implementation can currently lie about access or return aliased
references, violating the assumptions of safe query entry points.

Target decision:

- built-in query shapes use a sealed internal trait;
- third-party query-data extension is not public in the first extracted release;
- later custom query data requires a separately accepted `unsafe trait` contract,
  a narrow construction API, compile tests, Miri/sanitizer coverage, and external
  conformance;
- ordinary downstream users compose public built-in query shapes and filters.

This favors a sound small surface over premature arbitrary query extension.

### SystemParam

`SystemParam<'w>` uses cached lifetime-independent state, raw world/command
pointers, declared access, and unsafe extraction. The derive macro can generate
implementations, but manual public implementation carries the same soundness
obligations.

Target decision:

- `SystemParam` remains publicly derivable through `runenecs_macros`;
- the low-level implementation trait becomes sealed behind the derive bridge or
  explicitly unsafe and doc-hidden;
- ordinary manual implementation is not safe/public by default;
- generated code uses only the supported runtime bridge;
- tuple/group parameter breadth is implemented through one scalable derive/group
  path rather than permanent arity boilerplate where feasible;
- safety invariants receive focused Miri tests and local comments.

### Runtime

Current ECS runtime:

- converts query/system-param access to scheduler access keys;
- executes scheduler waves serially;
- collects commands per successful system;
- flushes at deferred-command barriers;
- accumulates setup errors as `anyhow::Error`;
- records wall-clock telemetry through process-global atomics.

Serial execution is the reference semantic model. Parallel execution is not
accepted until borrowing, command barriers, failures, cancellation, worker
lifetime, trace order, and serial-equivalence proofs are complete.

## Scheduler findings

The scheduler is generic over execution context, which justifies a separate
`runen_schedule` package in the RunenECS repository. It is not currently
repository-neutral.

Confirmed product coupling:

```text
ExecutionPhaseKind::PreUpdate
ExecutionPhaseKind::FixedUpdate
ExecutionPhaseKind::Update
ExecutionPhaseKind::RenderPrepare
ExecutionPhaseKind::RenderSubmit
ExecutionPhaseKind::FrameEnd

BarrierKind::ApplyDeferredCommands
BarrierKind::ProductPublication
BarrierKind::QuerySnapshotPublication
BarrierKind::RenderSubmit
BarrierKind::GenerationFinalization
BarrierKind::ReplayNetworkCapture
```

The scheduler also uses public `anyhow` results, process-global telemetry, and a
hard-coded slow-log exception for a render-submit node.

Target `runen_schedule`:

- owns schedule/set labels, access declarations, conflict detection, deterministic
  DAG/stage/wave formation, registered context-generic systems, and plan reports;
- uses structured `ScheduleBuildError`, `ScheduleRunError`, and diagnostics;
- has no Runenwerk phase enum;
- has no product-specific barrier enum;
- returns generic stages/waves/conflicts only;
- leaves deferred-command flushing to the RunenECS runtime after each accepted
  stage/wave;
- leaves publication, rendering, generation, replay, and network hooks to
  Runenwerk lifecycle policy;
- has no process-global logging/telemetry behavior;
- retires or errors on system-ID exhaustion instead of saturating reuse;
- treats labels/sets as process-local typed identities unless a separate stable
  authored key is introduced.

## Spatial findings

Current ECS publicly exports `SpatialIndex`, `SpatialHashIndex`, and
`SpatialHashConfig`. The index stores `geometry::Aabb3` and is embedded in
`World`; entity despawn automatically removes spatial membership.

The workspace already has separate `domain/spatial` and `domain/spatial_index`
packages.

Decision:

- remove ECS-owned spatial indexing entirely;
- remove the `geometry` dependency from `runenecs`;
- retain generic entity/component change observation needed by an adapter;
- Runenwerk owns mapping selected components/entities into its accepted spatial
  index implementation;
- a future RunenSpatial repository requires separate investigation and authority.

## Reflection findings

The crate has an explicit `TypeRegistry`, but also a process-global
`OnceLock<Mutex<TypeRegistry>>` and global registration functions.

Current `ReflectTypeId` is a sequential runtime number, while registration names
are `&'static str`. Duplicate stable-name insertion can replace prior entries,
and lock poisoning can panic.

Decision:

- remove process-global reflection authority;
- `World` or an explicitly supplied context owns/borrows a `TypeRegistry`;
- distinguish process-local Rust `TypeId`, registry-local opaque `ReflectTypeId`,
  and validated stable schema/type keys;
- duplicate stable keys are structured errors and never silently replace;
- macros produce descriptors/registration functions but do not mutate globals;
- registry lifetime, cloning/snapshot behavior, unload/reload, test isolation, and
  persisted-schema compatibility are explicit;
- no registry-local integer is presented as a durable serialized type identity.

## Messaging findings and decisions

### Broadcast events

Current broadcast streams provide typed storage, cursors, capacity, overflow,
lifetimes, observers, diagnostics, and system params. `FrameTransient` and
end-of-frame cleanup couple the framework to engine lifecycle. Overflow may
panic, and `DropOldest` uses front removal from a vector.

Decision:

- retain typed broadcast/event streams as generic ECS capability;
- rename semantics to framework-neutral event terminology during API review;
- replace frame-specific lifetime with explicit host/runtime finalization epochs
  or retain-until-cleared/retention policy;
- Runenwerk invokes finalization at its frame boundary;
- remove panic overflow from the normal production API;
- accepted/rejected/dropped outcomes return structured facts;
- use storage suitable for front retention without O(n) removal;
- sequence exhaustion is explicit, not saturating silent reuse;
- observers remain local diagnostics/notifications, not a second mutation path.

### Work queues

Network runtime currently consumes ECS work-queue readers/writers/drainers for
inbox/outbox integration. The primitive is otherwise generic: typed FIFO,
optional capacity, backpressure, read/peek, and destructive drain.

Decision:

- retain a generic typed world message/work queue in RunenECS;
- define it as deterministic FIFO storage, not a task executor;
- preserve exact rejected payload on backpressure where ownership matters;
- make destructive drain/clear explicit administrative capabilities;
- no implicit retry, acknowledgement, priority, transport, or worker semantics;
- network policy remains Runenwerk-owned.

### Tick buffers

Current tick buffers own tick-indexed buckets, current/finalized tick state,
provenance, retention, deduplication, and system params. Current consumers are
network prediction/runtime paths, and `World` also owns engine-style frame/tick
counters.

Decision:

- remove tick-buffer lifecycle from RunenECS core;
- move the current capability to Runenwerk simulation/network integration,
  initially under `engine_sim` or another reviewed Runenwerk owner;
- RunenECS exposes generic events, queues, resources, and change observation;
- do not manufacture a universal epoch-buffer abstraction without a second
  non-network consumer;
- tick/provenance/wire compatibility remain simulation/network policy.

## Change tracking and extraction findings

Query-local `Added<T>`/`Changed<T>` semantics are legitimate ECS behavior and use
world/archetype change ticks.

The world-level structural extraction layer currently includes:

- engine tick and frame windows;
- `u64::MAX` sentinel dimensions;
- component/resource local integer keys;
- ownership filters;
- arbitrary interest filters;
- entity/resource owner routing;
- network/editor-shaped structural batches.

Decision:

RunenECS retains:

- local monotonic change tick/sequence;
- component/resource/entity added/changed/removed facts;
- query-local change filters;
- a bounded generic change journal/cursor only if current consumers require
  complete structural history after the consumer inventory.

Runenwerk retains:

- frame indices;
- simulation tick types;
- ownership/authority filtering;
- interest policy;
- replication snapshots/deltas;
- replay retention and file formats;
- editor synchronization policy;
- mapping local entities/types to stable product/network IDs.

If the generic journal is retained, its API uses one local `ChangeSequence`
rather than mixed frame/tick sentinel windows. Journal retention and cursor-too-
old outcomes are explicit.

## Ownership findings and decision

Current ECS ownership provides owners with `Active`/`Observer` roles, entity and
resource targets, routing queries, transfer logs, and world-owned/unowned states.
Network runtime creates owners for connections and routes network targets through
this API.

This is application/network authority policy rather than ECS storage semantics.

Decision:

- remove owner-role, owner-routing, resource-owner, and transfer-log authority
  from RunenECS;
- move the current implementation to Runenwerk networking/product integration or
  replace it with product-owned components/resources;
- RunenECS remains capable of storing user-defined ownership components and
  querying/change-tracking them without understanding their semantics;
- structural change extraction no longer depends on ownership.

## Engine and network consumers

Confirmed direct package consumers include:

```text
engine -> ecs, scheduler
engine_sim -> ecs::World for codec contracts
engine_net -> ecs and engine_sim
engine_replay -> engine_sim
```

Engine networking directly consumes ECS work queues and owner routing.
`engine_net` owns replication descriptors, network identity, interest policy,
authority model, prediction, and network entity mapping. These remain outside
RunenECS.

Applications primarily consume ECS through `engine`; exact direct manifest/source
consumers require local verification.

## Macro findings and decisions

Current derives are hard-wired to the package name `ecs`. The future macros must
resolve `runenecs` correctly across renamed dependencies.

Decisions:

- `runenecs_macros` remains a separate proc-macro package;
- derives use public RunenECS contracts or a narrow doc-hidden bridge;
- no hidden global reflection registration;
- bundle derive participates in atomic preflight/commit;
- system-param derive is the supported extension path over sealed/unsafe internals;
- generic parameters, where clauses, visibility, crate renaming, and error spans
  are tested;
- an external downstream package owns compile-pass and compile-fail conformance;
- no macro emits Runenwerk paths.

## Error and panic findings

Current public/runtime paths mix structured errors, `anyhow`, `expect`, `assert`,
and configured panic overflow.

Target:

```text
runenecs public errors -> structured thiserror-style enums
runen_schedule errors  -> structured schedule build/run errors
Runenwerk composition  -> may wrap with anyhow at app boundaries
```

Ordinary stale entity, missing component/resource, registration conflict,
capacity, schedule cycle, invalid access, command, and setup failures do not
panic.

Panic policy for a user system is explicit: whether caught, propagated, or
terminalized is decided before parallel execution work. The initial serial
reference may propagate unwind according to Rust defaults but must not leave
internal command/borrow state reusable as success.

## Telemetry findings and decision

Both ECS and scheduler use process-global atomics and wall-clock measurements.
These are observational, but they complicate test isolation and library
embedding.

Decision:

- remove process-global telemetry as framework authority;
- retain deterministic counters in operation/plan reports where useful;
- benchmarks and Runenwerk integration measure wall-clock time externally;
- a future optional observer/sink requires explicit instance ownership and must
  not affect behavior;
- no hard-coded system-name logging exceptions.

## Documentation findings

Current ECS docs are useful but stale or incompatible with extraction:

- architecture states system arity eight while source supports sixteen;
- current docs treat spatial indexing as core ECS ownership;
- feature docs call reflection editor-facing/out-of-scope despite implemented
  global reflection;
- event terminology predates the broadcast/work-queue/tick-buffer split;
- SDF-first docs mix engine lifecycle and product publication into ECS/scheduler
  authority;
- networking inventories correctly show ownership/tick-buffer/change extraction
  as bridge foundations but not final repository ownership.

During extraction, reusable ECS docs move to RunenECS. Runenwerk retains engine,
network, replay, and migration authority.

## Move/stay/redesign/delete matrix

| Responsibility | Disposition | Final owner |
|---|---|---|
| Entity allocator/world/storage | Redesign safety then move | RunenECS |
| Component/resource/bundle traits | Redesign atomicity/API then move | RunenECS |
| Queries and filters | Seal unsafe core, then move | RunenECS |
| System params/runtime bridge | Redesign unsafe extension/errors, then move | RunenECS |
| Deferred commands | Redesign atomicity/failure, then move | RunenECS |
| Broadcast/event streams | Redesign lifecycle/overflow, then move | RunenECS |
| Generic FIFO work queues | Redesign payload/backpressure API, then move | RunenECS |
| Tick buffers | Move out of ECS before extraction | Runenwerk simulation/network |
| Change ticks/query filters | Move | RunenECS |
| Generic change journal | Retain only after consumer proof; redesign | RunenECS or delete |
| Frame/tick extraction windows | Remove from ECS | Runenwerk |
| Ownership registry/routing | Remove from ECS | Runenwerk network/product |
| Component secondary indexes | Review and likely move | RunenECS |
| Geometry/spatial index | Delete from ECS | Runenwerk spatial adapter |
| Explicit reflection registry | Redesign then move | RunenECS |
| Global reflection registry | Delete | none |
| ECS derives | Redesign/package rename then move | RunenECS macros |
| Generic DAG/access/stage planning | Redesign then move | RunenSchedule package |
| Engine phases/product barriers | Remove from scheduler | Runenwerk lifecycle |
| Global telemetry/log exceptions | Delete from framework | Runenwerk/benchmarks |
| Replication/network/replay | Stay | Runenwerk net packages |
| Original packages after cutover | Delete | none |
| Compatibility package names | Do not create | none |

## Independent conformance requirements

Before extraction, prove without Runenwerk:

- entity allocation, stale/double-free, generation exhaustion, and slot retirement;
- atomic spawn/bundle insert/remove behavior;
- component/resource lifecycle;
- dense/archetype storage invariants;
- query read/write/filter/change behavior;
- Miri/sanitizer coverage of query/system-param/command unsafe boundaries;
- deferred command ordering and failure isolation;
- event retention/cursor/overflow behavior;
- FIFO queue ordering/backpressure/exact rejected payload;
- explicit reflection registry isolation and duplicate errors;
- schedule labels/sets/cycles/conflicts/deterministic stages;
- no product phase/barrier vocabulary in `runen_schedule`;
- external derives and package-renaming support;
- no geometry dependency;
- no Runenwerk dependency;
- stable/MSRV validation and representative benchmarks.

At least one standalone simulation example must use only public RunenECS APIs.

## Required local completion gate

Before implementation activation, run:

```text
git status --short --branch
git rev-parse HEAD
cargo metadata --format-version 1 --locked
cargo tree -p ecs
cargo tree -i ecs --workspace
cargo tree -p scheduler
cargo tree -i scheduler --workspace
rg -n '^\s*(pub\s+)?(unsafe\s+)?trait|unsafe\s*\{' domain/ecs domain/scheduler domain/ecs_macros
rg -n '\becs\b|ecs::|scheduler::' --glob Cargo.toml --glob '*.rs' .
find domain/ecs domain/scheduler domain/ecs_macros -type f | sort
cargo test -p ecs --all-features --locked
cargo test -p scheduler --all-features --locked
cargo clippy -p ecs -p scheduler -p ecs_macros --all-targets --all-features --locked -- -D warnings
```

Also run the repository-authoritative MSRV command and Miri/sanitizer commands if
already supported. If not supported, the decision-closure phase must add a
bounded unsafe-validation plan before implementation.

## PT-RUNENECS-002 decision-closure requirements

The next phase must convert this report into one exact implementation sequence,
including:

1. unsafe trait sealing/bridge plan;
2. entity generation/exhaustion contract;
3. atomic bundle/spawn/command design;
4. structured error taxonomy;
5. explicit reflection registry API;
6. event/work-queue target API;
7. tick-buffer and ownership migration targets in Runenwerk;
8. generic change journal keep/delete decision from actual consumers;
9. scheduler product-vocabulary removal;
10. telemetry removal/replacement;
11. macro migration and downstream conformance;
12. exact file/phase splits so one PR does not rewrite the entire ECS at once.

No source movement or external repository creation is authorized by this
investigation.

## Stop conditions

Stop implementation if:

- safe public APIs still depend on untrusted query/system-param unsafe
  implementations;
- entity generation can silently saturate and reuse;
- bundle/command mutation can leave partial state without an explicit contract;
- scheduler still exposes Runenwerk phases/barriers;
- ECS still depends on geometry or owns spatial indexes;
- reflection still has process-global mutation;
- ownership/tick-buffer/network policy remains in core ECS;
- public API still erases branchable failures into `anyhow`;
- local consumer inventory reveals persisted formats using raw entity/type IDs;
- parallel execution is introduced before serial equivalence and soundness proof;
- final cutover requires duplicate/compatibility packages.

## Gate status

```text
Core source/architecture inspection: complete for ownership decisions
Unsafe boundary inspection: complete for identified public gates
Scheduler semantic inspection: complete for product-coupling decision
Spatial decision: complete
Reflection decision: complete
Messaging decision: complete
Ownership decision: complete
Change/replication split: complete
Manifest/selected consumer inspection: complete
Exact file/test/consumer count: local verification required
Command validation: not run; connector limitation
Complete investigation gate: complete subject to mandatory local inventory/baseline
Complete design gate: not yet implementation-complete; PT-RUNENECS-002 required
External extraction authorization: blocked
```

## Next action

After the repository-family charter and this investigation are validated and
reviewed, execute `PT-RUNENECS-002` as a bounded decision/specification PR. Do not
start ECS source movement or repository creation yet.
