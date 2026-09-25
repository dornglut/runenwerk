---
title: SDF-First Character Physics Ownership Investigation
description: Current-source investigation selecting the smallest physics owner and first character-motion contract required by the maintained Runenwerk small-game gold path.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_docs:
  - ../../domain/physics/README.md
  - ../../domain/world-sdf/README.md
  - ../../domain/simulation/README.md
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ./2026-09-25-shared-small-game-gold-path.md
---

# SDF-First Character Physics Ownership Investigation

## Purpose

Issue #948 asks what physics Runenwerk actually needs to unblock the maintained small-game gold
path without using that consumer as an excuse to invent a general physics framework.

The answer is narrower than the existing long-term Physics Track E envelope:

```text
world_sdf collision truth
        |
        v
small reusable kinematic-character response contract
        |
        v
same game command application
        |
        +--> local single-player authority
        +--> listen-server / dedicated authority
        `--> later predicted-client replay
```

The first implementation should be an SDF-backed spherical kinematic character with bounded
move-and-slide, support/slope classification, gravity, and explicit fail-closed collision
readiness. It should not start with rigid bodies, joints, a Rapier integration, or a generic
collider framework.

This report records the evidence and the decision. The durable owner contract selected here is
published separately at `docs-site/src/content/docs/domain/physics/README.md`.

## Evidence Baseline

The source census used accepted Runenwerk main:

```text
95cd2733241945108f3037c31ffcebbbc78224f5
```

Relevant current source and authority included:

```text
domain/world_sdf/src/collision.rs
domain/world_sdf/src/lib.rs
engine/src/plugins/world/adapters/resources.rs

domain/simulation/**
engine/src/plugins/simulation.rs
engine/src/runtime/fixed_time.rs
engine/src/runtime/fixed_step_executor.rs

engine/src/plugins/net/driver.rs
engine/src/plugins/net/prediction.rs

docs-site/src/content/docs/domain/world-sdf/README.md
docs-site/src/content/docs/domain/simulation/README.md
docs-site/src/content/docs/design/active/editor-procedural-content-and-simulation-workflow-plan.md
docs-site/src/content/docs/reports/investigations/2026-09-25-shared-small-game-gold-path.md
```

Search was used only for discovery. The ownership and readiness conclusions below were checked
against exact files at the accepted revision.

## Existing Collision Truth Is Already Strong Enough for the First Shape

`domain/world_sdf/src/collision.rs` currently owns:

- `CollisionQueryService`;
- `CollisionSample`;
- `CollisionHit`;
- `CollisionReadiness`;
- `CollisionSweepOutcome`;
- `SphereSweep`.

The authoritative sweep path is important:

```text
sweep_sphere_authoritative
  -> determine every chunk touched by the swept sphere bounds
  -> fail MissingPayload if any required chunk is unavailable
  -> test the start position
  -> sample the sweep at a bounded resolution
  -> refine the first collision interval
  -> estimate a field-gradient normal
  -> Hit or Clear
```

The query therefore already answers the first character controller's critical environmental
question: how far a sphere may travel through the currently collision-ready SDF world and what
surface normal blocked it.

The convenience `sweep_sphere` method maps both `MissingPayload` and `Clear` to `None`.
That is unsuitable for authoritative character movement. The first physics slice must use
`sweep_sphere_authoritative` and retain missing-payload truth.

## Existing Architecture Already Selects the Owner

The active procedural-content/simulation workflow plan explicitly states:

- no general `domain/physics` is implemented today;
- future `domain/physics` owns rigid-body, character, contact, constraint, trigger, and collision
  product contracts;
- `domain/world_sdf` retains SDF collision query readiness and sweep outcomes;
- Engine owns concrete runtime/solver integration;
- `docs-site/src/content/docs/domain/physics/README.md` must exist before physics implementation.

That owner split remains correct. The game investigation does not justify moving character physics
into `world_sdf`, Engine, Scene, or the game application.

What changes is the **activation breadth**: the maintained game needs only the character subset
first.

## Why the First Shape Should Be a Sphere

A conventional humanoid controller often uses a capsule. Current Runenwerk world collision truth
does not expose a capsule sweep.

There are three possible reactions:

1. invent a capsule API and approximate it through unrelated samples;
2. add a truthful capsule query to `world_sdf` before any consumer proves it is necessary;
3. design the first maintained game's player collision volume around the supported authoritative
   sphere sweep.

The third option is the smallest truthful path.

It produces a real game, exercises collision readiness, grounding, slopes, move-and-slide, replay,
and prediction constraints, and leaves capsule support as an evidence-driven extension instead of a
precondition.

This does not declare sphere the permanent Runenwerk character shape.

## Character Versus General Body Model

The investigation considered three activation shapes:

### A. Character-only first implementation

Add only the reusable state/config/result semantics required to evaluate one kinematic character
against SDF collision truth.

### B. Small general body/collider vocabulary first

Introduce body kind, collider kind, layers, masks, physics materials, and character behavior
together.

### C. General rigid/kinematic physics foundation first

Start the complete long-term Track E model and integrate a solver.

The selected first implementation is **A**.

B and C would front-load contracts that the first maintained game does not need, increase the
public-domain surface before a consumer exists, and risk forcing external-solver concepts into
Runenwerk's owner model.

The durable `domain/physics` owner contract still reserves those later responsibilities.

## Selected Character Semantics

The first character operation needs the following semantic state:

```text
physical position
physical velocity
grounded/support state
support normal
```

It consumes:

```text
fixed step duration
desired planar movement from the game
gravity
sphere radius
up vector
maximum walkable slope
world/partition/collision query context
```

It produces a typed result rather than mutating unrelated runtime state.

The game owns movement feel:

- desired planar movement;
- speed/acceleration policy;
- jump eligibility;
- jump impulse;
- camera-relative input mapping.

Physics owns reusable response:

- gravity integration;
- sweeps;
- contact clipping;
- move-and-slide;
- grounding/support classification;
- slope acceptance;
- explicit unavailable-collision outcome.

## Selected Move-and-Slide Algorithm

The first implementation should use a bounded iterative algorithm rather than a broad solver:

```text
begin tick
  -> form desired displacement
  -> authoritative sphere sweep
      Clear
        -> admit remaining displacement
      Hit
        -> advance to contact boundary
        -> project remainder onto contact tangent
        -> repeat with bounded iteration count
      MissingPayload
        -> reject tick movement atomically
  -> downward support probe
  -> classify grounded + slope
  -> commit physical state
```

Important constraints:

- the maximum slide iteration count is fixed and small;
- all values are validated as finite/bounded;
- collision skin/epsilon is explicit;
- the solver never loops until convergence;
- presentation state is not read;
- wall-clock/native frame time is not read.

The exact numeric constants remain implementation/test decisions.

## Collision Readiness Must Be Atomic

Fail-closed readiness is the most important correctness rule found by this investigation.

Consider a move that first sweeps through a ready chunk, contacts a wall, slides along it, then needs
a second chunk whose collision payload is absent. Committing the first partial movement would make
the authoritative result depend on which sub-query happened before the missing payload was
discovered.

For the first character slice:

> if any collision query required to resolve the tick returns `MissingPayload`, no physical
> movement from that character-motion evaluation is committed.

The result reports the missing `ChunkId`. The game/runtime can then retain or request world data
according to the world owner's policy.

## Grounding and Slopes

A short downward sphere sweep after the candidate movement supplies support truth.

Walkable support is determined by the relation between the support normal and the configured up
vector. A maximum walkable-slope threshold is sufficient for the first game.

The first slice does not need:

- stair stepping;
- step height;
- moving-platform velocity inheritance;
- ledge mantle/climb;
- arbitrary automatic steep-slope acceleration.

Those are useful character-controller features, but they are not prerequisites for the selected
arena.

## Gravity and Jump Ownership

Putting jump entirely inside physics would make a gameplay rule part of reusable physical
semantics. Putting gravity entirely inside the game would duplicate a basic physical response
across every future character consumer.

The selected split is:

```text
game
  admits jump and changes vertical velocity

physics
  integrates gravity and resolves resulting motion/contact
```

This also fits later prediction: the command application applies the same game jump rule and then
calls the same physics operation.

## Initial Overlap

The current SDF sweep reports a hit when the starting point is already within the requested sphere
radius.

The first physics implementation should not silently turn that into a general depenetration
algorithm. It should report an invalid/overlapping start condition and leave spawn/restart placement
to the game/world bootstrap path.

This keeps the first solver bounded and makes bad spawn data visible.

## Prediction and Replay Compatibility

Current Engine networking already constrains the design.

`InputDriver::apply_input` accepts:

```rust
world
tick
input batch
```

and prediction reconciliation directly calls that operation for retained historical ticks.

Therefore the character motion used by player commands cannot require that an ordinary
`FixedUpdate` system happened exactly once around it. The game command application must be able
to invoke character motion explicitly for the target simulation tick.

The physics operation itself should receive explicit fixed-step duration and physical/query state.
It must not derive behavior from render-frame count, presentation pose, or transport state.

## Determinism Claim

Current Runenwerk simulation architecture provides deterministic vocabulary and replay mechanisms,
but this investigation found no accepted authority promising arbitrary cross-hardware bitwise
identity for all floating-point physics.

The first physics contract therefore makes the narrower, testable claim:

```text
same implementation
+ same initial physical state
+ same fixed step
+ same character config
+ same game movement input
+ same SDF collision payloads
= same character result and diagnostics
```

A stronger numeric determinism class requires separate simulation-owner evidence.

## External Mechanism Comparison

Rapier's current Rust character-controller documentation is a useful pressure test, not an API
template.

It describes a kinematic controller as behavior that resolves a desired trajectory through repeated
ray/shape casts and move-and-slide logic, with optional slope, stair, small-obstacle, and
moving-platform handling. It also keeps controller behavior conceptually separate from
rigid-body/collider handles.

Useful mechanism:

```text
desired movement
  -> shape query
  -> contact normal
  -> corrected movement / slide
```

Runenwerk adaptation:

```text
game movement intent
  -> world_sdf authoritative SphereSweep
  -> domain/physics bounded character response
  -> physical state result
```

Foreign assumptions not adopted:

- Rapier body/collider handles;
- Rapier ECS components;
- Rapier query filters as Runenwerk layer policy;
- built-in stair/platform behavior;
- a requirement to instantiate a general physics world.

References:

- <https://rapier.rs/docs/user_guides/rust/character_controller/>
- <https://rapier.rs/docs/user_guides/rust/scene_queries_shape_casting/>

The result is that no external solver is required for the first character slice.

## Diagnostic Contract

The first implementation needs inspectable facts for:

- requested displacement;
- admitted displacement;
- readiness/missing chunk;
- hit position;
- hit normal;
- slide count;
- grounded/support state;
- slope rejection;
- invalid initial overlap/config.

These diagnostics are domain/runtime facts. A future Editor or renderer debug surface may visualize
them without owning them.

## Implementation Test Matrix

A future bounded P1 issue should require at least:

| Case | Required result |
| --- | --- |
| clear motion | full displacement admitted |
| wall | movement stops before penetration |
| oblique wall | residual movement slides along tangent |
| floor | support classified grounded |
| walkable slope | support accepted |
| steep slope | support rejected |
| airborne | gravity changes physical velocity/motion |
| jump | game-applied vertical velocity leaves support |
| missing first sweep | no movement committed |
| missing later slide/support sweep | no partial movement committed |
| initial overlap | explicit failure/diagnostic |
| identical replay | identical result/diagnostic sequence |
| headless use | no Render/Scene/UI/Net dependency |

## Deferred Capability Matrix

| Capability | P1 |
| --- | --- |
| SDF sphere sweep | required |
| move-and-slide | required |
| grounding/support | required |
| slope threshold | required |
| gravity | required |
| readiness diagnostics | required |
| capsule character | deferred |
| stairs/step climb | deferred |
| moving platforms | deferred |
| collision layers/masks | deferred |
| triggers | deferred |
| physics materials | deferred |
| rigid bodies | deferred |
| joints/constraints | deferred |
| Rapier/other solver adapter | deferred |
| Editor physics authoring | deferred |
| physics-produced world mutation | deferred |

## Decision

P0 selects:

1. `world_sdf` remains collision-query truth.
2. `domain/physics` is the reusable character/contact-response owner.
3. The first implemented shape is a sphere backed directly by authoritative `SphereSweep`.
4. The first algorithm is bounded move-and-slide plus support/slope classification.
5. Missing payload fails closed and prevents partial tick movement commit.
6. Gravity is physics policy; jump admission/impulse is game policy.
7. Initial overlap is explicit failure, not automatic depenetration.
8. No general rigid-body solver or external backend is activated for P1.
9. Physics execution is explicit/replayable and presentation-independent.
10. The canonical owner contract is `docs-site/src/content/docs/domain/physics/README.md`.

## Next Gate

After this investigation is accepted, P1 is still blocked until G1 has established the maintained
game's real command-application consumer.

At that point #946 may derive one implementation issue bounded to:

```text
domain/physics first crate/surface
+ SDF spherical character response
+ focused tests
+ smallest Engine/game integration needed to call it
```

It must not use P1 to activate the rest of Physics Track E.
