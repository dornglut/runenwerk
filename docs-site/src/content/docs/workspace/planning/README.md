---
title: Planning Records
description: Markdown-first planning records for scriptless Runenwerk workflow.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../authority-model.md
  - ../workflow-lifecycle.md
  - ../complete-design-gate.md
  - ../routines/roadmap-update-routine.md
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

A planning change must be understandable by reading Markdown. Do not require generated views, rendered diagrams, YAML parsing, Taskfile tasks, or local scripts to know the current state.

Legacy YAML and generated Markdown may remain as migration context or optional mirrors. If they disagree with these planning records, update the Markdown planning record first and report the mirror drift.

## Lifecycle rule

Use [`../workflow-lifecycle.md`](../workflow-lifecycle.md) when a planning change changes state.

Use [`../complete-design-gate.md`](../complete-design-gate.md) before planning authorizes implementation for architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work.

Common state transitions:

```text
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

Architecture acceptance does not authorize implementation. Active implementation requires exact owner, complete implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, stop conditions, and complete design gate evidence where applicable.

## Update checklist

- Active work has one clear current focus or an explicit no-current-focus state.
- Roadmap entries name state, owner, authority, evidence, known gaps, and next action.
- Active implementation entries point to complete design gate evidence where applicable.
- Active implementation entries name the complete implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, and stop conditions.
- Reusable/platform/public API entries include or link feature support, future-use-case pressure, hierarchy/composition where relevant, and ergonomics/usability evidence.
- Deferred work names the reason and reactivation condition.
- Completed work links evidence and remains a short index.
- Production tracks name strategic order, track type, gates, activation condition, and current blocker.
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
Complete design gate:
Implementation contract:
Allowed files/crates:
Forbidden files/crates:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Evidence:
Validation:
Known gaps:
Stop conditions:
Next action:
```

Specific files may add fields such as `Track type`, `Validation`, `Reason deferred`, `Reactivation condition`, `Completed on`, or `State transition`.
