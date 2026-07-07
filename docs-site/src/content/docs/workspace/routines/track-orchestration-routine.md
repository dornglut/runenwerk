---
title: Track Orchestration Routine
description: Scriptless routine for managing a production-track goal through bounded phase PRs.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../start-here.md
  - ../operating-model.md
  - ../authority-model.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
  - ./implementation-routine.md
  - ./pr-review-routine.md
  - ./phase-completion-drift-check-routine.md
  - ./roadmap-update-routine.md
  - ../planning/README.md
  - ../planning/active-work.md
  - ../planning/roadmap.md
  - ../planning/production-tracks.md
  - ../planning/decision-register.md
  - ../specs/phase-implementation-spec.md
---

# Track Orchestration Routine

## Use when

Use this routine when one goal owns a whole production track or a long multi-phase milestone, but implementation must proceed through bounded phase PRs.

Use it for:

```text
production-track execution management
multi-phase implementation sequencing
phase PR ordering
phase readiness checks
phase closeout sequencing
manager-style Codex or agent handoff
```

Do not use this routine to implement code directly. Use `implementation-routine.md` for the single phase that is currently authorized.

## Core rule

```text
A track manager may own the full production-track goal, phase order, active-work truth, roadmap/production-track consistency, PR readiness, closeout sequencing, and next-phase activation.

A track manager must not collapse a production track into one implementation PR.

Each implementation agent receives exactly one phase.
```

A track goal may be one-shot at the management level. The patch series must still be phase-bounded.

## Authority files to read

Read:

```text
AGENTS.md
ARCHITECTURE.md
DOMAIN_MAP.md
TESTING.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/operating-model.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md
docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
docs-site/src/content/docs/workspace/routines/pr-review-routine.md
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
docs-site/src/content/docs/workspace/specs/phase-implementation-spec.md
the owning architecture, design, investigation, closeout, ADR, or report docs for the track
```

For runtime platform work, also read the current `PT-UI-RUNTIME-PLATFORM` architecture and cutover-plan docs before activating a phase.

## Working files to inspect

Inspect:

```text
active-work.md
roadmap.md
production-tracks.md
decision-register.md
completed-work.md when completion is claimed
relevant closeout reports
owning design and architecture docs
current PR metadata and changed files when a PR exists
phase implementation spec when one exists
```

For the next phase, inspect the phase contract in the production-track plan or phase spec before creating a prompt or branch.

## What to decide before editing

Decide:

```text
current production track
current phase
current lifecycle state
current branch and PR state when applicable
whether the previous phase has truthful completion/closeout evidence
whether the next phase is active-planning or active-implementation
whether active implementation is separately authorized
whether complete investigation gate evidence is present where required
whether complete design gate evidence is present where required
whether the phase has exact scope
whether a phase implementation spec exists or should be created as handoff support
whether the requested goal conflicts with an authority doc
```

If the goal conflicts with an authority doc, update the owning authority file first. Do not patch the convenient duplicate first.

## State transitions produced

This routine may recommend or patch planning transitions between:

```text
production-track -> active-planning
active-planning -> active-implementation
active-implementation -> review
review -> completed
review -> active-planning for the next phase, only after completion truth is recorded
```

It may open the next phase as active planning after the previous phase is truthful.

It must not open the next phase as active implementation unless the planning record separately authorizes exact owner files/crates, complete implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, complete investigation/design gate evidence where applicable, and stop conditions.

## Patch rules

- Keep orchestration patches to workflow, planning, handoff, PR body, review, or closeout truth.
- Do not implement runtime/product/domain code from this routine.
- Do not combine multiple implementation phases into one branch or PR.
- Do not mark a phase complete without delivered contract, evidence classes, validation status, known gaps, and planning truth.
- Do not move the next phase to active implementation before the current phase is reviewed, merged when applicable, and truthfully closed or explicitly scheduled for closeout before implementation resumes.
- Use `pr-review-routine.md` and `complete-merge-readiness-gate.md` before recommending merge.
- Use `phase-completion-drift-check-routine.md` before starting the next implementation phase after a phase completes.
- Use `roadmap-update-routine.md` when active-work, roadmap, production-track, completed-work, deferred-work, or decision-register state must change.
- Use `phase-implementation-spec.md` when a phase handoff needs a compact structured contract derived from accepted docs.

## Phase handoff requirements

Every implementation phase handed to an implementation agent must name:

```text
phase id and title
lifecycle state
owner
owning authority docs
allowed files/crates
forbidden files/crates
expected public API or user-visible behavior
invariants
acceptance criteria
validation commands
evidence expectations
stop conditions
closeout expectations
next phase activation condition
```

A phase spec may carry this contract. A PR body or prompt may carry it when no spec exists yet. The human-readable planning/design docs remain the authority unless a spec is explicitly accepted as a machine contract for that scope.

## Manual validation checklist

Check:

```text
track goal is named
phase order is named
current phase is named
previous phase completion truth is clear
current active-work truth is clear
roadmap and production-track state agree
complete investigation gate status is named where required
complete design gate status is named where required
phase implementation authorization is explicit or blocked
phase handoff is bounded to one phase
allowed and forbidden files/crates are present for active implementation
validation envelope is present for active implementation
merge readiness is checked before merge recommendation
phase-completion drift check is required before next implementation
phase spec role is clear when used
no runtime/product/domain implementation is hidden inside orchestration work
```

## Stop conditions

Stop and report a blocker if:

```text
the goal would collapse a production track into one implementation PR
the current phase is unclear
the previous phase completion truth is missing
the next phase is being moved to active implementation without separate authorization
allowed files/crates or forbidden files/crates are missing for implementation
a phase handoff lacks validation, evidence, or stop conditions
merge readiness is unknown but merge is requested
planning records disagree about current focus or phase order
an accepted design conflicts with the requested manager goal
phase spec text conflicts with accepted Markdown authority
a validator, script, generated view, or local helper is treated as workflow authority without accepted contract status
```

## Evidence to report

Report:

```text
track id
current phase
current branch/PR when applicable
lifecycle state
planning files inspected
owning authority docs inspected
evidence classes used
complete investigation gate status
complete design gate status
phase implementation authorization status
merge-readiness status
closeout state
phase spec status when relevant
validation run or unavailable
next safe action
whether implementation is authorized or still blocked
```

## Optional local helpers

Commands may add evidence when a local checkout is available:

```text
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Do not require commands to understand the next action. Do not claim command validation unless it was run.
