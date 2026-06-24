---
title: Editor UI Workspace Tool Surface Architecture
description: Architecture for editor UI workspace surfaces, tool-surface ownership, viewport-local interaction, and presentation separation.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-06-19
related:
  - ../implemented/ui-definition-formation-foundation-design.md
  - ../implemented/editor-self-authoring-and-final-ui-design.md
  - ../implemented/editor-workspace-document-mode-panel-architecture.md
related_designs:
  - ../accepted/app-neutral-ui-composition-design.md
  - ../accepted/adaptive-ui-composition-design.md
  - ./editor-tool-suite-registry-and-workbench-host-design.md
---

# Editor / UI / Workspace / Tool-Surface Architecture

## Status

Active architecture baseline. The host/editor shell still owns runtime
composition, routing, and command execution, but authored editor/UI definitions
already exist for UI, layout, theme, menu, shortcut, binding, catalog, panel,
and tool-surface documents. The self-authoring/UI Designer path is a promoted
capability, not a distant future stage.

## Purpose
Define the long-term architecture for Runenwerk’s editor, UI, workspace, docking, tab-hosting, and tool-surface framework.

This document is not a visual polish plan and not a short-term panel refactor. It defines the architecture that should let Runenwerk:
- power the main editor
- power in-game UI
- power runtime/debug tooling
- host many specialized tool surfaces inside one editor workspace
- host and manage dockable, tabbed tool-surface workflows cleanly
- host source-backed authored editor/UI definitions without moving app IO,
  provider behavior, runtime state, or command execution into `domain/ui`

This architecture must correspond to Runenwerk’s nine-layer doctrine and avoid making the editor shell the semantic center of the system.

---

## Strategic Decision

Runenwerk is a **modular host editor and reusable UI/tooling framework** with a
source-backed authoring path for editor/UI definitions.

The durable rule is:

```text
Authored UI / editor definitions
  -> validation / normalization
  -> formed interaction contracts
  -> formed retained UI product
  -> ui_runtime enforcement
  -> render/product-surface output
```

The code-owned host/editor shell still owns runtime composition, app IO,
provider behavior, provider state, and command execution. Authored definitions
may configure and replace allowed products only through validation,
compatibility checks, formed products, and ratified shell/app command paths.

Renderer output is derived product data. It must not become UI authority.

### App-neutral cutover boundary

The accepted app-neutral composition design supersedes this document as the
target structural authority. This document remains current-code migration input
until the clean cutover removes the old workspace and tool-surface authorities:

```text
WorkspaceState and split/tab/floating records -> CompositionState and region graph
ToolSurfaceInstanceId                    -> MountedUnitId
EditorSurfaceProvider                    -> EditorUnitProvider
native editor window binding             -> PresentationTargetId adapter
```

Existing state remains temporary compatibility input only. After the editor
static-projection checkpoint it is read-only; after the docking checkpoint all
structural mutation goes through `ui_composition` transactions. Provider and
content semantics remain editor-owned.

---

## Problem Statement

Without a clear editor/UI/workspace architecture, new features tend to become:
- shell-specific panel hacks
- renderer-coupled widgets
- one-off interaction systems
- duplicated input/focus/capture logic
- ad hoc state ownership in the main editor app
- weak dock/tab/tool-surface identity
- special tooling paths that do not generalize to graphs, timelines, color tools, field viewers, or in-game UI

That would conflict with Runenwerk’s doctrine, which requires explicit boundaries between authored content, observation, expression, ratified change, retention, and tooling.

---

## Non-Negotiable Goals

The architecture must:
- support the main Runenwerk editor as a modular host
- support in-game UI on the same core substrate
- support runtime/debug tools on the same core substrate
- support advanced tool surfaces such as viewport, graphs, timelines, curves, color tools, and specialized viewers
- support docked and tabbed tool-surface workflows with stable identity
- preserve a strict split between observation, local interaction/session state, commands, and ratified mutations
- keep shell/workspace concerns separate from tool semantics
- support embedded products/surfaces such as viewport outputs and future field/atlas/volume views
- preserve source-backed authored editor/workspace/tool-definition architecture

The architecture should leave room for later:
- draggable tabs and tab reordering/reassignment
- docking/floating windows
- saved workspaces
- authored editor definitions and compatibility-gated replacement
- packaging/export of specialized editors

---

## Explicit Non-Goals For The First Major Version

The first major version should not try to fully solve:
- standalone editor packaging/export
- full docking/floating-window productization
- complete styling/animation/effects system
- all possible tool surfaces at once

The first major version should prove the framework and the promoted
self-authoring path, not finish the entire editor-construction platform.

---

## Architectural Principles

### 1. The shell is not the semantic center
The shell/workspace composes, hosts, routes, and persists layout-oriented editor concerns.
It does not own the meaning of every tool.

### 2. Tool surfaces are first-class
A viewport, graph editor, timeline, curve editor, color tool, atlas viewer, or diagnostics surface is not a special-case panel implementation.
Each is a first-class tool surface with its own observation model, presentation state, interaction/session state, command pathway, and mutation boundary.

### 3. Reuse the substrate, not the semantics
The shared framework should reuse:
- input
- focus/capture
- layout
- text
- theming
- render data
- canvas/surface embedding
- common controls

But tool semantics should remain in their tool domains.

### 4. Observation, interaction, commands, and ratification must stay distinct
Every serious tool surface must distinguish:
- what it observes
- what transient interaction/session state it owns
- what commands it emits
- what changes become committed/ratified mutations

### 5. Products and previews are explicit
Embedded rendered content must travel through explicit presentation/product contracts rather than hidden renderer-private assumptions.

### 6. Identity is first-class
Workspace, host, panel, document, tab, and tool-surface identity must be explicit and stable enough for composition, routing, persistence, and future authored definitions.

### 7. Build architecture first, breadth second
The framework should be architecturally correct before it is broad. New capabilities should extend existing seams instead of bypassing them.

### 8. Prevent shell creep
The shell/workspace layer must not become the hidden owner of tool state, domain meaning, or renderer semantics merely because it hosts and routes them.

---

## Nine-Layer Alignment

## Layer 1 — Runtime Simulation
Owns live runtime state used by tools and runtime UI.
Examples:
- game/runtime state
- simulation state
- live world state
- runtime animation state
- runtime debug counters

UI/framework relevance:
- tools and runtime UI may observe and interact with simulated state
- the framework must not make UI surfaces the owner of simulation truth

## Layer 2 — Mutation / Ratification
Owns committed changes.
Examples:
- confirmed graph edit
- committed property change
- accepted tool action
- command execution
- workspace definition change later

UI/framework relevance:
- local interaction must not be confused with committed mutation
- commands/actions should remain explicit when they affect authoritative editor/runtime state

## Layer 3 — Retention / Recovery
Owns reconstructability, history, persistence lineage, and later migration-ready editor definitions.
Examples:
- undo/redo
- workspace persistence
- panel layout history
- draft state recovery
- future editor-definition migration lineage

UI/framework relevance:
- workspace and tool-surface state must be retention-aware where appropriate
- authored editor-definition architecture later must fit this layer naturally

## Layer 4 — Observation
Owns observed forms used by tool surfaces and UI.
Examples:
- inspector observation frame
- outliner observation frame
- graph observation frame
- timeline observation frame
- viewport artifact observation frame

UI/framework relevance:
- tool surfaces consume observation-facing models instead of raw domain internals

## Layer 5 — Authority / Partition
Owns who may produce, expose, or mutate different realities.
Examples:
- local tool-only debug data
- partition-owned world data
- shared collaboration state later

UI/framework relevance:
- tool surfaces remain consumers and local interaction owners, not accidental authority owners

## Layer 6 — Asset / Content
Owns authored content.
Examples:
- scene documents
- graphs
- timelines/clips
- material definitions
- future workspace/editor definitions

UI/framework relevance:
- authored editor-definition architecture later belongs here
- many tool surfaces are authoring environments for Layer 6 content

## Layer 7 — Expression
Owns consumer-facing products and previews.
Examples:
- viewport products
- atlas previews
- field visualizations
- preview textures
- diagnostics visualizations

UI/framework relevance:
- tool surfaces embed products/previews through explicit presentation contracts

## Layer 8 — Sharing / Replication
Owns shared/remote tooling or runtime collaboration later.
Examples:
- collaborative editing
- streamed previews
- shared debug sessions

UI/framework relevance:
- keep boundaries clean so later sharing/replication is possible

## Layer 9 — Editor / Tooling
Owns workspace composition, tools, host editor behavior, and local tool interaction semantics.
Examples:
- shell/workspace
- viewport surface
- graph editor
- timeline
- color tool
- diagnostics panels

UI/framework relevance:
- this document primarily defines Layer 9 structure while preserving the integrity of the lower layers

---

## Scope Decision

### Architectural target
Runenwerk should be architected to support:
- editor
- in-game UI
- runtime/debug tools
- source-backed authored editor-definition workflows
- future standalone/specialized editor packaging compatibility

### Implementation target for the first serious framework version
Implement for:
- the main editor host
- in-game UI substrate compatibility
- runtime/debug tool compatibility
- advanced tool surfaces inside the host editor
- dock-aware workspace composition inside the host editor

Do not implement full authored-editor packaging or a complete external-tool platform in the first version.

---

## Core Architecture Shape

The framework should be organized into four major strata.

## 1. Core UI substrate
Reusable everywhere.

Owns:
- input event model
- focus and capture
- keyboard/pointer routing primitives
- layout primitives
- sizing/measurement primitives
- render-data primitives
- text primitives
- theming/tokens
- common control contracts
- embedded surface/canvas primitives

This layer should be host-agnostic.
It must serve:
- editor
- in-game UI
- runtime debug UI
- future specialized surfaces

## 2. Workspace / shell composition layer
Primarily for the host editor and serious tool environments.

Owns:
- workspace identity
- panel host identity
- panel instance identity
- tab-hosted tool-surface hosting
- split composition initially
- tabs later
- docking later
- workspace composition
- panel focus and activation
- workspace-local routing and persistence hooks

This layer composes tool surfaces and their hosts. It does not define tool semantics.

## 3. Tool-surface framework
Reusable pattern for advanced interactive surfaces.

Owns the common model for:
- observation input
- local presentation state
- local interaction/session state
- interaction routing
- canvas/surface behavior
- optional embedded products/previews
- command emission
- ratified mutation boundaries

This should be the shared pattern for:
- viewport
- graph editor
- timeline
- curve editor
- atlas/texture viewer
- field/volume viewer
- diagnostics surfaces

## 4. Domain-specific tools and UI
Concrete tools built on the substrate.

Examples:
- Runenwerk host editor
- inspector
- outliner
- viewport
- graph tools
- animation timeline
- color tools
- runtime debug overlays
- in-game menus/HUD

---

## Identity Model

Identity must be explicit and stable enough for routing, persistence, composition, and future authored definitions.

The architecture should distinguish at least conceptually between:
- `WorkspaceId`
- `PanelHostId`
- `PanelInstanceId`
- `ToolSurfaceInstanceId`
- `TabStackId`

### Principles
- a tool surface instance is not the same thing as a panel instance
- a panel host is not the same thing as a tab stack
- a tab stack is a workspace composition container, not a semantic owner
- identity must survive resize, relayout, activation changes, and later drag/reparent operations where appropriate

This model must remain explicit even before all later workspace features are implemented.

---

## Docking and Tab Hosting Model

The workspace architecture must eventually support docked and tabbed hosting for tool surfaces.

### Distinctions
- a **tool surface instance** is a semantic editing/viewing surface
- a **panel instance** is a hosted UI/container instance
- a **tab stack** is a workspace composition container
- a **panel host** is the layout-level host for one or more tabbed or non-tabbed panel instances

### Rules
- tabs are workspace composition constructs, not semantic owners of tools
- tool-surface identity must survive layout rearrangement
- tool semantics remain inside the hosted surface, not inside the tab system
- dragging/reordering tabs is a workspace concern
- future split/reparent/move operations must not change the meaning of the hosted tool surface

### First-version implication
Tabs do not need full implementation in the first serious version, but the identity and hosting model must not block them.

---

## Host Editor Responsibilities

The host editor is the concrete Layer 9 application that assembles the first serious tool environment.

The host editor owns:
- workspace bootstrapping
- tool registration
- default workspace composition
- dock host and tab-stack composition policy
- top-level command routing policy
- persistence integration
- cross-tool coordination where needed

The host editor does not own:
- every tool’s semantics
- renderer-private product generation semantics
- local tool interaction semantics
- domain truth merely because it hosts a tool

### Principle
The host editor is a composition and coordination authority, not the semantic owner of every domain-specific behavior.

---

## Recommended UI Model

The framework should use a **retained tree UI + tool-surface/canvas hybrid**.

### Retained tree UI for:
- ordinary controls
- layout hierarchy
- focus/capture ownership
- form-style panels
- menus/toolbars
- shell composition
- dock/tab host composition later

### Tool-surface/canvas model for:
- viewport
- graphs
- timelines
- curves
- atlas/field/volume viewers
- advanced visual tools

### Rejected alternatives

#### Immediate-mode as the primary core model
Rejected because it is weak for complex editor interactions, persistent focus/capture, advanced tool surfaces, and long-lived workspace state.

#### Runtime-world UI ownership as the primary architecture
Rejected because the editor shell must keep explicit authored/runtime separation, command-owned mutation, and retained UI frame/render-data lowering as the implemented execution path.

#### CSS-like implicit cascade as the primary styling model
Rejected because Runenwerk should prefer explicit, typed, auditable contracts over highly implicit styling behavior.

---

## Long-Term UI Definition Direction

### Decision

Keep the retained tree UI plus tool-surface/canvas hybrid as the production
execution target, and keep the UI definition formation framework above
`domain/ui` as source/IR.

The definition layer is not retained UI in disguise. It validates and
normalizes authored definitions, forms interaction contracts, and forms retained
UI products for the current runtime. If a future active design or accepted ADR
adds compiled-reactive or ECS-driven execution, those must be additional
formation targets from the normalized definition model plus formed interaction
contracts.

The current split is:

- `domain/ui/*`: retained UI tree/runtime, layout, input, focus/capture, widgets, popovers, menus, theme tokens, generic UI definition formation, interaction contracts, and render-data contracts.
- `domain/ui/ui_definition`: general authored UI templates, slots, repeaters, embeds, menus, availability, validation, normalization, execution-neutral source/IR, source maps, and formation into concrete UI products.
- `domain/editor/editor_shell`: active editor workspace, panel/tab/tool-surface instance state, shell projection, shell routing, and Strangler adapters while formed contracts replace older local composition.
- `domain/editor/editor_definition`: editor-specific toolbar, workspace catalogs, command route ids, availability descriptors, editor menus, shell chrome bindings, provider surface template bindings, and authored layout definitions.
- `apps/runenwerk_editor`: concrete provider registry, provider state, file/project IO, runtime integration, activation policy, and preview/app instantiation.

### Options Considered

| Option | Fit | Decision |
| --- | --- | --- |
| Keep retained UI and add compiler-inspired definition/formation | Matches current implementation, Runenwerk's description-to-execution doctrine, explicit command boundaries, and future authored definitions. | Chosen. |
| Full ECS-driven UI | May fit narrow world-bound labels or simulation-linked overlays later, but risks making ECS entity identity into authored UI/editor identity and conflicts with the accepted retained path. | Deferred; requires active design or ADR. |
| Immediate-mode/debug-tool UI | Useful for throwaway diagnostics, but weak for durable focus/capture, menus, popovers, layout persistence, and self-authoring. | Not a primary architecture. |
| Web/React-like declarative UI | Useful as inspiration for authored definitions, but the runtime model, cascade assumptions, and backend expectations do not match current domain boundaries. | Do not adopt wholesale. |
| Hybrid general UI definition plus editor-specific definitions | Keeps substrate general while letting editor workspaces, commands, and tool-surface policy remain editor-owned. | Chosen refinement. |

### Pain Point Mapping

- toolbar/menu changes requiring shell routing edits: menu/action definitions should form generic UI menu products with explicit command route slots, while editor-specific command binding remains in editor definition/app layers;
- hard-coded workspace profiles and default layouts: editor definitions should own workspace catalogs and default shell layout definitions, then form into `editor_shell` workspace products;
- dropdown/popover primitives: generic anchored popup nodes and renderer-respected overlay layer ordering now belong in `domain/ui`/render integration; authored menu/popover definitions still need first-class UI definition syntax and formation instead of shell-specific composition;
- disabled/unavailable feature representation: formed products should carry availability/diagnostic state without routing fake or unavailable commands;
- missing custom workspace catalog: editor definition should own the catalog, not the shell runtime enum alone;
- self-authoring: the promoted UI Designer path edits the same definition families, not a second editor-only UI model.
- future UI execution strategies: compiled-reactive or ECS-driven UI should consume the same normalized UI definitions through separate formation targets, rather than requiring authored template rewrites.

### Boundary Rules

- Do not turn `domain/ui` into an editor semantics crate.
- Do not persist `WidgetId`, focus/capture ids, `PanelInstanceId`, `ToolSurfaceInstanceId`, or ECS entity ids as authored UI/editor ids.
- Do not encode retained `UiNodeKind`, ECS components, or compiled update functions in authored UI source.
- Preserve explicit command and ratification boundaries; formed UI may expose route slots, but execution remains with the owning editor/app/domain command path.
- Keep self-authoring on the same source-backed definition families. It may configure and replace allowed products, but it must not own app IO, provider behavior, runtime state, or command execution.

---

## Workspace Model

## First serious version
Must support:
- one modular host editor workspace
- multiple hosted panels/tool surfaces in split composition
- stable panel/tool-surface identity
- focus routing and activation
- panel-local and workspace-local state retention hooks

## Later breadth
Should support:
- tabs
- draggable tabs and tab reorder/move between hosts
- docking
- floating windows
- saved workspaces/layouts
- detachable tool arrangements

### Workspace compatibility rule
Any first-class tool surface must survive:
- resize
- relayout
- hide/show
- activation changes
- later movement between hosts or tab stacks

without changing its semantic ownership model.

### Principle
Docking and tabs are architecturally first-class workspace concerns. Architect for them now, but do not block the first framework version on full workspace productization.

---

## Command Layers

The architecture should distinguish command layers rather than collapsing everything into one undifferentiated command stream.

### 1. Local interaction actions
Transient control/surface-level actions.
Examples:
- drag started
- marquee updated
- hover changed
- scrub preview advanced

### 2. Workspace commands
Commands that affect composition or workspace behavior.
Examples:
- activate panel
- split host
- move tab later
- activate next tab

### 3. Tool/domain commands
Commands that express intended tool/domain operations.
Examples:
- connect graph pins
- move keyframe
- change viewport product
- apply inspector edit

### 4. Ratified mutations
The committed effects of accepted commands.
Examples:
- graph changed
- property committed
- timeline edit recorded
- workspace/tool state committed where applicable

### Principle
Do not let the shell/workspace become a universal semantic command owner merely because it routes top-level interactions.

---

## Tool-Surface Pattern

Every first-class tool surface should follow the same architectural pattern.

## A. Observation-facing model
What the surface is allowed to see.
Examples:
- viewport artifact/product observation
- graph observation frame
- timeline observation frame
- inspector observation frame

## B. Presentation state
How the surface is configured for display.
Examples:
- selected viewport product
- visible overlay toggles
- graph layer visibility
- timeline zoom mode
- active color mode

## C. Interaction/session state
Transient local interaction state.
Examples:
- active drag
- current marquee
- hovered edge/node/control point
- orbit/pan/zoom in progress
- temporary probe state
- scrub/playhead preview state

## D. Command boundary
What intended actions the surface emits.
Examples:
- keyframe move requested
- node connect requested
- inspector value apply requested
- viewport selection/product command requested

## E. Ratified mutation boundary
What becomes committed change.
Examples:
- keyframe moved
- node connected
- inspector property changed
- viewport command committed

### Rule
No serious tool surface should collapse these concerns into one mutable widget state blob.

---

## First-Class Tool Surfaces

The framework should explicitly treat these as first-class over time.

## Immediate first-class surfaces
- viewport
- inspector surface
- outliner/tree surface
- diagnostics/console/log surfaces

## Near-term first-class surfaces
- graph editor
- timeline
- curve editor
- color tool/editor
- asset/atlas/texture viewer

## Later specialized surfaces
- field/volume viewer
- procgen editor surfaces
- SDF modeling surfaces
- animation authoring workspaces
- specialized debug and analysis surfaces

### Note on controls
The inspector is a tool surface, but many of its form and control primitives should live in the shared substrate.

The key requirement is that new surfaces extend the shared tool-surface framework rather than inventing new local interaction models from scratch.

---

## Input, Focus, and Interaction Model

The framework should use one shared interaction model across editor surfaces and reusable UI.

Must support:
- pointer events
- keyboard events
- text input where applicable
- focus ownership
- pointer capture
- propagation/stop behavior
- repaint/relayout signaling
- command routing from hosted tool surfaces

### Interaction bias
The framework should be balanced keyboard + pointer, with strong support for power-user editor workflows.

### Future-compatible but not immediate drivers
Leave room for:
- gamepad
- pen/tablet
- touch
- accessibility navigation

### Principle
Tool surfaces must integrate with the generic routing/focus/capture model rather than creating permanent side-channel interaction paths.

---

## Observation and Expression Alignment

The editor/UI/workspace framework must align explicitly with the governing doctrine’s observation and expression model.

### Observation alignment
Tool surfaces should consume **declared observation frames** rather than raw authority-internal structures.
Examples:
- inspector observation frame
- outliner observation frame
- graph observation frame
- timeline observation frame
- viewport artifact observation frame

### Expression alignment
Embedded previews, rendered surfaces, and consumer-facing visual products should travel through **expression products** or equivalent expression-frame contracts.
Examples:
- viewport presentation products
- atlas previews
- diagnostics overlays
- field/volume visualizations

### Rule
Session-local interaction state is neither authoritative reality nor a substitute for observation or expression contracts.

---

## Rendering and Presentation Model

## Immediate rendering target
Support:
- ordinary 2D UI primitives
- text
- themed controls
- embedded sampled/rendered surfaces
- layered overlays where needed

## Medium-term rendering target
Extend toward:
- richer canvas/vector-style rendering for graphs/timelines/curves
- more advanced overlay composition
- optional richer transitions/effects later

### Principle
Do not let the renderer become the semantic owner of UI/tool behavior.
Render data and embedded surface contracts should remain explicit outputs of the UI/tooling architecture.

---

## Styling and Theme Model

Use:
- theme tokens
- explicit component variants
- explicit style contracts

Avoid:
- highly implicit cascade as the architectural center

### Why
Runenwerk should prefer explicit, typed, local reasoning over implicit global styling behavior.

### Goal
Support both:
- editor theming
- in-game theming

using one token/variant-oriented styling substrate.

---

## State and Mutation Philosophy

The framework should explicitly distinguish:

## 1. Observed state
What a control or tool surface is reading.

## 2. Local interaction/session state
Transient UI/tool behavior.

## 3. Commands
Intended actions emitted by controls, surfaces, or workspace interactions.

## 4. Ratified mutations
Committed changes to domain/editor/runtime state.

This is mandatory for serious tooling.

### Why
This is the best fit for Runenwerk’s doctrine and is essential for:
- undo/redo
- deterministic editing flows
- persistence and recovery
- migration-ready authored content later
- clean separation between UI behavior and domain truth

---

## Reusable Controls vs Tool Surfaces

### Reusable controls
Should live in the shared substrate when they are broadly useful.
Examples:
- text input
- numeric field
- toggle
- dropdown
- button
- list/tree primitives
- color control primitives later

### Tool surfaces
Should remain distinct from controls.
Examples:
- viewport
- graph editor
- timeline
- curve editor
- atlas viewer

### Rule
Do not reduce complex tool surfaces to oversized controls.
Do not force ordinary controls to depend on heavy tool-surface infrastructure.

---

## Editor Host And Authored Editor Definitions

The main editor is still runtime-composed by code-owned host/editor shell
systems, but authored editor definitions are already live inputs. The current
architecture must therefore preserve both truths:

- code-owned host/editor shell owns runtime composition, provider wiring,
  command execution, app IO, and compatibility-gated activation;
- authored definitions own source-backed UI/layout/theme/menu/shortcut/catalog
  documents that validate, normalize, and form allowed products;
- self-authoring/UI Designer edits those definition families through explicit
  shell/app commands and apply/rollback boundaries;
- packaging/export remains later product breadth, not a reason to move runtime
  state or provider behavior into definition data.

The concrete self-authoring and final editor UI target is defined in
`docs-site/src/content/docs/design/implemented/editor-self-authoring-and-final-ui-design.md`.

---

## Future Editor-Definition Formation Path

Authored editor/workspace/tool definitions must not be treated as automatically
executable merely because they exist.

They follow the same governing formation logic as other authored platform
content.

### Formation path
`Authored -> Normalized -> Formed -> Instantiated`

Possible interpretation:
- **Authored**: editable workspace/tool/editor definitions
- **Normalized**: validated and canonicalized definitions
- **Formed**: runtime-ready host/package/tool composition products
- **Instantiated**: active editor/workspace/tool runtime instances

### Rule

Definition activation must pass through validation, compatibility checks,
formed products, and ratified host/app command paths. The architecture must
avoid collapsing editor-definition formation into opaque host-only code or into
definition-owned runtime execution.

---

## Definition-Ready Seams

The first host architecture should leave explicit future seams for:
- workspace definition
- panel host definition
- tab-stack definition
- tool-surface definition
- command binding definition
- presentation/product binding definition
- theme definition
- packaging/export definition

These seams do not need full authored implementations in the first version, but they must remain conceptually visible and not be collapsed into ad hoc host-editor code paths.

---

## Retention Classes For Workspace State

Workspace and tool-hosting state should not be treated as one undifferentiated persistence bucket.

The framework should distinguish at least conceptually between:

### 1. Ephemeral state
Examples:
- current hover
- temporary drag state
- transient tab preview state
- temporary focus transitions

### 2. Rebuildable state
Examples:
- derived panel measurements
- rebuildable toolbar composition
- rebuildable observation-facing UI forms

### 3. Session-retained state
Examples:
- current workspace arrangement for the active session
- tab activation order
- panel-local UI/session state that should survive relayout during the session

### 4. Saved workspace state
Examples:
- persisted dock layout
- named workspace presets
- saved tool arrangement preferences

### 5. Future authored workspace/tool definitions
Examples:
- authored workspace templates
- authored host compositions
- future editor-defined tool arrangements

### Rule
The first serious framework version does not need full durable workspace authoring, but it must distinguish ephemeral, session-retained, rebuildable, and later durable/authored state clearly enough to avoid architectural drift.

---

## Relationship To In-Game UI

The architecture serves editor, runtime/debug tooling, and in-game UI, but not every layer is equally relevant to all of them.

### Shared by all
- core UI substrate
- common controls where applicable
- theme/token system
- input/focus primitives
- render-data/presentation primitives

### Primarily editor/debug-host oriented
- workspace/shell composition layer
- panel/dock/tab hosting model
- multi-surface host-editor concerns

### Principle
Do not force ordinary in-game HUD/menu UI to depend on editor workspace semantics just because both use the same substrate.

The accepted app-neutral composition design makes this separation explicit: game UI
may share `Region`, `MountedUnit`, `UiComposition`, and proposal-only adaptive
layout contracts, but it must not import editor `Workspace`, `SurfaceSlot`,
`ToolSurfaceInstance`, or editor provider policy.

---

## Extension Policy For New Features

Every new feature should be added by extending the correct seam.

### Ask first
- which layer owns the truth?
- is this a substrate capability, a workspace capability, a new tool surface, or a new capability inside an existing tool surface?
- what is observed vs transient vs commanded vs ratified?
- does it need products/previews?

### Extension rules
- if it is specific to one surface, extend that surface
- if two or more serious surfaces need the same capability, promote it into the shared substrate or shared tool-surface framework
- keep shell concerns in the shell/workspace layer only
- keep domain semantics out of the shell

This is how Runenwerk should grow without endless refactors.

---

## What Should Be Built First

## Phase A — Prove the first serious tool surface architecture
- viewport architecture
- panel-owned embedding
- routed input/focus/capture compliance
- product/presentation boundaries

## Phase B — Strengthen the host editor framework
- workspace identity
- panel identity
- dock/tab/tool identity
- more formal shell/workspace composition
- tool-surface hosting conventions
- reusable toolbar/action/command routing patterns

## Phase C — Broaden reusable surface/canvas support
- shared zoom/pan canvas behavior
- selection/marquee patterns
- overlay/gizmo/probe support
- richer canvas-style rendering support

## Phase D — Add more first-class surfaces
- graph editor
- timeline
- curve editor
- color tools
- specialized viewers

## Phase E — Prepare authored editor-definition groundwork
- explicit definition seams
- persistence/migration planning
- authored-definition domain design later

---

## What Should Not Be Built First

Do not begin with:
- full no-code editor-builder workflows
- standalone editor export/package tooling
- complete docking/floating-window productization
- universal public tool SDK
- generalized abstractions driven only by hypothetical external-tool needs

Those belong later.

---

## Success Criteria For The First Framework Version

The first serious framework version is successful if:
- the viewport is implemented as a real first-class tool surface
- the shell/workspace can host multiple serious panels without owning their semantics
- the workspace identity model is strong enough to support draggable tabs, dock host reassignment, and later floating/detached hosts without redesign
- the UI substrate can serve both editor and runtime-style UI needs
- the framework can support at least one more advanced surface beyond the viewport without a major architecture rewrite
- observation/session/command/ratification boundaries stay explicit
- later authored editor-definition evolution is still possible without discarding the host editor architecture

---

## Rejected Long-Term Mistakes

Avoid these outcomes:
- making the shell the semantic center of the platform
- building multiple independent UI stacks for editor, runtime UI, and debug tools
- keeping every serious surface as a one-off panel implementation
- allowing local UI mutation to bypass explicit mutation boundaries everywhere
- turning future editor-definition ambitions into immediate phase-1 scope
- overgeneralizing for hypothetical external tools before the host/tool-surface model is proven
- leaving docking/tabs as a vague visual feature instead of a real workspace model

---

## Final Decision

Runenwerk should become a **modular host editor and reusable UI/tooling framework first**, built on:
- a reusable UI substrate
- a reusable workspace/shell composition layer
- a reusable tool-surface framework
- explicit identity and dock/tab-hosting concepts
- strict observation / interaction / command / ratification boundaries

It should later evolve into an **editor-construction platform** where the main editor can author workspace, docking/tab-hosting, tool-surface, and editor-definition content.

That is the best long-term path and the one most consistent with the nine-layer architecture.

---

## Short Version

Build now:
- reusable UI substrate
- reusable workspace/tool-surface host framework
- modular host editor
- explicit identity/dock-ready workspace model

Architect now for later:
- authored editor/workspace/tool definitions
- editor-defined editors

Do not skip the host architecture.
Do not let the shell become the whole platform.
Do not delay the reusable framework waiting for a future meta-editor.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:workbench-adoption -->
## Component Platform workbench adoption

Workbench surfaces consume story-proven reusable controls and surfaces. Toolbar uses Button/Toggle/Navigation. Inspector uses Label/Input/NumericInput/Picker. Outliner uses Tree/List. Entity table uses Table/List/Search/Sort. Menus consume Overlay/Menu/Popup contracts. Material graph consumes PortGraphCanvas. Timeline surfaces consume TrackSurface/Timeline. Skill/progression views consume NodeCanvas/ProgressionTreeView where applicable.

Workbench does not own reusable control semantics; it maps host intent proposals to editor/domain commands.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:workbench-adoption -->
