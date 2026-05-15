---
title: UI Substrate Architecture
description: Current-state architecture, ownership boundaries, and migration direction for Runenwerk UI.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-15
---

# UI Substrate Architecture

## Purpose
Establish the factual, current-state architecture for Runenwerk UI, define correct ownership boundaries, and document remaining migration direction from implemented substrate extraction to full editor/runtime convergence.

This page is intentionally current-state-first.

## Scope
This document covers:

- `domain/ui/*` crates
- retained UI runtime crates under `domain/ui/*`
- workspace/tool-surface host ownership in `editor_shell`
- runtime/app glue in `apps/runenwerk_editor`
- engine render/UI integration paths used to submit and draw UI frames
- general UI definition/formation contracts where they clarify `domain/ui` ownership

This document does not define visual design direction, docking product UX, or authored editor-definition workflows.

## Current Reality
As of the audited repository state:

- `domain/ui/*` now owns both primitive crates and retained runtime crates (`ui_tree`, `ui_runtime`, `ui_widgets`).
- Workspace identity/projection/reducer/tool-surface host infrastructure is implemented and belongs to `editor_shell`.
- `apps/runenwerk_editor` owns app runtime bridging and viewport runtime resources/bindings.
- Engine render integration for UI frame submission/extraction is implemented.
- Viewport slot semantic ownership is in `editor_viewport`; renderer-facing embed payload slots are opaque IDs in `ui_render_data`, mapped through integration adapters.
- Core shell control flows (outliner entity selection, viewport product selection, inspector field activation) now route through prepared `SurfacePresentationModel` + typed `SurfaceIntent` + host-side ratification adapters.
- Engine overlay/debug UI paths now route through substrate frame generation (`ui_runtime::build_ui_frame`) instead of ad hoc primitive assembly.
- `domain/ui/ui_definition` now provides the first authored UI definition and retained formation layer. Toolbar/menu structure, normal shell chrome, and console surface structure form through checked-in definitions. Authored scroll nodes support explicit horizontal, vertical, or two-axis behavior with per-axis input policy, while broader workspace/profile defaults and richer unavailable editor feature representation still need follow-up hardening.
- Prior fallback seams removed:
  - no `first_frame()`-based routing in editor runtime systems
  - no `ViewportId(0)` fallback in shell viewport adapter
- The normal viewport migration path is closed. The runtime still has one
  compatibility-only bootstrap selection seam in
  `apps/runenwerk_editor/src/runtime/viewport/routing.rs::resolve_structural_viewport_products`
  before first structural projection artifacts exist; private helpers
  `select_viewport_id_with_bootstrap_policy` and `bootstrap_single_viewport_id`
  require exactly one observed viewport and are not a path for new
  viewport/product work.
- Substrate output now has baseline snapshot tests and a lightweight gallery harness example (`domain/ui/ui_runtime/examples/substrate_gallery.rs`).

## Current Crate Map

- `domain/ui/ui_math`
  - UI math/geometry primitives (`UiRect`, `UiSize`, `UiPoint`, `UiVector`, `UiInsets`, axis types).
- `domain/ui/ui_input`
  - input and focus contracts (events, pointer/keyboard/text contract types, stylus-capable pointer packets, focus ids, shortcut contracts).
- `domain/ui/ui_layout`
  - stateless layout algorithms/contracts (`StackLayout`, `SplitLayout`, constraints, alignment, size policy).
- `domain/ui/ui_text`
  - text primitives and atlas-based layout contracts (`TextStyle`, buffer/cursor/selection, `TextLayouter`, `AtlasTextLayouter`).
- `domain/ui/ui_theme`
  - theme token scales and defaults (colors, spacing, radius, typography). The
    default editor substrate theme is a compact black/dark-gray theme with zero
    radius tokens; app/editor-owned authored themes can override those tokens
    through the editor definition activation path.
- `domain/ui/ui_render_data`
  - renderer-facing `UiFrame`/surface/layer/primitive contracts used by engine renderer extraction.
- `domain/ui/ui_tree`
  - retained tree contracts (`WidgetId`, node kinds/payloads, anchored popup nodes with explicit layer order, tree traversal, computed layout records).
- `domain/ui/ui_runtime`
  - retained runtime orchestration (layout engine, anchored popup layout/hit-testing, input routing, popup overlay layer ordering, runtime state, frame output generation). Button hit testing uses the full button bounds, including padding, while text and other content layout still uses content bounds. Scrollbars are overlay primitives: scroll layout no longer reserves a permanent gutter, two-axis scroll containers keep horizontal and vertical offsets independent, and scrollbar primitives are emitted only for active scroll interaction state. In two-axis containers, scrollbar visibility is tracked per axis: vertical wheel activity reveals only the vertical bar, horizontal middle-drag activity reveals only the horizontal bar, and viewport hover does not reveal every scrollbar.
- `domain/ui/ui_widgets`
  - ergonomic widget/control constructors over `ui_tree` node contracts.

## Adjacent Definition Layer

`domain/ui/ui_definition` is the UI domain crate for general authored UI definitions and their formation pipeline.

This layer owns general authored UI definitions and their formation pipeline:

- authored UI node, layout, menu, popover, theme-reference, and action-slot definitions;
- stable authored UI ids that are distinct from runtime `WidgetId`, focus, capture, and shell session ids;
- validation and normalization for generic UI structure, references, and availability descriptors;
- execution-neutral normalized UI templates that do not encode retained `UiNodeKind`, runtime `WidgetId`, ECS entity ids, or concrete command execution;
- first retained-tree formation target for templates, slots, repeaters, embeds, menus, and availability products consumed by `ui_tree`, `ui_widgets`, and `ui_runtime`.

The authored and normalized UI definition model is source/IR. It should remain stable if a future accepted design adds compiled-reactive or ECS-driven UI execution. Those strategies would be additional formation targets from the normalized model, not a reason to rewrite authored templates.

It must not own editor workspace profiles, `ToolSurfaceKind`, panel/tab identity, app provider registries, concrete command execution, or editor-specific command semantics. Those belong in editor definition/shell/app layers.

Applying an editor-owned definition into the live editor is outside
`domain/ui/ui_definition`. The generic crate can normalize and form retained UI
products, but the app/editor activation seam decides whether an applied editor
definition actually changes live shell/runtime resources.

## Interaction Formation Ownership

ADR 0009 adds an accepted interaction formation layer between normalized UI
definitions and retained UI products. That layer is execution-neutral contract
data, not a second runtime:

```text
NormalizedUiTemplate
  -> FormedInteractionModel
  -> FormedRetainedUiProduct
  -> ui_runtime enforcement
```

Ownership is split intentionally:

- `domain/ui/ui_definition` owns generic interaction vocabulary, validation,
  normalization, source maps, and formed interaction outputs;
- `domain/ui/ui_runtime` owns retained enforcement for layout, clipping, hit
  testing, focus, scroll ownership, input ownership, and frame output;
- `domain/editor/editor_definition` owns editor-specific descriptors that refer
  to generic contracts without making editor commands generic UI semantics;
- `domain/editor/editor_shell` owns shell composition and compatibility
  adapters from current shell state into formed contracts;
- `apps/runenwerk_editor` owns viewport arbitration, runtime integration, app
  IO, fixture loading, and concrete command execution.

The migration guardrail for every Interaction V2 slice is:

```text
definition vocabulary
  -> validation rule
  -> FormedInteractionModel record
  -> retained UI formation adapter
  -> ui_runtime enforcement
  -> editor/app guard
```

Shell polish and popup/adornment/drop-preview documents can supply migration
evidence, but they do not move long-term popup, scroll, focus, docking, chrome,
status, or viewport-input ownership out of this UI/editor/app split.

The current retained UI migration slice catalog is:

| Slice | `domain/ui` responsibility | Adjacent owner responsibility |
|---|---|---|
| `IV2-menu-stack` | generic popup/menu scope records, validation, retained formation, layer order, hit testing, outside-dismiss, and focus-return enforcement | `domain/editor/editor_definition` and `editor_shell` provide editor menu descriptors and shell adapters; `apps/runenwerk_editor` executes routed commands |
| `IV2-scroll-ownership` | scroll owner and axis policy, boundary-consumption, scrollbar capture, and input ownership reporting | editor/app layers keep viewport zoom and provider behavior outside UI authority until UI declines ownership |
| `IV2-menu-sizing` | menu sizing/stretch policy, retained menu measurement, clamp, and scroll fallback | editor descriptors provide menu intent without one-off shell/runtime defaults |
| `IV2-chrome-slots` | generic chrome slot vocabulary and retained slot formation/enforcement | editor shell owns tab/workspace chrome semantics, labels, commands, and route mapping |
| `IV2-dock-drop-zones` | generic drop-zone/hit priority vocabulary where reusable; retained runtime enforces formed preview hit policy | editor shell owns workspace docking intent, preview-only state, and workspace mutation on commit |
| `IV2-status-and-viewport-arbitration` | status overflow policy and UI input ownership reporting | app/editor layers own metrics data, viewport resources, and fail-closed viewport input arbitration |

Landed slice status as of 2026-05-15:

- `IV2-menu-stack` now has generic definition/runtime support through
  `domain/ui/ui_definition/src/interaction.rs`,
  `domain/ui/ui_definition/src/validate.rs::validate_menu`,
  `domain/ui/ui_runtime/src/input/hit_test.rs`, and
  `domain/ui/ui_runtime/src/runtime/ui_runtime.rs::dispatch_keyboard_event`.
  Toolbar and tab-stack menu adapters live in
  `domain/editor/editor_shell/src/composition/toolbar_definition.rs::build_defined_toolbar_menu_popup_with_binding`
  and
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_editor_shell_frame_with_docking_visual_state`.
- `IV2-scroll-ownership` now forms retained scroll owners in
  `domain/ui/ui_definition/src/form.rs::form_retained_ui` and reports wheel
  boundary ownership in
  `domain/ui/ui_runtime/src/input/pointer.rs::apply_scroll_wheel_delta`.
- App production viewport input remains app-owned. The guard is
  `apps/runenwerk_editor/tests/viewport_architecture_guards.rs::production_input_bridge_allows_viewport_scroll_only_after_ui_declines_ownership`.

Related non-`domain/ui` owners currently in the runtime path:

- `domain/editor/editor_shell`
  - shell composition, workspace host model, command routing, compatibility re-exports for substrate types.
- `apps/runenwerk_editor`
  - app runtime resource wiring, viewport presentation/product runtime resources, tool-surface runtime binding registry.
- `engine/src/plugins/render`
  - UI submission registry, prepared UI payloads, renderer extraction and draw path for UI primitives.

## What `domain/ui/*` Currently Owns

- reusable primitive contracts for UI math, input, layout, text, theme, and render-data payloads
- stylus-capable pointer vocabulary for pressure, tilt, twist, tangential
  pressure, eraser/tool kind, device id, timestamped raw/coalesced/predicted
  samples, hover/contact state, barrel buttons, calibration, and low-latency
  preview classification
- text atlas and layouter contracts plus concrete atlas layouter implementation
- renderer-facing UI frame/primitive data model consumed by engine render feature code
- retained runtime ownership via:
  - `ui_tree` (retained nodes/tree/layout records)
  - `ui_runtime` (tree orchestration, anchored popup layout/hit-testing, popup overlay layer ordering, input routing, frame generation)
  - `ui_widgets` (control/widget constructors)

## What `domain/ui/*` Does Not Yet Own

- fully converged app-side usage of all reusable controls in editor shell surfaces
- shell workspace host semantics (correctly owned by `editor_shell`)
- app/runtime glue and viewport product orchestration (correctly owned by `runenwerk_editor`)

## Relationship Between `domain/ui`, `editor_shell`, `runenwerk_editor`, and Engine Render Integration

### `domain/ui`
Provides reusable engine-agnostic UI primitives/contracts and renderer-facing frame data contracts.

### `editor_shell`
Correct owner for:

- workspace structural identity and graph state
- host/panel/tab/tool-surface composition logic
- shell command routing from UI interactions

### `runenwerk_editor`
Owns app/runtime glue:

- shell controller and dispatch bridge
- viewport runtime resource orchestration
- tool-surface runtime binding resource and viewport runtime systems

### Engine render integration
Owns:

- UI frame submission registry/resources
- prepared UI contribution payloads
- renderer extraction/render path for `UiFrame` primitives

This layer should continue consuming UI frame contracts, not owning UI semantics.

## Current Boundary Violations

1. Viewport semantic-to-render payload slot mapping is intentionally adapter-based and must remain centralized; avoid introducing parallel semantic taxonomies in runtime or renderer layers.
2. Existing shell surfaces still rely primarily on button/label primitives; newer reusable controls (`TextInput`, `Toggle`, `NumericInput`, `Tabs`) are present but not yet broadly adopted.
3. The compatibility-only bootstrap single-viewport seam exists in
   `apps/runenwerk_editor/src/runtime/viewport/routing.rs::resolve_structural_viewport_products`
   before first structural tool-surface binding generation. New viewport and
   product work must use structural viewport binding.

## Target Ownership Model
Target ownership (partially implemented):

- `domain/ui/*` owns reusable UI substrate runtime layers:
  - retained tree/runtime orchestration
  - reusable control runtime
  - input/focus/invalidation behavior
  - shared testing harness
- `domain/ui/ui_definition` owns general authored UI definition and formation contracts, while `domain/ui` runtime crates consume formed products.
- `editor_shell` owns workspace host semantics and shell-specific composition/command routing only.
- `runenwerk_editor` owns app/runtime wiring and viewport/editor-specific runtime integrations.
- engine render layer continues to own rendering integration and consumes UI frame contracts as data.

## Migration Direction
The migration direction should remain dependency-aware and incremental:

1. complete broader adoption of reusable controls across editor surfaces where ad hoc assembly remains.
2. keep render-data embed slot IDs opaque and generic while preserving semantic slot taxonomy in `editor_viewport`, with explicit mapping at integration edges.
3. keep substrate docs and tests aligned with implemented behavior per phase completion, including surface-flow tracing from observation to ratification.

## Testing and Verification Expectations

- Keep architecture guard tests that enforce structural identity and fail-closed routing behavior.
- Keep baseline unit coverage in primitive crates (`ui_math`, `ui_input`, `ui_layout`, `ui_theme`, `ui_render_data`, `ui_tree`, `ui_widgets`).
- Keep retained-runtime interaction coverage for keyboard/text/focus/invalidation and control interactions.
- Keep UI frame snapshot/fixture verification for stable render-data expectations (`domain/ui/ui_runtime/src/output/build_ui_frame.rs` tests).
- Keep the lightweight substrate gallery harness runnable (`domain/ui/ui_runtime/examples/substrate_gallery.rs`).
- Preserve smoke/architecture tests proving no fallback regression for viewport/tool-surface binding behavior.

## Explicit Non-Goals

- documenting fallback seam removal as complete before code actually removes it
- full docking/tab UX productization in this architecture document
- authored editor-definition/meta-editor system specification here
- turning `domain/ui` or `domain/ui/ui_definition` into editor semantics crates
- speculative future feature taxonomy beyond current audited constraints

## Related Architecture and Workspace Docs

- [Workspace Architecture Boundaries](../../guidelines/architecture.md)
- [Runenwerk Architecture Doctrine](../../guidelines/runenwerk-architecture.md)
- [Module Structure Guidelines](../../guidelines/module-structure-guidelines.md)
- [UI Definition Formation Framework Design](../../design/active/ui-definition-formation-foundation-design.md)
- [ADR 0009: UI Interaction Formation V2](../../adr/accepted/0009-ui-interaction-formation-v2.md)
- [Editor / UI / Workspace / Tool-Surface Architecture](../../design/active/editor-ui-workspace-tool-surface-architecture.md)
- [Viewport Expression Upgrade Design](../../design/active/workspace-viewport-expression-upgrade-design.md)
- [Workspace Identity Contract and Migration Map](../../design/active/workspace-identity-contract-and-migration-map.md)
- [UI Substrate Roadmap](./roadmap.md)
