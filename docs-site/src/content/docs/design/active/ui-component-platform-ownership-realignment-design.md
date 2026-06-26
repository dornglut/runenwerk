---
title: UI Component Platform Ownership Realignment Design
description: Corrects the UI Component Platform rule so owning crates define reusable vocabulary and ui_controls exposes control-facing requirements.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./ui-component-platform-layout-container-virtualization-design.md
  - ./ui-component-platform-accessibility-focus-inspection-design.md
  - ./ui-component-platform-theme-state-style-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Ownership Realignment Design

## Status

This is the planning and acceptance design for `PT-UI-COMPONENT-PLATFORM-009A`.

It is a correction pass before Phase 9 implementation. It stops the pattern where generic UI vocabulary is added directly to `ui_controls` just because `ui_controls` may legally depend on the owning crate.

## Decision

Owning crates define reusable UI vocabulary and contracts. `ui_controls` defines per-control requirements and summaries that reference those contracts.

```text
ui_input owns generic input vocabulary.
ui_state owns generic state vocabulary.
ui_binding owns host/data binding vocabulary.
ui_theme owns generic theme/style vocabulary.
ui_accessibility and ui_program own accessibility graph/proof vocabulary.
ui_layout owns generic layout/container/scroll/virtualization vocabulary.
ui_controls owns control packages, control kinds, and per-control requirements.
catalog/inspection exposes read-only summaries.
runtime, renderer, apps, editor, and game execute behavior later.
```

## Rule

`ui_controls` must not become the source of truth for generic UI concepts when an owning crate exists.

Allowed in `ui_controls`:

```text
ControlXDescriptor
ControlXRequirement
ControlXCapabilitySummary
ControlXInspectionFact
per-control requirement wrappers
compatibility aliases during migration
catalog/inspection projection
```

Not allowed in `ui_controls` as source of truth:

```text
generic input modes
generic state buckets
generic binding kinds
generic theme token kinds
generic visual states
generic accessibility roles
generic focus semantics
generic layout roles
generic container kinds
generic virtualization facts
generic renderer facts
generic runtime behavior
```

## Phase 5-8 classification

The completed Phase 5-8 code is useful because it is declarative and read-only, but some vocabulary is in the wrong owner.

```text
Phase 5 Input / Gesture / Device:
  move or reuse generic vocabulary from ui_input;
  keep ControlInputDescriptor as per-control requirements.

Phase 6 State Binding / Host Intent:
  move or reuse state vocabulary from ui_state;
  move or reuse binding vocabulary from ui_binding;
  keep ControlStateDescriptor as per-control requirements.

Phase 7 Theme / State / Style:
  move or reuse token/style vocabulary from ui_theme;
  keep ControlThemeDescriptor as per-control requirements.

Phase 8 Accessibility / Focus / Inspection:
  move or reuse accessibility/focus vocabulary from ui_accessibility and ui_program;
  keep ControlAccessibilityDescriptor as per-control requirements.
```

## Phase 9 correction

Do not implement Phase 9 as a broad `ui_controls/src/layout.rs` vocabulary owner.

Correct implementation order:

```text
009B Layout Foundation:
  add generic layout/container/scroll/virtualization vocabulary to ui_layout.

009C Control Layout Bridge:
  add ControlLayoutDescriptor in ui_controls referencing ui_layout types.
  add catalog inspection projection.
  add focused control-level tests.
```

## Migration strategy

Do not break the existing green phases in one large rewrite.

Use controlled migrations:

```text
1. Add owner-crate vocabulary first.
2. Change ui_controls wrappers to reference owner-crate types.
3. Keep compatibility aliases where needed.
4. Keep catalog inspection summaries stable where possible.
5. Remove duplicated vocabulary only after tests prove compatibility.
```

## Acceptance criteria

This correction pass is complete when:

- the ownership rule is recorded in a canonical active design;
- the active roadmap no longer asks Phase 9 to implement layout vocabulary directly in `ui_controls`;
- Phase 9 is split into owner-first layout foundation and control bridge follow-up;
- completed Phase 5-8 work is classified as useful but needing later owner-crate vocabulary migration;
- no Rust migration is attempted in this planning pass.
