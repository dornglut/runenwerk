---
title: Draw Composition Layouts
description: Usage and ownership guide for Runenwerk Draw's app-neutral static composition and Draw-owned projection state.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-06-20
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_roadmaps:
  - ./roadmap.md
---

# Draw Composition Layouts

Runenwerk Draw is a direct non-editor consumer of
`domain/ui/ui_composition`. The core owns structural identity and topology;
Draw owns what its mounted content means and how that structure becomes a
canvas-first frame.

## Normal App Usage

`RunenwerkDrawApp` constructs the built-in composition during startup. Normal
callers read the current projection without rebuilding or interpreting the
region graph:

```rust
use runenwerk_draw::app::RunenwerkDrawApp;

let app = RunenwerkDrawApp::new();
let projection = app.composition_projection();

assert!(projection.canvas_view.screen_bounds.width > 0.0);
assert!(projection.tool_rail_bounds.width > 0.0);
assert!(projection.support_panel_bounds.width > 0.0);
```

Advanced inspection is available through `composition_runtime()` and
`composition_content()`. Structural callers should use typed IDs and the
ratified `CompositionState`; display names and pixel geometry are not identity.

## Ownership

| Concern | Owner |
|---|---|
| target, root, region, split, mounted-unit identity | `ui_composition` |
| built-in wide definition | `apps/runenwerk_draw/src/app/composition/definition.rs` |
| content role and extension compatibility | `apps/runenwerk_draw/src/app/composition/extension.rs` |
| content liveness and fallback policy | `apps/runenwerk_draw/src/app/composition/content.rs` |
| target pixels, region bounds, canvas view | `apps/runenwerk_draw/src/app/composition/projection.rs` |
| document, stroke, brush, layer, drawing undo/history | `domain/drawing` and Draw app state |
| native window, DPI, monitor, restore, OS veto | app/engine/windowing code |

`DrawingCompositionExtensionV1` maps every `MountedUnitId` to exactly one
`DrawingContentRole`. It does not copy split topology, region parentage,
mounted-content references, canvas state, or native handles. Its canonical RON
normalizes records by mounted-unit ID and forms a linked
`CompositionBundleCandidate`, so future persistence can use the shared atomic
core/extension bundle rather than a Draw-specific layout format.

## Static Projection

The built-in `runenwerk.draw.wide` profile has one primary target/root and four
mounted roles:

1. top bar;
2. tool rail;
3. canvas;
4. support/tablet panel.

`DrawingCompositionProjection::project` recursively evaluates the ratified
`RegionKind` graph into app-owned target bounds. The canvas view is then derived
from the canvas mounted-unit bounds and the Draw paper margin. There is no
second hard-coded topology.

The profile remains structurally static at narrow widths. It scales its split
fractions and keeps all four units mounted. Responsive drawer conversion,
collapse/reflow proposals, and Region Compass interactions belong to the later
adaptive-composition checkpoint.

## Unavailable Content

`DrawingCompositionContentState` stores one `ContentLiveness` value per
`MountedUnitId`: resolved, missing, loading, suspended, denied, unsupported
profile, or crashed.

Projection always applies this order:

1. Draw-provided unavailable-content projection;
2. neutral diagnostic placeholder;
3. hidden only when both the mounted-unit policy and host policy permit it.

The built-in Draw host keeps a neutral placeholder available, so unavailable
content never removes structural regions. Each unavailable projection emits a
stable `draw_composition.*` diagnostic with stage, subject ID, message, and
ordered context.

## Drawing Authority

Composition changes only layout structure. It never owns drawing undo/redo,
document revision, stroke transactions, browser-like history, ink products, or
GPU promotion. Resizing a target or changing content liveness must leave the
`DrawingDocument`, drawing revision, and composition state revision unchanged.
