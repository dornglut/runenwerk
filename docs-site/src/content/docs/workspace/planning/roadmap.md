---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-spatial-canvas-design.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/investigations/phase-17-spatialcanvas-source-investigation.md
---

# Roadmap

This is the Markdown-first roadmap record for scriptless workflow.

## Current entries

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

Next action: Keep as completed dependency for `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas. This Phase 16 closeout does not authorize Phase 17 implementation.

### PT-UI-COMPONENT-PLATFORM-017

ID: `PT-UI-COMPONENT-PLATFORM-017`

Title: SpatialCanvas

State: active planning and design intake only

Lifecycle state: `active-planning`

Authority: `ui-component-platform-spatial-canvas-design.md` and `../../reports/investigations/phase-17-spatialcanvas-source-investigation.md`.

Evidence: Phase 16 Surface2D completed through PR #61 after PR #62 workflow hardening. PR #63 closed Phase 16 planning truth, and PR #64 extracted stale Surface2D future-pressure material into `../../reports/investigations/surface2d-future-pressure-branch-review.md`. Current branch and PR inspection for this intake found no open PRs and no `origin/surface2d-phase-16` remote branch. Source investigation inspected the current `ui_controls`, `ui_runtime`, `ui_static_mount`, `ui_render_data`, `ui_render_primitives`, `ui_input`, `ui_surface`, `ui_tree`, `ui_composition`, `ui_graph_editor`, `editor_viewport`, `scene`, `spatial`, and `spatial_index` paths.

Scope: define the reusable positioned-item `SpatialCanvas` planning contract above completed `Surface2D` without duplicating Surface2D coordinate/navigation truth and without taking ownership of renderer backends, camera/projection/scene resources, graph/node/port/timeline/product semantics, product/editor/game mutation, app composition, plugin framework work, new crates, or `foundation/meta`.

Complete investigation gate: complete for planning/design intake. It does not authorize implementation.

Complete design gate: proposed intake only. Implementation remains blocked until active planning records exact owners, exact files, complete implementation contract, allowed files/crates, forbidden files/crates, module decomposition map, principle compliance, validation envelope, evidence expectation, and stop conditions.

Next action: Review the SpatialCanvas design and investigation. Do not implement Phase 17 until explicit promotion to `active-implementation`.

## Rules

- Markdown must be enough to understand the current state.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
- Use `../workflow-lifecycle.md` before changing lifecycle state.
- Accepted direction does not authorize implementation.
