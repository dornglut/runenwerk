---
title: Implementation Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-28
related_docs:
  - ../start-here.md
  - ../workflow-lifecycle.md
  - ./phase-completion-drift-check-routine.md
  - ../../guidelines/programming-principles.md
---

# Implementation Routine

## Use when

Use this routine for bounded implementation work after the owner, intent, and acceptance scope are clear.

## Authority files to read

- `AGENTS.md`
- `ARCHITECTURE.md`
- `DEPENDENCY_RULES.md`
- `DOMAIN_MAP.md`
- `TESTING.md`
- `docs-site/src/content/docs/workspace/workflow-lifecycle.md`
- `docs-site/src/content/docs/guidelines/programming-principles.md`
- the owning design, ADR, planning record, issue, or crate/domain docs

For phase or production-track implementation, also read the current `active-work.md`, `roadmap.md`, `production-tracks.md`, and any prior closeout report consumed by the work.

## Working files to inspect

Inspect the target crate or document, nearby modules, public exports, tests, examples, and docs that define expected behavior.

For phase completion work, inspect the files that will be used as completion evidence and the planning records that will need closeout updates after merge or acceptance.

## What to decide before editing

- owning domain, crate, and subsystem;
- exact scope and non-goals;
- lifecycle state is `active-implementation` or explicitly authorized equivalent;
- invariant or behavior being changed;
- public API and docs impact;
- validation expectation;
- stop conditions;
- whether a design or ADR update is required;
- whether the patch is expected to complete the current active phase.

## State transitions produced

This routine may move work from active implementation to review.

It must not create active implementation from accepted direction alone. Use the roadmap update routine when planning state must change first.

When the implementation patch is merged or otherwise accepted and it completes an active phase, complete the phase completion drift check before starting the next implementation slice.

## Patch rules

- Keep the patch to the smallest coherent owned scope.
- Do not add speculative surfaces.
- Reuse existing patterns before adding abstractions.
- Keep dependency direction legal.
- Use explicit contracts across boundaries.
- Update docs when public behavior, ownership, or usage changes.
- If the patch is intended to complete an active phase, either include the closeout/planning updates or explicitly name the follow-up closeout patch required before the next implementation starts.

## Manual validation checklist

- Authority files inspected.
- Working files inspected by path.
- Lifecycle state and implementation authorization checked.
- Seven programming principles applied as a review lens.
- Dependency direction checked.
- Public API impact checked.
- Tests or local commands to run named.
- Command validation status stated honestly.
- Phase completion or closeout impact stated when the patch may finish a phase.

## Stop conditions

Stop and redesign if the requested implementation is only backed by accepted direction, has no exact owner/scope, violates dependency direction, needs architecture decision first, or requires validation that cannot be reported honestly.

Stop and do closeout/planning work instead if the next requested implementation depends on a previous phase that is merged in code but not truthfully closed in planning records.

## Evidence to report

Report changed files, exact functions/modules/sections, behavior impact, lifecycle state, authority files inspected, validation, remaining risks, phase closeout impact, and next step.

## Optional local helpers

Use focused formatting, tests, and docs validation when a local checkout is available. Do not block connector-mode work only because local commands are unavailable.
