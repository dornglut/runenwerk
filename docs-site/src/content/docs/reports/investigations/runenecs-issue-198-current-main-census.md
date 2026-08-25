---
title: RunenECS Issue 198 Current-Main Census
description: Command-verified current-main census and documentation-only authority reconciliation for the RunenECS boundary.
status: active
owner: ecs
layer: investigation
canonical: true
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

## Authority and scope

This report is the current-main evidence record for GitHub issue `#198`. It is
documentation only. It does not authorize implementation, package renames,
source movement, dependency changes, extraction state, compatibility paths, or
creation of the R1 issue.

The resolved base is:

```text
origin/main = 25c20a8b7643dc391ec49d870b24458767dd6033
```

That is the expected accepted base. The census branch started clean from that
commit; no unmerged feature branch was used as authority. The issue amendments
also establish that #200 and #201 are closed not-planned, PR #202 is closed
unmerged, and no external RunenScheduler repository or dependency is accepted.

## Package, dependency, and consumer closure

`cargo metadata --format-version 1 --locked` reports the current workspace
packages and direct edges:

| Current package | Manifest | Direct boundary facts |
|---|---|---|
| `ecs` | `domain/ecs/Cargo.toml` | depends on `ecs_macros`, `geometry`, and `scheduler`; no declared MSRV |
| `ecs_macros` | `domain/ecs_macros/Cargo.toml` | proc-macro crate; depends on `proc-macro-crate`, `proc-macro2`, `quote`, and `syn`; no declared MSRV |
| `scheduler` | `domain/scheduler/Cargo.toml` | depends on `anyhow` and `tracing`; no declared MSRV |

The inverse workspace closure is:

```text
ecs         <- editor_inspector, engine, engine_net, engine_sim,
               native_tablet_input, runenwerk_draw, runenwerk_editor, scene,
               ui_app_integration
ecs_macros  <- ecs
scheduler   <- ecs, engine
```

The exact commands were:

```text
cargo tree -p ecs --locked
cargo tree -i ecs --workspace --locked
cargo tree -p scheduler --locked
cargo tree -i scheduler --workspace --locked
```

The first three current packages form one mixed workspace boundary. `ecs ->
scheduler` is a real direct edge, and `engine` directly consumes scheduler
facilities. The target boundary therefore removes the ECS-to-generic-scheduler
edge; it does not replace it with `runen-scheduler`.

## Deterministic source inventory and public surface

```text
find domain/ecs domain/ecs_macros domain/scheduler -type f | sort
```

returned 128 files: 108 under `domain/ecs`, 3 under `domain/ecs_macros`, and 17
under `domain/scheduler`. The inventory includes ECS storage, bundles, commands,
queries, reflection, system/runtime code, world messaging/change/ownership
facilities, spatial indexing, tests, examples, and four Criterion benchmark
targets.

`domain/ecs/src/lib.rs` publicly reexports bundles, commands, components,
resources, entities, allocator/errors, spatial indexes, queries, all reflection
families, system runtime types, world runtime/state, messaging, ownership, and
change-related types. `domain/ecs/src/prelude.rs` repeats a broad version of
that surface, including `BroadcastReader`/`Writer`, work queues, tick buffers,
ownership types, `Runtime`, and `World`. `engine/src/prelude.rs` additionally
reexports ECS `Bundle`, `Component`, `Entity`, `Resource`, and `World`, plus
`scheduler::SystemSet`.

The target public topology is exactly:

```text
runenecs          library crate: ECS identity, storage/query, access/order,
                  deferred-command boundaries, reflection, and retained local
                  observation primitives
runenecs_macros   proc-macro crate: public derives/descriptors only
```

There is no `runen_schedule` crate, no generic scheduler dependency, and no
Runenwerk dependency in the standalone RunenECS proof. Current package names are
renamed only during a later accepted cutover; this report performs no rename.

## Entity identity and allocator evidence

The owner of the current identity invariant is `domain/ecs/src/entity.rs`, with
world validity in `domain/ecs/src/world/entity/lifecycle.rs` and allocator/error
surface in `domain/ecs/src/errors.rs`. Current `Entity` has public `id` and
`generation` fields. `EntityAllocator::free` accepts an arbitrary live-looking
value, increments generations with saturating arithmetic, and has no explicit
double-free or exhaustion error. A fresh `World` can produce the same
`Entity { id: 0, generation: 0 }` as another world.

The exact target contract is:

- `Entity` is an opaque, copyable, comparable, hashable world-local generational
  token.
- Validity is the relation between the token and its owning `World`; equal bits
  from another world are invalid and must be rejected as unknown/cross-world,
  never treated as the target world's entity.
- Stale, unknown, double-free, and invalid-world operations are fallible and do
  not mutate state. Generation exhaustion retires a slot; it never saturates
  into reusable validity.
- There is no public forgeable constructor. Diagnostic accessors exist only
  where current consumers require them and do not create a persistence contract.
- Persistence, networking, replication, replay, and editor records use their
  own stable identities. The editor's `EntityId` remains explicitly mapped to
  an ECS `Entity`; raw ECS bits are not serialized or sent over the network.
- Cross-world transfer is not a supported operation. A future globally
  namespaced identity would require a new accepted design; R1 does not infer
  one from the current fields.

The complete R1 consumer inventory is:

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

The production raw construction outside the ECS implementation is in
`engine/src/plugins/scene/ui/mod.rs`, where default sentinel entities are
constructed. Storage literals are test/internal construction sites. This is the
complete first-slice inventory for R1; the broad repository `Entity` search also
found editor UI and domain consumers that use the type opaquely and therefore
need compile verification, not identity redesign.

For reproducibility, the typed consumer file inventory returned 104 files from:

```text
rg -l '\bEntity\b|ecs::Entity|EntityAllocator|EntityError' domain engine net apps adapters --glob '*.rs' | sort
```

```text
apps/runenwerk_editor/src/editor_features/scene_commands.rs
apps/runenwerk_editor/src/editor_features/tools.rs
apps/runenwerk_editor/src/editor_features/viewport/interaction.rs
apps/runenwerk_editor/src/editor_features/viewport/tools.rs
apps/runenwerk_editor/src/editor_panels/inspector_panel.rs
apps/runenwerk_editor/src/editor_panels/viewport_panel.rs
apps/runenwerk_editor/src/editor_runtime/commands/scene_commands.rs
apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs
apps/runenwerk_editor/src/editor_runtime/ids.rs
apps/runenwerk_editor/src/editor_runtime/inspector.rs
apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
apps/runenwerk_editor/src/editor_runtime/realities/instantiated.rs
apps/runenwerk_editor/src/editor_runtime/runtime.rs
apps/runenwerk_editor/src/editor_runtime/scene.rs
apps/runenwerk_editor/src/editor_runtime/selection.rs
apps/runenwerk_editor/src/editor_runtime/tests/outliner.rs
apps/runenwerk_editor/src/editor_runtime/tests/scene_editing.rs
apps/runenwerk_editor/src/editor_runtime/tests/tool_actions.rs
apps/runenwerk_editor/src/editor_runtime/tests/transform_tools.rs
apps/runenwerk_editor/src/editor_runtime/tool_state.rs
apps/runenwerk_editor/src/persistence/files.rs
apps/runenwerk_editor/src/persistence/retained_changes.rs
apps/runenwerk_editor/src/runtime/expression/picking.rs
apps/runenwerk_editor/src/runtime/systems/frame_submit.rs
apps/runenwerk_editor/src/runtime/systems/input_bridge.rs
apps/runenwerk_editor/src/runtime/systems/picking.rs
apps/runenwerk_editor/src/runtime/viewport/picking_results.rs
apps/runenwerk_editor/src/shell/dispatch/entity_table.rs
apps/runenwerk_editor/src/shell/dispatch/inspector.rs
apps/runenwerk_editor/src/shell/providers/mod.rs
apps/runenwerk_editor/src/shell/providers/scene/entity_table.rs
apps/runenwerk_editor/src/shell/tests.rs
apps/runenwerk_editor/tests/scene_authoring_workflow_smoke.rs
apps/runenwerk_editor/tests/viewport_architecture_guards.rs
domain/ecs/benches/phase35.rs
domain/ecs/benches/phase4.rs
domain/ecs/benches/phase5b.rs
domain/ecs/benches/phase6.rs
domain/ecs/examples/phase35_profile.rs
domain/ecs/examples/phase4_profile.rs
domain/ecs/examples/phase5b_profile.rs
domain/ecs/examples/phase6_profile.rs
domain/ecs/src/bundle.rs
domain/ecs/src/commands/batch.rs
domain/ecs/src/commands/command_buffer.rs
domain/ecs/src/entity.rs
domain/ecs/src/errors.rs
domain/ecs/src/indexing/spatial_hash.rs
domain/ecs/src/indexing/spatial_index.rs
domain/ecs/src/lib.rs
domain/ecs/src/prelude.rs
domain/ecs/src/query/access_and_filters.rs
domain/ecs/src/query/orphaned.rs
domain/ecs/src/query/query_data_impls.rs
domain/ecs/src/query/traits_and_state.rs
domain/ecs/src/reflect/component_registration.rs
domain/ecs/src/storage/archetype/location.rs
domain/ecs/src/storage/archetype/registry.rs
domain/ecs/src/storage/dense/column.rs
domain/ecs/src/world/change_extraction/mod.rs
domain/ecs/src/world/change_tracking.rs
domain/ecs/src/world/component/access.rs
domain/ecs/src/world/component/introspection.rs
domain/ecs/src/world/component/registration.rs
domain/ecs/src/world/component_indexes.rs
domain/ecs/src/world/entity/access.rs
domain/ecs/src/world/entity/lifecycle.rs
domain/ecs/src/world/entity_handles.rs
domain/ecs/src/world/messaging/broadcast.rs
domain/ecs/src/world/ownership/model.rs
domain/ecs/src/world/ownership/registry.rs
domain/ecs/src/world/ownership/routing.rs
domain/ecs/src/world/spatial/indexes.rs
domain/ecs/src/world/state.rs
domain/ecs/tests/docs_examples.rs
domain/ecs/tests/query_orphaned_phase7.rs
domain/ecs/tests/query_phase6.rs
domain/ecs/tests/runtime_phase3.rs
domain/ecs/tests/stateful_component.rs
domain/ecs/tests/storage_phase6.rs
domain/ecs/tests/world.rs
domain/ecs_macros/src/lib.rs
domain/editor/editor_core/src/selection.rs
domain/editor/editor_inspector/src/bridge/ecs_bridge.rs
domain/editor/editor_inspector/src/target.rs
domain/editor/editor_persistence/src/scene_migration.rs
domain/editor/editor_scene/src/bridge/command_builder.rs
domain/editor/editor_scene/src/command.rs
domain/editor/editor_scene/src/command_descriptor.rs
domain/editor/editor_shell/src/composition/build_inspector_panel.rs
domain/editor/editor_shell/src/expression/mod.rs
domain/editor/editor_shell/src/observation/inspector.rs
domain/editor/editor_shell/src/surfaces/entity_table.rs
domain/editor/editor_shell/src/view_models/entity_table.rs
domain/editor/editor_shell/src/view_models/inspector.rs
domain/editor/editor_shell/src/workspace/surface_contract.rs
domain/editor/editor_viewport/src/hit.rs
engine/src/plugins/render/features/editor_picking/resource.rs
engine/src/plugins/scene/domain/mod.rs
engine/src/plugins/scene/runtime/overlay_ui.rs
engine/src/plugins/scene/ui/mod.rs
engine/src/plugins/world/streaming/replication.rs
engine/src/prelude.rs
net/engine_net/src/replication/extraction.rs
```

## Safety, atomicity, and reflection

The required repository-wide inventories were run with:

```text
rg -n '^\s*(pub\s+)?(unsafe\s+)?trait|unsafe\s*\{' domain/ecs domain/ecs_macros domain/scheduler
rg -n '\becs\b|ecs::|scheduler::' --glob Cargo.toml --glob '*.rs' .
rg -n 'Entity\s*\{|\.id\b|\.generation\b|EntityAllocator|EntityError' domain engine net apps adapters --glob '*.rs'
```

The query/SystemParam owner boundary is `domain/ecs/src/query/` and
`domain/ecs/src/system/`. `QueryData`, `QuerySpec`, `QueryWorldRef`, and
`SystemParam` are public, externally implementable traits whose unsafe fetch or
extract methods receive raw world/state pointers. `QueryState` caches a world
pointer; `Query`/`QueryIter` retain raw pointers and lifetime markers; tuple and
component implementations rely on derived access facts. The current safety
tests cover a narrow double-mutable rejection, not the complete unsafe contract.

Disposition: seal or privatize low-level implementation traits, keep raw-pointer
operations inside the ECS owner boundary, and expose only supported safe query
and derive-based parameter forms until an explicitly unsafe extension contract
has downstream, Miri, and sanitizer evidence. Access incompatibility must remain
distinct from semantic ordering.

Bundle insertion and `World::spawn` currently perform sequential mutation and
use an `expect` on insertion. `BatchCommands` applies prior commands before a
later failure. The exact redesign is R2/C2 atomic preflight and failure
semantics; R1 does not redesign bundles or commands.

Reflection is split between world-owned maps and
`domain/ecs/src/reflect/registry.rs`'s `GLOBAL_TYPE_REGISTRY`, a process-global
`OnceLock<Mutex<TypeRegistry>>`. Macros also generate `OnceLock` descriptors and
call global ID allocation. The target is one explicit instance-owned registry
with deterministic duplicate/stable-name policy; macros generate descriptors and
do not mutate hidden global authority. Rust `TypeId`, registry-local IDs, and
stable schema keys remain separate.

## Geometry, spatial, messaging, change, and lifecycle evidence

The spatial inventory was run with:

```text
rg -n 'SpatialIndex|SpatialHashIndex|SpatialHashConfig|geometry::Aabb3' .
```

`domain/ecs/src/indexing/` and `domain/ecs/src/world/spatial/` own geometry-based
entity indexes while `domain/spatial` and `domain/spatial_index` already exist.
The exact handoff is to remove geometry and general spatial index ownership from
RunenECS, retain generic ECS change observation only where a consumer proves it,
and let the accepted RunenSpatial/Runenwerk adapter map selected entity changes
to spatial indexes. #198 adds no RunenSpatial dependency or repository.

The messaging/change/ownership inventory was run with:

```text
rg -n 'OwnerId|OwnerRole|tick_buffer|work_queue|change_extraction|interest|replay|replication|rollback' domain engine net apps adapters
```

The disposition is:

| Current facility | Decision |
|---|---|
| ECS-local typed events/broadcast and bounded FIFO queue semantics | Stay/redesign in RunenECS if retention, cursor, overflow, and terminal behavior remain ECS-local |
| ECS-local component/resource change observation | Stay/redesign in RunenECS; no network meaning is implied |
| tick/frame provenance, work/retry/ack policy, lifecycle windows | Move to Runenwerk |
| `OwnerId`/`OwnerRole`/interest and authority routing | Move to Runenwerk policy or an explicit Runenwerk adapter |
| replication, prediction, rollback, transport, replay/history | Move to Runenwerk |
| unsupported generic queue/telemetry residue without a real consumer | Delete |

## Scheduler census and ownership

The scheduler inventory was run with:

```text
rg -n 'ExecutionPhaseKind|BarrierKind|set_slow_node_logging_enabled|frame_render_submit' .
```

Current evidence separates four concepts:

1. semantic ordering from explicit labels, sets, and before/after constraints;
2. ECS access incompatibility from read/write/drain/structural access facts;
3. deferred structural-command boundaries;
4. Runenwerk lifecycle and product barriers.

The current `domain/scheduler/src/plan.rs` does not make an access conflict into
semantic order. Its phase and barrier enums include `RenderPrepare`,
`RenderSubmit`, product publication, query snapshot publication, replay/network
capture, and generation finalization. `scheduler_core.rs`, `builder.rs`, `dag.rs`,
`utils.rs`, telemetry globals, filesystem DOT export, and the demo contain
generic or product-shaped residue. `engine` owns the live frame/fixed/render
execution calls and the application handles product/query barriers.

The complete ownership map is:

| Behavior | Owner/disposition |
|---|---|
| system identity and registration; ECS access facts; explicit ECS labels/sets and semantic ordering | RunenECS |
| access incompatibility, structural conflict classification, schedule validation, and deferred-command boundaries | RunenECS |
| deterministic standalone serial execution as correctness/reference behavior | RunenECS |
| frame, startup, fixed-step, render, network, replay, host, product-publication, and application lifecycle policy | Runenwerk |
| app-shaped phase/barrier names and product/query-snapshot publication handlers | Runenwerk |
| generic DAG/demo/DOT/global telemetry residue without a supported consumer | Delete after consumer migration |
| external `runen-scheduler`/`runen_schedule` package or dependency | Not required and not authorized |

The ECS-to-scheduler dependency is therefore removed in the target topology. No
parallel executor is required: deterministic serial execution remains the
reference implementation while any later parallel realization must prove access
safety, explicit barriers, panic/error policy, and serial equivalence.

## Validation and support census

The following commands were executed successfully on the accepted base:

```text
cargo metadata --format-version 1 --locked
cargo test -p ecs --all-features --locked       # 35 passed, 0 failed; doc-tests passed
cargo test -p scheduler --all-features --locked # 14 passed, 0 failed; doc-tests passed
cargo clippy -p ecs -p scheduler -p ecs_macros --all-targets --all-features --locked -- -D warnings
cargo validate
git diff --check
CI=true pnpm --dir docs-site build                  # 984 pages built
cargo bench -p ecs --bench phase6 --locked --no-run
cargo bench -p ecs --bench phase6 --locked -- 'w1_broad_transform_update'
```

`cargo validate` also ran the repository's workspace format, workspace tests,
workspace Clippy, documentation validation, and repository audit successfully.
The focused phase6 Criterion run completed the selected workload successfully;
its output included measured 50,000-entity and 200,000-entity cases.

Support gaps are recorded, not inferred away:

- No repository-authoritative Miri command, Miri workflow, or installed
  `cargo-miri` was found. Miri proof remains a required future safety gate.
- No repository-authoritative sanitizer command or sanitizer workflow was found;
  no sanitizer result is claimed.
- `ecs`, `ecs_macros`, and `scheduler` have no `rust-version` declaration and
  no repository MSRV command. The observed host toolchain was `rustc 1.97.1`
  on `aarch64-apple-darwin`; this is not MSRV evidence. `foundation/id`'s
  separate `rust-version = "1.85"` does not establish an ECS MSRV.
- Criterion support exists in four checked-in ECS benches. The full benchmark
  baseline and release policy remain C8 evidence, not an extraction claim.

## Canonical repair and extraction sequence

This sequence supersedes the conflicting `R1..R9` and `ECS-001..006` lists. The
old labels are retained only as traceability in the table; there is one active
sequence:

| Canonical slice | Historical mapping | Scope |
|---|---|---|
| C0 | ECS-001 | current-main census, authority binding, and complete evidence record; this report |
| C1 | R1 | opaque world-local Entity, structured errors, stale/unknown/double-free/cross-world/exhaustion behavior; first implementation slice after #198 acceptance |
| C2 | R2 | atomic structural mutation, spawn, bundle, command, and batch semantics |
| C3 | R3 | sealed query/SystemParam unsafe boundary and safety proof |
| C4 | R4 | instance-owned reflection registry and descriptor-only macros |
| C5 | R8, revised | ECS-native system/access/order/deferred-command semantics, removal of `ecs -> scheduler`, and serial reference proof |
| C6 | R5 | geometry/spatial handoff and ECS core dependency removal |
| C7 | R6/R7 | messaging/change observation split, ownership/lifecycle/network/replay separation |
| C8 | R9/ECS-004 | standalone two-crate proof, downstream conformance, focused/full benchmark, toolchain and validation baseline |
| C9 | ECS-005/ECS-006 | only after accepted independent-repository authority: source transfer, Runenwerk cutover, deletion, and provenance closeout |

C1 remains the correct first implementation slice, but no R1 issue is created by
this census. C9 is outside the current authorization.

## Exact move/stay/redesign/delete map

```text
STAY IN RUNENECS
  entity/world validity; component/resource/storage/query semantics;
  ECS-local access facts, labels, ordering, validation, deferred-command
  boundaries, deterministic serial reference; explicit reflection registry;
  proven local event/change-observation primitives.

MOVE TO RUNENWERK OR A RUNENWERK-OWNED ADAPTER
  frame/fixed/render/startup/shutdown and host lifecycle; product barriers;
  spatial index integration; ownership/interest/authority policy; tick/window
  provenance; replication/network/rollback/replay; editor/product mappings.

REDESIGN BEFORE STAYING
  Entity opacity and allocator errors; atomic structural mutation;
  QueryData/QuerySpec/QueryWorldRef/SystemParam extension boundaries;
  reflection registration; messaging retention/overflow/terminal semantics;
  ECS schedule/access/order/deferred-command API.

DELETE AFTER CONSUMER MIGRATION
  generic runen_schedule package; generic scheduler DAG/demo/DOT/filesystem
  export; app-shaped phase/barrier residue in ECS; process-global scheduler or
  telemetry switches; duplicate ECS-owned geometry/spatial indexes; unsupported
  generic queue/retry/ack residue without an accepted owner.
```

The standalone proof must show that the repaired `runenecs` and
`runenecs_macros` crates compile and operate without Runenwerk and without any
generic scheduler crate. This report is the evidence gate; it is not source
movement or implementation authorization.
