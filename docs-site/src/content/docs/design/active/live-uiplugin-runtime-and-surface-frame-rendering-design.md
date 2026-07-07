---
title: Live UiPlugin Runtime And Surface Frame Rendering
description: Design-gate record for the live engine UiPlugin runtime, app-facing typed screen/action ergonomics, and staged generic surface-frame render publication.
status: active
owner: ui
layer: design
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../../architecture/ui-framework-architecture.md
  - ./ui-framework-app-integration-direction-review.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/complete-merge-readiness-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
---

# Live UiPlugin Runtime And Surface Frame Rendering

ID: `PT-UI-RUNTIME-PLATFORM-001`

Lifecycle state: `active-planning` design-gate complete / implementation-planning required.

Implementation status: not started and not authorized by this document.

## Design-gate result

This design accepts the investigation result from:

```text
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
```

Complete investigation gate status: complete for current-state, authority, owner, dependency, vocabulary, capability, alternatives, confidence, and blocker evidence.

Complete design gate status: complete for opening a separate implementation-planning PR. Runtime implementation remains blocked until that later PR records exact owner files, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, stop conditions, and acceptance criteria.

Required final position:

```text
PT-UI-RUNTIME-PLATFORM-001 investigation/design gate is complete.
Runtime implementation is still not started; open a separate implementation-planning PR with exact contract.
```

## Target direction

The accepted long-term direction is an engine-owned live UI runtime plugin layer that reuses existing domain UI contracts.

Target app shape:

```rust
App::new()
    .add_plugins((RenderPlugin, UiPlugin, CounterPlugin))
    .run();

struct CounterPlugin;

impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Counter>();
        app.mount_ui(CounterScreen);
    }
}
```

Normal app authors should write typed screens and typed action handlers. They should not manually author host route maps, event packets, render submission registries, prepared render resources, or proof-local bridge code.

Target chain:

```text
RenderPlugin + UiPlugin + AppPlugin
  -> app.mount_ui(CounterScreen)
  -> typed UiScreen
  -> typed IntoUi
  -> typed UiActionHandler / TryUiActionHandler
  -> host-owned mutation
  -> source/program/evaluator-backed output
  -> UiPlugin-published frame submission
  -> RenderPlugin-consumed surface-frame contribution
```

## Non-goals for PR #74

This document and PR remain docs/planning/design only. They do not implement:

```text
engine UiPlugin
AppUiExt
app.mount_ui
UiScreen
IntoUi
UiActionHandler
TryUiActionHandler
render adapter code
SurfaceFrame migration code
SDF / world-space / SpatialCanvas
foundation/meta
domain/app_program
generic plugin framework
```

## Owner and dependency map

| Responsibility | Accepted owner | Notes |
|---|---|---|
| Public app plugin integration surface | future `engine::plugins::ui` | Engine owns `App`/`Plugin` composition. |
| Common mounting API | future `engine::plugins::ui::app_ext` and `mount` | Must expose `app.mount_ui(Screen)` and advanced `app.ui().mount(Screen)` without manual route maps. |
| Typed screen abstraction | future `engine::plugins::ui::screen` | Facade over `ui_definition` source records, not a new semantic model. |
| Typed source lowering | future `engine::plugins::ui::source` | Lowers `IntoUi` to `UiNodeDefinition` / `UiProgram` path. |
| Typed action handling | future `engine::plugins::ui::action` | Lowers to route/event/action contracts and host-owned mutation. |
| Runtime mounted surface/session registry | future `engine::plugins::ui::resources` using `ui_surface` | Reuse `SurfaceDefinition`, `MountedSurfaceInstance`, and session contracts. |
| Host route/output policy | `domain/ui/ui_hosts` contracts plus engine-side adapter | Domain owns route/output vocabulary; engine/app owns mutation. |
| Evaluator/runtime read model | `ui_evaluator`, `ui_runtime_view`, existing UI runtime crates | Engine runtime consumes; domain does not depend on engine. |
| Renderer-facing frame data | `ui_render_data` now, staged generic naming later | `UiFrame` remains current payload for first slice. |
| Render preparation/submission | `engine::plugins::render` | Render consumes frame submissions; render must not own UI semantics. |
| Proof-local Counter evidence | `domain/ui/ui_app_integration` | Historical/proof evidence only; not final framework owner. |
| Scene/debug overlay compatibility producers | future compatibility producer modules | Long-term outside core RenderPlugin UI producer collection. |

Dependency rule:

```text
engine/src/plugins/ui may depend on domain/ui contracts.
domain/ui crates must not depend on engine.
render may consume frame/submission data but must not own UI semantics.
ui_app_integration remains proof-local.
```

## Proposed future module decomposition

This path is proposed future implementation ownership. It does not exist today and is not created by this PR.

```text
engine/src/plugins/ui/
  mod.rs
  plugin.rs
  app_ext.rs
  schedule.rs
  resources.rs
  screen.rs
  mount.rs
  action.rs
  host.rs
  source.rs
  events.rs
  render_publish.rs
  report.rs
  diagnostics.rs
  compat_scene_overlay.rs
  compat_debug_overlay.rs
```

| Module | Responsibility | Must not own |
|---|---|---|
| `mod.rs` | Public module exports and owner boundary. | Domain UI semantics. |
| `plugin.rs` | `UiPlugin` install/resources/systems. | App-specific mutation. |
| `app_ext.rs` | `app.mount_ui` and `app.ui()` extension surface. | Manual route-map APIs as common path. |
| `schedule.rs` | UiPlugin runtime set labels and ordering. | Render backend scheduling internals. |
| `resources.rs` | Engine ECS resources for mounts, sessions, queues, reports. | Domain source/program definitions. |
| `screen.rs` | Typed `UiScreen` contract. | Renderer primitives or ECS entity semantic truth. |
| `mount.rs` | Mounted screen/surface lifecycle. | Persistence policy beyond declared session contract. |
| `action.rs` | Typed `UiActionHandler` / `TryUiActionHandler`. | Direct generic-control mutation. |
| `host.rs` | Host adapter around `ui_hosts` and app mutation boundary. | Product/editor/game domain semantics. |
| `source.rs` | `IntoUi` lowering facade. | Alternate semantic source truth. |
| `events.rs` | Event packet/action queue and fail-closed dispatch. | Immediate callback bypass. |
| `render_publish.rs` | Runtime/evaluator output to frame submission publication. | Render preparation/backend execution. |
| `report.rs` | Runtime/proof envelope. | Closeout/planning truth. |
| `diagnostics.rs` | User-facing runtime diagnostics. | Silent failure policy. |
| `compat_scene_overlay.rs` | Compatibility producer for existing scene overlay frames. | Core RenderPlugin producer ownership. |
| `compat_debug_overlay.rs` | Compatibility producer for existing debug overlay frames. | Core RenderPlugin producer ownership. |

## Allowed future implementation files/crates

The later implementation-planning PR may authorize a subset of these only after exact scope is accepted:

```text
engine/src/plugins/ui/**
engine/src/plugins/mod.rs
engine/src/prelude.rs
engine/Cargo.toml
engine tests/examples needed for live Counter proof
docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Potential additional read-only or dependency-only crates, if justified by the implementation contract:

```text
domain/ui/ui_definition
domain/ui/ui_program
domain/ui/ui_program_lowering
domain/ui/ui_compiler
domain/ui/ui_artifacts
domain/ui/ui_evaluator
domain/ui/ui_runtime_view
domain/ui/ui_surface
domain/ui/ui_hosts
domain/ui/ui_render_data
domain/ui/ui_app_integration tests as comparison evidence only
```

## Forbidden files/crates for the first implementation-planning contract

```text
foundation/meta
domain/app_program
new generic plugin framework crates
SDF/world-space/SpatialCanvas implementation crates
product/editor/game mutation crates unless explicitly scoped as host proofs
renderer backend rewrites unrelated to frame-submission consumption
changes that make domain/ui depend on engine
changes that make RenderPlugin own UI source/program/action semantics
changes that turn ui_app_integration into final framework
```

## Render-boundary design

RenderPlugin should move toward consuming generic surface-frame submissions. It may know producer identity, render surface identity, draw ordering, frame payloads, primitive families, font atlas/shader resources, render diagnostics, and prepared contribution status.

RenderPlugin must not know or own:

```text
UiScreen identity as app source truth
IntoUi lowering
UiActionHandler mapping
host mutation policy
route authorization semantics
UiStory mount eligibility
SDF/world-space semantic ownership
app/editor/game/product state mutation
```

Current `UiFrameSubmission` naming may remain for the first runtime slice. A direct immediate `SurfaceFrame` rename is optional and should be staged unless the implementation-planning PR proves the rename is small, isolated, and validated. The first slice should prefer a clean compatibility seam over a broad rename.

Existing scene/debug overlay collection inside render runtime is a compatibility producer path. Long-term, scene/debug overlays should publish submissions through compatibility producer modules, not direct RenderPlugin-owned UI producer collection.

## Ergonomics contract

| Path | Status | Decision |
|---|---|---|
| `app.mount_ui(CounterScreen)` | Primary API | Required normal path. |
| `app.ui().mount(CounterScreen)` | Advanced API | Required advanced path for configuration/reporting. |
| `app.ui().surface(...).source(...).host(...).mount()` | Internal/advanced builder | Allowed for platform/tests only. |
| `UiSurfaceFactory` | Forbidden common path | Too internal and surface-first. |
| Manual host adapter | Forbidden common path | Advanced/custom host only. |
| Typed `UiScreen` + `UiActionHandler` | Default model | Required action/mutation path. |

Normal app authors must not be forced to call manual APIs equivalent to:

```text
add_ui_action
add_ui_screen
add_ui_screen_router
manual route maps
manual event packets
manual host adapters
manual render submission registry writes
manual prepared frame resource writes
```

## Feature support matrix

| Feature | First implementation target | Deferred / forbidden in first slice |
|---|---|---|
| Engine `UiPlugin` skeleton | Yes, after separate implementation plan. | Broad generic plugin framework. |
| `app.mount_ui(Screen)` | Yes. | Alternate public API families before the primary path works. |
| `app.ui().mount(Screen)` | Yes as advanced path. | Factory-first public ergonomics. |
| Typed `UiScreen` | Yes. | ECS entities as durable UI semantic model. |
| Typed `IntoUi` | Yes. | Direct renderer primitive authoring as source truth. |
| Typed `UiActionHandler` / `TryUiActionHandler` | Yes. | Generic controls mutating app state directly. |
| `ui_surface` mounted sessions | Yes via engine resource wrapper. | New duplicate surface/session model. |
| `ui_hosts` action/route policy | Yes via adapter. | Manual host maps as common path. |
| Runtime/evaluator output to frame | Yes for bounded Counter live proof. | Alternate execution strategies. |
| Render publication | Yes through existing submission seam first. | Render owning UI semantics. |
| Generic `SurfaceFrame` naming | Staged. | Broad rename without scoped validation. |
| SDF/world-space/SpatialCanvas | No. | Any implementation in this track slice. |
| `foundation/meta` / generic plugin framework | No. | Any extraction/resurrection. |

## Future-use-case pressure matrix

| Future pressure | Design response now | Stop condition |
|---|---|---|
| Editor UI surfaces | Keep host mutation/editor commands outside generic UI; use host adapters. | Moving editor command semantics into domain/ui or render. |
| Game HUD | Treat as host/profile plus render target consumer. | Making game HUD source truth renderer/SDF-owned. |
| World-space UI | Keep as projection/target consuming proven UI output. | Implementing world-space/SDF/SpatialCanvas in this slice. |
| Visual designer | Output must become `ui_definition` source records. | Designer output bypasses source/program/proof. |
| External templates | Template frontend must lower through same source/program path. | Separate template runtime path. |
| Immediate/reactive authoring | Adapter may exist later if it captures source and route contracts. | Direct mutable callbacks without source maps/proof. |
| Multiple windows/surfaces | Use `ui_surface` and render-surface mappings. | New ad-hoc surface identity model. |
| Debug overlays | Move to compatibility producer. | Keeping RenderPlugin as permanent semantic owner. |
| Generic render producers | Staged surface-frame genericization. | Broad rename with unbounded fallout. |

## Hierarchy/composition matrix

| Hierarchy | Owner | Rule |
|---|---|---|
| App/plugin hierarchy | Engine `App` / `Plugin` | UiPlugin composes into existing plugin model. |
| UI source hierarchy | `ui_definition` | Typed screens lower into source records. |
| Semantic program hierarchy | `ui_program` | Route/event/program facts remain domain-owned. |
| Runtime/evaluator hierarchy | `ui_evaluator`, `ui_runtime_view`, existing runtime crates | Runtime derives output and diagnostics from program/artifacts. |
| Surface/session hierarchy | `ui_surface` plus engine resource wrapper | Engine stores live mount/session state without redefining semantics. |
| Host/action hierarchy | `ui_hosts` plus app-owned handlers | Host/app owns mutation. |
| Render hierarchy | Render prepared frame/contributions | Render consumes frame data; producers own semantic meaning. |
| Proof/report hierarchy | `ui_story`, `ui_testing`, future UiPlugin report | Visible output is not success without upstream proof facts. |

## Principle compliance matrix

| Principle | Status | Evidence / resolution |
|---|---|---|
| KISS | Pass for design gate | Primary path is direct: `app.mount_ui(Screen)` -> source/program/evaluator -> frame submission. |
| DRY | Pass | `ui_surface`, `ui_hosts`, `ui_evaluator`, `ui_runtime_view`, and `ui_render_data` are reused instead of duplicated in a broad runtime-platform crate. |
| YAGNI | Pass | No code, crate, SDF, SpatialCanvas, `foundation/meta`, or generic plugin framework is added by PR #74. |
| SOLID | Pass | Proposed module split gives screen/source/action/host/events/render/report separate reasons to change. |
| Separation of Concerns | Pass | Engine app wiring, domain UI semantics, host mutation, render consumption, and reports stay separate. |
| Avoid Premature Optimization | Pass | Generic `SurfaceFrame` rename is staged; first runtime slice can use current `UiFrame` seam. |
| Law of Demeter | Pass | Engine-side UI layer talks to direct domain contracts instead of reaching through render/scene internals. |

## Validation envelope for the future implementation PR

Docs-only PR #74 did not run local commands in this connector-only session.

The future implementation-planning PR must require, at minimum:

```text
cargo test -p engine <focused UiPlugin/runtime tests>
cargo test -p ui_app_integration
cargo test -p ui_hosts
cargo test -p ui_surface
cargo test -p ui_evaluator
cargo test -p ui_runtime_view
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Cargo scope may be refined only if the implementation contract proves a smaller or different crate set is sufficient. Do not claim optional cargo validation unless it is actually run.

## Evidence expectation for the future implementation PR

The future implementation PR must produce evidence for:

```text
UiPlugin installs its resources and systems
app.mount_ui(CounterScreen) compiles and mounts the screen
app.ui().mount(CounterScreen) compiles as advanced path
UiScreen lowers to ui_definition / UiProgram facts
IntoUi produces source-map-bearing source records
UiActionHandler / TryUiActionHandler dispatch is typed and fail-closed
unknown route/schema/capability/payload/action failures do not mutate host state
mounted surface/session registry records generation and host identity
runtime/evaluator output reaches UiFrame or the accepted frame payload
UiPlugin publishes frame submission without RenderPlugin querying UI semantics
Counter live app proof reports source, program, runtime, action, mutation, and render facts
scene/debug compatibility producer behavior is unchanged or explicitly migrated
```

## Implementation sequencing

Each slice below is future work. None is implemented by PR #74.

### 1. UiPlugin skeleton and resources

Scope: create future `engine::plugins::ui` module, `UiPlugin`, schedule labels, resources, diagnostics shell.

Allowed files/crates: `engine/src/plugins/ui/**`, `engine/src/plugins/mod.rs`, `engine/Cargo.toml` if dependencies are justified.

Forbidden files/crates: `foundation/meta`, `domain/app_program`, render backend rewrites, domain UI engine dependencies.

Tests/proofs: plugin installation initializes resources and is idempotent.

Stop conditions: stop if skeleton needs generic plugin framework extraction or domain/ui depends on engine.

### 2. `app.mount_ui(Screen)` / `app.ui().mount(Screen)`

Scope: add public/advanced engine API shape that records mount requests.

Allowed files/crates: future `app_ext.rs`, `mount.rs`, `resources.rs`, engine prelude if accepted.

Forbidden files/crates: manual route-map common APIs, proof-local public API leakage.

Tests/proofs: CounterScreen mount request is recorded with stable diagnostics.

Stop conditions: stop if common path requires manual host adapter or surface factory.

### 3. `UiScreen`, `IntoUi`, `UiActionHandler`

Scope: typed screen/source/action traits that lower to existing UI source/program/host contracts.

Allowed files/crates: future `screen.rs`, `source.rs`, `action.rs`, selected domain UI dependencies.

Forbidden files/crates: direct renderer primitive source truth, ECS-owned UI semantics.

Tests/proofs: source maps, route facts, and typed action identities are produced.

Stop conditions: stop if generic controls can mutate app state directly.

### 4. Mounted surface/session runtime using `ui_surface`

Scope: engine resource wrapper around mounted surface/session contracts.

Allowed files/crates: future `resources.rs`, `mount.rs`; `ui_surface` dependency.

Forbidden files/crates: duplicate surface/session model.

Tests/proofs: mount/unmount generation and retention-class reporting.

Stop conditions: stop if world-space/SDF/SpatialCanvas implementation becomes required.

### 5. Typed event/action dispatch using `ui_hosts`

Scope: event packet queue, action resolution, host-owned mutation envelope.

Allowed files/crates: future `events.rs`, `action.rs`, `host.rs`; `ui_hosts` dependency.

Forbidden files/crates: direct callbacks that bypass route/schema/capability validation.

Tests/proofs: fail-closed no-mutation cases equivalent to proof-local negative tests.

Stop conditions: stop if product/editor/game semantics move into `domain/ui`.

### 6. Runtime/evaluator output to frame

Scope: produce runtime/evaluator output and frame payload for mounted screens.

Allowed files/crates: future `source.rs`, `resources.rs`, `report.rs`; selected `ui_evaluator`, `ui_runtime_view`, `ui_render_data` dependencies.

Forbidden files/crates: new execution strategy without accepted design.

Tests/proofs: Counter screen output text and visual primitives are reported.

Stop conditions: stop if visible frame output bypasses source/program/evaluator reports.

### 7. UiPlugin render publication

Scope: publish runtime frame into render submission registry as a producer.

Allowed files/crates: future `render_publish.rs`, existing render submission resources if accepted.

Forbidden files/crates: RenderPlugin querying `UiScreen`, `UiActionHandler`, or host mutation semantics.

Tests/proofs: prepared render frame contains UiPlugin contribution by producer id/surface.

Stop conditions: stop if RenderPlugin becomes UI runtime owner.

### 8. Render genericization toward `SurfaceFrame` naming

Scope: staged rename/alias/adaptor from UI-specific naming to producer-generic surface-frame vocabulary.

Allowed files/crates: render frame/submission contracts and docs named by separate implementation plan.

Forbidden files/crates: broad unplanned migration.

Tests/proofs: existing UI frame submissions and compatibility producers still prepare deterministically.

Stop conditions: stop if migration cannot be scoped cleanly.

### 9. Counter live app proof

Scope: prove the full engine plugin path with Counter UI, typed action handling, host mutation, and render publication evidence.

Allowed files/crates: focused engine tests/examples and report fixtures.

Forbidden files/crates: SDF/world-space/SpatialCanvas, app_program, generic plugin framework.

Tests/proofs: positive live proof and fail-closed negative cases.

Stop conditions: stop if proof cannot show source/program/runtime/action/mutation/render facts.

### 10. Closeout and planning truth

Scope: record validation, evidence, merge readiness, planning transitions, known gaps.

Allowed files/crates: docs-only planning/closeout files.

Forbidden files/crates: next implementation start before closeout truth.

Tests/proofs: docs validation and diff hygiene after closeout edits.

Stop conditions: stop if validation or lifecycle truth is unavailable and not explicitly recorded.

## Acceptance criteria for this design gate

This design gate is accepted when:

```text
current-state investigation exists and is linked
owner/dependency direction is explicit
render boundary is explicit
common/advanced/internal ergonomics are explicit
module decomposition candidate is explicit and marked future
allowed/forbidden files and crates are explicit
validation envelope is explicit
evidence expectation is explicit
feature/future/hierarchy/principle matrices are present
implementation sequence and stop conditions are present
runtime implementation remains not authorized
```

This document satisfies those criteria for design/planning. It does not satisfy active implementation authorization.

## Stop conditions

Stop and report instead of implementing if the work requires:

```text
runtime Rust implementation in PR #74
engine UiPlugin code in PR #74
public AppUiExt code in PR #74
app.mount_ui implementation in PR #74
UiScreen / IntoUi implementation in PR #74
UiActionHandler implementation in PR #74
render adapter code in PR #74
SurfaceFrame type migration code in PR #74
SDF/world-space/SpatialCanvas implementation
foundation/meta
domain/app_program resurrection
generic plugin framework
changing PR #72/PR #75 closeout truth
making RenderPlugin own UI semantics
making ECS own durable UI semantics
making ui_app_integration the final framework
```

## Next step

Open a separate implementation-planning PR for the first runtime slice. That PR must use this design and the investigation report as authority, then record the exact implementation contract before any Rust code is written.