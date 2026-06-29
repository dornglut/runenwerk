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

ID: `PT-UI-COMPONENT-PLATFORM-012`

Title: UI Component Platform Generic Interaction implementation

State: review

Lifecycle state: `review`

Owner: `ui_controls` may own reusable control interaction declarations, semantic interaction states needed by reusable controls, control-level interaction requirements, descriptor/catalog/inspection facts for interaction support, and control kernel hooks only as declarations/contracts. `ui_input` owns normalized input packet vocabulary, device/gesture facts, pointer/key/focus/text-intent facts as reusable input data, and runtime input sample formation. `ui_runtime` owns runtime interaction formation over emitted or mounted UI, normalized input resolution against runtime UI structure, interaction facts/events for reusable controls, deterministic replay/report evidence, and runtime frame/session evidence. Hosts, apps, editor, and game layers own OS/window input collection, routing policy, command execution, app/editor/game mutation, game/world input policy, and product-specific behavior.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-input-gesture-device-design.md`, `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, and the Phase 11 closeout report.

Write scope: PR #43 implements the narrow Phase 12 generic reusable interaction envelope across `ui_controls`, `ui_input`, and `ui_runtime`. It may add reusable interaction declarations, catalog/inspection projection, normalized input facts, deterministic mounted replay/report evidence, negative proof cases, and no-bypass assertions. Preserve the focus, keyboard, and text-intent seams needed by later editable text controls. Do not change product code, do not add overlays/popups/layering, do not add full text editing behavior, do not add app/editor/game-specific mutation, do not extract a shared plugin framework, and do not introduce `foundation/meta`.

Validation expectation: PR #43 must run the Phase 12 implementation gate from the generic interaction design: `cargo fmt --all --check`, `cargo check -p ui_controls`, `cargo check -p ui_input`, `cargo check -p ui_runtime`, focused `ui_controls`, `ui_input`, and `ui_runtime` tests, `python3 tools/docs/validate_docs.py`, and `git diff --check`.

Known blockers: PR #43 remains draft until review fixes are complete and validation is rerun. No real gallery/story fixture path exists yet in this PR; the deterministic mounted replay/report proof is the accepted temporary visible proof for review, and the absence of a gallery fixture must be recorded in closeout if the phase completes from this PR.

Next action: Fix and review PR #43. The PR must prove descriptor-backed reusable interaction through compiled base-control descriptors, deterministic input replay, auditable interaction reports, and boundary assertions that host commands, product mutations, overlays, and full text editing did not run. Keep overlays/popups/layering in Phase 13. Keep full text editing in a later phase, but require it to consume the focus, keyboard, text-intent, and runtime interaction substrate shaped by Phase 12.

Evidence: Phase 11 completed through merged PR #37 on 2026-06-28. PR #43 opened as a draft Phase 12 implementation PR on branch `codex/phase-12-generic-interaction`.

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
