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
  - ../../reports/investigations/phase-16-surface2d-source-investigation.md
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

Owner: `ui_controls` owns package-backed reusable control declarations and catalog/inspection projection. `ui_runtime` owns renderer-neutral proof-frame projection for runtime control surfaces. `ui_static_mount` owns static validation of proof frames. `ui_render_data`, `ui_render_primitives`, `ui_input`, and `ui_surface` are not implementation owners for the current Phase 16 contract unless implementation evidence proves the accepted contract cannot be delivered without them. Host/product/editor/game layers own app-specific canvas semantics, graph semantics, timeline semantics, command execution, authored UI editing, product mutation, renderer backend implementation, and app-composition authority.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/investigation-routine.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`, `docs-site/src/content/docs/domain/ui/architecture.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/workspace/planning/completed-work.md`. `runenwerk-typed-app-composition-plugin-framework-design.md` and its roadmap are proposed architecture references only; they are not Phase 16 implementation authority.

Evidence classes: `E2` connector file/PR metadata/source inspection and `E8` accepted workspace/planning/design authority. No local command validation was run in the connector.

Complete investigation gate: source-level investigation is recorded in `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`. It identifies exact owner files, conditional files, focused tests, validation envelope, and implementation stop conditions. Local command validation remains required before merge.

Complete design gate: active. The design records the complete owner split, Phase 16 delivered contract, feature support matrix, future-use-case pressure matrix, hierarchy/composition matrix, ergonomics/usability matrix, validation envelope, non-owned responsibilities, and stop conditions. It is not implementation authorization until planning is promoted.

Write scope: Phase 16 investigation/design intake for reusable renderer-neutral `Surface2D` coordinate/navigation proof. The implementation contract input is now recorded, but this branch remains docs/planning only. No product code is authorized by this active-planning state.

Validation expectation: Planning docs must pass `python tools/docs/validate_docs.py` and `git diff --check`. A later implementation gate must include `cargo test -p ui_controls surface2d`, `cargo test -p ui_controls control_package`, `cargo test -p ui_runtime surface2d`, `cargo test -p ui_static_mount surface2d`, `cargo test --workspace`, docs validation, and diff validation. Conditional crate validation is required only if `ui_render_data`, `ui_render_primitives`, `ui_input`, or `ui_surface` are touched.

Known blockers: Phase 16 is not implementation-authorized. Before implementation, this intake branch should be reviewed, docs validation must be run, and planning must be promoted to active implementation with exact implementation scope. The accepted implementation contract is `ui_controls` + `ui_runtime` + `ui_static_mount`, with `ui_render_data`, `ui_render_primitives`, `ui_input`, and `ui_surface` untouched unless proven necessary. `Surface2D` sits below `ui_surface` and does not rename, replace, remove, or absorb `ui_surface` in Phase 16. Typed App Composition remains proposed reference direction only. Graph semantics, timeline semantics, Gallery catalog semantics, editor commands, renderer backend resources, product/editor/game mutation, authored UI editing, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, and phase-shaped public API names are non-owned responsibilities with named downstream owners.

Next action: Run docs validation for the intake branch, review/merge the docs/planning intake, then open an implementation branch promoted to active implementation with the exact file contract from the source investigation report.

Evidence: Phase 15 Generic Text completed through baseline PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff` and hardening PR #49 at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`. Final validation passed on 2026-07-02 with `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_tree`, `cargo test -p ui_runtime`, `cargo test -p ui_controls`, `cargo test -p ui_static_mount`, `cargo test -p ui_render_primitives`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`. PR #56 refreshed Phase 16 planning. PR #57 completed the mechanical UI domain split before Phase 16 and explicitly did not implement Surface2D. PR #59 merged the complete investigation/design/evidence/merge workflow gates. This branch records Phase 16 source-level intake only.

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
