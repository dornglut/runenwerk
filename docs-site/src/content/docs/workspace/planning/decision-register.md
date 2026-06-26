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

Context:

```text
Phase 5 is complete by user validation report. The next component-platform slice needs reusable declarations for state ownership, binding shape, editable lifecycle, validation state, host intent proposals, and route/capability decisions without moving host-owned truth or product mutation into ui_controls.
```

Reason:

```text
Reusable controls may describe state buckets, binding requirements, and host intent proposals. Actual app/editor/game mutation, route authorization, persistence, and domain-specific rules remain host-owned.
```

Affected files:

```text
docs-site/src/content/docs/design/active/ui-component-platform-state-binding-host-intent-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Follow-up:

```text
Review and accept the State Binding / Host Intent design before code.
```

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
