---
title: UI Component Platform State Binding And Host Intent Design
description: Implemented Runenwerk-local per-control state, binding, edit-lifecycle, validation-state, host-intent, and route-decision declaration contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-input-gesture-device-design.md
  - ./ui-component-platform-theme-state-style-design.md
  - ./ui-component-platform-catalog-discovery-inspection-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform State Binding And Host Intent Design

## Status

Implemented as the current Runenwerk-local control-facing state/binding/intent declaration model.

## Implemented contract

Current `domain/ui/ui_controls/src/state.rs` and base-control lowering provide:

- `ControlStateBucket` and bucket requirements;
- `ControlStateBindingKind` and `ControlStateBindingRequirement`;
- `ControlStateDescriptor`;
- `ControlEditLifecycle`;
- `ControlValidationState`;
- `ControlHostIntentProposal`;
- `ControlRouteCapabilityDecision`;
- derived capability and inspection summaries.

The implemented vocabulary distinguishes transient, preview, committed, focus, hover, drag, animation, host-fed, and package-owned state; read/write/collection/option/selection binding shapes; reusable validation-state facts; live/commit/cancel/rollback edit lifecycle; and host-intent proposals plus host-supplied route/capability decisions.

## Ownership

`ui_controls` owns per-control declarations and read-only projection. Route identity/schema/capability vocabulary remains owned by the program/routing contracts. Apps, editor, game hosts, and domain owners retain actual data truth, authorization, command execution, persistence, domain validation, and mutation.

The still-active ownership-realignment work may further normalize generic vocabulary ownership without invalidating the current control-facing descriptor contract.

## Boundary rules

- State buckets are ownership declarations, not storage engines.
- Bindings describe shape/requirements, not a hidden live data pipe.
- Host intent proposals are not executed commands.
- Route decisions are host-supplied facts, not component authorization.
- Validation-state facts do not replace domain validation truth.
- No declaration makes a control runtime-mount eligible.

## Evidence

Current state source, base-control lowering, `control_state_contract.rs`, `control_state_catalog_contract.rs`, and package/catalog integration establish the implemented contract and its no-host-mutation boundary.
