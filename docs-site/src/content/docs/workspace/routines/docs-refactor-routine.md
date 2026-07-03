---
title: Documentation Refactor Routine
description: Scriptless routine for documentation cleanup, movement, pruning, link repair, and authority alignment.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../documentation-structure.md
  - ../authority-model.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
  - ../../guidelines/programming-principles.md
---

# Documentation Refactor Routine

## Use when

Use this routine for documentation cleanup, movement, pruning, link repair, root-summary alignment, duplicate-authority removal, and archive/report placement.

Use `roadmap-update-routine.md` instead when the main task changes active planning state.

Use the complete investigation gate when cleanup may change authority, lifecycle state, design meaning, planning truth, ownership, compatibility, public API documentation, or current/future implementation direction.

## Authority files to read

Read `AGENTS.md`, `documentation-structure.md`, `authority-model.md`, `workflow-lifecycle.md`, `complete-investigation-gate.md`, `complete-design-gate.md`, `evidence-quality-taxonomy.md`, `complete-merge-readiness-gate.md`, `programming-principles.md`, and the owning doc for the affected area.

## Working files to inspect

Inspect affected docs, links, root summaries, indexes, planning records, reports, archives, generated mirrors, and any code/tests required to verify current-reality claims in the docs being changed.

## What to decide before editing

Classify each file by purpose and owner before patching. Decide whether the change is cleanup only, authority change, lifecycle change, design change, planning change, archival move, or generated-mirror repair.

Common purposes are root summary, guideline, investigation dossier, design, decision record, evidence taxonomy, merge readiness gate, production track, roadmap entry, active work, deferred work, completed work, closeout report, routine, task card, generated mirror, generated evidence, generated contract, and archive.

## State transitions produced

This routine may move active docs to superseded or archived status, move detailed completion evidence into closeout reports, align root summaries with docs-site authority, or report stale generated mirrors.

It must not move work into active planning, active implementation, or completed state by itself.

## Patch rules

- Patch the owning document first.
- Keep root docs concise.
- Remove duplicate authority instead of preserving parallel claims.
- Update links when moving or renaming files.
- Report old path to new path mapping.
- Keep generated views optional unless a narrow accepted contract says otherwise.
- Do not combine product architecture changes with documentation cleanup.
- Classify evidence when docs claims depend on current code, validation, CI, generated artifacts, or user-reported results.
- Check merge readiness before recommending merge.

## Manual validation checklist

Confirm file purpose, owner, links, root-summary length, duplicate-authority removal, stale references, planning authority placement, report placement, generated-file classification, complete investigation/design gate status where applicable, evidence classes, and merge-readiness status when relevant.

## Stop conditions

Stop and redesign if cleanup would make a root doc a long design or roadmap, require generated files for workflow comprehension, move active planning state into a design doc, remove historical evidence without an archive/report path, collapse distinct artifact jobs into one document, change implementation scope without a design/decision update, weaken evidence wording, or hide lifecycle changes as cleanup.

## Evidence to report

Report files changed, old path to new path mapping, link review, authority conflicts resolved, evidence classes used, validation status, complete investigation/design gate status where applicable, merge-readiness status when relevant, remaining drift, and next lifecycle state or routine.

## Optional local helpers

Run docs validation when available:

```text
python3 tools/docs/validate_docs.py
git diff --check
```

Local helpers are evidence only. They do not replace manual authority review.
