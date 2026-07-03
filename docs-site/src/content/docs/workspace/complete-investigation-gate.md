---
title: Complete Investigation Gate
description: Mandatory investigation gate before Runenwerk design, planning, or implementation decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./start-here.md
  - ./workflow-lifecycle.md
  - ./complete-design-gate.md
  - ./authority-model.md
  - ./operating-model.md
  - ./planning/README.md
  - ./routines/investigation-routine.md
  - ./routines/architecture-governance-review-routine.md
  - ./routines/roadmap-update-routine.md
  - ../guidelines/programming-principles.md
---

# Complete Investigation Gate

## Purpose

This document defines the mandatory investigation gate for Runenwerk work.

Use this gate before a design, planning record, or implementation task makes architecture-sensitive, reusable, platform, public API, production-track, workflow, domain-boundary, dependency, or ownership decisions.

The gate exists to make decisions extensive, evidence-based, and explicit before design or implementation starts.

## Core rule

No design, planning authorization, or implementation may proceed from an uninformed premise.

```text
Complete investigation first.
Complete design second.
Complete planning contract third.
Implementation fourth.
Completion only after the declared contract is proven.
```

A task may move quickly only when the investigation proves that the work is local, behavior-preserving, and already covered by accepted authority. If investigation reveals ownership uncertainty, public API impact, capability pressure, dependency risk, workflow authority changes, or stale evidence, the task remains investigation until the dossier is complete.

## Required when

Apply this gate whenever work may touch any of these:

```text
public API
reusable platform capability
domain boundary
durable vocabulary
workflow authority
production-track phase
app composition
host integration
renderer-neutral contract
input behavior
accessibility behavior
inspection/catalog/report surface
cross-domain dependency
new crate or shared extraction
phase closeout truth
stale planning record
conflicting design or ADR
unknown owner
unclear validation evidence
```

Investigation is also required when a user asks for a critical review, roadmap/design intake, generic framework direction, phase planning, branch/PR readiness review, or cleanup that may change authority.

## Investigation output

A complete investigation produces a dossier. The dossier may live in a design intake section, architecture review note, issue/PR body, planning note, or report, but it must be readable from repository files or the final review report.

The dossier must name the next gate:

```text
complete design gate
planning update
implementation routine
PR review
phase completion drift check
defer/reject/supersede
```

## Complete investigation checklist

An investigation is complete only when it records all applicable items below.

```text
Question:
  decision being investigated
  reason investigation is required
  expected downstream gate

Lifecycle:
  current lifecycle state
  candidate state transition
  whether implementation is currently forbidden

Authority:
  root authority files inspected
  workspace authority files inspected
  owning design/ADR/domain docs inspected
  planning records inspected
  closeout/report evidence inspected
  code/tests/fixtures inspected
  reference-only files identified

Current reality:
  current implementation behavior
  current docs/planning claims
  current tests/proofs/evidence
  current public API and usage paths
  current known gaps

Ownership:
  current owner
  candidate owner when current owner is wrong or unclear
  participating crates/domains
  forbidden owners
  dependency direction
  boundary risks

Vocabulary:
  current durable names
  candidate durable names
  user-facing/facade names
  compatibility names
  names that must not enter public API

Capability inventory:
  existing capabilities
  missing capabilities
  claimed capabilities without evidence
  capabilities that belong to named downstream owners
  capabilities that require complete design gate

Ergonomics and usability pressure:
  current authoring path
  current inspection/debug path
  user-facing friction
  safe-default gaps
  recovery/diagnostic gaps
  accessibility/input gaps

Future-use pressure:
  future consumers affected by the decision
  needs those consumers place on the current contract
  responsibilities those consumers must own themselves
  scope-leak risks

Alternatives:
  viable options
  rejected options
  tradeoffs
  long-term consequences
  selected recommendation

Evidence quality:
  direct evidence
  inferred evidence
  missing evidence
  confidence level per key finding
  validation that cannot be run in connector mode

Risks and blockers:
  blockers before design
  blockers before planning
  blockers before implementation
  risks if work proceeds without more evidence

Recommendation:
  recommended next gate
  required files/docs to update next
  explicit no-action/defer/reject/supersede recommendation when appropriate
```

If any required item is unknown, report it as a blocker or uncertainty. Do not convert unknowns into design assumptions.

## Authority/source matrix

Use this matrix for authority-sensitive work.

```text
| Claim | Source inspected | Authority level | Evidence found | Conflict / drift |
|---|---|---|---|---|
| <claim> | <file/test/report> | <code/test/design/planning/report/generated> | <evidence> | <conflict or none> |
```

This matrix prevents decisions from being based on stale summaries or memory.

## Current-state matrix

Use this matrix when docs, code, tests, or planning may disagree.

```text
| Area | Current code reality | Current docs reality | Tests/proofs | Gap |
|---|---|---|---|---|
| <area> | <observed code> | <observed docs> | <evidence> | <gap> |
```

The design gate must consume this matrix instead of starting from stale docs.

## Owner/dependency matrix

Use this matrix when ownership or dependency direction matters.

```text
| Responsibility | Current owner | Candidate owner | Dependencies | Boundary risk |
|---|---|---|---|---|
| <responsibility> | <owner> | <owner> | <deps> | <risk> |
```

This matrix prevents broad shared extraction, framework drift, and product behavior entering the wrong domain.

## Capability inventory matrix

Use this matrix for platform, public API, workflow, reusable, or production-track work.

```text
| Capability | Exists now | Evidence | Missing contract | Required owner |
|---|---|---|---|---|
| <capability> | <yes/no/claimed> | <file/test/report> | <missing contract> | <owner> |
```

This matrix feeds the complete design gate's feature support matrix.

## Alternatives and tradeoff matrix

Use this matrix when there is more than one viable design or planning path.

```text
| Option | Benefits | Costs | Boundary impact | Long-term fit | Recommendation |
|---|---|---|---|---|---|
| <option> | <benefits> | <costs> | <impact> | <fit> | <accept/reject> |
```

A recommendation without alternatives is acceptable only when investigation proves no realistic alternative exists.

## Confidence matrix

Use this matrix for key findings.

```text
| Finding | Confidence | Reason | Missing evidence to improve confidence |
|---|---|---|---|
| <finding> | <high/medium/low> | <why> | <evidence needed> |
```

Confidence must reflect inspected evidence, not tone. Low-confidence findings cannot authorize design, planning, or implementation by themselves.

## Promotion rule

Investigation may promote to proposed design only when:

```text
authority files were inspected
working files were inspected
current reality is recorded
owner/dependency/vocabulary impact is known
capability inventory is recorded when applicable
alternatives are recorded or ruled out
confidence and missing evidence are recorded
next gate is named
```

Investigation may promote directly to implementation only when the dossier proves all of these:

```text
work is local
behavior is already accepted
owner is clear
dependency direction is unchanged
public API is unchanged
no reusable/platform/domain-boundary concern exists
validation path is known
no complete design gate is required
```

Otherwise, the next gate is complete design or planning, not implementation.

## Stop conditions

Stop before design, planning, or implementation if investigation finds:

```text
unknown owner
conflicting authority
stale planning truth
missing closeout evidence
missing validation evidence
uninspected affected files
unclear public API impact
unclear domain boundary
unclear dependency direction
unresolved vocabulary conflict
unrecorded alternative with meaningful tradeoff
low-confidence finding needed for the decision
complete design gate required but not prepared
```

## Reporting requirement

Final reports for work using this gate must include:

```text
Complete investigation gate status:
Authority/source matrix status:
Current-state matrix status:
Owner/dependency matrix status:
Capability inventory status:
Alternatives/tradeoff status:
Confidence status:
Recommended next gate:
Remaining blockers:
```
