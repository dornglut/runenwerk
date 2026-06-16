---
title: Gallery And CLI Story Execution Implementation Plan
status: active
type: implementation-plan
wr: WR-176
milestone: PM-UI-STORY-003
---

# Gallery And CLI Story Execution

## Decision

Implement the story-execution cutover for the checked-in UI gallery fixtures and the `runenwerk_ui_gallery` inspection command. The gallery and CLI must both run through `UiStoryManifest`, `UiStoryRegistry`, `UiStoryRunner`, and `UiStoryRunReport`; neither surface may keep a hardcoded button fixture list as its source of truth.

This slice is intentionally limited to story discovery, source loading, parse/formation/compiler/runtime-view reporting, CLI inspection, and gallery preview consumption of run reports. Runtime rendering/static-mount proof remains PM-UI-STORY-004 and must not be claimed here.

## Exact Owners And Files

- `domain/ui/ui_story/src/gallery.rs`: checked-in gallery story catalog helpers for deterministic manifest construction over the existing `assets/ui_gallery/button/*.ron` sources.
- `domain/ui/ui_story/src/cli.rs`: CLI-facing summary/diagnostic formatting over `UiStoryRunReport` without app, renderer, ECS, or filesystem ownership.
- `domain/ui/ui_story/src/lib.rs`: export the new `gallery` and `cli` modules for normal story inspection workflows.
- `domain/ui/ui_story/src/registry.rs`: keep deterministic registry access usable by the gallery/CLI slice; no renderer, ECS, editor, or game dependencies.
- `domain/ui/ui_story/src/runner.rs`: add a fail-closed way for app-owned proof stages to be folded into a runner-owned `UiStoryRunReport`.
- `domain/ui/ui_story/src/report.rs`: add only report helpers needed to inspect story status/stages from CLI and gallery code.
- `apps/runenwerk_editor/src/bin/runenwerk_ui_gallery.rs`: support CLI story inspection while preserving the existing window launch when no inspection argument is passed.
- `apps/runenwerk_editor/Cargo.toml`: declare the existing workspace `ui_story` dependency for the app adapter.
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs`: replace the hardcoded fixture source of truth with `ui_story` manifests and produce gallery diagnostics from `UiStoryRunReport`.
- `assets/ui_gallery/button/basic.ron` and `assets/ui_gallery/button/selected.ron`: remain the checked-in source fixtures referenced by manifests; edit only if source metadata needs to stay consistent.
- `docs-site/src/content/docs/domain/ui/roadmap.md`: record that gallery/CLI story execution now consumes story reports and that runtime rendering proof remains downstream.

## Public Contract Requirements

- The checked-in gallery stories must be discoverable through `UiStoryRegistry`, not through an app-local fixture array.
- The CLI inspection path must run the same story execution path as the gallery and print/report `UiStoryRunReport` data, including story id, verdict, source stage, diagnostics, and mount eligibility.
- Gallery preview must consume story reports before preparing any visual frame. If a story report fails, the gallery must surface diagnostics and skip that story's visual output.
- Failure stories must be first-class: an expected-failure story must expose source/stage diagnostics and pass only because the expected failure was observed.
- Required stages that PM003 does not prove must remain missing-proof or out-of-scope; PM003 must not fake static mount or preview-frame success.
- `ui_story` must stay domain-owned. App code may provide file IO and UI compilation stage evidence, but renderer/ECS/editor/game code must not own story semantics.

## Non-Goals

- No story-gated render primitive, render-data, static-mount, preview-frame, or product host eligibility proof. PM-UI-STORY-004 owns that.
- No reusable component maturity, GraphCanvas, Timeline, text platform, interaction platform, Visual UI Builder, Designer Workbench product authoring, game HUD behavior, or world-space/entity-attached UI.
- No new crate creation and no dependency from `ui_story` to app, renderer, ECS, compiler, controls, runtime view, filesystem, or RON parsing crates.
- No permanent compatibility path where the gallery can succeed without `UiStoryRunReport`.

## Implementation Steps

1. Add `gallery` and `cli` modules to `ui_story`, with deterministic checked-in story manifest construction and report-summary helpers.
2. Extend `UiStoryRunner` so app-owned stage reports can be included in the final runner-owned `UiStoryRunReport`.
3. Move the existing gallery fixture list into story manifests and build a registry from those manifests.
4. Update `runenwerk_editor` gallery source loading/formation/compiler/runtime-view code to produce story stage reports and run reports before adding a story to the preview frame.
5. Add CLI inspection handling in `runenwerk_ui_gallery` that emits the story run reports and exits nonzero when unexpected failures occur.
6. Add focused unit coverage for registry discovery, CLI summaries, fail-closed source errors, and app gallery report consumption.
7. Update the UI roadmap with the PM003 boundary and downstream PM004 rendering-proof dependency.

## Validation

- `cargo test -p ui_story`
- `cargo test -p runenwerk_editor --bin runenwerk_ui_gallery`
- `cargo fmt --all --check`
- `task docs:validate`
- `task production:validate`
- `task roadmap:validate`

## Closeout Requirements

The closeout for PM-UI-STORY-003 must include runtime_test, artifact, diagnostics, and source_maps evidence showing that gallery and CLI story execution consume `UiStoryRunReport`. It must name runtime rendering/static-mount proof as PM-UI-STORY-004 scope, not as a PM003 gap.

## Stop Conditions

- Stop without owning WR and accepted production plan.
- Stop if gallery or CLI can claim success without `UiStoryRunReport`.
- Stop if PM003 reports render primitive, render-data, static mount, preview-frame, or product host proof as completed.
- Stop if builder/product authoring UX is implemented here.
- Stop if component maturity, game HUD, or world-space behavior is implemented here.
