# Runenwerk - Workspace Identity Contract and Migration Map

_Last updated: 2026-04-21_

## Purpose

Define the workspace identity architecture before docking, tab-stack, and broader workspace features are implemented, and map the exact migration targets in the current shell/runtime code.

This document is the identity planning contract and migration/status map. It is not a docking implementation spec.

---

## Scope

## In scope

- explicit identity contracts for:
  - `WorkspaceId`
  - `PanelHostId`
  - `PanelInstanceId`
  - `ToolSurfaceInstanceId`
  - `TabStackId`
- ownership boundaries between:
  - workspace composition
  - panel/container hosting
  - tool-surface semantics
  - runtime bindings (for example viewport runtime ids)
- allocator and lifecycle policy
- migration mapping to concrete files/functions
- architecture guard test plan

## Out of scope

- implementing docking
- implementing tab drag/drop
- implementing full workspace persistence productization
- changing already completed Phase 1 viewport runtime ownership architecture

---

## Identity Type Contract (Exact Definitions)

Implemented shape for workspace-visible ids (`domain/editor/editor_shell/src/workspace/identity.rs`):

```rust
use id_macros::id;

#[id]
pub struct WorkspaceId;

#[id]
pub struct PanelHostId;

#[id]
pub struct PanelInstanceId;

#[id]
pub struct ToolSurfaceInstanceId;

#[id]
pub struct TabStackId;
```

Type semantics:

- `WorkspaceId`
  - identifies one workspace composition root instance
  - structural identity
  - persistent for saved workspace definitions/state
  - must not be confused with `ViewportId`
- `PanelHostId`
  - identifies a composition host/container node
  - structural identity
  - persistent in layout state
  - must not be confused with `TabStackId` or widget ids
- `PanelInstanceId`
  - identifies one mounted panel container instance
  - structural runtime identity
  - session-scoped by default
  - must not be confused with tool/runtime semantic ids
- `ToolSurfaceInstanceId`
  - identifies one mounted semantic tool-surface instance
  - behavioral/content runtime identity
  - session-scoped by default
  - may bind to runtime identities (for example `ViewportId`) but is not one
- `TabStackId`
  - identifies a tab container
  - structural identity
  - persistent in layout state
  - must not be confused with selected tab index

---

## Allocation Policy

Introduce one explicit allocator for workspace identity.

Target module ownership:

- `domain/editor/editor_shell/src/workspace/identity.rs` (new)
  - `WorkspaceIdentityAllocator`
  - monotonic counters for each id family
  - deterministic seed constructor for tests

Allocator rules:

1. Ids are allocated only by the allocator, never by ad hoc literals in composition code.
2. `WorkspaceId` allocation boundary is workspace creation/load.
3. `PanelHostId` and `TabStackId` are allocated when structural containers are created.
4. `PanelInstanceId` is allocated when a panel is mounted.
5. `ToolSurfaceInstanceId` is allocated when tool content is mounted.
6. Rebuild/reflow does not allocate new ids.
7. Close/recreate allocates new `PanelInstanceId` and `ToolSurfaceInstanceId` unless explicit restore policy says otherwise.
8. Test paths use deterministic seed allocators to make id behavior reproducible.

---

## Core Invariants

1. Structural identity survives UI tree rebuild.
2. No panel identity is inferred from layout tree position or child index.
3. No tool-surface identity is inferred from `ViewportId`.
4. No tab-stack identity is inferred from selected tab state.
5. Host/container identity is distinct from panel/content identity.
6. Runtime viewport identity stays distinct from workspace/panel identities.
7. Widget ids are projection/runtime-routing ids only, not canonical workspace identity.
8. Moving panel between hosts/tab stacks preserves panel and tool-surface instance identity.
9. Persistence must encode structural identities directly, never inferred ordering.
10. No implicit "first frame/first viewport" fallback is allowed for workspace composition identity.

---

## Ownership Graph

- `WorkspaceState(WorkspaceId)` owns:
  - host graph (`PanelHostId`)
  - tab stacks (`TabStackId`)
  - panel instances (`PanelInstanceId`)
  - tool-surface instances (`ToolSurfaceInstanceId`)
- `PanelHostId` owns split/dock structure and leaf hosting policy.
- `TabStackId` owns ordered panel membership and active panel pointer.
- `PanelInstanceId` may have zero or one active `ToolSurfaceInstanceId` attachment.
- `ToolSurfaceInstanceId` may bind to runtime entities (for example `ViewportId`) through explicit binding records.

---

## Migration Map (File + Function Targets)

Progress snapshot (2026-04-21):

- completed: Step 1 (typed ids + allocator), Step 2 (canonical graph), Step 3 (projection artifacts), Step 4 (structural dispatch + stale-epoch fail-closed), Step 5 (runtime binding contracts), Step 6 (guard-test expansion before docking/tab behavior)
- next: docking/tab behavior implementation on top of locked contracts

## Shell composition and ids

- `domain/editor/editor_shell/src/composition/build_editor_shell.rs`
  - function: `build_editor_shell`
  - migrate to accept workspace composition projection model, not singleton panel fields.
  - must not allocate identity.
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs`
  - function: `build_viewport_panel`
  - input must include `PanelInstanceId` and `ToolSurfaceInstanceId` projection context.
  - runtime `ViewportId` remains tool binding data only.
- `domain/editor/editor_shell/src/ids/widget_ids.rs`
  - functions/constants:
    - `outliner_row_widget_id`
    - `inspector_field_widget_id`
    - `viewport_product_button_widget_id`
  - remove index-derived identity assumptions; use explicit projection maps for dynamic rows.
- `domain/editor/editor_shell/src/commands/map_interactions.rs`
  - function: `map_interactions_to_shell_commands`
  - route through structural ids (`PanelInstanceId`, `ToolSurfaceInstanceId`) rather than positional widget decoding.

## Shell app bridge

- `apps/runenwerk_editor/src/shell/controller.rs`
  - functions:
    - `rebuild_tree_with_viewport_products`
    - `dispatch_input_with_viewport_products`
  - replace singleton shell view model rebuild path with workspace-structured projection + interaction routing map.
- `apps/runenwerk_editor/src/shell/state.rs`
  - struct: `RunenwerkEditorShellState`
  - add cached projection map between widget ids and structural ids.
- `apps/runenwerk_editor/src/shell/viewport_adapter.rs`
  - function: `build_viewport_observation_frame`
  - remove implicit fallback `ViewportId(0)` semantics; require explicit bound tool-surface/viewport mapping.

## Runtime bridging and routing

- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`
  - functions:
    - `submit_editor_frame_system`
    - `sync_viewport_presentation_products_system`
  - remove single active viewport inference (`first_frame` path) for workspace composition routing.
  - consume explicit mounted tool-surface bindings.
- `apps/runenwerk_editor/src/runtime/viewport/layout_map.rs`
  - struct: `ViewportLayoutEntry`
  - retain `WidgetId` for runtime hit routing, but add structural references so routing is not widget-only.
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs`
  - functions:
    - `viewport_pointer_route`
    - `viewport_capture_active`
    - `dispatch_viewport_pointer_down`
  - route pointer dispatch via structural/tool identities first, then runtime viewport binding.
- `apps/runenwerk_editor/src/runtime/systems/bootstrap.rs`
  - function: `seed_viewport_runtime_contracts_system`
  - keep `MAIN_VIEWPORT_ID` bootstrap-only; no runtime workspace fallback semantics.

## Editor app/session state

- `apps/runenwerk_editor/src/editor_app/state.rs`
  - struct: `RunenwerkEditorApp`
  - move singleton viewport-facing UI state toward per-tool-surface instance ownership.
- `apps/runenwerk_editor/src/editor_features/viewport/interaction.rs`
  - struct: `ViewportInteractionState`
  - evolve to be keyed by `ToolSurfaceInstanceId` instead of one global state.

---

## Guard Tests Baseline Before Docking/Tab Work

- identity stability across rebuilds (same structural ids after rebuild)
- panel move between hosts preserves `PanelInstanceId`
- tab selection changes do not mutate `TabStackId`
- runtime `ViewportId` change does not mutate panel/tool-surface instance identity
- panel close/recreate lifecycle semantics enforce new id allocation only where intended
- no "first-frame" fallback for active workspace panel routing
- persistence roundtrip preserves structural ids (`WorkspaceId`, `PanelHostId`, `TabStackId`)

---

## Sequenced Implementation Planning Order

1. completed: add identity types + allocator contract module
2. completed: add workspace structural state model (hosts, stacks, panels, surfaces)
3. completed: add shell projection map (`WidgetId` -> structural ids)
4. completed: migrate shell interaction mapping to structural ids
5. completed: migrate runtime input/frame bridging to explicit surface bindings
6. completed: add guard tests and architecture tests
7. next: start docking/tab behavior implementation on top of the above

---

## Relationship to Completed Phase 1

Phase 1 viewport backend cleanup remains complete and unchanged.

This contract is the next architecture layer:

- keeps viewport runtime identity distinct from workspace composition identity
- prevents singleton leakage before docking/tab features
- hardens host/panel/tool ownership boundaries before breadth features

Status note: the workspace identity hardening track (Steps 1-6) is complete; this document now serves as the locked contract baseline for docking/tab behavior implementation.
