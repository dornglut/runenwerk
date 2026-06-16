---
title: Story-Gated Runtime Rendering Proof Implementation Plan
status: active
type: implementation-plan
wr: WR-177
milestone: PM-UI-STORY-004
---

# Story-Gated Runtime Rendering Proof

## Decision

Move the former static UI gallery rendering proof behind `UiStoryRunReport`. A gallery story may now claim visible preview/static mount success only when its story report contains passing `RenderPrimitives`, `RenderData`, `StaticMount`, `PreviewFrame`, and `MountEligibility` stages.

This slice does not introduce new renderer semantics or reusable component maturity. It adapts the existing button gallery proof path so rendering artifacts are evidence inside the story report, and the app submits only frames derived from story-derived mount eligibility.

## Exact Owners And Files

- `domain/ui/ui_story/src/report.rs`: keep the existing render/static-mount stage vocabulary inspectable for CLI and gallery reports.
- `domain/ui/ui_story/src/runner.rs`: continue deriving final verdict and mount eligibility after app-owned render proof stages are supplied.
- `domain/ui/ui_story/src/mount.rs`: keep mount eligibility derived only from final `UiStoryRunReport` verdict and story mount policy.
- `domain/ui/ui_story/src/gallery.rs`: update checked-in gallery pass stories to require render primitive, render data, static mount, preview-frame, and mount-eligibility stages and to become static-mount eligible only after those stages pass.
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs`: run the existing primitive/render-data/static-mount pipeline as story stage evidence; expose CLI inspection and runtime gallery frames from the same story reports.
- `apps/runenwerk_editor/src/bin/runenwerk_ui_gallery.rs`: keep CLI inspection wired to the story-gated reports.
- `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`: update the roadmap truth that the former standalone static gallery path is now story-gated.
- `docs-site/src/content/docs/domain/ui/roadmap.md`: record PM004 completion boundary and leave component/platform, designer, game HUD, and world-space work out of scope.

## Contract Requirements

- Rendering proof must be represented as story stages, not as app-local success state after a report is already complete.
- The app may submit a gallery `UiFrame` only from a story whose `UiStoryMountEligibility::from_report` is eligible.
- `UiStoryRunReport` must show source, runtime-view, render primitive, render-data, static-mount, preview-frame, mount-eligibility, diagnostics, and final verdict for the checked-in passing stories.
- Expected-failure stories must remain first-class and must not become mount eligible.
- Renderer-facing code may consume `UiFrame`/backend-neutral primitive artifacts; it must not own authored UI semantics or bypass story verdicts.

## Non-Goals

- No reusable component maturity, GraphCanvas, Timeline, rich text, interaction platform, Visual UI Builder, Designer Workbench product authoring, game HUD behavior, world-space UI, or entity-attached UI.
- No new crate creation.
- No renderer, ECS, editor, or game ownership of authored UI truth.
- No product host mounting beyond the standalone gallery proof frame.

## Implementation Steps

1. Update checked-in gallery story manifests to require the render/static-mount proof stages for passing stories and to grant `EligibleWhenPassed` mount policy only after those stages pass.
2. Move primitive/render-data/static-mount proof generation into the story execution path in `runenwerk_editor`.
3. Compose the runtime gallery frame only from story executions whose final story-derived mount eligibility is eligible.
4. Keep CLI inspection on the same story run path so the render/static-mount stages are visible outside the windowed gallery.
5. Add or update focused tests proving the gallery emits two eligible mounted stories, keeps the expected missing-source failure non-eligible, and includes render/static-mount stages in CLI output.
6. Update UI documentation to state that standalone static gallery proof has been replaced by story-gated rendering proof.

## Validation

- `cargo test -p ui_story`
- `cargo test -p ui_runtime_view`
- `cargo test -p ui_render_primitives`
- `cargo test -p ui_static_mount`
- `cargo test -p runenwerk_editor --bin runenwerk_ui_gallery`
- `cargo fmt --all --check`
- `task docs:validate`
- `task production:validate`
- `task roadmap:validate`

## Closeout Requirements

The PM-UI-STORY-004 closeout must include runtime_test, artifact, diagnostics, and visual evidence showing that checked-in gallery stories prove render/static-mount stages through `UiStoryRunReport` and that app frame submission is gated by `UiStoryMountEligibility`.

## Stop Conditions

- Stop without owning WR and accepted production plan.
- Stop if rendering proof bypasses story runner, report, or mount eligibility.
- Stop if renderer-owned UI semantics are introduced.
- Stop if component maturity, game HUD, world-space UI, or designer product authoring is implemented.
