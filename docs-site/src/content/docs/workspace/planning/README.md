---
title: Planning Records
description: Markdown-first planning records for scriptless Runenwerk workflow.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../authority-model.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
  - ../routines/roadmap-update-routine.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../guidelines/programming-principles.md
---

# Planning Records

Planning is Markdown-first from the scriptless workflow cutover onward.

Use these files:

- [Active Work](active-work.md)
- [Roadmap](roadmap.md)
- [Deferred Work](deferred-work.md)
- [Completed Work](completed-work.md)
- [Production Tracks](production-tracks.md)
- [Decision Register](decision-register.md)

## Rule

A planning change must be understandable by reading Markdown. Do not require generated views, rendered diagrams, YAML parsing, Taskfile tasks, local scripts, or phase specs to know the current state.

Legacy YAML and generated Markdown may remain as migration context or optional mirrors. If they disagree with these planning records, update the Markdown planning record first and report the mirror drift.

Phase implementation specs may be used as compact handoff contracts for one phase, but they derive from accepted Markdown authority and must not replace planning records.

## Lifecycle rule

Use [`../workflow-lifecycle.md`](../workflow-lifecycle.md) when a planning change changes state.

Use [`../complete-investigation-gate.md`](../complete-investigation-gate.md) before planning makes design, activation, or implementation decisions when current reality, ownership, authority, alternatives, evidence, or confidence is not already proven.

Use [`../complete-design-gate.md`](../complete-design-gate.md) before planning authorizes implementation for architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work.

Use [`../../guidelines/programming-principles.md`](../../guidelines/programming-principles.md) when planning activates non-trivial implementation, reusable platform work, public API changes, production-track work, workflow authority changes, or phase completion.

Use [`../evidence-quality-taxonomy.md`](../evidence-quality-taxonomy.md) when planning records depend on validation, current behavior, confidence, or freshness claims.

Use [`../complete-merge-readiness-gate.md`](../complete-merge-readiness-gate.md) before planning marks a PR/branch/phase as merge-ready or completed by merge.

Use [`../routines/track-orchestration-routine.md`](../routines/track-orchestration-routine.md) when one goal owns a production track but implementation must proceed through bounded phase PRs.

Common state transitions:

```text
idea -> investigating
investigating -> proposed-design
proposed-design -> accepted-direction
accepted-direction -> track-candidate
track-candidate -> production-track
production-track -> active-planning
active-planning -> active-implementation
active-implementation -> review
review -> completed
active-planning -> deferred
accepted-direction -> superseded
```

Architecture acceptance does not authorize implementation. Active implementation requires exact owner, complete implementation contract, allowed files/crates, forbidden files/crates, principle compliance matrix, module decomposition map, maintainability review status, validation envelope, evidence expectation, stop conditions, and complete investigation/design gate evidence where applicable.

A track manager may own strategic sequencing, but each implementation phase still needs separate active-implementation authorization.

## Track orchestration planning rule

A one-shot track goal is valid only as manager intent.

```text
One goal may own the whole production track.
Implementation still proceeds through bounded phase PRs.
One implementation agent receives exactly one phase.
```

Track orchestration planning must keep these claims explicit:

```text
current track
current phase
current lifecycle state
current branch/PR when applicable
previous phase closeout state
next phase activation condition
implementation authorization status
```

Do not mark the next phase active implementation until the previous phase has truthful completion or explicitly scheduled closeout and the next phase has exact implementation authorization.

## Phase spec planning rule

Use [`../specs/phase-implementation-spec.md`](../specs/phase-implementation-spec.md) when a phase needs a compact handoff contract.

A phase spec may support planning and implementation handoff. It does not replace:

```text
active-work truth
roadmap state
production-track order
decision-register transitions
complete investigation gate evidence
complete design gate evidence
merge-readiness gate
phase closeout truth
```

Use RON for phase specs. Use JSONL for append-only event/log/trace streams.

## Update checklist

- Active work has one clear current focus or an explicit no-current-focus state.
- Roadmap entries name state, owner, authority, evidence, known gaps, and next action.
- Planning decisions point to complete investigation gate evidence where applicable.
- Active implementation entries point to complete design gate evidence where applicable.
- Active implementation entries name the complete implementation contract, allowed files/crates, forbidden files/crates, principle compliance matrix, module decomposition map, maintainability review status, validation envelope, evidence expectation, and stop conditions.
- Reusable/platform/public API entries include or link feature support, future-use-case pressure, hierarchy/composition where relevant, ergonomics/usability evidence, principle compliance evidence, and module decomposition evidence.
- Merge/completion entries name evidence classes, principle compliance status, maintainability/decomposition status, validation status, merge readiness status when applicable, and branch cleanup impact when applicable.
- Deferred work names the reason and reactivation condition.
- Completed work links evidence and remains a short index.
- Production tracks name strategic order, track type, gates, activation condition, and current blocker.
- Track orchestration entries name current phase, closeout state, merge-readiness state, and next safe action.
- Phase specs derive from accepted docs and do not become parallel authority.
- Decision register explains priority changes and records state transitions where relevant.
- Closeout detail goes under `../../reports/closeouts/` when it would bloat planning records.

## Common entry fields

Use these fields across planning records where applicable:

```text
ID:
Title:
State:
Owner:
Authority:
Evidence classes:
Complete investigation gate:
Complete design gate:
Merge readiness:
Implementation contract:
Allowed files/crates:
Forbidden files/crates:
Principle compliance matrix:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Phase spec:
Evidence:
Validation:
Known gaps:
Stop conditions:
Next action:
```

Specific files may add fields such as `Track type`, `Validation`, `Reason deferred`, `Reactivation condition`, `Completed on`, `State transition`, `Branch cleanup`, `Merge readiness`, `Programming-principle compliance`, `Maintainability/decomposition`, `Current phase`, `Closeout state`, `Implementation authorization`, or `Next phase activation condition`.
