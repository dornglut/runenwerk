---
title: Code Refactor Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
  - ../../guidelines/programming-principles.md
---

# Code Refactor Routine

## Use when

Use this routine for behavior-preserving code refactors.

A refactor stops being a simple refactor when it changes public API, durable vocabulary, domain ownership, dependency direction, runtime behavior, validation contract, or reusable platform surface. In that case, use the complete investigation gate and, when required, the complete design gate before implementation.

## Authority files to read

Read `AGENTS.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `workflow-lifecycle.md`, `complete-investigation-gate.md`, `complete-design-gate.md`, `evidence-quality-taxonomy.md`, `complete-merge-readiness-gate.md`, `programming-principles.md`, and owning crate docs.

## Working files to inspect

Inspect implementation, call sites, tests, examples, docs, public exports, dependency edges, and nearby module conventions.

## What to decide before editing

Name the owner, behavior to preserve, refactor boundary, public API impact, dependency impact, validation expectation, evidence class for behavior preservation, whether complete investigation/design gates apply, and merge-readiness impact if the branch will be merged.

## State transitions produced

This routine may move a refactor task from active implementation to review.

It must not create active implementation from accepted direction alone and must not rename durable APIs or move ownership without an accepted design path.

## Patch rules

Keep the patch owned and behavior-preserving. Apply KISS, DRY, YAGNI, SOLID, Separation of Concerns, Avoid Premature Optimization, and Law of Demeter as review checks.

Do not hide behavior, public API, ownership, or dependency changes inside a refactor. Promote to investigation/design when the refactor reveals broader change pressure.

## Manual validation checklist

Confirm call sites, behavior preservation, dependency direction, public API impact, durable vocabulary impact, tests to run, evidence class, complete investigation/design gate status where applicable, and merge-readiness status when recommending merge.

## Stop conditions

Stop and switch routine if the change is not behavior-preserving, changes public API, changes domain ownership, changes dependency direction, creates migration or compatibility obligations, requires a new crate/shared extraction, or lacks evidence to support behavior preservation.

## Evidence to report

Report changed files, exact functions/modules, behavior status, evidence classes used, complete investigation/design gate status where applicable, validation status, merge-readiness status when relevant, and risks.

## Optional local helpers

Use focused formatting, tests, workspace checks, and docs validation when available. Commands are evidence only and do not replace manual authority review.
