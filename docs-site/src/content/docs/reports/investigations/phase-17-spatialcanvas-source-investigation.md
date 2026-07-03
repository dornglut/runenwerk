---
title: Phase 17 SpatialCanvas Source Investigation
description: Source-level investigation for PT-UI-COMPONENT-PLATFORM-017 SpatialCanvas planning and design intake.
status: active
owner: ui
layer: reports
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../../design/active/ui-component-platform-spatial-canvas-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/investigations/surface2d-future-pressure-branch-review.md
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../domain/ui/architecture.md
  - ../../domain/ui/crate-ownership.md
  - ../../domain/ui/dependency-boundaries.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
---

# Phase 17 SpatialCanvas Source Investigation

## Status

This report records the source-level investigation for opening `PT-UI-COMPONENT-PLATFORM-017` SpatialCanvas as planning/design intake.

Lifecycle state: `active-planning`.

Implementation authorization: not granted.

This report is current-reality evidence for the companion design:

```text
docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md
```

## Evidence classes

```text
E2 repository file and GitHub PR metadata inspection
E3 current source-code/test inspection
E5 local command output for git branch/PR status and later docs validation
E8 accepted workspace, UI-domain, planning, design, ADR, and closeout authority
```

Command validation for this docs/intake PR is limited to:

```text
python tools/docs/validate_docs.py
git diff --check
```

No cargo validation is expected unless product code changes, which would be a stop condition for this PR.

## Question

Which exact existing UI crates and source paths can support a future `SpatialCanvas` contract, which crates are explicit non-owners, and what must Phase 17 prove before any implementation is authorized?

Reason investigation is required:

```text
SpatialCanvas is reusable platform work.
SpatialCanvas depends on completed Surface2D.
SpatialCanvas may touch renderer-neutral contracts, input behavior, retained UI, inspection/catalog projection, and domain boundaries.
Existing graph-canvas code is graph-specific and must not be blindly reused as the generic owner.
```

Expected downstream gate:

```text
complete design gate before active implementation
roadmap/planning update for active-planning only in this PR
```

## Lifecycle

Current state:

```text
PT-UI-COMPONENT-PLATFORM-016 Surface2D: completed
PT-UI-COMPONENT-PLATFORM-017 SpatialCanvas: future before this intake
No active implementation focus existed before this intake
```

Candidate state transition:

```text
production-track -> active-planning
```

Implementation is currently forbidden.

## Current repository state

Inspected current branch and PR state:

```text
git status --short --branch
git branch --all --verbose --no-abbrev
git log --oneline --decorate -n 20
gh pr list --repo Crystonix/Runenwerk --state open --limit 50
gh pr list --repo Crystonix/Runenwerk --state all --limit 10
```

Findings:

```text
main and origin/main point to 05c51375986cf08e360884ebf44702ec62662c1e
PR #64 merged Surface2D future-pressure extraction at 05c51375986cf08e360884ebf44702ec62662c1e
PR #63 merged Phase 16 closeout at 53349154809bf779dba349269afeb1f3c3deb646
PR #62 merged workflow/principle/decomposition hardening at 6cfb82b81aa5478496ff6cbf3fa2eea607777aaf
PR #61 merged Phase 16 Surface2D implementation at 2e803620c91726fb599c5e5c4eee4b3984cd4a9d
GitHub reports no open PRs
origin/surface2d-phase-16 is absent from the current remote branch list
local stale gone branches exist but do not affect remote planning truth
```

Planning drift found:

```text
active-work.md and production-tracks.md still mentioned the old retained Surface2D branch before this intake update.
The current repo also contains surface2d-future-pressure-branch-review.md, which says the stale branch could be deleted after extraction.
Current branch metadata shows the extraction PR merged and the remote stale branch absent.
```

## Authority/source matrix

| Claim | Source inspected | Authority level | Evidence found | Conflict / drift |
|---|---|---:|---|---|
| Workflow requires complete investigation and design gates | `AGENTS.md`, workflow lifecycle, complete investigation/design gates | E8 | Production-track reusable UI work must not implement until gates are complete and planning authorizes exact scope. | None. |
| Phase 16 is closed | `completed-work.md`, `roadmap.md`, `production-tracks.md`, closeout | E8/E5 | PR #61/#62 and post-merge validation are recorded. | None for Phase 16 product scope. |
| Surface2D future pressure was extracted | `surface2d-future-pressure-branch-review.md`, PR #64 metadata | E2/E5/E8 | Report exists on `main` and PR #64 is merged. | Some planning text stale until this update. |
| SpatialCanvas is the next named milestone | `production-tracks.md`, roadmap, older SpatialCanvas design, UI platform roadmap | E8 | Phase 17 is named as SpatialCanvas. | Older design lacks complete gate evidence. |
| Surface2D owns coordinate/navigation substrate | Surface2D design, closeout, `ui_controls::surface2d`, `ui_runtime::surface2d` | E3/E8 | Descriptors, validation, summaries, proof reports, transforms, and proof frames exist. | None. |
| `ui_controls` has the package/catalog/inspection extension path | `ui_controls` package, catalog, inspection, Surface2D files/tests | E3 | Surface2D pattern is present and package-backed. | None. |
| `ui_runtime` owns runtime proof/report/frame evidence | `ui_runtime/src/surface2d`, runtime tests | E3 | Surface2D proof report and proof frame are split into report/proof/frame/transform. | None. |
| `ui_static_mount` consumes proof frames | `ui_static_mount/tests/base_controls_surface2d_static_mount.rs` | E3 | Static mount proof uses runtime Surface2D proof frame. | None. |
| Existing graph canvas is graph-specific | `ui_graph_editor`, `ui_tree`, `ui_runtime`, `ui_render_data` graph paths | E3 | Current code names graph nodes, ports, edges, graph hit targets, graph actions, graph primitive roles. | Must remain pressure evidence only. |
| `ui_surface` is separate semantic surface compatibility vocabulary | UI architecture, crate ownership, `ui_surface/src/lib.rs` | E3/E8 | It owns definition/mount/observation/session/presentation/intent/ratification/validation. | SpatialCanvas must not replace it. |
| `ui_composition` owns app-neutral structure, not in-canvas item semantics | UI architecture, crate ownership, `ui_composition/src/lib.rs` | E3/E8 | Composition owns structural definitions, transactions, state, persistence. | SpatialCanvas must not become app composition. |

## Current-state matrix

| Area | Current code reality | Current docs reality | Tests/proofs | Gap |
|---|---|---|---|---|
| Surface2D package path | `domain/ui/ui_controls/src/surface2d/` exists with split ids/support/descriptor/contribution modules. | Surface2D design and closeout mark it completed. | `ui_controls/tests/surface2d_package.rs`; package validation tests. | None for Surface2D. |
| Surface2D runtime proof | `domain/ui/ui_runtime/src/surface2d/` exists with transform/report/proof/frame modules. | Closeout records runtime proof and proof-frame projection. | `ui_runtime/tests/surface2d_runtime_proof.rs`. | None for Surface2D. |
| Static mount proof | `ui_static_mount` consumes the runtime proof frame. | Closeout records static proof. | `base_controls_surface2d_static_mount.rs`. | None for Surface2D. |
| SpatialCanvas code | No `spatial_canvas` module found in required UI crates. | Older active design exists as activation note only. | No Phase 17 tests. | Need design/planning before implementation. |
| Graph canvas compatibility | `ui_tree::GraphCanvasNode`, `ui_runtime` graph pointer/keyboard/emit code, `ui_render_data` graph primitive roles exist. | ADR 0010 separates graph truth from canvas/editor behavior. | `ui_graph_editor` unit tests and retained runtime tests. | Cannot serve as generic SpatialCanvas owner without redesign. |
| Viewport/camera | `editor_viewport` owns camera/projection/runtime settings. | UI platform roadmap says viewport projection is separate ownership. | Existing editor viewport code. | SpatialCanvas must not own cameras or scene resources. |
| Spatial/index crates | `domain/spatial` and `domain/spatial_index` exist for world/spatial contracts and indexes. | Domain map assigns spatial coordinates/indexing there. | Existing spatial index tests. | Phase 17 should not depend on them unless accepted need is proven. |

## Source facts

### `domain/ui/ui_controls`

Current Surface2D owner path:

```text
domain/ui/ui_controls/src/surface2d/mod.rs
domain/ui/ui_controls/src/surface2d/ids.rs
domain/ui/ui_controls/src/surface2d/support.rs
domain/ui/ui_controls/src/surface2d/descriptor.rs
domain/ui/ui_controls/src/surface2d/contribution.rs
domain/ui/ui_controls/src/package/surface2d_validation.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/src/base_control/lowering/surface2d_support.rs
domain/ui/ui_controls/tests/surface2d_package.rs
```

Observed Surface2D descriptor facts:

```text
proof_required defaults true
renderer_backend_required defaults false
executes_host_commands defaults false
mutates_product_state defaults false
graph_or_timeline_semantics defaults false
input modes include keyboard pan/zoom/fit, pointer capture, wheel scroll, trackpad/touch/controller status rows
layers include background, grid, diagnostic overlay, and selection box
budget evidence includes transform, pan/zoom, hover, selection rectangle, fit, large-content bounds, report generation, static mount, and primitive count
```

SpatialCanvas implication:

```text
ui_controls is the likely descriptor/catalog/inspection owner for Phase 17.
It should add SpatialCanvas descriptors only if they reference Surface2D support rather than duplicating Surface2D fields.
```

### `domain/ui/ui_runtime`

Current Surface2D proof owner path:

```text
domain/ui/ui_runtime/src/surface2d/mod.rs
domain/ui/ui_runtime/src/surface2d/transform.rs
domain/ui/ui_runtime/src/surface2d/report.rs
domain/ui/ui_runtime/src/surface2d/proof.rs
domain/ui/ui_runtime/src/surface2d/frame.rs
domain/ui/ui_runtime/tests/surface2d_runtime_proof.rs
```

Observed proof shape:

```text
Surface2DProofReport separates descriptor, transform, navigation, hover, selection, pointer-capture, gesture, accessibility/input, budget, diagnostics, catalog, inspection, static-mount expectation, and boundary-counter evidence.
Surface2DTransform maps world-to-screen and screen-to-world only when pan/zoom are finite and zoom is positive.
Surface2DProofRenderFrame projects report evidence into renderer-neutral UiFrame primitives.
Boundary counters track side effects, semantic writes, and backend resources.
```

SpatialCanvas implication:

```text
ui_runtime is the likely proof/report/frame owner for generic item facts, hit regions, hover, selection, marquee, drag-intent, culling, and budget evidence.
It should consume Surface2D proof/facts instead of creating a second transform owner.
```

### `domain/ui/ui_static_mount`

Observed path:

```text
domain/ui/ui_static_mount/tests/base_controls_surface2d_static_mount.rs
```

SpatialCanvas implication:

```text
ui_static_mount should remain a proof consumer.
Static mount tests must consume a runtime SpatialCanvas proof frame if implementation is later authorized.
```

### `domain/ui/ui_render_data`

Observed relevant paths:

```text
domain/ui/ui_render_data/src/frame/*
domain/ui/ui_render_data/src/primitives/*
domain/ui/ui_render_data/src/primitives/graph_canvas.rs
domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs
```

Current facts:

```text
UiFrame, UiSurface, UiLayer, rect, border, stroke, clip, glyph-run, product surface, image, and viewport embed data exist.
GraphCanvasPrimitiveBatch exists but is role-tagged with graph roles: node box, port, edge, label, selection outline, connection preview, overlay.
ViewportSurfaceEmbedPrimitive is slot-scoped and carries viewport id, slot, rect, UV rect, tint, and sort key.
```

SpatialCanvas implication:

```text
ui_render_data is not a default Phase 17 owner.
Existing primitives should be sufficient for proof frames unless accepted implementation evidence proves otherwise.
GraphCanvas primitive roles should not become SpatialCanvas source truth.
```

### `domain/ui/ui_render_primitives`

Current facts:

```text
ui_render_primitives generates backend-neutral primitives from runtime-view/button reports.
It is button/runtime-view oriented and does not own Surface2D proof.
```

SpatialCanvas implication:

```text
ui_render_primitives is not a Phase 17 intake owner.
Touching it would be a stop condition unless a later accepted design proves primitive generation is required.
```

### `domain/ui/ui_input`

Observed relevant paths:

```text
domain/ui/ui_input/src/facts.rs
domain/ui/ui_input/src/pointer.rs
domain/ui/ui_input/src/focus.rs
domain/ui/ui_input/src/keyboard.rs
domain/ui/ui_input/src/semantic.rs
```

Current facts:

```text
NormalizedInputFact carries pointer, keyboard, focus, semantic, text intent, text edit, composition, and selection facts.
Pointer facts carry pointer kind, position, delta, button, modifiers, click count, and packet data.
Pointer packets support mouse, trackpad, touch, stylus, pressure, tilt, twist, hover, eraser, barrel buttons, coalesced/predicted samples, calibration, and latency class.
Facts do not decide control behavior and do not execute product commands.
```

SpatialCanvas implication:

```text
ui_input is not a default Phase 17 owner.
SpatialCanvas should consume existing facts through ui_runtime.
New input vocabulary requires proof that existing normalized facts cannot express the accepted contract.
```

### `domain/ui/ui_surface`

Observed path:

```text
domain/ui/ui_surface/src/lib.rs
```

Current modules:

```text
capability
definition
diagnostics
intent
mount
observation
presentation
ratification
session
validation
```

SpatialCanvas implication:

```text
ui_surface remains separate semantic surface vocabulary.
SpatialCanvas must not rename, replace, remove, or absorb ui_surface without separate accepted design.
```

### `domain/ui/ui_tree`

Observed relevant paths:

```text
domain/ui/ui_tree/src/tree/node/graph_canvas.rs
domain/ui/ui_tree/src/tree/node/surface.rs
domain/ui/ui_tree/src/tree/node/mod.rs
```

Current facts:

```text
GraphCanvasNode stores ui_graph_editor::GraphCanvasViewModel plus focus, capture, wheel zoom, clipping, theme, and text style.
ViewportSurfaceEmbedNode stores viewport_id, slot, and min_size.
```

SpatialCanvas implication:

```text
ui_tree should not be assumed as a Phase 17 owner.
GraphCanvasNode is graph-specific and cannot be generalized by rename.
Touching ui_tree requires explicit retained-node proof, decomposition, and no semantic leakage.
```

### `domain/ui/ui_composition`

Observed path:

```text
domain/ui/ui_composition/src/lib.rs
```

Current facts:

```text
ui_composition owns app-neutral structural definitions, state, transactions, validation, history, promotion, persistence, content references, and diagnostics.
Crate ownership docs forbid composition from importing other production UI crates or app/editor/provider semantics.
```

SpatialCanvas implication:

```text
ui_composition is not a Phase 17 owner.
SpatialCanvas item containment is in-canvas proof/inspection data, not app layout persistence or mounted-content structure.
```

### Existing graph canvas and graph-editor pressure

Observed paths:

```text
domain/ui/ui_graph_editor/src/lib.rs
domain/ui/ui_tree/src/tree/node/graph_canvas.rs
domain/ui/ui_runtime/src/input/pointer/graph_canvas.rs
domain/ui/ui_runtime/src/runtime/ui_runtime/graph_canvas.rs
domain/ui/ui_runtime/src/output/emit/graph_canvas.rs
domain/ui/ui_render_data/src/primitives/graph_canvas.rs
```

Current facts:

```text
ui_graph_editor owns GraphCanvasId, GraphNodeKey, GraphPortKey, GraphEdgeKey, GraphSelectionKey, GraphPoint, GraphRect, GraphViewport, GraphHitTarget, GraphCanvasAction, GraphEditorAction, GraphShortcutAction, and GraphCanvasGestureState.
ui_runtime graph pointer dispatch emits graph selection, pan, zoom, node drag, connection, marquee, and graph action intents.
ui_runtime graph keyboard helpers emit graph shortcuts such as add node, undo, redo, build preview, and focus preview.
render-data graph primitive roles include node box, port, edge, label, selection outline, connection preview, and overlay.
```

SpatialCanvas implication:

```text
Existing graph canvas provides future-use pressure for item bounds, hit regions, selection, marquee, drag intent, labels, overlays, and culling.
It is not the generic SpatialCanvas owner because it already contains graph/node/port/edge/action vocabulary.
```

### Adjacent non-UI owners

Observed paths:

```text
domain/editor/editor_viewport/src/camera.rs
domain/editor/editor_viewport/src/lib.rs
domain/scene/src/lib.rs
domain/spatial/src/lib.rs
domain/spatial_index/src/lib.rs
```

Current facts:

```text
editor_viewport owns editor camera/projection/runtime settings and viewport debug stages.
scene owns scene transforms and schema.
spatial owns world positions, frames, clipmap, grids, and spatial coordinate contracts.
spatial_index owns spatial index traits, query, storage, and spatial hash index.
```

SpatialCanvas implication:

```text
camera, projection, scene resources, world coordinates, and indexing are non-owners for Phase 17 unless a later accepted design proves a direct dependency.
SpatialCanvas should use UI/Synthetic proof coordinates from Surface2D, not world/scene camera truth.
```

## Ownership recommendation

Recommended owners for a later implementation contract:

```text
ui_controls:
  package-backed SpatialCanvas declarations, descriptor validation, catalog summaries,
  inspection facts, base-control contribution, and no-owner guard fields.

ui_runtime:
  runtime proof report, generic item hit/hover/selection/marquee/drag-intent evidence,
  visible-window/culling facts, budget evidence, and renderer-neutral proof-frame projection.

ui_static_mount:
  static proof that the runtime SpatialCanvas proof frame mounts without bypassing
  package/catalog/inspection/runtime evidence.
```

Conditional owners only after accepted evidence:

```text
ui_tree:
  only if retained runtime behavior is accepted as part of Phase 17.

ui_render_data:
  only if existing primitives cannot express the proof frame.

ui_input:
  only if existing normalized facts cannot express required input evidence.

ui_surface:
  only if a separate accepted surface mapping/migration decision is required.

ui_composition:
  not expected; app composition is outside SpatialCanvas.
```

Explicit non-owners:

```text
ui_render_primitives
ui_graph_editor as generic owner
domain/spatial
domain/spatial_index
domain/scene
domain/editor/editor_viewport
apps/runenwerk_editor
engine renderer backends
product/editor/game command owners
foundation/meta
plugin/provider/app-composition framework owners
```

## Capability inventory

| Capability | Exists now | Evidence | Missing contract | Required owner |
|---|---|---|---|---|
| Surface2D transform/navigation facts | yes | Surface2D source/design/closeout | none for dependency | Surface2D owners |
| Spatial item identity/bounds | no generic contract | older design note only | descriptor + proof facts | `ui_controls`, `ui_runtime` |
| Generic hit regions | graph-specific only | `ui_graph_editor::GraphHitTestScene` | generic item hit-region proof | `ui_runtime` |
| Hover item fact | graph-specific only | graph pointer dispatch | generic item hover report | `ui_runtime` |
| Selection set fact | graph-specific only | graph selection/action code | UI-only generic selection fact | `ui_runtime` |
| Marquee selection | graph-specific only | `GraphMarqueeSelectionState` | generic item intersection report | `ui_runtime` |
| Drag item intent | graph-specific only | graph node drag action | generic begin/update/end/cancel intent fact | `ui_runtime` |
| Labels/badges/adornments | partially through text/render primitives and graph overlays | graph emit path, text primitives | generic adornment descriptor/proof facts | `ui_controls`, `ui_runtime` |
| Culling/visible window | claimed in older design, not generic source | no SpatialCanvas source | deterministic visible-window/budget proof | `ui_runtime` |
| Static proof | yes pattern exists | Surface2D static mount | SpatialCanvas proof-frame test | `ui_static_mount` |
| Retained SpatialCanvas node | no | only GraphCanvasNode exists | retained generic node decision | conditional `ui_tree` |
| Renderer backend integration | not owned | dependency rules and current render owners | none | engine renderer outside Phase 17 |

## Alternatives and tradeoff matrix

| Option | Benefits | Costs | Boundary impact | Long-term fit | Recommendation |
|---|---|---|---|---|---|
| Build SpatialCanvas as package/runtime/static proof over Surface2D | Reuses completed substrate, avoids app/product ownership, keeps proof path direct | Does not immediately replace retained GraphCanvas | Low boundary risk | Strong | Accept for planning |
| Generalize existing GraphCanvas code directly | Reuses existing hit/gesture code | Brings graph/node/port/action vocabulary into generic contract | High boundary risk | Weak | Reject |
| Add a new `ui_spatial_canvas` crate | Clean physical boundary | New crate before proof and architecture decision | Medium/high YAGNI risk | Unknown | Stop condition until explicitly accepted |
| Put SpatialCanvas in `ui_tree` first | Enables retained runtime integration | Skips package/proof evidence and risks graph-canvas copy | Medium/high risk | Weak for first slice | Reject for default path |
| Use `spatial_index` immediately for culling | May scale later | Adds optimization and dependency before need | Premature optimization risk | Weak now | Reject until budget evidence proves need |
| Treat SpatialCanvas as `ui_surface` replacement | Could consolidate names later | Disrupts existing semantic surface authority | High boundary risk | Not Phase 17 | Reject |

## Confidence matrix

| Finding | Confidence | Reason | Missing evidence to improve confidence |
|---|---|---|---|
| Surface2D is the required dependency and must not be duplicated | High | Completed source/design/closeout evidence align. | None for intake. |
| `ui_controls`, `ui_runtime`, and `ui_static_mount` are the likely first owners | High | Same successful pattern as Surface2D and current source supports it. | Exact implementation file list after design acceptance. |
| `ui_tree` should not be assumed | High | Existing retained graph node is graph-specific. | A later retained-node proof if needed. |
| `ui_render_data` changes are not required by default | Medium | Surface2D proof used existing primitives; SpatialCanvas proof may be more item-heavy. | Prototype/design proof that existing primitives are enough. |
| Existing `ui_input` facts are likely enough | Medium | Pointer/keyboard/focus facts are generic and rich. | Exact input acceptance matrix and tests. |
| `spatial_index` is not needed for first proof | Medium | No current budget evidence proves need. | Large item budget proof if implementation later shows candidate counts are insufficient. |

## Risks and blockers

Blockers before implementation:

```text
exact public API names are not accepted
exact file/crate scope is not accepted
module decomposition map is not accepted by planning
retained ui_tree decision is not made
ui_render_data need is not proven or rejected
accessibility/input acceptance matrix is not accepted
validation envelope is not accepted
stop conditions are not accepted in active-work planning
```

Risks if implementation proceeds without more evidence:

```text
duplicating Surface2D transform/navigation ownership
turning graph-canvas compatibility code into a generic platform contract
moving product selection mutation into domain/ui
adding a speculative spatial index or new crate
touching ui_surface without migration authority
using renderer primitive roles as semantic truth
```

## Recommendation

Open Phase 17 as planning/design intake only.

Recommended next gate:

```text
complete design gate review for SpatialCanvas
```

Required files/docs to update in this PR:

```text
docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md
docs-site/src/content/docs/reports/investigations/phase-17-spatialcanvas-source-investigation.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Do not update `completed-work.md` because Phase 17 is not completed.

Do not change product code in this PR.
