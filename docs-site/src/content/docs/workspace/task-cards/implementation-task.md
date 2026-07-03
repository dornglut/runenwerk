---
title: Implementation Task
description: Reusable task card for bounded implementation work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-design-gate.md
  - ../routines/implementation-routine.md
  - ../routines/phase-completion-drift-check-routine.md
---

# Implementation Task

Use this card for bounded implementation work.

Routine: `docs-site/src/content/docs/workspace/routines/implementation-routine.md`

Before editing, confirm the work is implementation-authorized by owner, complete implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, and stop conditions.

For architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work, also confirm the planning record points to complete design gate evidence:

```text
complete capability map
feature support matrix
future-use-case pressure matrix
hierarchy/composition matrix when relevant
ergonomics/usability contract
validation/evidence contract
```

Report changed files, exact modules or sections, complete design gate status, validation, risks, and lifecycle impact.

If the patch completes an active phase, include or explicitly schedule the phase closeout/planning handoff before the next implementation contract starts.
