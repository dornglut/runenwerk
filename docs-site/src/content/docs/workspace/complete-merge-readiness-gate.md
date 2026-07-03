---
title: Complete Merge Readiness Gate
description: Mandatory merge-readiness gate for Runenwerk PRs, phase slices, and branch cleanup decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./start-here.md
  - ./workflow-lifecycle.md
  - ./evidence-quality-taxonomy.md
  - ./complete-investigation-gate.md
  - ./complete-design-gate.md
  - ./authority-model.md
  - ./planning/README.md
  - ./routines/pr-review-routine.md
  - ./routines/phase-completion-drift-check-routine.md
  - ../guidelines/programming-principles.md
---

# Complete Merge Readiness Gate

## Purpose

This document defines the mandatory merge-readiness gate for Runenwerk changes.

Use it before merging a PR, branch, phase slice, docs workflow change, or production-track implementation branch.

The gate exists to prevent merges that are technically clean but workflow-incomplete, validation-unclear, lifecycle-drifted, branch-cleanup unsafe, principle-noncompliant, or maintainability-degraded.

## Core rule

A branch is merge-ready only when code/docs scope, validation, lifecycle, evidence, programming-principle compliance, maintainability, and post-merge state are all known.

```text
Complete investigation where required.
Complete design where required.
Complete implementation contract.
Complete principle compliance check.
Complete maintainability/decomposition check.
Complete validation report.
Complete lifecycle/planning truth.
Complete post-merge cleanup plan.
Then merge.
```

Do not merge from “looks good,” partial review, stale validation, uninspected planning state, assumed branch cleanup, “we can split it later,” or generic principle claims without evidence.

## Required when

Apply this gate before merging any of these:

```text
production code PR
docs workflow PR
architecture/design PR
phase implementation PR
phase closeout PR
behavior-preserving refactor branch
branch-stack consolidation
PR that updates planning state
PR that completes active work
PR that opens next active planning
PR that changes workflow authority
```

For tiny typo-only docs fixes, record that the merge-readiness gate is not required and why.

## Merge readiness checklist

A merge is ready only when all applicable items below are recorded.

```text
Scope:
  branch name
  base branch
  head commit
  changed files
  intended contract
  actual diff scope
  unrelated changes absent or named

Authority:
  owning routine
  owning design/ADR/planning record
  complete investigation gate status where required
  complete design gate status where required
  evidence taxonomy status

Programming principles:
  KISS status
  DRY status
  YAGNI status
  SOLID status
  Separation of Concerns status
  Avoid Premature Optimization status
  Law of Demeter status
  unresolved principle findings absent or owned

Maintainability:
  module decomposition map status
  single-responsibility file/module review
  public re-export/API shape review
  large or compound file justification if any
  split follow-up forbidden unless it has an accepted owner and activation condition

Validation:
  command validation run
  command validation unavailable
  CI validation status
  user-reported validation if used
  manual inspection performed
  validation gaps

Lifecycle:
  current lifecycle state
  state transition produced by merge
  active work impact
  roadmap impact
  production-track impact
  decision-register impact
  completed-work impact
  closeout report impact

Review:
  PR review status
  unresolved findings
  known risks
  blocked claims
  accepted follow-up if any

Merge mechanics:
  branch behind/ahead status
  conflict status where known
  expected merge target
  squash/merge/rebase decision when relevant
  branch deletion plan
  stacked branch or remote leftover plan

Post-merge truth:
  main branch expected state
  follow-up closeout if needed
  next phase state
  next implementation authorization status
```

If any required item is unknown, the PR is not merge-ready. It remains review, investigation, closeout, or planning work.

## Merge readiness matrix

Use this matrix for PR reviews and merge decisions.

```text
| Area | Required evidence | Status | Blocker | Owner / next action |
|---|---|---|---|---|
| Scope | changed files + intended contract | <ready/blocked/unknown> | <blocker> | <owner/action> |
| Principles | KISS/DRY/YAGNI/SOLID/SoC/Optimization/Demeter evidence | <ready/blocked/unknown> | <blocker> | <owner/action> |
| Maintainability | decomposition map + responsibility review | <ready/blocked/unknown> | <blocker> | <owner/action> |
| Validation | commands/CI/manual evidence | <ready/blocked/unknown> | <blocker> | <owner/action> |
| Lifecycle | planning and closeout truth | <ready/blocked/unknown> | <blocker> | <owner/action> |
| Evidence | evidence classes and confidence | <ready/blocked/unknown> | <blocker> | <owner/action> |
| Merge mechanics | branch state and cleanup plan | <ready/blocked/unknown> | <blocker> | <owner/action> |
```

## Phase merge rules

For phase or production-track PRs, merge readiness also requires:

```text
promised contract vs delivered contract checked
phase completion claim checked
programming-principle compliance checked
maintainability/decomposition checked
validation envelope checked
known gaps recorded
planning state update included or named as pre-next-implementation closeout
decision-register transition checked when lifecycle changes
next phase not moved to implementation unless separately authorized
```

A phase implementation may merge without closeout only when the follow-up closeout is explicitly named and no next implementation starts before that closeout.

## Branch cleanup rules

Before recommending branch deletion, check:

```text
PR merged or branch intentionally abandoned
branch head reachable from main or no longer needed
no stacked branch depends on it
no open PR targets it
remote leftover branch plan recorded
local branch cleanup cannot be claimed through connector unless user reports it
```

Do not assume branch deletion happened unless the tool/user reports it.

## Evidence requirement

Use [`evidence-quality-taxonomy.md`](evidence-quality-taxonomy.md) when reporting merge readiness.

A merge-ready report must distinguish:

```text
manual inspection
connector diff/metadata inspection
local command validation
CI validation
user-reported validation
stale evidence
blocked claims
```

## Stop conditions

Stop before merge if any of these are true:

```text
changed files do not match intended contract
validation is required but unavailable and not accepted as a known risk
CI status is failing or unknown for required checks
planning state would become misleading
phase completion is claimed without evidence
next implementation would start before closeout truth
complete investigation or design gate evidence is missing where required
programming-principle compliance evidence is missing for non-trivial work
maintainability/decomposition evidence is missing for compound work
branch state, target, or cleanup plan is unclear
unresolved PR findings affect correctness, ownership, lifecycle, programming principles, maintainability, or validation
```

## Reporting requirement

Final merge-readiness reports must include:

```text
Merge readiness status:
Scope status:
Evidence taxonomy status:
Complete investigation gate status:
Complete design gate status:
Programming-principle compliance status:
Maintainability/decomposition status:
Validation status:
Lifecycle/planning status:
Known blockers:
Branch cleanup plan:
Merge recommendation:
Next action:
```
