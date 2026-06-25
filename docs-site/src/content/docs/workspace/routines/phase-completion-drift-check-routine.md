---
title: Phase Completion Drift Check Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
---

# Phase Completion Drift Check Routine

## Use when

Use this routine after a completed phase or slice before starting the next one.

## Authority files to read

Read the accepted scope, changed files, tests, docs, planning records, reports, and `programming-principles.md`.

## Working files to inspect

Inspect files changed by the completed work and docs/planning records that claim completion.

## What to decide before editing

Decide whether the phase is complete, incomplete, drifted, or still risky.

## Patch rules

Do not start new implementation in this routine. Patch only status, docs, or planning records needed to make completion truthful.

## Manual validation checklist

Check promised work vs implemented work, code/docs drift, dependency drift, validation status, known gaps, and next safe action.

## Evidence to report

Report completed scope, files changed, evidence inspected, validation status, drift found, gaps, and next action.

## Optional local helpers

Run focused tests or docs validation when available.
