---
title: UI Component Platform Accessibility Focus And Inspection Design
description: Implemented Runenwerk-local contract for reusable control accessibility, focus declarations, semantic states, diagnostics, and inspection summaries.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-component-platform-theme-state-style-design.md
  - ./ui-component-platform-state-binding-host-intent-design.md
  - ./ui-component-platform-input-gesture-device-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Accessibility Focus And Inspection Design

## Status

Implemented in the current Runenwerk-local UI stack. This lifecycle classification records existing code/test truth; it does not perform or authorize a standalone RunenUI consumer cutover.

## Implemented contract

`domain/ui/ui_controls/src/accessibility.rs` provides the control-facing accessibility declaration model used by reusable control packages and read-only inspection surfaces. The implemented model includes:

- `ControlAccessibilityRole`;
- label and description requirements;
- `ControlSemanticHint`;
- `ControlFocusRequirement`;
- `ControlKeyboardActivation`;
- `ControlSemanticState`;
- `ControlValueRangeMetadata`;
- accessibility diagnostics and diagnostic kinds;
- `ControlAccessibilityDescriptor` and derived capability/inspection summaries.

The implemented role vocabulary covers button, label, checkbox, slider, text, list/list item, tree/tree item, table/row/cell, menu/menu item, dialog, panel, canvas, and custom roles. Keyboard activation and semantic-state facts remain declarations rather than platform execution.

## Landed ownership

`ui_controls` owns the current Runenwerk-local per-control declaration and projection contract. Platform-native accessibility bridges, OS integration, final localized copy, product focus routing, persistence, and product mutation remain outside this contract.

The broader owner-first cleanup described by the still-active Component Platform ownership-realignment work is separate authority work. Moving this document to implemented does not claim that all generic accessibility vocabulary has already been extracted to a standalone framework owner.

## Intentional landed differences

The proposal named a separate `ControlFocusOrder` concept. Current code represents focus order directly as `ControlFocusRequirement::focus_order: Option<u32>`. That is the implemented normalized shape; no compatibility alias is required.

The proposal also used candidate module/file language. The current module and focused tests are the implementation authority:

```text
domain/ui/ui_controls/src/accessibility.rs
domain/ui/ui_controls/tests/control_accessibility_contract.rs
domain/ui/ui_controls/tests/control_accessibility_catalog_contract.rs
```

## Boundary rules

- Accessibility declarations are semantic package facts, not native accessibility-tree execution.
- Focus requirements do not own runtime focus policy.
- Keyboard activation facts do not execute product commands.
- Semantic state facts do not become domain validation truth.
- Catalog and inspection projections remain read-only.
- No declaration grants runtime mount eligibility by itself.
- Renderer, app, editor, game, OS, localization, and persistence ownership remain separate.

## Evidence

Current source and focused contract/catalog tests exercise the implemented declaration vocabulary and its read-only inspection projection. Historical phase-planning and delivery evidence remains available in Git history and retained reports; it is not current implementation authority.
