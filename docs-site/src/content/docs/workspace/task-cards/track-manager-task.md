---
title: Track Manager Task
description: Reusable task card for managing a production track through bounded phase PRs.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../start-here.md
  - ../routines/track-orchestration-routine.md
  - ../workflow-lifecycle.md
  - ../complete-merge-readiness-gate.md
  - ../planning/README.md
---

# Track Manager Task

Use this card for manager-style Codex/agent work that owns a full production-track goal but must execute it through bounded phase PRs.

Routine: `docs-site/src/content/docs/workspace/routines/track-orchestration-routine.md`

Rules:

- Start at `docs-site/src/content/docs/workspace/start-here.md`.
- Follow `track-orchestration-routine.md`; this task card is reusable wording only and does not own process.
- Manage the production-track goal, phase order, planning truth, PR readiness, closeout sequencing, and next-phase activation.
- Do not collapse a production track into one implementation PR.
- Give each implementation agent exactly one phase.
- Do not activate implementation unless the planning record separately authorizes exact scope, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, and stop conditions.
- Use PR review and merge readiness before merge recommendations.
- Use phase-completion drift check before the next implementation phase starts.

Final report:

```text
Track:
Current phase:
Current branch/PR:
Lifecycle state:
Planning files inspected:
Authority files inspected:
Evidence classes used:
Complete investigation gate status:
Complete design gate status:
Merge-readiness status:
Closeout state:
Phase spec status:
Validation run or unavailable:
Implementation authorization:
Next safe action:
```
