---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-26
---

# Decision Register

Use this file to explain planning priority changes.

## Initial cutover decision

Date: 2026-06-25

Decision: Runenwerk workspace planning becomes Markdown-first for scriptless workflow.

Reason: GitHub connector and context-tool workflows cannot rely on full repo export, command execution, generated prompts, or rendered planning views.

Affected files: `planning/README.md`, `active-work.md`, `roadmap.md`, `deferred-work.md`, `completed-work.md`, `production-tracks.md`.

Follow-up: Copy detailed legacy rows into Markdown planning records as they are touched.

## Phase 2 authoring-kit planning decision

Date: 2026-06-25

Decision: Start Phase 2 as an authoring-kit design/planning intake before implementation.

Reason: The authoring kit needed accepted owner, scope, non-goals, validation, and stop conditions before code.

Follow-up: Completed by user validation report.

## Phase 3 story-proof-envelope planning decision

Date: 2026-06-25

Decision: Start Phase 3 as a Story Proof Envelope design/planning intake before implementation.

Reason: Story proof had to consume existing `ui_story` V2 authority instead of creating a parallel control-specific proof model.

Follow-up: Completed by user validation report.

## Phase 4 catalog planning decision

Date: 2026-06-26

Decision: Start Phase 4 as a Catalog / Discovery / Inspection design intake before implementation.

Reason: Catalog facts had to remain derived read-only projections from ControlPackage descriptors and proof summaries.

Follow-up: Completed by user validation report.

## Phase 5 input planning decision

Date: 2026-06-26

Decision: Start Phase 5 as an Input / Gesture / Device design intake before implementation.

Reason: Input, gesture, and device facts had to remain declarative package facts while runtime input collection, routing, and mutation stayed outside the component platform.

Follow-up: Completed by user validation report.

## Phase 6 state planning decision

Date: 2026-06-26

Decision: Start Phase 6 as a State Binding / Host Intent design intake before implementation.

Reason: Reusable controls may describe state buckets, binding requirements, and host intent proposals. Actual app/editor/game mutation, route authorization, persistence, and domain-specific rules remain host-owned.

Follow-up: Completed by user validation report.

## Phase 7 theme planning decision

Date: 2026-06-26

Decision: Start Phase 7 as a Theme / State / Style design intake before implementation.

Reason: Theme, visual state, and style facts need reusable declarations before controls can share consistent presentation semantics without moving renderer or product styling ownership into ui_controls.

Follow-up: Completed by user validation report.

## Phase 8 accessibility planning decision

Date: 2026-06-26

Decision: Start Phase 8 as an Accessibility / Focus / Inspection design intake before implementation.

Reason: Accessibility roles, focus semantics, keyboard navigation, and inspection metadata need reusable declarations before layout, render, base controls, or interaction phases consume them.

Follow-up: Review and accept the Accessibility / Focus / Inspection design before code.

## Decision shape

```text
Date:
Decision:
Context:
Options considered:
Reason:
Affected planning files:
Evidence:
Follow-up:
```
