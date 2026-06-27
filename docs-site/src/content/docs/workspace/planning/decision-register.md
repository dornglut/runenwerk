---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ../workflow-lifecycle.md
---

# Decision Register

Use this file to explain planning priority changes and lifecycle state transitions.

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

Follow-up: Completed by user validation report.

## Phase 9A ownership realignment decision

Date: 2026-06-26

Decision: Insert an ownership realignment pass before Phase 9 implementation.

Reason: Completed Phases 5-8 stayed declarative but duplicated some generic UI vocabulary in ui_controls instead of anchoring that vocabulary in owner crates.

Follow-up: Accept the realignment design, then implement layout foundation in ui_layout before adding the ui_controls bridge.

## Phase 9 closeout decision

Date: 2026-06-26

Decision: Mark `PT-UI-COMPONENT-PLATFORM-009` Layout / Container / Virtualization complete.

Context: PR #29 merged the corrected owner-first Phase 9 work into `main`. PR #30 is closed unmerged and superseded.

Options considered: Keep Phase 9 pending local validation; close Phase 9 based on user validation report; reopen the stale pre-realignment branch.

Reason: User reported the Phase 9 validation gate green after the merged 009A/009B/009C work. The remote repository shows the owner-first implementation on `main`, and PR #30 explicitly records the stale branch as superseded.

Affected planning files: `completed-work.md`, `roadmap.md`, `production-tracks.md`, `active-work.md`.

Evidence: 009A ownership realignment design, 009B `ui_layout` layout foundation, 009C `ui_controls` layout bridge over `ui_layout`, read-only catalog inspection bridge, focused tests, and user validation report.

Follow-up: Open Phase 10 Render Surface / Output planning. Do not revive PR #30 or `feature/ui-component-platform-009-layout`.

## Phase 10 render surface / output planning decision

Date: 2026-06-26

Decision: Start `PT-UI-COMPONENT-PLATFORM-010-PLANNING` as an owner-first Render Surface / Output design intake before implementation.

Context: Phase 10 needs reusable render/output evidence without repeating the Phase 9 ownership mistake.

Options considered: Put generic render/output vocabulary in `ui_controls`; use `ui_render_data` as renderer-facing output owner with `ui_runtime` and engine render as adjacent execution owners; defer Phase 10 entirely.

Reason: Repository authority and code inspection place renderer-facing `UiFrame`, surface, layer, primitive, product surface, and viewport embed contracts in `ui_render_data`; retained output generation in `ui_runtime`; and backend rendering execution in `engine/src/plugins/render`. `ui_controls` should only expose per-control render evidence requirements and summaries that reference owner contracts.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-render-surface-output-design.md`.

Evidence: `DOMAIN_MAP.md`, crate inventory, UI architecture docs, `ui_render_data` primitive exports, `ui_runtime::build_ui_frame`, `ui_controls` existing render evidence requirement fields, and Phase 9 ownership realignment rule.

Follow-up: Accept the Phase 10 design, then implement owner-first slices. No Rust implementation is authorized by this planning pass.

## Phase 10 closeout decision

Date: 2026-06-26

Decision: Mark `PT-UI-COMPONENT-PLATFORM-010` Render Surface / Output complete.

Context: PR #34 merged the full owner-first Phase 10 implementation into `main`.

Options considered: Keep P10 pending local validation; close P10 from PR #34 plus user validation report; split P10 into further render/runtime/backend subtasks.

Reason: User reported the full P10 validation gate green after PR #34 merged. The merged implementation covers `ui_render_data`, `ui_controls`, `ui_runtime`, and engine render proof while preserving owner boundaries.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `completed-work.md`, `decision-register.md`.

Evidence: PR #34, renderer-neutral output evidence, control render bridge, runtime output evidence generation, engine render submission proof, and user validation report.

Follow-up: Open Phase 11 Base Control Packages design intake.

## Phase 11 base control packages planning decision

Date: 2026-06-26

Decision: Start `PT-UI-COMPONENT-PLATFORM-011-PLANNING` as a Base Control Packages design intake before implementation.

Context: The component platform now has descriptor, proof, catalog, layout, accessibility, and render/output evidence layers. The base control package needs hardening before Gallery, Workbench, or UI Designer should rely on it as reusable product-facing inventory.

Options considered: Start full interaction immediately; harden base control packages first; skip to overlay/text/canvas phases.

Reason: Full interaction needs credible base control descriptors to operate on. Phase 11 should make Label, Button, InspectorField, ColorPicker, ActionPrompt, ListView, TreeView, and TableView package-quality without taking over Phase 12 interaction behavior.

Affected planning files: `active-work.md`, `roadmap.md`, `production-tracks.md`, `decision-register.md`, `ui-component-platform-base-control-packages-design.md`.

Evidence: Completed Phases 1-10 and the existing base control modules in `ui_controls`.

Follow-up: Review the Phase 11 design, then implement package hardening. Full interaction remains Phase 12.

## Lifecycle rule

Use `../workflow-lifecycle.md` for state transitions. New entries should include `State transition` when the decision changes lifecycle state.

## Decision shape

```text
Date:
Decision:
State transition:
Context:
Options considered:
Reason:
Affected planning files:
Evidence:
Follow-up:
Reactivation condition:
Supersedes:
Superseded by:
```

Existing entries may be migrated as they are touched.
