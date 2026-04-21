# Runenwerk — Phase 1 Plan: Viewport Backend Cleanup

_Last updated: 2026-04-19_

## Purpose

Finish the viewport upgrade at the backend/render ownership layer so the implementation fully matches the architectural direction already established in the viewport design and confirmed by the audit.

This phase is **not** another conceptual redesign.
It is a cleanup and completion phase for the viewport architecture that already exists.

---

## Why Phase 1 Comes First

The viewport upgrade is already architecturally real:

- typed expression products exist
- viewport presentation state exists
- product observation exists
- shell embedding of viewport surfaces exists
- viewport-local routing exists

But the audit also found several remaining backend-phase issues:

- backend surface ownership is still too binding-based
- product resolution is still too centralized around the main flow
- picking provenance/backend ownership is not yet as viewport-scoped as desired
- multi-viewport support exists structurally but is not yet meaningfully exercised

If this phase is skipped, the first serious tool surface remains only partially finished, and future surfaces risk inheriting the wrong backend patterns.

---

## Phase 1 Objective

Move the viewport implementation from:

**architecturally correct phase-1 contracts with a still-centralized backend**

to:

**backend ownership and presentation flow that clearly match the viewport design**

The viewport should finish this phase as:

- a true presentation consumer
- backed by clearer per-viewport surface ownership
- less dependent on one global/main producer path
- with cleaner viewport-scoped picking ownership
- ready to act as the reference implementation for future serious tool surfaces

---

## Scope

## In scope

- viewport backend cleanup
- presentation resolver cleanup
- stronger per-viewport surface ownership
- cleaner viewport-scoped picking backend path
- one additional exercised product path beyond the narrow main path
- one focused multi-viewport readiness pass
- tests/assertions for the new ownership model

## Out of scope

- full docking/tab workspace productization
- broad UI substrate rollout
- graph/timeline/curve tool surfaces
- full field-world producer implementation
- gameplay/runtime architecture
- full multi-viewport feature breadth
- broad new product taxonomy implementation

This phase is about finishing the viewport correctly, not broadening the entire editor.

---

## Main Problems To Solve

## 1. Surface ownership is too thin
The current implementation resolves presentation mostly through product-to-resource bindings.
That is good enough for phase 1 bring-up, but still thinner than the intended viewport-owned surface model.

### Required improvement
Strengthen the implementation so the viewport clearly owns its presentation surface relationship and does not merely look up the currently bound main-flow resource.

---

## 2. Product resolution is too centralized
The current path is still strongly biased toward one main producer flow.

### Required improvement
Reduce hard architectural dependence on one main flow being the implicit owner of all viewport products.

---

## 3. Picking backend provenance is still too centralized
Viewport-local interaction exists, but the picking backend still relies too much on globally produced picking state before wrapping it into a viewport expression result.

### Required improvement
Make picking ownership and resolution more explicitly viewport-scoped.

---

## 4. Multi-viewport readiness is not yet meaningfully exercised
The resource model is keyed by `ViewportId`, but the implementation still behaves mostly as a singleton-centered system.

### Required improvement
Exercise at least one more meaningful non-main viewport path or equivalent test coverage.

---

## Target Files / Areas

These are the primary areas to work in first:

### 1. Presentation resolution
- `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs`

### 2. Frame submission / shell embedding
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`

### 3. Picking expression/backend path
- `apps/runenwerk_editor/src/runtime/expression/picking.rs`

### 4. Viewport runtime resources
- `apps/runenwerk_editor/src/runtime/viewport/*`

### 5. Shell integration verification
- `apps/runenwerk_editor/src/shell/*`
- `apps/runenwerk_editor/src/editor_app/facade.rs`

---

## Recommended Work Order

## Step 1 — Lock the target ownership model

Before changing code further, restate the implementation target in one sentence inside the relevant runtime code/docs:

> Viewports consume typed expression products, resolve them through viewport-scoped presentation state, and embed viewport-owned presentation surfaces into the shell.

### Outcome
Every later edit can be judged against this sentence.

---

## Step 2 — Refactor presentation resolution around viewport ownership

### Goal
Make presentation resolution clearly viewport-driven rather than flow-driven.

### Required work
- review `presentation_resolver.rs`
- separate:
  - product lookup
  - viewport presentation selection
  - viewport surface binding/allocation ownership
- reduce direct assumptions that `EDITOR_MAIN_FLOW_ID` is the default semantic owner of everything
- make the resolver read as:
  - viewport id
  - selected product id
  - viewport-scoped surface relationship
  - resolved presentation binding

### Desired outcome
The code should read like viewport-owned presentation logic, not like a global render-flow lookup helper.

---

## Step 3 — Strengthen viewport surface ownership

### Goal
Make the surface model clearer and less implicit.

### Required work
- review the current `surface_set` and binding-related structures
- ensure the runtime has a clear per-viewport owned surface relationship
- prepare room for:
  - color surface
  - picking/id surface
  - overlay/composition surface later
- do not overbuild future breadth, but make the ownership model unmistakable

### Desired outcome
A future reader should immediately see that the viewport owns presentation surfaces as a set/bundle, even if only a narrow subset is currently exercised.

---

## Step 4 — Clean up the picking backend path

### Goal
Make picking backend ownership more explicitly viewport-scoped.

### Required work
- review `runtime/expression/picking.rs`
- trace where `EditorPickingResultResource` or equivalent global picking state still acts as the real owner
- refactor so the final mental model is:
  - viewport-local route
  - viewport-scoped picking product/frame
  - viewport hit resolution
- avoid leaving picking as “global result wrapped later” if that can be cleaned up now without destabilizing the system

### Desired outcome
Picking should feel like a viewport product path, not like a global side-channel adapted at the end.

---

## Step 5 — Exercise at least one additional product path

### Goal
Make sure the implementation is not only “main scene color works.”

### Candidate options
- explicitly exercise overlay product path
- explicitly exercise picking ids as a proper selected product path
- add one simple non-scene/debug product if low-cost enough

### Why
This is the best way to verify that typed product selection is real and not only decorative.

### Desired outcome
At least one additional product path is demonstrated and verified through the same presentation model.

---

## Step 6 — Add multi-viewport readiness checks

### Goal
Prove the current architecture is not secretly singleton-only.

### Required work
- add tests and/or assertions for:
  - multiple `ViewportId`s in registry/presentation state
  - independent selected products
  - independent layout rect handling
  - no accidental overwrite between viewports
- if practical, exercise one second viewport path minimally
- if not practical yet, add strong tests that simulate it

### Desired outcome
The architecture is still phase-1 narrow in breadth, but no longer ambiguous in structure.

---

## Step 7 — Strengthen tests and invariants

### Goal
Make regression harder.

### Add tests/assertions for
- shell frame contains viewport embed primitive
- overlay does not silently replace viewport embed path
- selected product resolution is viewport-specific
- invalid product selection fails cleanly
- picking hit resolution uses viewport-local route data
- one viewport’s presentation state changes do not affect another’s unexpectedly

### Desired outcome
The new architecture is protected by explicit tests, not only by convention.

---

## Acceptance Criteria

Phase 1 is complete when all of the following are true:

### Architecture / ownership
- viewport presentation resolution reads as viewport-owned, not global-flow-owned
- per-viewport surface ownership is explicit enough to match the design direction
- no critical path still depends conceptually on fullscreen viewport masking

### Picking
- picking is clearly viewport-local at both routing and backend resolution boundaries
- the picking path no longer feels like a global side-channel wrapped at the end

### Product model
- at least one additional non-main product path is exercised through the same product/presentation architecture
- selected product resolution is clearly keyed by `ViewportId`

### Multi-viewport readiness
- multi-viewport independence is at least tested, even if breadth is still narrow

### Confidence
- regression tests/assertions exist for the new ownership model

---

## Deliverables

By the end of this phase, you should have:

1. cleaned runtime viewport backend code
2. stronger per-viewport presentation ownership
3. cleaner viewport-scoped picking path
4. at least one additional exercised product path
5. multi-viewport readiness tests/assertions
6. a viewport implementation that is ready to act as the template for future serious tool surfaces

---

## What Should Follow Immediately After Phase 1

After this phase, the best next milestone is:

## UI substrate bring-up
Build the reusable primitives needed by:
- outliner
- inspector
- console
- game UI
- diagnostics UI

Recommended immediate primitives:
- HBox
- VBox
- stack / overlay
- scroll container
- panel / frame
- button
- text input
- integer input
- float input
- checkbox
- search field

Then the next proof step after that should be:

## Entity Table / Query Surface
This is the best non-viewport surface to prove the framework is no longer viewport-centric.

---

## Short Version

### Phase 1
Finish the viewport backend so the upgrade is fully real, not only contractually real.

### Main targets
- presentation resolver
- per-viewport surface ownership
- viewport-scoped picking backend
- one extra product path
- multi-viewport readiness checks

### Primary files
- `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs`
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`
- `apps/runenwerk_editor/src/runtime/expression/picking.rs`

### Exit condition
The viewport becomes the first fully correct serious tool surface and a valid template for future surfaces.
