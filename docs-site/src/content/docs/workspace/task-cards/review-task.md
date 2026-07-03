---
title: Review Task
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
  - ../routines/pr-review-routine.md
  - ../routines/phase-completion-drift-check-routine.md
---

# Review Task

Use this card for architecture review, pull request review, phase closeout, docs review, or merge-readiness review.

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
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md
docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md
docs-site/src/content/docs/guidelines/programming-principles.md
owning design, ADR, routine, roadmap, or docs file
```

Review lens:

- KISS: is the path understandable without ceremony?
- DRY: is there one owner for each durable claim?
- YAGNI: is every new surface needed by the declared contract?
- SOLID: are responsibilities and dependencies legal?
- Separation of Concerns: are code, docs, planning, reports, and tooling separated by purpose?
- Avoid Premature Optimization: is complexity evidence-driven?
- Law of Demeter: does the change use direct contracts instead of internals?
- Investigation gate: are current reality, authority, alternatives, confidence, and blockers recorded before later decisions?
- Design gate: does required work have a complete capability map, support matrix, future-use-case pressure matrix, hierarchy/composition matrix where relevant, and ergonomics/usability contract?
- Evidence quality: are validation, confidence, freshness, and user-reported claims classified correctly?
- Merge readiness: are scope, validation, lifecycle truth, branch state, branch cleanup, and post-merge truth known?
- Lifecycle consistency: does the patch truthfully update planning, closeout, and next-phase state when it completes or opens a phase?

For phase or production-track reviews, check whether the PR completes active work, opens new active planning, or requires a closeout before the next implementation contract.

Final report:

```text
Recommendation:
Files inspected:
Findings:
Evidence classes used:
Complete investigation gate status:
Complete design gate status:
Merge readiness status:
Validation evidence:
Lifecycle / closeout impact:
Risks:
Next action:
```
