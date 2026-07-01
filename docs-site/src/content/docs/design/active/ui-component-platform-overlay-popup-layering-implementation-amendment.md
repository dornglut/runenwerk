---
title: UI Component Platform Overlay Popup Layering Implementation Amendment
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-01
related_docs:
  - ./ui-component-platform-overlay-popup-layering-design.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/decision-register.md
---

# UI Component Platform Overlay Popup Layering Implementation Amendment

Lifecycle state: `active-implementation-amendment`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-013`.

This amendment corrects the implementation file placement from the Phase 13 design. It keeps the same owner boundaries, non-goals, proof requirements, validation gate, and stop conditions.

## Correction

The original file list placed overlay runtime files below `domain/ui/ui_runtime/src/input/`. That placement was wrong.

Overlay runtime consumes normalized input facts, but it is not input ownership. Overlay runtime owns overlay intent, session and stack state, placement, layer, focus, dismissal, pointer capture, keyboard evidence, report rows, proof-frame projection, and no-bypass assertions.

## Correct runtime module shape

Use a runtime-level overlay module:

```text
domain/ui/ui_runtime/src/overlay.rs
```

A later refactor may split that file into an `overlay/` module tree if the file becomes too large.

Do not place overlay runtime semantics below `domain/ui/ui_runtime/src/input/`.

## Corrected implementation file list

```text
domain/ui/ui_controls/src/overlay.rs
domain/ui/ui_controls/src/lib.rs

domain/ui/ui_input/tests/overlay_normalized_facts.rs

domain/ui/ui_runtime/src/lib.rs
domain/ui/ui_runtime/src/overlay.rs
domain/ui/ui_runtime/tests/overlay_layering_report.rs
domain/ui/ui_runtime/tests/executable_overlay_layering_story.rs

domain/ui/ui_static_mount/tests/base_controls_overlay_layering_static_mount.rs

domain/ui/ui_story/tests/executable_overlay_layering_workflow.rs
```

Package, catalog, lowering, and editor proof-host files remain future expansion unless review explicitly accepts adding them to Phase 13 after local validation.

## Preserved owner boundaries

- `ui_controls` owns overlay declarations and ergonomic descriptor builders only.
- `ui_input` owns normalized facts only.
- `ui_runtime` owns overlay behavior proof, replay/report state, and proof-frame projection.
- `ui_static_mount` validates renderer-neutral frames only.
- `ui_story` owns workflow/evidence envelope expectations only.
- Product/editor/game layers own commands, mutation, route authorization, persistence, authored editing, and app-specific modal lifecycle.

## Stop condition

Stop and redesign if implementation requires overlay behavior under `ui_runtime::input`, command execution in generic UI, product/editor/game mutation in generic UI, app-specific modal lifecycle, product gallery/designer surfaces, authored UI editing, full text editing, external plugin framework work, `foundation/meta`, shared plugin primitives, or Workbench/provider redesign.
