---
title: PM-UI-STORY-HARDEN-002 Story Gallery Composition And Renderer Ordering Hardening Closeout
status: completed
closeout_evidence:
  milestone_id: PM-UI-STORY-HARDEN-002
  wr_id: WR-179
  completion_quality: runtime_proven
  evidence_categories:
    - runtime_test
    - composition
    - renderer_ordering
    - border_width
    - gallery_no_bypass
    - governance
  validation_commands:
    - cargo fmt --check
    - cargo check --workspace
    - cargo test -p ui_render_data composition
    - cargo test -p runenwerk_editor --bin runenwerk_ui_gallery
    - cargo test -p runenwerk_editor story
    - cargo test -p engine extract_border_instances_preserves_border_width_and_order
    - cargo test -p engine rect_batch_grouping_splits_submission_and_surface_boundaries
    - cargo test -p engine draw_order_key_prioritizes_submission_surface_and_layer_before_family
    - cargo test -p engine ui
    - task production:render
    - task production:validate
    - task production:check
    - task roadmap:render
    - task roadmap:validate
    - task roadmap:check
    - task docs:validate
    - task planning:validate
  validation_results:
    - 'architecture_governance (task ai:architecture-governance -- --task "PM-UI-STORY-HARDEN-002 story gallery composition and UI renderer ordering hardening" --scope "domain/ui/ui_render_data/src/frame; apps/runenwerk_editor/src/runtime/ui_gallery.rs; engine/src/plugins/render/renderer; docs-site/src/content/docs/workspace") -> exit 0'
    - 'cargo:fmt (cargo fmt --check) -> exit 0'
    - 'cargo:check (cargo check --workspace) -> exit 0'
    - 'cargo:test (cargo test -p ui_render_data composition) -> exit 0'
    - 'cargo:test (cargo test -p runenwerk_editor --bin runenwerk_ui_gallery) -> exit 0'
    - 'cargo:test (cargo test -p runenwerk_editor story) -> exit 0'
    - 'cargo:test (cargo test -p engine extract_border_instances_preserves_border_width_and_order) -> exit 0'
    - 'cargo:test (cargo test -p engine rect_batch_grouping_splits_submission_and_surface_boundaries) -> exit 0'
    - 'cargo:test (cargo test -p engine draw_order_key_prioritizes_submission_surface_and_layer_before_family) -> exit 0'
    - 'cargo:test (cargo test -p engine ui) -> exit 0'
    - 'task:production:render (task production:render) -> exit 0'
    - 'task:truth:certify (PT-UI-PROGRAM-ARCHITECTURE ui-program-architecture-implementation) -> exit 0'
    - 'task:truth:certify (PT-UI-PROGRAM-ARCHITECTURE retained-ui-compatibility) -> exit 0'
    - 'task:truth:certify (PT-UI-PROGRAM-ARCHITECTURE ui-program-perfectionist-conformance) -> exit 0'
    - 'task:production:complete-track-contracts (PT-UI-PROGRAM, PT-TRACK-EXECUTION-HARNESS, PT-UI-PROGRAM-ARCHITECTURE, PT-UI-STORY-PLATFORM) -> exit 0'
    - 'task:production:validate (task production:validate) -> exit 0'
    - 'task:production:check (task production:check) -> exit 0'
    - 'task:roadmap:render (task roadmap:render) -> exit 0'
    - 'task:roadmap:validate (task roadmap:validate) -> exit 0'
    - 'task:roadmap:check (task roadmap:check) -> exit 0'
    - 'task:docs:validate (task docs:validate) -> exit 0'
    - 'task:planning:validate (task planning:validate) -> exit 0'
  files_changed:
    - domain/ui/ui_render_data/src/frame/composition.rs
    - domain/ui/ui_render_data/src/frame/mod.rs
    - apps/runenwerk_editor/src/runtime/ui_gallery.rs
    - apps/runenwerk_editor/src/bin/runenwerk_ui_gallery.rs
    - engine/src/plugins/render/renderer/mod.rs
    - engine/src/plugins/render/renderer/extract.rs
    - engine/src/plugins/render/renderer/prepare.rs
    - engine/src/plugins/render/renderer/setup.rs
    - docs-site/src/content/docs/reports/implementation-plans/wr-179-story-gallery-composition-and-renderer-ordering-hardening/plan.md
    - docs-site/src/content/docs/reports/closeouts/pm-ui-story-harden-002-story-gallery-composition-and-renderer-ordering-hardening/closeout.md
    - docs-site/src/content/docs/workspace/production-tracks.yaml
    - docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-story-platform.yaml
    - docs-site/src/content/docs/workspace/roadmap-items.yaml
  known_gaps:
    - Rich visible gallery labels/status, screenshot capture, GPU pixel-golden tests, broader component maturity, Designer/Workbench UI, game HUD behavior, and world-space/entity-attached UI remain out of WR-179 scope.
  closeout_path: docs-site/src/content/docs/reports/closeouts/pm-ui-story-harden-002-story-gallery-composition-and-renderer-ordering-hardening/closeout.md
  produced_at: 2026-06-17T00:00:00Z
---

# PM-UI-STORY-HARDEN-002 Story Gallery Composition And Renderer Ordering Hardening Closeout

WR-179 completed the gallery composition and renderer ordering hardening
prerequisite before `PM-UI-STORY-005`.

The implementation adds a backend-neutral `ui_render_data` frame-fragment
composition contract. The editor gallery now runs checked-in stories at the
fixed proof size and composes only mount-eligible preview frames into clipped
tiles sized for the current render target. Expected-failure stories remain
non-mounted, and failed reports still cannot publish a preview frame even when
mounted-frame data is present.

The renderer now carries submission order, composed surface order, layer order,
and primitive order through UI extraction, grouping, batching, and draw-plan
sorting. Primitive-family order is only a tie-breaker after those authored
ordering dimensions, preventing rect batches from later story surfaces from
drawing ahead of earlier glyphs.

Border primitives now preserve `BorderPrimitive.width` into the rect GPU
instance stream. The rect shader keeps filled-rect behavior when
`border_width == 0` and renders an outline ring when `border_width > 0`.
