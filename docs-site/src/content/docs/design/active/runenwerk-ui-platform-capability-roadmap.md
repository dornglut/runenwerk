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

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:roadmap -->
## UI Component Platform activation: ControlPackage and reusable surface roadmap

`PT-UI-COMPONENT-PLATFORM` owns reusable, story-proven `ControlPackage` and surface maturity. It starts after `PM-UI-STORY-004`, because reusable controls and surfaces must be proven through story reports. It does not need to wait for `PM-UI-STORY-005` unless a future accepted WR explicitly requires that dependency.

### Canonical vocabulary

- `ControlPackage` — packaged reusable control family with schema, state, interactions, diagnostics, fixtures, stories, accessibility, theme/token requirements, render facts, and host routes.
- `control kernel` — the reusable contract every `ControlPackage` follows.
- `control authoring kit` — templates, naming rules, fixture/story patterns, diagnostic conventions, and checklists that make new controls easy to define.
- `component story matrix` — story-proven scenarios for normal, edge, failure, accessibility, interaction, layout, text, mount, and render states.
- `story proof envelope` — `UiStoryRunReport`, evidence records, expected-failure matching, CLI/Gallery report projection, and mount eligibility.
- `catalog/discovery/inspection contract` — searchable, filterable, inspectable control/surface metadata for Gallery, UI Designer, Workbench, and docs.
- `host intent proposal` — UI proposes an action; app/editor/game domain decides mutation.
- `route/capability decision` — host-owned decision on whether a route/action is allowed.
- `state bucket` — explicit transient, preview, committed, focus, hover, drag, animation, host-fed, and package-owned state ownership.
- `binding/form validation` — read/write/collection/option bindings, validation state, dirty state, commit/cancel, and diagnostics.
- `theme/token state style` — token-backed normal, hover, pressed, focused, selected, disabled, error, warning, and info styles.
- `accessibility/focus/inspection facts` — role, label, focus order, keyboard activation, value/range metadata, and source-mapped inspection facts.
- `render/surface output contract` — renderer-neutral primitive and surface output with provenance.
- `layout/container/virtualization` — generic panel, row, column, stack, split, scroll, list, table, tree, large-data, and virtualization contracts.
- `overlay/popup/layering` — overlay root, popup, tooltip, context menu, submenu, picker popup, z-order, dismiss, focus return, and clipping policy.
- `Surface2D` — generic 2D coordinate/navigation primitive: transforms, pan, zoom, bounds, fit, overlays.
- `SpatialCanvas` — generic positioned-item surface: item bounds, hit regions, selection, marquee, labels, badges, culling.
- `NodeCanvas` — generic node/link surface for skill trees, tech trees, dialogue trees, quest trees, behavior trees, and graph viewers.
- `PortGraphCanvas` — editable port/socket graph specialization.
- `ProgressionTreeView` — reusable skill/tech/progression-tree package built on `NodeCanvas`; gameplay/progression rules remain outside Component Platform.
- `TrackSurface` — generic time/track surface; `Timeline` and `CurveEditor` specialize it.
- `Gallery / Workbench / Designer adoption gate` — proof that reusable controls/surfaces are inspectable, adopted by real editor surfaces, and not private demos.

### Non-negotiable rules

- General kernels before specializations.
- Story proof before mount eligibility.
- Control packages before product-specific surfaces.
- `Surface2D` before GraphCanvas/Timeline.
- `SpatialCanvas` before product-specific positioned-item surfaces.
- `NodeCanvas` before skill tree or node editor.
- `PortGraphCanvas` only for editable port/socket graph behavior.
- `TrackSurface` for time/track behavior; Timeline does not inherit graph semantics.
- Host intent before mutation.
- Renderer-neutral output before renderer-specific behavior.
- Gallery, Designer, and Workbench consume reusable controls/surfaces; they do not own reusable control semantics.

### Roadmap phases

| Phase | Branch | Milestone mapping | Output |
|---:|---|---|---|
| 0 | `feature/ui-component-platform-000-activation-vocabulary-ergonomics` | `PM-UI-COMPONENT-001` | Track activation, vocabulary, ergonomics doctrine, anti-overfitting rules. |
| 1 | `feature/ui-component-platform-001-control-kernel` | `PM-UI-COMPONENT-001`, `PM-UI-COMPONENT-002` foundation | `ControlPackage` / control-kernel contract. |
| 2 | `feature/ui-component-platform-002-authoring-kit` | `PM-UI-COMPONENT-002` foundation | Control authoring kit: templates, naming, fixtures, story matrix, diagnostics. |
| 3 | `feature/ui-component-platform-003-story-proof-envelope` | `PM-UI-COMPONENT-002` foundation | Story proof envelope for controls and surfaces. |
| 4 | `feature/ui-component-platform-004-catalog-discovery-inspection` | `PM-UI-COMPONENT-002` ergonomics | Catalog/discovery/inspection contract for Gallery, Designer, Workbench, docs. |
| 5 | `feature/ui-component-platform-005-input-gesture-device` | `PM-UI-COMPONENT-003` foundation | Input/gesture/device kernel. |
| 6 | `feature/ui-component-platform-006-state-binding-host-intent` | `PM-UI-COMPONENT-002`, `PM-UI-COMPONENT-003` | State buckets, binding/form validation, host intent proposals. |
| 7 | `feature/ui-component-platform-007-theme-state-style` | `PM-UI-COMPONENT-002`, `PM-UI-COMPONENT-006` foundation | Theme/token state style ergonomics. |
| 8 | `feature/ui-component-platform-008-accessibility-focus-inspection` | `PM-UI-COMPONENT-002`, `PM-UI-COMPONENT-003`, `PM-UI-COMPONENT-004` | Accessibility/focus/inspection facts. |
| 9 | `feature/ui-component-platform-009-layout-container-virtualization` | `PM-UI-COMPONENT-002`, `PM-UI-COMPONENT-003` support | Layout/container/virtualization kernel. |
| 10 | `feature/ui-component-platform-010-render-surface-output` | `PM-UI-COMPONENT-002`, `PM-UI-COMPONENT-004`, `PM-UI-COMPONENT-005`, `PM-UI-COMPONENT-006` | Render/surface output contract. |
| 11 | `feature/ui-component-platform-011-base-control-packages` | `PM-UI-COMPONENT-002` | Base `ControlPackage`s: Button, Label, Panel, Toggle, Input, TextInput, NumericInput, List, Table, Tree, Picker, Navigation, Tabs, StatusBadge, TooltipTrigger. |
| 12 | `feature/ui-component-platform-012-generic-interaction` | `PM-UI-COMPONENT-003` | Deterministic interaction traces, host intent, post-interaction frame proof. |
| 13 | `feature/ui-component-platform-013-overlay-popup-layering` | `PM-UI-COMPONENT-003` support | Overlay/popup/layering kernel. |
| 14 | `feature/ui-component-platform-014-minimum-text-editing` | `PM-UI-COMPONENT-004` foundation | Minimum single-line editing path. |
| 15 | `feature/ui-component-platform-015-generic-text` | `PM-UI-COMPONENT-004` | Generic text platform: wrapping, truncation, glyph runs, fail-closed diagnostics. |
| 16 | `feature/ui-component-platform-016-surface2d` | `PM-UI-COMPONENT-005` foundation | Generic `Surface2D` coordinate/navigation primitive. |
| 17 | `feature/ui-component-platform-017-spatial-canvas` | `PM-UI-COMPONENT-005` foundation | Generic positioned-item `SpatialCanvas`. |
| 18 | `feature/ui-component-platform-018-node-canvas` | `PM-UI-COMPONENT-005` foundation | Generic `NodeCanvas` node/link view. |
| 19 | `feature/ui-component-platform-019-port-graph-canvas` | `PM-UI-COMPONENT-005` | Editable `PortGraphCanvas` / GraphCanvas package. |
| 20 | `feature/ui-component-platform-020-progression-tree-view` | `PM-UI-COMPONENT-005` extension | Skill/tech/progression-tree package without gameplay rule ownership. |
| 21 | `feature/ui-component-platform-021-track-surface-timeline` | `PM-UI-COMPONENT-005` | `TrackSurface`, Timeline, CurveEditor. |
| 22 | `feature/ui-component-platform-022-transitions-effects` | `PM-UI-COMPONENT-006` | Generic transition/effect primitives. |
| 23 | `feature/ui-component-platform-023-adoption-gates` | `PM-UI-COMPONENT-007` readiness | Gallery, Workbench, Designer adoption gates. |
| 24 | `feature/ui-component-platform-024-runtime-proven-closeout` | `PM-UI-COMPONENT-007` | Runtime-proven closeout, handoffs, no-bypass audit. |

### Phase acceptance template

Each phase must document branch name, milestone mapping, decision, feature list, primary files/designs, validation gate, visible/usable output, and explicit out-of-scope.

### Reject list

- No Gallery-owned control platform.
- No GraphCanvas as the generic 2D canvas root.
- No skill-tree rules inside GraphCanvas or Component Platform.
- No ports inside generic NodeCanvas.
- No Timeline inheriting graph semantics.
- No renderer-owned UI semantics.
- No ECS-owned UI semantics.
- No global semantic event enum for every surface action.
- No hidden package registry.
- No app-only mutation shortcuts.
- No Designer product authoring in Component Platform.
- No screen-space game HUD runtime behavior in Component Platform.
- No world-space/entity-attached UI behavior in Component Platform.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:roadmap -->
