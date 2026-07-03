---
title: Surface2D Future Pressure Branch Review
description: Extracted future Surface2D/Canvas2D design pressure from stale branch surface2d-phase-16 without changing completed Phase 16 truth.
status: active
owner: ui
layer: reports
canonical: false
last_reviewed: 2026-07-03
related_docs:
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# Surface2D Future Pressure Branch Review

## Source

This report extracts still-useful future design pressure from the stale remote branch
`origin/surface2d-phase-16`.

Inspected source:

```text
git show origin/surface2d-phase-16:docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md
```

Branch evidence:

```text
origin/surface2d-phase-16 contains three non-equivalent commits beyond origin/main.
The branch changes only docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md.
The branch is reference material only and must not be merged or cherry-picked.
PR #64 later merged this extraction at 05c51375986cf08e360884ebf44702ec62662c1e.
Current Phase 17 intake branch inspection no longer lists origin/surface2d-phase-16.
```

Current authority remains `main`:

```text
docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md
docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
```

Evidence classes used:

```text
E2 repository file inspection for the stale branch design content
E5 local git command output for branch and diff shape
E8 accepted workspace, planning, design, and closeout authority on main
```

Freshness classification:

```text
stale:
  origin/surface2d-phase-16 branch content

current:
  main branch Phase 16 design, closeout, and planning truth
```

## Decision

Preserve only future/reference pressure that can help later Surface2D,
Canvas2D, and specialized canvas design work.

Do not modify completed Phase 16 truth. Phase 16 Surface2D is completed on
`main`. This report does not reopen Phase 16, does not start Phase 17 work,
and does not create a planning contract for any later canvas.

The extraction PR has merged. Current Phase 17 intake branch inspection no
longer lists the stale `surface2d-phase-16` remote branch.

## What Was Extracted

The following future pressure was worth keeping as reference material:

```text
Surface2D vs Canvas2D naming split
future canvas-family pressure
hierarchy boundary model
future-use-case guard matrix
ergonomic and default pressure
future feature ladder
non-goals for future design intake
```

The extracted content is deliberately restated as future pressure. It is not a
Phase 16 gate, not a current planning entry, and not a product code task.

## What Was Rejected

The stale branch also contained content that must not survive this extraction:

```text
pre-merge Phase 16 lifecycle wording
pre-merge implementation authorization wording
pre-merge completion-status claims
pre-merge delivery-status claims
single-file owner-path pressure from the stale branch
retained-tree owner proposals for the already completed Phase 16 scope
stale focused validation commands that no longer match the completed closeout
claims that would conflict with PR #61 or the later Phase 16 planning truth
anything that starts Phase 17 work
```

Rejected content remains historical branch context only. It is not copied into
the active design, planning records, or future implementation contracts.

## Surface2D vs Canvas2D Naming

Use this naming split for future design intake:

```text
Surface2D:
  durable renderer-neutral coordinate, navigation, transform, bounds,
  layer, input-fact, accessibility-fact, budget, and proof substrate.

Canvas2D:
  possible future author-facing facade that presents common canvas defaults
  and friendlier naming while delegating coordinate/navigation substrate facts
  to Surface2D.

Specialized future canvases:
  DrawingCanvas, ImageCanvas, TextureCanvas, CurveCanvas, NodeCanvas,
  GraphCanvas, TimelineCanvas, TrackSurface, SpatialCanvas, ViewportCanvas,
  UI Designer canvas, and product-specific editors.
```

Guard:

```text
Do not rename Surface2D to Canvas2D.
Do not introduce Canvas2D as a compatibility alias without a separate accepted design.
Do not let facade terminology leak into durable substrate APIs.
Do not let future specialized canvases make Surface2D own their semantic models.
```

The naming split keeps the substrate honest: `Surface2D` owns reusable
coordinate/navigation facts, while future canvas controls own author ergonomics
and domain-specific semantics.

## Future Canvas-Family Pressure

Future canvas families pressure the substrate in different ways. These are
pressure inputs only; they do not authorize implementation.

| Future canvas family | Needs from Surface2D | Must own outside Surface2D | Required guard |
|---|---|---|---|
| Canvas2D | safe defaults, fit, pan, zoom, hover coordinate, selection rectangle, accessibility facts, inspection facts | author-facing naming, simplified builders, gallery or app-builder UX | keep Canvas2D as a future facade |
| DrawingCanvas | stable content coordinates, pointer capture, gesture cancel/commit, layer bands, large-content budget evidence | strokes, brushes, snapping, guides, handles, drawing undo, layer model, document mutation | keep selection rectangle transient and non-semantic |
| ImageCanvas / TextureCanvas | content bounds, pixel/content coordinate mapping, zoom, pan, fit, hover coordinate, diagnostics | image assets, texture resources, sampling policy, image editing operations | keep renderer resources and texture ownership outside Surface2D |
| CurveCanvas | precise coordinates, zoom-sensitive hit-test evidence, hover facts, selection rectangle, layer bands | curves, handles, tangents, constraints, sampled previews, curve editing history | curve model and editing commands belong to CurveCanvas |
| NodeCanvas / GraphCanvas | panning, zooming, large-content bounds, selection rectangle, hover coordinate, diagnostic overlay, layer ordering | nodes, links, ports, sockets, graph selection, graph commands, graph layout | forbid graph semantics in Surface2D |
| TimelineCanvas / TrackSurface | coordinate mapping, pan/zoom, fit, hover coordinate, layer bands, diagnostics, budget evidence | time units, tracks, clips, keyframes, scrubber behavior, timeline commands | map time externally and keep time semantics out of Surface2D |
| UI Designer canvas | viewport/content bounds, coordinate conversion, fit, hover coordinate, selection rectangle evidence, UI child overlays | authored UI mutation, widget selection, drag/drop editing, property edits, designer command history | Surface2D may expose coordinates and overlays, not authored UI truth |
| SpatialCanvas / ViewportCanvas | 2D overlay navigation, viewport framing, diagnostics, input capture evidence, renderer-neutral bounds proof | camera, 3D world transforms, render targets, scene resources, viewport backend | keep Surface2D two-dimensional and renderer-neutral |
| Remote/headless proof surfaces | deterministic reports, stable fact counts, no backend handles, inspectable state, static mount proof | transport, streaming, host execution, remote session lifecycle | reports and proof frames only; no host/session effects in domain UI |

## Hierarchy Boundary Model

Future work should keep three hierarchy layers separate.

| Hierarchy layer | Owner | Surface2D may provide | Surface2D must not own |
|---|---|---|---|
| App/layout hierarchy | app, workspace, layout, and composition owners | mountable facts, bounds, intent/report data, and host-consumable proof evidence | app regions, panels, routes, layout persistence, or app composition mutation |
| Retained UI hierarchy | retained UI/runtime owners | retained surface state, renderer-neutral proof facts, UI overlays, diagnostics, labels, adornments, and child-control containment facts | product object hierarchy, authored document hierarchy, or app composition history |
| In-surface semantic hierarchy | specialized canvas, editor, product, or game owners | coordinates, transforms, navigation state, selection rectangle facts, input facts, and budget facts | nodes, links, strokes, clips, tracks, curves, product entities, semantic selection, commands, or persistence |

The important boundary is not whether a future surface can display children or
overlays. It can. The boundary is that UI containment remains UI containment,
while semantic content belongs to the specialized canvas or product domain.

## Future-Use-Case Guard Matrix

| Future use case | Pressure accepted as useful | Outside owner | Scope-leak risk | Required guard |
|---|---|---|---|---|
| Canvas2D facade | author-friendly defaults and vocabulary | future reusable canvas facade owner | facade names become substrate names | keep Surface2D durable and Canvas2D facade-only |
| DrawingCanvas | pointer capture, transient rectangles, layer bands, large-content bounds | drawing document/editor owner | stroke or brush semantics enter Surface2D | Surface2D emits interaction facts only |
| ImageCanvas / TextureCanvas | pixel/content coordinate mapping and fit | image or texture domain owner | texture resources enter UI substrate | Surface2D records coordinates, not resources |
| CurveCanvas | precise coordinate and hover evidence | curve editing owner | curve handles and constraints enter substrate | CurveCanvas owns curve model |
| NodeCanvas / GraphCanvas | scalable navigation and overlays | graph editor owner | generic surface becomes a graph scene tree | no node/link/port vocabulary in Surface2D |
| TimelineCanvas / TrackSurface | axis mapping and navigation facts | timeline or track owner | time/track semantics enter coordinate substrate | time is mapped by the timeline layer |
| UI Designer canvas | UI child overlays and inspection facts | UI Designer/editor definition owner | authored UI mutation enters Surface2D | authored mutations use editor-owned commands |
| SpatialCanvas / ViewportCanvas | viewport framing and overlay proof | spatial/viewport owner | Surface2D becomes a renderer viewport authority | Surface2D stays 2D and backend-neutral |
| Remote/headless proof | deterministic facts and reports | host/session/transport owners | proof work starts host execution policy | no host effects in Surface2D |

This matrix is a design-pressure checklist. It must be revalidated by any future
planning intake before it can shape implementation scope.

## Ergonomic And Default Pressure

Future facade or specialized-canvas intake should consider these defaults and
inspection facts without back-porting stale Phase 16 planning language:

```text
default zoom range
initial fit-content behavior
keyboard pan
keyboard zoom
keyboard fit
wheel and high-resolution scroll intent
focus-visible fact
screen-reader/inspection-readable name and bounds
reduced-motion behavior
layer-band ordering
deterministic budget limits
```

Guard:

```text
Defaults should make the common path pleasant and inspectable.
Advanced configuration should remain explicit.
Unsupported input modes should be reported explicitly by the owning future design.
Budget evidence should prefer deterministic fact/operation counts unless a future owner proves that wall-clock budgets are meaningful.
```

These ergonomic pressures are future intake criteria. They do not imply that the
completed Phase 16 contract is missing work.

## Future Feature Ladder

The useful ladder from the stale branch is retained as future reference:

```text
Surface2D substrate
  coordinates, viewport, transform, pan/zoom, fit, hover, selection rectangle,
  pointer capture, layer bands, diagnostics, budgets, hierarchy boundaries,
  future-use pressure evidence, and static proof.

Canvas2D facade
  author-friendly defaults, common canvas declaration, visible proof surfaces,
  and app-builder friendly terminology.

DrawableCanvas
  strokes, shapes, handles, snapping, guides, and layers.

NodeCanvas / GraphCanvas
  nodes, links, ports, sockets, graph layout, graph selection, and graph commands.

TimelineCanvas
  tracks, clips, time ruler, scrubber, and keyframes.

CurveCanvas
  curve handles, tangents, sampled preview, and editing constraints.

SpatialCanvas / ViewportCanvas
  camera-like navigation, spatial projection boundaries, and viewport-host integration.

Product-specific editors
  SDF graph, material graph, animation graph, gameplay graph, particle graph,
  UI Designer workflows, texture workflows, and other product-owned editors.
```

Every rung above `Surface2D` needs its own owner, investigation gate, design
gate, planning contract, validation envelope, and stop conditions before
implementation.

## Non-Goals

This report does not:

```text
merge surface2d-phase-16
cherry-pick surface2d-phase-16 commits
change product code
change completed Phase 16 design truth
reopen Phase 16
start Phase 17 work
start Canvas2D implementation
start specialized canvas implementation
create a new crate
change Surface2D public APIs
change validation envelopes for completed Phase 16
replace the Phase 16 closeout
```

## Follow-Up

Use this report only as reference pressure for later canvas-family planning.
It does not reopen Phase 16, start Phase 17 implementation, or authorize any
Surface2D API change.
