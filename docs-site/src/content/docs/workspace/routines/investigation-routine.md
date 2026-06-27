---
title: Investigation Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ../workflow-lifecycle.md
  - ../authority-model.md
  - ../../guidelines/programming-principles.md
---

# Investigation Routine

## Use when

Use this routine to understand repository state before deciding what to change.

## Authority files to read

Read `AGENTS.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `authority-model.md`, `workflow-lifecycle.md`, `programming-principles.md`, and the owning docs for the question.

## Working files to inspect

Inspect only relevant code, tests, docs, reports, planning records, and examples.

## What to decide before editing

Name the owner, authoritative files, missing evidence, lifecycle state, and next routine.

## State transitions produced

This routine may move work from idea to investigating, or from investigating to proposed-design, deferred, rejected, or a named next routine recommendation.

## Patch rules

Do not patch during pure investigation unless the user explicitly asks for changes.

## Manual validation checklist

Confirm authority files inspected, working files inspected by path, conflicts named, lifecycle state named, principles considered, and missing evidence reported.

## Stop conditions

Stop and ask for a design or planning update before implementation if ownership, active scope, validation, or dependency direction is unclear.

## Evidence to report

Report findings by domain, exact files inspected, blockers, confidence, lifecycle state, and next action.

## Optional local helpers

Search commands and local tests may add evidence when available.
