---
title: Runenwerk Next Phase Order
description: Recommended next implementation order after viewport backend cleanup and workspace identity hardening.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# Runenwerk - Recommended Phase Order After Viewport Upgrade

_Last updated: 2026-04-21_

## Purpose

Capture the revised recommended implementation order after the viewport upgrade audit so the next targets stay clear and stable.

This order is based on:

- the current viewport implementation state
- the viewport expression upgrade design
- the editor / UI / workspace / tool-surface architecture
- the field world and simulation platform design

---

## Current Assessment

The viewport upgrade backend cleanup (Phase 1) is complete.

What is now true:

- the old fullscreen-mask model is no longer the architectural center
- typed viewport products and presentation state are real
- shell embedding of viewport surfaces is real
- viewport-local routing and interaction are real
- viewport runtime ownership is viewport-keyed
- `MAIN_VIEWPORT_ID` is bootstrap-only, not runtime fallback
- binding resolution is derived from canonical runtime state
- picking backend ownership is per-viewport
- shell composition no longer manufactures viewport id `1`

What is now the immediate architecture gap:

- Step 1 completed: typed workspace identity families + allocator and bootstrap ownership
- Step 2 completed: canonical `WorkspaceState` graph + reducer-style structural mutations
- Step 3 completed: canonical projection artifacts and structural-id-driven routing metadata
- Step 4 completed: structural-context dispatch with stale-artifact fail-closed behavior
- Step 5 completed: explicit structural tool-surface to runtime binding contracts
- Step 6 completed: guard-test and architecture-test expansion before docking/tab behavior

Because of that, the next order should begin docking/tab behavior implementation on top of the now-locked identity/routing/binding contracts, then continue substrate and broader surface expansion.

---

## Recommended Order

## Phase 1 — Viewport backend cleanup (completed)

Status: complete.

### Goal
Completed: viewport backend/render ownership was aligned with the established architecture direction.

### Completion record
See:

- [Runenwerk - Phase 1 Plan (Completed)](./runenwerk-phase-1-plan.md)

---

## Immediate post-Phase 1 track — Workspace identity hardening (Steps 1-6)

### Goal
Complete the full workspace identity hardening track before docking/tab implementation begins.

### Artifacts
- [Workspace Identity Contract and Migration Map](./workspace-identity-contract-and-migration-map.md)
- Step implementation sequence: identity foundation -> structural graph -> projection artifacts -> structural dispatch -> runtime bindings -> guard tests

### Why now
Without finishing this track, upcoming docking/tab and multi-surface workspace work will likely inherit identity leakage and runtime ownership coupling.

### Success condition
Workspace structural identity, routing identity, and runtime binding identity are explicitly separated and test-guarded.

### Current status
- completed: Steps 1-6
- next target: docking/tab behavior implementation on top of the locked workspace identity contracts

---

## Phase 2 — Build the core UI substrate

This should come before broader workspace hardening.

### Goal
Create the reusable UI primitives that all editor, game, and diagnostics UI can share.

### Why now
Without the substrate, every new surface keeps inventing local widgets and layout behavior. That slows everything down and makes the framework less reusable.

### Must-have primitives
- HBox
- VBox
- stack / overlay
- scroll container
- split container
- spacer
- panel / frame / container
- text label

### Must-have controls
- button
- text input
- integer input
- float input
- checkbox
- dropdown / select
- search field

### Strong near-term additions
- list / table primitives
- tree primitives
- vector inputs
- multiline text input
- tabs

### Success condition
Inspector, outliner, console, game menus, and diagnostics can start using the same real substrate instead of one-off UI code.

---

## Phase 3 — Add one more serious non-viewport surface

This is the most important proof step after the substrate starts existing.

### Goal
Prove that the framework is not viewport-centric.

### Best candidates
1. Entity Table / Query Surface
2. Improved Inspector with proper field controls
3. Both, in that order if possible

### Why this matters
The framework is only truly validated if it can host another serious surface with a different interaction model.

### What this proves
- substrate reuse
- tool-surface reuse
- observation / interaction / command separation
- non-viewport hosting correctness
- future readiness for graphs, timelines, viewers, and diagnostics surfaces

### Success condition
At least one more serious surface exists without needing a major architecture rewrite.

---

## Phase 4 — Harden the workspace / tool-surface host model

This should happen after viewport cleanup and after at least one more real surface is running.

### Goal
Strengthen host/editor composition boundaries using real evidence instead of speculation.

### Focus
- WorkspaceId
- PanelHostId
- PanelInstanceId
- ToolSurfaceInstanceId
- tab-stack-ready structure
- stronger host/panel/surface separation
- persistence seams
- dock/tab readiness without full productization yet

### Why here
This becomes easier and more correct after the substrate exists and more than one serious surface is already being hosted.

### Success condition
The host editor composes and routes surfaces cleanly without becoming the semantic owner of those tools.

---

## Phase 5 — Use the substrate for editor, game, and diagnostics UI

### Goal
Start exploiting the main payoff of the UI architecture:
one shared substrate serving multiple environments.

### Target categories
- editor UI
- in-game UI
- debug / diagnostics UI

### Important rule
Reuse the substrate everywhere, and selectively reuse panel/tab composition where it is actually useful.
Do not force normal game HUD/menu UI into full editor workspace semantics.

### Success condition
You can build editor panels, game menus, and diagnostics surfaces on the same UI foundation.

---

## Phase 6 — Define the expression producer architecture

This is the missing bridge between the field world platform and the viewport/tool consumers.

### Goal
Define how products are actually produced, identified, invalidated, and exposed.

### It should cover
- producer identities
- product families
- registry ownership
- product descriptors
- binding/resolution rules
- invalidation rules
- scene / picking / overlay / debug producer paths

### Why this is important
The viewport now consumes products, and the field world design assumes producers and formed products, but the middle architecture is still not fully explicit.

### Success condition
There is a clear producer architecture between world/substrate and consumer surfaces.

---

## Phase 7 — Push field world producer implementation

### Goal
Move deeper into the long-term field-world architecture:
- formed chunk-local products
- multiscale bands / clipmaps
- explicit invalidation lineage
- stronger runtime residency and rebuild behavior

### Why later than phase 6
The field world platform should feed a clear producer layer, not push products directly into consumers through ad hoc paths.

### Success condition
The world substrate begins producing real field-world-driven products through the proper producer architecture.

---

## Phase 8 — Gameplay runtime architecture

### Goal
Define the long-term runtime gameplay model for:
- player
- enemies
- interactions
- combat
- worldgen/gameplay integration
- runtime authority boundaries

### Why last in this sequence
The editor/tooling/viewport/product/substrate foundations should be stable before major gameplay architecture grows on top of them.

### Success condition
Gameplay runtime architecture grows on top of stable platform seams rather than unstable implementation shortcuts.

---

## Condensed Version

### Immediate
1. Phase 1 viewport backend cleanup (completed)
2. Workspace identity hardening track (Steps 1-6 completed)
3. Docking/tab behavior implementation (next)

### Then
4. Build core UI substrate
5. Add one more serious non-viewport surface
6. Harden workspace / tool-surface host model
7. Use substrate for editor, game, and diagnostics UI
8. Define the expression producer architecture
9. Push field world producer implementation
10. Design gameplay runtime architecture

---

## Best Next Target

If reduced to one concrete milestone:

### Next milestone
**Docking/tab behavior implementation (identity-safe)**

### Why
Steps 1-6 are complete, including structural-to-runtime binding contracts and guard tests. The next risk-reducing move is implementing docking/tab behavior against the existing `WorkspaceState` + reducer + projection + structural routing contracts.

### Next milestone after that
**UI substrate bring-up**.

Then: **Entity Table / Query Surface** as non-viewport proof.

---

## Final Recommendation

Do not jump straight into docking/tab productization or deep field-world implementation before:

1. the Phase 1 viewport backend cleanup completion is locked, and
2. workspace identity hardening Steps 1-6 are complete.

The cleanest order from the current state is:

- close Phase 1 (done)
- close workspace identity hardening (done)
- implement docking/tab behavior on top of locked identity contracts
- build the reusable UI substrate
- prove the framework with another serious surface
- then harden host/workspace structure
- then push deeper producer/substrate architecture
- then grow gameplay on top

That is the most stable long-term path.
