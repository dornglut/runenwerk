---
title: RunenECS Issue 198 Current-Main Census
description: Command-verified current-main evidence for the RunenECS boundary reconciliation in GitHub issue 198.
status: active
owner: ecs
layer: investigation
last_reviewed: 2026-08-25
related_docs:
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../workspace/planning/roadmap.md
---

# RunenECS Issue 198 Current-Main Census

## Role and scope

This report is the source-grounded evidence record for GitHub issue `#198`.
It does not own durable architecture, phase sequencing, activation, or delivery
state. The active boundary design owns durable RunenECS ownership; the boundary
repair execution plan owns the one canonical C0-C9 sequence; GitHub owns current
work and acceptance state.

This report does not authorize implementation, package renames, source movement,
dependency changes, external-repository population, compatibility paths, or
creation of the C1/R1 issue.

The checked-out census resolved accepted `main` as:

```text
25c20a8b7643dc391ec49d870b24458767dd6033
```

The checkout started clean from that revision. No unmerged feature branch was
used as authority. Issue amendments establish that #200 and #201 are closed
not-planned and PR #202 is closed unmerged, so no external RunenScheduler
repository or dependency is an accepted prerequisite.

## Package, dependency, and consumer closure

`cargo metadata --format-version 1 --locked` and the required dependency-tree
commands establish:

| Current package | Manifest | Direct boundary facts |
|---|---|---|
| `ecs` | `domain/ecs/Cargo.toml` | depends on `ecs_macros`, `geometry`, and `scheduler`; no declared MSRV |
| `ecs_macros` | `domain/ecs_macros/Cargo.toml` | proc-macro crate; depends on `proc-macro-crate`, `proc-macro2`, `quote`, and `syn`; no declared MSRV |
| `scheduler` | `domain/scheduler/Cargo.toml` | depends on `anyhow` and `tracing`; no declared MSRV |

Inverse workspace closure:

```text
ecs         <- editor_inspector, engine, engine_net, engine_sim,
               native_tablet_input, runenwerk_draw, runenwerk_editor, scene,
               ui_app_integration
ecs_macros  <- ecs
scheduler   <- ecs, engine
```

Commands:

```text
cargo tree -p ecs --locked
cargo tree -i ecs --workspace --locked
cargo tree -p scheduler --locked
cargo tree -i scheduler --workspace --locked
```

The current `ecs -> scheduler` edge is real. The target removes that dependency;
it does not replace it with another generic scheduler framework.

## Target package identity

Current repository-family convention and the existing proc-macro requirement bind
the eventual standalone names as:

```text
repository                    Cargo package       Rust crate
dornglut/runen-ecs            runen-ecs           runen_ecs
                              runen-ecs-macros    runen_ecs_macros
```

The proc-macro package remains separate while technically required. This report
performs no rename or source movement.

## Deterministic source and public-surface inventory

```text
find domain/ecs domain/ecs_macros domain/scheduler -type f | sort
```

returned 128 files: 108 under `domain/ecs`, 3 under `domain/ecs_macros`, and 17
under `domain/scheduler`. The inventory covers storage, bundles, commands,
queries, reflection, system/runtime code, messaging/change/ownership facilities,
spatial indexing, tests, examples, and Criterion benchmarks.

`domain/ecs/src/lib.rs` publicly reexports bundles, commands, components,
resources, entities, allocator/errors, spatial indexes, queries, reflection,
system runtime types, world runtime/state, messaging, ownership, and change
families. `domain/ecs/src/prelude.rs` exposes a similarly broad surface.
`engine/src/prelude.rs` additionally reexports ECS `Bundle`, `Component`,
`Entity`, `Resource`, and `World`, plus `scheduler::SystemSet`.

## Entity identity and allocator evidence

Current owner files are `domain/ecs/src/entity.rs`,
`domain/ecs/src/world/entity/lifecycle.rs`, and `domain/ecs/src/errors.rs`.
Current `Entity` is publicly forgeable as `{ id: u32, generation: u32 }`.
`EntityAllocator::free` accepts arbitrary live-looking values, increments
with saturating arithmetic, and has no explicit double-free or exhaustion error.
Two fresh worlds can create identical entity bits, while `World::contains` tests
only the entity value stored in that world.

Therefore index+generation alone cannot satisfy guaranteed cross-world rejection.
The C1/R1 target mechanism is:

```text
Entity = opaque WorldScopeId + slot/index + generation
World  = owns exactly one matching WorldScopeId
```

Required semantics:

- the allocator emits only entities carrying its world's scope;
- every world operation validates scope before slot/generation;
- a foreign-world entity is rejected even when slot and generation coincide with
  a live local entity;
- world scopes are checked, non-reusing process-local runtime identities;
- world-scope exhaustion fails world creation rather than wrapping or reusing;
- slots retire permanently on generation exhaustion;
- stale, unknown/cross-world, double-free, index-exhaustion, and
  generation-exhaustion operations are structured and non-mutating on failure;
- there is no public forgeable entity constructor;
- diagnostic accessors do not create persistence or wire contracts;
- WorldScopeId, slot, and generation are never stable network/persistence IDs.

The direct C1/R1 migration inventory is:

```text
domain/ecs/src/entity.rs
domain/ecs/src/errors.rs
domain/ecs/src/world/entity/lifecycle.rs
domain/ecs/src/world/entity_handles.rs
domain/ecs/src/query/orphaned.rs
domain/ecs/src/storage/dense/column.rs
domain/ecs/src/storage/archetype/location.rs
domain/ecs/src/storage/archetype/registry.rs
engine/src/plugins/scene/ui/mod.rs
engine/src/plugins/scene/domain/mod.rs
engine/src/plugins/scene/runtime/overlay_ui.rs
apps/runenwerk_editor/src/editor_runtime/ids.rs and its explicit mapping users
editor persistence's separate EntityId/SceneEntityRecordV2 paths
```

The only production raw construction identified outside ECS implementation is in
`engine/src/plugins/scene/ui/mod.rs`, where default sentinel entities are
constructed. Storage literals are internal/test sites. The broader typed search:

```text
rg -l '\bEntity\b|ecs::Entity|EntityAllocator|EntityError' domain engine net apps adapters --glob '*.rs' | sort
```

returned 104 files. Most use `Entity` opaquely and therefore require compile and
behavior verification during C1 rather than identity redesign. Editor persistence
already uses a separate `EntityId`/scene-record identity; networking extraction
also has an explicit mapping boundary rather than a stable raw-Entity contract.

## Query/SystemParam safety and structural atomicity

Required inventories included:

```text
rg -n '^\s*(pub\s+)?(unsafe\s+)?trait|unsafe\s*\{' domain/ecs domain/ecs_macros domain/scheduler
rg -n '\becs\b|ecs::|scheduler::' --glob Cargo.toml --glob '*.rs' .
rg -n 'Entity\s*\{|\.id\b|\.generation\b|EntityAllocator|EntityError' domain engine net apps adapters --glob '*.rs'
```

`QueryData`, `QuerySpec`, `QueryWorldRef`, and `SystemParam` are public,
externally implementable traits whose unsafe fetch/extract paths receive raw
world or state pointers. `QueryState` caches a world pointer; `Query` and
`QueryIter` retain raw pointers and lifetime markers; tuple/component
implementations rely on declared access facts. Current tests cover only a narrow
subset of the unsafe contract.

Disposition: seal or privatize low-level implementation traits initially, keep
raw-pointer operations inside the ECS owner boundary, and expose supported safe
query and derive-based parameter forms. Any later unsafe extension contract needs
downstream, Miri, sanitizer, and explicit safety documentation.

Bundle insertion and `World::spawn` currently mutate sequentially and use an
`expect` on insertion. `BatchCommands` applies earlier commands before a later
failure. C2 therefore owns atomic preflight, failure, rollback/non-rollback, and
batch semantics; C1 does not silently absorb that redesign.

## Reflection evidence

Reflection is split between world-owned maps and
`domain/ecs/src/reflect/registry.rs`'s process-global
`GLOBAL_TYPE_REGISTRY` (`OnceLock<Mutex<TypeRegistry>>`). Macros also generate
`OnceLock` descriptors and use global ID allocation.

Target distinction:

```text
Rust TypeId        process-local concrete type identity
registry identity  explicit instance-local identity
stable schema key  separately governed persistence/schema identity
```

C4 removes hidden mutable registration authority. Macros generate descriptors;
they do not establish global registry state.

## Geometry and spatial evidence

```text
rg -n 'SpatialIndex|SpatialHashIndex|SpatialHashConfig|geometry::Aabb3' .
```

shows geometry-based entity indexes under ECS while separate spatial ownership
already exists. RunenECS therefore drops general geometry/spatial-index
ownership. It may retain generic local change facts; Runenwerk integration maps
selected ECS changes into accepted RunenSpatial facilities. No new RunenSpatial
repository or dependency is authorized by #198.

## Messaging, change, networking, replay, and lifecycle evidence

The required inventory included:

```text
rg -n 'OwnerId|OwnerRole|tick_buffer|work_queue|change_extraction|interest|replay|replication|rollback' domain engine net apps adapters
```

Evidence-backed disposition:

| Current facility | Owner/disposition |
|---|---|
| ECS-local typed events/broadcast and bounded FIFO semantics | RunenECS only when retention, cursor, overflow, terminal, and recovery behavior are ECS-local |
| ECS-local component/resource change observation | RunenECS; no network meaning implied |
| tick/frame provenance, lifecycle windows | Runenwerk |
| gameplay ownership/relevancy/world policy | Runenwerk/application |
| concrete ECS-to-network identity/state mapping | Runenwerk/application integration |
| protocol/schema identity and replication consistency | RunenNet |
| session/authority, delivery, ACK/resync/recovery, transport-independent contracts | RunenNet |
| prediction/interest semantics | RunenNet when separately accepted by RunenNet authority |
| archival/editor replay formats and retention | Runenwerk/application |
| unsupported generic queue/retry/ack residue without an owner | Delete |

This preserves the standalone RunenNet boundary: Runenwerk is a downstream
integration host, not the owner of reusable networking semantics.

## Scheduler census and ownership

```text
rg -n 'ExecutionPhaseKind|BarrierKind|set_slow_node_logging_enabled|frame_render_submit' .
```

Current evidence separates:

1. semantic ordering from explicit labels, sets, and before/after constraints;
2. ECS access incompatibility from read/write/drain/structural access facts;
3. deferred structural-command boundaries;
4. Runenwerk lifecycle and product barriers.

`domain/scheduler/src/plan.rs` does not convert an access conflict into semantic
order. Its phase/barrier enums nevertheless include render, product publication,
query-snapshot publication, replay/network capture, and generation-finalization
policy. `scheduler_core.rs`, `builder.rs`, `dag.rs`, `utils.rs`, telemetry globals,
filesystem DOT export, and the demo contain generic or product-shaped residue.
`engine` owns live frame/fixed/render execution calls and application product
barriers.

Ownership/disposition:

| Behavior | Owner/disposition |
|---|---|
| system identity and registration; ECS access facts; explicit ECS labels/sets and semantic ordering | RunenECS |
| access incompatibility, structural conflict classification, schedule validation, deferred-command boundaries | RunenECS |
| deterministic standalone serial execution as correctness/reference behavior | RunenECS |
| frame/startup/fixed/render/host/product lifecycle and application barriers | Runenwerk |
| app-shaped phase/barrier names and product/query-snapshot handlers | Runenwerk |
| unsupported generic DAG/demo/DOT/filesystem/global telemetry residue | Delete after consumer migration |
| external `runen-scheduler`/`runen_schedule` dependency | Not required or authorized |

C8 removes the `ecs -> scheduler` dependency only after C7 has separated lifecycle,
networking, replay, ownership, and host policy. Any later parallel executor is an
optimization and must be observationally equivalent to accepted serial ECS
semantics.

## Validation and support evidence

The checked-out executor reported successful execution during #198 preparation of:

```text
cargo metadata --format-version 1 --locked
cargo tree -p ecs --locked
cargo tree -i ecs --workspace --locked
cargo tree -p scheduler --locked
cargo tree -i scheduler --workspace --locked
cargo test -p ecs --all-features --locked
cargo test -p scheduler --all-features --locked
cargo clippy -p ecs -p scheduler -p ecs_macros --all-targets --all-features --locked -- -D warnings
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
cargo bench -p ecs --bench phase6 --locked --no-run
cargo bench -p ecs --bench phase6 --locked -- w1_broad_transform_update
```

Reported focused results were 35 ECS tests and 14 scheduler tests passing, with
doc-tests passing. Criterion phase6 compilation and the selected workload passed.
The exact merge-readiness revision and exact-head CI evidence are owned by the PR,
not by this report. Because the candidate is documentation-only, these source
facts remain tied to the accepted source base unless a later code-bearing head
changes them.

Support gaps are explicit:

- no repository-authoritative Miri command/workflow or installed `cargo-miri` was
  found;
- no repository-authoritative sanitizer command/workflow was found;
- `ecs`, `ecs_macros`, and `scheduler` declare no ECS MSRV and the repository has
  no ECS MSRV command;
- observed `rustc 1.97.1` on `aarch64-apple-darwin` is not MSRV evidence;
- Criterion support exists, but the full release benchmark baseline remains a C9
  conformance obligation.

## Move / stay / redesign / delete map

```text
STAY IN RUNENECS
  entity/world validity; component/resource/storage/query semantics;
  ECS-local access facts, explicit ordering/sets, validation, deferred-command
  boundaries, deterministic serial reference execution; explicit reflection;
  proven local event/change-observation primitives.

MOVE / MAP OUT OF ECS
  Runenwerk: frame/fixed/render/startup/shutdown lifecycle; product barriers;
             spatial integration; gameplay ownership/relevancy; tick/window
             provenance; ECS-to-network mapping; archival/editor replay.
  RunenNet:  reusable protocol/schema identity; replication consistency;
             session/authority; delivery/recovery; transport-independent
             networking and separately accepted prediction/interest semantics.

REDESIGN BEFORE STAYING
  Entity opacity/world scope/allocator errors; atomic structural mutation;
  QueryData/QuerySpec/QueryWorldRef/SystemParam extension boundaries;
  reflection registration; messaging retention/overflow/terminal semantics;
  ECS schedule/access/order/deferred-command API.

DELETE AFTER CONSUMER MIGRATION
  unsupported generic residue from current `domain/scheduler` / package
  `scheduler`, including generic DAG/demo/DOT/filesystem/global telemetry where
  no supported owner remains; duplicate ECS-owned geometry/spatial indexes;
  unsupported generic queue/retry/ack residue.
```

There is no existing `runen_schedule` package to delete.

## Evidence relationship to the canonical plan

This report does not duplicate the phase sequence. The active execution plan owns
C0-C9. Current evidence supports C1/R1 as the first implementation slice, C7
lifecycle/network separation before C8 scheduler decontamination, and C9
standalone conformance after all internal repairs.

The standalone proof must validate `runen-ecs` / `runen_ecs` and
`runen-ecs-macros` / `runen_ecs_macros` without Runenwerk and without a generic
scheduler dependency. External repository population and cutover remain a
separately accepted post-C9 delivery boundary.
