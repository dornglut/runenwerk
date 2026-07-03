---
title: UI Component Platform Surface2D Design
description: Active-folder design reference with completed Phase 16 evidence for the generic 2D coordinate, navigation, transform, bounds, overlay, input, and large-content primitive for reusable UI surfaces.
status: active
lifecycle_exception: active_phase_evidence
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
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../reports/investigations/phase-16-surface2d-source-investigation.md
  - ../../reports/investigations/surface2d-future-pressure-branch-review.md
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

This is the completed Phase 16 design reference for `PT-UI-COMPONENT-PLATFORM-016` Surface2D.

Lifecycle state: `completed`.

Implementation completed through PR #61 after PR #62 merged the workflow/principle hardening that supported the final review. The detailed closeout record is `docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md`.

`Surface2D` is a reusable renderer-neutral coordinate and navigation substrate. It is not Gallery-specific, not GraphCanvas-specific, not a product editor command system, not app composition, and not a renderer backend.

Typed App Composition documents remain proposed architecture references only. They did not authorize plugin framework work, shared extraction, `foundation/meta`, or app-composition implementation in Phase 16.

## Complete investigation gate status

Status: complete. The source investigation, design gate, implementation PR, post-merge validation, and closeout record are aligned.

Evidence classes used:

```text
E3 current source/history inspection
E5 post-merge local validation from main
E8 accepted workspace/planning/design authority
E9 closeout alignment across source, validation, and planning truth
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
docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md
PR #56 Phase 15 closeout / Phase 16 planning hardening
PR #57 UI domain split before Phase 16
PR #62 docs-only workflow/principle/decomposition hardening
PR #61 Phase 16 Surface2D implementation
```

Resolved source-investigation outputs:

```text
exact current source paths inspected for ui_controls, ui_runtime, ui_static_mount, ui_render_data, ui_render_primitives, ui_input, and ui_surface
exact implementation files recorded in the source investigation report
new renderer-neutral primitive data is not required for the accepted contract unless implementation evidence proves otherwise
package/catalog/inspection extension points are confirmed after Phase 15 and PR #57 splits
ui_surface remains semantic surface vocabulary and Surface2D has no direct Phase 16 dependency on it
focused test names and fixture/proof placement are recorded
```

Completion evidence:

```text
PR #62 merged at 6cfb82b81aa5478496ff6cbf3fa2eea607777aaf
PR #61 squash-merged at 2e803620c91726fb599c5e5c4eee4b3984cd4a9d
post-merge validation from main passed
Phase 16 closeout recorded
```

## Complete investigation dossier

### Question

What renderer-neutral `Surface2D` contract did Phase 16 deliver, which UI crates own each part, and what evidence closed the phase?

### Current lifecycle

```text
PT-UI-COMPONENT-PLATFORM-016
Lifecycle state: completed
Implementation authorization: completed through PR #61
Current branch purpose: historical design reference
```

### Authority/source matrix

| Claim | Source inspected | Evidence class | Evidence found | Conflict / drift |
|---|---|---:|---|---|
| Phase 16 is completed | `workspace/planning/completed-work.md`, closeout report | E8/E9 | Completed through PR #62 and PR #61, with post-merge validation recorded. | None. |
| No active implementation is authorized by closeout | `active-work.md`, closeout report | E8 | Active work records no current implementation focus after closeout. | None. |
| `Surface2D` must not own product/editor/game truth | existing design, UI architecture | E8 | Host/product/editor/game mutation remains outside the reusable UI substrate. | None. |
| `ui_surface` is existing semantic vocabulary | `DOMAIN_MAP.md`, UI architecture, source investigation | E2/E8 | `ui_surface` owns semantic surface modules and is not a Phase 16 implementation owner. | None. |
| Surface2D implementation exists on `main` | PR #61 metadata and source closeout | E3/E5/E9 | Phase 16 implementation is merged and validated post-merge. | None. |
| Typed App Composition is only reference direction | existing design/planning | E8 | It does not authorize plugin framework work in Phase 16. | None. |

### Current-state matrix

| Area | Current repository reality | Current docs/planning reality | Evidence | Closeout status |
|---|---|---|---|---|
| Active planning | No active implementation focus is selected after closeout. | Active work points to next planning intake only. | E8 | Closed. |
| Existing code | Surface2D implementation is merged on `main`. | Completed work, roadmap, production track, and closeout identify Phase 16 as completed. | E3/E5/E9 | Closed. |
| `ui_surface` relationship | `ui_surface` owns semantic surface modules and remains separate. | Surface2D sits below `ui_surface` as coordinate/navigation vocabulary. | E2/E8 | None for Phase 16. |
| Prior validation | Post-merge Phase 16 validation from `main` is recorded. | Closeout records focused Surface2D, workspace, docs, and diff validation. | E5/E9 | Closed. |
| Future consumers | Downstream canvases and graph/timeline surfaces are named consumers. | This design records downstream owner paths. | E8 | Implementation must not import their semantics into Surface2D. |

### Owner/dependency matrix

| Responsibility | Phase 16 owner | Dependencies | Boundary risk | Guard |
|---|---|---|---|---|
| Package-backed declarations and validation | `ui_controls` | existing package/catalog/inspection contracts | Descriptor grows product semantics | Surface2D descriptors contain coordinate/navigation facts only. |
| Catalog/inspection projection | `ui_controls` | package descriptor and inspection projection paths | Gallery semantics leak into Surface2D | Inspection reports generic surface facts only. |
| Runtime navigation/fact replay/report | `ui_runtime` | retained runtime, input facts, proof reports | Runtime mutates product/editor truth | Runtime emits facts/intents only. |
| Static mount proof | `ui_static_mount` | package-backed declaration and renderer-neutral proof frame | Static proof bypasses catalog/inspection | Static proof consumes the same package contract. |
| Renderer-neutral frame data | conditional `ui_render_data` | existing UiFrame/primitive contracts | Backend resources leak into domain/ui | Existing primitives are sufficient unless implementation evidence proves otherwise. |
| Renderer primitive contracts | conditional `ui_render_primitives` | primitive data contracts | Primitive expansion becomes renderer feature work | Do not touch for the accepted contract. |
| Input vocabulary | conditional `ui_input` | pointer/wheel/keyboard facts | Surface2D owns device policy | Reuse existing normalized input where possible. |
| `ui_surface` semantics | Existing `ui_surface` owner | domain UI surface semantics | Surface2D renames/replaces ui_surface accidentally | Phase 16 has no direct dependency on `ui_surface`. |
| Product/editor/game mutation | host/editor/game owners | host intent/command paths | Surface2D becomes editor command system | Explicit non-owned responsibility. |

### Vocabulary decision

Phase 16 adopts this relationship to existing `ui_surface` vocabulary:

```text
Surface2D is lower-level renderer-neutral coordinate/navigation vocabulary.
It may be consumed or mapped by ui_surface through a separate accepted contract, but Phase 16 does not rename, replace, remove, or absorb ui_surface contracts.
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
| Graph nodes/links | Named downstream contract | Graph/NodeCanvas design | Phase 18/19 owners |
| Timeline tracks | Named downstream contract | TrackSurface/Timeline design | Phase 21 owner |
| Product selection mutation | Explicit non-owned responsibility | boundary tests/proof no mutation | host/editor/game owners |
| Renderer backend resources | Explicit non-owned responsibility | absence of backend handles in domain/ui | engine renderer owner |

### Alternatives and tradeoffs

| Option | Benefits | Costs | Boundary impact | Long-term fit | Decision |
|---|---|---|---|---|---|
| Surface2D below `ui_surface` | Preserves existing semantic surface vocabulary and gives reusable coordinate substrate. | Requires a separate mapping contract if `ui_surface` consumes it. | Low boundary risk. | Strong. | Accepted for Phase 16. |
| Surface2D replaces part of `ui_surface` now | Removes compatibility sooner. | Requires migration proof and risks semantic churn. | High boundary risk. | Not appropriate for Phase 16 intake. | Rejected for Phase 16. |
| Surface2D stays UI Component Platform-local permanently | Avoids current semantic conflict. | Blocks reuse by surface semantics. | Medium downstream duplication risk. | Weak. | Rejected as final direction. |
| Build specialized GraphCanvas first | Directly serves graph needs. | Skips reusable coordinate/navigation substrate. | High product-semantic leak risk. | Weak. | Rejected. |

### Confidence matrix

| Finding | Confidence | Reason | Closeout evidence |
|---|---|---|---|
| Phase 16 is completed | High | PR #62 and PR #61 are merged, and post-merge validation from `main` passed. | Closeout report. |
| Surface2D should sit below `ui_surface` for Phase 16 | High | Source investigation confirms current `ui_surface` semantic modules and no Phase 16 dependency. | None for Phase 16. |
| Owner split used existing crates | High | Implementation stayed in `ui_controls`, `ui_runtime`, and `ui_static_mount` for delivered owners. | PR #61 and closeout report. |
| `ui_render_data` changes were not required by the accepted contract | High | Existing frame/primitive evidence carried the proof without renderer backend ownership. | PR #61 and closeout report. |

## Complete design gate status

Status: completed.

Phase 16 implementation completed through PR #61, and the closeout report records final validation, branch cleanup, principle compliance, and decomposition status.

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

Phase 16 implementation used the owner split recorded in `docs-site/src/content/docs/reports/investigations/phase-16-surface2d-source-investigation.md` and finalized in `docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md`.

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
  no implementation ownership for the accepted contract unless existing proof-frame data cannot carry
  the delivered Surface2D evidence.

ui_render_primitives:
  no implementation ownership for the accepted contract.

ui_input:
  no implementation ownership for the accepted contract unless existing normalized input lacks a required generic fact.

ui_surface:
  remains existing semantic surface vocabulary. Phase 16 does not rename, replace,
  remove, or absorb ui_surface contracts.

host/product/editor/game layers:
  product commands, graph edits, timeline edits, selection mutation,
  persistence, project data, renderer resources, and external effects.
```

Future changes that require a new crate must record that as an explicit planning decision before implementation. Do not create a new crate as an incidental implementation detail.

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

Phase 16 delivered the reusable substrate contract:

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

The Phase 16 proof is visible through package/catalog/inspection projection, runtime proof/report evidence, and static mount proof.

## Feature support matrix

| Capability | Status in Phase 16 | Owner | Evidence required | Downstream owner / activation condition |
|---|---|---|---|---|
| Coordinate and navigation substrate | Delivered | `ui_controls`, `ui_runtime`, `ui_static_mount` | descriptors, runtime report, static mount proof | — |
| Grid/background/diagnostic layers | Delivered as renderer-neutral facts | `ui_runtime`; existing `ui_render_data` primitives | proof-frame/report evidence | — |
| Large-content and budget evidence | Delivered as deterministic fact/report | `ui_runtime` | deterministic fixture and report | — |
| Accessibility/input acceptance | Delivered as descriptor/report facts | `ui_controls`, `ui_runtime` | keyboard/pointer/focus inspection facts | — |
| Specialized spatial item layout | Named downstream contract | `SpatialCanvas` owner | handoff from Surface2D coordinates | Phase 17 |
| Nodes/links/ports | Named downstream contract | `NodeCanvas` / `PortGraphCanvas` owner | graph-specific design | Phase 18/19 |
| Timeline tracks | Named downstream contract | `TrackSurface` / `Timeline` owner | timeline-specific design | Phase 21 |
| Curve/material/SDF/gameplay graphs | Named downstream contracts | graph owners | domain-specific graph designs | named downstream graph phase |
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

Phase 16 defined and proved acceptance for:

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

If a future input mode is not delivered by an accepted implementation contract, the descriptor/report must say so explicitly with a named downstream owner or activation condition.

## Performance and budget evidence

Phase 16 added a budget-evidence shape. Exact numbers may be conservative, but the evidence model covers:

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

Future budget expansion should use p95 targets for deterministic fixtures where practical. If wall-clock budgets are not useful, use deterministic operation/fact-count budgets and record why.

## Validation envelope

Post-merge validation from `main`:

```text
cargo test -p ui_controls surface2d
cargo test -p ui_controls control_package
cargo test -p ui_runtime surface2d
cargo test -p ui_static_mount surface2d
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

Conditional validation rule for future changes:

```text
cargo test -p ui_render_data        # only if ui_render_data changes
cargo test -p ui_render_primitives  # only if ui_render_primitives changes
cargo test -p ui_input              # only if ui_input changes
cargo test -p ui_surface            # only if ui_surface changes
```

The exact package list for future changes must match the accepted owner split and exact source files selected by that future implementation planning gate.

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

Future changes must stop and redesign if implementation requires:

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
ui_surface source changes
ui_render_primitives source changes for backend generation
ui_input source changes without proven missing normalized fact
```

## Closeout And Next Planning Action

Phase 16 is closed through `docs-site/src/content/docs/reports/closeouts/phase-16-surface2d-closeout.md`.

The next production-track planning step is `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas intake. This completed Surface2D design does not authorize Phase 17 implementation.

Required closeout validation:

```text
python tools/docs/validate_docs.py
git diff --check
```
