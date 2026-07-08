---
title: UI Framework Runtime Requirements Design
description: Long-term requirements for Runenwerk as a standalone UI framework covering controls, layout, style, text, input, focus, accessibility, animation, surfaces, game UI, world-space UI, inspection, proofs, and renderer boundaries.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-source-projection-and-program-lowering-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
  - ./domain-authoring-source-and-program-pattern.md
---

# UI Framework Runtime Requirements Design

## Status

Active long-term UI design direction. This document defines what Runenwerk must
support to become a mature standalone UI framework for editor UI, game UI,
world-space UI, headless proof, live preview, and future designer tooling.

This document does not authorize broad implementation, crate creation, renderer
changes, or product integration by itself.

## Decision

Runenwerk UI should become a standalone UI framework built on the existing
source/program/artifact/evaluator/host spine.

The mature UI framework target is:

```text
UiSource authoring
-> normalization and interaction formation
-> UiProgram
-> UiRuntimeArtifact
-> reactive evaluator
-> retained UI runtime state
-> UiOutput / UiFrame / UiEventPacket
-> host integration
-> inspection, diagnostics, proof, and live-preview reports
```

The framework must support declarative authoring and reactive updates, while
keeping app/domain mutation outside UI controls and keeping renderer primitives as
derived output only.

## Required Capability Areas

A mature Runenwerk UI framework must cover these areas as first-class contracts:

```text
source authoring
component/control packages
layout
style and theme
text and font handling
input normalization
actions and route mapping
focus and navigation
accessibility
state and binding
reactive invalidation
animation and transitions
overlay, popup, and layering
scrolling
surface mounting
game HUD and menus
world-space UI
live editing and preview
inspection and debugging
headless proof and replay
renderer output and artifact caching
migration and compatibility
```

## Framework Layers

### Source Layer

Owns human/tool-facing UI source:

```text
UiSource
source ids
source versions
source body kinds
package requirements
route/action bindings
localization keys
accessibility metadata
theme/style references
source-map provenance
```

### Program Layer

Owns durable semantic UI contracts:

```text
UiProgram
ControlGraph
LayoutGraph
StyleGraph
InteractionGraph
BindingGraph
AccessibilityGraph
VisualGraph
```

`UiProgram` is the stable executable contract. It is not a Rust builder output,
not a retained widget tree, and not a renderer frame.

### Artifact Layer

Owns optimized runtime products:

```text
UiRuntimeArtifact
UiRuntimeArtifactManifest
layout tables
style tables
text shaping requests
binding dependency tables
hit-test tables
navigation tables
accessibility tables
render packet plans
cache keys
source-map tables
```

Artifacts are reproducible products of source/program/package inputs.

### Runtime State Layer

Owns retained state that must survive source projection and frame updates:

```text
focus
hover
pressed
active
scroll offsets
text edit cursor and selection
IME composition
drag/drop state
animation clocks
transition state
popup stack
capture state
accessibility focus
host-fed ephemeral data
package-owned control state
```

Runtime state must be keyed by stable program/source identity and governed by a
state retention policy.

### Evaluator Layer

Owns deterministic evaluation from artifact plus runtime/host data into output:

```text
UiEvaluationContext
UiEvaluationRevision
UiDependencyGraph
UiInvalidationReport
UiOutput
UiOutputDelta
UiDiagnosticReport
UiInspectionReport
```

### Host Layer

Hosts connect UI output and event packets to concrete environments:

```text
EditorHost
GameHudHost
GameMenuHost
WorldSpaceHost
HeadlessHost
PreviewHost
RemoteDevtoolsHost
```

Hosts perform side effects. UI produces facts and proposals.

## Control And Component Requirements

Controls are package-backed. They are not central enum cases.

Base package minimum:

```text
Text
Button
Image
Icon
Column
Row
Grid
Stack
Spacer
ScrollArea
Panel
OverlayLayer
Popup
Menu
TextInput
Slider
Checkbox
Radio
ListView
TreeView
TableView
TabView
SplitView
DockArea
CanvasViewport
InspectorField
ColorPicker
ActionPrompt
ProgressBar
```

Each control package contribution must define:

```text
control id and version
property schema
event schema
required capabilities
layout behavior
style slots
visual intent
input behavior
focus behavior
accessibility role/name/value behavior
state retention needs
fixtures
diagnostics
migration hooks
inspection data
```

## Layout Requirements

The layout system must support:

```text
flex-like stacks
CSS-grid-like grid where appropriate
absolute/anchored overlay positioning
intrinsic sizing
min/max constraints
percentage and relative units
safe-area constraints
DPI scaling
responsive variants
overflow and clipping
scroll containers
text measurement constraints
world-space projection constraints
multi-surface constraints
```

Layout must produce explicit reports:

```text
layout input summary
layout graph revision
dirty layout scope
constraint failures
overflow warnings
final rects
hit regions
source-map links
```

## Style And Theme Requirements

Style and theme must be source/program facts, not renderer hardcoding.

Required concepts:

```text
theme tokens
style slots
variants
state selectors
host profile overrides
product profile overrides
dark/light/high-contrast variants
animation/transition tokens
spacing tokens
typography tokens
color tokens
radius tokens
border tokens
shadow/effect tokens
```

Theme resolution must be deterministic and reportable.

## Text Requirements

Text must support:

```text
font fallback
font style intent
text shaping requests
line breaking
wrapping
truncation
rich text spans
localization keys
format arguments
bidirectional text
inline icons/glyphs
accessibility labels
text selection
IME composition for editable text
```

Text shaping output is a derived artifact or evaluator product, not source truth.

## Input And Action Requirements

Input flow:

```text
host input facts
-> normalized ui input
-> hit/focus/navigation routing
-> control interaction facts
-> UiEventPacket
-> route-action mapping
-> app/domain action proposal
```

Required input classes:

```text
pointer
mouse
keyboard
gamepad/controller
touch
stylus/tablet
wheel/scroll
text input
IME
accessibility actions
virtual/remote input
```

UI controls emit typed UI events and route facts. They must not mutate app or game
state directly.

## Focus And Navigation Requirements

The framework must support:

```text
focus scopes
focus restoration
modal focus trapping
popup focus handoff
keyboard tab order
gamepad cardinal navigation
explicit navigation edges
automatic navigation fallback
multi-player focus ownership
pointer capture
input modality tracking
```

Game UI requires gamepad and controller navigation as a first-class feature, not
a later widget afterthought.

## Accessibility Requirements

Accessibility must be present in source/program/runtime artifacts:

```text
roles
names
descriptions
values
states
focus order
navigation hints
semantic grouping
live regions
keyboard activation
screen-reader compatible output where host supports it
```

Every control package must define accessibility behavior or explicitly report why
it is non-accessible in a given host profile.

## Animation And Transition Requirements

Animation must be declarative and host-compatible:

```text
transition source facts
animation clocks
interpolation curves
layout transitions
style transitions
visibility transitions
screen transitions
popup transitions
focus/hover/pressed transitions
reduced-motion policy
runtime animation state retention
```

Animations must not hide app mutation. They consume time/runtime state and produce
visual output facts.

## Overlay, Popup, And Layering Requirements

The framework must support:

```text
overlay stacks
popup ownership
modal layers
tooltips
menus
context menus
drag previews
drop targets
z-order policy
hit-test blocking policy
dismissal rules
focus trapping
host surface constraints
```

Layering is not renderer-owned; renderer receives derived draw/order facts.

## Surface Requirements

Surface support must include:

```text
desktop windows
editor panels
game HUD surfaces
game menu surfaces
world-space surfaces
remote preview surfaces
headless surfaces
multi-surface apps
surface lifetime
surface mount eligibility
projection transforms
visibility and occlusion facts
```

Surface semantics belong to UI/surface/host contracts, not product apps.

## Inspection, Debugging, And Proof Requirements

The framework must expose:

```text
source maps
program graph inspection
layout inspection
style/theme inspection
binding dependency inspection
runtime state inspection
accessibility inspection
input routing traces
focus/navigation traces
event packet traces
artifact manifests
cache key reports
replay traces
host compatibility matrices
```

A UI that cannot explain why it rendered, updated, focused, routed, or rejected an
action is not acceptable for Runenwerk.

## Rejected Framework Shapes

Reject:

```text
callback-first controls
renderer-owned source truth
ECS-owned UI semantics
central enum control catalog
hidden global mutable registries
immediate-mode-only framework for all UI
retained WidgetId in authored source
manual event packet construction in product code
manual prepared-frame construction in product code
generic graph interpretation in hot paths by default
```

## Maturity Gates

A framework implementation is not mature until it proves:

```text
small app proof
headless proof
editor panel proof
game HUD proof
game menu/gamepad proof
world-space proof
live-preview proof
accessibility proof
theme/style proof
text editing proof
animation/transition proof
artifact cache/invalidation proof
non-UI domain extraction restraint
```
