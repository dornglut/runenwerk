---
title: UI Substrate Roadmap
description: Phased roadmap for Runenwerk UI substrate ownership correction and maturation from the current audited state.
---

# UI Substrate Roadmap

## Purpose
Define the implementation roadmap for Runenwerk UI substrate work from the audited current repository state, with explicit ownership correction and dependency-aware sequencing.

This roadmap is not a speculative product roadmap. It is an architecture and implementation sequencing document grounded in current code reality.

## Current Status Summary

- `domain/ui/*` now contains both primitive and runtime substrate crates:
  - `ui_tree`
  - `ui_runtime`
  - `ui_widgets`
- `editor_shell` correctly owns workspace/tool-surface host architecture.
- `runenwerk_editor` owns app/runtime glue and viewport runtime bindings.
- engine render integration for `UiFrame` submission/extraction is in place.
- fallback seams (`first_frame`, `ViewportId(0)`) are removed from active routing paths.
- interaction substrate coverage now includes keyboard/text routing, Tab/Shift-Tab traversal, and explicit invalidation contract output.
- baseline unit coverage now exists across UI primitive/substrate crates.
- duplicated engine overlay/debug UI stacks now route through substrate frame generation.
- substrate frame snapshot tests and a lightweight gallery harness target are now present.

## Architectural Constraints

- preserve domain boundary direction from repository architecture doctrine.
- keep workspace host semantics in `editor_shell`; do not move host semantics into generic substrate crates.
- move reusable retained runtime ownership under `domain/ui` without changing semantic ownership of editor-specific logic.
- avoid documenting or coding fallback-based routing as final architecture.
- prefer contract normalization and verification before broad feature expansion.

## Phased Roadmap

## Phase 1 - Ownership correction and runtime extraction

### Status
Complete

### Goal
Move reusable retained runtime ownership from `editor_shell` into `domain/ui` runtime-oriented crates/modules while preserving behavior.

### Why this order
Every downstream substrate improvement depends on correct ownership boundaries. Without this extraction, new work keeps accumulating in the wrong owner.

### Concrete target areas/files/modules

- source owner now:
  - `domain/editor/editor_shell/src/runtime/*`
- target owner direction:
  - new runtime-oriented crate/module under `domain/ui` (for retained tree/runtime/layout/input/output/widgets)
- touch points requiring compatibility checks:
  - `domain/editor/editor_shell/src/lib.rs`
  - `apps/runenwerk_editor/src/shell/state.rs`
  - `apps/runenwerk_editor/src/shell/controller.rs`

### Done-when criteria

- reusable runtime modules no longer live under `editor_shell` runtime subtree.
- `editor_shell` composes/consumes runtime substrate rather than owning it.
- existing editor shell runtime tests/smokes remain green after extraction.

### Phase non-goals

- no redesign of workspace host semantics.
- no new broad control set introduction in this phase.
- no docking/floating UX feature expansion.

## Phase 2 - Contract normalization and fallback removal

### Status
Complete (with explicit bootstrap-only single-viewport seam before first structural binding).

### Goal
Remove fallback seams and normalize duplicated UI/viewport binding contracts.

### Why this order
After ownership is corrected, contract clarity must be enforced before adding behavior breadth. Otherwise new behavior builds on inconsistent identity/binding assumptions.

### Concrete target areas/files/modules

- fallback seams:
  - `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`
  - `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs`
  - `apps/runenwerk_editor/src/shell/viewport_adapter.rs`
- duplicate slot/binding contract surfaces:
  - `domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs`
  - `domain/editor/editor_viewport/src/expression/surface_set.rs`
  - `apps/runenwerk_editor/src/runtime/viewport/surface_set.rs`
  - `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs`

### Done-when criteria

- no active `first_frame`-driven routing fallback in editor runtime systems.
- no active `ViewportId(0)` fallback in viewport observation/view-model path.
- slot/binding mapping path is canonicalized and tested with clear owner boundaries.

### Phase non-goals

- no keyboard/focus behavior expansion yet.
- no full renderer architecture redesign.

## Phase 3 - Interaction completeness (keyboard/text/focus/invalidation)

### Status
Complete for baseline routing/traversal/invalidation contracts.

### Goal
Complete core retained-runtime interaction model beyond pointer-only behavior.

### Why this order
Interaction completeness should come after ownership and contracts are stable; otherwise interaction features are implemented against unstable boundaries.

### Concrete target areas/files/modules

- runtime dispatch and state:
  - retained runtime `dispatch_input` keyboard/text path
  - runtime focus traversal baseline (Tab/Shift-Tab)
- input contracts:
  - `domain/ui/ui_input/src/*`
- invalidation semantics:
  - explicit `UiInputOutcome.invalidation` result contract

### Done-when criteria

- keyboard and text input route through retained runtime tree paths.
- focus traversal behavior is explicit and test-covered.
- relayout/repaint invalidation semantics are explicit and verified in runtime tests.

### Phase non-goals

- no broad visual styling system expansion.
- no accessibility productization beyond foundational hooks.

## Phase 4 - Reusable controls needed by existing surfaces

### Status
In progress (substrate control primitives implemented; broad editor-surface migration still pending).

### Goal
Introduce and migrate the minimum reusable control set required by current editor surfaces.

### Why this order
Controls should be built after core interaction model is complete, so control behavior does not require rework from missing focus/keyboard/invalidation infrastructure.

### Concrete target areas/files/modules

- control construction/runtime modules in domain-owned UI substrate (target owner)
- existing shell composition consumers:
  - `domain/editor/editor_shell/src/composition/build_outliner_panel.rs`
  - `domain/editor/editor_shell/src/composition/build_inspector_panel.rs`
  - `domain/editor/editor_shell/src/composition/build_viewport_panel.rs`
  - `domain/editor/editor_shell/src/composition/build_console_panel.rs`

### Done-when criteria

- reusable control primitives exist for:
  - text input
  - toggle/checkbox
  - numeric input
  - tabs
- control interaction/layout/render behavior is test-covered.
- current editor surfaces use these controls where ad hoc equivalents remain.

### Phase non-goals

- no speculative full widget catalog.
- no broad UX productization beyond current surface needs.

## Phase 5 - Converge duplicated UI stacks where justified

### Status
Complete

### Goal
Reduce duplicated ad hoc UI runtime paths by reusing shared substrate where practical.

### Why this order
Convergence should happen only after substrate ownership and core behavior are stable, to avoid coupling scene/debug flows to moving internals.

### Concrete target areas/files/modules

- duplicate stack sites:
  - `engine/src/plugins/debug_metrics/mod.rs`
  - `engine/src/plugins/scene/runtime/overlay_ui.rs`
- shared integration seam:
  - `ui_runtime::build_ui_frame` and retained node/layout contracts
  - engine UI submission/render feature paths under `engine/src/plugins/render/features/ui/*`

### Done-when criteria

- duplicated ad hoc text/layout/frame assembly paths are reduced where feasible.
- duplicated debug metrics and scene overlay ad hoc text/layout/frame assembly paths are removed in favor of substrate frame generation.
- shared substrate is used for common behavior while feature-specific semantics stay local.
- render submission behavior remains stable under existing smoke tests.

### Phase non-goals

- no forced migration of every runtime overlay path in one step.
- no disruption of feature-local semantics to satisfy abstraction purity.

## Phase 6 - Testing/gallery/docs hardening

### Status
Complete

### Goal
Establish durable verification and documentation support for ongoing UI substrate evolution.

### Why this order
Hardening should follow architectural and behavior stabilization so tests/docs lock in a coherent model.

### Concrete target areas/files/modules

- UI primitive crates currently under-tested:
  - `domain/ui/ui_math`
  - `domain/ui/ui_input`
  - `domain/ui/ui_layout`
  - `domain/ui/ui_theme`
  - `domain/ui/ui_render_data`
- runtime/substrate integration test targets:
  - retained runtime interaction paths
  - frame snapshot expectations
- gallery/harness target:
  - `domain/ui/ui_runtime/examples/substrate_gallery.rs`
- docs:
  - `docs-site/src/content/docs/domain/ui/architecture.md`
  - `docs-site/src/content/docs/domain/ui/roadmap.md`
  - `domain/ui/README.md`

### Done-when criteria

- baseline unit coverage is added for core UI primitive/substrate crates.
- interaction-level verification is codified in repeatable runtime tests.
- frame output snapshot expectations exist in substrate output tests.
- docs remain aligned with implemented state and no longer claim removed fallback seams as unresolved.
- lightweight gallery target is added for substrate scenarios.

### Phase non-goals

- no broad documentation tree refactor outside UI area.
- no speculative roadmap expansion into unrelated product tracks.

## Explicit Roadmap Non-Goals

- do not treat ownership extraction as complete before code moves.
- do not restart sequencing from stale assumptions that contradict current code.
- do not prioritize broad feature breadth over ownership and contract correctness.
- do not present fallback seam removal as already done until tests and code confirm it.
