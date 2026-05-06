---
title: UI Definition Formation Framework Design
description: M3.5 design for execution-neutral UI definitions, RON-authored templates, slots, embeds, retained UI formation, and editor UI template migration.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-06
related:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../apps/runenwerk-editor/roadmap.md
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ./editor-self-authoring-and-final-ui-design.md
  - ../deferred/ui-model-multiple-execution-strategies-design.md
---

# UI Definition Formation Framework Design

## Status

Active M3.5 design. The implementation is at validated closeout-candidate state as of 2026-05-06; M3.6 visual self-authoring has not started.

This is an implementation design for a full first UI definition framework milestone. It is not a visual UI editor, plugin system, scripting system, or alternate UI execution runtime. The promoted M3.6 self-authoring workspace consumes this framework immediately after M3.5.

Current implementation truth for the M3.5 closeout candidate:

- `domain/ui/ui_definition` and `domain/editor/editor_definition` are active workspace crates.
- Checked-in RON fixtures exist under `assets/editor/ui/` for toolbar, shell chrome, common provider surfaces, and editor bindings.
- UI fixtures parse, validate, normalize, and can form retained UI products with route slots, embed slots, authored paths, diagnostics, and availability state.
- `editor_shell` uses formed toolbar route slots for migrated toolbar routing; the toolbar compatibility entrypoint is now a thin wrapper around definition formation.
- normal tab-stack shell chrome is formed from `assets/editor/ui/shell_chrome.ron`; dynamic drag/drop preview chrome remains live shell behavior.
- the console provider surface structure is formed from `assets/editor/ui/surfaces/console.ron` while provider data and follow-scroll behavior stay app/shell-owned.
- `apps/runenwerk_editor` has an app-owned checked-in fixture loading/validation path.
- Remaining common provider fixtures are checked-in and validated, but richer inspector/outliner/entity-table/viewport migrations should only proceed when their specific row/form/embed semantics can be preserved without moving provider behavior into templates.

## Purpose

M3.5 introduces `domain/ui/ui_definition` so reusable UI structure can be described as data, loaded from RON fixtures/templates, validated, normalized, and formed into the existing retained UI tree/runtime products.

The milestone also introduces the first editor-specific definition layer so editor chrome, menus, workspace switchers, and common provider surface structure can be defined through templates and bindings rather than more hard-coded shell branching.

The first acceptance surface is still the editor toolbar, but the milestone is not toolbar-only. It should establish the reusable framework needed for:

- top-level editor toolbar and menu rows;
- workspace switcher/catalog presentation;
- tab-stack and area chrome;
- common provider surfaces such as outliner, inspector, entity table, console, and viewport chrome;
- dynamic lists/forms through repeaters and collection slots;
- viewport/canvas/tool surfaces through embed slots.

Today, toolbar structure and routing are mostly code-defined through:

- `apps/runenwerk_editor/src/shell/toolbar_adapter.rs::build_toolbar_observation_frame`
- `domain/editor/editor_shell/src/composition/build_toolbar.rs::build_toolbar`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_frame_widget_routes`

Some provider surface structure still remains code-defined through builders such as:

- `domain/editor/editor_shell/src/composition/build_inspector_panel.rs::build_inspector_panel`
- `domain/editor/editor_shell/src/composition/build_outliner_panel.rs::build_outliner_panel`
- `domain/editor/editor_shell/src/composition/build_entity_table_panel.rs::build_entity_table_panel`
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::build_viewport_panel`

M3.5 should make these stable UI structures template-formed where practical, while preserving explicit provider/domain/app command routing.

## Core Decision

`domain/ui/ui_definition` owns execution-neutral UI source/IR.

The M3.5 formation target is retained UI:

```text
AuthoredUiTemplate
  -> validate
  -> NormalizedUiTemplate
  -> FormedRetainedUiProduct
  -> UiNode / UiTree
  -> ui_runtime
```

The authored and normalized model must not be retained UI in disguise. It must not encode `UiNodeKind`, `WidgetId`, focus/capture ids, ECS entity ids, concrete shell commands, or compiled update functions.

If a future active design or accepted ADR adds compiled-reactive or ECS-driven UI execution, those paths should become additional formation targets from `NormalizedUiTemplate`:

```text
NormalizedUiTemplate
  -> FormedRetainedUiProduct
  -> future CompiledUiProgram
  -> future EcsUiSpawnPlan
```

That future should not require rewriting authored toolbar templates.

The milestone should therefore be feature-complete for the definition framework, even though it is not feature-complete for visual self-authoring. Feature-complete here means templates, slots, repeaters, embeds, availability, diagnostics, retained formation, route slot products, RON fixtures, and migration of current editor chrome/common surface structure.

## Current Pain Points

The current toolbar path has several hard-coded seams:

- menu and workspace entries are constructed in app Rust code;
- toolbar composition maps `stable_name` strings to fixed widget ids;
- routing is assembled by matching the same `stable_name` strings;
- disabled or unavailable items are represented as button enablement and route suppression, not as reusable availability data;
- custom workspace catalogs and future menu/popover behavior would add more Rust branching.

M3.5 fixes the architecture of those seams for the toolbar and extends the same model to common editor chrome and provider surface structure.

Other current hard-coded UI structures share the same failure mode:

- tab-stack chrome in `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`;
- inspector rows and editable fields in `domain/editor/editor_shell/src/composition/build_inspector_panel.rs`;
- outliner tree/list structure in `domain/editor/editor_shell/src/composition/build_outliner_panel.rs`;
- entity table search/header/table structure in `domain/editor/editor_shell/src/composition/build_entity_table_panel.rs`;
- viewport chrome/product selector/details controls in `domain/editor/editor_shell/src/composition/build_viewport_panel.rs`;
- console panel list structure in `domain/editor/editor_shell/src/composition/build_console_panel.rs`.

These should become template-formed structure with provider-supplied data and route payloads.

## Ownership Split

### `domain/ui/ui_definition`

Owns:

- authored UI template ids, node ids, paths, and source locations;
- generic node definitions such as row, column, stack, panel, scroll, button, label, separator, text input, numeric input, toggle, select, tabs, table, tree, collection expansion, repeat, template reference, menu slot, route slot, embed slot, and availability reference;
- slot value contracts consumed during formation;
- validation and normalization diagnostics;
- first retained-tree formation target;
- formed route products, authored path maps, and source maps.

Must not own:

- editor workspace profiles;
- `ToolSurfaceKind`;
- `ToolbarCommandKind`;
- `ShellCommand`;
- app commands or project IO;
- ECS entities/components/systems;
- runtime `WidgetId` identity as authored identity.

### `domain/editor/editor_definition`

Owns in M3.5:

- editor toolbar bindings;
- editor menu definitions;
- workspace profile catalog definitions;
- command route ids and editor availability rule descriptors;
- surface template bindings for outliner, inspector, entity table, console, and viewport chrome.

It must not own active shell runtime state, provider execution, app IO, or command execution.

### `domain/editor/editor_shell`

Owns for M3.5:

- active shell/runtime bridge;
- mapping generic formed route slots to `RoutedShellAction`;
- integration with `build_editor_shell_frame_with_docking_visual_state`;
- conversion from editor definition outputs into shell route tables;
- explicit routing from accepted interactions to `ShellCommand`.

It should not keep editor menu/workspace definition semantics once `editor_definition` owns them.

### `apps/runenwerk_editor`

Owns:

- loading app-owned RON fixtures/assets where app IO is involved;
- fallback policy if editor UI definitions fail to load;
- concrete command execution;
- project/file/runtime integration.

## RON Authored Input

M3.5 should include RON-authored template and editor-binding input from day one.

Required:

- `AuthoredUiTemplate`, editor bindings, and related source types are serializable/deserializable.
- Checked-in RON fixtures exist for toolbar, shell chrome, and common provider surface templates.
- Tests parse the fixtures, validate them, normalize them, and form them.

Recommended fixture paths:

```text
assets/editor/ui/toolbar.ron
assets/editor/ui/shell_chrome.ron
assets/editor/ui/surfaces/inspector.ron
assets/editor/ui/surfaces/outliner.ron
assets/editor/ui/surfaces/entity_table.ron
assets/editor/ui/surfaces/console.ron
assets/editor/ui/surfaces/viewport.ron
assets/editor/ui/editor_bindings.ron
```

Runtime app loading is in scope. App startup may fall back to embedded defaults after reporting diagnostics, but tests should fail when checked-in fixtures are invalid.

M3.5 does not include the full authored document lifecycle:

- no user save/import/export workflow;
- no visual UI editor;
- no project-wide migration system;
- no packaging or plugin distribution;
- no arbitrary scripting.

## Definition Feature Set

M3.5 should define the full first reusable template model, not only the toolbar subset.

### Structural Nodes

Required:

- `Panel`
- `Row`
- `Column`
- `Stack`
- `Scroll`
- `Split` where needed for current shell compatibility
- `Spacer`
- `Separator` / divider

### Controls

Required where retained UI already has support:

- `Label`
- `Button`
- `Toggle`
- `TextInput`
- `NumericInput`
- `Select`
- `Tabs`
- `Table`
- `Tree`

### Dynamic Structure

Required:

- `ValueSlot`
- `SelectionSlot`
- `CollectionSlot`
- `Repeat`
- `TemplateRef`
- `RouteSlot`
- `AvailabilityRef`
- `EmbedSlot`

### Surface Embeds

Required:

- viewport/canvas embed slots that form to retained viewport embed products;
- provider-owned embed payload resolution outside `ui_definition`;
- route and source maps around embed chrome.

### Menus And Popovers

Required:

- menu definitions as data;
- menu item collections;
- active menu row formation for current toolbar behavior;
- popover/menu slot vocabulary for future real popovers.

M3.5 does not need animated or detached popover product polish, but it should not bake the current toolbar menu row into the authored model.

## Template Examples

### Toolbar

The toolbar template:

```ron
UiTemplate(
  id: "runenwerk.editor.toolbar",
  root: Row(
    id: "root",
    children: [
      Button(
        id: "menu_file",
        label: Static("File"),
        route: RouteSlot("editor.toolbar.menu.file"),
      ),
      Button(
        id: "menu_edit",
        label: Static("Edit"),
        route: RouteSlot("editor.toolbar.menu.edit"),
      ),
      Button(
        id: "menu_window",
        label: Static("Window"),
        route: RouteSlot("editor.toolbar.menu.window"),
      ),
      Separator(id: "separator"),
      CollectionButtons(
        id: "workspace_switcher",
        items: CollectionSlot("editor.workspace_profiles"),
        selected: SelectionSlot("editor.workspace.active"),
        route: RouteSlot("editor.workspace.activate"),
      ),
      Button(
        id: "add_workspace",
        label: Static("+"),
        route: RouteSlot("editor.workspace.create"),
        availability: AvailabilityRef("editor.workspace.create.available"),
      ),
      MenuRow(
        id: "active_menu_row",
        menu: MenuSlot("editor.toolbar.active_menu"),
      ),
    ],
  ),
)
```

This syntax is illustrative. The implementation may choose exact enum names that better match Rust and RON ergonomics.

### Inspector Surface

Inspector rows should use repeaters and field row templates:

```ron
UiTemplate(
  id: "editor.inspector.surface",
  root: Column(
    id: "root",
    children: [
      Label(id: "title", label: ValueSlot("inspector.title")),
      Repeat(
        id: "fields",
        items: CollectionSlot("inspector.fields"),
        template: TemplateRef("editor.inspector.field_row"),
      ),
    ],
  ),
)
```

```ron
UiTemplate(
  id: "editor.inspector.field_row",
  root: Row(
    id: "root",
    children: [
      Label(id: "name", label: ValueSlot("field.label")),
      TextInput(
        id: "value",
        value: ValueSlot("field.display_value"),
        route: RouteSlot("field.edit"),
        availability: AvailabilityRef("field.editable"),
      ),
    ],
  ),
)
```

The provider still owns field discovery, parse/commit behavior, and command proposals.

### Viewport Surface

Viewport chrome should be templated, while the canvas is embedded:

```ron
UiTemplate(
  id: "editor.viewport.surface",
  root: Column(
    id: "root",
    children: [
      Row(
        id: "chrome",
        children: [
          Select(
            id: "product_select",
            items: CollectionSlot("viewport.products"),
            selected: SelectionSlot("viewport.active_product"),
            route: RouteSlot("viewport.select_product"),
          ),
          Toggle(
            id: "details",
            label: Static("Details"),
            checked: ValueSlot("viewport.details_open"),
            route: RouteSlot("viewport.toggle_details"),
          ),
        ],
      ),
      EmbedSlot(
        id: "viewport_canvas",
        slot: "viewport.expression_product",
      ),
    ],
  ),
)
```

The provider/app/runtime still owns viewport product binding, picking, camera/tool state, and renderer integration.

## Menu And Popover Reality

M3.5 must be honest about current UI capability.

Current toolbar menu behavior is not a real detached popover. It is a second retained row controlled by active toolbar menu state. The M3.5 formation target may still render that behavior as a menu-row or collection expansion over the retained tree.

The authored model must still describe menus and popover intent generically. It should not encode "toolbar second row" as the only menu abstraction. Real detached overlay placement, animation, and richer focus behavior can remain a later retained UI/runtime improvement.

## Slot Values

Definitions describe structure. They need generic values supplied by the owner during formation.

`ui_definition` should include a generic slot value model:

```text
UiDefinitionContext
  templates
  slot values
  availability values
  widget id assignment policy
  source map
```

Useful first value types:

- static label text;
- collection item key and label;
- collection item metadata used by repeaters and menus;
- selected collection item key;
- route slot id;
- availability state;
- active/selected state.
- embed slot payload id.

Editor-specific code resolves these values from editor state. For example:

- workspace profile labels come from `WorkspaceProfileRegistry`;
- active workspace comes from shell state;
- inspector fields come from the inspector/provider DTOs;
- outliner rows come from retained tree/provider DTOs;
- viewport embed payloads come from tool-surface/provider runtime products;
- undo/redo availability comes from app/runtime history state;
- unavailable future commands become `Disabled` with a reason.

## Availability

Availability is mixed static and dynamic.

`ui_definition` owns generic representation:

```text
Available
Disabled(reason)
Unavailable(reason)
AvailabilityRef(id)
```

Editor or app layers resolve editor-specific rules such as:

- `RequiresActiveDocument`;
- `CanUndo`;
- `CanRedo`;
- `StaticDisabled("not_implemented")`.

Formation emits the current concrete availability into retained controls. Routing must also fail closed. A disabled retained button must not emit an activation, and shell route mapping must still refuse unavailable actions.

## Identity Model

Authored identity:

- stable semantic ids;
- stable authored paths;
- source locations for diagnostics;
- suitable for tests and future visual-editor inspection.

Runtime identity:

- `WidgetId`;
- focus/capture ids;
- `PanelInstanceId`;
- `ToolSurfaceInstanceId`;
- ECS entity ids;
- temporary preview ids.

Runtime identity must not be persisted as authored UI identity.

M3.5 needs an explicit widget id assignment contract:

```text
AuthoredUiNodePath -> WidgetId
```

For existing migrated editor chrome, adapters may map known paths to existing constants in `domain/editor/editor_shell/src/ids/widget_ids.rs` to preserve behavior and tests. Longer term, formation can use scoped deterministic allocation with collision diagnostics.

## Formed Retained Product

The first formed product should be explicit:

```text
FormedRetainedUiProduct
  root: UiNode
  routes_by_widget_id: map WidgetId -> FormedUiRoute
  paths_by_widget_id: map WidgetId -> AuthoredUiNodePath
  embeds_by_widget_id: map WidgetId -> FormedUiEmbed
  diagnostics
```

Routes remain inert:

```text
FormedUiRoute
  RouteSlot(id)
  CollectionItemRoute { collection, item_key, route }
```

`ui_definition` does not map these routes to `ShellCommand`.

`editor_shell` maps formed route slots to `RoutedShellAction`, and `domain/editor/editor_shell/src/commands/map_interactions.rs::map_interactions_to_shell_commands` remains the accepted interaction to command boundary.

## Existing Types To Reuse

Reuse:

- `ui_tree::UiNode`, `UiNodeKind`, and `WidgetId` for retained output only;
- `ui_widgets` constructors for retained formation;
- `ui_theme::ThemeTokens` and `ui_text::TextStyle` for formed rendering attributes;
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::RoutedShellAction`;
- `domain/editor/editor_shell/src/commands/shell_command.rs::ShellCommand`;
- `domain/editor/editor_shell/src/view_models/toolbar.rs::ToolbarViewModel` as an interim value source;
- `domain/editor/editor_shell/src/workspace/profile.rs::WorkspaceProfileRegistry` for workspace values;
- current editor shell composition functions as behavior references while they are migrated:
  - `domain/editor/editor_shell/src/composition/build_toolbar.rs::build_toolbar`;
  - `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`;
  - `domain/editor/editor_shell/src/composition/build_inspector_panel.rs::build_inspector_panel`;
  - `domain/editor/editor_shell/src/composition/build_outliner_panel.rs::build_outliner_panel`;
  - `domain/editor/editor_shell/src/composition/build_entity_table_panel.rs::build_entity_table_panel`;
  - `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::build_viewport_panel`;
  - `domain/editor/editor_shell/src/composition/build_console_panel.rs::build_console_panel`;
- `foundation/diagnostics` for diagnostic shape where practical.

Do not replace:

- `ui_runtime` input handling;
- `map_interactions_to_shell_commands`;
- app command execution in `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`;
- retained UI rendering/submission path.

## Implementation Sequence

### Phase 1 - Crate Skeletons

Add:

- `domain/ui/ui_definition/Cargo.toml`
- `domain/ui/ui_definition/src/lib.rs`
- `domain/editor/editor_definition/Cargo.toml`
- `domain/editor/editor_definition/src/lib.rs`
- workspace metadata in `Cargo.toml`
- crate inventory in `CRATES.md`
- ownership maps in `DOMAIN_MAP.md` and docs-site domain overview.

Dependency direction:

```text
domain/ui/ui_definition
domain/editor/editor_definition -> domain/ui/ui_definition
domain/editor/editor_shell -> domain/editor/editor_definition
apps/runenwerk_editor -> domain/editor/editor_shell
```

If `editor_shell` depending on `editor_definition` would create a cycle in the concrete implementation, move the active adapter into `apps/runenwerk_editor` and keep `editor_definition` as DTOs plus validation/formation helpers. Do not make `editor_definition` depend on `editor_shell`.

### Phase 2 - Source Model And RON Fixtures

Add:

- `identity.rs`
- `template.rs`
- `node.rs`
- `slot.rs`
- `availability.rs`
- `source.rs`
- `diagnostic.rs`
- `value.rs`
- `menu.rs`
- `embed.rs`

Add fixtures:

- `assets/editor/ui/toolbar.ron`
- `assets/editor/ui/shell_chrome.ron`
- `assets/editor/ui/surfaces/inspector.ron`
- `assets/editor/ui/surfaces/outliner.ron`
- `assets/editor/ui/surfaces/entity_table.ron`
- `assets/editor/ui/surfaces/console.ron`
- `assets/editor/ui/surfaces/viewport.ron`
- `assets/editor/ui/editor_bindings.ron`

Tests:

- parse all checked-in fixtures;
- reject malformed ids;
- reject duplicate node ids.

### Phase 3 - Definition Feature Coverage

Implement generic definitions for:

- structural nodes: `Panel`, `Row`, `Column`, `Stack`, `Scroll`, `Split`, `Spacer`, `Separator`;
- controls backed by existing retained widgets: `Label`, `Button`, `Toggle`, `TextInput`, `NumericInput`, `Select`, `Tabs`, `Table`, `Tree`;
- dynamic forms: `ValueSlot`, `SelectionSlot`, `CollectionSlot`, `Repeat`, `TemplateRef`;
- routing and state: `RouteSlot`, `AvailabilityRef`;
- integration holes: `EmbedSlot`.

This phase is still definition/data work. It must not add scripting, plugins, or a visual editor.

### Phase 4 - Validation And Normalization

Add:

- `validate.rs`
- `normalize.rs`

Validation:

- duplicate template ids;
- duplicate authored node ids in one template;
- empty ids;
- unresolved required slots;
- duplicate slot ids;
- unresolved template refs;
- repeaters with no item source;
- embed slots with no owning resolver;
- route slots with no editor binding where the editor layer requires one;
- invalid collection item keys;
- invalid availability references.

Normalization:

- stable authored paths;
- canonical child order;
- resolved slot references;
- source map preservation.

### Phase 5 - Retained Formation

Add:

- `form.rs`

Formation:

- form supported structural nodes and controls into `UiNode`;
- form repeaters, collection controls, tabs, tables, trees, menus, and current menu-row products;
- form embed placeholders for provider/runtime-owned products;
- emit route slots by `WidgetId`;
- emit authored path by `WidgetId`;
- emit embed slots by `WidgetId`;
- preserve disabled/unavailable state in retained controls;
- detect widget id collisions.

### Phase 6 - Editor Definition Layer

Add:

- `domain/editor/editor_definition/src/toolbar.rs`
- `domain/editor/editor_definition/src/menu.rs`
- `domain/editor/editor_definition/src/command.rs`
- `domain/editor/editor_definition/src/workspace.rs`
- `domain/editor/editor_definition/src/surface.rs`
- `domain/editor/editor_definition/src/availability.rs`
- `domain/editor/editor_definition/src/binding.rs`
- `domain/editor/editor_definition/src/validate.rs`
- `domain/editor/editor_definition/src/form_editor_ui.rs`

Responsibilities:

- define editor menu ids and menu item descriptors;
- define workspace profile catalog entries and selected profile slots;
- define editor route ids without importing `ShellCommand`;
- define availability rule descriptors such as `RequiresActiveDocument` and `StaticDisabled`;
- bind editor toolbar, shell chrome, and provider surface templates to editor data sources;
- validate editor bindings before shell/app activation.

### Phase 7 - Editor Toolbar And Shell Chrome Migration

Add:

- `domain/editor/editor_shell/src/composition/toolbar_definition.rs`
- `domain/editor/editor_shell/src/composition/shell_chrome_definition.rs`

Update:

- `domain/editor/editor_shell/src/composition/mod.rs`
- `domain/editor/editor_shell/src/composition/build_toolbar.rs::build_toolbar`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_tab_strip_from_frame`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_editor_shell_frame_with_docking_visual_state`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs::build_frame_widget_routes`

The shell build order should become:

```text
form toolbar definition
  -> get toolbar UiNode and formed routes
  -> build widget route map from formed routes plus workspace/surface routes
  -> assemble root UiTree
```

The toolbar path should stop depending on `stable_name` string matching for routing.

The tab-stack/area chrome path should use templates for stable chrome structure while keeping workspace mutation, panel identity allocation, and docking ratification in `editor_shell`.

Implementation note: normal tab-stack chrome is migrated through `shell_chrome.ron`; dynamic docking preview/drop-slot rendering remains live shell behavior.

### Phase 8 - Provider Surface Structure Migration

Update:

- `domain/editor/editor_shell/src/composition/build_inspector_panel.rs::build_inspector_panel`
- `domain/editor/editor_shell/src/composition/build_outliner_panel.rs::build_outliner_panel`
- `domain/editor/editor_shell/src/composition/build_entity_table_panel.rs::build_entity_table_panel`
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::build_viewport_panel`
- `domain/editor/editor_shell/src/composition/build_console_panel.rs::build_console_panel`

Target:

- template-form repeated list/tree/table/form/chrome structure;
- keep provider DTOs as the value/collection/embed source;
- keep provider-local route results and scene/app command proposals explicit;
- remove migrated ad hoc row/button/string routing code once parity tests pass.

This migration is structure-only. It must not move scene editing, inspector commit, viewport picking, renderer integration, or provider execution into `ui_definition`.

Implementation note: console panel structure is migrated through `surfaces/console.ron` and keeps console follow-scroll state in app/shell code. Inspector, outliner, entity-table, and viewport fixtures are checked in and validated; their retained builders should remain behavior-owned until richer row/form/embed parity can be preserved without encoding provider semantics in `ui_definition`.

### Phase 9 - App Fixture Loading

M3.5 should load checked-in RON fixtures through the app/runtime setup path.

Policy:

- tests fail on invalid checked-in fixtures;
- app startup reports structured diagnostics on invalid fixtures;
- app startup may fall back to embedded defaults only when the fallback is explicit and covered by tests.

## Tests

`domain/ui/ui_definition`:

- all RON UI fixtures parse;
- duplicate authored ids fail validation;
- unresolved slots fail validation;
- unresolved template refs fail validation;
- repeaters form stable paths for repeated children;
- embed slots form inert embed products;
- disabled availability forms a disabled retained button;
- formed widget ids are unique;
- formed routes are inert route slots;
- authored ids never equal or wrap runtime ids.

`domain/editor/editor_definition`:

- editor binding fixture parses;
- menu ids, workspace catalog slots, route ids, and availability refs validate;
- editor bindings reject unresolved UI template ids;
- editor route products do not import or execute `ShellCommand`;
- surface template bindings resolve provider slot names without provider execution.

`domain/editor/editor_shell/src/tests.rs`:

- formed toolbar contains File/Edit/Window controls;
- active menu produces the expected retained menu-row items;
- Add Workspace is disabled and produces no command;
- Scene/Modelling workspace items route to `ShellCommand::SwitchWorkspaceProfile`;
- File/Edit/Window route to `ShellCommand::ToggleToolbarMenu`;
- toolbar routing no longer depends on `stable_name` matching.
- tab-stack chrome routes still produce the existing shell commands;
- console surface formation preserves existing follow-scroll behavior;
- validated inspector/outliner/entity-table/viewport fixtures do not execute provider behavior.

`apps/runenwerk_editor`:

- valid UI/editor fixture set is accepted during shell setup;
- invalid fixture set falls back or fails according to policy and reports diagnostics;
- scene authoring smoke still passes after provider surface structure migration.

## Validation Commands

Required for M3.5 closeout:

```text
cargo test -p ui_definition
cargo test -p editor_definition
cargo test -p editor_shell
cargo test -p runenwerk_editor -p editor_shell -p ui_definition -p editor_definition
cargo metadata --no-deps
python3 tools/docs/validate_docs.py
```

Use `./quiet_full_gate.sh` only for milestone closeout when broader repository validation is appropriate.

## Exit Criteria

M3.5 is complete when:

- `domain/ui/ui_definition` is an active workspace crate;
- `domain/editor/editor_definition` is an active workspace crate;
- toolbar, shell chrome, and common provider surface templates can be RON-authored and parsed from checked-in fixtures;
- authored and normalized templates are execution-neutral;
- retained formation produces the editor toolbar, shell chrome, and migrated common surface structure;
- formed output contains route slots, embed slots, and authored path/source maps;
- shell maps formed route slots to existing explicit commands;
- disabled/unavailable actions render disabled and route fail-closed;
- no authored ids persist runtime widget, shell, or ECS identities;
- migrated legacy builders are removed or reduced to thin compatibility wrappers with no duplicated source-of-truth structure;
- docs and crate inventory are updated;
- validation commands pass.

## Explicit Deferrals

- visual UI editor;
- user-authored UI document lifecycle;
- project persistence and migrations for UI definitions;
- detached/animated dropdown or popover runtime polish beyond the current retained formation capability;
- compiled-reactive UI execution;
- ECS-driven UI execution;
- scripting or plugin-defined UI behavior;
- standalone editor packaging.

## Risks And Guards

### Risk: retained UI leaks into authored source

Guard:

- authored model has no `WidgetId` or `UiNodeKind`;
- tests assert formed output is the first place `WidgetId` appears.

### Risk: command execution leaks into `ui_definition`

Guard:

- route slots are inert ids;
- shell/app layers map them to command enums;
- `ui_definition` does not depend on `editor_shell`.

### Risk: circular dependency through editor definition

Guard:

- `editor_definition` depends on `ui_definition` and editor semantic contracts, not active shell runtime internals;
- `editor_shell` consumes editor definition products or an app-level adapter composes them;
- no `ShellCommand` import is allowed in `ui_definition`, and any `editor_definition` command ids remain semantic route descriptors.

### Risk: RON fixture becomes a half-built persistence system

Guard:

- fixture loading and serde are in scope;
- project document lifecycle, migration, import/export, and visual editing are out of scope.

### Risk: menus claim real popover support too early

Guard:

- M3.5 supports generic menu/popover definitions and forms current menus into retained products;
- detached overlay polish requires later `ui_tree`/`ui_runtime` primitives and tests.

### Risk: full migration pulls behavior into templates

Guard:

- templates own structure only;
- providers own data discovery, editor semantics, app IO, renderer/runtime payloads, and command proposals;
- route slots and embed slots stay inert until shell/app layers resolve them.

## Rule

M3.5 delivers the first full UI definition framework and migrates the stable editor chrome/common surface structure to it. It does not build the visual editor or move provider/app behavior into templates; M3.6 builds the visual authoring workspace on top of the formed definition contracts.
