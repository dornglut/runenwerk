---
title: Completed Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
  - ../../reports/closeouts/README.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
---

# Completed Work

Use this file for completed planning work.

This file is a short completion index. Detailed evidence belongs in `../../reports/closeouts/` when a completion record would become too large.

## Recently completed UI Component Platform work

- `PT-UI-COMPONENT-PLATFORM-011` Base Control Packages: completed 2026-06-28 through merged PR #37 and user validation report. Closeout report: `../../reports/closeouts/pt-ui-component-platform-011-base-control-packages-closeout.md`.
- `PT-UI-COMPONENT-PLATFORM-012` Generic Interaction: completed 2026-06-30 through merged PR #43 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f` and user validation report.
- `PT-UI-COMPONENT-PLATFORM-012A` Executable Interaction Story: completed 2026-06-30 through merged PR #43 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f` and user validation report.
- `PT-UI-COMPONENT-PLATFORM-013` Overlay / Popup / Layering full implementation: completed 2026-07-02 through merged PR #44 at merge commit `6f2d3827f315191d7aeaf68a64f523627197cad8`.

## PT-UI-COMPONENT-PLATFORM-013 evidence

PR #44 completed package-backed overlay declarations, base-control overlay lowering, main-path package validation, catalog projection, inspection projection, normalized input fact consumption, `ui_runtime::overlay` replay/report/stack/placement/focus/dismissal/suppression proof, proof-frame projection, static mount proof, no-bypass evidence, full local validation gate passed on 2026-07-02, and merge into `main`.

Validation: The full Phase 13 local validation gate passed on 2026-07-02 before merge. The gate included focused cargo checks and tests for `ui_controls`, `ui_input`, `ui_runtime`, `ui_static_mount`, `ui_story`, and `runenwerk_editor`, docs validation, and diff hygiene.

Known gaps: Text Editing / Editable Text Behavior is Phase 14 planning only. Product/editor/game behavior, authored UI editing, UI Gallery, UI Designer, Workbench/provider redesign, rich text editor behavior, code editor behavior, dynamic plugin framework, `foundation/meta`, shared plugin primitives, compatibility-only aliases/shims, and phase-shaped public API names remain out of scope.

Follow-up: Proceed to `PT-UI-COMPONENT-PLATFORM-014` Text Editing / Editable Text Behavior as design/planning intake only. Do not start implementation until active planning records exact owner files, implementation scope, validation, evidence, and stop conditions.

## Historical completed dependencies

Phases 001 through 010 remain completed dependencies by the prior user reports and PR evidence already recorded in roadmap, production track, and decision register. Their detailed text was intentionally not expanded here because this file is a short index.

## Entry shape

ID:

Title:

Lifecycle state: `completed`

Completed on:

Evidence:

Validation:

Known gaps:

Closeout report:

Follow-up:

## Rules

- Completion requires evidence.
- Validation must be reported as run, unavailable, or intentionally skipped with reason.
- Known gaps must stay visible.
- Historical closeouts and reports may contain detail; this file should remain an index.
- Use `../workflow-lifecycle.md` before moving work to completed.
- Put detailed evidence under `../../reports/closeouts/` when the completion entry would become a report archive.
