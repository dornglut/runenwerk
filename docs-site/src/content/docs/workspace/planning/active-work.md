---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-30
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-implementation-scope.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-012A`

Title: UI Component Platform Executable Interaction Story Cleanup / Validation

State: review / pending cleanup, validation, and merge

Lifecycle state: `review`

Owner: `ui_story` remains the workflow profile and evidence-envelope authority. `ui_runtime` owns interaction story session execution, replay/live application, semantic parity reporting, and visible proof formation. `ui_input` owns normalized input facts and conversion helpers. `ui_controls` owns reusable interaction descriptors and read-only declarations. `ui_static_mount` owns static frame validation. `runenwerk_editor` owns only the narrow base-controls proof-host adapter and must not claim UI Gallery product exposure in PR #43.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/documentation-structure.md`, `docs-site/src/content/docs/workspace/authority-model.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/planning/README.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-implementation-scope.md`, and `docs-site/src/content/docs/workspace/planning/decision-register.md`.

Write scope: Cleanup and validation for PR #43 only. Remove flawed 012B/UI Lab surface work and legacy compatibility shims. Keep current evidence scoped to reusable interaction descriptors, base-controls replay/report evidence, executable interaction story proof-host mechanics, semantic replay/live parity, static mount validation, and no-bypass assertions. UI Gallery product exposure is separate future work under `PT-UI-GALLERY-001`.

Validation expectation: Run and record `cargo fmt --all --check`, `cargo check -p runenwerk_editor`, `cargo test -p runenwerk_editor base_controls_interaction_proof_host`, `cargo check -p ui_controls`, `cargo check -p ui_input`, `cargo check -p ui_runtime`, `cargo check -p ui_static_mount`, `cargo test -p ui_controls control_interaction`, `cargo test -p ui_input input`, `cargo test -p ui_runtime executable_interaction_story`, `cargo test -p ui_runtime --test interaction_replay_report`, `cargo test -p ui_static_mount base_controls`, `python tools/docs/validate_docs.py`, and `git diff --check`.

Known blockers: PR #43 is not merged. Phase 12 and Phase 12A have implementation evidence on the PR branch, but remain in review until cleanup, validation, and merge complete. The deleted 012B/UI Lab proof-surface path must not be used as evidence. UI Gallery exposure requires a separate future plan.

Next action: Finish PR #43 cleanup, update the PR body to remove 012B/UI Lab proof-surface claims and legacy/compatibility language, validate the focused gate, and merge only after the review state is accurate.

Evidence: PR #43 contains Phase 12 lower-tier generic interaction evidence and Phase 12A executable interaction story evidence. User correction on 2026-06-30 split UI Gallery exposure out of PR #43 and required planning truth to mark 012/012A as review/pending merge rather than completed.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, and stop conditions are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Write scope:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
