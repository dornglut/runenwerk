---
title: Runenwerk Draw Pen-First Radial Tablet UX Design
description: Active UX and architecture design for pen-first radial menus, generic stylus input, tablet diagnostics, and low-latency drawing workflows in runenwerk_draw.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-19
related_docs:
  - ../../apps/runenwerk-draw/README.md
  - ./drawing-authoring-and-comic-layout-platform-design.md
  - ../../domain/drawing/README.md
  - ../../domain/ui/architecture.md
  - ../../adapters/native-tablet-input/README.md
---

# Runenwerk Draw Pen-First Radial Tablet UX Design

## Status

Active design.

This document is design-only. It defines the intended `runenwerk_draw`
pen-first interaction model and the implementation boundaries that later code
must follow.

It does not replace the broader drawing platform design. It specializes the UX
contract for radial menus, stylus/tablet setup, and canvas-first drawing
interaction inside the focused drawing app.

## Purpose

`runenwerk_draw` should feel like a pen-first drawing application, not a
general editor with a canvas panel attached.

The default posture is:

- the stylus stays in the dominant hand;
- offhand keyboard or tablet express input opens radial menus and temporary
  navigation modes;
- touch is disabled by default;
- the canvas remains the dominant surface;
- panels and toolbars are collapsible;
- frequent controls are reachable without forcing the user away from the
  drawing surface.

The central UX rule is:

```text
Pen contact draws immediately.
Explicit offhand input opens radial UI.
No hidden pen-contact delay may be added to distinguish drawing from menus.
```

## Current Repository Anchors

The first app shell and input path already exist:

- `apps/runenwerk_draw/src/app/input.rs` extracts pointer/stylus facts into
  drawing tool input DTOs. Its current `DrawingToolRouteKind` values are a
  narrow preview-stroke route model, not the final generic tool-session model.
- `apps/runenwerk_draw/src/app/state.rs` owns preview stroke routing, commit
  routing, and last visible drawing frame rebuilds.
- `apps/runenwerk_draw/src/runtime/systems.rs` bridges current mouse and touch
  runtime input into `ui_input` pointer events.
- `domain/ui/ui_input/src/pointer.rs` owns the platform-neutral stylus-capable
  pointer vocabulary.
- `adapters/native_tablet_input/src/lib.rs` owns the native tablet packet
  normalization proof.
- `domain/ui/ui_tree/src/tree/node.rs` already has a `RadialMenuNode`
  precedent in the retained UI tree model.

Those anchors are implementation facts, not the final UX contract.

## Current Implementation Gap

The current runtime path still routes winit touch samples as drawing input so
the app can exercise fallback pointer flow. That is not the desired default
product UX. Pen contact must continue to draw immediately with no hold delay,
while touch drawing enablement belongs to a future user profile/input policy and
should be disabled by default.

The rendering foundation baseline is DRF1 through DRF5: deterministic
preview/final CPU ink tile contracts, app-owned in-memory cache proof,
product-surface dynamic texture bridging, Draw-owned GPU validation through
public render-flow APIs, and per-generation GPU promotion/fallback are present.
CPU tile formation remains the correctness oracle. App cache, GPU validation,
visible product promotion, and CPU current or last-good fallback are app-owned
runtime state. Persistent cache storage, package IO, paper response,
watercolor, lasso, transform, fill, mask eraser compositing, radial menu
behavior, Workbench migration, and new GPU drawing behavior remain outside this
slice.

## Drawing Authority Boundaries

`DrawingDocument` in `domain/drawing` is the source authority for drawing
truth. Committed artwork mutations must enter through
`DrawingTransaction` and `DrawingCommand`; app preview state, cache state,
publication state, GPU validation state, and renderer output are all derived
runtime or execution state.

`apps/runenwerk_draw` owns the pen-first product workflow: tool input/session
routing, app cache payloads, dirty tile invalidation, product lifecycle,
visible product promotion, GPU validation records, and CPU current or last-good
fallback policy.

`engine/render` owns generic derived execution: render-flow invocation, dynamic
texture upload/allocation, capture/readback, texture diff, and UI composition.
Renderer/GPU output can feed app-owned promotion and fallback decisions, but it
must not become drawing truth.

`native_tablet_input` is normalization only. It turns native tablet facts into
`ui_input` pointer events and diagnostics. It must not own drawing semantics,
stroke commit policy, app tool state, or render products.

`ui_render_data` is projection vocabulary only. It carries neutral primitives
and product-surface bindings selected by the app; it does not own drawing
truth, product lifecycle policy, or renderer backend handles.

Workbench integration is deferred. It must not block standalone
`runenwerk_draw` drawing app work.

Canonical PlantUML source:
[draw-authority-and-product-flow.puml](../../apps/runenwerk-draw/diagrams/draw-authority-and-product-flow.puml).

## Tool-Session Boundary

The intended input and command pipeline is:

```text
PointerEvent / tablet packet
  -> DrawingToolInputEvent
  -> DrawingToolSession
  -> DrawingToolIntent
  -> DrawingCommand transaction or app-only navigation/selection action
  -> dirty tile invalidation / product lifecycle
```

`PointerEvent` and normalized tablet packets are platform-neutral input facts.
`DrawingToolInputEvent` is app-owned extraction of screen, canvas, source,
device, pressure, tilt, twist, eraser, barrel-button, coalesced, and predicted
sample facts. `DrawingToolSession` is the app-owned interaction
state machine for an active tool gesture. `DrawingToolIntent` should be the
tool-level semantic output: either a drawing-domain command transaction or an
app-only action such as navigation, selection preview, menu navigation, or
diagnostic UI state.

Only committed drawing mutations may produce `DrawingTransaction` and
`DrawingCommand` values. Document mutations then drive dirty tile invalidation
and the product lifecycle. App-only navigation or selection actions may rebuild
projection state without mutating `DrawingDocument`.

`DrawingToolRouteKind::BeginPreviewStroke`,
`DrawingToolRouteKind::UpdatePreviewStroke`, and
`DrawingToolRouteKind::EndPreviewStroke` are acceptable for the current simple
stroke path because that path has one contact lifecycle, one preview stroke,
one commit shape, and direct dirty tile invalidation. They must not become the
generic model for lasso, transform, fill, selection, eraser, masking, or radial
menu behavior.

Those future tools need different session contracts:

- lasso and selection need hit-testing, selection previews, cancel/confirm
  semantics, and often app-only state before any document mutation;
- transform needs handles, constraints, preview transforms, confirm/cancel, and
  document commands that are not stroke sample appends;
- fill needs region analysis, threshold/source policy, and product updates
  that are not preview stroke routes;
- eraser and masking need mask-aware document commands and product contracts,
  not visual deletion through stroke preview state;
- radial menus are explicit UI command surfaces and must be driven by offhand
  input/session state, not hidden pen-contact delays or preview stroke routing.

## Radial Menu Model

Radial menus are explicit command surfaces. They are not a replacement for
normal stroke input and must not introduce drawing latency.

Required behavior:

- barrel buttons are detected and remappable, but unassigned by default;
- a pen hold or hold-deadzone gesture must not open the radial menu;
- explicit offhand input opens radial menus, either through keyboard shortcuts
  or tablet express keys;
- once open, pen movement may select radial entries;
- command palette access is the fallback for users without convenient offhand
  input;
- radial menus open at the current pointer and clamp to screen or canvas bounds
  so entries remain reachable;
- adaptive user-defined slot counts are allowed, but profile validation must
  reject unreadable or unsafe hit sizes.

Selection should support both fast and careful use:

- quick express-key hold, move, and release selects an entry;
- longer open states may expose radial sliders, nested values, or tap-to-select
  controls;
- releasing or returning to a cancel zone must cancel without mutating document
  or profile state.

## Radial UX

Radial menus should be authored through generic UI definition concepts and
activated by a user drawing profile.

Ownership:

- `runenwerk_draw` user profile owns active radial menu choices, shortcut
  bindings, tablet mappings, and UX preferences;
- drawing documents own artwork only;
- generic UI definitions should describe radial menu structure where possible;
- the drawing app preferences/setup surface edits and activates those
  definitions for `runenwerk_draw`;
- profile changes and artwork changes use separate transaction histories.

The first radial set should include:

- a brush-context radial with brush family, eraser, size, opacity, flow, recent
  brushes, and color controls;
- radial arc sliders for size, opacity, and flow, with live preview and commit
  on release;
- a compact mini color wheel in the brush radial for direct color adjustment;
- a separate navigation radial for temporary pan, zoom, rotate, fit, and reset
  view actions.

The navigation radial selects temporary modes. It must not replace the active
brush unless the user explicitly switches tools.

## Tablet And Stylus UX

The UX contract is generic stylus first. Wacom and macOS are important adapter
targets, but they are not the authority for the product model.

The app should detect and expose the full stylus fact set when available:

- pressure;
- tilt;
- twist;
- tangential pressure;
- hover and contact state;
- eraser state;
- barrel buttons;
- calibration and cursor offset;
- raw, coalesced, and predicted samples.

Missing capabilities degrade gracefully:

- drawing continues with available facts;
- unsupported controls are visibly disabled;
- diagnostics explain missing capabilities without inventing values;
- default emulation must not pretend that real pressure, tilt, hover, or
  calibration exists.

Predicted samples are temporary preview data only. They may improve brush ghost
or live ink preview, but they must be replaced by raw or coalesced sample truth
before committed stroke state becomes authoritative.

The feedback target is within one render frame for:

- live preview ink;
- brush ghost;
- radial open, highlight, and slider feedback.

## Tablet Setup Surface

Tablet setup is a dedicated preferences/setup surface, not a mandatory
first-run wizard.

The setup surface should show:

- detected device and capability facts;
- pressure curve test area;
- hover/contact state;
- cursor offset and calibration state;
- sample behavior for raw, coalesced, and predicted packets;
- tilt, twist, tangential pressure, eraser, and button diagnostics;
- express-key and shortcut mapping state.

Barrel buttons remain remappable but unassigned by default. They should be
visible in setup diagnostics so users can choose mappings intentionally.

## Eraser UX

Immediate eraser UX is required, but the first correct target is a mask eraser,
not destructive raster deletion or stroke deletion.

Target behavior:

- eraser writes to an active layer mask;
- erasing respects the active selection or mask context;
- if no active selection or mask context exists, the app shows a non-blocking
  prompt or toast instead of silently erasing the whole layer;
- eraser strokes are undoable artwork transactions;
- eraser compositing and mask tile products must be owned by drawing-domain and
  product contracts, not by app-side visual tricks.

This is a future implementation requirement because current drawing ink tile
formation is built around visible ink strokes and does not yet own the complete
mask eraser product path.

## Visual Chrome

The default visual shape is canvas-first:

- central canvas dominates the window;
- toolbars and side panels can collapse;
- transient radial menus carry frequent brush and navigation controls;
- persistent panels remain available for discoverability and detailed editing;
- status overlays should stay compact and avoid covering the active drawing
  point.

When stylus hover exists, normal drawing should show a brush ghost:

- brush footprint;
- color and opacity hint;
- orientation from tilt or twist when available;
- degraded cursor feedback when hover or orientation facts are missing.

## Implementation Requirements

Future implementation must add or extend:

- drawing-domain support for active layer mask eraser products;
- user-profile-owned radial menu, shortcut, tablet mapping, and UX preference
  persistence;
- generic UI-definition support for editable radial menu definitions if the
  current definition model cannot express them;
- a `runenwerk_draw` preferences/setup surface for radial editing and tablet
  diagnostics;
- a runtime bridge from native tablet packets into `runenwerk_draw`, preserving
  the `ui_input` ownership boundary.

The app may route and present product UX, but it must not own drawing document
semantics, native tablet APIs, or renderer-private eraser compositing.

## Validation Plan

Documentation validation:

```text
python3 tools/docs/validate_docs.py
```

Later implementation should add focused tests for:

- radial open, select, cancel, and clamp behavior;
- editable radial profile validation;
- stylus missing-capability diagnostics and disabled unsupported controls;
- predicted-sample preview replacement by raw or coalesced samples;
- mask eraser command and product behavior;
- separate document undo and profile undo histories.

## Non-Goals

This design does not require:

- a Wacom-specific UX model;
- barrel-button default commands;
- pen-contact hold gestures;
- touch drawing by default;
- destructive raster erasing;
- saving radial menu choices inside drawing documents;
- moving native tablet integration into `domain/drawing`.
