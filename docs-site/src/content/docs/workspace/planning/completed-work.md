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
  - ../../design/active/ui-component-platform-generic-text-design.md
---

# Completed Work

Use this file for completed planning work.

This file is a short completion index. Detailed evidence belongs in `../../reports/closeouts/` when a completion record would become too large.

## Recently completed UI Component Platform work

- `PT-UI-COMPONENT-PLATFORM-011` Base Control Packages: completed 2026-06-28 through merged PR #37 and user validation report. Closeout report: `../../reports/closeouts/pt-ui-component-platform-011-base-control-packages-closeout.md`.
- `PT-UI-COMPONENT-PLATFORM-012` Generic Interaction: completed 2026-06-30 through merged PR #43 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f` and user validation report.
- `PT-UI-COMPONENT-PLATFORM-012A` Executable Interaction Story: completed 2026-06-30 through merged PR #43 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f` and user validation report.
- `PT-UI-COMPONENT-PLATFORM-013` Overlay / Popup / Layering full implementation: completed 2026-07-02 through merged PR #44 at merge commit `6f2d3827f315191d7aeaf68a64f523627197cad8`.
- `PT-UI-COMPONENT-PLATFORM-014` Text Editing / Editable Text Behavior: completed 2026-07-02 through merged PR #46 at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`.
- `PT-UI-COMPONENT-PLATFORM-015` Generic Text: completed 2026-07-02 through merged PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`.

## PT-UI-COMPONENT-PLATFORM-014 evidence

Evidence: PR #46 completed package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/value/caret/selection/composition/suppression/no-bypass proof, proof-frame projection, static mount validation, focused tests, final proof-frame cleanup, and merge into `main`.

Merge evidence: PR #46, `UI/text editing behavior phase 14`, merged into `main` on 2026-07-02 at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Post-merge inspection found `main` identical to that merge commit.

Validation: The full Phase 14 local validation gate passed on 2026-07-02 before merge. The gate covered formatting, focused cargo checks, focused cargo tests, docs validation, and diff hygiene.

Known non-goals: Generic Text display/layout has completed as Phase 15. Rich text editor behavior, code editor behavior, product document buffers, undo/redo, clipboard integration, UI Designer, UI Gallery product surface, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, product/editor/game mutation, authored UI editing, compatibility-only aliases/shims, and phase-shaped public API names remain out of scope.

Follow-up: Proceed to `PT-UI-COMPONENT-PLATFORM-016` Surface2D as design/planning intake only. Do not start implementation until active planning records exact owner files, implementation scope, validation, evidence expectation, and stop conditions.

## PT-UI-COMPONENT-PLATFORM-015 evidence

Evidence: PR #48 branch `ui/generic-text-phase-15` implements the renderer-neutral Generic Text substrate across `ui_text`, `ui_render_data`, `ui_controls`, `ui_runtime`, and `ui_static_mount`. The local closeout keeps text display separate from Phase 14 text editing; removes the old `ui_text::GlyphRun` / `PositionedGlyph` compatibility path; keeps `GlyphRunPrimitive` backed by `TextBlockLayoutResult` / `TextVisualRun` evidence; adapts the renderer-neutral frame/extract path to consume `TextVisualRun` / `TextGlyph` evidence without adding a renderer backend; adds package-backed Generic Text descriptors, validation reasons, catalog projection, and `TextDisplay` inspection projection; migrates text-editing proof frames to visual-run evidence; and adds runtime/static-mount proof coverage.

PR evidence: PR #48, `ui/generic-text-phase-15`, implementation commit `32e402b108d1e72d7cc5b4113af29d8d29626680`, merged into `main` at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`.

Validation: The Phase 15 local validation gate passed on 2026-07-02 with `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_controls`, `cargo test -p ui_runtime`, `cargo test -p ui_static_mount`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`.

Known non-goals: Renderer backend implementation, GPU atlas upload, OS font discovery, rich/code editor behavior, document buffers, undo/redo, clipboard, LSP/syntax highlighting, app localization policy, product/editor/game mutation, authored UI editing, UI Designer/Gallery product surface, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, and compatibility-only public API shims remain out of scope.

Follow-up: Proceed with `PT-UI-COMPONENT-PLATFORM-016` Surface2D as planning intake only. Do not start implementation until active planning records exact owner files, implementation scope, validation, evidence expectation, and stop conditions.

## Historical completed dependencies

Phases 001 through 010 remain completed dependencies by the prior user reports and PR evidence already recorded in roadmap, production track, and decision register. Their detailed text was intentionally not expanded here because this file is a short index.

## Rules

- Completion requires evidence.
- Validation must be reported as run, unavailable, or intentionally skipped with reason.
- Known gaps must stay visible.
- Historical closeouts and reports may contain detail; this file should remain an index.
- Use `../workflow-lifecycle.md` before moving work to completed.
- Put detailed evidence under `../../reports/closeouts/` when the completion entry would become a report archive.
