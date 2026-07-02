---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-016`

Title: Surface2D

State: active planning for reusable renderer-neutral 2D coordinate and navigation surface contracts

Lifecycle state: `active-planning`

Owner: `ui_controls` owns package-backed reusable control declarations and catalog/inspection projection. `ui_runtime` owns renderer-neutral proof-frame projection for runtime control surfaces. `ui_static_mount` owns static validation of proof frames. Phase 16 planning must settle the exact `Surface2D` owner split before implementation. Host/product/editor/game layers own app-specific canvas semantics, graph semantics, timeline semantics, command execution, authored UI editing, product mutation, and renderer backend implementation.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, `docs-site/src/content/docs/workspace/planning/active-work.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/workspace/planning/completed-work.md`.

Write scope: Design intake for reusable renderer-neutral `Surface2D` coordinate/navigation proof. The planning scope is surface identity, content and viewport bounds, world/screen transforms, pan, zoom, fit, selection rectangle, hover coordinate, pointer capture, gesture cancel/commit, overlay/diagnostic layers, grid/background vocabulary, large-content bounds, LOD readiness, and budget evidence. Prefer docs-only planning until owner files, implementation scope, validation, evidence expectations, and stop conditions are exact.

Validation expectation: Planning docs must pass `python tools/docs/validate_docs.py` and `git diff --check`. A later implementation gate must include focused checks/tests for the selected `Surface2D` owner crates, package/catalog/inspection projection, runtime proof, static mount proof, and any affected renderer-neutral primitive data.

Known blockers: No Phase 15 implementation blocker remains in local validation. Phase 16 is not implementation-authorized. Graph semantics, timeline semantics, Gallery catalog semantics, editor commands, renderer backend resources, product/editor/game mutation, authored UI editing, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, and phase-shaped public API names remain out of scope.

Next action: Review and refine the Phase 16 Surface2D design intake. Do not start implementation until active planning is promoted with exact owner files, implementation scope, validation, evidence, and stop conditions.

Evidence: Phase 15 Generic Text completed through merged PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`; implementation commit `32e402b108d1e72d7cc5b4113af29d8d29626680` carried the validated code closeout. Validation passed on 2026-07-02 with `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_controls`, `cargo test -p ui_runtime`, `cargo test -p ui_static_mount`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`.

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
