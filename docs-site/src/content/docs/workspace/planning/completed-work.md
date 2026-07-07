---
title: Completed Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../../reports/closeouts/README.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
---

# Completed Work

Use this file for completed planning work.

This file is a short completion index. Detailed evidence belongs in `../../reports/closeouts/` when a completion record would become too large.

## Recently completed UI Runtime Platform work

- `PT-UI-RUNTIME-PLATFORM-001` Live UiPlugin Runtime and Generic Surface-Frame Rendering intake/design-gate hardening: completed 2026-07-07 through merged PR #74. Runtime implementation remained blocked.
- `PT-UI-RUNTIME-PLATFORM-002` Live UiPlugin Runtime Full Platform Cutover Plan: completed 2026-07-07 through merged PR #76 at merge commit `1697942c968afd9648872c202972826dc4c406b2`. Evidence lives in `../../design/active/live-uiplugin-runtime-full-cutover-plan.md`, `../../architecture/live-uiplugin-runtime-platform-architecture.md`, and workspace planning records. Runtime implementation remained blocked until the workflow gate completed.
- `PT-WORKFLOW-TRACK-ORCHESTRATION-001` Track Orchestration and Phase Spec Handoff Workflow: completed 2026-07-07 through merged PR #77 at merge commit `8b7a6b558bef79303e66d6a9f329dc71e00a0931`. Closeout report: `../../reports/closeouts/pt-workflow-track-orchestration-001-closeout.md`. Runtime implementation remained out of scope; its Phase 003 follow-up is fulfilled by PR #79.
- `PT-UI-RUNTIME-PLATFORM-003` UiPlugin Foundation: completed 2026-07-07 through merged PR #79 at merge commit `0135850277e904b4be2c336e3ef6507b3fc88b72`. Closeout report: `../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md`. Follow-up fulfilled by completed `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API`.
- `PT-UI-RUNTIME-PLATFORM-004` App Mounting API: completed 2026-07-07 through merged PR #82 at merge commit `9fb86f0d426385be7e425ff943c7a9d5450e1edb`. Closeout report: `../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md`. Next safe action is `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` planning only; implementation remains blocked until separately authorized.

## Recently completed UI Component Platform work

- `PT-UI-COMPONENT-PLATFORM-011` Base Control Packages: completed 2026-06-28 through merged PR #37 and user validation report. Closeout report: `../../reports/closeouts/pt-ui-component-platform-011-base-control-packages-closeout.md`.
- `PT-UI-COMPONENT-PLATFORM-012` Generic Interaction: completed 2026-06-30 through merged PR #43 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f` and user validation report.
- `PT-UI-COMPONENT-PLATFORM-012A` Executable Interaction Story: completed 2026-06-30 through merged PR #43 at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f` and user validation report.
- `PT-UI-COMPONENT-PLATFORM-013` Overlay / Popup / Layering full implementation: completed 2026-07-02 through merged PR #44 at merge commit `6f2d3827f315191d7aeaf68a64f523627197cad8`.
- `PT-UI-COMPONENT-PLATFORM-014` Text Editing / Editable Text Behavior: completed 2026-07-02 through merged PR #46 at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`.
- `PT-UI-COMPONENT-PLATFORM-015` Generic Text: completed 2026-07-02 through baseline PR #48 at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff` and hardening PR #49 at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9`.
- `PT-UI-COMPONENT-PLATFORM-016` Surface2D: completed 2026-07-03 through docs-hardening PR #62 at merge commit `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf` and implementation PR #61 at merge commit `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Closeout report: `../../reports/closeouts/phase-16-surface2d-closeout.md`.
- `PT-UI-FRAMEWORK-APP-INTEGRATION-002` ECS-backed Counter UI Story Proof: completed 2026-07-06 through merged PR #72 at merge commit `e093eb1affdc465b96430200960f8e3cdca0d26b`. Closeout report: `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.

## PT-UI-RUNTIME-PLATFORM-002 evidence

Implementation evidence: PR #76 was docs-only. It completed the full-platform cutover planning contract after PR #74 investigation/design-gate hardening. It added the accepted phase order from `003 UiPlugin Foundation` through `014 Closeout and Adoption Lock`, architecture-owned runtime diagrams, producer-generic render-boundary ordering before UiPlugin render publication, human and agent Counter product acceptance requirements, phased UI-runtime trace/history, source reload and persistence boundaries, and SDF UI backend deferral to downstream render-backend work.

Merge evidence: PR #76, `Docs: plan Live UiPlugin runtime full cutover`, merged into `main` on 2026-07-07 at merge commit `1697942c968afd9648872c202972826dc4c406b2`.

Validation: This completion index records merge evidence and planning truth only. The PR body recorded connector-only validation limits before merge. Do not infer cargo/runtime validation from this docs-only planning completion.

Known non-goals: runtime Rust implementation, engine `UiPlugin`, public `AppUiExt`, `app.mount_ui`, `UiScreen`, `IntoUi`, `UiActionHandler`, render adapters, generic render boundary implementation, `apps/ui_counter_runtime`, source reload/persistence implementation, SDF/world-space/SpatialCanvas implementation, `foundation/meta`, `domain/app_program`, generic plugin framework, and phase-spec validator tooling remained out of scope.

Follow-up: Fulfilled by completed `PT-WORKFLOW-TRACK-ORCHESTRATION-001`. Use the completed cutover plan as authority for `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation`.

## PT-WORKFLOW-TRACK-ORCHESTRATION-001 evidence

Implementation evidence: PR #77 delivered the track orchestration routine, track manager task card, phase implementation spec docs, RON phase spec template, authority-model links, planning links, and decision-register workflow record needed to manage one production-track goal through bounded phase PRs.

Merge evidence: PR #77, `Docs: add track orchestration and phase spec workflow`, merged into `main` on 2026-07-07 at merge commit `8b7a6b558bef79303e66d6a9f329dc71e00a0931`.

Validation: This completion index records merge evidence and planning truth only. The closeout report records that no cargo validation is required for the workflow-only completion and that the planning-closeout PR must run docs validation and diff hygiene commands.

Known non-goals: runtime Rust implementation, engine `UiPlugin`, public `AppUiExt`, `app.mount_ui`, typed screen/source/action contracts, render adapter work, generic producer boundary implementation, runtime Counter product, source reload/persistence implementation, SDF/world-space/SpatialCanvas implementation, `foundation/meta`, `domain/app_program`, generic plugin framework, validator tooling, and docs validator script changes remained out of scope.

Follow-up: Fulfilled by completed `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` through PR #79 and closeout truth. Use Phase 004 planning as the next runtime-platform state.

## PT-UI-RUNTIME-PLATFORM-003 evidence

Implementation evidence: PR #79 delivered the engine-owned `engine::plugins::ui`
foundation shell. Evidence covers the module root, `UiPlugin` install/build
behavior, `UiRuntimeSet` schedule labels, `UiRuntimeResource`,
`UiRuntimeReportResource`, `UiRuntimeDiagnosticsResource`, explicit plugin
export wiring, and focused `ui_plugin_*` engine tests.

Merge evidence: PR #79, `Add UiPlugin foundation`, squash-merged into `main`
on 2026-07-07 at merge commit `0135850277e904b4be2c336e3ef6507b3fc88b72`.

Validation: Before merge, `cargo test -p engine ui_plugin`, `cargo test -p
engine`, `python tools/docs/validate_docs.py`, `git diff --check`, `git diff
--check main...HEAD`, `git status --short --branch`, and `git diff --stat
main...HEAD` passed. GitHub reported no checks for the branch.

Known non-goals: public `AppUiExt`, `app.mount_ui`, `UiScreen`, `IntoUi`,
`UiActionHandler`, render adapters, SurfaceFrame generic producer-boundary
implementation, scene/debug overlay migration, source reload/persistence,
`apps/ui_counter_runtime`, SDF/world-space/SpatialCanvas, `foundation/meta`,
`domain/app_program`, generic plugin framework, phase spec validator, and
tools/docs validator changes remained out of scope.

Follow-up: Fulfilled by completed `PT-UI-RUNTIME-PLATFORM-004 — App Mounting
API` through PR #82 and closeout truth. Use Phase 005 planning as the next
runtime-platform state.

## PT-UI-RUNTIME-PLATFORM-004 evidence

Implementation evidence: PR #82 delivered the engine-owned App Mounting API.
Evidence covers `AppUiExt`, the normal `app.mount_ui` path, the advanced
`app.ui().mount` path, mount request/config/report types, mount-request
storage, mount rejection diagnostics with screen identity/mount source/stable
failure reason, `AppUiExt` prelude export, and focused `ui_mount_*` engine
tests.

Merge evidence: PR #82, `PT-UI-RUNTIME-PLATFORM-004 App Mounting API`,
squash-merged into `main` on 2026-07-07 at merge commit
`9fb86f0d426385be7e425ff943c7a9d5450e1edb`.

Validation: Before merge, `cargo test -p engine ui_mount`, `cargo test -p
engine`, `python tools/docs/validate_docs.py`, `git diff --check`, `git diff
--check main...HEAD`, `git status --short --branch`, and `git diff --stat
main...HEAD` passed. GitHub reported no checks for the branch.

Known non-goals: `UiScreen`, `IntoUi`, `UiActionHandler`, mounted session
runtime, host action dispatch, runtime trace, render adapters, SurfaceFrame
generic producer-boundary implementation, scene/debug overlay migration, source
reload/persistence, `apps/ui_counter_runtime`, SDF/world-space/SpatialCanvas,
`foundation/meta`, `domain/app_program`, generic plugin framework, phase spec
validator, and tools/docs validator changes remained out of scope.

Follow-up: Use the closeout report as the prerequisite for
`PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` active
planning. Do not start Phase 005 implementation until this closeout/planning
truth is merged and Phase 005 active implementation is separately authorized.

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

## PT-UI-FRAMEWORK-APP-INTEGRATION-002 evidence

Implementation evidence: PR #72 delivered the new `domain/ui/ui_app_integration`
crate as a small UI-owned ECS-backed proof bridge. Evidence covers
code-authored Counter and Win source records, lowering through
`ui_definition` / `ui_program_lowering` into `UiProgram` route facts,
`UiEventPacket` route/event resolution, ECS-backed Counter mutation,
next-output text facts, deterministic proof reports, and fail-closed tests for
unknown routes, wrong schema, missing capability, payload diagnostics/schema
mismatch, unformed routes, missing host action data, and rejected no-mutation
paths.

Merge evidence: PR #72, `UI: implement Counter app integration proof`, merged
into `main` on 2026-07-06 at merge commit
`e093eb1affdc465b96430200960f8e3cdca0d26b`.

Validation: Closeout validation ran from the docs-only closeout branch after
inspection. See `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`
for the exact command list and results.

Known non-goals: public `AppUiExt`, engine `UiPlugin`, render adapter/runtime
render proof, SDF, SpatialCanvas, world-space UI, `foundation/meta`,
`domain/app_program`, and generic plugin framework behavior remain out of
scope.

Follow-up: Review and harden PR #74 / `PT-UI-RUNTIME-PLATFORM-001` intake.
This completed proof does not authorize Live `UiPlugin` runtime implementation.

## Historical completed dependencies

Phases 001 through 010 remain completed dependencies by the prior user reports and PR evidence already recorded in roadmap, production track, and decision register. Their detailed text was intentionally not expanded here because this file is a short index.

## Rules

- Completion requires evidence.
- Validation must be reported as run, unavailable, or intentionally skipped with reason.
- Known gaps must stay visible.
- Historical closeouts and reports may contain detail; this file should remain an index.
- Use `../workflow-lifecycle.md` before moving work to completed.
- Put detailed evidence under `../../reports/closeouts/` when the completion entry would become a report archive.
