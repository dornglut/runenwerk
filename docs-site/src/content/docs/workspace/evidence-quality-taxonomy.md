---
title: Evidence Quality Taxonomy
description: Shared evidence classes and reporting rules for Runenwerk workflow decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./start-here.md
  - ./workflow-lifecycle.md
  - ./complete-investigation-gate.md
  - ./complete-design-gate.md
  - ./complete-merge-readiness-gate.md
  - ./authority-model.md
  - ./operating-model.md
  - ./routines/pr-review-routine.md
  - ./routines/phase-completion-drift-check-routine.md
  - ../guidelines/programming-principles.md
---

# Evidence Quality Taxonomy

## Purpose

This document defines how Runenwerk classifies evidence.

Use it when investigation, design, planning, implementation review, merge readiness, or closeout depends on claims about code behavior, docs state, validation, ownership, or lifecycle truth.

The goal is to stop vague claims such as “validated,” “looks good,” “probably complete,” or “docs say so” from authorizing decisions without naming what was actually inspected.

## Core rule

Every important claim must name its evidence class.

```text
Evidence class first.
Claim second.
Confidence third.
Decision only after the evidence supports the decision.
```

Do not present inference, stale generated output, memory, or unchecked user reports as direct validation.

## Evidence classes

Use these classes in reports, reviews, planning records, and closeouts.

```text
E0 unsupported claim
  No inspected source supports the claim.
  Cannot authorize investigation closure, design, planning, implementation, merge, or closeout.

E1 memory or prior conversation context
  Useful for orientation only.
  Must be confirmed against repository files or current external authority before decisions.

E2 connector file inspection
  Repository file contents, PR metadata, branch metadata, or diff metadata inspected through connector tools.
  Good for authority, scope, and text review.
  Does not prove local command validation.

E3 source-code/test inspection
  Code, tests, fixtures, captures, or generated proof inputs inspected by path.
  Good for current behavior analysis.
  Does not prove the inspected tests pass unless command output exists.

E4 user-reported validation
  User reports command output, PR status, or local results.
  Acceptable only when recorded as user-reported and concrete.
  Prefer exact command names and result summaries.

E5 local command validation
  Actual command output is available from the current environment.
  Strong evidence for the command's checked scope.
  Does not prove untested scopes.

E6 CI or hosted check evidence
  GitHub status checks, workflow jobs, or review gates inspected from current PR/commit.
  Strong evidence for configured CI scope.
  Must name commit, job/check, and status.

E7 runtime/proof artifact evidence
  Generated reports, captures, proof frames, snapshots, or artifacts inspected as evidence.
  Strong only when freshness and source commit are known.

E8 accepted authority
  Accepted ADR, accepted design, root architecture doc, workspace authority doc, or decision-register entry.
  Strong for policy and direction.
  Does not override current code behavior without migration decision.

E9 current code/test plus validation plus authority alignment
  Code/test inspection, validation output, and authoritative docs agree.
  Strongest normal evidence for merge/closeout.
```

## Confidence levels

Use confidence separately from evidence class.

```text
High
  Direct current evidence supports the claim and no known conflicting authority exists.

Medium
  Evidence supports the claim, but validation, scope, freshness, or conflict review is incomplete.

Low
  Claim is inferred, partially inspected, stale, or blocked by missing evidence.

Blocked
  Claim is needed for a decision but evidence is missing or conflicting.
```

Low-confidence or blocked findings may guide investigation. They must not authorize design acceptance, active implementation, merge, or closeout.

## Evidence matrix

Use this matrix in investigations, PR reviews, merge readiness, and closeouts when claims matter.

```text
| Claim | Evidence class | Source / command / artifact | Freshness | Confidence | Decision impact |
|---|---|---|---|---|---|
| <claim> | <E0-E9> | <file, command, PR check, report> | <current/stale/unknown> | <high/medium/low/blocked> | <authorizes/blocks/informs> |
```

## Validation reporting rules

When reporting validation, use precise wording.

```text
Allowed:
  Command validation run: cargo test -p ui_runtime surface2d passed locally in this session.
  Command validation unavailable: GitHub connector cannot execute commands.
  User-reported validation: user reports cargo test --workspace passed after commit <sha>.
  CI evidence: PR check <name> passed on commit <sha>.
  Manual inspection: reviewed files <paths>; no command validation claimed.

Forbidden:
  validated
  should pass
  looks green
  seems fine
  tested by inspection
  CI probably passed
```

A final report may say “manual validation performed” only for checklist/file inspection. It must separately state command validation and CI validation.

## Freshness rules

Evidence must name freshness when relevant.

```text
current
  tied to the current branch, PR, or commit being reviewed

stale
  tied to older commit, older branch, old planning state, or prior phase

unknown
  source did not expose commit/date/state, or freshness was not checked
```

Stale evidence may explain history. It does not authorize current merge or closeout without current confirmation.

## Conflict rules

When evidence conflicts:

```text
code/test behavior beats stale docs for current reality
accepted design/ADR beats informal notes for intended direction
workspace authority beats task cards for workflow process
planning Markdown beats generated planning mirrors unless a contract says otherwise
current CI/local validation beats older validation reports for current branch readiness
```

If current behavior and accepted direction disagree, do not silently choose one. Record drift and choose the appropriate next gate: investigation, design update, implementation fix, defer, reject, or supersede.

## Decision thresholds

Use these thresholds unless an owning design requires stricter proof.

```text
Investigation closure
  Requires E2/E3/E8 for authority/current reality and confidence recorded.

Design acceptance
  Requires investigation evidence plus E8-level authority update or accepted design record.

Active implementation
  Requires accepted design/planning authority and enough current evidence to define exact scope.

Merge readiness
  Requires current diff evidence, required validation evidence, lifecycle/planning evidence, and no blocked merge-critical findings.

Closeout
  Requires delivered contract evidence, validation status, planning state update, known gaps, and closeout evidence where needed.
```

## Final-report requirement

Any workflow using this taxonomy must report:

```text
Evidence classes used:
Highest evidence class reached:
Command validation:
CI validation:
User-reported validation:
Manual inspection:
Stale or unknown evidence:
Blocked claims:
Decision supported:
```
