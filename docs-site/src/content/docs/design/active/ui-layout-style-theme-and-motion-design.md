---
title: UI Layout Style Theme And Motion Design
description: Long-term layout, style cascade, theme token, responsive variant, source order, motion, animation, and transition requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-accessibility-internationalization-and-text-conformance-design.md
  - ./ui-performance-virtualization-assets-and-profiling-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Layout Style Theme And Motion Design

## Status

Active long-term UI design direction. This document defines the layout, style,
theme, responsive variant, source-order, motion, animation, and transition model
needed for a mature Runenwerk UI framework. It does not authorize implementation
by itself.

## Decision

Layout, style, theme, and motion are UI program/artifact facts. They are not
renderer constants, product-app code, or hidden widget behavior.

Correct shape:

```text
UiSource layout/style/theme/motion intent
-> NormalizedUiTemplate
-> UiProgram layout/style/visual/motion graphs
-> UiRuntimeArtifact tables
-> evaluator output facts
-> renderer-facing packets
```

Renderer backends consume derived packets. They do not own layout, style, or
motion semantics.

## Layout Families

Runenwerk should support multiple layout families through UI-owned semantics:

```text
StackLayout
FlexLikeLayout
GridLayout
AbsoluteLayout
AnchorLayout
OverlayLayout
DockLayout
SplitLayout
ScrollLayout
CanvasViewportLayout
WorldSpaceProjectedLayout
```

The framework may borrow concepts from web layout where useful, especially
flex-like single-axis distribution and grid-like two-dimensional placement, but it
must not pretend to be CSS unless it commits to exact CSS compatibility.

## Source Order And Accessibility

Visual order and semantic/navigation order are distinct.

Rules:

```text
source order is the default semantic order
layout may produce visual reordering facts
visual reordering must report accessibility impact
keyboard/controller navigation must be explicit when visual order differs
screen-reader/semantic traversal must not be accidentally derived from draw order
```

Grid and docking systems must preserve source-map and semantic-order evidence.

## Layout Inputs

Layout consumes:

```text
program node tree/graph
layout constraints
content intrinsic sizes
text measurement requests
image/icon intrinsic sizes
surface constraints
safe-area constraints
host profile constraints
DPI/font scale
world-space projection constraints
runtime state where allowed, such as scroll offset
```

## Layout Outputs

Layout produces:

```text
resolved rects
clip rects
scroll extents
hit-test regions
focus navigation geometry
accessibility bounds
render transform facts
overflow diagnostics
layout invalidation facts
source-map links
```

## Constraint And Measurement Policy

Measurement must be explicit and bounded.

Required concepts:

```text
AvailableSize
IntrinsicSize
MinSize
MaxSize
PreferredSize
AspectRatio
Baseline
ContentSize
MeasureFunction
MeasureCacheKey
LayoutPassLimit
LayoutCycleDiagnostic
```

Controls with custom measurement must declare measurement dependencies and budget
class.

## Responsive And Host Variants

Responsive behavior should be declared as source/program facts:

```text
surface size variants
aspect-ratio variants
safe-area variants
input-modality variants
locale/text-expansion variants
accessibility-scale variants
host-profile variants
product-profile variants
world-space distance/LOD variants
```

Variant selection must be deterministic and reportable.

## Style Model

Style resolution should support:

```text
style slots
theme tokens
component variants
state selectors
host selectors
accessibility selectors
input-modality selectors
specificity or priority policy
inheritance policy
fallback policy
source-map provenance
```

Runenwerk may borrow CSS-like concepts such as tokens, selectors, and cascade, but
must define its own exact priority model if it does not implement CSS.

## Theme Model

Theme packages should define:

```text
color tokens
typography tokens
spacing tokens
radius tokens
border tokens
shadow/effect tokens
motion tokens
icon tokens
density tokens
high-contrast variants
dark/light variants
reduced-motion variants
color-blind safe hints
```

Theme resolution must be deterministic, cacheable, inspectable, and host-profile
aware.

## Motion And Animation

Motion is declarative visual/runtime state, not app mutation.

Required concepts:

```text
TransitionSource
AnimationSource
MotionToken
AnimationClock
EasingCurve
Duration
Delay
RepeatPolicy
InterruptPolicy
ReducedMotionFallback
LayoutTransition
StyleTransition
VisibilityTransition
ScreenTransition
PopupTransition
FocusTransition
HoverPressedTransition
```

Animations consume time/runtime state and produce output facts. They must not hide
model mutation, host IO, or renderer-owned state.

## Reduced Motion And Safety

Motion must respect accessibility policy:

```text
reduced motion
animation disable/replace
flashing/seizure diagnostics
motion-triggered input diagnostics
large motion warning
```

Motion policy must be part of host/theme/accessibility compatibility reports.

## Invalidation

Layout/style/theme/motion changes must feed the reactive runtime:

```text
DirtyLayoutScope
DirtyStyleScope
DirtyThemeTokenSet
DirtyTextMeasureScope
DirtyMotionScope
DirtyRenderPacketScope
```

A style change should not force full source normalization unless schema/package
requirements changed.

## Reports

Required reports:

```text
UiLayoutReport
UiLayoutConstraintReport
UiStyleResolutionReport
UiThemeResolutionReport
UiVariantResolutionReport
UiMotionReport
UiReducedMotionReport
UiLayoutInvalidationReport
UiLayoutPerformanceReport
```

## Proof Requirements

A mature framework must prove:

```text
flex-like stack layout
2D grid layout
source order differs from visual order with accessible navigation preserved
safe-area variant changes layout
text expansion changes layout without clipping where constraints allow
style token change invalidates only affected style/render scopes
reduced-motion policy replaces animation
layout cycle produces diagnostics instead of infinite loop
```

## Rejected Shapes

Reject:

```text
layout hidden inside renderer
style constants hardcoded in product widgets
theme resolution by global mutable state
visual order treated as semantic order by default
unbounded measurement recursion
animation as hidden app mutation
reduced motion as optional polish
style/theme changes forcing complete app rebuild by default
```
