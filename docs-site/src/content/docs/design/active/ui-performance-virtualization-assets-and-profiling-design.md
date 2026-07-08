---
title: UI Performance Virtualization Assets And Profiling Design
description: Long-term performance, virtualization, asset loading, caching, renderer packet, memory, profiling, and budgeting requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Performance Virtualization Assets And Profiling Design

## Status

Active long-term UI design direction. This document defines performance,
virtualization, asset loading, caching, renderer packet, memory, profiling, and
budgeting requirements. It does not authorize implementation by itself.

## Decision

Runenwerk UI must be designed for editor-scale and game-runtime-scale workloads.
Performance must be a first-class program/artifact/runtime contract, not an
emergent renderer optimization.

Hot paths consume optimized artifacts and runtime state. They must not interpret
authoring source graphs by default.

## Performance Budgets

Every host profile should declare budgets:

```text
frame time budget
UI evaluation budget
layout budget
text shaping budget
render packet generation budget
input-to-action latency budget
memory budget
allocation budget
asset loading budget
hot-swap budget
preview evaluation budget
```

Budget violations must produce diagnostics and profiler events.

## Artifact And Cache Model

Required caches:

```text
UiRuntimeArtifactCache
LayoutCache
StyleResolutionCache
TextMeasurementCache
TextShapingCache
GlyphAtlasPreparationCache
ImageDecodeCache
IconVectorCache
RenderPacketCache
HitTestCache
NavigationCache
AccessibilityTreeCache
```

Cache keys must include relevant source/program/package/theme/host/font/locale
revisions. Cache hits and misses must be reportable.

## Virtualization

Large collections require virtualization.

Required virtualization concepts:

```text
VirtualList
VirtualGrid
VirtualTree
VirtualTable
VirtualizedInspector
VirtualizedOutliner
VirtualizedTimeline
VirtualizedLog
```

Virtualization must handle:

```text
visible range
overscan
stable item ids
item measurement
estimated size
realized item set
recycled item containers
selection state
focus state
scroll anchoring
keyboard/gamepad navigation
accessibility active-descendant behavior
partial data availability
streaming data sources
```

UI virtualization and data virtualization are separate. UI virtualization limits
realized visual/control objects. Data virtualization limits loaded data.

## Container Recycling

Recycling must preserve item-bound state correctly.

Rules:

```text
container runtime state must not leak across item ids
item state must be keyed by stable item id
container state reset/migration must be reported
recycling must preserve accessibility and focus correctness
recycling must not break source-map diagnostics
```

## Asset Loading

UI assets include:

```text
fonts
icons
images
sprites
vector images
nine-slice/sliced images
cursor assets
controller glyphs
theme assets
sound cues where UI owns the cue intent
localized text bundles
```

Asset loading must support:

```text
asset ids
asset versions
asset dependency graph
async loading
placeholder/fallback assets
last-known-good assets
failure diagnostics
host capability checks
cache keys
preload hints
streaming hints
memory budget tags
```

## Renderer Output Boundaries

UI may produce renderer-facing packets, but renderer must not own UI meaning.

Renderer-facing output should include:

```text
visual packets
draw-neutral primitive packets
clip stacks
transform stacks
z/layer order facts
text/glyph run requests
image/icon references
scissor/viewport facts
opacity/effect facts
hit-test geometry where needed by host
```

Renderer-specific GPU resources belong to renderer/backend layers. UI output must
stay draw-neutral unless a specific host contract authorizes a backend-specific
artifact.

## Memory And Lifetime

The framework must define lifetimes for:

```text
source snapshots
program snapshots
runtime artifacts
runtime state entries
preview sessions
asset handles
render packet buffers
text shaping buffers
recycled containers
diagnostics
proof traces
```

Long-running editor sessions require explicit cleanup, weak-reference policy, and
leak diagnostics for retained state, assets, event subscriptions, and preview
sessions.

## Profiling

Required profiler surfaces:

```text
UiEvaluationProfiler
UiLayoutProfiler
UiStyleProfiler
UiTextProfiler
UiBindingProfiler
UiInputRoutingProfiler
UiRenderPacketProfiler
UiAccessibilityProfiler
UiAssetProfiler
UiLivePreviewProfiler
UiMemoryProfiler
```

Profiler events must include source/program/artifact ids where possible.

## Performance Reports

Required reports:

```text
UiPerformanceBudgetReport
UiCacheReport
UiVirtualizationReport
UiRecyclingReport
UiAssetLoadingReport
UiMemoryReport
UiProfilerTraceReport
UiRendererPacketReport
UiHotPathViolationReport
```

## Acceptance Proofs

A mature framework must prove:

```text
large virtualized list does not realize all items
collection diff updates only affected visible range
item state does not leak across recycled containers
text/layout cache is reused across stable revisions
asset fallback works when an icon/font/image is missing
budget violation produces diagnostics
headless profiler trace is reproducible
```

## Rejected Shapes

Reject:

```text
full source graph interpretation in frame hot path
large lists realizing every row by default
container recycling without stable item identity
asset loads without placeholder/fallback/diagnostics
renderer-owned UI semantics
unreported cache invalidation
unbounded allocation in UI frame loop
profiling only through ad hoc logs
```
