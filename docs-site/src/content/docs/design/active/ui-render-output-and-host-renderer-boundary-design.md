---
title: UI Render Output And Host Renderer Boundary Design
description: Long-term renderer-facing UI output, draw-neutral packet, text/glyph, clipping, layering, render-target, invalidation, and backend-boundary requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-performance-virtualization-assets-and-profiling-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Render Output And Host Renderer Boundary Design

## Status

Active long-term UI design direction. This document defines the renderer-facing UI
output contract and host/renderer boundary. It does not authorize renderer changes
or backend implementation by itself.

## Decision

UI owns visual intent and renderer-facing UI output facts. Renderer backends own
GPU resources, render passes, backend-specific batching, and submission.

Correct direction:

```text
UiProgram
-> UiRuntimeArtifact
-> UiEvaluator
-> UiOutput / UiFrame / UiRenderPackets
-> Host renderer adapter
-> renderer/backend submission
```

Rejected direction:

```text
product app -> render primitives
renderer -> UI source truth
renderer -> action semantics
renderer -> layout/style/theme ownership
```

## UiOutput

`UiOutput` is the high-level evaluated UI result.

It may contain:

```text
frame id
surface outputs
render packet groups
hit-test regions
event route tables
focus/navigation facts
accessibility semantic facts
inspection facts
diagnostics
source-map references
```

## UiFrame

`UiFrame` is a renderer-facing frame product for a host surface.

Fields:

```text
surface id
surface kind
viewport/scissor facts
DPI scale
safe-area facts
clip stack
transform stack
layer/z-order facts
visual packets
text/glyph packets
image/icon refs
opacity/effect facts
hit-test geometry refs where host needs them
diagnostic overlays where enabled
```

## Draw-Neutral Visual Packets

UI packets should be draw-neutral unless a specific backend contract authorizes a
backend artifact.

Packet classes:

```text
RectPacket
RoundedRectPacket
BorderPacket
ImagePacket
VectorIconPacket
TextRunPacket
GlyphRunRequest
ClipPacket
TransformPacket
NineSlicePacket
MeshLikeUiPacket where explicitly authorized
EffectPacket
DebugOverlayPacket
```

Packet semantics are UI-owned visual facts. GPU buffers, textures, pipelines, and
command encoders are renderer-owned.

## Text And Glyph Boundary

UI owns:

```text
text content facts
font intent
font fallback requests
text shaping requests
glyph identity keys
text layout metrics
glyph atlas preparation keys
source-map and accessibility provenance
```

Renderer owns:

```text
glyph atlas GPU resources
texture uploads
sampling policy
backend-specific text rendering path
```

If the renderer cannot satisfy a text/glyph request, it must report capability
diagnostics instead of silently substituting incompatible output.

## Layering And Clipping

UI owns semantic layering:

```text
surface layer
HUD layer
popup layer
modal layer
tooltip layer
drag-preview layer
debug layer
world-space overlay layer
```

Renderer consumes resolved order facts.

Clipping output must include:

```text
clip id
clip shape kind
clip rect/path where supported
source node id
parent clip id
hit-test clipping policy
render clipping policy
```

## Render Targets And Surfaces

Host renderer adapters must support:

```text
window surface
editor panel surface
game HUD surface
game menu surface
world-space projected surface
render-to-texture surface
remote preview surface
headless/null render surface
```

Surface compatibility must report unsupported render target kinds, DPI/scaling
limits, text/font limitations, effect limitations, and clipping limitations.

## Color And Output Policy

UI visual facts should carry color/style intent, not backend pixel assumptions.

Required output facts:

```text
color token resolved value
color space tag where applicable
opacity
blend intent
effect intent
high-contrast variant decision
HDR/SDR host compatibility where applicable
```

## Invalidation And Batching

Renderer output invalidation should be explicit:

```text
DirtyVisualPacketSet
DirtyTextPacketSet
DirtyImagePacketSet
DirtyClipStack
DirtyTransformStack
DirtyLayerOrder
DirtySurfaceOutput
```

Renderer adapters may batch, cache, and reorder only within constraints provided by
UI output facts. Reordering must not change hit-test, focus, layering, or visual
semantics.

## Renderer Capability Reports

Required reports:

```text
UiRendererCompatibilityReport
UiRenderPacketReport
UiFrameReport
UiTextGlyphRequestReport
UiClipStackReport
UiLayeringReport
UiRendererFallbackReport
UiRendererBoundaryViolationReport
```

## Headless Rendering And Proof

Headless hosts may produce structural frame summaries without GPU output.

Required proof facts:

```text
packet count
layer order summary
text request summary
hit region summary
clip stack summary
surface output summary
diagnostics
```

Visual/golden tests may render screenshots, but screenshots must be paired with
structural frame reports.

## Rejected Shapes

Reject:

```text
renderer-owned UI source or action meaning
product app constructing render packets manually
GPU resources inside UiSource
backend-specific packets as the default UI output
render order as accessibility/focus order
silent renderer fallback without diagnostics
text rendering as opaque draw call with no source-map/accessibility link
```

## Acceptance Criteria

A first renderer-boundary proof should demonstrate:

```text
UiOutput produces draw-neutral packets
renderer adapter consumes packets without owning UI semantics
text request has source-map and glyph cache key
layer order differs from source order but focus/accessibility order is preserved
unsupported renderer feature produces compatibility diagnostics
headless frame summary matches rendered frame structure
```
