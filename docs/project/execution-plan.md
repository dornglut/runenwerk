# Scene Manager Abstraction Execution Plan

## Objective
Deliver a higher-level application and scene API so example/game entrypoints no longer manage boilerplate runtime wiring.

Target outcomes:
- scene registration and transitions use automatic scene handles instead of manually managed ids
- tracing setup and event-loop lifecycle move into `App` automatically
- `Box::new(Plugin)` is no longer required in user code
- app startup becomes `App::new()...run()`
- plugin configure/setup can be auto-wired by the app runtime
- scene example `main.rs` is reduced to declarative registration
- resource loading and hot-reload are handled through ECS resources

## Project Areas Requiring Extension

### 1) `ecs` crate (new resource model)
Why:
- current ECS has entity/component storage only; no first-class resource storage

Needs extension:
- add `Resource` trait and typed resource storage to `ecs::World`
- support `insert/get/get_mut/remove` resource APIs
- add tests for resource borrow, replacement, and lifecycle behavior

Likely files:
- `ecs/src/lib.rs`
- `ecs/src/world/mod.rs`
- `ecs/tests/` (new tests)

### 2) `engine` app lifecycle abstraction
Why:
- `main.rs` currently owns tracing guard creation and event loop creation/run

Needs extension:
- move tracing setup into `App::run()` (configurable defaults)
- hide `EventLoop::new()` and `run_app(...)` behind `App::run()`
- preserve manual escape hatches for advanced use-cases

Likely files:
- `engine/src/platform/app.rs`
- `engine/src/main.rs`
- `game/src/main.rs`
- `engine/examples/**/main.rs`

### 3) plugin ergonomics and auto-registration
Why:
- users currently manage plugin boxing and explicit plugin ordering in entrypoints

Needs extension:
- allow `add_plugin(MyPlugin)` and/or `add_plugins((A, B, C))` without explicit boxing
- add app-level defaults for core plugin stack
- auto-configure and auto-setup plugin lifecycle from one entrypoint

Likely files:
- `engine/src/runtime/plugin.rs`
- `engine/src/platform/app.rs`
- `engine/src/plugins/mod.rs`

### 4) scene manager identity/handle model
Why:
- scene logic currently relies on fixed enum ids and manual mapping

Needs extension:
- introduce `SceneHandle` as stable runtime handle
- introduce scene registration API that returns/records handles
- map external labels to handles automatically
- transition APIs operate on handles (with compatibility adapters for old ids during migration)

Likely files:
- `engine/src/plugins/scene/domain/mod.rs`
- `engine/src/plugins/scene/domain/manager.rs`
- `engine/src/plugins/scene/domain/registry.rs`
- `engine/src/plugins/scene/mod.rs`

### 5) resource-driven asset loading and hot-reloading
Why:
- template/reload state is currently spread across plugin-local fields and engine subsystems

Needs extension:
- move template catalogs, file-modified metadata, and reload status into ECS resources
- add unified reload coordinator resource for scene/ui/template assets
- keep shader/model hot-reload compatible, then incrementally migrate state into resource layer

Likely files:
- `engine/src/plugins/ui/domain/template.rs`
- `engine/src/plugins/scene/manifest/mod.rs`
- `engine/src/plugins/shared/reload.rs`
- `engine/src/runtime/mod.rs`

### 6) scene-manager example and game migration
Why:
- examples should validate the new abstractions and stay minimal

Needs extension:
- refactor `engine/examples/scene_manager_ui/main.rs` to declarative-only scene registration
- keep asset-driven scene definitions in `engine/examples/scene_manager_ui/assets/`
- update game runtime entrypoint to new `App` lifecycle API

Likely files:
- `engine/examples/scene_manager_ui/main.rs`
- `engine/examples/scene_manager_ui/README.md`
- `game/src/main.rs`

### 7) tests, docs, and compatibility
Why:
- cross-cutting API changes need migration safety and clear guidance

Needs extension:
- add unit/integration tests for handles, registration, and resource reload
- add regression tests for old behavior where compatibility layers are kept
- update engineering docs and example docs

Likely files:
- `engine/src/plugins/**/tests.rs`
- `ecs/tests/**`
- `docs/project/engineering-guidelines.md`
- `docs/project/planned-features.md`

## Phased Delivery Plan

### Phase 0: API Design Lock
Deliverables:
- finalize `App` public API (`run`, plugin registration, scene registration)
- finalize `SceneHandle` lifecycle and compatibility rules
- finalize ECS resource API surface

Exit criteria:
- approved API doc and migration strategy

### Phase 1: App Runtime Abstraction
Deliverables:
- implement `App::run()` with internal tracing + event loop management
- remove tracing/event-loop boilerplate from engine/game/example mains

Exit criteria:
- `engine`, `game`, and example binaries run with no manual event loop code

### Phase 2: Plugin Ergonomics
Deliverables:
- remove required `Box::new` from user-facing API
- support auto plugin setup/configure path from `App`

Exit criteria:
- user code compiles with ergonomic plugin registration only

### Phase 3: Scene Handle Infrastructure
Deliverables:
- add handle-based scene registry and transitions
- add compatibility adapter from legacy scene ids

Exit criteria:
- scene transitions work without direct enum-id coupling in user code

### Phase 4: ECS Resource Layer
Deliverables:
- add resource storage APIs to ECS world
- move scene/template runtime state to resources

Exit criteria:
- reload + scene state use resource storage instead of ad hoc plugin fields

### Phase 5: Hot-Reload Unification
Deliverables:
- unify scene/template reload tracking via resource-driven coordinator
- keep existing shader/model reload behavior working

Exit criteria:
- scene/ui template hot-reload works through unified resource state

### Phase 6: Migration and Cleanup
Deliverables:
- migrate `scene_manager_ui` example to final API shape
- remove obsolete helper code and compatibility shims no longer needed
- update docs/tests

Exit criteria:
- docs and examples match final API exactly
- workspace checks and tests pass

## Risks and Mitigations

- Risk: large API breakage across engine/game/examples.
Mitigation: ship compatibility adapters for one migration phase; remove later.

- Risk: resource borrowing conflicts in ECS.
Mitigation: implement explicit borrow tests and deterministic access patterns.

- Risk: scene handle instability across reloads.
Mitigation: define stable handle generation rules and test persistence semantics.

- Risk: event-loop abstraction limits advanced users.
Mitigation: keep optional advanced constructor/runner escape hatch.

## Definition of Done

- `main.rs` for engine/game/example uses only high-level `App` API and `.run()`
- scene registration and transitions are handle-driven
- plugin registration is ergonomic (no mandatory boxing)
- tracing + event loop setup are internalized by app runtime
- scene/template resource loading and reload tracking run via ECS resources
- tests and docs are updated and green across workspace

## Implementation Status (2026-02-23)

Completed:
- `App::run()` now owns tracing setup and event loop execution.
- `App::add_plugin(...)` supports non-boxed plugin registration.
- `App::new()` now auto-loads default engine plugins.
- Example `scene_manager_ui/main.rs` now declares only scene templates and calls `.run()`.
- Scene template flow and hot-reload moved into core `ScenePlugin` (`template_flow` module), backed by ECS resources.
- Scene catalog now provides runtime `SceneHandle` mapping for registered scenes.
- ECS world resource API (`insert/get/get_mut/remove`) and tests are in place.
- Game command layer is now label-based (`set_scene`, `set_world`, `push_overlay`) and resolves to registered handle-scenes first, with legacy `SceneId` fallback.
- Game entrypoint now uses `App::new().add_plugin(...).run()` without boxed plugin vectors.
- Core scene domain now accepts label-based transition commands (`ReplaceWorldByLabel`, `ReplaceOverlayByLabel`, `PushOverlayByLabel`) and resolves labels inside `SceneManager`.
- Scene plugin exposes label-first helper APIs (`switch_scene_by_id`, `set_world_by_id`, `push_overlay_by_id`) used by game command execution.
- Workspace checks are green (`cargo check`, `cargo test -p ecs`, `cargo test -p engine`, `cargo test -p game`).

Status:
- Execution plan objectives are complete for the requested scope.

Optional follow-up cleanup:
- Remove remaining legacy `SceneId` internals and migrate world/overlay runtime transitions to fully dynamic label/handle metadata.
