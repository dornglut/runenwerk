---
title: Live UiPlugin Runtime Platform Architecture
description: Architecture and implementation-handoff model for the engine-owned Live UiPlugin runtime, render publication, agent-controllable Counter product, trace history, source reload, and state persistence boundaries.
status: active
owner: ui
layer: architecture
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ./ui-framework-architecture.md
  - ./diagrams/live-uiplugin-runtime-platform.puml
  - ./diagrams/live-uiplugin-runtime-sequence.puml
  - ../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../design/active/ui-runtime-rendering-pipeline-roadmap.md
  - ../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
---

# Live UiPlugin Runtime Platform Architecture

ID: `PT-UI-RUNTIME-PLATFORM-002` architecture handoff support.

This document owns the app/engine/render architecture diagrams and runtime platform answers that should not live only in the planning document.

## Current code facts inspected for this architecture

| Area | Current fact | Source path |
|---|---|---|
| App composition | `App` owns a `World`, scheduler, runner, mode, title, and control flow; `add_plugin`, `add_plugins`, resource insertion, render-flow registration, and `world()/world_mut()` are already public. | `engine/src/app/domain/app.rs` |
| Running apps | `App::run()` dispatches to windowed or headless mode; `run_for_frames` and `run_for_ticks` are headless helpers. | `engine/src/app/runtime/lifecycle.rs` |
| Windowed runtime | Windowed mode uses `winit_runner::run(self.into_windowed_state())`. | `engine/src/app/platform/windowed.rs` |
| Input/redraw loop | Winit keyboard, mouse, cursor, wheel, and touch events become platform/input events and request redraw on success. `RedrawRequested` runs the engine frame. | `engine/src/runtime/winit_runner.rs` |
| Frame pacing | Default policy is `ContinuousCapped { target_fps: 60 }`; `OnDemand` exists and has no continuous deadline. | `engine/src/runtime/frame_pacing.rs` |
| Render plugin ownership today | `RenderPlugin` initializes UI frame submission resources and runs `collect_runtime_ui_frame_submissions_system`, `prepare_ui_feature_resource_system`, frame prepare, and frame submit. | `engine/src/plugins/render/plugin.rs` |
| Legacy UI producer path today | Render runtime currently collects scene overlay and debug overlay UI frames directly from scene/debug resources. | `engine/src/plugins/render/runtime/ui_submission.rs` |
| Frame publication today | `UiFrameSubmissionRegistryResource` stores whole `UiFrameSubmission` values keyed by producer/surface; replacement is per producer/surface, not per element. | `engine/src/plugins/render/features/ui/submission.rs` |
| Frame preparation today | Frame prepare builds `PreparedRenderFrame` packets per render surface and applies UI contribution per surface. | `engine/src/plugins/render/runtime/frame_prepare.rs` |
| Frame submit today | Frame submit pulls the prepared frame, selects UI rect shader/font atlas inputs, and calls `gfx.render(...)`. | `engine/src/plugins/render/runtime/frame_submit.rs` |
| UI render payload today | `UiFrame` contains surfaces, surfaces contain layers, layers contain primitives. | `domain/ui/ui_render_data` |
| UI primitive model today | UI primitives are rect, border, glyph run, image, stroke, viewport-surface embed, product surface, and clip. | `domain/ui/ui_render_data/src/primitives/ui_primitive.rs` |

## Rendering answer: raster, SDF, and frame cadence

Current UI rendering is renderer-facing `UiFrame` primitive rendering, not SDF-owned UI semantics.

The inspected UI primitive model contains raster-style draw primitives: `Rect`, `Border`, `GlyphRun`, `Image`, `Stroke`, viewport-surface embeds, product-surface primitives, and clipping. The render submit path passes a prepared frame, an optional UI rect shader, and `UiFontAtlasResource` into `gfx.render(...)`.

SDF exists elsewhere in the render/world stack as render capability and future projection target. It does not currently own UI source, UI routes, UI actions, UI state, or UI primitive generation. The live UiPlugin cutover must keep SDF/world-space UI as a later target/projection consumer unless a separate target contract promotes it.

Current frame cadence is redraw-driven through winit with default continuous capped pacing. Because the default `FramePacingPolicyResource` is 60 FPS continuous, normal windowed runtime can redraw continuously. `OnDemand` mode exists and avoids continuous deadlines, and input events request redraw. The current inspected UI submission seam replaces a whole producer/surface `UiFrameSubmission`; it does not prove element-level incremental rendering.

Target frame policy for the cutover:

```text
1. Preserve the existing continuous capped mode for animated/runtime-heavy scenes.
2. Make non-animated UI capable of on-demand redraw.
3. Add generic dirty/invalidation records at screen, source, binding, layout, primitive, surface, and render-publication levels.
4. Do not claim element-level incremental render until a phase proves stable dirty scopes and backend support.
5. Do not rebuild/republish UI frames when no source, host data, input, layout, theme, text, or surface dependency changed, unless continuous animation policy requests it.
```

## Authoring, live changes, and hot reload

The canonical source truth remains `ui_definition` / `UiProgram`, not renderer primitives or app state.

Supported authoring directions:

| Authoring form | Runtime-platform decision |
|---|---|
| Rust typed screen/builder | Primary app-author path for `app.mount_ui(CounterScreen)`. This is the required path for the runtime Counter product. |
| RON/authored templates | Supported source format through `ui_definition`; should be usable for checked-in fixtures and later data-driven screens. |
| Visual designer output | Future product authoring surface; must save/export source IR, not renderer primitives or direct mutation logic. |
| Compiler DSL / reactive / immediate adapters | Future frontends only if they capture source records, route/action contracts, source maps, and proof facts. |

Hot reload decision:

```text
Rust code is not treated as hot-reloadable UI source.
Live UI changes are supported through reloadable data-backed source revisions: RON/templates, designer output, or future source IR files.
A reload must revalidate, re-lower, recompile/evaluate, preserve session state by stable source/runtime IDs where valid, and report any migration loss.
```

The Counter product may be Rust-authored, but the runtime platform must still design the source-revision mechanism so future RON/designer-authored screens can reload without a second runtime architecture.

## State persistence decision

State persistence is split by ownership:

| State kind | Owner | Persistence rule |
|---|---|---|
| App/domain state, such as `Counter` | App/host owner | Persist through explicit host-owned snapshot/load hooks. Generic UI must not own it. |
| UI session state, such as focus, hover, pressed, selected surface, scroll, input capture | UiPlugin session resources using domain UI contracts | May be snapshotted/replayed for tests and restored only by stable source/runtime IDs. |
| Source state, such as templates or designer output | UI source owner / product authoring owner | Persist as source IR, RON/template, or designer project output. |
| Render state | Render backend owner | Cache/pipeline state is not UI state and must not become user-state persistence. |

The runtime Counter product should demonstrate host-owned persistence with a small explicit state file option, not hidden automatic UI persistence:

```text
cargo run -p ui_counter_runtime -- --state-file target/ui_counter_runtime/counter.state.ron
```

## Agent-controllable runtime product

The runnable Counter app must serve both humans and agents.

Required modes:

| Mode | Command shape | Purpose |
|---|---|---|
| Human window | `cargo run -p ui_counter_runtime` | Opens a native window and supports pointer interaction. |
| Agent/headless script | `cargo run -p ui_counter_runtime -- --headless --agent-script assets/ui_counter_runtime/scripts/increment_reset.ron --trace-jsonl target/ui_counter_runtime/trace.jsonl --exit-after-script` | Lets a simple agent drive actions deterministically and inspect machine-readable output. |
| Deterministic test | `cargo test -p ui_counter_runtime` plus focused engine tests | Proves source/program/action/mutation/render facts without manual UI. |

Agent scripts should name semantic actions and optional pointer gestures. They must resolve through the same route/capability/payload validation path as human interaction. They must not call `Counter` mutation directly.

## Runtime trace, history, and console visibility

The platform must have a generic trace/history model, not Counter-specific logging.

Required event families:

```text
UiRuntimeMounted
UiSourceRevisionLoaded
UiSourceLowered
UiProgramFormed
UiSessionStateRestored
UiInputObserved
UiHitTestResolved
UiRouteProposed
UiCapabilityChecked
UiActionDispatched
UiHostMutationApplied
UiHostMutationRejected
UiStateSnapshotWritten
UiRuntimeEvaluated
UiFramePublished
UiFramePresented
UiRuntimeDiagnostic
```

Trace requirements:

```text
Every event has monotonic sequence, frame index if available, screen id, source id, route id where applicable, capability verdict, host id, result, and diagnostic code.
Trace is available as an in-memory bounded ring buffer resource.
Trace can be exported as JSONL for agents and CI.
Trace has a human-readable console/debug surface in the Counter product.
Trace must be source-map aware where source identity exists.
Trace must not expose renderer internals as UI source truth.
```

This trace is the basis for both agent use and human debugging. The Counter app should show a small console/history panel or overlay with the last actions and diagnostics.

## Architecture diagrams

PlantUML source files:

```text
docs-site/src/content/docs/architecture/diagrams/live-uiplugin-runtime-platform.puml
docs-site/src/content/docs/architecture/diagrams/live-uiplugin-runtime-sequence.puml
```

These diagrams are architecture artifacts. Planning documents should link to them instead of embedding the architecture diagrams inline.

## Stop conditions

Stop implementation and update planning if a phase requires:

```text
renderer-owned UI source/action/host semantics
SDF-owned UI source/action/host semantics
permanent old UI runtime path
agent scripts mutating host state directly
Counter-specific tracing instead of generic runtime trace events
state persistence hidden inside generic UI controls
Rust-code hot reload claims
per-element incremental rendering claims without dirty-scope proof
```
