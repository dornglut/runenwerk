---
title: Editor Self-Authoring and UI Workspace Design
description: Now-track product and architecture design for Runenwerk editor/UI self-authoring, UI workspace, layout design, styling, creation and management flows.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-08
related:
  - ./ui-definition-formation-foundation-design.md
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ./editor-workspace-document-mode-panel-architecture.md
  - ./workspace-identity-contract-and-migration-map.md
  - ../../apps/runenwerk-editor/execution-priority-checklist.md
  - ../../domain/ui/roadmap.md
  - ../../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
---

# Editor Self-Authoring and UI Workspace Design

## Status

Active design.

This document is now part of the immediate editor/UI roadmap. The implementation order is M3.5 UI definition formation framework first, then M3.6 UI self-authoring workspace before the asset/procedural/gameplay expansion milestones.

The required order is:

1. M1 through M3 editor structural, shell, docking/tab, document/mode, and scene-authoring foundations;
2. M3.5 UI definition formation framework;
3. M3.6 UI workspace and self-authoring implementation;
4. Integrated M4 UI/editor live replacement plus asset foundation work, where active menus, shortcuts, command bindings, panel registries, and tool-surface registries are consumed before new asset/import/field-product surfaces expand the editor.
5. Later procedural, gameplay, runtime, overlay, and in-game UI work built on authored UI/style definitions instead of new hard-coded shell/app UI.

## Purpose

Define the target user experience and architecture for Runenwerk to design, style, validate, preview, and manage:

- editor workspace layouts;
- dock hosts, tab stacks, panels, and tool surfaces;
- UI layouts and UI documents;
- runtime/game UI templates and overlay/debug UI templates as authored definitions;
- menus, command bindings, shortcuts, and themes;
- authored editor definitions that can later be packaged into specialized editors.

This is the design for the long-term goal:

```text
Code defines the first editor.
The editor now starts defining editor UI.
Later features consume authored UI instead of adding new hard-coded UI paths.
```

## Core Definitions

### Editor Area

An editor area is a hosted workspace region. In current code this maps to:

- `PanelHostId` for layout containers;
- `TabStackId` for tab containers;
- `PanelInstanceId` for a hosted tab/panel instance.

### Editor Type

An editor type is the selected tool surface hosted in an editor area.

In Runenwerk this maps to:

- `ToolSurfaceKind`;
- later `SurfaceDefinitionId`;
- a mounted `ToolSurfaceInstanceId`.

This is the Runenwerk equivalent of Blender's editor type selector.

### Area Tab

An area tab is a `PanelInstanceId` in a `TabStackId`. It hosts a `ToolSurfaceInstanceId`.

Area tabs are not document tabs.

### Document Tab

A document tab is an open authored or inspected target in `editor_core`.

Examples:

- scene document;
- UI document;
- graph document;
- script document;
- runtime debug query document;
- editor-design document.

Document tabs determine the semantic context that providers observe.

### Authored Definition

An authored definition is editable content that describes editor or UI structure before it becomes active runtime/editor state.

Examples:

- workspace layout definition;
- UI layout definition;
- menu definition;
- theme definition;
- shortcut definition;
- command binding definition;
- panel registry definition;
- tool-surface definition.

### General UI Definition

A general UI definition is editor-agnostic authored UI structure. It can describe UI nodes, layout, menu/popover structure, theme-token references, and generic action slots.

It must not encode editor workspace profiles, panel/tab identity, `ToolSurfaceKind`, provider registry policy, or concrete editor command execution.

It also must not encode retained runtime details such as `UiNodeKind`, `WidgetId`, focus/capture ids, ECS entity ids, or compiled update functions. Those belong to concrete formation targets. The M3.5 target is retained UI formation; future accepted compiled-reactive or ECS-driven strategies should add separate formation products from the same normalized source model.

### Editor Definition

An editor definition is editor-specific authored structure. It can describe workspace profiles, default layouts, editor menus, shortcuts, command bindings, panel registries, and tool-surface definitions.

It may reference general UI definitions and formed UI products, but it owns editor semantics separately from `domain/ui`.

## Product Goals

The final editor UI must let users:

- open and manage multiple documents;
- switch workspaces by task;
- add, close, move, split, and float editor areas;
- switch an area's editor type through a visible selector;
- add a new area tab through a visible plus button;
- design editor layouts visually;
- design UI layouts visually;
- design and preview editor, debug overlay, runtime overlay, and game UI templates as authored definitions;
- style UI through theme tokens, state styles, typography, spacing, colors, and layout constraints;
- create, duplicate, rename, delete, import, export, validate, and migrate authored definitions;
- preview a definition before applying it;
- apply valid definitions through ratified commands;
- undo/redo self-authoring edits;
- understand validation errors without reading logs.

## Non-Goals For The First Self-Authoring Version

Do not start by building:

- standalone editor packaging/export tooling;
- a public plugin marketplace;
- a universal no-code game builder;
- arbitrary scripting execution inside UI definitions;
- full animation/effects tooling;
- every future tool surface.

The first version should prove authored editor/UI definition editing and activation inside the host editor.

It may author runtime/game UI templates before the runtime binding path is complete. In that case preview uses explicit fixtures and diagnostics, and activation into the game runtime remains gated by the later owning runtime/game UI integration.

## Final Editor UI Design

### Top-Level Layout

The target editor window is organized as:

```text
+--------------------------------------------------------------------------------+
| Main menu | Workspace switcher | Command palette | Save/Play/Simulate status   |
+--------------------------------------------------------------------------------+
| Document tabs: Scene | UI | Graph | Editor Design | +                          |
+--------------------------------------------------------------------------------+
| Mode/tool bar: Select Move Rotate Scale Layout Bind Style Preview ...          |
+--------------------------------------------------------------------------------+
| Left dock     | Center dock / viewport / canvas / graph       | Right dock     |
| tab stacks    | tab stacks                                    | tab stacks     |
|               |                                                |                |
+--------------------------------------------------------------------------------+
| Bottom dock: console, diagnostics, validation, profiler                         |
+--------------------------------------------------------------------------------+
| Status bar: selection, document dirty state, validation count, focused context  |
+--------------------------------------------------------------------------------+
```

### Main Menu

Required menus:

- File;
- Edit;
- View;
- Workspace;
- Document;
- Create;
- Tools;
- Window;
- Help.

Menu content is eventually authored through menu definition documents.

### Workspace Switcher

The workspace switcher chooses the active task preset.

Initial target workspaces:

- Layout;
- UI;
- Graphs;
- Scripting;
- Debug;
- Editor Design.

Switching a workspace changes layout, default tool surfaces, and mode set. It does not change document payload data.

### Document Tab Bar

Document tabs manage authored targets.

Required controls:

- active document tab;
- document dirty marker;
- close document;
- plus button for new/open document;
- context menu for duplicate, rename, save as, reveal, close others.

Document tabs are backed by `DocumentId`, not `PanelInstanceId`.

### Mode/Tool Bar

The mode/tool bar exposes context-valid modes and tools.

Examples:

- Select;
- Translate;
- Rotate;
- Scale;
- Place;
- Layout;
- Style;
- Bind;
- Preview;
- Validate.

Mode availability is derived from `(workspace_profile, document_kind)`.

### Editor Area Chrome

Every tab stack/area should have consistent chrome:

```text
+-------------------------------------------------------------------+
| [Editor Type Select] [Tab A] [Tab B] [+]        [split] [menu] [x] |
+-------------------------------------------------------------------+
| active tool surface content                                        |
+-------------------------------------------------------------------+
```

Required controls:

- editor type selector dropdown;
- area tabs;
- plus button for new area tab;
- area menu;
- split controls;
- close control;
- drag target for docking/reordering.

### Editor Type Selector

The editor type selector changes the active tab/panel's hosted tool surface while preserving workspace structural identity.

Initial options:

- Viewport;
- Outliner;
- Entity Table;
- Inspector;
- Console;
- UI Hierarchy;
- UI Canvas;
- Style Inspector;
- Bindings;
- Actions;
- Graph Canvas;
- Node Palette;
- Diagnostics;
- Theme Editor;
- Shortcut Editor;
- Menu Editor;
- Command Palette;
- Panel Registry;
- Tool Surface Registry.

Current code has the internal command path for this behavior through `ShellCommand::SwitchPanelToolSurfaceKind`. The final UI must expose it as a select/dropdown route.

### Area Plus Button

The plus button creates a new area tab in the current `TabStackId`.

Expected flow:

1. User clicks `+` in a tab stack.
2. A new tab menu opens.
3. User chooses an editor type.
4. The shell allocates a new `PanelInstanceId`.
5. The shell allocates a new `ToolSurfaceInstanceId`.
6. The new panel is inserted into the target `TabStackId`.
7. The new tab becomes active.
8. Provider resolution builds the selected surface for the active document context.

The plus button is separate from document-tab creation.

### Area Menu

The area menu owns structure-level commands:

- split horizontally;
- split vertically;
- duplicate area;
- close area;
- close other tabs;
- move tab to new floating area;
- reset area;
- lock area type;
- save area layout as preset.

### Docking and Split UX

Target interactions:

- drag an area tab to reorder inside the same tab stack;
- drag an area tab to another tab stack;
- drag an area tab to a dock marker to split;
- drag an area tab out to create a floating host;
- drag split handles to resize;
- save layout;
- restore layout;
- reset workspace layout.

### Status Bar

The status bar should show:

- active workspace;
- active document;
- active mode;
- selected item count;
- validation error count;
- dirty state;
- focused surface;
- command feedback.

## Final Workspace Designs

### Layout Workspace

Purpose: 3D scene and SDF graybox authoring.

Default layout:

- left: Outliner + Entity Table;
- center: Viewport;
- right: Inspector;
- bottom: Console + Diagnostics.

Default modes:

- select;
- place;
- translate;
- rotate;
- scale;
- paint;
- lighting;
- nav;
- physics-debug.

### UI Workspace

Purpose: authored UI layout and runtime UI preview.

Default layout:

- left: UI Hierarchy + Asset/Widget Palette;
- center: UI Canvas Preview;
- right: Style Inspector + Binding Inspector;
- bottom: Diagnostics + Preview Console.

Default modes:

- layout;
- style;
- bind;
- animation;
- preview;
- validate.

### Graphs Workspace

Purpose: graph document editing.

Default layout:

- left: Graph Outliner + Node Palette;
- center: Graph Canvas;
- right: Params + Inspector;
- bottom: Preview + Diagnostics.

Default modes:

- edit;
- connect;
- simulate;
- debug.

### Scripting Workspace

Purpose: script assets and hot-reload/debug workflows.

Default layout:

- left: API Browser + File/Script Outliner;
- center: Script Editor;
- right: Command/Event References;
- bottom: Diagnostics + Console.

Default modes:

- edit;
- run-test;
- reload;
- trace.

### Debug Workspace

Purpose: runtime inspection.

Default layout:

- left: Runtime Outliner + ECS Query Viewer;
- center: Viewport/Runtime Preview;
- right: Component Inspector;
- bottom: Event Log + Profiler + Console.

Default modes:

- inspect;
- capture;
- replay;
- profiling.

### Editor Design Workspace

Purpose: author editor and UI definitions inside Runenwerk.

Default layout:

- left: Definition Outliner;
- center: Dock/Layout Preview or UI Canvas;
- right: Inspector + Registry Panels;
- bottom: Validation + Command Diff + Console.

Default modes:

- layout-edit;
- ui-layout-edit;
- command-bind;
- create/manage;
- style-edit;
- validation;
- preview.

## Self-Authoring Document Types

### Workspace Layout Definition

Owns:

- split host tree;
- tab stacks;
- panel instances;
- default tool-surface definitions;
- default active panels;
- default floating hosts;
- constraints and restore policy.

### UI Layout Definition

Owns:

- UI node tree;
- layout policies;
- style classes;
- bindings;
- action routes;
- preview states;
- responsive variants.

### Menu Definition

Owns:

- menu tree;
- command references;
- labels;
- enablement rules;
- workspace/document filters.

### Theme Definition

Owns:

- color tokens;
- spacing tokens;
- typography tokens;
- border/radius tokens;
- state colors;
- contrast rules.

### Shortcut Definition

Owns:

- key chords;
- command references;
- context filters;
- conflict policy;
- platform overrides.

### Command Binding Definition

Owns:

- command ids;
- capability requirements;
- UI placement;
- route target;
- undo/redo semantics;
- validation behavior.

### Panel Registry Definition

Owns:

- available panel/editor area types;
- display labels;
- icons;
- default tool surface;
- allowed document/workspace contexts.

### Tool-Surface Definition

Owns:

- surface identity;
- required capabilities;
- retention class;
- provider family;
- presentation requirements;
- command route contract.

## Definition Lifecycle

Self-authored definitions follow:

```text
Authored -> Normalized -> Formed -> Instantiated
```

### Authored

Editable, user-facing source content.

Allowed to contain:

- human names;
- draft ids;
- incomplete values;
- disabled items;
- validation warnings.

### Normalized

Canonical, validated definition representation.

Responsibilities:

- resolve references;
- assign stable ids;
- canonicalize ordering;
- reject structural cycles;
- reject missing required fields;
- report warnings.

### Formed

Runtime/editor-ready product.

Responsibilities:

- convert definitions into workspace shell contracts;
- prepare provider requests;
- prepare command routes;
- prepare theme tokens;
- prepare concrete UI products for the selected execution target.

For M3.5, the selected target is a retained UI product consumed by `ui_tree`, `ui_widgets`, and `ui_runtime`. Future accepted compiled-reactive or ECS-driven strategies may add `CompiledUiProgram` or `EcsUiSpawnPlan`-style products without changing authored template identity.

### Instantiated

Active runtime/editor state.

Responsibilities:

- allocate session identities;
- mount tool surfaces;
- create retained UI tree state;
- bind document context;
- enter command routing.

## Create And Manage Flows

### Create Definition

1. User chooses `Create`.
2. User chooses definition type.
3. The editor creates an authored draft.
4. The draft opens as a document tab.
5. Validation runs immediately.
6. The user edits until no blocking issues remain.
7. The user saves or applies the definition.

### Duplicate Definition

1. User selects an existing definition.
2. User chooses duplicate.
3. The editor copies authored source with new stable definition identity.
4. Internal references are remapped where ownership requires it.
5. The duplicate opens as a dirty document.

### Delete Definition

Delete should be guarded.

Rules:

- active definitions cannot be deleted without replacement;
- referenced definitions require dependency confirmation;
- deletion creates a ratified change and is undoable where possible.

### Rename Definition

Rename changes display identity, not stable identity.

Stable ids must not be inferred from labels.

### Import Definition

Import runs normalization and migration before the definition can be applied.

### Export Definition

Export writes authored source plus version metadata.

Packaging/export of standalone editors is later scope.

### Migrate Definition

Migrations are explicit versioned steps.

The UI must show:

- source version;
- target version;
- changed fields;
- blocking issues;
- manual follow-up requirements.

## Layout Editing UX

### Workspace Layout Editing

Core operations:

- add area;
- split area horizontally;
- split area vertically;
- close area;
- move tab;
- add tab;
- change editor type;
- float area;
- dock floating area;
- resize split;
- reset area;
- save as preset.

All operations produce explicit workspace-definition commands.

### UI Layout Editing

Core operations:

- add UI node from palette;
- select UI node;
- reparent UI node;
- reorder UI node;
- edit layout policy;
- edit text/style/token references;
- bind action;
- bind data source;
- preview state;
- validate responsive variants.

The UI design surface should use the same retained UI substrate as runtime UI, but authored UI definitions must not directly mutate active runtime UI state.

## Final Self-Authoring UI Surfaces

### Definition Outliner

Shows:

- open definitions;
- document hierarchy;
- references;
- validation state;
- dirty state.

### Dock/Layout Preview

Shows:

- split tree;
- tab stacks;
- panels;
- tool-surface placeholders;
- floating hosts;
- drop targets.

### UI Canvas

Shows:

- authored UI tree;
- layout bounds;
- selected node handles;
- responsive preview size;
- state preview.

### Registry Panels

Show:

- available editor types;
- panels;
- tool surfaces;
- commands;
- menus;
- themes;
- shortcuts.

### Inspector

Edits selected definition nodes.

Inspector fields must route through command proposals and ratification, not direct state mutation.

### Validation Panel

Shows:

- blocking errors;
- warnings;
- migrations;
- unresolved references;
- capability issues;
- command route issues.

### Command Diff Panel

Shows the exact change that will be applied before formation or activation.

## Command And Ratification Model

Self-authoring mutations must use explicit commands.

Command families:

- `editor.definition.create`;
- `editor.definition.duplicate`;
- `editor.definition.delete`;
- `editor.definition.rename`;
- `editor.definition.import`;
- `editor.definition.export`;
- `editor.workspace_layout.split_area`;
- `editor.workspace_layout.close_area`;
- `editor.workspace_layout.add_tab`;
- `editor.workspace_layout.move_tab`;
- `editor.workspace_layout.set_editor_type`;
- `editor.ui_layout.add_node`;
- `editor.ui_layout.remove_node`;
- `editor.ui_layout.reparent_node`;
- `editor.ui_layout.set_property`;
- `editor.menu.add_item`;
- `editor.menu.remove_item`;
- `editor.theme.set_token`;
- `editor.shortcut.bind`;
- `editor.shortcut.unbind`.

Ratification must check:

- current workspace/document compatibility;
- capability requirements;
- valid target identity;
- no structural cycles;
- no duplicate stable ids;
- no dangling references;
- command route safety;
- migration compatibility.

## Persistence And Versioning

Definitions should be persisted as versioned authored documents.

Target ownership:

- editor definition schemas: future domain editor definition crate/module;
- workspace structural contracts: `domain/editor/editor_shell`;
- document/session references: `domain/editor/editor_core`;
- UI node/layout contracts: `domain/ui`;
- app file IO and project storage: `apps/runenwerk_editor`.

Do not store active runtime-only identities as durable authored identities.

Durable ids:

- definition id;
- authored node id;
- authored panel id;
- authored tool-surface slot id;
- command id;
- theme token id.

Session ids:

- `PanelInstanceId`;
- `ToolSurfaceInstanceId`;
- UI runtime widget focus/capture state;
- temporary preview ids.

## Architecture Ownership

### Planned `domain/ui/ui_definition`

Owns general UI definition and formation contracts:

- authored UI node/layout/menu/popover/theme-reference/action-slot definitions;
- stable authored UI ids distinct from runtime widget, focus, capture, shell, and ECS ids;
- validation, normalization, migration where needed, and formation into concrete UI products;
- diagnostics for generic UI structure, references, availability, and unsupported formed products.

It must not own editor workspaces, tool-surface semantics, provider registries, or command execution.

### `domain/editor/editor_core`

Owns:

- document descriptors;
- document kind taxonomy;
- active document switching;
- mode contracts;
- ratified editor change records.

### `domain/editor/editor_shell`

Owns:

- workspace structural graph;
- tab stack and panel identity;
- tool-surface host contracts;
- surface capability/retention contracts;
- projection/routing contracts.

### Future Editor Definition Domain

Owns:

- editor-specific authored definition schemas;
- workspace profile catalogs and default layout definitions;
- editor menu, shortcut, command binding, panel registry, and tool-surface definitions;
- normalization;
- formation;
- migration;
- validation;
- self-authoring operation contracts.

It may consume general UI definition contracts, but it must not move generic UI substrate behavior or runtime widget identity into editor-authored definitions.

### `domain/ui`

Owns:

- retained UI nodes;
- layout;
- input;
- focus/capture;
- text;
- theme tokens;
- reusable widgets;
- render data contracts.

It must not own editor semantics.

### `apps/runenwerk_editor`

Owns:

- concrete host registry composition;
- app-owned provider implementations;
- file IO;
- runtime integration;
- preview instantiation;
- engine/window integration.

It must not become the semantic owner of self-authoring definitions.

## Guard Coverage Required

Add or preserve guards that prevent:

- shell layout code from owning document semantics;
- UI substrate from depending on editor-specific semantics;
- authored definitions from becoming active without formation;
- editor area type switches from changing `PanelInstanceId`;
- new tab creation from reusing stale `ToolSurfaceInstanceId`;
- document tabs from being confused with area tabs;
- self-authoring commands from mutating definitions without ratification;
- invalid definitions from becoming active;
- active layouts from persisting session-only ids as authored ids.

## Implementation Phases

### Roadmap Placement

This design is implemented as M3.6 in `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`.

M3.6 is not a replacement for M3.5. It consumes the M3.5 definition framework and makes it usable through an editor workspace before the integrated M4 UI/editor/asset foundation expands live definition consumption and introduces asset/import/field-product surfaces.

The later M8 roadmap slot is no longer the first self-authoring implementation. It is reserved for packaging, externalization, and long-lived migration hardening after the main editor feature tracks exist.

### Phase 0 - Design Closeout

Outputs:

- this design;
- updated Now checklists;
- explicit acceptance criteria.

### Phase 0.5 - UI Definition Formation Framework

Implement:

- planned `domain/ui/ui_definition` contracts for authored UI ids, nodes, layout, menu/popover structure, slots, repeaters, template refs, embeds, theme references, validation, normalization, and read-only formation into retained UI products as the first target;
- planned `domain/editor/editor_definition` bindings for editor toolbar, menus, workspace catalogs, shell chrome, and common provider surface templates;
- generic availability/diagnostic representation for disabled or unavailable UI without routing unavailable commands;
- tests proving authored ids do not reuse runtime `WidgetId`, focus/capture ids, `PanelInstanceId`, `ToolSurfaceInstanceId`, or ECS entity ids.

This is the M3.5 roadmap slice. It enables the M3.6 visual authoring workspace.

### Phase 1 - M3.6 Authored Definition Lifecycle

Implement:

- editor definition document lifecycle;
- UI definition document lifecycle;
- durable schemas for workspace layout definitions, UI layouts, themes, menus, shortcuts, command bindings, and references to formed UI definitions;
- normalization and validation reports;
- read-only formation into current workspace contracts.

### Phase 2 - M3.6 UI Workspace Shell

Implement:

- UI workspace profile;
- editor type selector dropdown;
- plus/new-tab button;
- new panel/tool-surface allocation command;
- tab close/menu controls;
- shell interaction mapping for select/dropdown changes;
- default UI workspace layout with UI hierarchy, UI canvas, style inspector, bindings/actions, validation, and command diff surfaces.

### Phase 3 - M3.6 Workspace Layout Designer

Implement:

- Editor Design workspace;
- workspace outliner;
- dock/layout preview;
- split/add/move/close operations;
- layout validation;
- apply preview.

### Phase 4 - M3.6 UI Layout Designer

Implement:

- UI document kind;
- UI hierarchy surface;
- UI canvas preview;
- style inspector;
- binding/action panel;
- UI layout validation.

### Phase 5 - M3.6 Styling, Menus, Shortcuts, And Themes

Implement:

- menu definition documents;
- shortcut definition documents;
- theme definition documents;
- style inspector property editing;
- theme token editing;
- state style previews for normal, hover, active, disabled, selected, and focused control states;
- conflict validation;
- preview/apply flows.

### Phase 6 - M3.6 Formation, Migration, And Activation

Implement:

- versioned migrations;
- preview fixtures for editor, debug overlay, runtime overlay, and game UI templates;
- apply command;
- rollback/revert;
- import/export for authored definition documents.

### Phase 7 - M8 Packaging Later

Later only:

- standalone specialized editor packaging;
- plugin distribution;
- external editor-definition SDK.

## Acceptance Criteria

M3.6 may start after M3.5 closes with active `domain/ui/ui_definition` and `domain/editor/editor_definition` crates, checked-in RON fixtures, retained formation, diagnostics, and explicit route/embed products.

The promoted self-authoring milestone is complete when:

- a user can create a workspace layout definition;
- a user can add/split/move/close areas in the layout designer;
- a user can add a tab and choose editor type;
- a user can create and edit a UI definition document in the UI workspace;
- a user can select UI nodes through the hierarchy and canvas;
- a user can style selected UI nodes through theme tokens and state style controls;
- a user can bind value, collection, route, availability, and embed slots through explicit binding panels;
- editor, debug overlay, runtime overlay, and game UI template categories can be authored and previewed with fixture data;
- validation catches malformed layouts;
- validation catches malformed UI templates, slot refs, route refs, theme refs, and unsupported embeds;
- a valid definition can be formed into an active workspace/UI preview;
- valid definitions can be applied or rolled back through ratified commands;
- edits are undoable through ratified commands;
- no active runtime/session-only ids are persisted as authored ids.
