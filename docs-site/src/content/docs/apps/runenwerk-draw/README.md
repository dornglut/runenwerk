---
title: Runenwerk Draw
description: Current architecture overview for the focused Runenwerk drawing app shell.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-10
related_designs:
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../../design/active/drawing-domain-crate-design.md
---

# Runenwerk Draw

`apps/runenwerk_draw` is the focused drawing application shell. It reuses the
shared engine runtime, UI frame contracts, render submission registry, and pure
`domain/drawing` contracts while keeping drawing product composition outside
`apps/runenwerk_editor`.

## Entry Points

- `apps/runenwerk_draw/src/main.rs`: binary entry point.
- `apps/runenwerk_draw/src/lib.rs`: public app crate surface.
- `apps/runenwerk_draw/src/runtime/app.rs`: engine app construction and render
  flow registration.
- `apps/runenwerk_draw/src/runtime/plugin.rs`: drawing app runtime plugin.
- `apps/runenwerk_draw/src/app/state.rs`: app-owned drawing shell state and
  input routing facade.

## Current Behavior

The Phase 4 shell starts independently, opens a ratified minimal
`DrawingDocument`, projects a canvas-first workspace, submits a simple UI frame
through the shared render UI pipeline, and routes pointer/stylus-compatible
`ui_input` events into preview stroke state.

The preview stroke is app-shell state only. It does not commit authored stroke
truth into `DrawingDocument`; deterministic ink tile formation and committed
stroke commands remain later drawing phases.

## Ownership Boundary

`runenwerk_draw` owns product-level wiring, canvas-first workspace setup, app
state, runtime plugin registration, and shell-level input routing. It should not
own drawing document semantics, graph ratification, brush/paper contracts,
native tablet APIs, renderer-private tile formation, package IO, or export
adapters.

Those belong to their owning crates:

- drawing truth and ratification: `domain/drawing`;
- platform-neutral input vocabulary: `domain/ui/ui_input`;
- native tablet packet normalization: `adapters/native_tablet_input`;
- render execution and UI frame composition: `engine`;
- later package and export contracts: future focused designs.
