---
title: Physics Domain
description: Accepted Physics domain ownership, first SDF-character contract, and general-purpose backend integration boundaries.
status: accepted
owner: physics
layer: domain
canonical: true
last_reviewed: 2026-10-16
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

`domain/physics` now implements the deliberately narrow first SDF-character slice required by the
maintained small-game gold path. This document is the owner contract for that implementation and
for later physics capabilities only when separately activated.

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

The physics domain owns reusable semantics for:

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

The implemented first slice is one **SDF-first kinematic character motion** contract.

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

The later general-purpose Physics architecture selects Rapier as its first concrete solver, with a
backend-neutral domain contract and separate adapter. The selection is governed by the second-stage
principles below; it does not retroactively change the accepted P1 character behavior.

## General-Purpose Physics Direction

The general-purpose Physics direction is **accepted architecture**, not a statement that its
solver adapter, plugin, broader public API, or gameplay migration is already implemented. It
preserves the first SDF-character contract above until an independently validated cutover.

### Semantic authority and dependency direction

`domain/physics` is the **one reusable semantic owner** of physical-scene, body/collider,
command, physical-result, and supported-contact meaning. One selected backend session owns the
mutable realization of a physical scene: body and joint state, contacts, solver caches, and
backend-local handles. A second runtime/ECS/Game representation may request or observe that
state, but must not independently advance the same authoritative body.

**Rapier is the selected initial general-purpose solver.** Its concrete adapter belongs in a
separate dependency boundary so vendor state, algorithms, precision settings, feature choices,
and revision changes do not define the reusable Physics contract. The Engine consumes the
Physics-owned semantic interface and receives the selected backend from explicit application
composition; the Engine must not compile against Rapier merely to expose an ordinary Physics
plugin. Rapier types and handles must not leak into game authoring, durable content, ECS
public observations, or the neutral API. Exact Rust traits, registration mechanism, package
layout, feature flags, and backend-version pin require executable qualification rather than
being prescribed by this document.

The Physics semantic API must remain usable for a **source-independent ordinary rigid-body
scene** containing conventional primitive colliders. It must not require a WorldSDF source,
rendering, a native window, or a separate App runtime. The currently accepted P1 evaluator
still depends directly on `world_sdf`; this is an existing **transitive compile dependency**,
not evidence that the intended independent dependency closure is already achieved. A clean
P1-specific boundary cutover is required if that compilation isolation becomes an activated
contract; a cosmetic wrapper, shadow physics authority, or permanent compatibility facade is
not an acceptable substitute.

### Execution, identity, and time

Engine integrates Physics into the **existing opt-in App/plugin and fixed-step execution**
architecture. Physics must not introduce a second global scheduler, application clock, or
independent gameplay command source. A Physics session has explicit lifecycle and one
backend-owned mutable solver state, with stable Physics identities mapped to private
generation-checked backend handles. Authorable descriptions remain distinct from live solver
state; ECS and render transformations derived from a committed Physics result are not solver
authority.

The fixed-step integration must distinguish input/attempted simulation tick from the
**last committed Physics result**, perform required command/source preflight, advance
free-running dynamic bodies even when no game command exists, and expose a defined
fault/recovery state. The existing Simulation owner retains tick and profile semantics;
Physics must not silently redefine `SimulationTick` or assert a global transaction over
independently owned World, Game, and Physics state. A backend step that has mutated internal
state and subsequently reports a fault must not be treated as an unperformed step. Abort,
quarantine, reset, and restore policies require observable, tested contracts.

### World collision and source validity

Ordinary primitive/mesh colliders can be authoritative in a Physics scene without SDF.
For **WorldSDF-derived** geometry, the World owner retains source identity, current revision,
coverage, readiness, metric and approximation facts. A World-to-Physics bridge must admit an
explicitly compatible geometry realization for the tick and retain its provenance through
replacement and retirement. A backend ray or shape cast returning no hit among installed
colliders does not certify continuous clearance of an incomplete, stale, or approximated
SDF source. Missing or insufficiently qualified coverage must remain distinguishable from
valid clear space.

Physics may consume qualified World collision products but must not duplicate RunenSDF
mathematics, WorldSDF source authority, WorldOps mutation policy, or a second numerical
world-process solver. Source-collision correctness and radius-aware swept clearance are
independent WorldSDF acceptance questions; selecting Rapier does not resolve them.

### Capabilities, predictability, and evolution

Backend-neutrality means stable **integration semantics**, not identical numerical dynamics
between solvers. Friction, solver contacts, CCD, character motion, sleeping, constraints,
precision, and ordering may differ. Unsupported capabilities or simulation profiles must be
reported explicitly instead of silently emulated or relabelled as equivalent. Application
composition selects a backend; runtime hot switching and cross-backend checkpoint transfer
are not accepted requirements.

Simulation and Replay retain their profile, history, snapshot, prediction, and rollback
authority. A pinned backend configuration, stable identity mapping, and verified capture
and restore path are prerequisites for corresponding Physics guarantees; the existing
Scene replay path is not such a proof. Rendering and editor diagnostics observe accepted
Physics facts without becoming collision or solver truth.

A **bounded, headless ordinary-primitive consumer proof** is required before
stabilizing concrete API traits, adapter/resource constraints, numerical error budgets,
or a clean dependency cutover. A generic backend trait, broad joint/controller surface,
general Engine Physics plugin, production WorldSDF realization bridge, and existing-game
actor migration are **not** implementation-ratified merely by this architectural direction.

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

Beyond the separately authorized first ordinary headless Rapier primitive qualification, the
following capabilities remain unimplemented and require bounded consumer evidence:

- general rigid-body breadth beyond the qualified primitive witness;
- arbitrary primitive/compound collider authoring beyond the qualification fixture;
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

## First-Slice Activation Gate

The first implementation may be opened only after:

1. the shared game gold-path architecture is accepted;
2. the maintained game command/authority spine is accepted and supplies the real fixed-tick
   consumer;
3. current `world_sdf` collision semantics still match this contract;
4. active writers are reconciled;
5. the implementation issue is narrowed to the SDF-character slice above.

The implementation must not expand into the rest of the physics roadmap merely because
`domain/physics` becomes a workspace crate.
