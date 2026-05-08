---
title: Workspace Routines
description: Repeatable maintenance routines for Runenwerk documentation, refactors, and contributor workflows.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-08
---

# Workspace Routines

This folder contains bounded maintenance routines for humans and AI coding agents.

Use routines when a task needs repeated inspection, patching, validation, and repair.

## Available Routines

- [Code Refactor Routine](./code-refactor-routine.md)
- [Commit Splitting Routine](./commit-splitting-routine.md)
- [Crate Implementation Routine](./crate-implementation-routine.md)
- [Documentation Refactor Routine](./docs-refactor-routine.md)
- [Phase Completion Drift Check Routine](./phase-completion-drift-check-routine.md)
- [Public API Review Routine](./public-api-review-routine.md)

## Routine Rules

- Routines are bounded.
- Routines must have explicit stop conditions.
- Routines must identify validation commands.
- Routines must not use unbounded loops.
- Routines must preserve unrelated work.
- Routines must report what was changed, skipped, blocked, or left for follow-up.

## Related Docs

- [`../planning-and-implementation-workflow.md`](../planning-and-implementation-workflow.md)
- [`../prompt-templates/README.md`](../prompt-templates/README.md)
- [`../agents.md`](../agents.md)
- [`../documentation-structure.md`](../documentation-structure.md)
