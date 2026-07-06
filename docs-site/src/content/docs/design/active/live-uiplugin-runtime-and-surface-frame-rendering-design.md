---
title: Live UiPlugin Runtime And Surface Frame Rendering
description: Design intake for the live UiPlugin runtime, ergonomic app UI authoring, typed action dispatch, and generic surface-frame render publication path.
status: active
owner: ui
layer: design
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../../architecture/ui-framework-architecture.md
  - ../../workspace/workflow-lifecycle.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ./ui-framework-app-integration-direction-review.md
---

# Live UiPlugin Runtime And Surface Frame Rendering

## Status

Lifecycle state: `active-planning` intake review.

Implementation authorization: **not authorized**.

Complete investigation gate: required and not complete for runtime
implementation.

Complete design gate: required and not complete for runtime implementation.

Exact implementation contract: required before code.

This document is a design intake for `PT-UI-RUNTIME-PLATFORM-001`. It records
the target shape, current evidence, owner map, capability map, ergonomics
contract, and gate gaps that must be closed before runtime code is opened.

The workflow requirement is strict: complete investigation first, complete design second, complete planning contract third, implementation fourth.

## Decision summary

The long-term public app shape should be `App + RenderPlugin + UiPlugin +
AppPlugin`.

Target app setup:

```rust
App::new()
    .add_plugins((
        RenderPlugin,
        UiPlugin,
        CounterPlugin,
    ))
    .run();
```

Target app-authoring shape:

```rust
impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Counter>();
        app.mount_ui(CounterScreen);
    }
}
```

A normal app plugin should be able to register app state and mount a screen with
a direct API such as `app.mount_ui(CounterScreen)`. Normal app authors should
not need to handle factories, route maps, event packets, host adapters, render
registries, or frame-preparation resources.

The target authoring model is typed and resource-local by default:

- app state is a normal resource;
- actions are typed app enums;
- resources can implement a typed UI action handler;
- screens implement a typed `UiScreen` contract;
- screen views lower through `ui_definition`, interaction formation, `UiProgram`, runtime/evaluator output, host-owned mutation, and derived render output.

Target normal-user model:

```text
UiScreen
IntoUi
UiActionHandler
TryUiActionHandler
app.mount_ui(Screen)
app.ui().mount(Screen) as advanced path
```

Forbidden normal-user model:

```text
UiSurfaceFactory
manual EngineUiHostAdapter
manual HostRouteMap
manual UiEventPacket
manual render submission registry
manual PreparedUiFrameResource
```

## Problem

Runenwerk has the source-backed UI substrate and a proof-local ECS-backed counter bridge, but it does not yet have the clean live runtime platform that app authors should use.

Current gaps:

- no public `UiPlugin` install surface;
- no `app.mount_ui(Screen)` ergonomic API;
- no typed `UiScreen` / typed UI action-handler path for normal app authors;
- no live app runtime session path that connects mounted UI to app state, events, mutation, frame output, and render preparation;
- render currently has UI-specific naming and producer collection that should become generic surface-frame consumption;
- `ui_app_integration` is proof-local and ECS-host-specific, not the final host-neutral framework.

## Current Investigation Evidence

Already inspected for this intake hardening:

```text
AGENTS.md
workspace workflow/gate/authority/planning docs
ui-framework architecture spine
ui-framework app-integration direction review
PT-UI-FRAMEWORK-APP-INTEGRATION-002 closeout
active-work, roadmap, production-tracks, completed-work, decision-register
engine App / Plugin / IntoPlugins APIs
render UI submission, prepared-frame, contribution, and runtime collection path
current hardcoded scene overlay and debug metrics UI producer path
ui_surface definition, mount, and session contracts
ui_hosts route/output contracts
ui_evaluator output contract
ui_runtime_view read-model path
ui_render_data UiFrame payload
ui_app_integration proof boundary
```

Current code reality:

```text
engine App/Plugin composition exists through App::add_plugin, App::add_plugins,
Plugin::build, IntoPlugins tuple wiring, resources, systems, schedules, and
world access.

No engine UiPlugin, app.mount_ui, UiScreen, IntoUi, UiActionHandler, or public
AppUiExt path exists in current code.

RenderPlugin currently initializes UiFrameSubmissionRegistryResource,
PreparedUiFrameResource, and UI render-feature preparation. The current render
runtime collector publishes scene overlay and debug metrics frames into the UI
submission registry.

ui_app_integration proves source -> UiProgram -> UiEventPacket -> bridge
resolution -> ECS host mutation -> next output for the Counter proof, but its
crate docs explicitly say it must not become a generic app framework.
```

Remaining complete investigation items before implementation:

```text
- engine App/Plugin public API and schedule/resource model
- render UI submission/prepared-frame path
- current hardcoded scene/debug UI producer path
- ui_surface definition/mount/session contracts
- ui_hosts route/output contracts
- ui_evaluator and ui_runtime_view output/read-model path
- ui_render_data UiFrame/evidence model
- ui_app_integration proof boundary and non-goals
- dependency direction between engine and domain/ui crates
- existing tests/examples relevant to app plugin + render feature integration
```

The remaining list is intentionally repeated even where this hardening pass
performed source inspection, because implementation authorization requires a
complete investigation dossier with matrices, confidence, alternatives, and
evidence classification rather than this intake summary alone.

## Goals

- Provide a live `UiPlugin` runtime integration layer under the proposed future
  `engine::plugins::ui` module.
- Make normal user setup `RenderPlugin + UiPlugin + AppPlugin`.
- Provide `app.mount_ui(Screen)` as the primary ergonomic API.
- Let app authors define typed screens and typed resource-local actions.
- Lower ergonomic Rust UI authoring into `ui_definition`, then `UiProgram`, then runtime/evaluator outputs.
- Keep app/editor/game/domain mutation owned by hosts/app plugins.
- Make render consume generic surface-frame submissions rather than UI semantics.
- Reuse existing `domain/ui` crates instead of adding a broad new `ui_runtime_platform` crate.
- Produce deterministic runtime reports that prove source, program, event, mutation, frame, and render-preparation facts.

## Non-owned responsibilities

This contract does not own SDF projection, SpatialCanvas, world-space UI, visual designer, RON/TOML/Svelte frontends, immediate/reactive execution, generic cross-domain plugin framework extraction, editor command policy, game mutation policy, renderer-owned UI semantics, or full GPU pixel validation.

## Owner map

| Responsibility | Owner | Notes |
|---|---|---|
| Live App/Plugin integration, `UiPlugin`, `AppUiExt`, schedules/resources, runtime systems, engine-side host adapter, render publication | Proposed future `engine::plugins::ui` module | New owner candidate for the live runtime layer; not implemented yet. The concrete source path must be named by the accepted implementation contract because the module does not exist in current repo truth. |
| Surface definition, mount, and session semantics | `domain/ui/ui_surface` | Existing owner; do not duplicate in engine. |
| Host route and output contracts | `domain/ui/ui_hosts` | Existing owner; engine adapter may consume these contracts. |
| Source/program/evaluator/read-model contracts | `domain/ui/ui_definition`, `ui_program`, `ui_program_lowering`, `ui_evaluator`, `ui_runtime_view` | Engine facade must lower into and consume these contracts instead of bypassing them. |
| Frame/layer/primitive payloads and evidence | `domain/ui/ui_render_data` | Owns renderer-neutral UI payload shape and evidence. |
| Render preparation/submission | `engine/src/plugins/render` | Consumes surface-frame submissions; does not own UI semantics. |
| App resource mutation | app/plugin owner through engine `World` | Generic UI must not mutate directly except through typed host/action boundary. |
| Story/proof envelope | `domain/ui/ui_story`, `ui_testing` | Required for source-to-mount proof path. |
| Proof-local Counter bridge | `domain/ui/ui_app_integration` | Evidence only; must not become final framework. |

Forbidden owners and extractions:

```text
domain/ui/ui_runtime_platform as a broad god crate
foundation/meta
domain/app_program
generic plugin framework
renderer-owned UI semantics
ECS-owned durable UI semantics
```

## Render Boundary

RenderPlugin should consume generic surface-frame submissions. Producer plugins
should publish them.

`UiPlugin` is one producer. Debug/scene overlay compatibility producers are
separate producers. RenderPlugin must not own routes, actions, `UiProgram`, app
mutation, or screen lifecycle.

Current implementation names are still UI-specific:

```text
UiFrame
UiFrameSubmission
UiFrameSubmissionRegistryResource
PreparedUiFrameResource
PreparedUiFrameContribution
UI_RENDER_FEATURE_ID
```

The design target is staged genericization toward:

```text
SurfaceFrame
SurfaceFrameSubmission
SurfaceFrameSubmissionRegistryResource
PreparedSurfaceFrameResource
PreparedSurfaceFrameContribution
SURFACE_FRAME_RENDER_FEATURE_ID
```

Do not require immediate rename unless the complete design gate proves it is
safe. The first accepted implementation contract may use compatibility names
while preserving the producer-agnostic boundary.

## Vocabulary and naming

User-facing names: `UiPlugin`, `UiScreen`, `IntoUi`, `UiActionHandler`, `TryUiActionHandler`, `app.mount_ui(...)`, and `app.ui().mount(...)`.

Render-facing target names should move toward `SurfaceFrame`, `SurfaceFrameSubmission`, `SurfaceFrameSubmissionRegistryResource`, `PreparedSurfaceFrameResource`, `PreparedSurfaceFrameContribution`, and `SURFACE_FRAME_RENDER_FEATURE_ID`.

Compatibility/current names include `UiFrame`, `UiFrameSubmission`, `UiFrameSubmissionRegistryResource`, `PreparedUiFrameResource`, `PreparedUiFrameContribution`, and `UI_RENDER_FEATURE_ID`.

Names forbidden from the normal public app-author API: `UiSurfaceFactory`, `EngineUiHostAdapter`, `UiAppRouteBridge`, `UiEventPacket`, `HostRouteMap`, `PreparedUiFrameResource`, and render submission registries.

## Capability inventory

| Capability | Exists now | Missing contract | Required owner |
|---|---:|---|---|
| Engine App plugin composition | yes | none | `engine/src/app`, `engine/src/plugin.rs` |
| Public `UiPlugin` | no | install surface, resources, schedules | proposed future `engine::plugins::ui` module |
| `app.mount_ui(Screen)` | no | ergonomic API | proposed future engine app extension module |
| Source-backed UI IR | yes | typed builder lowering | `ui_definition` + engine UI facade |
| Typed screen authoring | conceptual | `UiScreen`, `IntoUi` | proposed future `engine::plugins::ui` module |
| Typed action handling | no public path | `UiActionHandler` path | proposed future engine UI action module |
| Host route vocabulary | yes | engine adapter to `World` mutation | `ui_hosts` + proposed future engine UI host adapter module |
| Renderer-facing frame payload | yes | generic render vocabulary / alias strategy | `ui_render_data` + render plugin |
| Render submission/preparation | yes/current UI-specific | producer-agnostic surface-frame contract | `engine/src/plugins/render` |
| Full live report chain | no | runtime report | proposed future engine UI report module |

## Complete Design Gate Gaps

Complete design gate status: not satisfied.

Before implementation, the accepted design/planning record must include:

```text
module decomposition map
allowed files/crates
forbidden files/crates
validation envelope
evidence expectation
principle compliance matrix
feature support matrix
future-use-case pressure matrix
hierarchy/composition matrix
ergonomics/usability matrix
stop conditions
owner/dependency map
```

Implementation must remain blocked if any required matrix is missing or if any
unknown is converted into an assumption.

## Alternatives considered

| Option | Long-term fit | Recommendation |
|---|---|---|
| Grow `ui_app_integration` into final framework | poor | reject |
| Add broad `domain/ui/ui_runtime_platform` crate | poor | reject |
| Add proposed future `engine::plugins::ui` module and reuse domain crates | strong | accept |
| Make RenderPlugin own UI runtime | poor | reject |
| Genericize render around surface-frame submissions | strong | accept, staged |
| Force users to write host adapters | poor | reject for normal path |
| Typed `UiScreen` + `UiActionHandler` | strong | accept |

## Implementation gate

Implementation may not begin until:

- PR #75 closeout truth remains preserved: PR #72 completed
  `PT-UI-FRAMEWORK-APP-INTEGRATION-002`, and that closeout is evidence, not
  runtime authorization;
- complete investigation dossier is recorded;
- complete design gate checklist is satisfied;
- planning record promotes this beyond intake only with an exact implementation
  contract;
- allowed and forbidden files/crates are named;
- validation envelope is accepted;
- module decomposition map is accepted;
- feature support, future-use-case, hierarchy/composition, ergonomics, and
  principle compliance matrices are present;
- decision-register records the lifecycle transition.

## Candidate Future Sequence

1. `UiPlugin` skeleton and resources.
2. `app.mount_ui(Screen)` / `app.ui().mount(Screen)`.
3. `UiScreen`, `IntoUi`, and typed `UiActionHandler`.
4. Mounted surface/session runtime using `ui_surface`.
5. Typed event/action dispatch using `ui_hosts` contracts.
6. Runtime/evaluator output to frame.
7. UiPlugin render publication.
8. Render genericization from UI-specific frame names toward surface-frame names.
9. Counter live app proof.
10. Closeout report and planning truth.

This sequence is not implementation authorization.

## Stop conditions

Stop and redesign if a follow-up tries to make renderer or SDF targets own UI
semantics, make ECS entities the durable UI semantic model, make
`ui_app_integration` the final host-neutral framework, require normal app
authors to write manual route maps/host adapters, introduce `Factory`
vocabulary as primary public API, duplicate `ui_surface` or `ui_hosts`, create
a broad `domain/ui/ui_runtime_platform` crate, bypass `ui_definition`,
`FormedInteractionModel`, `UiProgram`, or `UiStory`, merge scene/debug overlay
producer logic into `RenderPlugin` as durable API, implement
SDF/SpatialCanvas/world-space/visual designer/alternate execution strategy
here, add `foundation/meta`, resurrect `domain/app_program`, add a generic
cross-domain plugin framework, or change PR #72/PR #75 closeout truth.

## Relationship to current work

`PT-UI-FRAMEWORK-APP-INTEGRATION-002` is completed through PR #72 and PR #75
closeout truth. The current focus is PR #74 /
`PT-UI-RUNTIME-PLATFORM-001` intake review and hardening.

This intake may be reviewed and hardened as a design/planning artifact. It must
not move to the implementation lifecycle state until complete investigation evidence,
complete design gate evidence, exact implementation contract, validation
envelope, evidence expectation, principle compliance, module decomposition, and
stop conditions are accepted.
