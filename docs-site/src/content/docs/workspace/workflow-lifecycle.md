---
title: Workflow Lifecycle
description: Lifecycle model for moving Runenwerk work from idea to investigation, design, decision, planning, implementation, review, and closeout.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ./start-here.md
  - ./operating-model.md
  - ./authority-model.md
  - ./documentation-structure.md
  - ./planning/README.md
  - ./routines/architecture-governance-review-routine.md
  - ./routines/roadmap-update-routine.md
  - ./routines/phase-completion-drift-check-routine.md
  - ../guidelines/programming-principles.md
---

# Workflow Lifecycle

## Purpose

This document defines how Runenwerk work moves from idea to investigation, design, decision, planning, implementation, review, and closeout.

It exists to keep architecture-heavy work scriptless, explicit, and reviewable without adding public-governance overhead.

## Core rule

Architecture acceptance is not implementation authorization.

```text
Accepted direction
  means the target shape is approved.

Active implementation
  requires an exact owner, scope, validation envelope, stop conditions, and active planning entry.
```

## Artifact jobs

Every artifact has one primary job.

| Artifact | Owns | Must not own |
|---|---|---|
| Root summary | Short entrypoint and links | Long design, roadmap, historical evidence |
| Guideline | Stable doctrine and engineering rules | Current task status |
| Design | Target architecture, vocabulary, boundaries, tradeoffs | Active task status |
| ADR / decision record | Accepted decision and rationale | Full design exploration |
| Production track | Strategic multi-phase sequence and gates | Detailed implementation manual |
| Roadmap entry | Milestone state and next work | Deep architecture rationale |
| Active work | One current focus | Long historical archive |
| Deferred work | Postponed work and reactivation condition | Completed work |
| Completed work | Short completion index | Full closeout detail |
| Closeout report | Historical evidence | Current planning authority |
| Routine | Repeatable procedure | Track-specific decision authority |
| Task card | Reusable prompt or handoff wording | Process authority |
| Generated file | Mirror, evidence, or narrow contract | Default human workflow authority |

## Lifecycle states

Use these states for non-trivial work.

```text
idea
  raw concept, not authority

investigating
  authority files and working files are being inspected

proposed-design
  design exists, not accepted

accepted-direction
  architecture direction accepted, implementation not authorized

track-candidate
  may become a production track after proof or priority decision

production-track
  strategic multi-phase sequence exists

active-planning
  current planning focus, no implementation yet

active-implementation
  exact implementation scope authorized

review
  PR, patch, or doc change exists and validation/review is pending

completed
  evidence exists and completion is recorded

deferred
  valid but inactive; reactivation condition required

rejected
  intentionally not pursued

superseded
  replaced by a named newer artifact

archived
  retained as historical, not authority
```

## Promotion rules

### Idea to investigation

Promote when ownership, current state, risk, or expected change scope is unclear.

Use the investigation routine. Do not patch during pure investigation unless explicitly requested.

### Investigation to proposed design

Promote when the change may affect ownership, dependency direction, durable vocabulary, migration shape, host behavior, app composition, or multiple future implementation slices.

Output: a proposed design document or review recommendation.

### Proposed design to accepted direction

Promote only when the design records:

```text
owner boundaries
vocabulary
non-goals
tradeoffs
acceptance criteria
implementation gate
stop conditions
relationship to current work
```

Output: decision-register entry with a state transition.

### Accepted direction to production track

Promote only when the work spans multiple phases, has strategic priority, needs evidence gates, and competes for roadmap sequencing.

Do not create a production track for every accepted design.

### Production track to active planning

Promote only when the track or milestone is the selected current planning focus or a named planning dependency.

Output: active-work entry.

### Active planning to active implementation

Promote only when exact implementation scope, owner files/crates, validation envelope, and stop conditions are known.

### Active implementation to completed

Promote only when delivered scope, validation status, evidence, known gaps, and follow-up are recorded.

Detailed evidence belongs in a closeout report when the completion record would become too large.

## Design and roadmap placement

A design may contain a short implementation outline.

A companion roadmap may live next to a proposed design while the design is under review.

Once accepted, operational sequencing must move into workspace planning if it governs actual work:

```text
workspace/planning/production-tracks.md
workspace/planning/roadmap.md
workspace/planning/active-work.md
workspace/planning/decision-register.md
```

Keep the long design as architecture direction. Keep planning files as work-state authority.

## Generated file classes

Generated files have one of three classes.

```text
mirror
  convenience copy of Markdown authority

evidence
  generated proof/report from code, tests, validators, or captures

contract
  machine-readable authority only when explicitly authorized by an accepted design
```

Generated files are not default workflow authority. They become authoritative only when a narrow accepted design names them as a contract.

## Decision register transition field

Decision-register entries for workflow-relevant changes should include:

```text
State transition:
```

Examples:

```text
proposed-design -> accepted-direction
accepted-direction -> production-track
production-track -> active-planning
active-planning -> active-implementation
active-implementation -> completed
active-planning -> deferred
accepted-direction -> superseded
```

## Long design minimum shape

Every long design should include:

```text
Status
Decision summary
Problem
Current reality
Goals
Non-goals
Vocabulary
Owner map
Target architecture
Workflows / use cases
Alternatives considered
Risks
Acceptance criteria
Implementation gate
Stop conditions
Relationship to current work
```

The decision summary should fit on the first screen.

## Closeout placement

`completed-work.md` remains a short index.

Detailed evidence belongs under:

```text
docs-site/src/content/docs/reports/closeouts/
```

A closeout report should include:

```text
ID
Title
Completed on
Owner
Scope promised
Scope delivered
Files changed
Validation run
Validation unavailable
Known gaps
Drift found
Follow-up
Evidence links
```

## Stop conditions

Stop and redesign the workflow patch if it requires:

```text
root docs becoming full workflow manuals
local scripts to understand active work
generated views becoming default authority
multiple current active focuses without explicit reason
accepted direction being treated as implementation authorization
planning files duplicating full design rationale
completed-work.md becoming a detailed report archive
```

## Implementation note

This lifecycle document governs future workflow organization. Existing records may be migrated as they are touched unless a specific cleanup phase says otherwise.
