---
title: WR-179 Story Gallery Composition And Renderer Ordering Hardening
description: Implementation contract for hardening UI story gallery composition, renderer UI draw ordering, and border primitive lowering before PM-UI-STORY-005.
status: active
owner: ui
layer: domain/ui ui_render_data / editor gallery adapter / engine renderer
canonical: false
last_reviewed: 2026-06-17
related_designs:
  - ../../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../../design/active/ui-runtime-rendering-pipeline-roadmap.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/track-execution-manifests/pt-ui-story-platform.yaml
---

# WR-179 Story Gallery Composition And Renderer Ordering Hardening

## Goal

Execute `PM-UI-STORY-HARDEN-002` / `WR-179` as the final hardening prerequisite
before `PM-UI-STORY-005`.

This slice fixes the current gallery overlap by composing eligible story preview
frames into deterministic clipped tiles and hardens renderer UI ordering so
unrelated submissions or composed story surfaces cannot interleave rects,
glyphs, strokes, or surface embeds incorrectly.

## Architecture Governance

Kickoff command:

```text
task ai:architecture-governance -- --task "PM-UI-STORY-HARDEN-002 story gallery composition and UI renderer ordering hardening" --scope "domain/ui/ui_render_data/src/frame; apps/runenwerk_editor/src/runtime/ui_gallery.rs; engine/src/plugins/render/renderer; docs-site/src/content/docs/workspace"
```

Governance decision:

- `domain/ui/ui_render_data` owns backend-neutral frame composition contracts.
- `apps/runenwerk_editor` owns gallery adapter composition and may publish only
  story-derived, mount-eligible preview frames.
- `engine/src/plugins/render/renderer` owns backend draw ordering and primitive
  lowering, not authored UI semantics.
- No ADR is required for bounded composition helpers, renderer order fields, or
  border instance data. Require an ADR or accepted design update before moving
  authored UI semantics into the renderer or broadening this into component,
  designer, game HUD, or world-space UI work.

## Required Changes

- Add `domain/ui/ui_render_data/src/frame/composition.rs` and export
  `UiFramePlacement`, `UiFrameFragment`, and `compose_frame_fragments`.
- Compose eligible gallery story previews into clipped `320x128` tiles with
  `16` px padding, `12` px gaps, and columns derived from output width.
- Keep story execution, CLI inspection, and gallery mount eligibility driven by
  `UiStoryRunReport` plus `UiStoryMountEligibility`.
- Preserve renderer UI order by carrying submission, surface, layer, and
  primitive order through extraction, grouping, batching, and draw-plan sorting.
- Restrict batching to adjacent compatible work inside the same submission,
  surface, layer, scissor, and texture/source identity.
- Preserve `BorderPrimitive.width` into rect GPU instance data and render
  borders as outline rings when `border_width > 0`.

## Non-Goals

- Do not implement `PM-UI-STORY-005` product code. PM-005 remains closeout-only.
- Do not add visible gallery labels/status in this slice.
- Do not add screenshot capture, GPU pixel-golden tests, broader component
  maturity, Designer/Workbench UI, game HUD behavior, or world-space UI.

## Acceptance Criteria

- Two passing button stories produce non-overlapping preview button rects.
- Duplicate source `UiSurfaceId(0)` values do not leak into the composed gallery
  frame; the gallery publishes one output surface.
- Expected-failure stories do not mount, and failed reports still cannot publish
  preview frames.
- Renderer ordering preserves submission order, then surface order, then layer
  and primitive order before primitive-family tie-breaks.
- Border primitives preserve width into renderer data and no longer render as
  filled rectangles.

## Validation

```text
cargo fmt --check
cargo check --workspace
cargo test -p ui_render_data composition
cargo test -p runenwerk_editor --bin runenwerk_ui_gallery
cargo test -p runenwerk_editor story
cargo test -p engine extract_border_instances_preserves_border_width_and_order
cargo test -p engine rect_batch_grouping_splits_submission_and_surface_boundaries
cargo test -p engine draw_order_key_prioritizes_submission_surface_and_layer_before_family
cargo test -p engine ui
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
task planning:validate
```
