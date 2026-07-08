---
title: UI Component Composition Slots And Authoring Design
description: Long-term component, slot, template, reusable composition kit, authoring frontend, and design-system model for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-layout-style-theme-and-motion-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Component Composition Slots And Authoring Design

## Status

Active long-term UI design direction. This document defines the component,
composition, slot, template, and authoring-frontend model needed for Runenwerk to
be a reusable standalone UI framework. It does not authorize implementation by
itself.

## Decision

Runenwerk UI needs two distinct extension layers:

```text
Control packages
  define primitive and complex UI controls with schemas, kernels, accessibility,
  fixtures, diagnostics, and migration.

Composition components/templates
  define reusable arrangements of controls, slots, bindings, styles, and actions.
```

Controls are package-owned UI semantics. Components are reusable source/program
composition units. Do not collapse them into one central enum or one universal
node model.

## Authoring Frontends

The framework should support multiple authoring frontends that all lower to
`UiSource`:

```text
Rust builder authoring
RON/template source assets
visual designer authoring
generated app projection source
imported/compatibility source
future textual DSL if justified
```

Each frontend must preserve:

```text
stable source ids
source-map provenance
package requirements
schema references
localization keys
accessibility metadata
style/theme references
action bindings
preview metadata
```

## Component Model

A `UiComponentSource` should define:

```text
component id
component version
public props schema
public event/action slots
required packages
required capabilities
slot declarations
style slots
theme token references
accessibility contract
state retention needs
source body
fixtures
diagnostics
migration hooks
```

A component is not a retained runtime widget. It lowers through source,
normalization, interaction formation, program formation, artifact generation, and
evaluation like other UI source.

## Slot Model

Slots are typed insertion points.

Required slot kinds:

```text
ContentSlot
ActionSlot
ToolbarSlot
HeaderSlot
FooterSlot
InspectorSlot
OverlaySlot
MenuSlot
StatusSlot
PreviewSlot
WorldSpaceAnchorSlot
```

A slot must declare:

```text
slot id
accepted source/control/component kinds
required capabilities
layout constraints
accessibility expectations
fallback content
empty-state behavior
multiplicity: zero/one/many
ordering policy
source-map behavior
```

Slots must lower into explicit program facts, not hidden dynamic callbacks.

## Template And Instance Model

Templates allow reusable source definitions.

Template instance facts:

```text
template id
template version
instance id
prop values
slot fills
style overrides
action bindings
localization overrides
source-map origin
migration status
```

Template instance lowering must be deterministic and reportable.

## Reusable Composition Kits

Reusable composition kits are package-backed component families. Candidate kits:

```text
TabsControl
DockLayout
SplitLayout
ScrollArea
OverlayLayer
PopupMenu
CommandPalette
InspectorPanel
PropertyGrid
CanvasViewport
TimelinePanel
OutlinerTree
StatusBar
Toolbar
DataTable
VirtualList
RadialMenu
GameHudPanel
WorldSpacePrompt
```

Composition kits must not own product semantics. They provide generic UI
composition behavior and expose typed slots/actions.

## Design System Integration

Components and kits should bind to design-system concepts:

```text
theme tokens
style slots
variant names
state selectors
motion tokens
spacing scale
typography scale
icon slots
content density
platform profile variants
accessibility variants
```

Design-system facts are source/program facts, not renderer constants.

## Component State Boundaries

Component-local state is allowed only when it is UI runtime state:

```text
open/closed disclosure state
selected tab id
scroll position
active descendant
hover/pressed/focus state
animation progress
text edit state
virtualization window state
```

Component-local state must be keyed by stable source/program identity and governed
by `UiStateRetentionPolicy`.

Forbidden component state:

```text
app model truth
gameplay state
asset source truth
material/procgen/domain meaning
renderer resource ownership
```

## Collection Components

Collection components must handle:

```text
stable item ids
item key policy
insert/remove/move/update diffs
selection state
multi-selection
range selection
sorting
filtering
grouping
virtualization
recycling
keyboard/gamepad navigation
accessibility active-descendant behavior
```

Collection components must not identify items only by visible index unless a proof
explicitly demonstrates that index identity is safe.

## Authoring Ergonomics Requirements

Rust authoring should be terse for simple products but still lower to explicit
source/program facts.

Desired properties:

```text
small app files stay small
no macro dependency required for basic authoring
no manual route/event/render plumbing
explicit action declarations
explicit package requirements or inferred with report
stable ids visible or derivable
advanced authoring can opt into source assets and visual design
```

## Reports

Required reports:

```text
UiComponentValidationReport
UiSlotResolutionReport
UiTemplateExpansionReport
UiCompositionKitReport
UiComponentMigrationReport
UiComponentAccessibilityReport
UiComponentPreviewReport
```

## Acceptance Criteria

A first component slice should prove:

```text
package-backed Button and Text controls
one reusable component with typed props
one composition component with typed slots
one product component using props and slots
source validation rejects invalid slot content
slot fallback content is deterministic
source maps survive template expansion
state retention is keyed by stable component/source identity
headless proof can inspect expanded source and resulting UiProgram
```

## Rejected Shapes

Reject:

```text
one universal component enum
slots as untyped dynamic bags
components mutating app/game state directly
visual designer-only semantics
component runtime state keyed by unstable runtime ids
template expansion without source-map provenance
reusable kits that own product semantics
```
