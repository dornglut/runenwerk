---
title: Editor UI Runtime V2 And Interaction Formation Design
description: Active design gate for execution-neutral UI interaction contracts formed before retained UI runtime execution.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-15
related:
  - ./ui-definition-formation-foundation-design.md
  - ./editor-self-authoring-and-final-ui-design.md
  - ./editor-shell-menu-and-tab-chrome-polish-design.md
  - ./editor-ui-popup-adornment-drop-preview-contract.md
  - ../deferred/ui-model-multiple-execution-strategies-design.md
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
  - ../../domain/ui/roadmap.md
  - ../../apps/runenwerk-editor/roadmap.md
---

# Editor UI Runtime V2 And Interaction Formation Design

## Status

Active design for the accepted Interaction V2 architecture in
`docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`.
This document does not promote compiled-reactive UI, ECS-driven UI, or an
external UI framework.

The first implementation target remains the existing retained UI stack. The new long-term decision is to form interaction contracts before retained UI execution so repeated menu, popup, scroll, chrome, docking, and viewport-input regressions are solved at the shared contract layer.

## Problem

Recent editor shell bugs repeat the same pattern:

- popup menus are built as local retained nodes without a shared popup stack contract;
- scroll containers return unhandled input at boundaries, allowing viewport zoom to receive menu wheel events;
- submenu anchoring depends on parent popup nodes that disappear when a single active menu enum switches kind;
- menu width and item stretch are controlled by ad hoc button and popup defaults;
- tab close and active indicators are overlays instead of structural chrome slots;
- docking split previews compete with tab-strip reorder hit targets;
- viewport status/statistics controls lack an explicit overflow policy.

These are not independent polish defects. They are missing interaction contracts in the accepted retained UI path.

## Core Decision

Add an execution-neutral interaction formation step above retained UI:

```text
AuthoredUiTemplate
  -> validate
  -> normalize
  -> FormedInteractionModel
  -> FormedRetainedUiProduct
  -> ui_runtime enforcement
  -> render/product-surface output
```

`FormedInteractionModel` is not a second runtime and not a compiled reactive program. It is the validated interaction contract that retained UI consumes first and that future execution targets may consume later.
Renderer and product-surface output remain derived products, not UI authority.

## Migration Spine

Interaction V2 is a contract spine, not a broad shell polish implementation.
Each migrated slice follows the same order:

```text
authored/editor definition vocabulary
  -> validation rule
  -> normalized interaction contract
  -> retained UI formation adapter
  -> ui_runtime enforcement
  -> editor/app guard
```

This order is the boundary between WR-025 and later retained-UI work such as
WR-024. WR-025 defines the vocabulary, validation, ownership rule, migration
adapter boundary, and required guard. WR-024 may then implement the retained UI
slice only after the relevant contract has an explicit owner and expected
fitness function.

Compatibility-only work is allowed when it records current behavior without
claiming long-term ownership. Compatibility work must name the old path, the new
contract it will migrate toward, and the guard that prevents the old path from
expanding.

## Ownership

`domain/ui/ui_definition` owns generic authored interaction vocabulary, validation, normalization, source maps, and formation outputs that do not mention editor commands, runtime `WidgetId`, ECS entities, or concrete shell sessions.

`domain/ui/ui_runtime` owns runtime input routing, focus, scroll ownership, layout measurement, clipping, hit testing, and frame output for formed retained UI products.

`domain/editor/editor_definition` owns editor-specific menu, workspace, chrome, command binding, panel registry, and tool-surface definition descriptors that reference generic UI contracts without moving editor semantics into `domain/ui`.

`domain/editor/editor_shell` owns shell composition, workspace host semantics, command routing, provider presentation, and Strangler adapters from current shell state into formed interaction contracts.

`apps/runenwerk_editor` owns app/runtime integration, viewport input arbitration, file/project IO, fixture loading policy, and concrete command execution.

## Interaction Contracts

The first V2 contract set must cover:

- `FormedInteractionModel`: one formed interaction artifact associated with a normalized template or editor shell surface;
- popup stack and menu scopes: parent/child menus, submenu anchors, focus return, outside dismiss, escape behavior, and layer policy;
- scroll ownership policy: nearest owner selection, axis ownership, boundary consumption, scrollbar drag ownership, and viewport fallback rejection;
- focus scope and dismissal policy: menus, popups, text inputs, modal-like surfaces, and ordinary panels;
- menu sizing policy: max intrinsic item measurement, popup clamp, scroll fallback, and item fill width;
- dock/drop-zone policy: tab reorder zones, split zones, precedence, invalid targets, and preview-only state;
- chrome slot policy: fixed close slot, active indicator slot, label slot, and drag region for tabs and workspace buttons;
- status/metadata overflow policy: scrollable or wrapped status bars with stable priority for essential metrics.

## Guardrail Contract

Every Interaction V2 slice must answer these questions before retained UI code
changes:

- owning definition vocabulary: generic `domain/ui/ui_definition` or
  editor-specific `domain/editor/editor_definition`;
- formed contract output: the record in `FormedInteractionModel` that retained
  UI consumes;
- runtime enforcement point: the `domain/ui/ui_runtime` layout/input/focus/hit
  testing behavior that must become deterministic;
- shell adapter: the `domain/editor/editor_shell` composition or workspace
  projection path that maps existing shell state into the formed contract;
- app boundary: the `apps/runenwerk_editor` viewport/input/command behavior that
  must not become UI authority;
- regression guard: definition validation, runtime tests, shell tests, app
  guards, and screenshot or primitive-order checks where frame order matters.

If a slice cannot name those six parts, it is not ready for implementation
outside explicit compatibility evidence.

## Retained UI Slice Catalog

WR-025 defines the retained UI migration slices as contract work. The slice
names are intentionally stable so WR-024 and later implementation work can cite
the exact contract it consumes instead of redefining policy in shell or app
code.

| Slice | Old retained path | Formed contract | Retained adapter and runtime enforcement | Guard before WR-024 consumption |
|---|---|---|---|---|
| `IV2-menu-stack` | Toolbar, viewport, tab action, and Switch Type menus open from local shell state and concrete runtime anchors. | Popup stack, menu scope, parent/child anchor, outside-dismiss, escape, and focus-return records in `FormedInteractionModel`. | Editor shell maps menu descriptors into formed menu scopes; retained formation creates anchored menu products; `ui_runtime` owns layer order, hit testing, dismiss routing, and focus return. | Definition validation rejects submenu anchors without a stable parent scope; runtime tests cover popup layer order and outside-dismiss; shell tests cover Switch Type submenu anchoring and focus return. |
| `IV2-scroll-ownership` | Scroll widgets report boundary input as unhandled, letting viewport zoom or sibling surfaces receive wheel input. | Scroll owner, axis policy, boundary-consumption, scrollbar-drag ownership, and viewport-fallback rejection records. | Retained adapter marks scrollable menu/status/panel regions with ownership policy; `ui_runtime` reports input ownership separately from content mutation. | Runtime tests cover nearest scroll owner, clipping, axis ownership, boundary consumption, and scrollbar capture; app guards prove viewport wheel input is delivered only after UI declines ownership. |
| `IV2-menu-sizing` | Menu width, item fill, clamp, and scroll fallback are inferred from popup/button defaults. | Menu intrinsic measurement, max item width, popup clamp, item fill, and overflow fallback policy. | Editor definition/shell descriptors declare sizing intent; retained formation emits a menu product with explicit measurement policy; `ui_runtime` enforces item fill and scroll fallback after clamp. | Definition validation rejects menu lists without sizing/stretch policy; runtime layout tests prove clamped popups stretch items to measured width and scroll when required. |
| `IV2-chrome-slots` | Tab and workspace close/active indicators are overlay popups competing with labels and drag regions. | Structural chrome slots for close affordance, active indicator, label, command area, and drag region. | Editor shell maps tab/workspace chrome descriptors into formed chrome slots; retained formation emits structural nodes instead of overlay adornments; `ui_runtime` preserves slot hit precedence. | Shell tests prove close, active indicator, label, and drag slots do not overlap; primitive-order tests cover visual precedence where overlay behavior used to hide conflicts. |
| `IV2-dock-drop-zones` | Dock previews and tab reorder zones compete through local hit-test order and reserved preview spacing. | Dock/drop-zone priority, invalid target, tab reorder, split insertion, floating host, and preview-only state records. | Editor shell projects drag state into formed drop-zone candidates; retained formation renders preview-only products; `ui_runtime` applies drop-zone hit precedence without mutating workspace layout. | Shell/controller tests cover split-border precedence, tab reorder precedence, invalid targets, candidate cycling, and preview-only state without layout reservation. |
| `IV2-status-and-viewport-arbitration` | Viewport metrics/status controls are projected as local text or overlays with no explicit overflow/input ownership rule. | Status overflow, essential metric priority, compact wrapping/scrolling, and viewport input arbitration policy. | App/runtime metrics remain app-owned data; editor shell maps them into formed status descriptors; retained formation emits status products; `ui_runtime` owns overflow hit/input behavior. | Shell tests cover FPS/frame-time projection and status overflow; app guards prove viewport input remains fail-closed while a status or popup surface owns pointer or wheel input. |

These slices are not an implementation order for all future UI work. They are
the minimum contract catalog needed before retained UI polish can proceed
without recreating the same policy in several layers.

2026-05-15 implementation status:

- `IV2-menu-stack` has its first code-bearing retained slice. `domain/ui/ui_definition/src/interaction.rs`
  defines menu-scope records, `domain/ui/ui_definition/src/validate.rs::validate_menu`
  rejects invalid scope identity/parenting, and `domain/ui/ui_runtime/src/input/hit_test.rs`
  plus `domain/ui/ui_runtime/src/runtime/ui_runtime.rs::dispatch_keyboard_event`
  enforce popup layer order and Escape dismissal.
- Outside pointer dismissal and focus return are enforced in
  `domain/ui/ui_runtime/src/input/pointer.rs::dispatch_pointer_event`.
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_editor_shell_frame_with_docking_visual_state`
  adapts tab action and Switch Type menus into formed interaction scopes, and
  `domain/editor/editor_shell/src/composition/toolbar_definition.rs::build_defined_toolbar_menu_popup_with_binding`
  adapts toolbar menu popups.
- `IV2-scroll-ownership` has its first code-bearing retained slice.
  `domain/ui/ui_definition/src/form.rs::form_retained_ui` forms scroll-owner
  records for retained scroll nodes, and
  `domain/ui/ui_runtime/src/input/pointer.rs::apply_scroll_wheel_delta`
  reports boundary wheel input as owned even when the offset does not mutate.
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs::production_input_bridge_allows_viewport_scroll_only_after_ui_declines_ownership`
  guards the production bridge boundary without moving viewport input authority
  into `domain/ui`.
- `IV2-menu-sizing` has its first code-bearing retained slice.
  `domain/ui/ui_definition/src/interaction.rs` defines formed menu-sizing
  records, `domain/ui/ui_definition/src/validate.rs::validate_menu` rejects
  item menus without sizing policy, and
  `domain/ui/ui_runtime/src/layout/engine.rs::layout_popup` preserves clamped
  menu measurement while stretching scroll-backed fill-width menu items.
  `domain/editor/editor_shell/src/composition/toolbar_definition.rs::build_defined_toolbar_menu_popup_with_binding`
  and `domain/editor/editor_shell/src/composition/build_editor_shell.rs::tab_stack_popup_interaction_model`
  adapt toolbar and tab-stack menus into formed menu-sizing records.
- `IV2-chrome-slots` has its first code-bearing retained slice.
  `domain/ui/ui_definition/src/interaction.rs` defines formed chrome-slot
  records for close affordance, command area, label, drag region, and active
  indicator roles. `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`
  and `domain/editor/editor_shell/src/composition/toolbar_definition.rs::project_workspace_close_buttons`
  now emit structural tab/workspace chrome slot rows instead of close/indicator
  overlay adornments, and `domain/ui/ui_runtime/src/input/hit_test.rs` covers
  child slot hit precedence.
- `IV2-dock-drop-zones` has its first code-bearing retained slice.
  `domain/ui/ui_definition/src/interaction.rs` defines formed dock/drop-zone
  records for tab reorder, split insertion, and floating-host targets with
  scope, side, active/candidate/invalid state, priority, and preview-only
  policy.
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs::dock_drop_zone_interaction_model`
  adapts current tab-drag preview state into formed drop zones and maps
  editor-owned `DockDropCandidateState::Invalid` values to the generic
  `UiDockDropZoneStateDefinition::Invalid` contract while keeping workspace
  mutation in the editor/app command path. `apps/runenwerk_editor/src/shell/controller.rs`
  forms source-only same-area and same-host split candidates as invalid and
  excludes them from commit target resolution, while
  `apps/runenwerk_editor/src/shell/state.rs::cycle_active_tab_drag_preview_candidate`
  skips invalid candidates during cycling. `domain/ui/ui_runtime/src/input/hit_test.rs`
  covers preview overlay child hit precedence.
- `IV2-status-and-viewport-arbitration` has its first code-bearing retained
  slice. `domain/ui/ui_definition/src/interaction.rs` defines formed viewport
  status-region, overflow, metric-priority, and input-arbitration records.
  `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::viewport_status_overlay`
  emits a horizontal scroll-owned status region for details, FPS/frame-time,
  and overlay status lines. Persistent viewport chrome/status overlays use
  `PopupDismissPolicy::None` so they can share anchored-popup layout without
  joining the dismissible menu stack.
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs::viewport_surface_interaction_model`
  maps viewport options/tools popups plus status regions into formed menu,
  scroll, status, and viewport-fallback contracts, while
  `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs::viewport_pointer_route_rejects_viewport_status_dispatch_target`
  and `apps/runenwerk_editor/tests/viewport_architecture_guards.rs::production_input_bridge_allows_viewport_scroll_only_after_ui_declines_ownership`
  prove status/chrome dispatch targets cannot route into scene fallback and
  viewport wheel input is delivered only after UI declines ownership. The
  source-string guard in
  `apps/runenwerk_editor/tests/viewport_architecture_guards.rs::viewport_status_arbitration_is_formed_before_scene_fallback`
  remains a broad regression guard, not the primary proof.

All named WR-025 Interaction V2 retained slices now have code-bearing contract
spines plus doctrine-repair behavior evidence recorded in
`docs-site/src/content/docs/reports/closeouts/wr-025-interaction-v2-doctrine-repair/closeout.md`.
WR-024 may consume the landed menu-stack, scroll-ownership, menu-sizing,
chrome-slot, dock/drop-zone, and status/viewport arbitration behaviors, but any
larger execution target change still needs a separate accepted design or ADR.

## Strangler Migration

Phase 1 - Design Gate

- Accept ADR 0009, update roadmap source/indexes, and keep subordinate links from the existing polish design.
- Do not change runtime behavior in this phase.
- Exit when roadmap render/check and docs validation pass and the retained UI
  slice catalog is linked from the UI and editor roadmaps.

Phase 2 - Menus And Scroll

- Introduce popup stack/menu scope contracts.
- Update retained runtime wheel dispatch to report ownership separately from mutation.
- Introduce menu sizing policy for item fill, popup clamp, and scroll fallback.
- Migrate toolbar menus, viewport tools/options/details, tab action menus, and Switch Type submenu.
- Exit only after tests prove outside-dismiss, focus return, menu layer order,
  nearest scroll owner selection, menu item fill after popup clamp, and viewport
  wheel rejection when UI owns the event.

Phase 3 - Chrome

- Replace overlay close/indicator behavior with structural tab and workspace chrome slots.
- Remove Unicode glyph dependence for active indicators.
- Keep authored/runtime identity boundaries intact.
- Exit only after shell tests prove close slot, active indicator slot, label
  slot, and drag region do not overlap or reorder unpredictably.

Phase 4 - Docking

- Make tab-strip reorder zones suppress split previews.
- Represent split previews through semantic drop zones and preview-only state.
- Exit only after split-border precedence, tab reorder precedence, invalid
  targets, candidate cycling, commit-target exclusion, and preview-only state
  are covered.

Phase 5 - Metrics And Status

- Project FPS/frame time from an always-on runtime/editor metric source.
- Apply explicit overflow policy to viewport statistics/details/status bars.
- Exit only after status overflow cannot steal viewport input, persistent
  status/chrome overlays are not treated as dismissible menu-stack popups, and
  app guards prove viewport arbitration remains fail-closed.

## Deferred Execution Targets

Compiled-reactive UI and ECS-driven UI remain deferred. If a future accepted design or ADR promotes either path, it must consume the same normalized UI definitions and formed interaction contracts as an additional target. No future target may replace authored UI identity, source maps, command ratification, or the renderer-as-derived-product rule:

```text
NormalizedUiTemplate
  -> FormedInteractionModel
  -> FormedRetainedUiProduct
  -> future CompiledUiProgram
  -> future EcsUiSpawnPlan
```

No implementation in this design may make compiled-reactive or ECS-driven UI the default editor path.

## Fitness Functions

The design is enforceable only with guard tests:

- UI definition/editor definition validation rejects submenu anchors without stable parent scope, scrollable popups without scroll ownership policy, menu lists without sizing/stretch policy, and tab strips without reorder-zone priority.
- UI runtime tests prove scroll boundary wheel input is owned, nearest scroll owner selection is deterministic, popup stack anchoring survives parent menu presence, and menu layout stretches all items to measured width.
- Editor shell tests prove Switch Type submenu anchoring, viewport stats overflow, left chrome slot order, and tab-strip reorder precedence over split previews.
- App guard tests prove viewport zoom only receives wheel after UI explicitly declines ownership and deferred UI execution strategies remain absent from production paths.

## Non-Goals

- No external UI framework adoption.
- No compiled-reactive runtime implementation.
- No ECS UI runtime implementation.
- No movement of editor command semantics into `domain/ui`.
- No rewrite of self-authoring source identity, route slots, or provider ratification boundaries.
