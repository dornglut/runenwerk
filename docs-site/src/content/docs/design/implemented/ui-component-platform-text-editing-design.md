---
title: UI Component Platform Text Editing Editable Text Behavior Design
description: Implemented Runenwerk-local package-backed editable-text declaration, runtime transaction proof, renderer-neutral frame projection, and static-mount contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ./ui-component-platform-generic-text-design.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-overlay-popup-layering-design.md
---

# UI Component Platform Text Editing Editable Text Behavior Design

## Status

Implemented through the completed Phase 14 editable-text delivery. It is a reusable behavior/proof contract, not a product document editor.

## Implemented flow

```text
ui_controls editable-text declarations
  -> base-control editable-text lowering
  -> ControlPackageDescriptor validation / catalog / inspection
  -> normalized text-intent/input facts
  -> ui_runtime::text_editing transaction/replay/report proof
  -> renderer-neutral proof frame
  -> ui_static_mount validation
```

The current package contract uses `ControlEditableTextDescriptor` and related support/inspection facts. InspectorField/base-control lowering attaches the package-backed declaration; runtime proof consumes package declarations and normalized input rather than inventing product text state.

## Ownership

- `ui_controls` owns reusable editable-text declarations, lowering, validation, catalog, and inspection projection.
- `ui_input` owns normalized text/input facts.
- `ui_runtime::text_editing` owns reusable edit-session/transaction proof and renderer-neutral report/frame evidence.
- `ui_text` owns lower-level text buffer/cursor/selection/layout primitives where applicable.
- hosts/products own product document persistence, command routing, rich/code editor behavior, product undo/redo integration, authored UI editing, and domain mutation.

## Implemented boundary

Editable-text behavior is deliberately narrower than a product editor. The reusable path may prove insertion/deletion, cursor/selection/edit intent, cancel/commit behavior, read-only suppression, and deterministic proof evidence without taking ownership of application documents or commands.

Generic Text remains the display/layout owner for renderer-neutral text presentation; editable text does not create a competing display-layout authority.

## Boundary rules

- No product/editor/game truth is mutated by reusable proof code.
- No rich-text or code-editor framework is implied.
- No authored UI editing, dynamic plugin framework, or shared meta framework is authorized.
- Backend rendering remains separate.
- Package support and proof evidence do not imply universal product mount policy.

## Evidence

Current `ui_controls/src/editable_text.rs`, base-control text-editing lowering, `ui_runtime/src/text_editing/`, package/runtime/static-mount tests, and the accepted Phase 14 delivery establish implementation parity.
