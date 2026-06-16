---
title: Runenwerk UI Platform Capability Roadmap
description: Split UI production roadmap for story proof substrate, reusable components, visual designer work, game screen HUD, and deferred game world-space UI ownership.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-16
related_designs:
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-runtime-rendering-pipeline-roadmap.md
  - ./game-runtime-ui-projection-and-hud-platform-design.md
  - ./viewport-camera-and-projection-contract-platform-design.md
  - ../accepted/ui-designer-workbench-product-design.md
  - ./ui-program-architecture.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../domain/ui/story-acceptance-and-review-checklist.md
  - ../../workspace/production-tracks.yaml
---

# Runenwerk UI Platform Capability Roadmap

## Status

This is an active UI planning roadmap. It corrects the UI production-track
ownership split and does not authorize implementation, new crates, generated
planning document edits, runtime UI behavior, renderer changes, ECS changes, or
app/product behavior by itself.

Each implementation slice still requires normal WR, production-track,
validation, and closeout authority.

## Decision

`UiStory` is the proof substrate, not the owner of every future UI feature.

Correct ownership order:

```text
PT-UI-STORY-PLATFORM
  -> story manifests, registry, runner, run report, mount eligibility
  -> gallery/CLI story execution
  -> story-gated static/runtime rendering proof

PT-UI-COMPONENT-PLATFORM
  -> reusable base components, interaction, text, GraphCanvas, Timeline
  -> generic component transitions/effects only when component-scoped

Designer/Workbench tracks
  -> Visual UI Builder, authoring UX, editor/workbench product workflows

PT-GAME-RUNTIME-UI
  -> screen-space game HUD target profile, view-model packets, intents,
     engine UI submission, SDF screen HUD runtime proof

PT-GAME-WORLDSPACE-UI
  -> deferred nameplates, damage numbers, boss frames, split-screen attachment,
     entity-attached UI, world-space projection consumption, spatial anchors,
     culling, occlusion, and world/game input policy

PT-VIEWPORT-PROJECTION
  -> camera, projection, viewport, and surface-fit contracts only
```

Downstream UI work consumes `UiStoryRunReport` where story-derived eligibility,
preview, diagnostics, or runtime proof is relevant. Consuming story proof does
not move component, designer, game, world, or viewport ownership into
`PT-UI-STORY-PLATFORM`.

## Governing Rules

- Story proof comes before expanded rendering or product mounting claims.
- Reusable component maturity starts after `PM-UI-STORY-004`, not after the
  entire story track must close.
- Visual authoring and product builder workflows stay in Designer/Workbench
  tracks.
- Screen-space game HUD behavior stays in `PT-GAME-RUNTIME-UI`.
- World-space and entity-attached game UI stays deferred in
  `PT-GAME-WORLDSPACE-UI`.
- `PT-VIEWPORT-PROJECTION` remains the owner of camera/projection/surface-fit
  contracts and never becomes a game UI feature owner.

## Story Proof Substrate Roadmap

Owning track: `PT-UI-STORY-PLATFORM`.

Scope:

```text
domain/ui/ui_story/
story manifests/assets
UiStoryManifest
UiStoryRegistry
UiStoryRunner
UiStoryRunReport
UiStoryMountEligibility
gallery/CLI story execution
story-gated static/runtime rendering proof
story acceptance docs/checklists
```

### PM-UI-STORY-001 - Story Workflow Authority And Track Activation

Activate the story-first production track as planning and sequencing authority
only. This milestone records the bounded split, keeps `ai_executable: false`,
defers the standalone static gallery rendering path, and forbids runtime code,
crate creation, gallery migration, product mounting, component maturity,
designer product work, game HUD behavior, and world-space UI.

### PM-UI-STORY-002 - Story Manifest, Registry, Runner, And Report Contract

Record and later implement the public story contracts:

- `UiStoryManifest`
- `UiStoryRegistry`
- `UiStoryRunner`
- `UiStoryRunReport`
- `UiStoryMountEligibility`

The output is the proof envelope used by gallery preview, CLI inspection, static
mount, and product mount eligibility. This milestone does not own controls,
text, GraphCanvas, Timeline, designer product UX, game HUD behavior, or
world-space UI.

### PM-UI-STORY-003 - Gallery And CLI Story Execution

Gallery preview and CLI inspection consume story manifests and
`UiStoryRunReport`. Failure stories are first-class and expose source, stage,
diagnostic, and eligibility verdicts.

This milestone may adapt gallery/CLI inspection to the story proof envelope; it
does not implement Visual UI Builder product authoring.

### PM-UI-STORY-004 - Story-Gated Runtime Rendering Proof

The former static button-gallery rendering evidence is re-run as a story stage.
Static mount success requires `UiStoryMountEligibility`, and renderer-facing code
consumes backend-neutral primitives or frames without owning authored UI
semantics.

This milestone is the earliest dependency point for reusable component maturity.

### PM-UI-STORY-005 - Story Platform No-Gap Audit And Closeout

Close only the story proof substrate. The no-gap audit checks story contracts,
reports, gallery/CLI proof, story-gated rendering proof, truth evidence,
generated planning state, and bypass paths.

Component maturity, visual builder/product authoring, screen HUD behavior, and
world-space/entity-attached UI remain delegated to their own tracks and are not
story-platform gaps.

## Reusable UI Component Platform Roadmap

Owning track: `PT-UI-COMPONENT-PLATFORM`.

Dependency: starts after `PM-UI-STORY-004` so reusable components can be proven
through story reports. It does not need to wait for story-platform no-gap
closeout unless the future WR explicitly requires that.

Scope:

```text
domain/ui/ui_controls/
domain/ui/ui_interaction/
domain/ui/ui_text/
domain/ui/ui_graph_canvas/
domain/ui/ui_timeline/
domain/ui/ui_effects/     # only generic component-level primitives
component stories/assets
component diagnostics/docs/examples
```

### PM-UI-COMPONENT-001 - Component Platform Boundary And Track Activation

Record reusable component ownership and stop conditions. This milestone blocks
designer product authoring, screen HUD behavior, world-space/entity-attached UI,
and viewport projection contract ownership from entering the component track.

### PM-UI-COMPONENT-002 - Base Component Story Matrix

Cover button, label, input, panel, list, picker, and navigation controls with
story matrices for normal, edge, failure, and accessibility states.

### PM-UI-COMPONENT-003 - Generic Interaction Platform

Own reusable interaction traces, hit testing, focus, hover, pressed, disabled,
route proposal, and host intent boundaries for components. Game input policy,
world input policy, and editor product workflow policy are not implemented here.

### PM-UI-COMPONENT-004 - Generic Text Platform

Own reusable text layout, shaping requests, editing behavior, accessibility
labels, diagnostics, and render primitive evidence. Game HUD vocabulary,
world-space label binding, and Designer/Workbench product copy workflows are
out of scope.

### PM-UI-COMPONENT-005 - GraphCanvas And Timeline Components

Own reusable GraphCanvas and Timeline component maturity. Editor workflows may
consume these components, but product-specific visual authoring surfaces remain
Designer/Workbench ownership.

### PM-UI-COMPONENT-006 - Generic UI Transition And Effects Primitives

Own only reusable component-level transitions/effects that can be represented as
story-report stages and renderer-facing primitive boundaries. Game damage
numbers, nameplate effects, world-space visibility, product animation tools, and
renderer-authored UI semantics are out of scope.

### PM-UI-COMPONENT-007 - Component Platform Docs Evidence And Runtime-Proven Closeout

Close component maturity at `runtime_proven` quality with honest known gaps and
handoffs to Designer/Workbench, `PT-GAME-RUNTIME-UI`, and
`PT-GAME-WORLDSPACE-UI`.

## Visual Designer And Workbench Roadmap

Owning tracks: existing Designer/Workbench tracks, including
`PT-UI-DESIGNER-WORKBENCH` and related UI Designer productization tracks.

Scope:

- Visual UI Builder
- authoring UX
- editor/workbench authoring surfaces
- UI Designer product workflows
- app/editor product integration for designer workflows

The story platform supplies proof reports, and the component platform supplies
reusable controls. The builder consumes those contracts; it does not make
`PT-UI-STORY-PLATFORM` the owner of visual authoring product work.

## Game Runtime Screen HUD Roadmap

Owning track: `PT-GAME-RUNTIME-UI`.

Scope:

- game-runtime target profile
- runtime view-model packets
- validated intent proposals
- engine UI submission
- SDF screen HUD proof
- runtime-proven game HUD closeout

The screen HUD track may consume story proof and reusable components where
relevant, but it remains the owner of concrete screen-space game runtime UI.

## Deferred Game World-Space UI Roadmap

Owning track: `PT-GAME-WORLDSPACE-UI`.

Deferred scope:

- nameplates
- damage numbers
- boss frames
- split-screen attachment UI
- entity-attached UI
- world-space projection consumption
- spatial anchors
- culling and occlusion
- world/game input policy
- gameplay/world/entity binding boundaries

Dependencies before activation:

- `PM-UI-STORY-004` for story-gated rendering proof and story-derived
  eligibility where authored UI is relevant.
- `PT-GAME-RUNTIME-UI` target/profile work for game runtime view-model and
  intent boundaries.
- `PT-VIEWPORT-PROJECTION` readiness where camera, projection, viewport, and
  surface-fit contracts are required.
- Renderer/projection readiness where rendering and visibility evidence is
  required.

This track must not be implemented as a patch to `PT-GAME-RUNTIME-UI` screen HUD
proof, as a story-platform feature, or as a viewport-projection feature.

## Viewport Projection Boundary

Owning track: `PT-VIEWPORT-PROJECTION`.

Scope:

- camera contracts
- projection contracts
- viewport contracts
- surface-fit contracts
- projection diagnostics

Non-scope:

- game HUD behavior
- world-space UI features
- nameplates
- damage numbers
- boss frames
- entity-attached UI
- gameplay/world/entity binding

World-space game UI consumes viewport projection contracts after those contracts
are ready; it does not move feature ownership into `PT-VIEWPORT-PROJECTION`.

## Stop Conditions

Stop and replan if any future slice tries to:

- implement reusable component maturity inside `PT-UI-STORY-PLATFORM`;
- implement Visual UI Builder or product authoring UX inside
  `PT-UI-STORY-PLATFORM` or `PT-UI-COMPONENT-PLATFORM`;
- implement game HUD behavior inside story/component/designer tracks;
- implement world-space, entity-attached, projected attachment, or
  gameplay/world binding UI outside `PT-GAME-WORLDSPACE-UI`;
- make `PT-VIEWPORT-PROJECTION` own game UI features;
- claim mount eligibility, gallery preview, CLI inspection, or static/product
  mount success without `UiStoryRunReport` where story proof is required;
- hand-edit generated production or roadmap docs.
