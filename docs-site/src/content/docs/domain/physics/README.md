---
title: Physics Domain
description: Accepted ownership and first-slice contract for Runenwerk physics semantics over world-SDF collision truth.
status: accepted
owner: physics
layer: domain
canonical: true
last_reviewed: 2026-09-25
publication: primary
related_docs:
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../world-sdf/README.md
  - ../simulation/README.md
  - ../../engine/reference/plugins/fixed-step/architecture.md
  - ../../engine/reference/plugins/simulation/architecture.md
---

# Physics Domain

Runenwerk reserves `domain/physics` as the owner of reusable physics semantics that are not
signed-distance-field query truth, runtime scheduling, renderer state, or game-specific movement
feel.

No `domain/physics` crate is implemented yet. This document establishes the owner contract and
the deliberately narrow first implementation boundary required by the maintained small-game gold
path.

## Ownership

The boundary is:

```text
world_sdf
  authoritative SDF collision availability and query facts
        |
        v
domain/physics
  reusable body / character / contact response semantics
        |
        v
Engine integration
  fixed-step scheduling, resource adaptation, optional concrete solver adapters
        |
        v
game/application
  desired movement, jump policy, tuning, and gameplay consequences
```

### `domain/world_sdf`

`world_sdf` remains authoritative for:

- formed SDF chunk/page/brick payloads;
- collision-payload readiness;
- `SphereSweep`;
- `CollisionHit`;
- signed-distance sampling;
- `CollisionSweepOutcome::{MissingPayload, Hit, Clear}`;
- hit position and SDF-derived contact normal facts.

Physics must not copy SDF storage or redefine missing payload as empty space.

### `domain/physics`

The future physics domain owns reusable semantics for:

- physical body and collider meaning when those capabilities are activated;
- kinematic-character motion and contact-response rules;
- grounded/support classification;
- slope acceptance;
- reusable collision/contact outcomes;
- physics materials, layers, masks, triggers, and constraints when a maintained consumer activates
  them;
- physics-domain diagnostics and deterministic inspection values.

The first implementation is intentionally much smaller than this long-term ownership envelope.

### Engine

Engine integration owns:

- fixed-step scheduling of accepted physics operations;
- adaptation from Engine world resources to domain inputs;
- concrete external solver/backend adapters when one is actually required;
- runtime diagnostics publication and optional debug rendering adapters.

An external solver must not become the public semantic owner merely because Engine uses it.

### Game/application

The maintained game owns:

- input-to-intent mapping;
- movement speed and acceleration feel;
- jump eligibility and jump impulse policy;
- camera-relative movement policy;
- game-specific reaction to blocked/readiness outcomes;
- health, hazards, interaction, objective, and other gameplay consequences.

The physics domain must not become a gameplay framework.

## First Implementation Contract

The first implementation after the game command spine exists is one **SDF-first kinematic
character motion** slice.

It is not a general physics engine.

### Character shape

The first supported character collision shape is a sphere.

This is deliberate: current `world_sdf` authoritative collision truth already exposes a
`SphereSweep`. A first implementation must use the query semantics that actually exist rather
than present a capsule or arbitrary-shape API backed by weaker approximations.

The maintained first arena must therefore use an actor representation whose authoritative collision
volume is acceptable as a sphere.

Capsule, compound, primitive-set, and foreign-mesh character colliders are deferred until a
maintained consumer proves the need and the collision owner can supply truthful query semantics.

### Character state

The first reusable character-motion state needs only the physical facts required by the game slice:

- current physical position;
- current physical velocity;
- grounded/support state;
- current support normal when grounded.

Rotation dynamics, angular velocity, mass, impulse accumulation, sleeping, rigid-body islands, and
constraint graphs are not required for the first character contract.

### Motion request

One character-motion evaluation consumes explicit simulation inputs rather than wall-clock or
presentation state:

- fixed-step duration;
- current physical state;
- desired planar movement supplied by the game;
- gravity configuration;
- character radius;
- up vector;
- maximum walkable-slope policy;
- the world/partition/query context needed to issue authoritative SDF sweeps.

Jump is game policy. The game changes the character's vertical velocity when its own jump rules
admit a jump; physics then resolves the resulting motion against collision truth.

### Motion result

The result must make readiness and collision outcomes explicit. At minimum the semantic result must
be able to distinguish:

- movement completed;
- movement completed with one or more contacts/slides;
- blocked by unavailable collision payload;
- invalid/unsupported starting state.

Successful results expose:

- resulting physical position;
- resulting physical velocity;
- grounded/support state;
- support normal when present;
- bounded contact facts sufficient for diagnostics.

Exact Rust type names and storage layout remain implementation decisions. The behavior above is the
contract.

## Movement Law

The first character solver follows a bounded move-and-slide model:

1. validate finite, bounded input/configuration;
2. form the desired tick displacement from current velocity, game-supplied planar intent, gravity,
   and fixed-step duration;
3. query `CollisionQueryService::sweep_sphere_authoritative` for the intended movement;
4. on `Clear`, admit the movement;
5. on `Hit`, move only to the admissible contact boundary and project the remaining movement onto
   the contact tangent;
6. repeat the residual movement for a small fixed maximum number of slide iterations;
7. perform a bounded downward support probe from the candidate final position;
8. classify support as walkable only when its normal satisfies the configured up-vector/slope rule;
9. commit the resulting state only when all collision payloads required by the tick were ready.

The exact contact skin/offset, iteration cap, epsilon, and velocity projection rules must be
ratified by focused tests rather than copied from a foreign controller.

### Atomic readiness

`MissingPayload` is not a collision miss.

If any authoritative query required to resolve the tick returns `MissingPayload`, the first
implementation must fail closed and **must not partially commit character movement for that tick**.
This prevents a character from moving into unknown world state because an earlier sub-sweep happened
to be ready.

The result must preserve enough readiness information to identify the missing `ChunkId`.

### Initial overlap

The first implementation does not need a general depenetration solver.

A character starting in an invalid overlap is a typed failure/diagnostic condition. Spawn and
restart placement for the maintained game must use valid collision-ready positions.

A later consumer may activate bounded depenetration as a separate capability.

### Slopes

Walkable support is classified from the contact normal and configured up vector.

The first implementation needs:

- a maximum walkable slope;
- projection/slide behavior for obstacle contacts;
- rejection of support that is too steep.

Separate automatic stair stepping, step height, ledge climbing, moving-platform inheritance, and
arbitrary slope-slide acceleration are deferred.

### Gravity and jumping

Physics owns generic gravity integration because it is physical response shared by local,
authoritative, and predicted execution.

The game owns when a jump is allowed and the jump impulse/vertical-velocity change because those are
gameplay feel and rules.

This keeps replay/prediction on one physical motion operation without moving game-specific jump
policy into the reusable physics domain.

## Determinism and Replay

The first implementation must be callable from the same explicit tick-command application path used
by local authority and later client prediction/reconciliation.

Required determinism claim:

> For the same accepted implementation, initial physical state, fixed-step duration, character
> configuration, game movement input, and identical collision-ready SDF payloads, the character
> operation produces the same domain result and diagnostics.

This contract does not claim cross-architecture bitwise identity for arbitrary floating-point
hardware. Any stronger determinism class must be established by the simulation owner and proven
separately.

Physics must not read:

- render transforms;
- presentation interpolation state;
- native frame delta;
- wall-clock time;
- UI state;
- network transport state.

## External Solver Boundary

The first SDF-character implementation does **not** require Rapier or another general rigid-body
solver.

Current Rapier documentation remains useful as a mechanism comparison: its kinematic character
controller resolves a desired translation using repeated ray/shape queries, move-and-slide,
ground/slope policy, and optional stairs/platform behavior. That supports the architectural
separation between character behavior and collision-query truth, but Runenwerk does not adopt
Rapier handles, collider types, ECS components, or controller options as domain semantics.

Reference:

- <https://rapier.rs/docs/user_guides/rust/character_controller/>
- <https://rapier.rs/docs/user_guides/rust/scene_queries_shape_casting/>

A future rigid-body or broader collider consumer may justify an Engine-owned Rapier adapter or a
different backend. That decision is not made by this first character slice.

## Diagnostics

The first physics implementation must expose structured, deterministic inspection facts for:

- collision readiness and missing payload;
- requested and admitted displacement;
- contact hit position and normal;
- slide iteration count;
- grounded/support state;
- rejected steep support;
- invalid initial overlap/configuration.

Debug drawing is a consumer of these facts. Debug rendering is not physics semantic authority.

## First-Slice Validation

The first implementation must include focused tests covering at least:

- clear-space movement;
- stop against a wall;
- tangent slide along a wall;
- floor support/grounding;
- walkable slope acceptance;
- too-steep support rejection;
- gravity while airborne;
- game-applied jump velocity leaving grounded support;
- missing payload producing no partial movement commit;
- invalid initial overlap failing explicitly;
- fixed iteration bounds;
- repeated evaluation from identical state/query payload producing identical output;
- no dependency on Scene, Render, UI, or Net state.

The maintained game must then prove the same motion operation through its local command-application
path before multiplayer prediction is activated.

## Deferred Physics Breadth

The accepted long-term physics owner may later include the following, but none is authorized by the
first character slice:

- general rigid-body dynamics;
- arbitrary primitive/compound collider authoring;
- capsule character queries;
- collision layers and masks;
- triggers;
- physics materials;
- constraints and joints;
- moving-platform inheritance;
- stair/step climbing;
- sleeping/island management;
- foreign mesh/reference colliders;
- physics-produced world mutation;
- Editor physics authoring/debug providers.

Each item requires current consumer evidence and a bounded issue.

## Activation Gate

The first implementation may be opened only after:

1. the shared game gold-path architecture is accepted;
2. the maintained game command/authority spine is accepted and supplies the real fixed-tick
   consumer;
3. current `world_sdf` collision semantics still match this contract;
4. active writers are reconciled;
5. the implementation issue is narrowed to the SDF-character slice above.

The implementation must not expand into the rest of the physics roadmap merely because
`domain/physics` becomes a workspace crate.
