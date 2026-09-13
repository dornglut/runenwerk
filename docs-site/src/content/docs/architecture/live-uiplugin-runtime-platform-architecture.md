---
title: Live UiPlugin Runtime Platform Architecture
description: Architecture and implementation-handoff model for the engine-owned Live UiPlugin runtime, render publication, agent-controllable Counter product, trace history, source reload, and state persistence boundaries.
status: active
owner: ui
layer: architecture
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ./ui-framework-architecture.md
  - ./diagrams/live-uiplugin-runtime-platform.puml
  - ./diagrams/live-uiplugin-runtime-sequence.puml
  - ../design/deferred/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../design/archived/live-uiplugin-runtime-full-cutover-plan.md
  - ../domain/ui/roadmap.md
  - ../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../reports/closeouts/pt-ui-runtime-platform-011-closeout.md
---

# Live UiPlugin Runtime Platform Architecture

ID: `PT-UI-RUNTIME-PLATFORM-002` architecture handoff support.

This document preserves the app/engine/render architecture and the landed Runenwerk-local runtime boundary that should not live only in historical planning records.

## Authority boundary

Current source/tests and the sections explicitly describing current code facts are Runenwerk-local implementation authority. The accepted tree already contains the engine-owned `UiPlugin`, typed mounting/action contracts, runtime evaluation, producer-generic surface-frame publication, and the scene/debug producer migration described below.

Proposal-era language about the old "cutover", Counter product, source reload, persistence, phase numbers, future SDF work, or phase-spec sequencing is retained only as historical/deferred design context. It does **not** authorize continuation of the archived phase program or new Runenwerk-local reusable-framework expansion.

Any future Runenwerk consumer integration with standalone RunenUI must be re-derived from an exact accepted standalone RunenUI revision and activated by a new owning issue. The deferred consumer-integration record owns that reactivation boundary.

## Current code facts inspected for this architecture

| Area | Current fact | Source path |
|---|---|---|
| App composition | `App` owns a `World`, scheduler, runner, mode, title, and control flow; `add_plugin`, `add_plugins`, resource insertion, render-flow registration, and `world()/world_mut()` are already public. | `engine/src/app/domain/app.rs` |
| Running apps | `App::run()` dispatches to windowed or headless mode; `run_for_frames` and `run_for_ticks` are headless helpers. | `engine/src/app/runtime/lifecycle.rs` |
| Windowed runtime | Windowed mode uses `winit_runner::run(self.into_windowed_state())`. | `engine/src/app/platform/windowed.rs` |
| Input/redraw loop | Winit keyboard, mouse, cursor, wheel, and touch events become platform/input events and request redraw on success. `RedrawRequested` runs the engine frame. | `engine/src/runtime/winit_runner.rs` |
| Frame pacing | Default policy is `ContinuousCapped { target_fps: 60 }`; `OnDemand` exists and has no continuous deadline. | `engine/src/runtime/frame_pacing.rs` |
| Render plugin ownership today | `RenderPlugin` initializes generic surface-frame submission resources and runs `prepare_ui_feature_resource_system`, frame prepare, and frame submit. It no longer imports, exports, or schedules a scene/debug UI semantic collector after Phase 011. | `engine/src/plugins/render/plugin.rs` |
| Scene/debug producer path today | Scene and debug owners publish their overlay UI frames through `SurfaceFrameSubmissionRegistryResource`; the prior render-owned `ui_submission.rs` collector is deleted and guarded by tests. | `engine/src/plugins/scene/lifecycle/overlay_update.rs`, `engine/src/plugins/debug_metrics/mod.rs`, `engine/tests/runtime_surface_guard.rs` |
| UiPlugin publication today | `UiPlugin` publishes evaluated runtime frames through `SurfaceFrameSubmissionRegistryResource` with `RenderFrameProducerId` and `RenderSurfaceId`; `RenderPlugin` consumes the prepared packet without querying screens, sources, actions, host mutation, or route policy. | `engine/src/plugins/ui/render_publish.rs` |
| Frame publication today | `SurfaceFrameSubmissionRegistryResource` stores whole `SurfaceFrameSubmission` values keyed by producer/surface; replacement is per producer/surface, not per element. | `engine/src/plugins/render/features/ui/submission.rs` |
| Frame preparation today | Frame prepare builds `PreparedRenderFrame` packets per render surface and applies UI contribution per surface. | `engine/src/plugins/render/runtime/frame_prepare.rs` |
| Frame submit today | Frame submit pulls the prepared frame, selects UI rect shader/font atlas inputs, and calls `gfx.render(...)`. | `engine/src/plugins/render/runtime/frame_submit.rs` |
| UI render payload today | `UiFrame` contains surfaces, surfaces contain layers, layers contain primitives. | `domain/ui/ui_render_data` |
| UI primitive model today | UI primitives are rect, border, glyph run, image, stroke, viewport-surface embed, product surface, and clip. | `domain/ui/ui_render_data/src/primitives/ui_primitive.rs` |

## Rendering answer: raster, SDF, and frame cadence

Current UI rendering is renderer-facing `UiFrame` primitive rendering, not SDF-owned UI semantics.

The inspected UI primitive model contains raster-style draw primitives: `Rect`, `Border`, `GlyphRun`, `Image`, `Stroke`, viewport-surface embeds, product-surface primitives, and clipping. The render submit path passes a prepared frame, an optional UI rect shader, and `UiFontAtlasResource` into `gfx.render(...)`.

SDF exists elsewhere in the render/world stack as render capability and future projection target. It does not currently own UI source, UI routes, UI actions, UI state, or UI primitive generation. Any future SDF/world-space UI remains a separate target/projection concern unless a current target contract promotes it.

Current frame cadence is redraw-driven through winit with default continuous capped pacing. Because the default `FramePacingPolicyResource` is 60 FPS continuous, normal windowed runtime can redraw continuously. `OnDemand` mode exists and avoids continuous deadlines, and input events request redraw. The current inspected surface-frame submission seam replaces a whole producer/surface `SurfaceFrameSubmission`; it does not prove element-level incremental rendering.

The old cutover plan recorded this target frame policy for possible later work:

```text
1. Preserve the existing continuous capped mode for animated/runtime-heavy scenes.
2. Make non-animated UI capable of on-demand redraw.
3. Add generic dirty/invalidation records at screen, source, binding, layout, primitive, surface, and render-publication levels.
4. Do not claim element-level incremental render until a phase proves stable dirty scopes and backend support.
5. Do not rebuild/republish UI frames when no source, host data, input, layout, theme, text, or surface dependency changed, unless continuous animation policy requests it.
```

This target policy is historical/deferred context, not current implementation sequencing.

## Render-boundary generalization decision

The producer-generic render submission boundary described by the former cutover plan is now implemented in the accepted tree.

The decision that drove the landed boundary was:

```text
Move producer-generic surface-frame semantics before UiPlugin render publication.
Do not let UiPlugin runtime code stabilize on UI-specific render ownership names.
RenderPlugin consumes producer/surface/frame packets; UiPlugin is one producer, not the render-frame owner.
```

Current boundary shape:

| Concept | Current owner | Rule |
|---|---|---|
| Source/program/action/session semantics | `domain/ui` plus `engine::plugins::ui` integration | Must not move into render. |
| Producer identity | Engine/runtime producer contract | UI, debug overlays, scene overlays, product surfaces, and future producers publish as producers. |
| Surface/frame packet | Render-facing producer-generic contract | Must not encode `UiPlugin` as the owner of the generic frame model. |
| Render preparation/submission | `RenderPlugin` | Consumes packets; does not query screens, source, route, host state, or actions. |

The historical implementation order created the generic producer/surface-frame seam before durable UiPlugin render publication and migrated scene/debug overlays to that producer path. Current code facts above are the authority for the landed result.

This boundary does not authorize a broader render rewrite and does not transfer source/program/action semantics into render.

## Historical and deferred expansion context

The remaining sections preserve design rationale from the former runtime-platform program. They are useful when evaluating current code or a future, newly authorized consumer-integration proposal, but they are not live phase sequencing. Phase labels, required product shapes, and future implementation instructions below must not be used to start work without a new owning issue and current authority resolution.

## SDF UI future-backend position

SDF UI is possible, but it was not part of the Live UiPlugin runtime delivery. It belongs to a separate render-backend/projection design if future authority promotes it.

Historical recommended future track:

```text
PT-UI-RENDER-BACKEND-SDF-001 — Analytical SDF UI Primitive Backend
```

Historical recommended first SDF scope:

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
The runtime product path should prove source/program/action/host/render-publication ownership before adding another render backend.
Text fidelity, shaping, accessibility, and source maps are already difficult enough without making the first product proof an SDF backend proof.
```

Later SDF work may explore MSDF text/icons, world-space panels, holographic UI, glow/soft-edge effects, and animated shader parameters. Those must consume `UiFrame`/SurfaceFrame-style output and must not own UI source, routes, actions, host mutation, or runtime session truth.

## Authoring, live changes, and hot reload

The canonical source truth remains `ui_definition` / `UiProgram`, not renderer primitives or app state.

Historical authoring directions:

| Authoring form | Runtime-platform decision |
|---|---|
| Rust typed screen/builder | Primary app-author path for `app.mount_ui(CounterScreen)`. |
| RON/authored templates | Supported source format through `ui_definition`; intended for checked-in fixtures and data-driven screens. |
| Visual designer output | Future product authoring surface; must save/export source IR, not renderer primitives or direct mutation logic. |
| Compiler DSL / reactive / immediate adapters | Future frontends only if they capture source records, route/action contracts, source maps, and proof facts. |

Historical hot-reload decision:

```text
Rust code is not treated as hot-reloadable UI source.
Live UI changes are supported through reloadable data-backed source revisions: RON/templates, designer output, or future source IR files.
A reload must revalidate, re-lower, recompile/evaluate, preserve session state by stable source/runtime IDs where valid, and report any migration loss.
```

This reload direction is not implementation authorization; current code/tests decide what is actually supported.

## State persistence decision

The former program split persistence ownership as follows:

| State kind | Owner | Persistence rule |
|---|---|---|
| App/domain state, such as `Counter` | App/host owner | Persist through explicit host-owned snapshot/load hooks. Generic UI must not own it. |
| UI session state, such as focus, hover, pressed, selected surface, scroll, input capture | UiPlugin session resources using domain UI contracts | May be snapshotted/replayed for tests and restored only by stable source/runtime IDs. |
| Source state, such as templates or designer output | UI source owner / product authoring owner | Persist as source IR, RON/template, or designer project output. |
| Render state | Render backend owner | Cache/pipeline state is not UI state and must not become user-state persistence. |

The historical Counter product proposal used this optional state-file shape only after a persistence phase:

```text
cargo run -p ui_counter_runtime -- --state-file target/ui_counter_runtime/counter.state.ron
```

This remains historical design context unless current code independently proves it.

## Agent-controllable runtime product

The archived program proposed a runnable Counter app serving both humans and agents.

Historical modes:

| Mode | Command shape | Purpose |
|---|---|---|
| Human window | `cargo run -p ui_counter_runtime` | Opens a native window and supports pointer interaction. |
| Agent/headless script | `cargo run -p ui_counter_runtime -- --headless --agent-script assets/ui_counter_runtime/scripts/increment_reset.ron --trace-jsonl target/ui_counter_runtime/trace.jsonl --exit-after-script` | Lets a simple agent drive actions deterministically and inspect machine-readable output. |
| Deterministic test | `cargo test -p ui_counter_runtime` plus focused engine tests | Proves source/program/action/mutation/render facts without manual UI. |

The historical rule was that agent scripts name semantic actions and optional pointer gestures, resolve through the same route/capability/payload validation path as human interaction, and never call `Counter` mutation directly.

Agent script format decision:

```text
RON is acceptable for the first repo-native fixture format because Runenwerk already uses RON-style fixtures.
JSONL remains the required trace output format for agents and CI.
Support for JSONL agent input may be added later if cross-tool agent integration needs it, but the first Counter product must not block on dual input formats.
```

## Counter product screen contract

The archived phase program proposed this product identity:

```text
binary: ui_counter_runtime
window title: Runenwerk UI Counter Runtime
mounted screen type: CounterScreen
host plugin type: CounterPlugin
host resource: Counter { value: i64 }
initial value: 0 unless loaded from explicit host-owned state file after the persistence phase exists
```

Historical visible structure:

```text
root surface: CounterScreen
root layout: vertical stack or equivalent semantic container
header label: Runenwerk UI Counter Runtime
count label: Count: {value}
action row: Increment, Decrement, Reset
trace console: last N runtime trace entries, newest last or clearly ordered
status line: last action result or diagnostic summary
```

Historical semantic actions and routes:

| UI control | Route id | Required capability | Payload | Host mutation |
|---|---|---|---|---|
| Increment button | `counter.increment` | `counter.write` | none or unit payload | `Counter.value += 1` |
| Decrement button | `counter.decrement` | `counter.write` | none or unit payload | `Counter.value -= 1` |
| Reset button | `counter.reset` | `counter.write` | none or unit payload | `Counter.value = 0` |

Historical read-capability rule:

```text
counter.read permits rendering the current count.
counter.write permits mutating count through the three actions.
missing or rejected capability must not mutate Counter.
```

Historical agent-script semantics:

```text
semantic action names resolve to the same route ids as visible controls
optional scripted pointer activation must hit-test to the same route ids
agent scripts cannot mutate Counter directly
JSONL trace is the machine-readable source of action/mutation/evaluation/frame evidence
```

Historical visible behavior:

```text
Increment updates visible count by +1.
Decrement updates visible count by -1.
Reset updates visible count to 0.
Every accepted action adds a trace console row.
Every rejected action adds a diagnostic trace row and leaves count unchanged.
```

## Runtime trace, history, and console visibility

The former program required a UI-runtime trace/history model rather than Counter-specific logging or a premature engine-wide tracing framework.

Historical ownership decision:

```text
Trace starts in engine::plugins::ui because the first required events are UI-semantic events.
Do not extract a cross-engine tracing framework during this cutover.
Counter consumes and displays trace; it does not define the trace model.
```

Historical Phase 007 minimum event set:

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

Historical later-phase event additions:

| Phase | Added event families |
|---|---|
| Phase 008 | `UiRuntimeEvaluated`, state/invalidation trace facts, optional `UiSessionStateRestored` when session replay is implemented. |
| Phase 010 | `UiFramePublished`, `UiFramePresented`. |
| Phase 013 | `UiSourceRevisionLoaded`, `UiSourceLowered`, `UiProgramFormed`, `UiStateSnapshotWritten`. |

Historical trace requirements:

```text
Every event has monotonic sequence, frame index if available, screen id, source id where applicable, route id where applicable, capability verdict where applicable, host id where applicable, result, and diagnostic code where applicable.
Trace is available as an in-memory bounded ring buffer resource.
Trace can be exported as JSONL for agents and CI.
Trace has a human-readable console/debug surface in the Counter product.
Trace must be source-map aware where source identity exists.
Trace must not expose renderer internals as UI source truth.
```

## Phase implementation spec decision

The former program proposed lightweight phase implementation specs, without making them a blocking requirement.

Historical decision:

```text
Use human-readable Markdown for design rationale and architecture.
Use a compact machine-checkable phase spec later as the compiled implementation handoff contract.
Prefer RON for repo-native phase specs unless later tooling proves JSON is better.
Keep JSONL for runtime traces and CI/agent output.
Do not make specs a parallel authority; specs are generated or maintained from accepted design/planning truth.
```

Historical recommended location:

```text
docs-site/src/content/docs/specs/ui-runtime-platform/PT-UI-RUNTIME-PLATFORM-003.uiplatform-spec.ron
```

A historical phase-spec record was expected to contain:

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

Historical workflow role:

```text
Investigation dossier -> architecture/design doc -> decision-register entry -> phase spec -> Codex prompt -> tests/proofs -> closeout.
```

This phase-spec sequencing is historical evidence and does not override current repository workflow authority.

## Architecture diagrams

PlantUML source files:

```text
docs-site/src/content/docs/architecture/diagrams/live-uiplugin-runtime-platform.puml
docs-site/src/content/docs/architecture/diagrams/live-uiplugin-runtime-sequence.puml
```

These diagrams are architecture artifacts. Historical planning documents may link to them; they do not grant implementation authority.

## Preserved boundary stop conditions

The following rules remain useful architecture guardrails where current authority still preserves them:

```text
renderer-owned UI source/action/host semantics
SDF-owned UI source/action/host semantics
agent scripts mutating host state directly
Counter-specific tracing instead of generic runtime trace events
state persistence hidden inside generic UI controls
Rust-code hot reload claims
per-element incremental rendering claims without dirty-scope proof
SDF UI backend implementation without separate authority
new UiPlugin runtime code publishing to UI-specific render ownership names
phase spec becoming a second source of truth separate from accepted docs
```

They do not reactivate the archived phase program; any new implementation still requires current owner/code inspection and a new owning issue.
