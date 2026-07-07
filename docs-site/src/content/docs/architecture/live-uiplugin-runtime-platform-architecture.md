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
| Render plugin ownership today | `RenderPlugin` initializes generic surface-frame submission resources and still runs the legacy scene/debug collection system until Phase 011 removes it. It also runs `prepare_ui_feature_resource_system`, frame prepare, and frame submit. | `engine/src/plugins/render/plugin.rs` |
| Legacy UI producer path today | Render runtime currently collects scene overlay and debug overlay UI frames directly from scene/debug resources. | `engine/src/plugins/render/runtime/ui_submission.rs` |
| UiPlugin publication today | `UiPlugin` publishes evaluated runtime frames through `SurfaceFrameSubmissionRegistryResource` with `RenderFrameProducerId` and `RenderSurfaceId`; `RenderPlugin` consumes the prepared packet without querying screens, sources, actions, host mutation, or route policy. | `engine/src/plugins/ui/render_publish.rs` |
| Frame publication today | `SurfaceFrameSubmissionRegistryResource` stores whole `SurfaceFrameSubmission` values keyed by producer/surface; replacement is per producer/surface, not per element. | `engine/src/plugins/render/features/ui/submission.rs` |
| Frame preparation today | Frame prepare builds `PreparedRenderFrame` packets per render surface and applies UI contribution per surface. | `engine/src/plugins/render/runtime/frame_prepare.rs` |
| Frame submit today | Frame submit pulls the prepared frame, selects UI rect shader/font atlas inputs, and calls `gfx.render(...)`. | `engine/src/plugins/render/runtime/frame_submit.rs` |
| UI render payload today | `UiFrame` contains surfaces, surfaces contain layers, layers contain primitives. | `domain/ui/ui_render_data` |
| UI primitive model today | UI primitives are rect, border, glyph run, image, stroke, viewport-surface embed, product surface, and clip. | `domain/ui/ui_render_data/src/primitives/ui_primitive.rs` |

## Rendering answer: raster, SDF, and frame cadence

Current UI rendering is renderer-facing `UiFrame` primitive rendering, not SDF-owned UI semantics.

The inspected UI primitive model contains raster-style draw primitives: `Rect`, `Border`, `GlyphRun`, `Image`, `Stroke`, viewport-surface embeds, product-surface primitives, and clipping. The render submit path passes a prepared frame, an optional UI rect shader, and `UiFontAtlasResource` into `gfx.render(...)`.

SDF exists elsewhere in the render/world stack as render capability and future projection target. It does not currently own UI source, UI routes, UI actions, UI state, or UI primitive generation. The live UiPlugin cutover must keep SDF/world-space UI as a later target/projection consumer unless a separate target contract promotes it.

Current frame cadence is redraw-driven through winit with default continuous capped pacing. Because the default `FramePacingPolicyResource` is 60 FPS continuous, normal windowed runtime can redraw continuously. `OnDemand` mode exists and avoids continuous deadlines, and input events request redraw. The current inspected surface-frame submission seam replaces a whole producer/surface `SurfaceFrameSubmission`; it does not prove element-level incremental rendering.

Target frame policy for the cutover:

```text
1. Preserve the existing continuous capped mode for animated/runtime-heavy scenes.
2. Make non-animated UI capable of on-demand redraw.
3. Add generic dirty/invalidation records at screen, source, binding, layout, primitive, surface, and render-publication levels.
4. Do not claim element-level incremental render until a phase proves stable dirty scopes and backend support.
5. Do not rebuild/republish UI frames when no source, host data, input, layout, theme, text, or surface dependency changed, unless continuous animation policy requests it.
```

## Render-boundary generalization decision

The generic render submission boundary must be corrected before UiPlugin publishes durable runtime frames.

Decision:

```text
Move producer-generic surface-frame semantics before UiPlugin render publication.
Do not let new UiPlugin runtime code stabilize on UI-specific render ownership names if those names are already known to be wrong.
RenderPlugin consumes producer/surface/frame packets; UiPlugin is one producer, not the render-frame owner.
```

Required boundary shape:

| Concept | Long-term owner | Rule |
|---|---|---|
| Source/program/action/session semantics | `domain/ui` plus `engine::plugins::ui` integration | Must not move into render. |
| Producer identity | Engine/runtime producer contract | UI, debug overlays, scene overlays, product surfaces, and future producers publish as producers. |
| Surface/frame packet | Render-facing producer-generic contract | Must not encode `UiPlugin` as the owner of the generic frame model. |
| Render preparation/submission | `RenderPlugin` | Consumes packets; does not query screens, source, route, host state, or actions. |

Implementation implication:

```text
The cutover order should create the generic producer/surface-frame seam before the UiPlugin render-publication phase.
Old scene/debug overlay migration then targets the generic producer path instead of a UI-specific transitional path.
```

This is a long-term cleanup decision, not a broad render rewrite authorization. The genericization phase may touch only the render frame/submission seams named by its phase contract and must not alter source/program/action semantics.

## SDF UI future-backend position

SDF UI is possible, but it is not part of the live UiPlugin runtime cutover. It belongs to a separate render-backend/projection design after the runtime product path is proven.

Recommended future track:

```text
PT-UI-RENDER-BACKEND-SDF-001 — Analytical SDF UI Primitive Backend
```

Recommended first SDF scope:

```text
RectPrimitive -> rounded rectangle SDF shader
BorderPrimitive -> SDF border / outline
optional shadow / glow primitive parameters
current glyph atlas text path retained
no route/action/source ownership
no world-space UI ownership
no source reload or designer ownership
```

Reasoning:

```text
SDF is a render strategy for derived primitives, not a UI semantic model.
The first runtime product must prove source/program/action/host/render-publication ownership before adding another render backend.
Text fidelity, shaping, accessibility, and source maps are already difficult enough without making the first product proof an SDF backend proof.
```

Later SDF work may explore MSDF text/icons, world-space panels, holographic UI, glow/soft-edge effects, and animated shader parameters. Those must consume `UiFrame`/SurfaceFrame-style output and must not own UI source, routes, actions, host mutation, or runtime session truth.

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

The runtime Counter product should demonstrate host-owned persistence with a small explicit state file option only after the persistence phase is accepted, not as hidden automatic UI persistence:

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

Agent script format decision:

```text
RON is acceptable for the first repo-native fixture format because Runenwerk already uses RON-style fixtures.
JSONL remains the required trace output format for agents and CI.
Support for JSONL agent input may be added later if cross-tool agent integration needs it, but the first Counter product must not block on dual input formats.
```

## Counter product screen contract

The Counter product is not allowed to leave the screen shape to implementation guesswork.

Required app identity:

```text
binary: ui_counter_runtime
window title: Runenwerk UI Counter Runtime
mounted screen type: CounterScreen
host plugin type: CounterPlugin
host resource: Counter { value: i64 }
initial value: 0 unless loaded from explicit host-owned state file after the persistence phase exists
```

Required visible structure:

```text
root surface: CounterScreen
root layout: vertical stack or equivalent semantic container
header label: Runenwerk UI Counter Runtime
count label: Count: {value}
action row: Increment, Decrement, Reset
trace console: last N runtime trace entries, newest last or clearly ordered
status line: last action result or diagnostic summary
```

Required semantic actions and routes:

| UI control | Route id | Required capability | Payload | Host mutation |
|---|---|---|---|---|
| Increment button | `counter.increment` | `counter.write` | none or unit payload | `Counter.value += 1` |
| Decrement button | `counter.decrement` | `counter.write` | none or unit payload | `Counter.value -= 1` |
| Reset button | `counter.reset` | `counter.write` | none or unit payload | `Counter.value = 0` |

Required read capability:

```text
counter.read permits rendering the current count.
counter.write permits mutating count through the three actions.
missing or rejected capability must not mutate Counter.
```

Required agent script semantics:

```text
semantic action names resolve to the same route ids as visible controls
optional scripted pointer activation must hit-test to the same route ids
agent scripts cannot mutate Counter directly
JSONL trace is the machine-readable source of action/mutation/evaluation/frame evidence
```

Required visible behavior:

```text
Increment updates visible count by +1.
Decrement updates visible count by -1.
Reset updates visible count to 0.
Every accepted action adds a trace console row.
Every rejected action adds a diagnostic trace row and leaves count unchanged.
```

## Runtime trace, history, and console visibility

The platform must have a UI-runtime trace/history model, not Counter-specific logging and not a premature engine-wide tracing framework.

Ownership decision:

```text
Trace starts in engine::plugins::ui because the first required events are UI-semantic events.
Do not extract a cross-engine tracing framework during this cutover.
Counter consumes and displays trace; it does not define the trace model.
```

Phase 007 minimum event set:

```text
UiRuntimeMounted
UiInputObserved
UiRouteProposed
UiCapabilityChecked
UiActionDispatched
UiHostMutationApplied
UiHostMutationRejected
UiRuntimeDiagnostic
```

Later phase event additions:

| Phase | Added event families |
|---|---|
| Phase 008 | `UiRuntimeEvaluated`, state/invalidation trace facts, optional `UiSessionStateRestored` when session replay is implemented. |
| Phase 010 | `UiFramePublished`, `UiFramePresented`. |
| Phase 013 | `UiSourceRevisionLoaded`, `UiSourceLowered`, `UiProgramFormed`, `UiStateSnapshotWritten`. |

Trace requirements:

```text
Every event has monotonic sequence, frame index if available, screen id, source id where applicable, route id where applicable, capability verdict where applicable, host id where applicable, result, and diagnostic code where applicable.
Trace is available as an in-memory bounded ring buffer resource.
Trace can be exported as JSONL for agents and CI.
Trace has a human-readable console/debug surface in the Counter product.
Trace must be source-map aware where source identity exists.
Trace must not expose renderer internals as UI source truth.
```

This trace is the basis for both agent use and human debugging. The Counter app should show a small console/history panel or overlay with the last actions and diagnostics, but the trace subsystem remains generic UI runtime infrastructure.

## Phase implementation spec decision

Runenwerk should adopt lightweight phase implementation specs, but not as a blocking requirement for this planning PR.

Decision:

```text
Use human-readable Markdown for design rationale and architecture.
Use a compact machine-checkable phase spec later as the compiled implementation handoff contract.
Prefer RON for repo-native phase specs unless later tooling proves JSON is better.
Keep JSONL for runtime traces and CI/agent output.
Do not make specs a parallel authority; specs are generated or maintained from accepted design/planning truth.
```

Recommended future location:

```text
docs-site/src/content/docs/specs/ui-runtime-platform/PT-UI-RUNTIME-PLATFORM-003.uiplatform-spec.ron
```

A future phase-spec record should contain:

```text
phase id and title
lifecycle state
owning subsystem
accepted authority docs
allowed paths
forbidden paths
public API surface
invariants
acceptance criteria
validation commands
stop conditions
```

Professional workflow role:

```text
Investigation dossier -> architecture/design doc -> decision-register entry -> phase spec -> Codex prompt -> tests/proofs -> closeout.
```

Spec adoption should be planned as a separate workflow hardening item after PR #76 or as part of the first implementation prompt template. It should not delay the runtime platform cutover unless the first implementation agent cannot follow the Markdown contract safely.

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
SDF UI backend implementation inside the live runtime cutover
new UiPlugin runtime code publishing to known-wrong UI-specific render ownership names
phase spec becoming a second source of truth separate from accepted docs
```
