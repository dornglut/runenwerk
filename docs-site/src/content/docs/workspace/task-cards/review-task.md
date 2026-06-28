---
title: Review Task
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-28
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/pr-review-routine.md
  - ../routines/phase-completion-drift-check-routine.md
---

# Review Task

Use this card for architecture review, pull request review, phase closeout, or docs review.

Repository: `Crystonix/Runenwerk`

Routine:

```text
Choose the matching routine from docs-site/src/content/docs/workspace/start-here.md.
```

Authority files:

```text
AGENTS.md
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
TESTING.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/guidelines/programming-principles.md
owning design, ADR, routine, roadmap, or docs file
```

Review lens:

- KISS: is the path understandable without ceremony?
- DRY: is there one owner for each durable claim?
- YAGNI: is every new surface needed now?
- SOLID: are responsibilities and dependencies legal?
- Separation of Concerns: are code, docs, planning, reports, and tooling separated by purpose?
- Avoid Premature Optimization: is complexity evidence-driven?
- Law of Demeter: does the change use direct contracts instead of internals?
- Lifecycle consistency: does the patch truthfully update planning, closeout, and next-phase state when it completes or opens a phase?

For phase or production-track reviews, check whether the PR completes active work, opens new active planning, or requires a closeout before the next implementation slice.

Final report:

```text
Recommendation:
Files inspected:
Findings:
Validation evidence:
Lifecycle / closeout impact:
Risks:
Next action:
```
