---
title: Superseded UI Component Platform Ownership Realignment Design
description: Historical owner-first correction plan for Component Platform vocabulary; current landed owner splits live in implemented Runenwerk-local contracts.
status: superseded
owner: ui
layer: history
canonical: false
last_reviewed: 2026-09-13
replacement_docs:
  - ../implemented/ui-component-platform-layout-container-virtualization-design.md
  - ../implemented/ui-component-platform-accessibility-focus-inspection-design.md
  - ../implemented/ui-component-platform-theme-state-style-design.md
  - ../implemented/ui-component-platform-render-surface-output-design.md
  - ../../domain/ui/architecture.md
---

# Superseded UI Component Platform Ownership Realignment Design

## Status

Superseded historical correction record for `PT-UI-COMPONENT-PLATFORM-009A`.

The durable rule from this plan remains valid in the current Runenwerk-local
implementation: owning crates define reusable local vocabulary and `ui_controls`
defines per-control requirements and summaries that reference those owners.
Current implemented contracts and code/tests now own the landed split; this file
no longer authorizes future Component Platform phases or reusable-framework work.
Future reusable UI-framework semantics belong to standalone `dornglut/runen-ui`.

## Historical decision

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

## Historical rule

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

## Historical Phase 5-8 classification

The completed Phase 5-8 code was useful because it was declarative and read-only, but some vocabulary was in the wrong owner.

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

## Historical Phase 9 correction

The plan rejected implementing Phase 9 as a broad `ui_controls/src/layout.rs` vocabulary owner.

Its intended order was:

```text
009B Layout Foundation:
  add generic layout/container/scroll/virtualization vocabulary to ui_layout.

009C Control Layout Bridge:
  add ControlLayoutDescriptor in ui_controls referencing ui_layout types.
  add catalog inspection projection.
  add focused control-level tests.
```

That owner-first outcome is now represented by current implemented contracts and
source rather than this planning record.

## Historical migration strategy

The plan required controlled owner-first migration rather than a large rewrite:

```text
1. Add owner-crate vocabulary first.
2. Change ui_controls wrappers to reference owner-crate types.
3. Keep compatibility aliases where needed.
4. Keep catalog inspection summaries stable where possible.
5. Remove duplicated vocabulary only after tests prove compatibility.
```

## Supersession boundary

Use current implemented Component Platform documents and code/tests for Runenwerk
local truth. Do not reactivate the old numbered phase program from this file.
This record does not establish standalone RunenUI adoption and does not authorize
future reusable-framework targets inside Runenwerk.
