---
title: Pull Request Review Routine
description: Scriptless routine for reviewing proposed repository patches.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../authority-model.md
  - ./phase-completion-drift-check-routine.md
  - ../../guidelines/programming-principles.md
---

# Pull Request Review Routine

## Use when

Use this routine to review a proposed patch.

## Authority files to read

Read changed files, owning docs, tests, `AGENTS.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `workflow-lifecycle.md`, `complete-investigation-gate.md`, `complete-design-gate.md`, and `programming-principles.md`.

For production-track or phase PRs, also read `active-work.md`, `roadmap.md`, `production-tracks.md`, and relevant closeout reports.

## Working files to inspect

Inspect changed files, call sites, tests, docs, affected public APIs, affected planning records, complete investigation evidence, and complete design gate evidence when applicable.

For phase-completing PRs, inspect whether completion evidence and next-phase state are both represented truthfully.

## What to decide before editing

Decide whether the patch is acceptable, needs changes, or needs more evidence.

Also decide whether the PR changes lifecycle state, completes an active phase, opens a next phase, or leaves planning drift that must be fixed before merge or before the next implementation starts.

For architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work, decide whether design/planning/implementation was authorized by complete investigation evidence and complete design gate evidence before code was written.

## State transitions produced

This routine does not normally change lifecycle state by itself. It may recommend acceptance, changes, closeout work, deferral, or supersession.

## Patch rules

Tie every finding to an exact file and function, module, or section.

For phase PRs, treat lifecycle, complete investigation evidence, complete design gate evidence, and planning consistency as part of correctness, not as optional documentation polish.

## Manual validation checklist

Check correctness, owner fit, dependency direction, investigation evidence, API usability, ergonomics, user-facing terminology, safe defaults, feature support matrix, future-use-case pressure matrix, hierarchy/composition matrix, docs impact, validation evidence, lifecycle impact, phase completion or cutover impact, known gaps, and the seven principles.

For phase-completing PRs, check that one of these is true:

```text
completion/planning updates are included
completion/planning updates are intentionally split into a named follow-up before next implementation
PR does not claim phase completion
```

For work requiring `complete-investigation-gate.md` or `complete-design-gate.md`, check that implementation does not proceed from accepted direction alone.

## Stop conditions

Stop and request changes or a follow-up closeout if the PR completes a phase but leaves active work, roadmap, production track, completed work, or decision-register state misleading.

Stop and request redesign if the PR treats accepted direction as implementation authorization, hides validation gaps, lacks required complete investigation evidence, lacks required complete design gate evidence, moves product behavior into the wrong owner, or starts the next implementation before completion truth is recorded.

## Evidence to report

Report recommendation, findings, files inspected, complete investigation gate status where applicable, complete design gate status where applicable, validation evidence, lifecycle impact, phase cutover status, risks, and next action.

## Optional local helpers

Run focused tests, docs validation, or diff tools when available.
