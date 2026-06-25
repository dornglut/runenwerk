---
title: Code Refactor Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
---

# Code Refactor Routine

## Use when

Use this routine for behavior-preserving code refactors.

## Authority files to read

Read `AGENTS.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `programming-principles.md`, and owning crate docs.

## Working files to inspect

Inspect implementation, call sites, tests, examples, docs, and public exports.

## What to decide before editing

Name the owner, behavior to preserve, refactor boundary, public API impact, and validation expectation.

## Patch rules

Keep the patch owned and behavior-preserving. Apply KISS, DRY, YAGNI, SOLID, Separation of Concerns, Avoid Premature Optimization, and Law of Demeter as review checks.

## Manual validation checklist

Confirm call sites, behavior preservation, dependency direction, public API impact, and tests to run.

## Evidence to report

Report changed files, exact functions/modules, behavior status, validation status, and risks.

## Optional local helpers

Use focused formatting, tests, and workspace checks when available.
