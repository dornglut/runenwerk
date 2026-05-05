---
title: Editor Self-Authoring and Final UI Design
description: Target product and architecture design for Runenwerk editor/UI self-authoring, layout design, creation and management flows, and final editor UI.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-05
related:
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ./editor-workspace-document-mode-panel-architecture.md
  - ./workspace-identity-contract-and-migration-map.md
  - ../../apps/runenwerk-editor/execution-priority-checklist.md
  - ../../domain/ui/roadmap.md
  - ../../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
---

# Editor Self-Authoring and Final UI Design

## Status

Active design.

This document makes the self-authoring track implementation-design-level. It does not mean implementation should start before the current editor/UI structural priorities are closed.

The required order remains:

1. document tabs and active document switching;
2. provider registry and app-domain operation migration closeout;
3. scoped workspace/document modes;
4. docking/tab product completion;
5. reachable editor-area type switching and plus/new-tab affordances;
6. self-authoring implementation.

## Purpose

Define the target user experience and architecture for Runenwerk to design and manage:

- editor workspace layouts;
- dock hosts, tab stacks, panels, and tool surfaces;
- UI layouts and UI documents;
- menus, command bindings, shortcuts, and themes;
- authored editor definitions that can later be packaged into specialized editors.

This is the design for the long-term goal:

```text
Code defines the first editor.
The editor can later define editors.
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

## Product Goals

The final editor UI must let users:

- open and manage multiple documents;
- switch workspaces by task;
- add, close, move, split, and float editor areas;
- switch an area's editor type through a visible selector;
- add a new area tab through a visible plus button;
- design editor layouts visually;
- design UI layouts visually;
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
- prepare UI tree templates.

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

- authored definition schemas;
- normalization;
- formation;
- migration;
- validation;
- self-authoring operation contracts.

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

### Phase 0 - Design Closeout

Outputs:

- this design;
- updated Now checklists;
- explicit acceptance criteria.

### Phase 1 - Definition Foundation

Implement:

- authored definition ids;
- minimal schemas for workspace layout and UI layout definitions;
- normalization and validation reports;
- read-only formation into current workspace contracts.

### Phase 2 - Area Chrome Productization

Implement:

- editor type selector dropdown;
- plus/new-tab button;
- new panel/tool-surface allocation command;
- tab close/menu controls;
- shell interaction mapping for select/dropdown changes.

### Phase 3 - Workspace Layout Designer

Implement:

- Editor Design workspace;
- workspace outliner;
- dock/layout preview;
- split/add/move/close operations;
- layout validation;
- apply preview.

### Phase 4 - UI Layout Designer

Implement:

- UI document kind;
- UI hierarchy surface;
- UI canvas preview;
- style inspector;
- binding/action panel;
- UI layout validation.

### Phase 5 - Menus, Shortcuts, Themes

Implement:

- menu definition documents;
- shortcut definition documents;
- theme definition documents;
- conflict validation;
- preview/apply flows.

### Phase 6 - Formation, Migration, And Publishing

Implement:

- versioned migrations;
- apply/publish command;
- rollback/revert;
- import/export.

### Phase 7 - Packaging Later

Later only:

- standalone specialized editor packaging;
- plugin distribution;
- external editor-definition SDK.

## Acceptance Criteria

The self-authoring system is ready for implementation only when:

- document tabs and active document switching are implemented;
- provider routing is closed enough to support new document/workspace contexts;
- scoped modes are available;
- docking/tab controls are productized;
- editor type switching is exposed in UI;
- plus/new-tab creation is exposed in UI;
- authored definition schemas are specified;
- validation/formation is specified;
- command/ratification boundaries are specified;
- persistence/migration strategy is specified.

The first implementation milestone is complete when:

- a user can create a workspace layout definition;
- a user can add/split/move/close areas in the layout designer;
- a user can add a tab and choose editor type;
- validation catches malformed layouts;
- a valid definition can be formed into an active workspace preview;
- edits are undoable through ratified commands;
- no active runtime/session-only ids are persisted as authored ids.
