---
title: App-Neutral UI Composition, Adaptive Layout, Inspection, and Story Proof Vocabulary Design
description: App-neutral vocabulary and architecture doctrine for Runenwerk UI composition, mounted units, adaptive/fluid layout, inspection, stories, and app-specific specializations without editor/game/browser lock-in.
status: draft
owner: ui
layer: design
canonical: false
last_reviewed: 2026-06-18
supersedes_drafts:
  - /mnt/data/ui_surface_host_vocabulary_and_inspection_design.md
  - /mnt/data/app_neutral_ui_composition_vocabulary_design.md
related_adrs:
  - docs-site/src/content/docs/adr/accepted/0004-separate-description-from-execution.md
  - docs-site/src/content/docs/adr/accepted/0005-projections-are-derived-state.md
  - docs-site/src/content/docs/adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md
  - docs-site/src/content/docs/adr/accepted/0010-graph-substrate-canvas-boundary.md
related_code:
  - domain/ui/ui_surface/src/definition.rs
  - domain/ui/ui_surface/src/capability.rs
  - domain/ui/ui_surface/src/inspection.rs
  - domain/ui/ui_story/src/proof.rs
  - domain/ui/ui_definition/src/lib.rs
  - domain/ui/ui_program/src/lib.rs
  - domain/editor/editor_shell/src/surface_provider.rs
  - domain/editor/editor_shell/src/workspace/
  - apps/runenwerk_editor/src/shell/providers/
---

# App-Neutral UI Composition, Adaptive Layout, Inspection, and Story Proof Vocabulary Design

## 1. Purpose

Runenwerk needs UI vocabulary that can scale beyond the current editor and beyond games.

The platform should eventually support many application classes:

- editor workbenches;
- browser-like apps with tabs, pages, navigation, detachable windows, and popups;
- drawing/design apps with canvases, palettes, overlays, inspectors, and fluid panels;
- dashboards with cards, panels, responsive grids, modals, and drill-down views;
- terminal-like apps with tabs, panes, sessions, command palettes, and split views;
- mobile/touch-first apps with screens, navigation stacks, sheets, drawers, and adaptive layouts;
- games with HUDs, menus, world-space prompts, overlays, and debug panels;
- headless validation/proof hosts with no visible native window;
- embedded UI targets, offscreen render targets, and external preview processes.

Earlier vocabulary centered on this hierarchy:

```text
NativeWindow
  └─ Host
      └─ Workspace
          └─ SurfaceSlot
              └─ SurfaceInstance
                  ├─ SurfaceProvider
                  ├─ SurfaceSession
                  └─ SurfaceContent
```

That shape is useful for the current editor, but it is too editor/workbench-shaped as a universal base model. This document replaces it with an app-neutral hierarchy and treats editor, browser, game, draw, dashboard, terminal, and headless validation as specializations.

## 2. Critical Review Summary

The current direction is architecturally strong but vocabularily under-specified.

What should stay:

- authored UI source, semantic program, runtime artifact, runtime view, and render output remain separate;
- stories remain proof scenarios, not components or runtime owners;
- domains own meaning;
- apps own IO, host policy, lifecycle, and concrete wiring;
- render output remains derived;
- provider seams remain app/workbench-owned;
- generic UI composition should provide reusable mechanisms, not app authority.

What must improve:

- `Workspace` is not universal enough;
- `Surface` is overloaded;
- `NativeWindow` is too narrow;
- `SurfaceProvider` is too editor-provider-shaped for base vocabulary;
- tabs, docks, splits, overlays, and popups must not be locked in `editor_shell` vocabulary;
- fluid layout needs first-class terms for dragging, snapping, detaching, resizing, floating, and responsive reflow;
- SOLID guardrails must be explicit so the model does not turn into a god abstraction.

## 3. Core Doctrine

### 3.1 App-neutral base, app-specific aliases

The base vocabulary must not speak editor/game/browser language.

Base vocabulary:

```text
PresentationTarget
AppHost
CompositionRoot
RegionGraph
Region
MountedUnit
UnitProvider
UnitSession
UnitContent
UiControl
UiComposition
AdaptiveCompositionRuntime
InspectableTarget
StoryProof
```

Editor vocabulary is a specialization:

```text
NativeWindowTarget
  └─ EditorShell
      └─ Workspace
          └─ SurfaceSlot
              └─ ToolSurfaceInstance
```

Browser vocabulary is a specialization:

```text
NativeWindowTarget
  └─ BrowserHost
      └─ BrowserShell
          └─ PageRegion
              └─ BrowserPageInstance
```

Game vocabulary is a specialization:

```text
GameViewportTarget
  └─ GameUiHost
      └─ GameScreen / HudRoot
          └─ HudRegion / WorldAnchorRegion
              └─ HudInstance / PromptInstance
```

### 3.2 Mechanism is not authority

Generic UI owns reusable mechanisms:

- buttons;
- labels;
- inputs;
- tabs;
- popups;
- scroll areas;
- split layouts;
- dock affordances;
- overlay layers;
- navigation stacks;
- canvas viewport mechanics;
- inspector/property layout patterns;
- drag, snap, detach, resize, and reflow proposals.

App hosts/domains own meaning:

- browser navigation;
- editor workspace mutation;
- drawing document mutation;
- game inventory actions;
- graph node semantics;
- entity selection;
- asset import policy;
- story proof acceptance;
- external process lifecycle.

A generic tab strip can show tabs, reorder tabs, and emit tab intents. It must not own browser history, editor workspace persistence, game inventory semantics, or document save state.

### 3.3 Domains own meaning; platform owns neutral structure

The platform may define neutral structure:

- ids;
- profiles;
- capabilities;
- diagnostics;
- source maps;
- mount envelopes;
- proof envelopes;
- layout proposals;
- region graphs;
- route descriptors;
- presentation target descriptors.

The owning domain/app defines semantic meaning:

- what a page is;
- what a document is;
- what a graph node means;
- what a game entity means;
- what a material, texture, field product, drawing stroke, or world prompt means;
- whether an adaptive layout proposal is allowed.

### 3.4 Source, program, runtime, render, and proof remain separate

For UI content:

```text
AuthoredUiTemplate
  └─ UiNodeDefinition
      └─ UiProgram
          └─ UiRuntimeArtifact
              └─ UiRuntimeView
                  └─ UiFrame / UiPrimitive
```

For story proof:

```text
Story
  └─ proves Source / Program / Artifact / RuntimeView / Render / Mount / MountedUnit scenario
```

For inspection:

```text
Inspector
  └─ observes InspectableTarget and SourceMapChain
      └─ never owns target truth
```

## 4. Canonical App-Neutral Model

The canonical base hierarchy is:

```text
PresentationTarget
  └─ AppHost
      └─ CompositionRoot
          ├─ RegionGraph
          │   └─ Region
          │       └─ MountedUnitIdentity
          │
          ├─ UnitProviderRegistry
          ├─ UnitSessionStore
          ├─ UnitContentRegistry
          ├─ UiComposition
          └─ AdaptiveCompositionRuntime
```

A more readable association view is:

```text
PresentationTarget
  └─ AppHost
      └─ CompositionRoot
          ├─ RegionGraph
          │   └─ Region
          │       └─ MountedUnit
          │           ├─ UnitProvider
          │           ├─ UnitSession
          │           └─ UnitContent
          │               ├─ UiContent
          │               ├─ CanvasContent
          │               ├─ DocumentContent
          │               ├─ InspectorContent
          │               ├─ ConsoleContent
          │               ├─ ProductPreviewContent
          │               ├─ ExternalContent
          │               └─ HeadlessContent
          │
          ├─ UiComposition
          │   ├─ Tabs
          │   ├─ Split
          │   ├─ Dock
          │   ├─ Popup
          │   ├─ Scroll
          │   ├─ Overlay
          │   ├─ NavigationStack
          │   ├─ CommandPalette
          │   ├─ InspectorLayout
          │   └─ CanvasViewport
          │
          └─ AdaptiveCompositionRuntime
              ├─ DragSession
              ├─ ResizeSession
              ├─ PlacementProposal
              ├─ SnapZone
              ├─ DropPreview
              ├─ FloatingRegion
              ├─ ResponsiveRule
              └─ LayoutTransaction
```

Important: this is a conceptual association model, not a mandate to implement one giant nested object.

## 5. Vocabulary Definitions

### 5.1 PresentationTarget

A `PresentationTarget` is where output and input happen.

Examples:

- native OS window;
- embedded panel;
- web canvas;
- game viewport;
- VR/XR view;
- offscreen render target;
- headless validation target;
- external preview process target.

`NativeWindow` is one specialization, not the base concept.

Preferred terms:

```text
PresentationTarget
NativeWindowTarget
EmbeddedTarget
GameViewportTarget
WebCanvasTarget
OffscreenTarget
HeadlessTarget
ExternalPreviewTarget
```

### 5.2 AppHost

An `AppHost` owns application lifecycle and policy for a composition tree.

Responsibilities:

- lifecycle;
- input ingress;
- focus policy;
- mounted unit lifecycle;
- host capability policy;
- trust policy;
- app-level command routing;
- acceptance/rejection of adaptive layout proposals.

Specializations:

```text
EditorShell
BrowserHost
GameUiHost
DrawHost
DashboardHost
TerminalHost
MobileAppHost
HeadlessStoryHost
```

An `AppHost` must not become a god object. Concrete implementations should split lifecycle, routing, policy, session storage, and composition state internally.

### 5.3 CompositionRoot

A `CompositionRoot` is the app-level visual and interactive structure managed by an `AppHost`.

Specializations:

```text
Workspace
BrowserShell
GameScreen
HudRoot
DocumentShell
DashboardRoot
TerminalPaneRoot
NavigationStackRoot
ValidationRunRoot
```

`Workspace` is an editor/workbench specialization, not the universal base term.

### 5.4 RegionGraph

A `RegionGraph` is the structural topology of regions inside a composition root.

It owns layout topology, not application meaning.

Examples:

- split tree;
- tab stack tree;
- dashboard grid;
- screen stack;
- page/chrome layout;
- overlay/modal layer tree;
- HUD anchor graph;
- terminal pane graph.

### 5.5 Region

A `Region` is a place where content can appear, be mounted, or be overlaid.

Specializations:

```text
DockRegion
TabRegion
PageRegion
PanelRegion
OverlayRegion
ModalRegion
HudRegion
WorldAnchorRegion
CanvasOverlayRegion
ToolbarRegion
SidebarRegion
SheetRegion
DrawerRegion
```

Editor `SurfaceSlot` is a specialization of `Region`.

### 5.6 MountedUnit

A `MountedUnit` is one mounted thing with identity and lifecycle.

Examples:

```text
ToolSurfaceInstance
BrowserPageInstance
DocumentViewInstance
HudInstance
PromptInstance
InspectorInstance
ConsoleInstance
PreviewInstance
TerminalSessionInstance
DashboardCardInstance
```

A `MountedUnit` must have a narrow base contract:

- identity;
- lifecycle;
- capabilities;
- content reference;
- diagnostics;
- optional local session reference;
- optional provider/controller reference.

A `MountedUnit` must not require editor workspace identity, native windows, visual rendering, docking, browser navigation, game entities, or command execution.

### 5.7 UnitProvider

A `UnitProvider` produces observations, presentation data, and local route proposals for a mounted unit.

It should not become a blob that owns everything.

Conceptual sub-responsibilities:

```text
ObservationProducer
PresentationProducer
RouteAdapter
LifecycleAdapter
CapabilityAdapter
```

The provider may map local interaction intents to typed command proposals, but concrete command execution remains app/domain-owned.

### 5.8 UnitSession

A `UnitSession` stores local per-mounted-unit state.

Examples:

- scroll offset;
- focused local control;
- expanded inspector sections;
- current page zoom;
- selected local tab;
- local filter text;
- transient drag state;
- local preview settings.

A `UnitSession` must not become domain truth. A drawing document, scene document, graph document, browser history, or game state belongs to its owning app/domain.

### 5.9 UnitContent

`UnitContent` describes what kind of content a mounted unit contains.

Content kinds:

```text
UiContent
CanvasContent
DocumentContent
InspectorContent
ConsoleContent
ProductPreviewContent
ExternalContent
HeadlessContent
```

A content kind is not necessarily a UI program. Some content is a canvas, external process preview, terminal session, render product preview, or headless proof record.

### 5.10 UiContent

`UiContent` is content authored or formed through the UI pipeline.

Canonical chain:

```text
AuthoredUiTemplate
  └─ UiNodeDefinition
      └─ UiProgram
          └─ UiRuntimeArtifact
              └─ UiRuntimeView
                  └─ UiFrame / UiPrimitive
```

`UiContent` may contain controls and compositions.

### 5.11 UiControl

A `UiControl` is a reusable semantic UI unit.

Examples:

```text
Button
Label
TextInput
NumericInput
Select
Toggle
Checkbox
Radio
Slider
Tree
Table
List
Breadcrumb
ToolbarButton
```

Use `Control` for public semantic vocabulary.

Use `Widget` only for retained runtime compatibility or concrete implementation vocabulary.

### 5.12 UiComposition

`UiComposition` owns reusable UI structure and interaction mechanisms.

Examples:

```text
Tabs
Split
Dock
Popup
Scroll
Overlay
NavigationStack
CommandPalette
InspectorLayout
CanvasViewport
PropertyGrid
ResponsiveGrid
Drawer
Sheet
Accordion
Toolbar
MenuBar
```

`UiComposition` is UI stuff. It belongs in the UI platform vocabulary.

It does not own the semantic state behind the UI.

Examples:

- `Tabs` can render and interact with tab items;
- browser host owns page sessions/history;
- editor host owns workspace/surface persistence;
- game host owns menu/inventory state.

### 5.13 AdaptiveCompositionRuntime

`AdaptiveCompositionRuntime` owns generic fluid layout mechanics.

It covers modern layout behavior:

- dragging tabs/panels/units;
- docking;
- snapping to sides/edges;
- detaching into floating regions or new presentation targets;
- resizing regions;
- responsive reflow;
- collapse/overflow rules;
- ghost previews;
- placement candidates;
- drop previews;
- accepted layout transactions.

It emits proposals. It does not mutate app/domain truth by itself.

Concepts:

```text
DragSession
ResizeSession
PlacementProposal
SnapZone
DropPreview
FloatingRegion
ResponsiveRule
LayoutTransaction
```

### 5.14 CanvasContent

`CanvasContent` is content with coordinate space, viewport, hit testing, tools, overlays, and domain-specific selection/interaction.

Specializations:

```text
GraphCanvasContent
ViewportCanvasContent
DrawingCanvasContent
StoryPreviewCanvasContent
MaterialPreviewCanvasContent
WorldEditorCanvasContent
```

Canvas mechanics can be generic. Canvas truth is domain-owned.

### 5.15 InspectorContent

`InspectorContent` renders information about one or more `InspectableTarget`s.

The inspector observes targets. It does not own target truth.

### 5.16 ProductPreviewContent

`ProductPreviewContent` displays formed/renderable products or product descriptors.

Examples:

- texture preview;
- material preview;
- field preview;
- render target preview;
- static mount preview;
- external runtime preview frame.

Renderer output remains derived state.

### 5.17 ExternalContent

`ExternalContent` represents content produced by a process, adapter, embedded runtime, or external system.

Examples:

- browser page process;
- runtime preview child process;
- embedded document viewer;
- remote session;
- plugin sandbox output.

### 5.18 HeadlessContent

`HeadlessContent` represents validation/proof/execution data with no visible output requirement.

Examples:

- story proof run;
- manifest validation;
- package compatibility check;
- layout proposal validation;
- diagnostics-only output.

### 5.19 Story

A `Story` is a proof scenario.

It may prove:

- source loading;
- source parsing;
- definition validation;
- program formation;
- compiler/artifact formation;
- runtime view formation;
- render primitive formation;
- render data formation;
- static mount;
- preview frame;
- mount eligibility;
- app/surface scenario behavior.

A story is not a component, surface, mounted unit, runtime artifact, render frame, or app.

### 5.20 InspectableTarget

An `InspectableTarget` is a typed reference to something selected, hovered, focused, diagnosed, or inspected.

Examples:

```text
MountedUnitTarget
RegionTarget
StoryTarget
UiNodeTarget
ProgramNodeTarget
RuntimeArtifactTarget
RuntimeViewTarget
RenderPrimitiveTarget
GraphNodeTarget
DocumentTarget
AssetTarget
DiagnosticTarget
ExternalTarget
```

It should be represented as stable target refs plus owner/domain metadata, not direct imports of every domain type.

### 5.21 SourceMapChain

A `SourceMapChain` connects derived output back to source.

Example chain:

```text
RenderPrimitive
  → RuntimeView node
  → UiProgram node
  → UiRuntimeArtifact source map
  → UiNodeDefinition path
  → authored source file
  → story manifest
```

Inspectors consume source map chains. They do not own source truth.

## 6. Fluid Layout and Modern App Behavior

Modern apps need fluid composition:

- browser tab dragging;
- tab detaching into new windows;
- side snapping;
- edge tiling;
- split view creation;
- drag-to-dock;
- floating palettes;
- adaptive sidebars;
- responsive collapse;
- modal and sheet transitions;
- mobile navigation transitions;
- drag previews and ghost regions.

This requires a split between resting structure and transition mechanics.

### 6.1 Resting structure

The resting structure is represented by:

```text
CompositionRoot
  └─ RegionGraph
      └─ Region
          └─ MountedUnit
```

### 6.2 Interactive transition

The transition is represented by:

```text
AdaptiveCompositionRuntime
  ├─ DragSession
  ├─ ResizeSession
  ├─ PlacementProposal
  ├─ SnapZone
  ├─ DropPreview
  ├─ FloatingRegion
  ├─ ResponsiveRule
  └─ LayoutTransaction
```

### 6.3 Proposal flow

```text
Pointer / keyboard / gesture input
  → UiComposition hit testing
  → AdaptiveCompositionRuntime session
  → PlacementProposal
  → AppHost policy validation
  → domain/app command or layout mutation proposal
  → ratification
  → LayoutTransaction
  → new RegionGraph
  → derived frame output
```

### 6.4 Mechanism-policy split

Generic mechanism:

```text
start drag
capture pointer
find candidate regions
calculate snap zones
project drop preview
calculate proposed layout graph
emit placement proposal
```

App policy:

```text
is this move legal?
does this unit support this region?
does it detach, dock, tab, float, or create a new target?
does it persist?
which command handles it?
which capability gates it?
```

## 7. Browser-Like App Example

A browser-like app uses the same generic vocabulary without becoming an editor.

```text
PresentationTarget = NativeWindowTarget
AppHost = BrowserHost
CompositionRoot = BrowserShell
RegionGraph = BrowserChrome + PageRegion + OverlayRegion
Region = TabStripRegion / AddressRegion / PageRegion / OverlayRegion
MountedUnit = BrowserPageInstance
UnitSession = BrowserPageSession
UnitContent = WebPageContent / UiContent / ExternalContent
UiComposition = Tabs / Popup / Scroll / Overlay
AdaptiveCompositionRuntime = tab drag / detach / snap / reflow
```

Tabs are generic UI composition.

Browser-owned meaning:

- URLs;
- origin/security policy;
- navigation history;
- reload;
- downloads;
- bookmarks;
- page lifecycle;
- session restore.

Generic UI-owned mechanism:

- tab strip layout;
- active tab visual state;
- tab close affordance;
- tab drag intent;
- overflow behavior;
- address text input;
- popup menus;
- scroll areas;
- overlay layers.

Example event flow:

```text
Tabs emits TabCloseRequested(page_id)
  → BrowserHost validates close policy
  → BrowserSession closes/suspends page
  → RegionGraph removes or replaces BrowserPageInstance
  → UiComposition renders updated tabs
```

## 8. Editor/Workbench Example

The current editor maps to the generic model:

```text
PresentationTarget = NativeWindowTarget
AppHost = EditorShell
CompositionRoot = Workspace
RegionGraph = workspace split/tab graph
Region = SurfaceSlot / DockRegion / TabRegion / OverlayRegion
MountedUnit = ToolSurfaceInstance
UnitProvider = EditorSurfaceProvider
UnitSession = ToolSurfaceInstanceId-scoped surface session
UnitContent = UiContent / CanvasContent / InspectorContent / ConsoleContent / ProductPreviewContent
UiComposition = Tabs / Split / Dock / Popup / Scroll / Overlay / CanvasViewport
AdaptiveCompositionRuntime = tab drag / split resize / dock proposal / preview
```

Editor-owned meaning:

- workspace persistence;
- tool surface compatibility;
- provider registry composition;
- command routing;
- document compatibility;
- editor domain commands;
- project IO.

Generic UI-owned mechanism:

- tab strip;
- split layout;
- dock affordance;
- popup stack;
- scroll/focus;
- overlay layers;
- drag/drop previews.

## 9. Game UI Example

```text
PresentationTarget = GameViewportTarget
AppHost = GameUiHost
CompositionRoot = GameScreen / HudRoot
RegionGraph = screen layer graph / world anchor graph
Region = HudRegion / ModalRegion / OverlayRegion / WorldAnchorRegion
MountedUnit = HudInstance / PauseMenuInstance / PromptInstance / NameplateInstance
UnitContent = UiContent / CanvasContent / ProductPreviewContent
UiComposition = Tabs / Popup / Scroll / Overlay / NavigationStack
AdaptiveCompositionRuntime = responsive HUD placement / movable debug panels / overlay docking
```

Game-owned meaning:

- game state;
- entity anchors;
- world-space visibility;
- culling/occlusion;
- input mode;
- multiplayer authority;
- gameplay commands.

Generic UI-owned mechanism:

- layout;
- controls;
- overlays;
- tabs;
- prompts;
- popups;
- frame output.

## 10. Drawing App Example

```text
PresentationTarget = NativeWindowTarget
AppHost = DrawHost
CompositionRoot = DrawingDocumentShell
RegionGraph = canvas + palette + layer panel + overlay regions
Region = CanvasRegion / ToolbarRegion / PaletteRegion / OverlayRegion
MountedUnit = DrawingCanvasInstance / ToolPaletteInstance / LayerPanelInstance
UnitContent = CanvasContent / UiContent / ProductPreviewContent
UiComposition = Tabs / Popup / Overlay / Scroll / CanvasViewport
AdaptiveCompositionRuntime = floating palettes / snap sidebars / responsive reflow
```

Drawing-owned meaning:

- drawing document truth;
- stroke commands;
- tile products;
- brush semantics;
- layer semantics;
- export/package policy.

Generic UI-owned mechanism:

- controls;
- overlays;
- panels;
- canvas viewport mechanics;
- floating palette drag/snap behavior.

## 11. Dashboard Example

```text
PresentationTarget = NativeWindowTarget / WebCanvasTarget
AppHost = DashboardHost
CompositionRoot = DashboardRoot
RegionGraph = responsive grid / card layout
Region = CardRegion / ModalRegion / FilterRegion / OverlayRegion
MountedUnit = DashboardCardInstance / FilterPanelInstance / ChartInstance
UnitContent = UiContent / ProductPreviewContent / ExternalContent
UiComposition = ResponsiveGrid / Tabs / Popup / Scroll / Overlay
AdaptiveCompositionRuntime = resize cards / reorder cards / breakpoint reflow
```

Dashboard-owned meaning:

- data sources;
- queries;
- permissions;
- drill-down navigation;
- chart semantics;
- refresh policy.

## 12. Terminal App Example

```text
PresentationTarget = NativeWindowTarget
AppHost = TerminalHost
CompositionRoot = TerminalPaneRoot
RegionGraph = tab/pane split graph
Region = PaneRegion / TabRegion / OverlayRegion
MountedUnit = TerminalSessionInstance
UnitContent = ExternalContent / TextBufferContent / UiContent
UiComposition = Tabs / Split / Popup / Scroll / CommandPalette
AdaptiveCompositionRuntime = split panes / tab drag / detach session
```

Terminal-owned meaning:

- shell process;
- PTY lifecycle;
- command history;
- environment;
- session restore.

## 13. Headless Validation Example

```text
PresentationTarget = HeadlessTarget
AppHost = HeadlessStoryHost
CompositionRoot = ValidationRunRoot
RegionGraph = synthetic or absent
MountedUnit = StoryScenarioInstance
UnitContent = HeadlessContent
Story = proof scenario
```

Headless validation does not require native windows, visual render output, docking, or pointer input.

## 14. Surface Vocabulary Reclassification

The word `surface` remains useful but must be narrowed.

Preferred interpretation:

```text
Surface = one kind of MountedUnit specialization.
```

Editor terms:

```text
ToolSurface = editor/workbench specialization of MountedUnit.
SurfaceSlot = editor/workbench specialization of Region.
SurfaceProvider = editor/workbench specialization of UnitProvider.
```

Render/product terms:

```text
ProductSurfacePrimitive = render/product projection.
```

Do not confuse product/render surfaces with workspace-mounted surfaces.

Bad:

```text
Viewport product surface and editor viewport surface are both just Surface.
```

Good:

```text
ViewportSurfaceInstance is a mounted unit.
ProductSurfacePrimitive is derived render/product output displayed inside it.
```

## 15. SOLID Guardrails

### 15.1 Single Responsibility Principle

Each vocabulary object must have one reason to change.

```text
PresentationTarget
  changes when target classes change.

AppHost
  changes when app lifecycle/policy changes.

CompositionRoot
  changes when top-level composition model changes.

RegionGraph
  changes when structural topology rules change.

Region
  changes when placement/containment semantics change.

MountedUnit
  changes when mounted identity/lifecycle/capability contract changes.

UiComposition
  changes when reusable UI mechanisms change.

AdaptiveCompositionRuntime
  changes when drag/snap/dock/reflow mechanics change.
```

`UnitProvider` must not absorb session storage, command execution, domain truth, lifecycle policy, rendering, diagnostics, and app IO into one blob.

### 15.2 Open/Closed Principle

New app types and mounted unit types should not require central enum edits.

Prefer:

- ids;
- registries;
- profiles;
- capability declarations;
- compatibility records;
- diagnostic contracts.

Avoid closed universal enums like:

```text
AppHostKind::Editor | Game | Browser | Draw | Terminal | Dashboard
```

Use open profiles instead:

```text
HostProfileId("editor.workbench")
HostProfileId("browser.host")
HostProfileId("game.ui")
HostProfileId("headless.story")
```

Built-in known constants are acceptable. The extension point must not require central semantic ownership.

### 15.3 Liskov Substitution Principle

Specializations must satisfy only the base contract.

A `BrowserPageInstance`, `ToolSurfaceInstance`, `HudInstance`, `PromptInstance`, `DrawingCanvasInstance`, and `StoryScenarioInstance` can all be `MountedUnit`s only if `MountedUnit` stays narrow.

A base `MountedUnit` must not require:

- workspace tab identity;
- dock behavior;
- visual rendering;
- native window;
- browser history;
- game entity anchors;
- editor commands;
- pointer input.

### 15.4 Interface Segregation Principle

Mounted units expose only the capabilities they support.

Capabilities may include:

```text
Renderable
Focusable
Inspectable
Navigable
Dockable
Detachable
Resizable
StoryProven
ExternalContent
CanvasInteraction
HeadlessOnly
WorldAnchored
```

A headless proof unit should not depend on render/dock/window interfaces.
A game HUD should not depend on editor workspace interfaces.
A simple mobile screen should not depend on docking or inspector contracts.

### 15.5 Dependency Inversion Principle

High-level app policy depends on abstract contracts, not concrete low-level widgets.

Good:

```text
BrowserHost consumes Tabs contract.
EditorShell consumes Dock/Split/Tab composition contracts.
GameUiHost consumes UiComposition contracts.
Inspector consumes InspectableTarget contracts.
UiStory consumes proof evidence contracts.
```

Bad:

```text
BrowserHost imports editor workspace tabs.
Game HUD imports editor_shell for layout.
Inspector imports every domain type directly.
UiStory owns renderer internals.
EditorShell owns generic tab mechanics forever.
```

## 16. Anti-Patterns

### 16.1 Everything is a UI element

Wrong:

```text
Shell is a UI element.
Canvas is a UI element.
Gallery is a UI element.
HUD is a UI element.
```

Better:

```text
Shell is an AppHost specialization.
Canvas is CanvasContent or CanvasSurface specialization.
Gallery is a mounted unit / story gallery specialization.
HUD is a mounted unit under GameUiHost.
Buttons and labels are UI controls.
```

### 16.2 Editor surface as universal base

Wrong:

```text
All apps use Workspace → SurfaceSlot → SurfaceInstance.
```

Better:

```text
All apps use CompositionRoot → Region → MountedUnit.
Editor uses Workspace → SurfaceSlot → ToolSurfaceInstance as a specialization.
```

### 16.3 UI composition owns app meaning

Wrong:

```text
Tabs close browser pages.
Dock moves editor surfaces.
Popup executes commands.
Inspector mutates selected target.
```

Better:

```text
Tabs emit intents.
Dock emits placement proposals.
Popup emits route intents.
Inspector emits edit proposals.
App/domain validates and executes.
```

### 16.4 Story owns pipeline semantics

Wrong:

```text
Story owns UI compiler/runtime/render pipeline.
```

Better:

```text
Story declares proof requirements and verifies evidence emitted by owning stages.
```

### 16.5 Adaptive layout mutates truth directly

Wrong:

```text
DragSession directly rewrites workspace, browser, game, or drawing state.
```

Better:

```text
DragSession emits PlacementProposal.
AppHost validates and produces a domain/app command or layout transaction.
```

### 16.6 Inspector imports every domain

Wrong:

```text
Inspector knows concrete ECS, graph, browser, drawing, asset, story, material, and UI structs.
```

Better:

```text
Inspector consumes InspectableTarget refs and asks owning adapters/providers for presentation details.
```

## 17. Naming Rules

### 17.1 Preferred base terms

```text
PresentationTarget
AppHost
CompositionRoot
RegionGraph
Region
MountedUnit
UnitProvider
UnitSession
UnitContent
UiControl
UiComposition
AdaptiveCompositionRuntime
InspectableTarget
SourceMapChain
StoryProof
```

### 17.2 Allowed specialization terms

```text
NativeWindowTarget
EditorShell
Workspace
SurfaceSlot
ToolSurfaceInstance
BrowserHost
BrowserShell
BrowserPageInstance
GameUiHost
HudRoot
HudInstance
WorldAnchorRegion
DrawingCanvasInstance
TerminalSessionInstance
DashboardCardInstance
HeadlessStoryHost
```

### 17.3 Terms to use carefully

```text
Surface
  Use as mounted unit specialization, not universal base and not render product.

Widget
  Use for retained implementation detail, not public semantic control vocabulary.

Canvas
  Use for coordinate-space content/profile, not normal button-like UI control.

Window
  Use for native or presentation target only, not generic panel/region.
```

### 17.4 Terms to avoid as base concepts

```text
Editor
Game
Workspace
ToolSurface
Viewport
Entity
StoryGallery
HUD
BrowserPage
```

These are valid specializations but not generic base vocabulary.

## 18. Relationship to Current Runenwerk Crates

This document does not demand immediate crate creation.

Current likely mappings:

```text
domain/ui/ui_controls
  UiControl semantics.

domain/ui/ui_definition
  AuthoredUiTemplate, UiNodeDefinition, validation, normalization, source maps.

domain/ui/ui_program
  UiProgram semantic program contracts.

domain/ui/ui_compiler
  UI compiler/package/capability validation.

domain/ui/ui_artifacts
  UiRuntimeArtifact, runtime tables, artifact source maps.

domain/ui/ui_runtime_view
  runtime read model.

domain/ui/ui_render_primitives and ui_render_data
  derived render primitive/frame/product-surface output.

domain/ui/ui_story
  story proof contracts and evidence verification.

domain/ui/ui_surface
  current seed for mounted-unit/surface/capability/inspection vocabulary.

domain/editor/editor_shell
  editor specialization: Workspace, SurfaceSlot, ToolSurfaceInstance, provider seam.

apps/runenwerk_editor
  concrete editor host, provider registry composition, IO, app commands.
```

Potential future conceptual crates, only after repeated proof:

```text
domain/ui/ui_composition
  Tabs, Split, Dock, Popup, Scroll, Overlay, NavigationStack, InspectorLayout, CanvasViewport contracts.

domain/ui/ui_adaptive_composition
  DragSession, PlacementProposal, SnapZone, DropPreview, ResizeSession, ResponsiveRule, LayoutTransaction contracts.
```

Do not create these crates prematurely. First prove the vocabulary in docs and through at least two consumers.

## 19. Suggested Implementation Path

### Phase 0: Documentation only

Add this document as:

```text
docs-site/src/content/docs/design/active/app-neutral-ui-composition-vocabulary-design.md
```

No behavior changes.

### Phase 1: Vocabulary alignment in docs

Update docs to prefer:

```text
PresentationTarget → AppHost → CompositionRoot → Region → MountedUnit
```

Use editor aliases only when discussing editor code.

### Phase 2: Strengthen `ui_surface` neutrally

Candidate files:

```text
domain/ui/ui_surface/src/definition.rs
domain/ui/ui_surface/src/capability.rs
domain/ui/ui_surface/src/inspection.rs
```

Add neutral vocabulary only:

```text
SurfaceClass / MountedUnitClass
HostProfileId
RegionClassId
CapabilitySet
InspectionProfile
```

Do not add editor/game/browser semantics.

### Phase 3: Document editor specialization mapping

Candidate files:

```text
docs-site/src/content/docs/domain/editor/editor-shell/app-neutral-mapping.md
domain/editor/editor_shell/src/surface_provider.rs
```

Clarify:

```text
Workspace = CompositionRoot specialization.
SurfaceSlot = Region specialization.
ToolSurfaceInstance = MountedUnit specialization.
EditorSurfaceProvider = UnitProvider specialization.
```

### Phase 4: Extract reusable UI composition only after repetition

Extract only after two or more consumers need the same mechanism.

Candidate examples:

- editor workspace tabs + browser-like page tabs;
- editor split panes + terminal split panes;
- editor popups + game/menu popups;
- editor inspector layout + story inspector layout;
- editor canvas viewport + drawing canvas viewport.

### Phase 5: Story proof integration

Allow stories to prove:

```text
control scenario
composition scenario
adaptive layout scenario
mounted unit scenario
surface/tool scenario
headless proof scenario
```

But story remains proof-only.

## 20. Acceptance Criteria

This design is accepted when future docs can describe all of the following without editor/game lock-in:

- full editor workbench with docked tool surfaces;
- browser-like app with tabs, page sessions, navigation, and detachable windows;
- drawing app with canvas, palettes, overlays, and inspector;
- dashboard with responsive cards and modals;
- terminal app with panes/tabs/sessions;
- mobile-style navigation stack;
- game HUD and world-space prompt;
- headless story validation host.

The design must also preserve these rules:

- no generic UI crate owns editor commands;
- no generic UI crate owns game ECS/entity semantics;
- no story crate owns compiler/runtime/render semantics;
- no inspector imports every domain directly;
- no adaptive layout runtime mutates domain truth directly;
- no central enum must be edited for every new app type;
- render output remains derived;
- authored source, semantic program, runtime artifact, runtime view, and render output remain distinct.

## 21. Handoff Prompt

```text
Review and integrate the app-neutral UI composition vocabulary design.

Target document path:
docs-site/src/content/docs/design/active/app-neutral-ui-composition-vocabulary-design.md

Preserve the core doctrine:
- PresentationTarget, AppHost, CompositionRoot, Region, MountedUnit are the app-neutral base.
- NativeWindow, EditorShell, Workspace, SurfaceSlot, ToolSurfaceInstance, GameUiHost, HudRoot, BrowserShell, and BrowserPageInstance are specializations.
- UiComposition owns reusable UI mechanisms such as tabs, splits, docks, popups, scroll, overlays, navigation stacks, inspector layouts, and canvas viewport mechanics.
- AdaptiveCompositionRuntime owns fluid layout mechanisms such as drag sessions, snap zones, placement proposals, drop previews, floating regions, responsive rules, and layout transactions.
- AppHost/domain owners decide policy, command execution, persistence, and semantic meaning.
- Stories prove scenarios; they do not own compiler/runtime/render semantics.
- Inspectors observe InspectableTarget refs and source-map chains; they do not own target truth.
- Use SOLID guardrails: narrow MountedUnit, split provider responsibilities, prefer registries/profiles/capabilities over central enums, and keep dependencies inverted toward generic contracts.

Do not perform code behavior changes in the same patch unless explicitly requested.
```

## 22. Final Summary

The final app-neutral vocabulary is:

```text
PresentationTarget
  where input/output happens

AppHost
  owns app lifecycle and policy

CompositionRoot
  top-level app composition structure

RegionGraph
  structural placement topology

Region
  place where content can appear

MountedUnit
  mounted identity/lifecycle/capability record

UnitProvider
  observation/presentation/route adapter

UnitSession
  local per-unit state, not domain truth

UnitContent
  content kind: UI, canvas, document, inspector, product preview, external, headless

UiControl
  reusable semantic UI control

UiComposition
  reusable UI structure and interaction mechanism

AdaptiveCompositionRuntime
  fluid layout proposal and transition mechanism

InspectableTarget
  typed reference for inspection

Story
  proof scenario
```

The platform goal is not an editor UI system and not a game UI system.

The platform goal is an app-neutral composition system where editors, browsers, games, drawing tools, dashboards, terminals, mobile-style apps, and headless proof hosts are all specializations over the same neutral vocabulary.
