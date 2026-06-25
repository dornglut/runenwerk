---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
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

Decision: Start `PT-UI-COMPONENT-PLATFORM` Phase 2 as an authoring-kit design/planning intake before implementation.

Context:

```text
Phase 1 is complete by user report and establishes the ControlPackage / ControlKernel contract. The next useful slice is authoring ergonomics, but implementation must be design-gated because the authoring kit could otherwise blur boundaries with story proof, catalog/discovery, Gallery, Designer, Workbench, or runtime behavior.
```

Options considered:

```text
1. Start authoring-kit Rust implementation immediately.
2. Treat Phase 2 as design/planning first, then authorize a bounded implementation pass.
3. Defer authoring-kit work until later component phases.
```

Reason:

```text
Option 2 follows AGENTS.md and the workspace routines: planning records and active design must define owner, scope, non-goals, validation, and stop conditions before implementation. It also preserves YAGNI and separation-of-concerns boundaries.
```

Affected planning files:

```text
docs-site/src/content/docs/design/active/ui-component-platform-authoring-kit-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Evidence:

```text
AGENTS.md, start-here.md, roadmap-update-routine.md, implementation-routine.md, authority-model.md, programming-principles.md, UI Component Platform roadmap, and Phase 1 control-kernel design were inspected before this planning update.
```

Follow-up:

```text
Review and accept the authoring-kit design. After acceptance, run a bounded implementation pass that adds only the authoring API and tests under ui_controls, preserving Phase 1 validation and avoiding Phase 3+ behavior.
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
