---
title: Completed Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../../reports/closeouts/README.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
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
- `PT-UI-COMPONENT-PLATFORM-015` Generic Text: completed 2026-07-02 through baseline PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff` and hardening PR #49 at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`.
- `PT-UI-COMPONENT-PLATFORM-016` Surface2D: completed 2026-07-03 through docs-hardening PR #62 at merge commit `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf` and implementation PR #61 at merge commit `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Closeout report: `../../reports/closeouts/phase-16-surface2d-closeout.md`.

## PT-UI-COMPONENT-PLATFORM-014 evidence

Evidence: PR #46 completed package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/value/caret/selection/composition/suppression/no-bypass proof, proof-frame projection, static mount validation, focused tests, final proof-frame cleanup, and merge into `main`.

Merge evidence: PR #46, `UI/text editing behavior phase 14`, merged into `main` on 2026-07-02 at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Post-merge inspection found `main` identical to that merge commit.

Validation: The full Phase 14 local validation gate passed on 2026-07-02 before merge. The gate covered formatting, focused cargo checks, focused cargo tests, docs validation, and diff hygiene.

Known non-goals: Generic Text display/layout has completed as Phase 15. Rich text editor behavior, code editor behavior, product document buffers, undo/redo, clipboard integration, UI Designer, UI Gallery product surface, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, product/editor/game mutation, authored UI editing, compatibility-only aliases/shims, and phase-shaped public API names remain out of scope.

Follow-up: Fulfilled by completed `PT-UI-COMPONENT-PLATFORM-016` Surface2D. Keep Phase 14 as a completed dependency.

## PT-UI-COMPONENT-PLATFORM-015 evidence

Baseline evidence: PR #48 implemented the renderer-neutral Generic Text substrate across `ui_text`, `ui_render_data`, `ui_controls`, `ui_runtime`, and `ui_static_mount`. It added Generic Text descriptors, validation, catalog projection, `TextDisplay` inspection projection, runtime proof reporting, static mount proof, renderer-neutral frame/extract adaptation to `TextVisualRun` / `TextGlyph` evidence, and removal of the old `ui_text::GlyphRun` / `PositionedGlyph` compatibility path.

Hardening evidence: PR #49 completed the Phase 15 hardening pass without starting Phase 16. It corrected Generic Text layout evidence, added stable-ID text constructors, added text layout policy helpers, moved button defaults away from role-specific badge vocabulary, segmented visual runs by homogeneous evidence, exposed text direction policy through Generic Text inspection, renamed runtime text helpers to `text_emission`, and split large runtime output emission into focused modules.

Merge evidence: PR #48 merged at `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`. PR #49 merged at `338a8092d534dbb412da89363d50a46cd5efeae9`.

Validation: The final Phase 15 validation gate passed on 2026-07-02 with `cargo test -p ui_text`, `cargo test -p ui_render_data`, `cargo test -p ui_tree`, `cargo test -p ui_runtime`, `cargo test -p ui_controls`, `cargo test -p ui_static_mount`, `cargo test -p ui_render_primitives`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`.

Known non-goals: Renderer backend implementation, GPU atlas upload, OS font discovery, rich/code editor behavior, document buffers, undo/redo, clipboard, LSP/syntax highlighting, app localization policy, product/editor/game mutation, authored UI editing, UI Designer/Gallery product surface, Workbench/provider redesign, dynamic plugin framework, `foundation/meta`, shared plugin primitives, and compatibility-only public API shims remain out of scope.

Follow-up: Fulfilled by completed `PT-UI-COMPONENT-PLATFORM-016` Surface2D. Keep Phase 15 as a completed dependency.

## PT-UI-COMPONENT-PLATFORM-016 evidence

Implementation evidence: PR #61 delivered package-backed Surface2D declarations, validation, catalog projection, inspection facts, runtime proof reporting, renderer-neutral proof-frame projection, and static mount proof across `ui_controls`, `ui_runtime`, and `ui_static_mount`. PR #62 preceded and complemented the implementation by merging docs-only workflow, principle, decomposition, and merge-readiness hardening.

Merge evidence: PR #62 merged at `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged at `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`.

Validation: Post-merge validation from `main` passed with `cargo test -p ui_controls surface2d`, `cargo test -p ui_controls control_package`, `cargo test -p ui_runtime surface2d`, `cargo test -p ui_static_mount surface2d`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`.

Known non-goals: Renderer backend ownership, product/editor/game mutation, host command execution inside `domain/ui`, graph/timeline public API semantics, new crates, plugin framework work, `foundation/meta`, and broad workflow rewrites remain out of scope for Phase 16.

Follow-up: Use the completed Surface2D substrate as the dependency for the next production-track planning intake. The next named milestone is `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas, but this closeout does not authorize Phase 17 implementation.

## Historical completed dependencies

Phases 001 through 010 remain completed dependencies by the prior user reports and PR evidence already recorded in roadmap, production track, and decision register. Their detailed text was intentionally not expanded here because this file is a short index.

## Rules

- Completion requires evidence.
- Validation must be reported as run, unavailable, or intentionally skipped with reason.
- Known gaps must stay visible.
- Historical closeouts and reports may contain detail; this file should remain an index.
- Use `../workflow-lifecycle.md` before moving work to completed.
- Put detailed evidence under `../../reports/closeouts/` when the completion entry would become a report archive.
