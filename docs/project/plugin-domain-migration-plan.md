# Plugin Domain Migration Plan

## Goal
Finish moving engine feature domains under `engine/src/plugins` so ownership is explicit and folder layout is consistent:

- `plugins/render/`
- `plugins/ui/`
- `plugins/input/`
- `plugins/scene/`
- `plugins/time/`

This plan addresses the current split where some code still lives in shared engine roots (`engine/src/render`, `engine/src/runtime`, `engine/src/ui`) while plugin entrypoints already exist.

## Current State
- Plugin registration is centralized in `engine/src/plugins/mod.rs`.
- Plugin folders now exist and contain plugin-facing systems.
- Rendering, runtime, and UI still have substantial implementation outside plugin folders:
  - `engine/src/render/*`
  - `engine/src/runtime/*`
  - `engine/src/ui/*`

## Target State
- Each plugin folder owns:
  1. plugin type and schedule wiring,
  2. plugin-specific systems,
  3. plugin-specific services/helpers,
  4. tests co-located with plugin module.
- Cross-plugin infrastructure remains in neutral roots only when truly shared.

Target high-level layout:

```text
engine/src/plugins/
  render/
    mod.rs
    plugin.rs
    systems.rs
    graph/
    pipelines/
    upload/
    tests/
  ui/
    mod.rs
    plugin.rs
    systems.rs
    template/
    tests/
  scene/
    mod.rs
    plugin.rs
    systems.rs
    registry/
    lifecycle/
    tests/
  input/
    mod.rs
    plugin.rs
    systems.rs
  time/
    mod.rs
    plugin.rs
    systems.rs
```

## Migration Phases

### Phase 1: Stabilize Plugin Boundaries
1. Lock plugin public API to plugin entrypoints only.
2. Ensure game crate depends on `engine::plugins::*` only.
3. Add/retain plugin schedule tests in each plugin folder.

Done criteria:
- No game references to internal engine domain paths.
- Plugin schedule build tests pass.

### Phase 2: Render Domain Internal Move
1. Move render plugin execution-specific code from `engine/src/render/*` into `plugins/render/*`.
2. Keep thin compatibility re-exports temporarily for compile safety.
3. Move render tests next to `plugins/render`.

Done criteria:
- `plugins/render` owns render plugin execution path.
- Legacy render module is either removed or thin facade only.

### Phase 3: UI Domain Internal Move
1. Move UI plugin-specific pipeline and builders into `plugins/ui/*`.
2. Keep pure data schema/shared UI types separate only if shared with non-plugin code.
3. Co-locate current UI tests under `plugins/ui/tests`.

Done criteria:
- `plugins/ui` is sole owner of UI plugin behavior.
- UI stage tests pass from plugin folder.

### Phase 4: Scene + Runtime Domain Split
1. Move scene management runtime logic used by scene plugin into `plugins/scene/*`.
2. Keep engine runtime loop minimal (app loop + scheduler execution + plugin setup).
3. Remove scene-specific logic from generic runtime modules.

Done criteria:
- `plugins/scene` owns scene transition/update/lifecycle orchestration.
- `runtime` no longer owns scene behavior details.

### Phase 5: Input + Time Consolidation
1. Move input finalize and time systems/helpers into `plugins/input/*` and `plugins/time/*`.
2. Keep only generic input state types in neutral modules if reused.

Done criteria:
- Input and time plugin behavior is fully co-located.

### Phase 6: Remove Legacy Facades
1. Delete temporary re-export layers after call sites are migrated.
2. Remove dead files/modules and update docs references.

Done criteria:
- No compatibility facades left for old domain locations.
- `cargo test` green across workspace.

## Risk Controls
- Move one domain at a time (render -> ui -> scene -> input/time).
- Keep compatibility re-exports for one phase maximum.
- After each phase:
  - run `cargo test`,
  - run focused plugin tests,
  - verify plugin dependency order constraints.

## Validation Checklist per Phase
1. `cargo test` passes.
2. No new cyclic dependencies introduced.
3. No cross-domain import leakage (e.g. UI plugin importing render internals directly).
4. Plugin schedule order unchanged unless explicitly intended.
5. Docs updated with final module paths.

## Completion Criteria
- Engine feature code is organized by plugin domain folders.
- Core engine loop remains slim and plugin-driven.
- Game crate remains decoupled and integrates through plugin composition.
