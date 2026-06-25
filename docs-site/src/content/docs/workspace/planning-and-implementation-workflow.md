---
title: Planning and Implementation Workflow
description: Scriptless router for planning, implementation, routines, task cards, validation, and closeout in Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./authority-model.md
  - ./routines/README.md
  - ./task-cards/README.md
  - ./planning/README.md
---

# Planning and Implementation Workflow

This page is a compatibility router for the new scriptless workflow.

Start here instead:

```text
docs-site/src/content/docs/workspace/start-here.md
```

## Core rule

Runenwerk planning and implementation workflows must be usable from file inspection alone.

Do not require a Taskfile command, generated prompt, rendered planning view, shell script, full repository export, or local command execution to decide the next action.

## Normal workflow

```text
1. Choose a task shape in start-here.md.
2. Read the selected routine.
3. Read the authority files named by that routine.
4. Inspect the exact files to change.
5. Patch only the owned scope.
6. Use the routine's manual validation checklist.
7. Report local command validation as run, skipped, or unavailable.
8. Close out with changed files, exact sections/modules, risks, and next step.
```

## Active workflow surfaces

- [`start-here.md`](start-here.md): daily router.
- [`operating-model.md`](operating-model.md): scriptless operating doctrine.
- [`authority-model.md`](authority-model.md): authority conflict rules.
- [`documentation-structure.md`](documentation-structure.md): doc placement and lifecycle rules.
- [`routines/README.md`](routines/README.md): repeatable procedures.
- [`task-cards/README.md`](task-cards/README.md): copy-paste task cards.
- [`planning/README.md`](planning/README.md): Markdown-first planning records.

## Implementation work

Use:

```text
docs-site/src/content/docs/workspace/routines/implementation-routine.md
```

The routine requires authority review, scoped patching, manual validation, and an evidence report. Commands are optional helpers only.

## Architecture-sensitive work

Use:

```text
docs-site/src/content/docs/workspace/routines/architecture-governance-review-routine.md
```

Use an ADR or design update when a durable architecture decision changes.

## Documentation work

Use:

```text
docs-site/src/content/docs/workspace/routines/docs-refactor-routine.md
```

Root docs summarize only. Canonical details live under docs-site.

## Planning work

Use:

```text
docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md
docs-site/src/content/docs/workspace/planning/README.md
```

Planning is Markdown-first from the scriptless workflow cutover onward. Legacy YAML and generated Markdown may remain as optional mirrors or migration context.

## Closeout work

Use:

```text
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
```

A closeout must state changed files, exact sections/modules, evidence inspected, validation performed or unavailable, known gaps, and the next safe action.

## Optional local helpers

A local checkout may provide additional validation evidence, for example formatting, focused tests, workspace tests, or docs validation. These helpers do not own workflow authority and must not be the only way to understand the task.
