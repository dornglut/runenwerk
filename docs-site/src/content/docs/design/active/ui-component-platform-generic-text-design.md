---
title: UI Component Platform Generic Text Design
description: Phase 15 design intake for reusable renderer-neutral text display and layout proof.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-02
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./ui-component-platform-text-editing-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
---

# UI Component Platform Generic Text Design

Lifecycle state: `active-planning`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-015`.

Generic Text is the reusable renderer-neutral text display and layout proof for the UI Component Platform. It is display/layout infrastructure. It is not text editing, not a rich text editor, not a code editor, and not a product document surface.

## Decision summary

Text display/layout should become package-visible, inspectable, proof-frame-visible, and static-mount-validatable in the same owner-first style as generic interaction, overlay/layering, and editable text.

The phase should prove that reusable controls can declare and expose text display requirements without requiring a renderer backend, product document buffer, authored UI editor, UI Gallery product surface, or app-specific text rendering policy.

## Problem

Current text evidence exists in limited renderer-neutral primitives and proof frames. That is not yet a reusable Generic Text platform proof.

Phase 15 must make text display/layout behavior explicit enough that future controls can rely on consistent text runs, inline spans, wrapping, alignment, truncation, line metrics, glyph/run evidence, catalog projection, inspection projection, visual proof, and static mount proof.

The failure mode to avoid is treating a simple glyph run as the whole text platform. Generic Text must prove layout behavior and evidence, not only character emission.

## Goals

- Define reusable renderer-neutral text display/layout vocabulary.
- Cover text runs and inline spans without implementing rich text editing.
- Cover wrapping, alignment, truncation, and ellipsis behavior.
- Cover line metrics including baseline, line-height evidence, and measured content size.
- Emit glyph/run evidence that is stable enough for visual proof and static mount validation.
- Make generic text capability visible through package validation, catalog projection, and inspection projection.
- Provide a renderer-neutral visual proof frame that shows text layout state and evidence.
- Add static mount proof for the generic text frame.
- Preserve clean owner boundaries between `ui_text`, `ui_controls`, `ui_runtime`, `ui_static_mount`, renderer backends, and product/editor/game layers.
- Avoid public phase-shaped API names and compatibility-only aliases/shims.

## Explicit non-goals

This phase does not implement or authorize text editing, rich text editor behavior, code editor behavior, product document buffers, undo/redo, clipboard integration, LSP/syntax highlighting, app-specific text rendering policy, renderer backend implementation, dynamic plugin framework, `foundation/meta`, shared plugin primitives, UI Designer, UI Gallery product surface, Workbench/provider redesign, product/editor/game mutation, command execution, authored UI editing, compatibility-only aliases/shims, or phase-shaped public API names.

## Owner boundaries

`ui_text` owns renderer-neutral text display/layout contracts and evidence vocabulary. Phase 15 planning should inspect whether current `TextLayoutRequest`, `GlyphRun`, `PositionedGlyph`, `TextStyle`, wrapping, alignment, overflow, and atlas-backed layout contracts are sufficient or need clean cutover changes.

`ui_controls` owns package-backed generic-text declarations, package validation, catalog projection, and inspection projection for reusable controls that display text.

`ui_runtime` owns renderer-neutral generic-text proof reporting and proof-frame projection.

`ui_static_mount` owns static validation of the renderer-neutral generic-text proof frame.

Renderer backends may later consume text layout/glyph evidence, but backend implementation is not Phase 15 scope.

Host/product/editor/game layers own app-specific copy, localization policy, persistence, document buffers, authored UI editing, code editing, undo/redo, commands, and domain mutation.

## Planning scope

The accepted implementation scope should be limited to a vertical proof for:

```text
ui_text text run/span/layout vocabulary
  -> ui_controls package-backed generic-text declaration
  -> package validation
  -> catalog projection
  -> inspection projection
  -> generic text layout evidence
  -> generic text visual proof
  -> GenericTextProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

Exact file scope must be recorded before implementation starts. Candidate owner areas to inspect during planning:

```text
domain/ui/ui_text/src/layout.rs
domain/ui/ui_text/src/style.rs
domain/ui/ui_text/src/buffer.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_runtime/src/
domain/ui/ui_static_mount/tests/
```

These candidates are inspection targets for the design intake, not blanket authorization to edit all of them.

## Required design details before implementation

Before moving to `active-implementation`, the design must specify:

- stable public vocabulary for text runs and inline spans;
- exact wrapping model;
- exact alignment model;
- truncation and ellipsis policy;
- line metric evidence;
- glyph/run evidence needed for visual proof;
- package descriptor shape and validation failure modes;
- catalog facts and inspection facts;
- visual proof structure and static mount expectations;
- tests that prove positive, negative, and no-bypass behavior;
- explicit stop conditions.

## Proof requirements

Phase 15 should prove:

- at least one package-backed text display declaration;
- at least one simple text run;
- at least one inline span scenario;
- no-wrap layout;
- wrapping layout;
- horizontal alignment;
- truncation and ellipsis;
- line metrics and measured size evidence;
- glyph/run evidence with stable draw order;
- catalog projection;
- inspection projection;
- visual proof frame with text layout evidence;
- static mount acceptance of the renderer-neutral proof frame;
- rejection or diagnostics for unsupported text layout declarations;
- no product/editor/game mutation;
- no command execution;
- no authored UI editing;
- no text editing behavior;
- no dynamic plugin or `foundation/meta` dependency.

## Ergonomics requirements

Generic Text should be easy for future control authors to consume. The implementation should avoid repeated manual layout/proof boilerplate by providing narrow builders, fixtures, or helper constructors where they improve clarity.

The design should prefer a clean cutover if existing text contracts are too narrow. Do not add compatibility-only aliases or maintain parallel legacy vocabulary.

## Stop conditions

Stop and redesign if Phase 15 requires text editing behavior, rich text editor behavior, code editor behavior, product document buffer ownership, command execution inside generic UI, product/editor/game mutation, authored UI editing, UI Designer, UI Gallery product surface, Workbench/provider redesign, renderer backend ownership inside `ui_text`, `ui_controls`, or `ui_runtime`, dynamic plugin framework, `foundation/meta`, shared plugin primitives, compatibility-only aliases/shims, or phase-shaped public API names.

## Validation planning

Docs-only planning changes must pass:

```text
python tools/docs/validate_docs.py
git diff --check
```

A later implementation gate must be recorded before promotion to `active-implementation`. It should include focused checks/tests for `ui_text`, `ui_controls`, `ui_runtime`, `ui_static_mount`, `ui_story`, package/catalog/inspection projection, visual proof, and static mount proof.

## Relationship to completed work

Phase 14 is completed through PR #46 and remains the editable-text behavior proof. Phase 15 builds adjacent display/layout infrastructure. It must not reopen Phase 14 text editing scope.
