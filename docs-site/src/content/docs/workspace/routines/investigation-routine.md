---
title: Investigation Routine
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
  - ../../guidelines/programming-principles.md
---

# Investigation Routine

## Use when

Use this routine to understand repository state before deciding what to change.

For architecture-sensitive, reusable, platform, public API, production-track, workflow, domain-boundary, dependency, ownership, phase planning, or stale-planning work, this routine must satisfy `complete-investigation-gate.md` before design, planning, or implementation decisions proceed.

## Authority files to read

Read `AGENTS.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `authority-model.md`, `workflow-lifecycle.md`, `complete-investigation-gate.md`, `complete-design-gate.md`, `programming-principles.md`, and the owning docs for the question.

## Working files to inspect

Inspect relevant code, tests, fixtures, docs, reports, planning records, closeout evidence, examples, public API entrypoints, and generated evidence or mirrors when they may affect the question.

## What to decide before editing

Name the owner, authoritative files, current reality, docs/planning reality, missing evidence, lifecycle state, whether complete investigation gate applies, whether complete design gate applies, confidence level, and next routine.

## State transitions produced

This routine may move work from idea to investigating, or from investigating to proposed-design, active-implementation, deferred, rejected, superseded, or a named next routine recommendation.

Direct promotion from investigation to active implementation is allowed only when the complete investigation gate proves that no complete design gate is required.

## Patch rules

Do not patch during pure investigation unless the user explicitly asks for changes.

If the investigation reveals incomplete authority, ownership drift, missing evidence, unclear dependency direction, public API impact, reusable/platform pressure, or stale planning truth, patch only investigation, design, or planning authority needed to record the truth. Do not implement runtime or product behavior from that finding in the same routine.

## Manual validation checklist

- Authority files inspected.
- Working files inspected by path.
- Current code reality recorded.
- Current docs/planning reality recorded.
- Authority/source matrix completed where applicable.
- Current-state matrix completed where applicable.
- Owner/dependency matrix completed where applicable.
- Capability inventory completed where applicable.
- Alternatives/tradeoff matrix completed where applicable.
- Confidence matrix completed where applicable.
- Lifecycle state named.
- Complete investigation gate applicability checked.
- Complete design gate applicability checked.
- Principles considered.
- Missing evidence and blockers reported.
- Next gate recommended.

## Stop conditions

Stop before design, planning, or implementation if ownership, active scope, current reality, complete investigation gate evidence, complete design gate evidence, validation, confidence, or dependency direction is unclear.

Stop before implementation if the investigation finds any architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary concern without complete design gate evidence.

## Evidence to report

Report findings by domain, exact files inspected, blockers, confidence, lifecycle state, complete investigation gate status, complete design gate status where applicable, and next action.

## Optional local helpers

Search commands and local tests may add evidence when available.
