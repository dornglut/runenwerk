---
title: Implementation Batch Prompt
description: Prompt template for bounded implementation work in Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../routines/code-refactor-routine.md
  - ../routines/crate-implementation-routine.md
---

# Implementation Batch Prompt

Use this template for a bounded implementation task.

Generated implementation prompts should include the canonical quality doctrine
marker `runenwerk-quality-doctrine-v1`; use the canonical doctrine instead of
duplicating long-term quality language in local prompt copies.

## Template

```text
Implement this Runenwerk change:

Task:
- <task>

Scope:
- <crate/files/subsystem>

Requirements:
1. Inspect existing code before editing.
2. Reuse existing abstractions where appropriate.
3. Preserve domain boundaries and dependency direction.
4. Implement the smallest coherent change.
5. Add or update tests for changed invariants.
6. Update docs when public behavior, architecture, routines, or usage changes.
7. Run the smallest relevant validation commands.

Output after implementation:
1. What changed.
2. Files and exact functions/modules changed.
3. Why the change belongs there.
4. Tests/validation run.
5. Remaining risks or follow-up tasks.
```

## Stop Conditions

Stop and report instead of continuing when:

- ownership is unclear;
- a required dependency would violate layer direction;
- validation fails for a reason unrelated to the change;
- the task expands beyond the requested scope.
