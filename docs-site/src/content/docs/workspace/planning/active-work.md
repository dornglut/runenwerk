---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-29
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

No current focus is active after `PT-UI-COMPONENT-PLATFORM-012` Generic Interaction closeout.

Most recent completed focus: `PT-UI-COMPONENT-PLATFORM-012` Generic Interaction.

State: completed in PR #43 implementation evidence.

Lifecycle state: `completed`

Owner: Phase 12 completed across `ui_controls` for package-backed reusable interaction declarations and catalog/inspection projection, `ui_input` for normalized pointer/keyboard/focus/semantic/text-intent facts, `ui_runtime` for descriptor-driven mounted replay, formation reports, renderer-neutral visible proof, and proof-frame projection, and `ui_static_mount` for static `UiFrame` mount validation.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-input-gesture-device-design.md`, `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, and the Phase 11 closeout report.

Write scope: Phase 12 implementation is closed. Do not add overlay/popup/layering, full text editing behavior, app/editor/game command mutation, shared plugin framework extraction, generic plugin primitives, or `foundation/meta` as part of Phase 12.

Validation expectation: Completed Phase 12 evidence used the implementation gate recorded in the closeout report: `cargo fmt --all --check`, `cargo check -p ui_controls`, `cargo check -p ui_input`, `cargo check -p ui_runtime`, `cargo check -p ui_static_mount`, focused `ui_controls`, `ui_input`, `ui_runtime`, and `ui_static_mount` tests, `python3 tools/docs/validate_docs.py`, and `git diff --check`.

Known blockers: None for Phase 12. PR #43 remains draft until repository review/merge handling is complete, but the implementation evidence and closeout record the intended completed Phase 12 scope. Phase 13 overlays/layering and later full text editing remain deferred.

Next action: Review/merge PR #43 or explicitly open the next planning focus. Do not start Phase 13 implementation without a new active-work entry and accepted owner/scope/validation envelope.

Evidence: PR #43 on branch `codex/phase-12-generic-interaction` implements package-backed interaction descriptors, package/catalog/inspection interaction visibility, normalized input facts, descriptor-driven replay/report, `InteractionVisualProof`/`InteractionProofFrame` visible proof, `InteractionProofRenderFrame`/`UiFrame` static mount proof, positive and negative interaction scenarios, read-only text-intent probe behavior, and zero host-command/product-mutation/overlay/text-edit boundary assertions.

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
