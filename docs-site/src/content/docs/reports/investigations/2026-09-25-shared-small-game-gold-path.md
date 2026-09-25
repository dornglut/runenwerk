---
title: Shared Single-Player and Multiplayer Small-Game Gold Path Investigation
description: Current-source investigation selecting the smallest Runenwerk game architecture and implementation sequence that can support local play, listen-server play, remote clients, and headless authority without parallel gameplay implementations.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_docs:
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../net/multiplayer-replication-implementation-roadmap.md
  - ../../design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ./2026-08-12-application-composition-and-networking-ergonomics.md
---

# Shared Single-Player and Multiplayer Small-Game Gold Path Investigation

## Purpose

Issue #938 asks a product-facing question rather than another framework-completeness question:

> What is the smallest truthful Runenwerk path to one maintained small game that can run as local
> single-player, listen-server/host, remote multiplayer client, and headless dedicated authority
> without creating separate gameplay implementations?

This report records the current-source answer and the bounded implementation sequence derived from
it.

The central conclusion is:

```text
one maintained game
one App-world gameplay model
one tick-command vocabulary
one deterministic command-application path

local single-player
listen-server / host
remote predicted client
headless dedicated authority

same gameplay semantics
different authority, routing, and presentation
```

Runenwerk already contains enough fixed-step, ECS, world-query, RunenNet, RunenInput, UI, and
renderer substrate to make that direction credible. It does **not** yet contain one maintained
game product that integrates those capabilities, and several integration seams remain missing or
predecessor-shaped.

This report does not implement those seams and does not promote speculative framework breadth.

## Evidence baseline

The final pre-publication source census used accepted Runenwerk main:

```text
f3a5d75448db614dbe770e7511d23e599280da4d
```

Pinned standalone framework revisions relevant to the investigation are:

```text
RunenECS   84fd396f6c6b6fecb7df8cb5f71a7e74770d156a
RunenNet   e8dce38b2578eb3559356968beb2351cc03c6758
RunenInput 2751e19fa42255b86e786e7cd837198c917b7b25
RunenGPU   789b430fdefeda89bfe59de86d548618b8f8ab9a
RunenSpatial 2bb6b2ed89894aefaa39bf0e39d296b703916821
```

Open writers at the final source guard were:

- #941 — Simulation/Replay rehome;
- #940 — normalized input-trace replay;
- #931 — RunenRender temporal-resolution work;
- #915 — Editor shell workspace-state retirement.

The report is documentation-only and does not take ownership of those implementation paths. The
simulation/replay rehome is especially relevant because it may move physical source paths while
retaining the current crate-facing simulation contracts. Any implementation derived from this report
must re-resolve the accepted state after that writer lands or is abandoned.

Important current sources inspected include:

```text
engine/src/plugins/mod.rs
engine/src/plugins/fixed_step.rs
engine/src/plugins/simulation.rs
engine/src/plugins/input/**
engine/src/runtime/schedules.rs
engine/src/runtime/fixed_step_executor.rs

engine/src/plugins/net/**
engine/tests/network_plugins/**
engine/tests/network_plugins/replicated_view_r0.rs

engine/src/plugins/scene/**
engine/src/plugins/world/**
domain/world_sdf/src/collision.rs

engine/src/plugins/render/**
engine/src/plugins/ui/**
apps/runenwerk_editor/src/runtime/systems/frame_submit.rs

domain/editor/editor_persistence/**
domain/editor/editor_preview/src/product.rs
apps/runenwerk_runtime_preview/**

docs-site/src/content/docs/adr/accepted/0019-batteries-included-application-composition.md
docs-site/src/content/docs/net/multiplayer-replication-implementation-roadmap.md
docs-site/src/content/docs/design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
docs-site/src/content/docs/design/active/editor-procedural-content-and-simulation-workflow-plan.md
```

Search was used only for discovery. Absence conclusions below were checked against the relevant
owner/source surfaces rather than inferred from an empty search alone.

# 1. Current capability reality

## 1.1 App and fixed-step simulation are sufficient foundations

The ordinary runtime baseline is intentionally narrower than a complete game stack.

Current `default_plugins()` installs:

```text
TimePlugin
FixedStepPlugin
SimulationPlugin
ReplayPlugin
InputFinalizePlugin
DiagnosticsPlugin
```

Scene, Render, UI, World, and Net remain separately selected capabilities.

The frame schedule is:

```text
PreUpdate
-> FixedStepBegin -> FixedUpdate      zero or more times
-> Update
-> RenderPrepare
-> RenderSubmit
-> FrameEnd
```

`SimulationPlugin` advances `SimulationTick` at `FixedStepBegin`. Fixed cadence and simulation
identity remain separate capabilities.

This is enough to establish a game-authority spine without creating a second runtime or scheduler.

## 1.2 The existing execution-role vocabulary already points toward one game

Current simulation profile vocabulary includes:

```text
LocalSinglePlayer
DeterministicLockstep
RollbackSession
DedicatedAuthority
HighThroughputAuthority
```

and current authority vocabulary includes:

```text
Local
Client
Server
Peer
```

`NetPlugin<TDriver>` currently composes `Client`, `Server`, and `Host` roles. Host installs
both client and server integration into one App and selects `AuthorityRole::Peer`.

This does not prove the final game topology by itself, but it makes a separate
`SinglePlayerGame`/ `MultiplayerGame` simulation split unnecessary and directionally wrong.

The gold path should therefore treat role as **routing and authority policy over one gameplay
model**.

## 1.3 RunenECS should hold live gameplay state

Standalone RunenECS already owns reusable:

- entities and components;
- resources;
- queries;
- deferred structural mutation;
- schedules, ordering, and system sets;
- deterministic serial execution and accepted parallel realization.

It explicitly does not own gameplay-event semantics, networking protocol, world meaning, rendering,
or application lifecycle.

The maintained game should therefore keep live gameplay entities/components in the App's RunenECS
World and keep game meaning in the game/product layer.

No new generic gameplay event bus or second gameplay state container is justified.

# 2. ScenePlugin is not the future gameplay-world authority

Current Scene runtime remains predecessor-shaped for game use.

`SceneManager` creates a `WorldSceneRuntime` containing its own nested `runen_ecs::World`.
The current world runtime contains:

- a frame counter;
- a debug position/velocity pair;
- scene input projections;
- stub notification behavior.

The active world-scene system performs debug motion and stub notifications. Existing
`player_move_x` / `player_move_y` state is synchronized from actions but does not drive a real
player entity.

The current Scene IDs are likewise stub/product-shell shaped:

```text
GameplayStub
HubStub
ConsoleUi
HudUi
InventoryUi
```

### Decision

The new maintained game must **not** deepen `ScenePlugin`'s nested World into general gameplay
authority.

Use:

```text
App World
  -> live gameplay state and simulation authority

Scene integration
  -> only the presentation/lifecycle compatibility responsibilities actually required
     by current Engine integration until those seams are separately normalized
```

This avoids introducing two live ECS worlds for one game and avoids moving product semantics into a
legacy scene stub.

# 3. Input requires an explicit frame-to-tick boundary

RunenInput correctly stops at physical/device observation and confirmed state. Runenwerk owns
product actions and bindings.

Current `InputFinalizePlugin` projects `ActionState` during `PreUpdate` in `CoreSet::Input`.
`ActionState` has two materially different kinds of information:

- held state such as `action_down`;
- frame-local edges such as `action_pressed`.

Frame-local action edges are cleared at `FrameEnd`.

Because the fixed-step loop may run zero, one, or multiple times between two rendered frames,
reading a frame-local edge directly from arbitrary fixed-step gameplay code is not a sufficient
long-term input contract.

### Decision

The game needs an explicit product-owned input accumulator:

```text
RunenInput confirmed state
       |
ActionState projection in PreUpdate
       |
Game input accumulator
  - latest continuous intent
  - latched one-shot edges
       |
zero or more fixed ticks
       |
one tick-targeted GameCommand batch per participant/tick
```

The collector belongs after `CoreSet::Input` and before the fixed-step loop.

Required behavior:

- if no fixed tick runs, a one-shot edge remains latched for the next tick;
- if multiple fixed ticks run, continuous movement can be sampled for each tick;
- a one-shot action is consumed only once unless a later physical/action edge occurs;
- command formation is independent of whether the command will be applied locally, predicted, or
  sent to authority.

This is the first required game-specific contract.

# 4. Prediction makes the command-application function more important than a system name

Current RunenNet-backed Engine prediction does more than buffer input.

For each fixed tick, `prediction_step_system<TDriver>`:

1. obtains the current `SimulationTick`;
2. drains authority input for that tick;
3. calls `TDriver::take_local_input`;
4. stages the local batch for the same tick;
5. on a Client, admits the encoded batch to RunenNet prediction before applying it;
6. calls `TDriver::apply_input(world, tick, commands)`;
7. on Server/Peer authority, combines legal authority input with local input and applies the same
   driver path.

RunenNet reconciliation later replays retained predicted input by calling
`TDriver::apply_input(world, target_tick, commands)` directly.

The Net prediction set currently runs after an optional `CoreSet::Simulation`, and authority
replication runs after prediction.

### Consequence

A predicted gameplay effect cannot depend solely on "whatever ordinary FixedUpdate systems happened
to run once."

Any state that must replay correctly must be expressible through one deterministic function
callable with:

```text
World
SimulationTick
participant/tick command batch
```

### Decision

The game should establish one app-owned command-application operation, directionally:

```text
apply_game_commands(world, tick, commands)
```

The exact public/internal type names are not selected here.

That operation is used by:

```text
Local single-player
  local fixed-step command executor
      -> apply_game_commands

Host / dedicated authority
  admitted local/remote command path
      -> apply_game_commands

Remote client
  RunenNet-admitted prediction
      -> apply_game_commands

Prediction reconciliation
  retained target-tick replay
      -> apply_game_commands
```

This does **not** require every game system to move into one function. It means the predicted subset
of gameplay—starting with controlled-character motion—must be replayable from the same tick command
contract.

Unpredicted authority-only simulation may remain ordinary fixed-step systems when that distinction
is explicit.

# 5. Execution-role matrix

The first maintained game should preserve this role model unless implementation evidence falsifies
it.

| Role | App World meaning | Local command path | Remote command path | Presentation |
| --- | --- | --- | --- | --- |
| Local single-player | authoritative gameplay state | direct local tick admission/application | none | local frame interpolation |
| Host/listen server | authoritative gameplay state | local authority application, not client prediction | RunenNet authority admission then same application | local presentation plus remote replicas as needed |
| Remote client | predicted local + replicated/confirmed remote state, not world authority | RunenNet prediction admission then same application | not authority | predicted-local smoothing + remote interpolation |
| Dedicated server | authoritative gameplay state | optional server-local/admin commands only | RunenNet authority admission then same application | none |

Current `WorldPlugin` already maps:

```text
Client -> ReadOnly
Local / Server / Peer -> Writable
```

for world mutation, which aligns with this authority model.

## Host disposition

Current Host is a one-App/one-World client+server composition. Existing tests specifically prove that
peer-host local input is applied as authority and is not retained as client prediction.

No current game evidence proves that this composition must be replaced.

### Decision

**Preserve current Host composition provisionally.**

The maintained game must pressure-test it with real movement, remote participant state, replication,
and presentation. Open a Host-isolation redesign only if the real consumer demonstrates an actual
authority/replica collision or lifecycle defect.

Do not manufacture separate host client/server worlds in advance.

# 6. First gameplay physics boundary

Runenwerk does not need collision detection from zero.

`domain/world_sdf` already provides strict query truth:

```text
CollisionQueryService
SphereSweep
CollisionHit
CollisionReadiness
CollisionSweepOutcome
  MissingPayload
  Hit
  Clear
```

The authoritative sweep checks required chunk payload availability and can fail closed with
`MissingPayload`.

The active procedural/simulation design separately records that a general `domain/physics` does
not yet exist and directs future physics toward body/collider/character contracts plus an
Engine-owned solver adapter.

### Decision

The first playable game does **not** require a general rigid-body solver.

The first physics capability should be a bounded SDF-first kinematic character-motion contract with
at least:

- character shape/radius or another first bounded shape;
- desired translation/velocity;
- gravity;
- sweep-based obstacle stopping;
- grounding/support result;
- surface normal;
- slide behavior;
- slope policy;
- missing-payload/readiness propagation;
- deterministic result suitable for tick-command replay.

Step climbing may be deferred unless the selected arena geometry demonstrates it is required.

Game-specific acceleration, run speed, jump tuning, air control, and similar "feel" remain
game-owned configuration/policy.

### Owner split

```text
domain/world_sdf
  strict collision/query truth

future domain/physics
  reusable body/collider/character/contact semantic contracts

engine
  concrete runtime/solver adapter where one is required

game
  movement feel and gameplay consequences
```

### External pressure test: Rapier

Rapier's current Rust character-controller documentation models character control as desired
translation corrected by shape/ray queries and supports obstacle stopping, slope sliding, stairs,
small obstacles, and moving platforms. It also explicitly notes that player-character control is
often game-specific.

Source:
https://rapier.rs/docs/user_guides/rust/character_controller/

Runenwerk adaptation:

- adopt the useful mechanism distinction between desired motion and query-corrected motion;
- keep strict SDF query authority in `world_sdf`;
- keep public physics semantics Runenwerk-owned;
- evaluate a Rapier adapter later for rigid-body/general-solver demand.

Explicit non-adoption:

- no Rapier type becomes the Runenwerk physics contract in this first slice;
- no Rapier dependency is selected by this report.

# 7. Simulation pose and presentation pose must be separate

A proper small game cannot render fixed-tick physical position directly and expect smooth motion.

The fixed loop may execute zero or multiple ticks per rendered frame. The game therefore needs a
presentation boundary such as:

```text
PreviousPhysicalPose
CurrentPhysicalPose
        |
        | frame alpha / overstep fraction
        v
PresentationPose
```

The presentation pose is derived and must not become the next authoritative simulation input.

This principle is useful for both local and networked play:

```text
local frame interpolation
  previous fixed pose -> current fixed pose

client predicted presentation
  predicted fixed pose -> frame-smoothed presentation

remote network interpolation
  confirmed snapshot A -> confirmed snapshot B on a buffered timeline
```

These are related presentation mechanisms but not the same timeline or authority.

### External pressure tests

Bevy's current fixed-timestep movement example explicitly accumulates input before the fixed loop,
runs physics in fixed updates, and interpolates a separate visual transform between previous and
current physical positions after the fixed loop.

Source:
https://bevy.org/examples/movement/physics-in-fixed-timestep/

Lightyear's current documentation separately distinguishes:

- client prediction for locally controlled entities;
- buffered snapshot interpolation for remote entities;
- frame interpolation between local fixed ticks.

Sources:

- https://cbournhonesque.github.io/lightyear/book/tutorial/advanced_systems.html
- https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html

Runenwerk adaptation:

- establish an owner-correct physical/presentation split;
- reuse the same movement logic for authority and prediction;
- add remote snapshot interpolation only when the maintained multiplayer consumer exists.

Explicit non-adoption:

- no Bevy schedule/component ontology;
- no Lightyear replication API or Bevy-specific entity model;
- RunenNet remains the reusable networking semantic authority.

# 8. Rendering is not yet a generic gameplay-ECS path

Current Runenwerk rendering has a strong prepared-contribution boundary, but the maintained game
consumer is missing.

Important current facts:

- `RenderPlugin` initializes prepared draw/material/world/UI contribution resources;
- Render submit consumes prepared state rather than discovering live ECS truth;
- Editor multi-entity rendering is deliberately app-owned extraction from Editor scene reality into
  an `EditorViewportSceneRenderPacket`;
- current `frame_render_prepare_system` still reads `SceneResource.manager` for primary frame
  size/route context and clears the prepared frame when no manager exists;
- `RenderPlugin` may provision an empty `SceneResource`, but only `ScenePlugin` activates the
  Scene manager.

Therefore:

```text
RunenECS gameplay entity
  -/-> automatic current rendered game object
```

There is no accepted generic App-World ECS-to-render extraction path to assume.

### Decision

The maintained game should use a **game-owned presentation adapter** that projects gameplay truth
into existing prepared render contracts. The renderer must not gain live gameplay/ECS ownership.

Before a windowed game can avoid predecessor Scene coupling completely, one bounded integration
issue must also resolve the remaining Render frame-context dependence on a live Scene manager or
otherwise provide an owner-correct game frame context.

That repair is distinct from gameplay authority and should not be hidden inside physics.

# 9. UI can be used earlier than world-space UI attachment

Unlike game-world rendering, Engine UI already exposes an app-facing mount seam:

```text
UiPlugin
AppUiExt::mount_ui
app.ui().mount(...)
```

`UiPlugin` publishes evaluated UI frames through the prepared surface-frame submission path.

The current design still defers entity-attached/world-space UI.

### Decision

The first game HUD should be an ordinary screen-space app-mounted UI.

Do not make world-space UI attachment, Editor provider infrastructure, or Scene stub overlay state a
prerequisite for the first small game.

# 10. Authored project persistence is not yet standalone game bootstrap

Current Editor scene persistence is real:

```text
SceneFileV2
-> normalization
-> FormedScenePackageV2
-> Editor runtime application
```

but the concrete application target is `RunenwerkEditorRuntime`, not the ordinary gameplay App
World.

Current project persistence has reached `ProjectFileV3` and includes startup-document and asset
source/cache/catalog information.

Runtime Preview is also a real external process using RunenNet QUIC, but its current
`build_preview_app` installs only `default_plugins()`.

The preview product protocol distinguishes Scene, Material, Texture, Shader, UI, field/procgen and
world-SDF product kinds, but `RuntimeProductPayload` currently carries:

```text
Descriptor(RuntimeProductRef)
WorldSdf(WorldSdfPayloadPackage)
```

Only the world-SDF case contains that concrete payload package directly. A Scene product reference
does not by itself instantiate a playable game world.

### Decision

A standalone game/content-bootstrap seam is genuinely missing.

Do not:

- call Editor scene loading from the game;
- make `RunenwerkEditorRuntime` a gameplay dependency;
- make `ProjectFileV3` permanent runtime semantic authority merely because it has
  `startup_document_id`;
- claim Runtime Preview product descriptors already form a game.

The correct later flow is directionally:

```text
project/editor source
-> owner normalization/formation
-> ratified runtime-facing products
-> game startup/bootstrap
-> App World / world products / material products / UI products
```

The exact runtime package contract needs a separate bounded issue after the maintained game
consumer exists.

# 11. Multiplayer substrate is advanced, but the gameplay transport consumer is missing

The post-RN8 Engine/RunenNet path already covers substantial difficult semantics:

- compatibility/session membership;
- participant authority-input admission;
- client replication consistency/history/recovery;
- authority replication consistency/history/recovery;
- prediction and reconciliation lineage;
- ACK and real `DeliveryAcceptance`;
- bounded Engine staging;
- role-aware fixed-step integration.

Current low-level game integration still uses:

```text
ReplicationDriver
SnapshotApplyDriver
InputDriver
```

and Replicated View R0 currently proves explicit complete state-product formation, stable replicated
identity, deterministic ordering, exact byte accounting, and atomic activation as test evidence.

The active multiplayer roadmap explicitly withholds final ordinary Replicated View syntax until a
maintained consumer proves the missing contract.

### Concrete transport

Standalone `runen-net-quic` already exposes production:

```text
EndpointConfig
ProfileConfig
ClientEndpoint
ServerEndpoint
ProfileReadyConnection
Connection
```

with explicit finite resource policy and multi-connection endpoint capacity.

Runenwerk Runtime Preview proves one maintained QUIC consumer, but its protocol is Editor preview
control, not gameplay.

### Decision

Do not add a generic Engine QUIC runtime now.

After the maintained game has a local playable command/authority model, add gameplay QUIC as an
**app-owned maintained consumer** around `runen-net-quic` and the existing Engine Net semantic
boundary.

That real game then becomes the evidence required to advance #322's ordinary Replicated View path.

# 12. Selected first vertical slice

The first gold-path game should be deliberately small and system-rich rather than content-rich.

Recommended bounded slice:

```text
one small SDF arena
one or two player actors
third-person or simple free-look camera
move + jump
SDF collision / grounding
one interact action
one hazard that changes health
one objective that can be completed
HUD: health + objective state
win/lose state
restart
```

This slice is enough to pressure:

- frame-to-tick input;
- fixed simulation;
- replayable/predictable movement commands;
- SDF collision readiness;
- local presentation interpolation;
- game-owned render projection;
- HUD;
- authoritative health/objective state;
- replication;
- host + remote client;
- dedicated headless authority.

It intentionally does **not** require:

- enemy AI;
- skeletal animation;
- runtime prefabs;
- particles;
- scripting;
- gameplay graphs;
- quests/abilities framework;
- general rigid-body physics;
- matchmaking;
- account/backend services.

Simple SDF/procedural actors are preferable initially if they prevent those unrelated feature
families from becoming blockers.

# 13. Capability activation matrix

| Capability | First local playable slice | First multiplayer slice | Parallel after consumer exists | Deferred |
| --- | --- | --- | --- | --- |
| Game App + command authority spine | Required | Required |  |  |
| SDF kinematic character motion | Required | Required |  |  |
| General rigid-body solver |  |  |  | Until demonstrated |
| Local fixed/frame interpolation | Required | Required |  |  |
| Remote snapshot interpolation |  | Required |  |  |
| Game-owned render presentation adapter | Required | Required |  |  |
| Runtime-authored content bootstrap | Required for gold-path completion, not first command-spine issue | Required for full product path | May proceed with presentation work once owner boundary is fixed |  |
| Screen-space HUD through UiPlugin | Required | Required |  |  |
| Game audio |  |  | High-value after maintained game exists | Not a structural blocker |
| Gamepad/controller |  |  | High-value; activate in RunenInput first from real consumer evidence |  |
| Save/checkpoint persistence |  |  | High-value after authoritative game state exists |  |
| Material/PBR closure | Only semantics required by chosen visuals | Same | Existing #841/#842 remain owners | Full matrix not required |
| Runtime prefab instancing |  |  |  | Until repeated authored spawnable content requires it |
| Animation |  |  |  | Until chosen actor representation requires it |
| Gameplay graph |  |  |  | Until typed gameplay model demonstrates authoring pressure |
| Scripting |  |  |  | Until concrete game-authoring pressure |
| Particles/VFX |  |  |  | Cosmetic/post-core |
| Relevancy/interest |  | Not required for two-player small arena unless evidence proves otherwise | Later scale work |  |
| Matchmaking/lobbies/accounts |  |  |  | Separate backend-services concern |
| Export/package tooling | Not required for first integration proof | Headless/client binaries must launch normally | Productization later | Dedicated packaging redesign deferred |

## Audio note

Current Runenwerk has no concrete game-audio runtime/backend.

Kira 0.12.4 is a plausible later adapter candidate because its current API provides game-oriented
static/streaming playback, mixer tracks, clocks, and spatial audio:

https://docs.rs/kira/latest/kira/

This report does not select Kira. Audio needs an owner/backend investigation once the maintained game
provides the consumer.

## Gamepad note

Standalone RunenInput explicitly deferred gamepad/controller and generic axes until a maintained
consumer demonstrates real device/control/axis/lifecycle pressure.

The maintained game is the correct future activation evidence. Reusable controller semantics belong
in RunenInput first, followed by immutable-pin adoption in Runenwerk.

# 14. Ordered implementation sequence

The smallest coherent sequence is not the earlier subsystem shopping list.

## G1 — maintained local-game executable and tick-command authority spine

**First implementation candidate after this investigation is accepted.**

Expected bounded goal:

- create one maintained game application under `apps/`;
- keep live gameplay state in the App World;
- establish game-owned participant/player identity and minimal player state;
- collect product input after `CoreSet::Input` into a persistent frame-to-tick accumulator;
- form one tick-targeted game-command batch;
- implement one deterministic game command-application operation;
- execute it in local single-player fixed-step authority;
- prove the same operation is callable for an arbitrary retained target tick, so later Net
  `InputDriver::apply_input` does not require a second movement implementation;
- keep presentation, physics, networking, audio, content loading, and broad game mechanics out of
  this first cut.

Critical tests should cover:

- zero fixed ticks do not lose a one-shot action;
- multiple fixed ticks do not multiply a one-shot action;
- held movement remains available for each relevant tick;
- command application uses the supplied `SimulationTick`;
- identical initial state + identical tick-command sequence gives identical game state;
- the ordinary local path does not depend on NetPlugin;
- no Scene nested World becomes gameplay authority.

This slice creates the maintained consumer needed by the later tracks without pretending the game is
already visually complete.

## P0/P1 — SDF-first character physics

After G1 establishes the command application point:

1. create the required `domain/physics` owner/design slice required by the active procedural and
   simulation plan;
2. implement only the kinematic character-motion contracts needed by the arena;
3. integrate the current `world_sdf` strict sweep/readiness path;
4. call the same character movement from the G1 command application function.

P0 owner/design work can be prepared in parallel with later G1 work only when current writers and
changed paths are demonstrably disjoint. P1 implementation waits for P0 acceptance.

## GP — game presentation/runtime seam

In parallel with physics after G1:

- establish the bounded game-owned App-World -> prepared-render adapter;
- resolve Render frame-context dependence on a live Scene manager without assigning gameplay
  authority to ScenePlugin;
- establish physical pose -> presentation pose interpolation;
- keep renderer submission free of live gameplay/ECS authority reads.

## G2 — first local playable arena

Compose accepted G1 + physics + presentation with:

- code-owned/formed minimal arena content;
- camera;
- move/jump;
- hazard/health;
- interact/objective;
- screen-space HUD;
- win/lose/restart.

Acceptance must include real window/GPU execution evidence where repository testing requires it.
Synthetic readiness DTOs are not sufficient.

## RB0 — runtime content/bootstrap closure

Once the game runtime consumer exists, define the smallest runtime-facing formed product/package
needed to launch the same game from authored project content.

This may proceed in parallel with early multiplayer transport work if ownership/write sets are
disjoint.

## MP0 — real gameplay QUIC consumer

Only after the same game is playable locally:

- add app-owned `runen-net-quic` client/server endpoint realization;
- admit real session bindings into current RunenNet owners;
- move game command frames over the real transport;
- feed real authority replication submissions into transport delivery and report exact
  `DeliveryAcceptance`;
- prove two processes first, then two remote clients where the selected topology requires it;
- preserve concrete transport outside reusable Engine networking semantics.

## MP1 — ordinary Replicated View for the actual game state

Use the real game to decide the smallest explicit network-visible state contract.

Target direction:

```text
game-owned Replicated View
-> standard Runenwerk extraction/application
-> RunenNet replication semantics
```

Do not restore deleted component-registration macros and do not serialize the raw ECS world.

Keep the custom driver path for expert/special representations.

## MP2 — remote interpolation and correction presentation

Add:

- predicted local player presentation;
- authoritative correction smoothing where needed;
- buffered interpolation for remote players;
- explicit separation between confirmed/predicted physical state and visual state.

## MP3 — listen-server and dedicated-authority product proof

With the real game and transport in place:

- prove current one-App/one-World Host composition or expose a concrete defect;
- prove headless dedicated authority with no render/UI/audio activation;
- prove local single-player remains the same gameplay implementation;
- only then derive batteries-included Local Game / Client / Host / Dedicated Server composition
  groups under ADR 0019.

# 15. Safe parallelism

After G1 is accepted, the following can be prepared in parallel when exact writers remain disjoint:

```text
physics owner/design
game presentation adapter
audio investigation
RunenInput controller investigation
runtime bootstrap investigation
```

Do not implement all of them merely because they are parallelizable.

The dependency-critical path remains:

```text
G1 command/authority spine
  -> character motion
  -> local playable game
  -> real gameplay transport
  -> ordinary replicated view
  -> remote interpolation
  -> host/dedicated product proof
```

Networking syntax should not race ahead of the maintained game's state model.

# 16. Product/plugin groups come after proof

ADR 0019 already establishes the correct long-term usability law:

> internal decomposition must not determine application complexity.

This report does not select final group names or memberships.

The game must first prove the real compositions. Only then should Runenwerk encode transparent
recipes for concepts directionally equivalent to:

```text
local game
multiplayer client
listen host
dedicated authority
```

Those recipes lower to the same App/plugins/resources; they are not another runtime or semantic
authority.

# 17. Explicit non-selections

This investigation does not authorize or select:

- a second gameplay World;
- a new scheduler;
- a generic gameplay event framework;
- final Replicated View macros/attributes;
- generic Engine QUIC ownership;
- Rapier as the physics contract;
- Kira as the audio contract;
- a scripting language/runtime;
- gameplay graph implementation;
- runtime prefab implementation;
- animation implementation;
- gamepad semantics inside Runenwerk;
- a managed backend-service abstraction;
- matchmaking/lobby/account systems;
- full PBR completion as a small-game prerequisite;
- general rigid-body simulation as the first physics slice.

# 18. First follow-up issue after acceptance

The first implementation issue should be:

```text
[Game][G1] Establish a maintained local-game executable and tick-command authority spine
```

It should remain intentionally smaller than a playable visual game.

Its purpose is to establish the architectural invariant that all later work depends on:

```text
physical/product input
-> retained frame-to-tick intent
-> tick-targeted game command
-> one deterministic command application
-> App-world gameplay state
```

Local single-player proves it first.

Multiplayer later routes the **same command vocabulary and same application function** through
RunenNet prediction/authority instead of adding a second gameplay implementation.

No other implementation child should be activated merely by this report before G1's exact accepted
surface shows what additional contract is actually needed.

# Conclusion

Runenwerk is not blocked on inventing another broad game framework.

It is blocked on establishing one maintained game consumer and closing a small number of concrete
integration seams around that consumer:

```text
1. frame input -> fixed tick commands
2. replayable deterministic participant command application
3. SDF-first character motion
4. authoritative/predicted state -> presentation state
5. App-world gameplay -> prepared render presentation
6. authored products -> standalone game bootstrap
7. real gameplay QUIC transport
8. ordinary replicated view
9. remote interpolation
```

The strongest long-term direction is therefore one game with one simulation model and multiple
authority/routing roles, not separate single-player and multiplayer games.

The first correct implementation step is G1, not networking breadth, general physics, scripting,
animation, audio, or another renderer proof.
