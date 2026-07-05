---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-05
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
---

# Roadmap

This is the Markdown-first roadmap record for scriptless workflow.

## Current entries

### PT-UI-FRAMEWORK-APP-INTEGRATION-001

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-001`

Title: UI Framework App Integration Direction Review

State: active planning; implementation not authorized

Lifecycle state: `active-planning`

Authority: `ui-framework-app-integration-direction-review.md`.

Evidence: Current UI architecture and roadmap documents show that Runenwerk already has `ui_definition`, `UiProgram`, runtime artifacts/views, `UiStory`, component packages, binding/host data, and retained runtime proof substrate. The active decision is to settle how App/Plugin/ECS-hosted code uses those contracts as a real framework before continuing the manual `app_program` proof or promoting SpatialCanvas implementation.

Next action: Review and accept, revise, or reject the direction review. If accepted, write a separate implementation-planning contract for `ECS-backed Counter UI Story Proof`.

### PT-UI-COMPONENT-PLATFORM-013

ID: `PT-UI-COMPONENT-PLATFORM-013`

Title: Overlay / Popup / Layering full implementation

State: completed through merged PR #44

Lifecycle state: `completed`

Evidence: PR #44 merged into `main` at merge commit `6f2d3827f315191d7aeaf68a64f523627197cad8`. Evidence covers package-backed overlay declarations, base-control overlay lowering, main-path package validation, catalog projection, inspection projection, normalized input fact consumption, `ui_runtime::overlay` replay/report/stack/placement/focus/dismissal/suppression proof, proof-frame projection, static mount proof, no-bypass evidence, and full local validation gate passed on 2026-07-02.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-014

ID: `PT-UI-COMPONENT-PLATFORM-014`

Title: Text Editing / Editable Text Behavior

State: completed through merged PR #46

Lifecycle state: `completed`

Authority: `ui-component-platform-text-editing-design.md`.

Evidence: PR #46 merged into `main` at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Evidence covers package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/value/caret/selection/composition/suppression/no-bypass proof, proof-frame projection, static mount validation, focused tests, final proof-frame cleanup, and full local validation gate passed on 2026-07-02.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-015

ID: `PT-UI-COMPONENT-PLATFORM-015`

Title: Generic Text

State: completed through baseline PR #48 and hardening PR #49

Lifecycle state: `completed`

Authority: `ui-component-platform-generic-text-design.md`.

Evidence: PR #48 merged into `main` at merge commit `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`. Implementation commit `32e402b108d1e72d7cc5b4113af29d8d29626680` contains the renderer-neutral Generic Text substrate, package-backed descriptors and validation, catalog and `TextDisplay` inspection projection, runtime proof report/frame, static mount proof, renderer-neutral frame/extract adaptation to `TextVisualRun` / `TextGlyph` evidence, and the migration away from the old `ui_text::GlyphRun` / `PositionedGlyph` compatibility path. PR #49 merged into `main` at merge commit `338a8092d534dbb412da89363d50a46cd5efeae9` and completed the hardening pass: resolved source-run/cluster evidence, height overflow evidence, stable-ID constructors, homogeneous visual-run segmentation, `button_label()` policy cleanup, `text_emission` naming, Generic Text direction-policy inspection, and runtime output-emission splits. Final local validation passed on 2026-07-02 with the full Phase 15 cargo workspace/docs/diff gate.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-016

ID: `PT-UI-COMPONENT-PLATFORM-016`

Title: Surface2D

State: completed through docs-hardening PR #62 and implementation PR #61

Lifecycle state: `completed`

Authority: `ui-component-platform-surface2d-design.md`.

Evidence: PR #62 merged docs-only workflow, principle, decomposition, and merge-readiness hardening at merge commit `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged the Phase 16 Surface2D implementation at merge commit `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Post-merge validation from `main` passed with `cargo test -p ui_controls surface2d`, `cargo test -p ui_controls control_package`, `cargo test -p ui_runtime surface2d`, `cargo test -p ui_static_mount surface2d`, `cargo test --workspace`, `python tools/docs/validate_docs.py`, and `git diff --check`. Detailed closeout: `../../reports/closeouts/phase-16-surface2d-closeout.md`.

Next action: Keep as completed dependency. The next named production-track milestone is `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas planning intake, but this Phase 16 closeout does not authorize Phase 17 implementation. The active framework-direction review must be settled before SpatialCanvas is promoted as part of the real app-facing UI framework path.

## Rules

- Markdown must be enough to understand the current state.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
- Use `../workflow-lifecycle.md` before changing lifecycle state.
- Accepted direction does not authorize implementation.
