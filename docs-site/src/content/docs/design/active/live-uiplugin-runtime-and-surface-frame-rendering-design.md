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
  - ./ui-framework-app-integration-direction-review.md
---

# Live UiPlugin Runtime And Surface Frame Rendering

## Status

Lifecycle state: `investigating -> proposed-design`.

Implementation authorization: **not authorized**.

This document is a design intake for `PT-UI-RUNTIME-PLATFORM-001`. It records the target shape, investigation findings, owner map, capability map, ergonomics contract, and implementation gate needed before runtime code is opened.

The workflow requirement is strict: complete investigation first, complete design second, complete planning contract third, implementation fourth.

## Decision summary

The long-term public app shape should be `App + RenderPlugin + UiPlugin + AppPlugin`.

A normal app plugin should be able to register app state and mount a screen with a direct API such as `app.mount_ui(CounterScreen)`. Normal app authors should not need to handle factories, route maps, event packets, host adapters, render registries, or frame-preparation resources.

The target authoring model is typed and resource-local by default:

- app state is a normal resource;
- actions are typed app enums;
- resources can implement a typed UI action handler;
- screens implement a typed `UiScreen` contract;
- screen views lower through `ui_definition`, interaction formation, `UiProgram`, runtime/evaluator output, host-owned mutation, and derived render output.

## Problem

Runenwerk has the source-backed UI substrate and a proof-local ECS-backed counter bridge, but it does not yet have the clean live runtime platform that app authors should use.

Current gaps:

- no public `UiPlugin` install surface;
- no `app.mount_ui(Screen)` ergonomic API;
- no typed `UiScreen` / typed UI action-handler path for normal app authors;
- no live app runtime session path that connects mounted UI to app state, events, mutation, frame output, and render preparation;
- render currently has UI-specific naming and producer collection that should become generic surface-frame consumption;
- `ui_app_integration` is proof-local and ECS-host-specific, not the final host-neutral framework.

## Goals

- Provide a live `UiPlugin` runtime integration layer under `engine/src/plugins/ui`.
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
| Public `UiPlugin` install surface | `engine/src/plugins/ui` | Engine runtime owns plugin composition, schedules, resources, and app lifecycle. |
| `app.mount_ui(Screen)` / `app.ui().mount(Screen)` | `engine/src/plugins/ui/app_ext.rs` | Public ergonomic app API. |
| `UiScreen`, `IntoUi`, typed view facade | `engine/src/plugins/ui` initially; normalized semantics in `domain/ui/ui_definition` | Must lower to source IR. |
| Surface definition, mounted instance, session semantics | `domain/ui/ui_surface` | Existing owner; do not duplicate. |
| Host route and output contracts | `domain/ui/ui_hosts` | Existing owner; engine adapter may use these contracts. |
| App resource mutation | app/plugin owner through engine `World` | Generic UI must not mutate directly except through typed host/action boundary. |
| Render submission registry and preparation | `engine/src/plugins/render` | Should consume generic surface-frame submissions. |
| Renderable frame/layer/primitive data | `domain/ui/ui_render_data` currently | May stay as payload source while render-facing vocabulary genericizes. |
| Story/proof envelope | `domain/ui/ui_story`, `ui_testing` | Required for source-to-mount proof path. |
| Proof-local Counter bridge | `domain/ui/ui_app_integration` | Evidence only; must not become final framework. |

## Vocabulary and naming

User-facing names: `UiPlugin`, `UiScreen`, `IntoUi`, `UiActionHandler`, `TryUiActionHandler`, `app.mount_ui(...)`, and `app.ui().mount(...)`.

Render-facing target names should move toward `SurfaceFrame`, `SurfaceFrameSubmission`, `SurfaceFrameSubmissionRegistryResource`, `PreparedSurfaceFrameResource`, `PreparedSurfaceFrameContribution`, and `SURFACE_FRAME_RENDER_FEATURE_ID`.

Compatibility/current names include `UiFrame`, `UiFrameSubmission`, `UiFrameSubmissionRegistryResource`, `PreparedUiFrameResource`, `PreparedUiFrameContribution`, and `UI_RENDER_FEATURE_ID`.

Names forbidden from the normal public app-author API: `UiSurfaceFactory`, `EngineUiHostAdapter`, `UiAppRouteBridge`, `UiEventPacket`, `HostRouteMap`, `PreparedUiFrameResource`, and render submission registries.

## Capability inventory

| Capability | Exists now | Missing contract | Required owner |
|---|---:|---|---|
| Engine App plugin composition | yes | none | `engine/src/app`, `engine/src/plugin.rs` |
| Public `UiPlugin` | no | install surface, resources, schedules | `engine/src/plugins/ui` |
| `app.mount_ui(Screen)` | no | ergonomic API | `engine/src/plugins/ui/app_ext.rs` |
| Source-backed UI IR | yes | typed builder lowering | `ui_definition` + engine UI facade |
| Typed screen authoring | conceptual | `UiScreen`, `IntoUi` | `engine/src/plugins/ui` |
| Typed action handling | no public path | `UiActionHandler` path | `engine/src/plugins/ui/action.rs` |
| Host route vocabulary | yes | engine adapter to `World` mutation | `ui_hosts` + `engine/src/plugins/ui/host.rs` |
| Renderer-facing frame payload | yes | generic render vocabulary / alias strategy | `ui_render_data` + render plugin |
| Render submission/preparation | yes/current UI-specific | producer-agnostic surface-frame contract | `engine/src/plugins/render` |
| Full live report chain | no | runtime report | `engine/src/plugins/ui/report.rs` |

## Alternatives considered

| Option | Long-term fit | Recommendation |
|---|---|---|
| Grow `ui_app_integration` into final framework | poor | reject |
| Add broad `domain/ui/ui_runtime_platform` crate | poor | reject |
| Add `engine/src/plugins/ui` and reuse domain crates | strong | accept |
| Make RenderPlugin own UI runtime | poor | reject |
| Genericize render around surface-frame submissions | strong | accept, staged |
| Force users to write host adapters | poor | reject for normal path |
| Typed `UiScreen` + `UiActionHandler` | strong | accept |

## Implementation gate

Implementation may not begin until:

- PR #72 closeout/post-merge truth is recorded;
- complete investigation dossier is recorded;
- complete design gate checklist is satisfied;
- planning record promotes this to active-planning or active-implementation with exact contract;
- allowed and forbidden files/crates are named;
- validation envelope is accepted;
- module decomposition map is accepted;
- feature support, future-use-case, hierarchy/composition, and ergonomics matrices are present;
- decision-register records the lifecycle transition.

## Proposed implementation sequence after accepted design

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

Stop and redesign if a follow-up tries to make renderer or SDF targets own UI semantics, make ECS entities the durable UI semantic model, make `ui_app_integration` the final host-neutral framework, require normal app authors to write manual route maps/host adapters, introduce `Factory` vocabulary as primary public API, duplicate `ui_surface` or `ui_hosts`, create a broad `domain/ui/ui_runtime_platform` crate, bypass `ui_definition`, `FormedInteractionModel`, `UiProgram`, or `UiStory`, merge scene/debug overlay producer logic into `RenderPlugin` as durable API, implement SDF/SpatialCanvas/world-space/visual designer/alternate execution strategy here, or add `foundation/meta` / a generic cross-domain plugin framework.

## Relationship to current work

This intake depends on PR #72 closeout/post-merge truth. It may be reviewed as a proposed design, but it must not move to active implementation until the current active work is closed out and the complete design gate is satisfied.
