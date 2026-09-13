---
title: UI Component Platform Render Surface Output Design
description: Implemented owner-first renderer-neutral output evidence contract with ui_render_data ownership and a control-facing ui_controls bridge.
status: implemented
owner: ui_render_data
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../superseded/ui-component-platform-ownership-realignment-design.md
  - ./ui-component-platform-layout-container-virtualization-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Render Surface Output Design

## Status

Implemented in the owner-first shape. This document no longer describes Phase 10 as future implementation work.

## Implemented owner split

`ui_render_data` owns renderer-facing, backend-neutral output vocabulary such as frame/surface/layer/primitive families, ordering keys, product-surface transport, primitive-family evidence, and expected primitive-count evidence.

`ui_controls` owns the control-facing bridge through `ControlRenderDescriptor`, `ControlRenderCapabilitySummary`, and `ControlRenderInspectionFact`. That bridge references `ui_render_data` types and evidence ids; it does not define generic primitive/source-of-truth output semantics.

`ui_runtime` remains the owner of retained/runtime UI state to renderer-neutral frame formation. Engine render owns extraction, GPU/backend resources, materialization, submission, and draw behavior.

## Implemented contract

A control render descriptor can require owner-defined primitive families, record expected primitive counts, and associate render-evidence ids. Its derived summary is read-only and explicitly records `has_backend_render_behavior: false`.

Current implementation evidence includes:

```text
domain/ui/ui_render_data/src/frame/output_evidence.rs
domain/ui/ui_controls/src/render.rs
```

plus the corresponding base-control lowering/catalog/inspection and focused render-evidence tests.

## Intentional landed differences

The planning document proposed delivery labels `010A` through `010E`. The repository did not preserve those labels as separate public/runtime architecture. Instead the proven owner-first shape landed directly across the established owners. The semantic outcome is preserved:

```text
owner output vocabulary
  -> control requirement/summary
  -> read-only inspection
  -> runtime/story evidence
  -> backend consumption
```

The phase labels are historical decomposition, not current authority.

## Boundary rules

- `ui_controls` must not own generic primitive families, draw/sort semantics, texture binding, renderer diagnostics, or backend execution.
- `ui_render_data` is transport/evidence authority, not authored control source truth.
- Runtime/frame output does not become package/source authority.
- Backend renderer behavior remains outside this component contract.
- Render evidence does not automatically grant mount eligibility.

## Evidence

Current `ui_render_data` output-evidence contracts, `ui_controls` render bridge, base-control lowering/projection, and focused tests establish implementation parity with the owner-first design.
