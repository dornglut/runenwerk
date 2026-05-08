---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-08
related_designs:
  - ../../design/active/workspace-viewport-expression-upgrade-design.md
related_roadmaps:
  - ./viewport-expression-implementation-roadmap.md
---

# Runenwerk Editor Current Architecture

`apps/runenwerk_editor` is the runnable editor application. It composes editor
domain crates, UI substrate crates, engine runtime systems, persistence, and
viewport expression routing into a concrete tool.

## Entry Points

- `apps/runenwerk_editor/src/main.rs`: binary entry point.
- `apps/runenwerk_editor/src/lib.rs`: public app crate surface.
- `apps/runenwerk_editor/src/runtime/app.rs`: app/runtime construction.
- `apps/runenwerk_editor/src/runtime/plugin.rs`: engine plugin integration.

## App-Owned Areas

- `editor_app`: high-level app state and facade.
- `editor_runtime`: scene state, command ratification, history, selection, and
  tool state.
- `editor_features`: editor feature actions and viewport tools.
- `editor_panels`: concrete panel/widget composition.
- `runtime`: engine-facing systems, resources, viewport routing, expression
  product registration, picking, and frame submission.
- `shell`: app-owned concrete editor surface providers, provider registry
  composition, shell controller wiring, and surface-session state.
- `persistence`: retained change storage, project files, runtime persistence,
  and workspace layout.

## Ownership Boundary

The app owns concrete wiring and host policy. It should not redefine editor
domain semantics, UI surface contracts, world edit contracts, or engine runtime
contracts that already live in owning crates.

## Surface Provider Architecture

The editor shell uses `EditorShellFrameModel`, where mounted surfaces are
resolved by `ToolSurfaceInstanceId`:

```text
Workspace/profile/document context
+ mounted ToolSurfaceInstanceId
+ ToolSurfaceKind / SurfaceDefinitionId
        -> app-owned provider registry
        -> provider-owned artifact + provider-local routes
        -> shell host chrome/docking/tabs
```

Concrete providers live in `apps/runenwerk_editor/src/shell/providers/`.
The M3.6 self-authoring workspace registers app-owned provider surfaces for
definition outliner, UI hierarchy, UI canvas/retained preview, style inspector,
bindings, dock/layout preview, theme editor, shortcut editor, menu editor,
definition validation, and command diff summaries. These surfaces inspect,
preview, and author editor/UI definition documents through retained control
routes that propose shell commands; they do not execute app commands directly
or move provider behavior into `domain/ui/ui_definition`.
Provider contracts that are app/runtime neutral live in
`domain/editor/editor_shell/src/surface_provider.rs`.

The registry is explicitly composed by the editor app/plugin host. It is not a
global mutable registry. Provider resolution is deterministic and fail-closed:
duplicate provider ids are rejected, equal-priority provider ambiguity produces
an ambiguous diagnostic artifact, unsupported surfaces render an unsupported
artifact, and diagnostic artifacts emit no provider-local routes.

Surface-local UI state is stored per `ToolSurfaceInstanceId` in
`apps/runenwerk_editor/src/shell/surface_session.rs`. Console lines, app
diagnostics, runtime/session state, and toolbar state remain app/global; console
view state, entity table filters, inspector draft/focus state, and viewport
interaction/details state are surface-session concerns.

Inspector enum editing is typed end to end. Reflected unit/no-payload ECS enums
are described by `domain/ecs/src/reflect/enum_info.rs::EnumInfo` through
`domain/ecs/src/reflect/type_info.rs::ReflectShape::Enum`; the derive macro
rejects payload variants for now. `domain/editor/editor_inspector` projects
those fields as enum inspector controls and applies
`InspectorEditValue::EnumSymbol` by calling the reflected unit-variant setter.
`apps/runenwerk_editor/src/shell/dispatch/inspector.rs` still owns the
app/runtime dispatch path. Reflected payload enum variants remain deferred until
the reflection/adapter layer has a concrete payload field design.

## Shell Layout

The app toolbar is produced by
`apps/runenwerk_editor/src/shell/toolbar_adapter.rs::build_toolbar_observation_frame`
and rendered by
`domain/editor/editor_shell/src/composition/build_toolbar.rs::build_toolbar`.
It exposes File, Edit, and Window menu controls, followed by the currently open
workspace profile switches plus a workspace `+` menu. The default open
workspace row contains Scene and Modelling only; Editor Design is added to that
row when it is activated from the `+` menu. The `+` menu opens the available
workspace profiles; Scene and Modelling are therefore no longer duplicated under
Window. Menu items are projected by
`domain/editor/editor_shell/src/composition/toolbar_definition.rs::build_defined_toolbar_menu_popup`
as a retained popup anchored to the active top-level menu button, rather than as
an additional toolbar row. Items whose workflows are not implemented are emitted
as disabled popup buttons so the retained UI renders them as unavailable instead
of routing them to app behavior.

The checked-in toolbar definition in `assets/editor/ui/toolbar.ron` authors the
File/Edit/Window group, then a thin fixed-length vertical separator, then the
default workspace controls.
`domain/editor/editor_shell/src/composition/toolbar_definition.rs::build_defined_toolbar`
forms that authored separator through the generic retained UI definition path
and injects dynamic open-workspace buttons, such as Editor Design after plus
activation, before the workspace `+` button. It also trims the toolbar root
panel so it does not add top padding above the first toolbar row. The separator
is not text; it is a retained `Divider` formed from
`UiNodeDefinition::Separator`, with spacing around it supplied by the toolbar row
gap in `build_defined_toolbar`.

Tab-stack chrome is projected directly from the workspace graph by
`domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip`.
The tab strip renders tabs first and keeps the `+` tab button as the final tab
row control. The tab-stack container is a transparent clipping panel, so tab
headings, the `+` button, and tab adornments cannot paint into adjacent dock
headings when a split is resized narrow. Tab close controls are emitted by the
tab-strip projection as small overlay buttons anchored inside the right side of
each tab button, rather than as separate row items or a post-formation promotion
pass.
Those close buttons use
`ui_tree::ButtonNode::reveal_on_hover_anchor`, so they render only when the tab
or close button is hovered; they keep a small right offset and a 50% alpha
rounded background even when the global shell theme has square controls.
Clicking a tab activates that panel; it does not switch the panel's
editor/surface type. Surface type changes are available through the tab
secondary-click action popup: the popup contains a Switch Type item that opens
the surface-type submenu beside that item. Surface type options are projected through
`domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_stack_surface_menu_popup`
and routed as explicit `ShellCommand::SwitchPanelToolSurfaceKind` commands.
Split/duplicate/reset/lock/close area commands are no longer rendered as inline
`H V D R Lock x` controls and no `...` button is rendered in the tab strip.
They are opened by secondary-clicking a tab, then projected by
`domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_stack_action_menu_popup`
as a retained popup anchored to that tab. Routing still lives in
`domain/editor/editor_shell/src/composition/build_editor_shell.rs::register_tab_stack_chrome_routes`
so the shell projection remains the command authority for tab-stack actions.
Outside primary or secondary clicks close the active tab action/type popups via
`apps/runenwerk_editor/src/shell/controller.rs::handle_tab_popup_dismiss_event`.
Toolbar menu popups use the same outside-click close policy through
`apps/runenwerk_editor/src/shell/controller.rs::handle_toolbar_menu_dismiss_event`.
Closing the last tab in a tab stack is treated as an area close by
`apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::dispatch_shell_command`,
so the empty area is removed or collapsed instead of leaving an inert one-tab
shell. Persisted workspace layouts are also normalized by
`domain/editor/editor_shell/src/workspace/reducer.rs::compact_empty_tab_stack_areas`
when loaded or saved, so older layouts that already contain empty non-root
tab-stack hosts are repaired instead of being reloaded as permanent empty docks.

Viewport details are projected by
`domain/editor/editor_shell/src/composition/build_viewport_panel.rs::build_viewport_panel`
through an Options popup, not by a direct Show Details button. The Options popup
contains reusable toggle controls for Details, Statistics, and viewport root
opacity; product choices are rendered as selectable product buttons and
descriptor-only products remain disabled. Enabled viewport metadata
is rendered as a retained bottom overlay anchored inside the viewport canvas,
uses a 50% alpha panel background, and does not push the viewport surface down.
Dock split resizing is owned by
`apps/runenwerk_editor/src/shell/controller.rs::handle_split_resize_event`; the
current split hit target is the projected split border zone rather than a
visible handle widget. The workspace projection assigns stable split widget ids
and structural metadata directly from the graph, so default profile layouts and
dynamic user-created splits use the same resize path. Split border intersections
create a corner-resize session through
`apps/runenwerk_editor/src/shell/state.rs::CornerSplitResizeSession`; dragging
there updates both participating split fractions. A resize started without Shift
may then use Shift during the drag to preserve the current first-quadrant
width/height ratio. A pointer-down that already has Shift held does not start a
resize session, leaving Shift-drag corner gestures available for area splitting.
Split-border hit testing runs before tab activation and tab dragging, so a
pointer on the border still starts resize even when the same pixel also overlaps
the edge of a tab button.
Cursor
feedback for invisible split border and corner zones is derived by
`apps/runenwerk_editor/src/shell/controller.rs::RunenwerkEditorShellController::cursor_intent_for_pointer`
and written to `engine::WindowState` by
`apps/runenwerk_editor/src/runtime/systems/input_bridge.rs::dispatch_editor_input_system`.
Shift-dragging a non-split area corner inward creates a new split area through
`apps/runenwerk_editor/src/shell/state.rs::CornerAreaSplitSession` and
`ShellCommand::SplitTabStackArea`; the command still goes through the normal
workspace reducer rather than mutating projected layout state directly.
Tab dragging uses the same workspace projection to infer insertion from
tab-stack/container geometry; highlighted tab insertion is represented as
reserved tab-strip space, not a button-like drop-zone control. Dock split
dragging carries explicit area, group, and workspace scope candidates:
individual panel edges split the area, enclosing split-host edges/gaps split the
group, and outer body edges split the workspace. All valid scope candidates are
projected as visual-only previews, the active preview receives the stronger
accent and label, and `Tab` cycles overlapping candidates on the same side.
Empty dock placeholders and explicit shelf targets keep their structural hit
targets without rendering instructional "drop here" copy. Retained popups carry
explicit layer order:
viewport-local overlays use the lower overlay layer, while menu/dropdown popups
use the higher menu layer so viewport overlays cannot cover open menus. The
longer-term node split for menu popups, overlay adornments, dock previews, and
radial menus is tracked in
`docs-site/src/content/docs/design/active/editor-ui-popup-adornment-drop-preview-contract.md`.

Viewport runtime binding is app-owned. `apps/runenwerk_editor/src/runtime/systems/viewport_lifecycle.rs::sync_viewport_instances_system`
syncs explicit viewport instances from workspace state before frame submission,
persists viewport-owned camera/debug/root/product settings back to workspace
state, and prunes closed viewport runtime records,
and `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::populate_viewport_layout_map_from_shell_tree`
scans the projected shell tree for retained `ViewportSurfaceEmbed` nodes and
binds each embedded viewport through its structural widget context. Replacement
viewport surfaces created by the surface-type menu receive their viewport id from
`apps/runenwerk_editor/src/runtime/viewport/instance_registry.rs::ViewportInstanceRegistryResource`
rather than inheriting an unrelated lone observed viewport. `apps/runenwerk_editor/src/runtime/viewport/layout_map.rs::ViewportLayoutMapResource`
stores entries by `StructuralWidgetRoutingContext`, not only by `ViewportId`,
so split/replacement viewport surfaces do not overwrite each other's
shell/runtime binding. `apps/runenwerk_editor/src/runtime/viewport/render_state.rs::ViewportRenderStateResource`
records per-viewport bounds and viewport-local camera/debug/root snapshots used
by viewport render jobs and picking. Frame submit derives per-viewport render
state from runtime bindings and retained workspace settings; the retained
`EditorViewportRenderState` resource remains only as render-flow default/helper
state, not the live viewport authority.

The viewport migration path is closed. Per-viewport product targets are allocated
by `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs::ViewportProductTargetRegistryResource`,
one render job is published per visible viewport by
`apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::ViewportRenderJobResource`,
and `assets/shaders/editor_viewport_scene_product.wgsl` renders one
viewport-local target per job without multi-rectangle containment. The
product catalog now exposes scene color, picking ids, overlay, depth,
diagnostics, scalar field, vector field, atlas, volume slice, brickmap debug,
and history color descriptors. Only descriptors with a concrete app-owned target
record are marked available; future field/asset/volume/history descriptors are
visible as unavailable products rather than being routed to fallback surfaces.
The
no-compromise target and follow-up product maturity work are documented in
`docs-site/src/content/docs/design/active/workspace-viewport-expression-upgrade-design.md`;
the end-to-end implementation sequence is
`docs-site/src/content/docs/apps/runenwerk-editor/viewport-expression-implementation-roadmap.md`.

Default workspace profiles are defined in
`domain/editor/editor_shell/src/workspace/profile.rs::default_workspace_profile_registry`.
The Scene and Modelling profiles are distinct workspace profiles; both currently
use graph-backed shell templates while retaining separate profile identity and
profile-addressed layout persistence. Scene defaults to viewport plus right
outliner/entity and inspector stacks above a bottom console. Modelling defaults
to outliner/entities on the left, viewport in the middle, inspector on the
right, and the same bottom console band. The Editor Design profile uses
`WorkspaceState::bootstrap_editor_design_layout` to expose self-authoring
definition, preview, validation, styling, binding, and diff surfaces.

The default structural layout is defined in
`domain/editor/editor_shell/src/workspace/state.rs::WorkspaceState::bootstrap_current_layout`.
It places the viewport in the expanding left/middle area, the scene hierarchy
above the inspector in the right sidebar, and the console/log surface in the
bottom band. Shell composition renders that structure through
`domain/editor/editor_shell/src/workspace/projection.rs::project_workspace_for_shell`;
there is no separate fixed-layout composition path.
Workspace layout persistence is profile-addressed. When
`apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::load_workspace_profile_layout`
loads a saved profile layout, it verifies that the saved mounted surface kinds
match the target profile before accepting it; incompatible saved layouts fall
back to the profile's default workspace state instead of silently booting the
wrong workspace shape.

## Self-Authoring State

The app-owned self-authoring document lifecycle lives in
`apps/runenwerk_editor/src/shell/self_authoring.rs::SelfAuthoringWorkspaceState`.
It loads checked-in UI fixtures and editor-owned schema documents as editable
definition documents, validates them through `domain/editor/editor_definition`,
forms retained previews through `domain/ui/ui_definition`, edits draft UI node
text/theme color/workspace-layout data through explicit shell commands, and
keeps explicit applied snapshots for rollback. Apply and rollback are shell commands handled in
`apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`, preserving the
app/shell command boundary.

Applied definition activation is intentionally separate from draft editing and
snapshot storage. `apps/runenwerk_editor/src/shell/applied_editor_definition.rs`
is the stable facade for the app-owned activation seam from an applied editor
definition document into live editor products. Activation intent mapping lives
in `apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs`,
active catalog storage lives in
`apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs`, and
replacement compatibility checks live in
`apps/runenwerk_editor/src/shell/applied_editor_definition/compatibility.rs`.
Theme documents now form `ui_theme::ThemeTokens` through
`domain/editor/editor_definition/src/theme.rs::form_theme_tokens`; the editor
app queues the applied document, and
`apps/runenwerk_editor/src/runtime/resources.rs::EditorHostResource::apply_pending_editor_definition_activations`
updates the live host theme before the next submitted shell frame is built.
Workspace layout documents now also use this activation seam: applied workspace
layout definitions are formed through
`domain/editor/editor_shell/src/workspace/definition_form.rs::form_workspace_state_from_definition`
and installed with
`apps/runenwerk_editor/src/shell/state.rs::RunenwerkEditorShellState::replace_workspace_state`.
The checked-in self-authoring default theme seeded by
`apps/runenwerk_editor/src/shell/self_authoring.rs::SelfAuthoringWorkspaceState::from_checked_in_fixtures`
is intentionally black/dark, keeps control radius at zero, and uses compact
panel spacing so applying that authored theme does not reintroduce the older
lighter rounded shell look.

Definition export is versioned through
`apps/runenwerk_editor/src/shell/self_authoring.rs::EditorDefinitionExportPackage`
instead of serializing a bare definition document.

This live activation path is implemented for theme, workspace layout, UI
template, editor binding, menu, shortcut, command-binding, panel-registry, and
tool-surface-registry documents. Active definition catalogs live in
`apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs::ActiveEditorDefinitionCatalogs`
and are installed by
`apps/runenwerk_editor/src/runtime/resources.rs::EditorHostResource::apply_pending_editor_definition_activations`
before the next shell frame is submitted.

Activated UI templates and editor bindings feed the next shell frame through
`domain/editor/editor_shell/src/surface_provider.rs::EditorShellFrameModel::with_active_ui_definitions`.
The retained toolbar and shell chrome consume the active templates while keeping
fallback checked-in fixtures. Menu, shortcut, and command-binding definitions
activate into app-owned catalogs; command bindings map authored route targets to
existing command ids and never execute commands from definition data. Activated
tool-surface registries also feed future switch/create surface choices through
`EditorShellFrameModel::with_available_tool_surface_kinds`. Existing workspace
state is left in place unless a workspace layout definition is applied, and
panel/tool-surface registry activation is blocked when the current workspace
still references removed or incompatible definitions.

## Related Docs

- Domain editor contracts: [`../../domain/editor/README.md`](../../domain/editor/README.md)
- Editor definition architecture: [`../../domain/editor/editor-definition/current-architecture.md`](../../domain/editor/editor-definition/current-architecture.md)
- UI architecture: [`../../domain/ui/architecture.md`](../../domain/ui/architecture.md)
- Editor roadmap: [`roadmap.md`](roadmap.md)
