---
title: UI Component Platform SpatialCanvas Design
description: Phase 17 design intake for a reusable SpatialCanvas contract built on completed Surface2D evidence.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-03
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./ui-component-platform-surface2d-design.md
  - ./ui-component-platform-node-canvas-design.md
  - ./ui-component-platform-port-graph-canvas-design.md
  - ./ui-component-platform-track-surface-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../domain/ui/crate-ownership.md
  - ../../domain/ui/dependency-boundaries.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../reports/investigations/phase-17-spatialcanvas-source-investigation.md
  - ../../reports/investigations/surface2d-future-pressure-branch-review.md
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../workspace/workflow-lifecycle.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
---

# UI Component Platform SpatialCanvas Design

## Status

This document opens `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas as a Phase 17 planning and design intake.

Lifecycle state: `active-planning`.

Implementation authorization: not granted.

Phase 17 implementation must not start from this document alone. It requires explicit promotion after the complete investigation gate, complete design gate, principle compliance matrix, module decomposition map, validation envelope, exact owner/files, and stop conditions are accepted in planning.

## Complete investigation gate status

Gate status: complete for opening the planning/design intake; not sufficient to authorize implementation.

Source investigation:

```text
docs-site/src/content/docs/reports/investigations/phase-17-spatialcanvas-source-investigation.md
```

Evidence classes used:

```text
E2 repository file and branch/PR metadata inspection
E3 source-code/test inspection
E5 local command output for git/GitHub state and later docs validation
E8 accepted workspace, UI-domain, planning, design, ADR, and closeout authority
```

Current branch and PR state at intake:

```text
main and origin/main point to 05c51375986cf08e360884ebf44702ec62662c1e
PR #64 merged the Surface2D future-pressure extraction
PR #63 merged the Phase 16 closeout
PR #62 merged workflow/principle/decomposition hardening
PR #61 merged Phase 16 Surface2D implementation
GitHub reports no open PRs
origin/surface2d-phase-16 is not present in the current remote branch list
```

## Complete design gate status

Status: proposed design intake only.

This document records the complete design questions and proposed owner shape required before a later implementation task can be promoted. It does not yet authorize active implementation because Phase 17 still needs review acceptance of:

```text
exact public API names
exact files/crates allowed
exact files/crates forbidden
implementation sequence
test names
proof-frame expectations
whether ui_tree is touched at all
whether ui_render_data changes are truly necessary
```

## Question

What exactly should `SpatialCanvas` own in the UI Component Platform after completed `Surface2D`, and how can it provide reusable spatial viewport/canvas behavior without becoming a renderer backend, app composition system, retained tree authority, graph editor, product scene graph, or product/editor/game command system?

## Current lifecycle

```text
PT-UI-COMPONENT-PLATFORM-016 Surface2D: completed
PT-UI-COMPONENT-PLATFORM-017 SpatialCanvas: active-planning intake
Candidate transition from this PR: production-track -> active-planning
Implementation: forbidden until separately promoted
```

## Authority/source matrix

| Claim | Source inspected | Authority level | Evidence found | Conflict / drift |
|---|---|---:|---|---|
| Phase 16 Surface2D is completed | `phase-16-surface2d-closeout.md`, planning files, PR #61/#62 metadata | E5/E8 | Surface2D package/catalog/inspection, runtime proof, and static mount evidence are merged and validated. | None for product scope. |
| The stale Surface2D branch has been extracted and removed | `surface2d-future-pressure-branch-review.md`, `git branch --all`, PR #64 metadata | E2/E5/E8 | Future-pressure report exists on `main`; `origin/surface2d-phase-16` is absent; no open PRs. | Some planning text still mentioned the old branch and is updated by this intake. |
| SpatialCanvas is an existing planned milestone | `production-tracks.md`, UI platform capability roadmap, older SpatialCanvas design | E8 | Phase 17 is named as SpatialCanvas and described as generic positioned-item surface on Surface2D. | Older design was an activation note, not a complete gate. |
| Surface2D owns coordinate/navigation facts | Surface2D design and source files | E3/E8 | `ui_controls::surface2d` descriptors and `ui_runtime::surface2d` proofs own transforms, pan/zoom, input/accessibility status, layers, diagnostics, and budget evidence. | None. |
| Current graph canvas code is graph-specific | `ui_graph_editor`, `ui_tree`, `ui_runtime`, `ui_render_data` graph-canvas paths | E3 | Existing retained graph canvas uses graph nodes, ports, edges, graph hit targets, graph actions, and graph primitive roles. | It is pressure evidence, not the generic SpatialCanvas owner. |
| `ui_surface` remains separate surface semantics | UI architecture, `ui_surface/src/lib.rs` | E3/E8 | `ui_surface` owns definition, mount, observation, session, presentation, intent, ratification, diagnostics, and validation. | SpatialCanvas must not replace it without a separate migration decision. |

## Current-state matrix

| Area | Current code reality | Current docs reality | Tests/proofs | Gap |
|---|---|---|---|---|
| Surface2D substrate | Implemented in `ui_controls`, `ui_runtime`, and `ui_static_mount`. | Completed Phase 16 design and closeout record it as reusable coordinate/navigation substrate. | Focused Surface2D tests and post-merge validation are recorded. | None for Phase 16. |
| SpatialCanvas | No `SpatialCanvas` source module exists. | Older design names generic positioned items, but lacks complete gate evidence. | No Phase 17 tests exist. | This intake records the target and blocks implementation. |
| GraphCanvas retained path | Existing code is graph-specific and references `ui_graph_editor`. | ADR 0010 separates graph truth from graph canvas/editor surface behavior. | `ui_graph_editor` tests prove graph hit testing and gesture actions. | Must not be reused as generic SpatialCanvas without redesign. |
| Viewport/camera | `editor_viewport` owns editor camera/projection/runtime settings. | UI platform roadmap says camera/projection contracts belong to viewport-projection ownership. | Editor viewport code exists outside `domain/ui`. | SpatialCanvas must not own camera or scene resources. |
| UI structural composition | `ui_composition` owns app-neutral structural layout and transactions. | UI domain ownership docs forbid product/app semantics in composition. | Composition tests exist outside Phase 17. | SpatialCanvas must not become app composition or persisted product hierarchy. |

## Owner/dependency matrix

| Responsibility | Current owner | Candidate Phase 17 owner | Dependencies | Boundary risk |
|---|---|---|---|---|
| Coordinate/navigation substrate | `ui_controls::surface2d`, `ui_runtime::surface2d`, `ui_static_mount` | Reused, not duplicated | Surface2D descriptors/reports/proof frames | Duplicating Surface2D truth. |
| SpatialCanvas package descriptor, validation, catalog, inspection | none | `ui_controls` | Existing package/catalog/inspection path and Surface2D summary evidence | Descriptor grows product/graph semantics. |
| Generic spatial item facts | none | `ui_controls` declaration; `ui_runtime` proof | Surface2D coordinate space and existing UI math types | Item facts become product objects. |
| Hit regions, hover item, selection facts, marquee facts | graph-specific today in `ui_graph_editor` | `ui_runtime` proof for generic facts only | Existing normalized pointer/focus/keyboard facts | Selection becomes product mutation. |
| Item labels, badges, adornments, diagnostics | graph-specific/current controls today | `ui_runtime` proof; `ui_controls` descriptor support | Existing render-data primitives and text proof where needed | Renderer primitive roles become semantic source truth. |
| Culling/budget evidence | partial Surface2D budget facts | `ui_runtime` proof | deterministic item count / visible window reports | Premature spatial index or renderer optimization. |
| Static proof | `ui_static_mount` | `ui_static_mount` test/proof consumer | Runtime proof frame | Static proof bypasses runtime/package evidence. |
| Retained widget node | existing `ui_tree::GraphCanvasNode` is graph-specific | no default owner; conditional only | complete design must prove need | `ui_tree` becomes generic SpatialCanvas without sufficient proof. |
| Renderer-facing primitive data | `ui_render_data` | non-owner by default | existing rect, border, stroke, glyph/clip/frame primitives | New primitive roles become renderer or graph semantics. |
| Render primitive generation | `ui_render_primitives` | non-owner | runtime-view/button primitive path | Backend primitive generation work leaks into Phase 17. |
| Input vocabulary | `ui_input` | non-owner unless missing generic fact is proven | pointer, keyboard, focus, semantic facts | Device policy enters SpatialCanvas. |
| Surface mount/session semantics | `ui_surface` | non-owner | possible future mapping only | SpatialCanvas replaces `ui_surface`. |
| App/layout composition | `ui_composition`, app/editor owners | non-owner | mounted content refs and app policies | In-canvas items become app structure. |
| Product/editor/game mutation | product/editor/game owners | forbidden owner | host intent/command paths | Generic UI mutates domain truth. |

## Relationship to Surface2D

`SpatialCanvas` builds on completed `Surface2D`; it does not rename or duplicate it.

Surface2D remains the owner of:

```text
surface identity
content bounds
viewport bounds
world/screen transforms
pan and zoom state
fit-content evidence
hover coordinate facts
selection rectangle facts
pointer capture facts
gesture cancel/commit facts
grid/background/diagnostic layer facts
large-content and budget evidence
```

SpatialCanvas may add only the generic item layer above those facts:

```text
spatial item identity
spatial item bounds
spatial hit regions
hovered item facts
generic selection membership facts
marquee selection facts
optional drag intent facts
item label/badge/adornment facts
group bounds facts
visible item window or culling facts
item-count and hit-test budget evidence
```

Surface2D must remain usable without SpatialCanvas. SpatialCanvas must report which Surface2D facts it consumes rather than re-declaring them as a second source of truth.

## Relationship to ui_surface

`ui_surface` remains the generic surface definition, mount, observation, presentation, intent, ratification, session, validation, and diagnostics boundary.

SpatialCanvas is not a replacement for `ui_surface`. A future mapping may allow a mounted UI surface to carry SpatialCanvas proof or presentation facts, but that requires a separate accepted design decision if it changes `ui_surface`.

Phase 17 must not touch `ui_surface` unless source evidence proves the accepted SpatialCanvas contract cannot be expressed through package/catalog/inspection/runtime/static proof paths.

## Relationship to ui_render_data / ui_render_primitives

`ui_render_data` already has renderer-neutral frame, surface, layer, rect, border, stroke, clip, glyph-run, viewport-embed, and graph-canvas primitive data. Surface2D proof used existing primitives without renderer backend ownership.

SpatialCanvas should start from the same rule:

```text
use existing renderer-neutral primitives for proof frames first
do not add renderer backend resources or handles
do not make render primitive roles the semantic source of item truth
```

`ui_render_data` is conditional only if implementation evidence proves the proof frame needs a generic spatial primitive role that cannot be expressed by existing primitives. `ui_render_primitives` is not an owner for Phase 17 intake; it currently generates backend-neutral primitives from runtime-view/button reports and should not become the SpatialCanvas semantic owner.

## Relationship to ui_input

`ui_input` already exposes normalized pointer, keyboard, focus, semantic, text, and device/stylus-capable facts. Phase 17 should consume those facts through `ui_runtime` where possible.

SpatialCanvas must not add device policy, raw polling, OS gesture interpretation, game input policy, or world input policy. New `ui_input` vocabulary is a stop condition unless investigation proves a missing generic normalized fact blocks the accepted contract.

## Relationship to ui_tree, if any

`ui_tree` currently contains `GraphCanvasNode`, but that node stores a `ui_graph_editor::GraphCanvasViewModel` and graph-specific runtime flags. It is not a generic SpatialCanvas implementation.

Phase 17 planning should not assume `ui_tree` ownership. The first accepted implementation path should prefer package-backed declarations, runtime proof, proof-frame projection, and static mount proof.

Touching `ui_tree` is allowed only after design proves all of these:

```text
retained runtime behavior is required for the accepted Phase 17 contract
the generic node can be named and decomposed without graph/product semantics
the public node shape has one responsibility
existing package/runtime/static proof cannot prove the contract alone
tests prevent graph, port, timeline, product, or app-composition leakage
```

## Relationship to app/editor/product/game layers

SpatialCanvas emits generic UI facts and host intent proposals. App/editor/product/game owners decide mutation.

SpatialCanvas must not own:

```text
scene resources
camera resources
render targets
graph documents
nodes, links, ports, sockets
timeline tracks, clips, scrubbers, or keyframes
drawing strokes, brushes, image assets, texture resources, or curve models
product selection mutation
editor commands
game state
workspace/app layout persistence
plugin framework or provider registration
```

## Canonical vocabulary

- `SpatialCanvas` - reusable positioned-item canvas contract built on Surface2D coordinate/navigation facts.
- `SpatialCanvasId` - stable identity for a SpatialCanvas descriptor or proof fixture.
- `SpatialItemId` - generic item identity for UI proof and inspection, not product identity.
- `SpatialItemBounds` - item rectangle or bounds in Surface2D world/content coordinates.
- `SpatialHitRegion` - generic hit region for hover/selection/drag-intent evidence.
- `SpatialItemState` - generic presentation state such as normal, hovered, selected, disabled, invalid, or hidden.
- `SpatialSelectionSet` - UI selection fact set; product selection mutation remains outside.
- `SpatialMarquee` - transient selection rectangle interpreted against generic item bounds.
- `SpatialDragIntent` - generic item drag intent fact; product commit stays outside.
- `SpatialAdornment` - label, badge, diagnostic, or overlay fact attached to a generic item.
- `SpatialVisibleWindow` - visible item/culling evidence derived from Surface2D viewport and item bounds.
- `SpatialCanvasBudgetEvidence` - deterministic item-count, hit-test, culling, proof-frame, and inspection-report budget evidence.

Forbidden public vocabulary for Phase 17:

```text
node
link
port
socket
track
clip
keyframe
stroke
brush
texture
scene
camera
entity
material
timeline
plugin
provider
```

These names belong to downstream owners or adjacent domains unless a later accepted design explicitly promotes them.

## Feature support matrix

| Capability | Status in current contract | Owner | Evidence required | Downstream owner / activation condition |
|---|---|---|---|---|
| Surface2D coordinate/navigation consumption | Required dependency | Surface2D owners; SpatialCanvas consumes | report links to Surface2D descriptor/runtime proof | Phase 16 completed |
| Spatial item identity and bounds | Proposed Phase 17 delivery | `ui_controls`, `ui_runtime` | descriptor, validation, runtime report, inspection facts | — |
| Spatial hit regions | Proposed Phase 17 delivery | `ui_runtime` | deterministic hit-test proof | — |
| Hover item facts | Proposed Phase 17 delivery | `ui_runtime` | pointer/focus report facts | — |
| Generic selection set facts | Proposed Phase 17 delivery | `ui_runtime` | proof that selection is UI fact only | Product/editor selection owner handles mutation |
| Marquee selection facts | Proposed Phase 17 delivery | `ui_runtime` | selection rectangle from Surface2D plus item-intersection evidence | — |
| Optional item drag intent | Proposed Phase 17 delivery only as intent evidence | `ui_runtime` | begin/update/end/cancel report; no commit | Product/editor command owners |
| Item labels, badges, adornments, diagnostics | Proposed Phase 17 delivery as renderer-neutral facts | `ui_controls`, `ui_runtime` | inspection/proof-frame evidence | Text/layout owners still own text semantics |
| Culling/visible item window | Proposed Phase 17 delivery as deterministic evidence | `ui_runtime` | item-count and visible-window report | `spatial_index` only if later design proves need |
| Static mount proof | Proposed Phase 17 delivery | `ui_static_mount` | proof frame consumed through static mount report | — |
| Retained widget node | Conditional, not authorized | `ui_tree` only if design accepts | retained tests and public node decomposition | Later promotion decision |
| Graph nodes, links, ports | Explicit non-owned responsibility | NodeCanvas / PortGraphCanvas | handoff evidence | Phase 18/19 |
| Timeline tracks/clips/keyframes | Explicit non-owned responsibility | TrackSurface / Timeline | handoff evidence | Phase 21 |
| Camera/projection/scene resources | Explicit non-owned responsibility | editor viewport / viewport projection / scene owners | no domain/ui imports or handles | Separate tracks |
| Product/editor/game mutation | Explicit non-owned responsibility | product/editor/game owners | no mutation counters, host intent only | Outside Phase 17 |
| Renderer backend resources | Explicit non-owned responsibility | engine renderer owners | no backend handles/resources in domain UI | Outside Phase 17 |

## Future-use-case pressure matrix

| Future use case | Needs from current contract | Must own outside current contract | Scope-leak risk | Required guard |
|---|---|---|---|---|
| NodeCanvas | generic item bounds, hit regions, selection, labels, overlays, culling | nodes, links, graph layout, graph commands | SpatialCanvas grows node/link semantics | no node/link vocabulary in Phase 17 |
| PortGraphCanvas | item bounds, hit regions, drag intent, overlays | ports, sockets, connection validation, graph truth | port semantics enter SpatialCanvas | downstream owner and tests forbid port names |
| ProgressionTreeView | item bounds and states, labels, badges, paths | progression rules, unlock costs, persistence | gameplay rules enter UI platform | generic state facts only |
| TrackSurface / Timeline | item/window proof and culling pressure | time axis, tracks, clips, scrubber, keyframes | time semantics enter SpatialCanvas | time mapping belongs to track owner |
| UI Designer canvas | item bounds, hit regions, adornments | authored UI mutation, widget hierarchy edits, persistence | authored UI truth enters SpatialCanvas | host intent proposals only |
| Drawing/Image/Curve canvases | bounds, pointer capture, drag/marquee facts | strokes, brushes, assets, texture resources, curves, handles | product document semantics enter UI | item facts only |
| SpatialCanvas / ViewportCanvas family | viewport framing, overlays, diagnostics | camera, projection, render targets, scene resources | renderer viewport authority enters UI | Surface2D stays 2D; camera owners stay outside |
| Remote/headless proof surfaces | deterministic item/fact reports | transport, sessions, host execution | proof starts host/session policy | reports only, no host effects |

## Hierarchy/composition matrix

| Hierarchy layer | Owner | Current contract may do | Current contract must not do |
|---|---|---|---|
| App/layout hierarchy | `ui_composition`, app/editor/workspace owners | Mount or host a SpatialCanvas proof/control as content | Persist app regions, panels, routes, or app recipes |
| Retained UI hierarchy | `ui_tree` / `ui_runtime` if later accepted | Hold generic UI containment only after explicit retained-node decision | Own product object hierarchy or editor document hierarchy |
| Spatial item hierarchy | Candidate `ui_controls`/`ui_runtime` facts | Record generic item/group bounds and visible-window evidence | Persist semantic product hierarchy or command history |
| Product/semantic hierarchy | product/editor/game owners | Consume SpatialCanvas facts and decide mutation | Delegate semantic truth to SpatialCanvas |
| Renderer/backend hierarchy | engine renderer owners | Consume derived `UiFrame` data | Make render resources part of SpatialCanvas truth |

## Ergonomics/usability matrix

| Actor action | Complete authoring path | Default behavior | Inspectable evidence | Failure/recovery behavior |
|---|---|---|---|---|
| Declare a SpatialCanvas | package descriptor backed by Surface2D support | require Surface2D dependency evidence and item bounds | catalog and inspection facts | descriptor diagnostic for missing Surface2D or bounds |
| Add generic items | descriptor/proof fixture item list | stable item ids, finite bounds, deterministic ordering | item-count and bounds inspection | reject duplicate ids or invalid bounds |
| Inspect hover | normalized pointer fact through Surface2D transform and hit regions | topmost eligible item reported | hover report row | empty/disabled item reports no hit |
| Select by click | generic hit region fact | selection fact changes in UI report only | selection-set report | no product mutation; host owns commit |
| Select by marquee | Surface2D selection rectangle interpreted against item bounds | deterministic intersecting item set | marquee report rows | cancel clears transient fact |
| Drag item intent | pointer capture over item | begin/update/end intent facts | drag report rows | cancel emits no commit |
| Show adornments | descriptor labels/badges/diagnostic facts | renderer-neutral overlay facts | proof frame and inspection rows | unsupported adornment kind fails validation |
| Prove large item set | deterministic fixture | visible window and item-count budget evidence | budget report | stop if proof needs speculative spatial index |

## Accessibility/input acceptance

Phase 17 must record accessibility and input acceptance before implementation promotion.

Minimum expected evidence:

```text
keyboard navigation between spatial items
keyboard selection proposal
keyboard cancel behavior
focus-visible canvas and item state
inspection-readable canvas name, item count, selected count, and focused/hovered item
pointer hover, press, capture, drag, release, and cancel behavior
wheel/trackpad navigation consumed through Surface2D evidence
reduced-motion behavior for animated item transitions if any transitions are accepted
unsupported input modes reported explicitly with downstream owner or activation condition
```

SpatialCanvas must consume existing `ui_input` facts where possible. New input vocabulary remains a stop condition unless accepted by design.

## Performance/budget evidence

Phase 17 should use deterministic budget evidence first.

Required budget rows before implementation completion:

```text
item count accepted by descriptor/proof fixture
visible item window count
hit-test candidate count
marquee intersection candidate count
adornment count
runtime report row count
proof-frame primitive count
static mount report generation count
no backend resource count
```

Wall-clock budgets are optional and must not drive speculative spatial index or renderer machinery. If a future owner proves a need for `spatial_index`, that need requires a separate accepted owner/dependency decision.

## Validation envelope

For this docs/intake PR:

```text
python tools/docs/validate_docs.py
git diff --check
```

If implementation is later authorized with the expected owner split, required validation should include:

```text
cargo test -p ui_controls spatial_canvas
cargo test -p ui_controls control_package
cargo test -p ui_runtime spatial_canvas
cargo test -p ui_static_mount spatial_canvas
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

Conditional validation:

```text
cargo test -p ui_tree              # only if retained tree source changes
cargo test -p ui_render_data       # only if renderer-neutral primitive data changes
cargo test -p ui_render_primitives # only if primitive-generation source changes; should not be required
cargo test -p ui_input             # only if input vocabulary changes
cargo test -p ui_surface           # only if surface semantics change
cargo test -p ui_composition       # only if app-neutral composition changes; should not be required
```

## Non-owned responsibilities

The following are explicitly outside Phase 17 ownership:

```text
Surface2D coordinate/navigation source truth
renderer backend resources or handles
camera/projection/scene resources
viewport render targets
product/editor/game mutation
graph nodes, links, ports, sockets, and graph commands
timeline tracks, clips, scrubbers, keyframes, and time semantics
drawing strokes, brushes, image assets, texture resources, curve handles
authored UI document mutation or UI Designer persistence
app composition, provider registration, plugin framework, or workspace recipes
foundation/meta
new crate creation without explicit architecture decision
```

## Principle compliance matrix

| Principle | Required evidence | Phase 17 intake status | Stop signal |
|---|---|---|---|
| KISS | Direct path: Surface2D facts -> SpatialCanvas item facts -> runtime proof -> static proof | Proposed owner path is direct and package-backed. | Need for app composition, renderer resources, or product commands. |
| DRY | Surface2D truth referenced, not duplicated | Surface2D remains dependency; SpatialCanvas adds only item-layer facts. | Re-declaring pan/zoom/transform ownership. |
| YAGNI | No speculative crate, renderer backend, plugin framework, or spatial index | New crate and `spatial_index` usage are stop conditions. | Adding extension points before proof requires them. |
| SOLID | Each candidate owner has one reason to change | `ui_controls` declares, `ui_runtime` proves, `ui_static_mount` verifies. | Compound file or crate owns declaration, runtime, render, and product policy. |
| Separation of Concerns | UI item facts separate from product/editor/game semantics | Non-owner list names downstream semantic owners. | Product object hierarchy or command mutation enters SpatialCanvas. |
| Avoid Premature Optimization | Deterministic budget evidence before optimization | Culling proof is fact-count/window evidence first. | Adding spatial index or renderer machinery from imagined scale. |
| Law of Demeter | Consumers use direct descriptors/reports/proof frames | Planned API uses package/catalog/inspection/report contracts. | Reaching through Surface2D internals or graph-canvas internals. |

## Module decomposition map

If Phase 17 is later promoted, a compound implementation must use a decomposed shape. A monolithic `spatialcanvas.rs` or `spatial_canvas.rs` is not accepted unless a later design proves one cohesive responsibility and names a split trigger.

| Module / file | Responsibility | Public API exported | Tests proving it | Split trigger |
|---|---|---|---|---|
| `domain/ui/ui_controls/src/spatial_canvas/mod.rs` | module wiring and re-exports | descriptor/support/id APIs | package tests compile through module root | more than re-exports |
| `domain/ui/ui_controls/src/spatial_canvas/ids.rs` | stable control kind and proof ids | `SPATIAL_CANVAS_CONTROL_KIND_ID` | package descriptor tests | additional id families beyond control/proof ids |
| `domain/ui/ui_controls/src/spatial_canvas/support.rs` | support enums and acceptance structs | input/layer/budget/item support enums | validation tests | independent validation logic appears |
| `domain/ui/ui_controls/src/spatial_canvas/descriptor.rs` | descriptor, summary, inspection facts | `ControlSpatialCanvasDescriptor`, summary/facts | catalog/inspection tests | large descriptor plus projection logic |
| `domain/ui/ui_controls/src/spatial_canvas/contribution.rs` | base control contribution | contribution helper | base-control package tests | multiple contributions |
| `domain/ui/ui_controls/src/package/spatial_canvas_validation.rs` | package validation | none outside package module | invalid descriptor tests | unrelated validators appear |
| `domain/ui/ui_controls/src/base_control/lowering/spatial_canvas_support.rs` | base-control lowering to descriptor | none outside lowering | base-control package tests | broader authoring/lowering policy appears |
| `domain/ui/ui_runtime/src/spatial_canvas/mod.rs` | module wiring and re-exports | report/proof/frame APIs | runtime tests compile through module root | more than re-exports |
| `domain/ui/ui_runtime/src/spatial_canvas/items.rs` | generic item bounds and hit-region facts | item fact structs if public | hit/selection tests | product-specific item variants appear |
| `domain/ui/ui_runtime/src/spatial_canvas/selection.rs` | hover, selection, marquee, drag-intent proof helpers | selection fact structs if public | selection/marquee tests | command commit logic appears |
| `domain/ui/ui_runtime/src/spatial_canvas/culling.rs` | visible-window and budget facts | culling report structs if public | large item budget tests | spatial index dependency needed |
| `domain/ui/ui_runtime/src/spatial_canvas/report.rs` | proof report data | `SpatialCanvasProofReport` | report row tests | frame projection enters report |
| `domain/ui/ui_runtime/src/spatial_canvas/proof.rs` | deterministic proof construction | `base_controls_spatial_canvas_report` | runtime proof tests | fixture setup grows independently |
| `domain/ui/ui_runtime/src/spatial_canvas/frame.rs` | renderer-neutral proof-frame projection | proof frame type/function | static mount and frame tests | backend-specific primitive generation appears |
| `domain/ui/ui_static_mount/tests/base_controls_spatial_canvas_static_mount.rs` | static mount proof test | test only | static mount command | multiple fixture families appear |

## Non-negotiable rules

- `SpatialCanvas` consumes `Surface2D`; it does not duplicate it.
- Generic item facts must stay generic and renderer-neutral.
- Selection facts are UI facts until a host/product owner commits mutation.
- Existing graph-canvas code is evidence, not a generic owner.
- `ui_tree` is conditional, not assumed.
- `ui_render_data`, `ui_render_primitives`, `ui_input`, `ui_surface`, and `ui_composition` are conditional/non-owners unless accepted source evidence proves otherwise.
- No new crate creation without explicit architecture decision.
- No product/editor/game mutation.
- No renderer backend ownership.
- No plugin framework or `foundation/meta`.

## Stop conditions

Stop and redesign if Phase 17 would require:

```text
product/editor/game mutation
renderer backend resources or handles
app-composition/provider/plugin-framework implementation
foundation/meta
new crate creation without explicit architecture decision
graph/node/port/timeline/drawing/product-specific semantics
Surface2D public API mutation without compatibility/design decision
bypassing package/catalog/inspection projection
static proof that bypasses runtime proof evidence
changing ui_surface, ui_tree, ui_render_data, ui_render_primitives, ui_input, or ui_composition without source evidence that the accepted contract cannot be expressed otherwise
using ui_graph_editor graph actions or graph hit targets as the generic SpatialCanvas API
using spatial_index or domain/spatial as an optimization before deterministic budget evidence proves the need
```

## Next planning action

Review this design with the source investigation report.

If accepted, the next planning update may promote Phase 17 from `active-planning` to `active-implementation` only after it records:

```text
exact owner files/crates
allowed files/crates
forbidden files/crates
public API names
implementation sequence
module decomposition map
principle compliance status
validation envelope
evidence expectation
stop conditions
```

Until that promotion happens, Phase 17 remains planning/design only.
