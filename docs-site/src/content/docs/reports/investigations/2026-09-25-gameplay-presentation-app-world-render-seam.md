---
title: Gameplay Presentation and App-World-to-Render Seam Investigation
description: Current-source investigation selecting the smallest Runenwerk presentation seam for a maintained App-World game without assigning gameplay authority to Scene or Render.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_docs:
  - ./2026-09-25-shared-small-game-gold-path.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../engine/reference/plugins/render/architecture.md
  - ../../engine/reference/plugins/scene/architecture.md
---

# Gameplay Presentation and App-World-to-Render Seam Investigation

## Purpose

Issue #949 asks for the smallest owner-correct path from the maintained game's App-World state to
Runenwerk presentation.

The required end state is:

```text
authoritative / predicted gameplay state in App World
        |
        | game-owned extraction / presentation derivation
        v
prepared render-facing state
        |
        v
existing Render prepare / submit contracts
```

with these invariants:

- Scene does not become gameplay authority;
- Render does not discover or own arbitrary gameplay components;
- presentation state is derived and never fed back as simulation truth;
- a dedicated authority can run with no Render, UI, camera, or presentation state.

This report selects the owner boundaries and the first implementation slices. It does not implement
them.

## Evidence baseline

The source census was re-resolved after the accepted RunenRender temporal delivery and uses accepted
Runenwerk main:

```text
56dee6df9e31f107e23cfa4248e5e518b27f2020
```

This baseline includes accepted PR #953, which added fixed internal-resolution execution plus
surface-scoped frame requests and dynamic-target routing. GP0 was re-audited against that accepted
implementation rather than relying on its earlier Render baseline.

The investigation read current repository governance and the owning Render, Scene, UI, fixed-step,
native-surface, and Editor viewport paths. Important source included:

```text
AGENTS.md
TESTING.md
ARCHITECTURE.md
docs-site/src/content/docs/workspace/documentation-structure.md
docs-site/src/content/docs/reports/investigations/2026-09-25-shared-small-game-gold-path.md

engine/src/plugins/render/plugin.rs
engine/src/plugins/render/runtime/frame_prepare.rs
engine/src/plugins/render/runtime/frame_submit.rs
engine/src/plugins/render/frame/contribution_registry.rs
engine/src/plugins/render/frame/contributions.rs
engine/src/plugins/render/frame/packet.rs

engine/src/plugins/scene/plugin.rs
engine/src/plugins/scene/types.rs
engine/src/plugins/scene/**

engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/**
engine/src/runtime/fixed_time.rs
engine/src/runtime/fixed_step_executor.rs
engine/src/runtime/winit_runner.rs

apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs
apps/runenwerk_editor/src/runtime/systems/frame_submit.rs
```

Search was discovery only. Ownership conclusions below come from the exact owning source.

## Post-#953 Render re-audit

Accepted PR #953 materially changed `frame_prepare.rs`, but it did not remove the Scene-manager
lifecycle dependency identified by GP0.

At the accepted baseline:

- `frame_render_prepare_system` still clears the prepared frame and returns when
  `SceneResource.manager` is absent;
- `frame_render_submit_system` still returns early when the Scene manager is absent;
- `RenderPlugin` still initializes `SceneResource` as shared substrate; accepted Scene architecture
  explicitly permits shared resource presence while reserving manager/runtime activation to
  `ScenePlugin`;
- attached surface extent still comes from `RenderSurfaceRegistryResource`;
- `PrimaryPresentationMetricsResource` remains the host-neutral primary logical-presentation
  owner and is not installed by `RenderPlugin`;
- Scene-route feature dependencies remain ordering-only;
- the Scene-route feature still owns `EmptyContribution` fallback semantics.

What #953 **did** strengthen is surface ownership. Prepared frame requests, automatic-main
replacement claims, fixed-resolution execution, and dynamic target requests are now scoped to the
owning `RenderSurfaceId`.

That does not change GP0's owner decision. It adds a hard GP1A preservation rule:

> removing the Scene gate must not collapse, globalize, or bypass the accepted per-surface
> preparation/routing semantics introduced by #953.


# 1. The current Scene dependency is accidental Render integration coupling

The G0 report correctly identified that Render currently cannot prepare an ordinary frame without a
live Scene manager. GP0 finds that the coupling is stronger than one prepare-time read.

## 1.1 RenderPlugin does not activate Scene

`RenderPlugin` initializes `SceneResource`, but only `ScenePlugin` installs
`SceneIntegrationActivation` and the scene runtime systems.

That separation is intentional, tested, and documented by the canonical Scene architecture:
shared `SceneResource` presence is substrate only; explicit private `ScenePlugin` activation gates
Scene manager/runtime-control semantics.

Therefore, the GP1A defect is **not** the existence of shared `SceneResource` substrate. The defect
is Render treating `SceneResource.manager` as a mandatory frame-lifecycle prerequisite.

GP1A should remove that semantic dependency without broadening into Scene substrate ownership
cleanup.

## 1.2 RenderPrepare hard-gates on SceneManager

`frame_render_prepare_system` begins by reading `SceneResource.manager`.

When no manager exists it:

```text
clear_prepared_frame(...)
return Ok(())
```

When a manager exists it derives three values from Scene:

1. fallback primary target size from `manager.overlay_runtime.ui.screen_size`;
2. `manager.world.active.label()`;
3. `manager.active_overlay().label()`.

It also routes shader hot-reload log lines into Scene overlay UI state.

These are four distinct responsibilities and should not be treated as one required Scene contract.

## 1.3 RenderSubmit independently hard-gates on SceneManager

`frame_render_submit_system` performs another early check:

```rust
if world.resource::<SceneResource>()?.manager.is_none() {
    return Ok(());
}
```

This means fixing prepare alone would still leave an ordinary Render-selected game unable to submit
its prepared native frame.

The first integration repair must therefore remove the mandatory Scene-manager gate from both
prepare and submit.

# 2. Native surface identity and size already have the correct owner

The Scene overlay screen size is not the authoritative native render-surface size.

Current native runtime state already contains the owner-correct path:

```text
NativeWindowId
        |
WindowStateRegistryResource
        |
RenderSurfaceRegistryResource
  - RenderSurfaceId
  - native window binding
  - lifecycle state
  - target_size_px
        |
PreparedSurfaceInfo
```

`prepared_surface_infos` already prefers attached surface records and copies
`record.target_size_px` into each `PreparedSurfaceInfo`.

`frame_render_submit_system` then validates the exact prepared surface against
`RenderSurfaceRegistryResource`, and the renderer sizes the actual surface from that prepared
surface/view state.

### Decision

For a native windowed game, **RenderSurfaceRegistryResource owns the concrete render target size**.

Scene overlay UI size must not remain a prerequisite for native Render preparation or submission.

The existing `primary_target_size` parameter in `prepared_surface_infos` is only a fallback for the
unbound-primary path when no attached native surface exists.

Current source already has the owner-correct fallback:
`PrimaryPresentationMetricsResource`. It is explicitly host-neutral logical-presentation state;
the native Host keeps it synchronized with primary-window size/scale, Scene itself consumes it to
size its overlay viewport, and caller-supplied headless metrics are preserved.

### Decision

For attached native surfaces, `RenderSurfaceRegistryResource` remains authoritative for each
surface's concrete target extent.

For the unbound-primary path, GP1A may use
`PrimaryPresentationMetricsResource::size_px()` **when logical presentation has already been
selected by its owner**. `RenderPlugin` must not initialize
`PrimaryPresentationMetricsResource` merely to manufacture a fallback target: current App
architecture intentionally leaves logical presentation absent from a bare App, while native Host
realization and Scene selection initialize it when required.

Therefore the fallback law is:

```text
attached Render surface
  -> use RenderSurfaceRegistryResource.target_size_px

no attached surface + PrimaryPresentationMetricsResource present
  -> prepare unbound primary from selected logical presentation size

no attached surface + no logical presentation metrics
  -> no presentable/unbound-primary frame target is inferred
```

A headless/offscreen consumer that genuinely requires an unbound-primary logical presentation must
select or insert the logical presentation metrics explicitly. GP1A does not need a new render-size
resource and must not turn Render selection into presentation-metrics ownership.

A headless dedicated game does not select Render at all, so GP1A does not make native presentation a
headless-server concern.

# 3. Scene route is optional presentation metadata, not a Render lifecycle dependency

The current Scene-derived world and overlay labels are packaged into
`PreparedSceneRouteContribution`.

The Render feature registry does declare ordering edges such as:

```text
scene.route -> ui
scene.route -> world.draw
```

but current `RenderFeatureDescriptor::depends_on(...)` is used only by
`resolve_feature_order(...)` to topologically order features. No current execution path uses those
edges as semantic availability gates.

A complete source search also finds no renderer behavior that consumes the world/overlay label
values as scene authority. The payload is currently used for prepared-frame inspection/signature
hashing; Scene remains the only semantic owner of those labels.

The built-in Scene-route feature already declares:

```text
FeatureFallbackPolicy::EmptyContribution
```

### Decision

Do **not** introduce a generic presentation-route abstraction for GP1A.

When Scene is active, prepare a real `PreparedSceneRouteContribution` from Scene-owned labels.

When Scene is absent, Scene route should be represented truthfully as an absent/missing feature with
its existing `EmptyContribution` fallback semantics, without producing an error diagnostic and
without fabricating placeholder Scene identities.

Directionally:

```text
Scene active
  -> Some(PreparedSceneRouteContribution)
  -> scene.route = Ready

Scene absent
  -> None
  -> scene.route = Missing + EmptyContribution
```

The existing feature-order edges may remain because they are ordering constraints. They do not
justify mandatory Scene activation.

This implies `build_frame_feature_contributions` / registered feature collection should accept an
optional Scene route rather than requiring two strings for every Render frame.

# 4. Shader reload diagnostics must not require Scene UI

Shader-registry polling belongs to Render preparation. Current reload messages are subsequently
pushed directly into:

```text
SceneManager
  -> overlay_runtime
  -> ui.log_lines
```

That diagnostic sink is optional product behavior, not a Render lifecycle requirement.

### Decision

GP1A should preserve shader polling and Render diagnostics without requiring Scene.

If Scene is active, a Scene-specific adapter may continue to surface Render diagnostics in a Scene
overlay. Otherwise messages should remain available through Render/diagnostics owners or normal
tracing.

Do not create an application UI dependency merely to preserve the old overlay log side effect.

# 5. The first game should own gameplay-to-presentation extraction

The Editor already demonstrates the correct ownership pattern without being the generic game API.

Editor source reality is converted by Editor-owned code into explicit render products and requests.
For example, viewport code forms product views/flow invocations and publishes them through
`PreparedRenderFrameRequestResource`. Editor scene extraction retains Editor ownership of authored
scene semantics.

The reusable lesson is:

```text
source-domain truth
  -> source owner extracts a render-facing product
  -> Render consumes prepared contracts
```

The Editor packet types themselves are Editor semantics and must not be promoted as generic gameplay
types.

### Decision

The maintained game owns its presentation adapter.

Directionally:

```text
Game physical/predicted components
        |
        | game presentation system
        v
Game presentation snapshot / projection
        |
        +--> PreparedWorldFeatureResource / world-SDF presentation
        +--> PreparedDrawFeatureResource for explicit drawable actor data when required
        +--> PreparedMaterialFeatureResource for material selections when required
        +--> PreparedRenderFrameRequestResource for view/flow requests when required
        |
        v
RenderPrepare freezes the frame
```

The exact subset must be chosen from the first game's actual arena and actor representation. GP1B
must not create a universal `Renderable` component, derive macro, or generic ECS query bridge in
advance.

# 6. First actor/world representation

The selected game slice needs only enough visible state to pressure the seam:

- one small SDF arena;
- one player actor;
- one gameplay camera;
- simple material identity;
- visibility sufficient to show movement and collision results.

The arena should reuse the existing world/SDF prepared path where truthful.

The player may use the smallest existing prepared draw/world representation that can depict a simple
actor. It does not justify skeletal animation, imported model ownership, prefab semantics, or a new
render object model.

### Decision

GP1B should explicitly project only the first game's arena, actor, and camera into current prepared
Render contracts.

If current prepared contracts cannot express that without a reusable Render semantic addition, stop
and split the missing semantic into the owning Render boundary. Do not tunnel game types into the
renderer.

# 7. Physical pose and presentation pose must be separate

Current fixed cadence exposes:

```text
FixedTimeConfig.step_seconds
FixedTimeState.accumulator_seconds
FixedTimeState.steps_ran_last_frame
FixedTimeState.total_completed_steps
```

The fixed-step executor subtracts one step after each completed fixed update, leaving the
sub-step remainder in `accumulator_seconds` unless the catch-up budget saturates, in which case the
remainder is deliberately dropped.

That is sufficient source data to derive ordinary local frame interpolation after the fixed loop.

The first game should retain two physical poses around fixed advancement:

```text
previous physical pose
current physical pose
```

and derive a presentation pose using a bounded interpolation factor directionally equivalent to:

```text
alpha = clamp(accumulator_seconds / step_seconds, 0, 1)
presentation = interpolate(previous, current, alpha)
```

This deliberately renders between the previous and current fixed states and therefore introduces
roughly one fixed-tick of visual latency in exchange for stable interpolation. That tradeoff is
acceptable as the first policy and must remain presentation-only.

Teleport/restart/respawn discontinuities must reset interpolation history
(`previous == current`) so presentation never sweeps visually across a discontinuity.

The exact helper/API belongs to implementation after G1/P1 reveals the player state shape.

### Decision

- physical poses are simulation/gameplay state;
- presentation pose is derived frame state;
- camera follows presentation state for smooth local viewing;
- presentation pose never feeds the next fixed simulation step;
- prediction correction later changes the source physical/predicted timeline and may add visual
  smoothing, not simulation feedback;
- remote snapshot interpolation later uses a separate confirmed-state buffer/timeline.

Do not conflate local fixed-step interpolation with network snapshot interpolation.

# 8. Camera ownership

The camera is game/product presentation policy.

Render should consume a prepared observation/view; it should not own "the player camera" as gameplay
meaning.

The first game therefore owns:

- which player/actor the camera follows;
- camera offset/orientation policy;
- conversion from presentation pose to the prepared Render view/observation.

This remains disabled in a dedicated authority composition.

# 9. HUD can use the existing app-facing UI path

`UiPlugin` is independently selectable and publishes evaluated UI frames in `RenderPrepare`
before Render frame preparation when both plugins are present.

The first game does not need Scene overlay UI for its HUD.

### Decision

Use app-mounted screen-space UI for:

- health;
- objective state;
- win/lose state;
- restart affordance/message.

The game owns projection from authoritative gameplay resources/components into UI host data/state.
UI owns evaluation and render publication.

World-space UI and entity-attached UI remain deferred.

# 10. Headless composition stays clean

The selected presentation seam has no role in authoritative simulation correctness.

Required composition law:

```text
Local / Host / Client presentation:
  Game simulation
  + optional UiPlugin
  + RenderPlugin
  + native Host

Dedicated authority:
  Game simulation
  + Net when multiplayer
  - RenderPlugin
  - UiPlugin
  - camera/presentation systems
```

GP1B presentation systems must therefore be installed through an explicit presentation/game-render
plugin or equivalent product composition, not by making gameplay components require render
resources.

# 11. Selected implementation split

The investigation proves that two separately owned deliveries are required.

## GP1A — remove accidental Scene-manager requirement from generic Render lifecycle

Owner: Runenwerk Engine Render/App integration.

Bounded scope:

1. remove the `SceneResource.manager` early-return gate from
   `frame_render_prepare_system`;
2. remove the independent Scene-manager early-return gate from
   `frame_render_submit_system`;
3. preserve the accepted shared-`SceneResource` substrate contract unless a separate owner change
   proves it unnecessary; GP1A must only remove Render's dependency on a live Scene manager;
4. derive attached native target size from the existing render-surface owner;
5. use existing `PrimaryPresentationMetricsResource` for the unbound-primary logical extent only
   when that logical presentation state already exists; do not make `RenderPlugin` initialize it;
6. when neither an attached surface nor logical presentation metrics exists, do not manufacture a
   presentable target;
7. make Scene route optional during contribution collection;
8. publish no-Scene route as truthful `Missing + EmptyContribution`, not an error or fabricated
   Scene identity;
9. keep Scene-specific route labels available when Scene is selected;
10. preserve shader reload diagnostics without requiring Scene overlay UI;
11. prove Render can prepare and submit a native frame without `ScenePlugin`;
12. prove app-mounted UI can prepare/render without `ScenePlugin` when no Scene semantics are
   selected;
13. prove Scene-selected behavior still contributes its route/overlay metadata;
14. preserve the accepted #953 surface-scoped semantics for prepared frame requests,
    automatic-main replacement claims, fixed-resolution execution, dynamic texture targets, and
    prepared surface identity across primary and secondary surfaces.

Non-goals:

- gameplay rendering;
- Scene rewrite/removal;
- changing Scene lifecycle semantics;
- broad Render architecture redesign;
- direct WGPU game code;
- changing RunenRender semantic contracts unless a concrete missing owner contract is proven.

At this re-audit baseline, no open PR owns the Render runtime paths GP1A needs; accepted #953 is no
longer an active writer. GP1A implementation must still re-resolve active writers immediately before
publication and serialize if a new overlapping Render writer appears.

## GP1B — maintained-game presentation adapter

Owner: maintained game product.

Activation requires accepted G1, accepted first movement/physics shape, and accepted GP1A if the
current Scene gate remains present.

Bounded scope:

1. install the game's presentation systems only in rendered compositions;
2. retain previous/current physical pose needed for local interpolation;
3. derive per-frame presentation pose after fixed advancement;
4. derive the first gameplay camera from presentation state;
5. project the arena/player into existing prepared Render resources;
6. publish only the views/flow inputs needed by the maintained game;
7. mount the first HUD through the current app-facing UI runtime;
8. prove simulation remains valid when presentation plugins are absent.

Non-goals:

- networking correction/interpolation;
- generic ECS-to-render framework;
- prefab/animation;
- world-space UI;
- editor viewport abstraction reuse;
- runtime content bootstrap.

# 12. Multiplayer reuse

This presentation split intentionally prepares for multiplayer without implementing it.

Later client presentation becomes:

```text
locally controlled actor
  predicted physical state
  -> optional correction smoothing
  -> presentation pose

remote actor
  confirmed snapshot buffer
  -> delayed interpolation
  -> presentation pose
```

Both feed the same game-owned Render projection, while neither changes the authoritative/predicted
simulation contract.

The renderer remains unaware of whether a pose came from local authority, prediction, or remote
interpolation.

# 13. Rejected directions

GP0 rejects:

- activating `ScenePlugin` merely so Render will run;
- moving gameplay state into `SceneManager.world_runtime`;
- treating Scene overlay screen size as native render-surface authority;
- keeping RenderSubmit gated on Scene after prepare is decoupled;
- fabricating placeholder Scene-route identities merely to satisfy feature ordering;
- making shader reload logging a mandatory Scene/UI dependency;
- reusing Editor viewport packets as the generic game API;
- adding a generic `Renderable` ECS abstraction before the game proves one is needed;
- feeding interpolated presentation pose back into simulation;
- requiring Render/UI/camera resources in dedicated authority;
- solving remote-network interpolation before a real replicated game actor exists.

# 14. External mechanism pressure test

The selected physical/presentation split matches current engine practice without requiring foreign
API adoption.

Bevy's maintained fixed-timestep example keeps current and previous physical positions separately,
accumulates input before the fixed loop, interpolates the visual transform after the fixed loop using
the fixed-time overstep fraction, and moves the camera from the interpolated player position:

- <https://bevy.org/examples-webgpu/movement/physics-in-fixed-timestep/>

Godot 4.7 documents the same previous/current fixed-state interpolation model and explicitly notes
the visual-latency tradeoff. Its guidance also requires interpolation history to be reset around
teleports/discontinuities and recommends special care for cameras:

- <https://docs.godotengine.org/en/4.7/tutorials/physics/interpolation/physics_interpolation_introduction.html>
- <https://docs.godotengine.org/en/4.7/tutorials/physics/interpolation/physics_interpolation_quick_start_guide.html>
- <https://docs.godotengine.org/en/4.7/tutorials/physics/interpolation/advanced_physics_interpolation.html>

Runenwerk adopts only the mechanism:

```text
fixed authoritative physical state
-> previous/current physical history
-> frame-derived presentation pose
-> camera/render presentation
```

It does not adopt Bevy or Godot component, scene, transform, or scheduler ownership.

# 15. Sequencing consequence

The game roadmap can safely proceed as:

```text
G1 command/authority spine
        |
        +--> P1 SDF-first character movement
        |
        +--> GP1A Render/Scene lifecycle decoupling
                    |
                    v
                 GP1B game presentation adapter
                    |
                    v
                 G2 local playable arena
```

GP1A is owner-correct Engine integration work. At the re-audit baseline its Render write set is
currently free, so it may begin once GP0 is accepted and the implementation issue is derived from
this decision.

GP1B remains consumer-gated by the accepted game/player state rather than inventing a generic
presentation ontology.

# Conclusion

Runenwerk does not need a new renderer or a second gameplay world to make the maintained game
visible.

The current blocking presentation defect is narrower:

```text
RenderPrepare and RenderSubmit still require a live SceneManager
```

even though native surface identity/size, prepared frame resources, feature contributions, UI frame
publication, and Render submission already have independent owners.

The correct repair is to remove that accidental lifecycle gate without conflating shared Scene
substrate presence with Scene activation, use existing host-neutral primary presentation metrics
rather than Scene overlay state, represent absent Scene route truthfully through the existing empty
fallback semantics, and let the maintained game own projection from App-World gameplay state into
existing prepared Render contracts.

After that, the first game presentation slice can remain small: one interpolated player pose, one
camera, one SDF arena, and one screen-space HUD.