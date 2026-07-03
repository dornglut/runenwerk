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

State: active implementation for reusable renderer-neutral 2D coordinate and navigation surface contracts

Lifecycle state: `active-implementation`

Owner: `ui_controls` owns package-backed reusable control declarations and catalog/inspection projection. `ui_runtime` owns renderer-neutral proof-frame projection for runtime control surfaces. `ui_static_mount` owns static validation of proof frames. `ui_render_data`, `ui_render_primitives`, `ui_input`, and `ui_surface` are not implementation owners for the current Phase 16 contract unless implementation evidence proves the accepted contract cannot be delivered without them.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`, `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`, `docs-site/src/content/docs/domain/ui/architecture.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/workspace/planning/completed-work.md`.

Evidence classes: `E2` connector file/PR metadata/source inspection and `E8` accepted workspace/planning/design authority. No local command validation was run in the connector.

Complete investigation gate: satisfied for active implementation. Source-level investigation is recorded in `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`; it identifies exact owner files, conditional files, focused tests, validation envelope, and implementation stop conditions.

Complete design gate: satisfied for active implementation. The design records the complete owner split, Phase 16 delivered contract, feature support matrix, future-use-case pressure matrix, hierarchy/composition matrix, ergonomics/usability matrix, validation envelope, non-owned responsibilities, and stop conditions. The workflow has now been strengthened so compound work also requires a module decomposition map.

Implementation contract: implement the exact file contract from `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md`. Required owners are `ui_controls`, `ui_runtime`, and `ui_static_mount`. Conditional owners must remain unchanged unless implementation evidence proves a required capability cannot be delivered within the accepted contract and the design is updated first.

Allowed files/crates: the `ui_controls`, `ui_runtime`, and `ui_static_mount` files/tests named in the source investigation report. Compound paths must be split by responsibility rather than implemented as single catch-all files.

Non-owned files/crates: `ui_render_data`, `ui_render_primitives`, `ui_input`, `ui_surface`, graph/timeline/product/editor/game command owners, Typed App Composition implementation files, plugin framework files, `foundation/meta`, shared plugin primitive files, and new crates.

Module decomposition map: active for this branch. `ui_controls/src/surface2d/` is split into `ids`, `support`, `descriptor`, and `contribution`. `ui_runtime/src/surface2d/` is split into `transform`, `report`, `proof`, and `frame`. Public API stability is preserved by module-root re-exports. New compound files are not acceptable unless the design/planning record names one cohesive responsibility and a split trigger.

Maintainability review status: blocked until local validation runs and review confirms no remaining compound Surface2D files, no mixed validation/projection/runtime responsibilities, no public API drift from the split, and `surface2d_validation` is wired or explicitly redesigned before merge.

Feature support matrix: see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Future-use-case pressure matrix: see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Hierarchy/composition matrix: see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Ergonomics/usability: see `docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md`.

Validation expectation: `cargo test -p ui_controls surface2d`, `cargo test -p ui_controls control_package`, `cargo test -p ui_runtime surface2d`, `cargo test -p ui_static_mount surface2d`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`. Conditional crate validation is required only if a conditional owner changes.

Known blockers: not merge-ready. `surface2d_validation` still needs to be wired into package validation or explicitly redesigned, command validation has not run, and maintainability/decomposition review must pass after the split. Stop immediately if implementation requires a conditional owner change, renderer backend resources, product/editor/game mutation, GraphCanvas or Timeline vocabulary in the Surface2D public API, host command execution inside `domain/ui`, new crate creation, package/catalog/inspection bypass, or static mount proof that does not use the runtime proof frame.

Next action: wire or redesign Surface2D validation, run local validation, and perform a merge-readiness review including maintainability/decomposition status.

Evidence: Phase 15 Generic Text completed through baseline PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff` and hardening PR #49 at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`. Final validation passed on 2026-07-02 with `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_tree`, `cargo test -p ui_runtime`, `cargo test -p ui_controls`, `cargo test -p ui_static_mount`, `cargo test -p ui_render_primitives`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`. PR #56 refreshed Phase 16 planning. PR #57 completed the mechanical UI domain split before Phase 16 and explicitly did not implement Surface2D. PR #59 merged the complete investigation/design/evidence/merge workflow gates. PR #60 merged Phase 16 source-level investigation and design intake.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, and module decomposition status are known.

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
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
