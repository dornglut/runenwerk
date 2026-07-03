---
title: UI Component Platform Surface2D Design
description: Complete investigation and design intake for the generic 2D coordinate, navigation, transform, bounds, overlay, input, and large-content primitive for reusable UI surfaces.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-03
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../workspace/workflow-lifecycle.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
---

# UI Component Platform Surface2D Design

## Status

This is the active Phase 16 investigation and design-intake document for `PT-UI-COMPONENT-PLATFORM-016` Surface2D.

Lifecycle state: `active-planning`.

Implementation is not authorized by this document. Implementation requires a planning promotion that records exact owner files, exact implementation contract, validation envelope, evidence expectation, stop conditions, and merge-readiness expectations.

`Surface2D` is a reusable renderer-neutral coordinate and navigation substrate. It is not Gallery-specific, not GraphCanvas-specific, not a product editor command system, not app composition, and not a renderer backend.

Typed App Composition documents remain proposed architecture references only. They do not authorize plugin framework work, shared extraction, `foundation/meta`, or app-composition implementation in Phase 16.

## Complete investigation gate status

Status: active, partially satisfied by connector evidence and planning authority. Code-level current reality still needs focused source inspection before implementation authorization.

Evidence classes used:

```text
E2 connector file inspection
E8 accepted workspace/planning/design authority
E2 PR metadata inspection for prior Phase 15/16 planning and UI split PRs
```

Current evidence inspected:

```text
AGENTS.md and workflow authority through merged PR #59
DOMAIN_MAP.md
CRATES.md
docs-site/src/content/docs/domain/ui/architecture.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md
PR #56 Phase 15 closeout / Phase 16 planning hardening
PR #57 UI domain split before Phase 16
```

Current investigation blockers before implementation:

```text
inspect current source paths in ui_controls, ui_runtime, ui_static_mount, ui_render_data, ui_render_primitives, ui_input, and ui_surface
record exact implementation files
confirm whether new renderer-neutral primitive data is needed
confirm current package/catalog/inspection extension points after Phase 15 and PR #57 splits
confirm current ui_surface symbols and whether Surface2D needs no direct code dependency on them
confirm focused test names and fixture placement
```

## Complete investigation dossier

### Question

What complete renderer-neutral `Surface2D` contract should Phase 16 deliver, which UI crates own each part, and what evidence must exist before implementation starts?

### Current lifecycle

```text
PT-UI-COMPONENT-PLATFORM-016
Lifecycle state: active-planning
Implementation authorization: no
Current branch purpose: investigation/design intake
```

### Authority/source matrix

| Claim | Source inspected | Evidence class | Evidence found | Conflict / drift |
|---|---|---:|---|---|
| Phase 16 is current active planning | `workspace/planning/active-work.md` | E8 | Active focus is `PT-UI-COMPONENT-PLATFORM-016` Surface2D. | None. |
| Phase 16 is not implementation-authorized | `active-work.md`, this design | E8 | Planning requires exact owners, scope, validation, evidence, and stop conditions before implementation. | None. |
| `Surface2D` must not own product/editor/game truth | existing design, UI architecture | E8 | Host/product/editor/game mutation remains outside the reusable UI substrate. | None. |
| `ui_surface` is existing semantic/compatibility vocabulary | `DOMAIN_MAP.md`, UI architecture | E8 | `ui_surface` owns UI surface semantics and remains live where supersession is incomplete. | Surface2D relationship must be explicit. |
| No Surface2D implementation exists from Phase 15/PR #57 | PR #49/#57 metadata | E2 | Prior PRs explicitly excluded Phase 16 implementation. | Source inspection still required. |
| Typed App Composition is only reference direction | existing design/planning | E8 | It does not authorize plugin framework work in Phase 16. | None. |

### Current-state matrix

| Area | Current repository reality | Current docs/planning reality | Evidence | Gap before implementation |
|---|---|---|---|---|
| Active planning | Phase 16 active planning is recorded. | Active work, roadmap, production track all identify Surface2D. | E8 | Update active work with new gate fields after intake. |
| Existing code | Not fully source-inspected in this intake pass. | Planning says no implementation is authorized. | E2/E8 | Inspect exact crate files before implementation. |
| `ui_surface` relationship | UI architecture says `ui_surface` remains temporary compatibility input and no new independent mutation path may be added. | Existing design requires relationship decision before implementation. | E8 | Adopt Phase 16 relationship decision below. |
| Prior validation | Phase 15 and PR #57 validation are recorded in planning/PR metadata. | Phase 16 planning validation remains docs/diff. | E2/E8 | Run current branch docs validation locally. |
| Future consumers | Future canvases and graph/timeline surfaces are named consumers. | Existing design lists future consumers. | E8 | Convert future consumers into downstream owner matrix. |

### Owner/dependency matrix

| Responsibility | Phase 16 owner | Dependencies | Boundary risk | Guard |
|---|---|---|---|---|
| Package-backed declarations and validation | `ui_controls` | existing package/catalog/inspection contracts | Descriptor grows product semantics | Surface2D descriptors contain coordinate/navigation facts only. |
| Catalog/inspection projection | `ui_controls` | package descriptor and inspection projection paths | Gallery semantics leak into Surface2D | Inspection reports generic surface facts only. |
| Runtime navigation/fact replay/report | `ui_runtime` | retained runtime, input facts, proof reports | Runtime mutates product/editor truth | Runtime emits facts/intents only. |
| Static mount proof | `ui_static_mount` | package-backed declaration and renderer-neutral proof frame | Static proof bypasses catalog/inspection | Static proof consumes the same package contract. |
| Renderer-neutral frame data | `ui_render_data` | existing UiFrame/primitive contracts | Backend resources leak into domain/ui | Only data facts; no backend handles. |
| Renderer primitive contracts | `ui_render_primitives` only if needed | primitive data contracts | Primitive expansion becomes renderer feature work | Add only renderer-neutral primitives required by proof. |
| Input vocabulary | `ui_input` only if needed | pointer/wheel/keyboard facts | Surface2D owns device policy | Reuse existing normalized input where possible. |
| `ui_surface` semantics | Existing `ui_surface` owner | domain UI surface semantics | Surface2D renames/replaces ui_surface accidentally | Relationship decision below forbids Phase 16 replacement. |
| Product/editor/game mutation | host/editor/game owners | host intent/command paths | Surface2D becomes editor command system | Explicit non-owned responsibility. |

### Vocabulary decision

Phase 16 adopts this relationship to existing `ui_surface` vocabulary:

```text
Surface2D is lower-level renderer-neutral coordinate/navigation vocabulary.
It may later be consumed or mapped by ui_surface, but Phase 16 does not rename, replace, remove, or absorb ui_surface contracts.
```

This selects the safe form of option A from the prior planning gate: `Surface2D` is below `ui_surface`, not a competing semantic surface authority.

### Capability inventory

| Capability | Phase 16 delivery status | Evidence required | Owner |
|---|---|---|---|
| Surface identity | Deliver | descriptor, runtime report, static proof | `ui_controls`, `ui_runtime`, `ui_static_mount` |
| Content bounds | Deliver | descriptor validation and runtime proof | `ui_controls`, `ui_runtime` |
| Viewport bounds | Deliver | descriptor/runtime/static mount proof | `ui_controls`, `ui_runtime`, `ui_static_mount` |
| World/screen transforms | Deliver | bidirectional transform proof and invalid diagnostic | `ui_runtime` |
| Pan/zoom state | Deliver | input/replay/report proof | `ui_runtime` |
| Fit-content request/evidence | Deliver | request and resolved evidence report | `ui_controls`, `ui_runtime` |
| Hover coordinate fact | Deliver | normalized coordinate proof | `ui_runtime` |
| Selection rectangle fact | Deliver | transient fact proof, no product mutation | `ui_runtime` |
| Pointer capture fact | Deliver | capture/cancel/commit proof | `ui_runtime` |
| Gesture cancel/commit | Deliver | proof report and diagnostics | `ui_runtime` |
| Grid/background fact | Deliver | renderer-neutral layer fact | `ui_controls`, `ui_runtime` |
| Diagnostic overlay fact | Deliver | report/frame evidence | `ui_runtime` |
| Large-content bounds fact | Deliver | deterministic large-bounds fixture | `ui_runtime` |
| Budget evidence fact | Deliver | operation/fact-count or p95 report | `ui_runtime` |
| Graph nodes/links | Named downstream | Graph/NodeCanvas design | future Phase 18/19 owners |
| Timeline tracks | Named downstream | TrackSurface/Timeline design | future Phase 21 owner |
| Product selection mutation | Explicit non-owned responsibility | boundary tests/proof no mutation | host/editor/game owners |
| Renderer backend resources | Explicit non-owned responsibility | absence of backend handles in domain/ui | engine renderer owner |

### Alternatives and tradeoffs

| Option | Benefits | Costs | Boundary impact | Long-term fit | Decision |
|---|---|---|---|---|---|
| Surface2D below `ui_surface` | Preserves existing semantic surface vocabulary and gives reusable coordinate substrate. | Requires later mapping if `ui_surface` consumes it. | Low boundary risk. | Strong. | Accepted for Phase 16. |
| Surface2D replaces part of `ui_surface` now | Removes compatibility sooner. | Requires migration proof and risks semantic churn. | High boundary risk. | Not appropriate for Phase 16 intake. | Rejected for Phase 16. |
| Surface2D stays UI Component Platform-local forever | Avoids current semantic conflict. | Blocks later reuse by surface semantics. | Medium future duplication risk. | Weak as permanent policy. | Rejected as final direction. |
| Build specialized GraphCanvas first | Directly serves future graph needs. | Skips reusable coordinate/navigation substrate. | High product-semantic leak risk. | Weak. | Rejected. |

### Confidence matrix

| Finding | Confidence | Reason | Missing evidence to improve confidence |
|---|---|---|---|
| Phase 16 should remain active planning, not implementation | High | Planning/design authority agrees. | None. |
| Surface2D should sit below `ui_surface` for Phase 16 | High | UI architecture preserves `ui_surface` semantics while design needs coordinate substrate. | Source inspection of current `ui_surface` symbols. |
| Owner split can use existing crates | Medium | Planning points to existing owner crates. | Source inspection of extension points after PR #57 splits. |
| `ui_render_data` changes may not be required | Medium | Existing UiFrame/primitive data may carry proof facts. | Inspect existing proof-frame and primitive contracts. |
| No Surface2D implementation exists | Medium | PR metadata excludes it. | Code search/source inspection. |

## Complete design gate status

Status: active design intake, not active implementation.

Phase 16 design acceptance requires resolving the remaining source-inspection blockers and then promoting planning with exact files and validation commands.

## Decision

`Surface2D` is the generic 2D coordinate/navigation primitive underneath future reusable surfaces.

It owns reusable surface facts and intent vocabulary for:

```text
surface identity
content bounds
viewport bounds
world/screen transforms
pan and zoom state
fit requests
hover coordinate facts
selection rectangle facts
pointer capture facts
gesture cancel/commit facts
overlay and diagnostic layer facts
grid/background facts
large-content and LOD-readiness facts
budget evidence
```

It does not own product truth, graph truth, timeline truth, editor commands, renderer resources, authored UI mutation, or app recipe composition.

## Complete owner split

Phase 16 implementation must use this owner split unless source inspection proves a different split is required and updates this design before implementation.

```text
ui_controls:
  package-backed Surface2D declarations, descriptors, validation reasons,
  catalog projection, and inspection facts.

ui_runtime:
  runtime-local Surface2D state projection, input normalization consumption,
  pan/zoom/fit/hover/selection/capture intent evidence, proof report,
  and renderer-neutral proof-frame projection.

ui_static_mount:
  static proof that Surface2D declarations lower to mountable renderer-neutral evidence
  without bypassing package/catalog/inspection contracts.

ui_render_data:
  renderer-neutral frame data only when existing proof-frame data cannot carry
  the delivered Surface2D evidence.

ui_render_primitives:
  renderer-neutral primitive contracts only when existing primitives cannot express
  delivered grid/background/diagnostic facts.

ui_input:
  no new ownership unless existing normalized input lacks a required generic fact.

ui_surface:
  remains existing semantic surface vocabulary. Phase 16 does not rename, replace,
  remove, or absorb ui_surface contracts.

host/product/editor/game layers:
  product commands, graph edits, timeline edits, selection mutation,
  persistence, project data, renderer resources, and external effects.
```

If the owner split would require a new crate, record that as an explicit planning decision before implementation. Do not create a new crate as an incidental implementation detail.

## Canonical vocabulary

- `Surface2D` - generic renderer-neutral 2D coordinate/navigation surface contract.
- `Surface2DId` - stable identity for a reusable surface instance or proof fixture.
- `Surface2DViewport` - visible screen-space or local-frame bounds.
- `Surface2DContentBounds` - renderer-neutral world/content bounds.
- `Surface2DTransform` - world-to-screen and screen-to-world mapping evidence.
- `Surface2DNavigationState` - pan/zoom/fit state facts.
- `Surface2DInteractionIntent` - normalized intent emitted by reusable surface behavior.
- `Surface2DGestureState` - pointer capture, drag, cancel, commit, and active gesture facts.
- `Surface2DSelectionBox` - transient rectangle evidence, not product selection mutation.
- `Surface2DOverlayLayer` - diagnostic/grid/background/adornment layer facts.
- `Surface2DBudgetEvidence` - explicit evidence for large bounds, LOD readiness, and report/runtime budgets.

## Non-negotiable rules

- General coordinate/navigation contracts come before specialized canvases.
- Story/proof evidence comes before mount eligibility.
- `Surface2D` must not collapse into Gallery, Workbench, UI Designer, GraphCanvas, Timeline, or a product editor.
- `Surface2D` emits facts and host intents only; it does not mutate product/editor/game truth.
- Graph semantics stay out of `Surface2D`.
- Timeline semantics stay out of `Surface2D`.
- Renderer resources and backend handles stay out of `Surface2D`.
- Host/app/editor/game mutation remains outside `domain/ui` through explicit host intent or command paths.
- UI Story owns proof orchestration only.
- Gallery, Workbench, and UI Designer consume platform contracts; they do not own reusable surface semantics.
- Renderer output remains backend-neutral and must not become UI source truth.
- Typed App Composition remains proposed reference direction only unless accepted separately.

## Phase 16 delivered contract

Phase 16 must deliver the reusable substrate contract:

```text
surface id
content bounds
viewport bounds
world-to-screen transform
screen-to-world transform
pan state
zoom state
fit-content request/evidence
hover coordinate fact
selection rectangle fact
pointer capture fact
gesture cancel/commit fact
grid/background fact
diagnostic overlay fact
large-content bounds fact
budget evidence fact
invalid transform expected-failure diagnostic
```

The Phase 16 proof must be visible through package/catalog/inspection projection, runtime proof/report evidence, and static mount proof.

## Feature support matrix

| Capability | Status in Phase 16 | Owner | Evidence required | Downstream owner / activation condition |
|---|---|---|---|---|
| Coordinate and navigation substrate | Delivered | `ui_controls`, `ui_runtime`, `ui_static_mount` | descriptors, runtime report, static mount proof | — |
| Grid/background/diagnostic layers | Delivered as renderer-neutral facts | `ui_runtime`, `ui_render_data` if needed | proof-frame/report evidence | — |
| Large-content and budget evidence | Delivered as deterministic fact/report | `ui_runtime` | deterministic fixture and report | — |
| Accessibility/input acceptance | Delivered as descriptor/report facts | `ui_controls`, `ui_runtime` | keyboard/pointer/focus inspection facts | — |
| Specialized spatial item layout | Named downstream contract | future `SpatialCanvas` owner | handoff from Surface2D coordinates | Phase 17 |
| Nodes/links/ports | Named downstream contract | future `NodeCanvas` / `PortGraphCanvas` owner | graph-specific design | Phase 18/19 |
| Timeline tracks | Named downstream contract | future `TrackSurface` / `Timeline` owner | timeline-specific design | Phase 21 |
| Curve/material/SDF/gameplay graphs | Named downstream contracts | future graph owners | domain-specific graph designs | Future phases |
| Product/editor/game mutation | Explicit non-owned responsibility | host/editor/game owners | boundary proof/no mutation | Outside Phase 16 |
| Renderer backend resources | Explicit non-owned responsibility | engine renderer owner | no backend handles in domain/ui | Outside Phase 16 |
| App recipe/plugin framework | Explicit non-owned responsibility | typed app composition track | accepted separate design | Outside Phase 16 |

## Future-use-case pressure matrix

| Future use case | Needs from Surface2D | Must own outside Surface2D | Scope-leak risk | Required guard |
|---|---|---|---|---|
| SpatialCanvas | coordinate transform, pan/zoom, large bounds | spatial item semantics and selection mutation | Surface2D starts owning scene/editor truth | facts/intents only |
| NodeCanvas | transform, selection rectangle, hover coordinate, pointer capture | graph nodes/links/ports and commands | graph semantics leak into Surface2D | no node/link vocabulary in Surface2D |
| PortGraphCanvas | transform, pointer capture, overlays | port/socket semantics and graph validation | port semantics become UI substrate | downstream graph owner |
| Timeline | pan/zoom, fit, selection rectangle | time units, tracks, clips, playback commands | timeline semantics leak into Surface2D | no time/track vocabulary |
| UI Designer canvas | coordinates, overlays, diagnostics | authored UI mutation and persistence | authored editor changes enter domain/ui | host intent boundary |
| Material/SDF/gameplay graph views | navigation and coordinate proof | graph/domain semantics | engine/product semantics leak | graph-specific downstream design |

## Hierarchy and composition matrix

| Hierarchy layer | Owner | Phase 16 may do | Phase 16 must not do |
|---|---|---|---|
| App/layout hierarchy | app/editor/workspace owners | receive reusable Surface2D facts through host intent/report paths | mutate workspace, tab, panel, provider, or app recipe state |
| Runtime/retained UI hierarchy | `ui_runtime` / `ui_tree` | project renderer-neutral proof facts and handle retained runtime evidence | persist app/product semantic hierarchy |
| Package/control hierarchy | `ui_controls` | declare reusable Surface2D descriptors and inspection facts | encode Gallery/Graph/Timeline semantics |
| Product/semantic hierarchy | product/editor/game owners | consume Surface2D facts downstream | delegate semantic ownership to Surface2D |
| Renderer/backend hierarchy | engine renderer owners | consume UiFrame/primitive data | make renderer resources part of Surface2D truth |

## Ergonomics and usability matrix

| Actor action | Complete authoring path | Default behavior | Inspectable evidence | Failure/recovery behavior |
|---|---|---|---|---|
| Declare a surface | package descriptor / fixture declaration | valid identity, viewport, content bounds, transform defaults | catalog and inspection facts | validation reason for missing/invalid bounds |
| Navigate by pan | normalized pointer/keyboard intent into runtime state | bounded pan report | runtime report and proof frame | cancel/commit evidence |
| Navigate by zoom | wheel/keyboard/gesture intent into runtime state | centered zoom with deterministic clamp | runtime report | invalid transform diagnostic |
| Fit content | fit request descriptor/runtime intent | fit full content bounds into viewport | fit evidence report | diagnostic for empty/invalid content |
| Inspect hover coordinate | pointer fact through transform | screen-to-world coordinate emitted | inspection/runtime report | no coordinate if transform invalid |
| Draw selection box | pointer capture drag facts | transient rectangle fact | runtime report/proof frame | cancel clears transient fact |
| Display diagnostics | descriptor/runtime diagnostics | diagnostic layer facts | proof-frame/report evidence | expected-failure diagnostic remains visible |

## Accessibility and input acceptance

Phase 16 must define and prove acceptance for:

```text
keyboard pan
keyboard zoom
keyboard fit-content
focus-visible surface state
screen-reader/inspection-readable surface name and bounds
reduced-motion behavior for animated navigation
pointer capture cancellation
wheel and high-resolution scroll input
trackpad pinch status explicitly reported
touch pan/zoom status explicitly reported
controller navigation status explicitly reported
```

If an input mode is not delivered by the Phase 16 implementation contract, the descriptor/report must say so explicitly with a named downstream owner or activation condition.

## Performance and budget evidence

Phase 16 must add a budget-evidence shape before implementation. Exact numbers may be conservative, but the evidence model must cover:

```text
transform projection cost
pan/zoom update cost
hover coordinate update cost
selection rectangle update cost
fit-content calculation cost
large-content bounds projection cost
runtime report generation cost
static mount report generation cost
primitive count or fact count budget
```

Initial budgets should be recorded as p95 targets for deterministic fixtures where practical. If wall-clock budgets are not useful yet, use deterministic operation/fact-count budgets and record why.

## Validation envelope

Planning validation:

```text
python tools/docs/validate_docs.py
git diff --check
```

Implementation validation must be set before moving to `active-implementation`. Expected shape:

```text
cargo test -p ui_controls surface2d
cargo test -p ui_runtime surface2d
cargo test -p ui_static_mount surface2d
cargo test -p ui_render_data    # only if renderer-neutral primitive data changes
cargo test -p ui_render_primitives # only if primitive contracts change
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

The exact package list must match the accepted owner split and exact source files selected by the implementation planning gate.

## Non-owned responsibilities

The following responsibilities are explicitly outside Phase 16 ownership, but each has a named owner or downstream activation path:

```text
graph semantics -> future NodeCanvas / PortGraphCanvas phases
timeline semantics -> future TrackSurface / Timeline phase
Gallery catalog semantics -> Gallery/product owner; may consume Surface2D inspection facts
editor commands -> editor command/session owners
renderer resources -> engine renderer owner
product/editor/game mutation -> product/editor/game owners
authored UI editing -> UI Designer/editor definition owners
full app composition -> Typed App Composition track after accepted implementation plan
plugin framework implementation -> separate accepted track
foundation/meta -> separate accepted architecture decision
shared plugin primitives -> separate accepted architecture decision
```

## Stop conditions

Stop and redesign if implementation requires:

```text
Surface2D mutating product/editor/game truth
Surface2D owning graph or timeline semantics
Surface2D owning renderer backend resources
Surface2D bypassing package/catalog/inspection projection
Surface2D replacing ui_surface vocabulary without an accepted migration decision
Surface2D depending on Typed App Composition as implementation authority
shared plugin framework extraction
foundation/meta
host command execution inside domain/ui
new crate creation without explicit planning decision
```

## Next planning action

Before active implementation, complete a source-inspection pass that records:

```text
exact ui_controls files
exact ui_runtime files
exact ui_static_mount files
whether ui_render_data changes are needed
whether ui_render_primitives changes are needed
whether ui_input changes are needed
current ui_surface symbols and proof that Phase 16 does not conflict with them
focused test names
fixture/proof names
exact implementation stop conditions
```
