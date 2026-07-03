---
title: Active Work
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
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../design/active/runenwerk-typed-app-composition-plugin-framework-design.md
  - ../../design/active/runenwerk-typed-app-composition-plugin-framework-implementation-roadmap.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-016`

Title: Surface2D

State: active planning for reusable renderer-neutral 2D coordinate and navigation surface contracts

Lifecycle state: `active-planning`

Owner: `ui_controls` owns package-backed reusable control declarations and catalog/inspection projection. `ui_runtime` owns renderer-neutral proof-frame projection for runtime control surfaces. `ui_static_mount` owns static validation of proof frames. `ui_render_data`, `ui_render_primitives`, and `ui_input` are conditional owners only if source inspection proves existing contracts cannot carry the Phase 16 proof. Host/product/editor/game layers own app-specific canvas semantics, graph semantics, timeline semantics, command execution, authored UI editing, product mutation, renderer backend implementation, and app-composition authority.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/investigation-routine.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, `docs-site/src/content/docs/domain/ui/architecture.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/workspace/planning/completed-work.md`. `runenwerk-typed-app-composition-plugin-framework-design.md` and its roadmap are proposed architecture references only; they are not Phase 16 implementation authority.

Evidence classes: `E2` connector file/PR metadata inspection and `E8` accepted workspace/planning/design authority. No local command validation was run in the connector.

Complete investigation gate: active. The current design intake records authority/source evidence, current-state evidence, owner/dependency matrix, capability inventory, alternatives/tradeoff matrix, and confidence matrix. Source-level inspection of exact owner files is still required before active implementation.

Complete design gate: active. The design now records the complete owner split, Phase 16 delivered contract, feature support matrix, future-use-case pressure matrix, hierarchy/composition matrix, ergonomics/usability matrix, validation envelope, non-owned responsibilities, and stop conditions. It is not implementation authorization.

Write scope: Phase 16 investigation/design intake for reusable renderer-neutral `Surface2D` coordinate/navigation proof. The planning scope is surface identity, content and viewport bounds, world/screen transforms, pan, zoom, fit, selection rectangle, hover coordinate, pointer capture, gesture cancel/commit, overlay/diagnostic layers, grid/background vocabulary, large-content bounds, LOD readiness, budget evidence, accessibility/input acceptance, no-mutation boundaries, and proof visibility. No product code is authorized by this active-planning state.

Validation expectation: Planning docs must pass `python tools/docs/validate_docs.py` and `git diff --check`. A later implementation gate must include focused checks/tests for the selected `Surface2D` owner crates, package/catalog/inspection projection, runtime proof, static mount proof, diagnostics, no-bypass boundaries, accessibility/input facts, budget evidence, and any affected renderer-neutral primitive data.

Known blockers: Phase 16 is not implementation-authorized. Before implementation, source inspection must settle exact owner files, focused test names, fixture/proof names, whether `ui_render_data`, `ui_render_primitives`, or `ui_input` require changes, no product/editor/game mutation proof, accessibility/input acceptance proof, budget evidence shape, and current `ui_surface` symbol relationship. The planning decision is that `Surface2D` sits below `ui_surface` and does not rename, replace, remove, or absorb `ui_surface` in Phase 16. Typed App Composition remains proposed reference direction only. Graph semantics, timeline semantics, Gallery catalog semantics, editor commands, renderer backend resources, product/editor/game mutation, authored UI editing, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, and phase-shaped public API names are non-owned responsibilities with named downstream owners.

Next action: Complete source-level investigation for exact owner files and implementation contract. Do not start implementation until active planning is promoted with exact owner files, implementation scope, validation, evidence, and stop conditions.

Evidence: Phase 15 Generic Text completed through baseline PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff` and hardening PR #49 at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`. Final validation passed on 2026-07-02 with `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_tree`, `cargo test -p ui_runtime`, `cargo test -p ui_controls`, `cargo test -p ui_static_mount`, `cargo test -p ui_render_primitives`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`. PR #56 refreshed Phase 16 planning. PR #57 completed the mechanical UI domain split before Phase 16 and explicitly did not implement Surface2D.

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
Evidence classes:
Complete investigation gate:
Complete design gate:
Write scope:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
