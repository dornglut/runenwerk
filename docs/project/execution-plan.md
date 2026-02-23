# Execution Plan

## Purpose
Define what should be plugin-owned versus infrastructure-owned so engine/game decoupling stays clean without over-pluginizing core crates.

## Execution Status (2026-02-23)
- Phase 1 (Runtime Boundary Hardening): completed.
  - `runtime` no longer owns scene/input/time modules directly.
  - `runtime` is host-only (`Engine`, `EngineData`, plugin setup/execution, update/resize).
- Phase 2 (Scene Manifest Ownership Move): completed.
  - `scene_manifest` implementation moved to `engine/src/plugins/scene/manifest/mod.rs`.
- Phase 3 (Utils Split By Ownership): completed.
  - reload/watch diagnostics helpers moved to `engine/src/plugins/shared/reload.rs`.
  - `engine::utils` now contains tracing-focused shared infrastructure.
- Phase 4 (Grid Integration Plugin): completed (initial implementation).
  - added `GridPlugin` in `engine/src/plugins/grid/mod.rs`.
  - plugin is included in `default_engine_plugins()` and prepares grid/world render chunk settings each frame.
- Phase 5 (Scheduler Diagnostics Plugin): completed (optional implementation).
  - added `SchedulerDiagnosticsPlugin` in `engine/src/plugins/scheduler_diagnostics/mod.rs`.
  - plugin is optional via `default_engine_plugins_with_diagnostics()`.
- Phase 6 (Facade Cleanup): completed.
  - removed legacy public root facades (`engine::render`, `engine::ui`, `engine::scene_manifest`).
  - migrated game crate to plugin-domain imports (`engine::plugins::{render,ui,scene}::*`).
  - removed runtime re-export leakage of scene/input/time domain types.
- Validation:
  - `cargo test -p engine -p game` passed after migration.

## Architecture Decision

### Keep As Infrastructure (Not Plugins)
- `engine::runtime` core loop and plugin host:
  - owns `Engine`, `EngineData`, plugin registration/setup execution, frame update, resize, and app integration hooks.
  - reason: plugins need a host; the host itself should not be a plugin.
- `engine::utils` shared low-level helpers:
  - tracing setup and tracing-related shared infra.
  - reason: cross-cutting support code used by multiple plugins.
- `scheduler` crate:
  - remains standalone scheduling primitive.
  - reason: plugin orchestration depends on scheduler; making scheduler a plugin creates inverted ownership.
- `grid` crate:
  - remains standalone domain library.
  - reason: data structures/algorithms are reusable engine primitives, not runtime behavior.

### Plugin-Owned
- `plugins/render` render pipeline, frame graph usage, shader/model reload integration.
- `plugins/ui` UI ECS state, input/render UI stages, UI template handling.
- `plugins/scene` scene transitions, lifecycle routing, scene registry, scene manifest consumption.
- `plugins/input` per-frame input finalization and input-related systems.
- `plugins/time` time tick/update systems.
- Optional wrappers:
  - `GridPlugin` for grid-specific runtime systems, while `grid` crate stays a pure dependency.
  - `SchedulerDiagnosticsPlugin` for scheduling metrics/diagnostics, while `scheduler` crate stays core.

## Target Layout
```text
engine/src/
  plugins/
    render/
    ui/
    scene/
      manifest/        # scene manifest loader + descriptor mapping
    shared/
    input/
    time/
    grid/
    scheduler_diagnostics/
  runtime/
    mod.rs             # Engine host + shared runtime types only
    plugin.rs          # EnginePlugin trait + schedule builder
  utils/               # shared non-domain helpers
```

## Phased Execution

### Phase 1: Runtime Boundary Hardening
- Restrict `runtime` to host responsibilities only.
- Ensure scene/input/time behavior is only plugin-owned.
- Keep compatibility re-exports where needed short-term.
- Validation:
  - `cargo test -p engine -p game`
  - plugin order/dependency tests green.

### Phase 2: Scene Manifest Ownership Move
- Move `scene_manifest` logic under `plugins/scene/manifest`.
- Update scene registry in `plugins/scene` to use local manifest module.
- Leave `engine::scene_manifest` as temporary facade for one release window if needed.
- Validation:
  - scene registry tests pass.
  - manifest parsing tests pass.

### Phase 3: Utils Split By Ownership
- Keep only true shared helpers in `engine::utils`.
- Move domain-specific reload/diagnostics helpers into owning plugin folders.
- Standardize status payload shape once in a shared type used by plugin diagnostics.
- Validation:
  - no plugin imports another plugin’s private helpers.
  - tests pass with no behavior regressions.

### Phase 4: Grid Integration Plugin (Without Pluginizing `grid` Crate)
- Add `GridPlugin` to wire grid systems/resources into the engine schedule.
- Keep `grid` crate API pure and runtime-agnostic.
- Validation:
  - grid-related systems run via plugin registration only.
  - game crate composes `GridPlugin` explicitly.

### Phase 5: Scheduler Diagnostics Plugin (Without Pluginizing `scheduler` Crate)
- Add optional `SchedulerDiagnosticsPlugin` to expose timing/order/health diagnostics.
- Keep scheduler crate as foundational dependency.
- Validation:
  - scheduler core tests unchanged.
  - diagnostics plugin can be enabled/disabled without affecting behavior.

### Phase 6: Facade Cleanup
- Remove temporary `#[path = ...]` bridge modules once all imports are plugin-domain stable.
- Remove deprecated public re-exports that leak old ownership model.
- Validation:
  - full workspace tests pass.
  - docs and module tree match actual ownership.

## Definition Of Done
- Core host code is minimal and stable in `runtime`.
- Feature behavior lives in plugin domains.
- `grid` and `scheduler` remain reusable crates, not runtime plugins.
- Game crate composes behavior by adding plugins, not by importing engine internals.
- Documentation reflects final boundaries and no stale migration notes remain.
