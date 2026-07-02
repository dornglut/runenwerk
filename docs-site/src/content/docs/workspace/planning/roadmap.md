---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
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

State: active planning for reusable renderer-neutral 2D coordinate and navigation surface contracts

Lifecycle state: `active-planning`

Authority: `ui-component-platform-surface2d-design.md`.

Evidence: Phase 15 Generic Text completed through PR #48 baseline and PR #49 hardening. Phase 16 planning starts from the existing Surface2D design, but that design must be refreshed before implementation to settle owner files, minimum implementation scope, validation envelope, no-mutation boundaries, accessibility/input acceptance, performance budget evidence, and the current relationship with existing `ui_surface` vocabulary. Typed App Composition is a proposed architecture reference only and does not authorize Phase 16 implementation.

Next action: Harden the Phase 16 design intake. Do not start implementation until planning records exact owner files, implementation scope, validation, evidence expectation, and stop conditions.

## Rules

- Markdown must be enough to understand the current state.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
- Use `../workflow-lifecycle.md` before changing lifecycle state.
- Accepted direction does not authorize implementation.
