---
title: Implementation Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../start-here.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ./phase-completion-drift-check-routine.md
  - ../../guidelines/programming-principles.md
---

# Implementation Routine

## Use when

Use this routine for bounded implementation work after the owner, intent, complete implementation contract, and acceptance scope are clear.

For architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work, implementation must not start until the active planning record points to complete investigation gate evidence and complete design gate evidence where required.

## Authority files to read

- `AGENTS.md`
- `ARCHITECTURE.md`
- `DEPENDENCY_RULES.md`
- `DOMAIN_MAP.md`
- `TESTING.md`
- `docs-site/src/content/docs/workspace/workflow-lifecycle.md`
- `docs-site/src/content/docs/workspace/complete-investigation-gate.md`
- `docs-site/src/content/docs/workspace/complete-design-gate.md`
- `docs-site/src/content/docs/guidelines/programming-principles.md`
- the owning design, ADR, planning record, issue, or crate/domain docs

For phase or production-track implementation, also read the current `active-work.md`, `roadmap.md`, `production-tracks.md`, and any prior closeout report consumed by the work.

## Working files to inspect

Inspect the target crate or document, nearby modules, public exports, tests, examples, docs, complete investigation evidence, and complete design gate evidence that define expected behavior.

For phase completion work, inspect the files that will be used as completion evidence and the planning records that will need closeout updates after merge or acceptance.

## What to decide before editing

- owning domain, crate, and subsystem;
- exact complete implementation contract and non-owned responsibilities;
- lifecycle state is `active-implementation` or explicitly authorized equivalent;
- complete investigation gate status where applicable;
- complete design gate status where applicable;
- feature support matrix status where applicable;
- future-use-case pressure matrix status where applicable;
- hierarchy/composition matrix status where applicable;
- ergonomics and usability contract status where applicable;
- invariant or behavior being changed;
- public API and docs impact;
- durable vocabulary versus proof, migration, or test-fixture names;
- validation expectation;
- stop conditions;
- whether a design or ADR update is required;
- whether the patch is expected to complete the current active phase.

## State transitions produced

This routine may move work from active implementation to review.

It must not create active implementation from accepted direction alone. Use the roadmap update routine when planning state must change first.

When the implementation patch is merged or otherwise accepted and it completes an active phase, complete the phase completion drift check before starting the next implementation contract.

## Patch rules

- Keep the patch to the complete owned contract authorized by planning.
- Do not add speculative surfaces outside the accepted contract.
- Reuse existing patterns before adding abstractions.
- Keep dependency direction legal.
- Use explicit contracts across boundaries.
- Update docs when public behavior, ownership, usage, ergonomics, or support status changes.
- Give durable public APIs, stable ids, reusable fixture helpers, and platform vocabulary domain-shaped names. Keep phase and PR labels in planning, tests, reports, or migration notes unless an accepted design explicitly says otherwise.
- If the patch is intended to complete an active phase, either include the closeout/planning updates or explicitly name the follow-up closeout patch required before the next implementation starts.

## Manual validation checklist

- Authority files inspected.
- Working files inspected by path.
- Lifecycle state and implementation authorization checked.
- Complete investigation gate evidence checked where applicable.
- Complete design gate evidence checked where applicable.
- Feature support matrix checked where applicable.
- Future-use-case pressure matrix checked where applicable.
- Hierarchy/composition matrix checked where applicable.
- Ergonomics and usability contract checked where applicable.
- Seven programming principles applied as a review lens.
- Dependency direction checked.
- Public API impact checked.
- Durable naming checked for public exports, stable ids, fixtures, and docs.
- Tests or local commands to run named.
- Command validation status stated honestly.
- Phase completion or closeout impact stated when the patch may finish a phase.

## Stop conditions

Stop and redesign if the requested implementation is only backed by accepted direction, has no exact owner/scope, lacks complete investigation gate evidence where required, lacks complete design gate evidence where required, violates dependency direction, needs architecture decision first, or requires validation that cannot be reported honestly.

Stop and redesign if a proof, migration helper, or test fixture starts becoming a durable public API, stable id, reusable platform vocabulary, or product-facing surface without an accepted owner-first design.

Stop and do closeout/planning work instead if the next requested implementation depends on a previous phase that is merged in code but not truthfully closed in planning records.

## Evidence to report

Report changed files, exact functions/modules/sections, behavior impact, lifecycle state, complete investigation gate status where applicable, complete design gate status where applicable, authority files inspected, validation, remaining risks, phase closeout impact, and next step.

## Optional local helpers

Use focused formatting, tests, and docs validation when a local checkout is available. Do not block connector-mode work only because local commands are unavailable.
