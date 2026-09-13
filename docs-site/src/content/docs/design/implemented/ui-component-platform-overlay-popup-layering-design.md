---
title: UI Component Platform Overlay Popup Layering Design
description: Implemented Runenwerk-local overlay, popup, dropdown, tooltip, focus-containing, placement, layering, dismissal, proof, and no-bypass contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ../../reports/closeouts/pt-ui-component-platform-012a-executable-interaction-story-implementation-scope-closeout.md
  - ./ui-component-platform-text-editing-design.md
---

# UI Component Platform Overlay Popup Layering Design

## Status

Implemented through the completed Phase 13 overlay/layering slice. This is the current owner-boundary reference, not an active implementation plan.

## Implemented proof chain

```text
ui_controls overlay declarations
  -> base-control overlay lowering
  -> ControlPackageDescriptor overlay descriptors
  -> package validation / catalog / inspection
  -> ui_input normalized facts
  -> ui_runtime::overlay replay, stack, placement, focus, dismissal and suppression proof
  -> overlay visual proof / renderer-neutral UiFrame
  -> ui_static_mount validation
```

The implemented package vocabulary covers overlay kind, trigger, placement, layer, dismissal, focus policy, support summaries, package validation, catalog projection, and inspection projection.

Runtime proof covers package-backed fixture formation, open intents, stack ordering, topmost dismissal, outside-pointer behavior, focus containment/restoration evidence, keyboard/pointer-capture cases, viewport/anchor recomputation, suppression, deterministic replay, and proof-frame formation.

## Ownership

- `ui_controls` owns reusable overlay declarations, lowering, package validation, catalog, and inspection.
- `ui_input` owns normalized input facts only.
- `ui_runtime::overlay` owns renderer-neutral runtime stack/placement/focus/dismissal/suppression evidence and proof formation.
- `ui_static_mount` owns static renderer-neutral frame validation.
- app/editor/game/product owners retain command execution, product state mutation, authored editing, app-specific modal lifecycle, persistence, and product policy.

## Boundary rules

Overlay support must not become product command execution, product/editor/game mutation, raw device ownership, authored UI editing, text-edit transaction ownership, backend rendering, or a shared plugin/meta-framework.

The implementation keeps overlay runtime semantics under `ui_runtime::overlay`; it is not a compatibility path under generic input modules.

## Evidence

Current overlay source/tests under `ui_controls`, `ui_runtime`, `ui_input`, and `ui_static_mount` establish package-backed declaration, runtime proof, static proof, and no-bypass behavior. Historical Phase 13 delivery command lists remain provenance rather than current validation doctrine.
