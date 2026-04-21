# Runenwerk — Recommended Phase Order After Viewport Upgrade

_Last updated: 2026-04-19_

## Purpose

Capture the revised recommended implementation order after the viewport upgrade audit so the next targets stay clear and stable.

This order is based on:

- the current viewport implementation state
- the viewport expression upgrade design
- the editor / UI / workspace / tool-surface architecture
- the field world and simulation platform design

---

## Current Assessment

The viewport upgrade is **architecturally real and successful as phase 1**, but it is not yet the full end-state.

What is now true:

- the old fullscreen-mask model is no longer the architectural center
- typed viewport products and presentation state are real
- shell embedding of viewport surfaces is real
- viewport-local routing and interaction are real

What is still incomplete:

- backend surface ownership is still somewhat binding-based
- the producer side is still too centralized around the main flow
- picking backend ownership is not yet as viewport-scoped as the long-term design wants
- multi-viewport is structurally supported, but not yet broadly exercised

Because of that, the next order should prioritize finishing the first serious surface correctly, then building the reusable substrate beneath future surfaces, then proving the framework with at least one more serious non-viewport surface.

---

## Recommended Order

## Phase 1 — Finish viewport backend cleanup

This remains the highest priority.

### Goal
Finish the viewport upgrade properly at the backend/render ownership layer.

### Why first
If this is left half-finished, the viewport becomes the first “almost-right” surface and future surfaces will inherit the wrong patterns.

### Focus
- strengthen explicit per-viewport surface ownership
- clean up presentation resolution
- reduce hard dependence on a single main producer flow
- make picking provenance/backend ownership more clearly viewport-scoped
- exercise at least one additional product path beyond the narrow main one

### Success condition
The viewport is not only correct at the contract/presentation level, but also clearly correct at the backend ownership level.

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
1. Finish viewport backend cleanup

### Then
2. Build core UI substrate
3. Add one more serious non-viewport surface
4. Harden workspace / tool-surface host model
5. Use substrate for editor, game, and diagnostics UI
6. Define the expression producer architecture
7. Push field world producer implementation
8. Design gameplay runtime architecture

---

## Best Next Target

If reduced to one concrete milestone:

### Next milestone
**Viewport backend cleanup + UI substrate bring-up**

### Why
These two together:
- finish the first serious surface correctly
- prevent future surfaces from inventing bespoke UI
- give immediate reusable value to editor, game UI, and diagnostics work

### Next milestone after that
**Entity Table / Query Surface**

That is the best proof that the framework is no longer viewport-centric.

---

## Final Recommendation

Do not jump straight into broader workspace abstraction or deep field-world implementation before:

1. the viewport backend is fully cleaned up, and
2. the UI substrate exists strongly enough to support more than one real surface.

The cleanest order from the current state is:

- finish the first serious surface correctly
- build the reusable UI substrate
- prove the framework with another serious surface
- then harden host/workspace structure
- then push deeper producer/substrate architecture
- then grow gameplay on top

That is the most stable long-term path.
