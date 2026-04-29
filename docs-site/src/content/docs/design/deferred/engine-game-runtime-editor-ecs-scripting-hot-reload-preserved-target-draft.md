---
title: Game Runtime, Editor, ECS, Scripting, and Hot Reload Preserved Target Draft
description: Preserved long-form target draft moved from the active architecture/gap design to keep detail without mixing current and aspirational states.
status: deferred
owner: engine
layer: engine-runtime
canonical: false
last_reviewed: 2026-04-29
related_designs:
  - ../active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
---

# Game Runtime, Editor, ECS, Scripting, and Hot Reload Preserved Target Draft

This document preserves the prior long-form target draft verbatim so information is not lost.

Interpretation:

- This is deferred and aspirational material.
- Examples may use crate names, APIs, and asset formats that do not match the current workspace.
- Use the active design for current-state boundaries and gap tracking.

## Purpose

This document defines the target architecture for building games on top of Runenwerk, with a focus on the boundary between code, editor-authored content, runtime execution, ECS world state, scripting, prefab instantiation, and reload workflows.

The central goal is to make the engine scalable for real game production without turning ECS into a dumping ground for assets, editor state, scripting state, or service orchestration.

## Core Doctrine

```text
Code defines capabilities.
Editor composes content.
Runtime executes the result.
ECS stores live world state.
```

This means:

- Rust code defines component types, systems, runtime plugins, validation rules, behavior, scheduling, and low-level engine capabilities.
- The editor creates and modifies authored content such as scenes, prefabs, input maps, materials, skill trees, abilities, quests, and component values.
- Runtime loads, validates, instantiates, and executes authored content.
- ECS stores live instances and mutable simulation state, not authored definitions or editor documents.
- Scripts can request behavior through safe APIs, but should not directly own ECS internals.

## Architectural Direction

The preferred dependency direction is:

```text
foundation
  -> domain definitions
  -> ECS/runtime projection
  -> engine/runtime
  -> apps/editor/tools
  -> games
```

A game built with Runenwerk should be a separate crate or package that depends on engine/runtime/domain crates instead of modifying engine internals for every gameplay feature.

Example project shape:

```text
runenwerk/
  crates/
    foundation/
    domain/
    runtime/
    apps/
  games/
    my_game/
      Cargo.toml
      src/
        main.rs
        game.rs
        plugins/
          mod.rs
          player.rs
          combat.rs
        components/
        systems/
      assets/
        scenes/
          boot.rwscene
        prefabs/
          player.rwprefab
        input/
          player_default.rwinput
        abilities/
        materials/
        skill_trees/
```

## ECS Ownership

### What Should Be an ECS Entity

An ECS entity represents a live thing inside the simulated world.

Good ECS entities:

```text
player instance
enemy instance
projectile
door
pickup
camera
light
trigger volume
spawned ability effect
runtime VFX emitter
runtime audio emitter
spawn point instance
runtime UI-world marker
```

An entity should answer:

```text
Does this thing exist now in the simulated world?
```

If yes, it is probably an ECS entity.

### What Should Not Be an ECS Entity

Do not model authored definitions, documents, editor state, or services as ECS entities.

Bad ECS entities:

```text
ability definition
skill tree definition
material graph definition
quest definition
dialogue definition
item definition
weapon definition
prefab asset
scene document
editor tab
asset database
renderer backend
physics world service
network server
input device manager
undo history
```

These belong in domain crates, asset documents, editor state, runtime services, or resources.

## Component Ownership

### What Should Be an ECS Component

A component is a small piece of data attached to a live entity.

Good ECS components:

```text
Transform
GlobalTransform
Velocity
Health
Mana
Faction
Renderable
Collider
CharacterBody
CharacterMovement
CharacterMotorCommand
MovementIntent
LookIntent
ControlSource
GroundedState
CameraRig
Inventory
Equipment
StatusEffects
AbilityLoadout
NetworkAuthority
Lifetime
Projectile
DamageDealer
Interactable
```

A component should answer:

```text
What data does this live entity currently have?
```

Good components are usually:

- plain data;
- independently queryable;
- serializable or snapshot-friendly where useful;
- domain-shaped;
- small enough to avoid becoming a god-object;
- owned by known Rust types or registered schemas.

### What Should Not Be an ECS Component

Bad components:

```text
Renderer
PhysicsEngine
AssetDatabase
InputManager
SkillTreeCompiler
UndoStack
SaveGameManager
NetworkServer
RhaiRuntime
ScriptVm
EditorPanel
DatabaseConnection
```

Those are systems, resources, services, or editor concepts.

Avoid god components:

```text
CharacterEverything
PlayerStateMegaBlob
AbilitySystemComponent
SceneObjectFullState
EditorObject
```

Prefer smaller components with clear ownership.

## Authored Data vs Runtime State

Authored definitions should live outside ECS:

```text
AbilityDefinition
SkillTreeDefinition
MaterialGraphDefinition
ItemDefinition
WeaponDefinition
QuestDefinition
DialogueDefinition
PrefabDocument
SceneDocument
InputContextDefinition
AnimationGraphDefinition
```

Runtime ECS state should reference these definitions by stable IDs or handles:

```text
AbilityInstance -> AbilityDefinitionId
MaterialInstance -> MaterialId
SkillProgression -> SkillTreeId
ItemStack -> ItemDefinitionId
Renderable -> MeshId + MaterialId
```

The authored definition describes what something is. The ECS component describes the current live state of an instance.

## Code vs Editor Boundary

### Code Defines Capabilities

Use Rust code for new capabilities, behavior, rules, systems, validators, component types, and runtime plugins.

Code owns:

```text
ECS storage
scheduler
asset loading
prefab spawning
scene loading
input runtime
physics runtime
render runtime
audio runtime
network runtime
scripting runtime
diagnostics and validation
component definitions
system definitions
runtime plugin definitions
game-specific rules
combat execution
ability execution
AI behavior when not data-driven
save/load behavior
replication behavior
```

Use code when defining how something works:

```text
how movement works
how damage is calculated
how input becomes intent
how abilities execute
how enemies choose targets
how networking authority works
how stamina regenerates
```

### Editor Composes Content

Use the editor for instances, assets, tuning, composition, and authored data.

The editor owns:

```text
scenes
prefabs
entities in scenes
component values
input bindings
materials
material graphs
meshes/import settings
animation graphs
skill trees
abilities
items
quests
dialogue
spawn points
encounters
VFX/audio references
UI layout
```

Use the editor when choosing concrete values or assembling known capabilities:

```text
This player prefab has these components.
This movement speed is 5.5.
This keybinding is WASD.
This scene contains this spawn point.
This enemy prefab uses this mesh.
This fireball ability has this cost and cooldown.
This skill tree has these nodes.
This material uses this graph.
```

The editor cannot invent behavior the runtime does not understand. It can only compose content from known capabilities, registered schemas, and data-driven primitives.

## Game Crate Startup Shape

A game crate should compose engine capabilities and game-specific plugins.

Example entry point:

```rust
// file: games/my_game/src/main.rs
// function: main

use my_game::game::build_game;

fn main() {
    build_game().run();
}
```

Example game composition root:

```rust
// file: games/my_game/src/game.rs
// function: build_game

use runenwerk_app::App;

pub fn build_game() -> App {
    let mut app = App::new();

    app.add_plugin(runenwerk_input_runtime::InputRuntimePlugin);
    app.add_plugin(runenwerk_physics_runtime::PhysicsRuntimePlugin);
    app.add_plugin(runenwerk_character_runtime::CharacterRuntimePlugin);
    app.add_plugin(runenwerk_render_runtime::RenderRuntimePlugin);

    app.add_plugin(crate::plugins::player::PlayerPlugin);

    app.load_boot_scene("assets/scenes/boot.rwscene");

    app
}
```

The first concrete game setup should usually include:

```text
1. game crate
2. boot scene
3. player prefab
4. input context
5. player/game plugin
6. run app
```

## Player Endgoal Shape

The player should not be a hardcoded engine mega-object.

Prefer this model:

```text
ControllableCharacter + LocalPlayer ControlSource = Player
ControllableCharacter + AI ControlSource = Bot
ControllableCharacter + Network ControlSource = Remote Player
ControllableCharacter + Replay ControlSource = Replay Ghost
```

A player is primarily a prefab composition.

Example live ECS entity:

```text
Entity(Player)
  Transform
  GlobalTransform
  ControlSource(LocalPlayer0)
  MovementIntent
  LookIntent
  CharacterBody
  CharacterMovement
  CharacterMotorCommand
  Velocity
  GroundedState
  CameraRig
  Renderable
  Health
```

### Player Movement Flow

Do not directly move `Transform` from keyboard input.

Correct runtime flow:

```text
physical input
  -> input action state
  -> MovementIntent / LookIntent
  -> CharacterMotorCommand
  -> physics-resolved movement
  -> Transform update
  -> camera/render update
```

System order:

```text
InputCollectSystem
PlayerInputIntentSystem
CharacterMovementSystem
CharacterPhysicsSystem
CameraRigSystem
RenderExtractSystem
```

This allows the same movement stack to support:

```text
local player
AI-controlled character
networked player
replay ghost
scripted control
cutscene possession
```

### Player Prefab Example

```ron
// file: games/my_game/assets/prefabs/player.rwprefab
// document: player prefab

Prefab(
    id: "player",
    components: [
        Transform(
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        ),

        ControlSource(LocalPlayer(0)),

        MovementIntent(),
        LookIntent(),

        CharacterBody(
            capsule_radius: 0.35,
            capsule_height: 1.8,
        ),

        CharacterMovement(
            walk_speed: 5.0,
            sprint_speed: 8.0,
            acceleration: 24.0,
            jump_impulse: 6.5,
        ),

        Velocity(),
        GroundedState(),

        CameraRig(
            mode: FirstPerson,
            fov_degrees: 90.0,
        ),
    ],
)
```

### Input Context Example

```ron
// file: games/my_game/assets/input/player_default.rwinput
// document: player input context

InputContext(
    id: "player_default",
    actions: [
        Action(
            id: "move",
            value_type: Vec2,
            bindings: [
                KeyboardAxis2D(
                    up: "W",
                    down: "S",
                    left: "A",
                    right: "D",
                ),
            ],
        ),
        Action(
            id: "look",
            value_type: Vec2,
            bindings: [MouseDelta],
        ),
        Action(
            id: "jump",
            value_type: Bool,
            bindings: [Keyboard("Space")],
        ),
    ],
)
```

## Runtime Entity and Component Mutation

Runtime is allowed to mutate ECS state.

Runtime can:

```text
spawn entities
despawn entities
add known components
remove known components
instantiate prefab assets
generate temporary spawn plans
create procedural entities
```

This is normal ECS behavior.

Examples:

```text
A character enters water -> add Swimming
A target catches fire -> add Burning
A projectile arms after 0.2 seconds -> add ArmedProjectile
An ability spawns a fireball -> instantiate projectile prefab
A procedural spawner creates an enemy -> spawn generated SpawnPlan
```

The key rule:

```text
Runtime may add/remove registered component types.
Runtime should not casually invent new component schemas.
```

Avoid arbitrary script-defined or unregistered components in core gameplay.

## Prefab Model

Use precise terminology:

```text
PrefabDocument
  Authored asset saved to disk.

PrefabInstance
  Live ECS instance spawned from a prefab.

SpawnPlan
  Runtime-generated plan for creating one or more entities.

EntityArchetype
  Reusable runtime composition pattern.

ComponentSchema
  Registered description of a component type and fields.

ComponentInstance
  Actual component data on an entity.
```

Rules:

```text
Editor creates PrefabDocuments.
Runtime instantiates PrefabDocuments.
Runtime may generate SpawnPlans.
ECS stores ComponentInstances.
Rust/register/schema owns ComponentSchemas.
```

Runtime-created spawn plans are useful for:

```text
procedural dungeon rooms
mod-generated variants
loot rolls
runtime-composed projectiles
editor previews
network snapshot spawning
```

A runtime-generated spawn plan only becomes a stable authored prefab if the editor extracts, validates, and saves it as a PrefabDocument.

## Scripting Model

### Core Decision

Rust owns engine correctness. Rhai is the first embedded gameplay scripting language.

```text
Scripts do not own the world.
Scripts propose changes.
Rust validates and applies them.
```

Do not let Rhai or another scripting language directly own ECS internals.

Good Rhai host API:

```rust
// file: games/my_game/assets/scripts/encounters/goblin_ambush.rhai
// function: on_trigger_enter

fn on_trigger_enter(ctx, other) {
    if ctx.tags_has(other, "player") {
        ctx.spawn_prefab("prefab.enemy.goblin", ctx.self_position());
        ctx.play_audio("ambush_stinger");
    }
}
```

Bad script API:

```rust
// avoid
entity.add_component("RandomComponent", #{ anything: true });
entity.health.current -= 10;
world.raw_spawn_entity();
```

Internally, scripts should emit typed commands:

```text
SpawnPrefab
Despawn
ApplyDamage
ApplyStatus
StartCooldown
EmitEvent
```

Rust then validates and applies them.

### Rust vs Rhai vs Graphs vs WASM

Use Rust for:

```text
engine systems
component definitions
physics
networking
serialization
save/load
deterministic simulation
performance-critical combat
replication
validation
asset compilers
host API implementation
```

Use Rhai for:

```text
encounter scripts
quest reactions
NPC behavior glue
cutscene callbacks
one-off gameplay reactions
debug commands
prototype gameplay glue
editor automation commands
test scenario scripts
```

Use domain-specific node graphs for:

```text
abilities
materials
skill trees
dialogue
behavior trees
animation/state machines
VFX graphs
quest flow where visual authoring is better than imperative scripting
```

Consider WASM later for:

```text
sandboxed mods
third-party plugin isolation
language-neutral scripting
stronger security boundaries
marketplace-style extensions
```

Do not start with WASM unless mod security or multi-language plugin isolation is a first-class requirement.

### Why Rhai Is the First Scripting Choice

Rhai fits Runenwerk better than Lua as the initial embedded scripting layer because the engine is Rust-first and values small, explicit, inspectable boundaries.

Rhai advantages for this project:

```text
Rust-native embedding
small dependency surface
simple host API binding
good hot-reload fit
Rust-like syntax
no broad external runtime ecosystem requirement
clean adapter isolation
```

Lua remains a reasonable alternative for public modder familiarity, but it is not the preferred first choice for Runenwerk.

### Rhai Adapter Boundary

Rhai-specific types should not leak into domain crates or generic runtime contracts.

Preferred crate split:

```text
crates/domain/script
  ScriptAssetId
  ScriptModuleId
  ScriptCallbackName
  ScriptBindingDefinition
  ScriptDiagnostic

crates/runtime/script_runtime
  language-neutral script lifecycle
  script command queue
  script host API
  script hot reload model
  script diagnostics

crates/adapters/script_rhai
  Rhai engine integration
  Rhai host API bindings
  Rhai AST/module cache
  Rhai diagnostic mapping
```

The generic runtime should know that a script callback exists. Only the adapter should know that the callback is implemented in Rhai.

Example command boundary:

```rust
// file: crates/runtime/script_runtime/src/commands.rs
// enum: ScriptCommand

pub enum ScriptCommand {
    SpawnPrefab {
        prefab: PrefabId,
        transform: Transform,
    },
    Despawn {
        entity: EntityRef,
    },
    ApplyDamage {
        target: EntityRef,
        amount: DamageAmount,
        source: Option<EntityRef>,
    },
    AddStatus {
        target: EntityRef,
        status: StatusEffectId,
        duration_seconds: f32,
    },
    EmitEvent {
        event: ScriptEvent,
    },
}
```

Example Rhai binding location:

```rust
// file: crates/adapters/script_rhai/src/bindings/world_bindings.rs
// function: register_world_bindings

pub fn register_world_bindings(engine: &mut rhai::Engine) {
    engine.register_fn("spawn_prefab", ScriptCtx::spawn_prefab);
    engine.register_fn("despawn", ScriptCtx::despawn);
    engine.register_fn("emit_event", ScriptCtx::emit_event);
}
```

### Script-Owned State

Avoid arbitrary script-defined ECS components for core gameplay.

If scripts need local persistent state, store it behind a controlled runtime component:

```rust
// file: crates/runtime/script_runtime/src/components/script_state.rs
// type: ScriptState

pub struct ScriptState {
    pub script_id: ScriptAssetId,
    pub scope_id: ScriptScopeId,
    pub state: ScriptValueMap,
}
```

This keeps script state inspectable, serializable if needed, and isolated from core ECS component schemas.

## Data-Driven Middle Ground

Some systems should be data-driven after the runtime primitives exist.

Example ability system:

Code defines primitives:

```text
ApplyDamage
SpawnPrefab
ApplyStatus
ConsumeResource
StartCooldown
AreaQuery
ProjectileCast
```

Editor composes abilities:

```text
Fireball Ability
  cost: 20 mana
  cooldown: 1.2s
  cast action: SpawnPrefab(projectile.fireball)
  on hit: ApplyDamage(40 fire)
  on hit: ApplyStatus(burning, 3s)
```

No new code is needed for each ability if existing primitives are sufficient.

New primitive behavior still requires code:

```text
ChainLightning
TimeRewind
Possession
PortalTeleport
```

unless these are already expressible with existing primitives.

## Hot Reload and Code Refresh

Do not make “hot reload all Rust code” the initial goal.

Use three levels:

```text
Level 1: hot-reload data/assets
Level 2: refresh code registry after rebuild
Level 3: hot-swap running Rust code
```

Recommended target:

```text
Assets hot reload live.
Rust code rebuilds.
Editor refreshes component/system/plugin registry.
Play session restarts for structural Rust changes.
```

### Desired Workflow

```text
1. Create new Rust component/system/plugin.
2. Register it in the game plugin.
3. Cargo rebuilds.
4. Editor detects the new build.
5. Editor refreshes the registry.
6. New component appears in Add Component menu.
7. Add it to prefab/entity.
8. Restart play session.
9. New behavior is visible.
```

### Live Hot Reload Targets

Good live hot reload targets:

```text
scene files
prefab values
input bindings
ability data
skill trees
material graphs
shader assets
animation graphs
Rhai scripts
quest data
dialogue data
```

Usually requires restart/reload:

```text
new Rust component type
component memory layout change
new Rust system
plugin graph changes
ECS internals
renderer backend changes
physics backend changes
network schema changes
```

### Type and Plugin Registry

Every code-defined thing should register itself.

Example:

```rust
// file: games/my_game/src/plugin.rs
// method: MyGamePlugin::build

pub struct MyGamePlugin;

impl Plugin for MyGamePlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Stamina>();
        app.register_component::<MovementState>();

        app.add_systems(Update, stamina_regen_system);

        app.register_prefab_validator::<PlayerPrefabValidator>();
    }
}
```

The editor should not guess. It asks the registry:

```text
Which components exist?
Which fields do they expose?
Which systems exist?
Which assets are valid?
Which validators exist?
Which editor panels are provided?
```

### Runtime Restart Boundary

For structural Rust changes, prefer:

```text
stop play session
unload world
refresh registry
reload scene/prefabs
recreate ECS world
restart play session
```

Component memory layout changes should not be hot-swapped casually. They require either restart or explicit migrations.

## Editor and Runtime Process Model

The strongest long-term architecture is process separation:

```text
Editor process
  owns UI, asset editing, inspectors, project browser

Game runtime process
  owns loaded game code, ECS simulation, runtime plugins
```

When code changes:

```text
cargo build
  -> stop old game runtime process
  -> start new game runtime process
  -> editor reconnects
  -> reload scene/prefabs
  -> refreshed components/systems appear
```

Benefits:

```text
editor does not crash when game code crashes
no dynamic library unload pain
clean play-session restart
good foundation for multiplayer/server testing
clear editor-runtime protocol
```

Dynamic Rust plugins are possible but should be treated as optional and later. Rust ABI stability and dynamic library unloading are hard problems.

## Recommended Crate Areas

A future crate/domain layout could include:

```text
crates/domain/input
  InputContextDefinition
  InputActionDefinition
  ControlSource
  MovementIntent
  LookIntent

crates/domain/character
  CharacterBody
  CharacterMovement
  CharacterMotorCommand
  GroundedState

crates/domain/prefab
  PrefabDocument
  PrefabComponentEntry
  PrefabValidation

crates/domain/scene
  SceneDocument
  SpawnPoint

crates/runtime/input_runtime
  device input collection
  action state resolution
  player input binding

crates/runtime/character_runtime
  character movement systems

crates/runtime/physics_runtime
  character controller physics

crates/runtime/prefab_runtime
  prefab instantiation into ECS

crates/runtime/script_runtime
  language-neutral script command API

crates/adapters/script_rhai
  Rhai adapter for script_runtime

crates/apps/editor_app
  prefab editor
  input editor
  scene editor
  play mode orchestration
  registry refresh UI
```

Actual names should follow the current repository naming and crate doctrine.

## Implementation Phases

### Phase 1: Rust-Only Core

Build the reliable engine/runtime foundation first:

```text
ECS
components
systems
prefab spawning
scene loading
input runtime
character runtime
physics runtime
diagnostics
```

No Rhai scripting is required for basic player movement.

### Phase 2: Data-Driven Content

Add authored definitions:

```text
PrefabDocument
SceneDocument
InputContext
AbilityDefinition
SkillTreeDefinition
MaterialGraphDefinition
ItemDefinition
```

Runtime ECS instances should reference these definitions by IDs.

### Phase 3: Editor Composition

Editor can create and edit:

```text
scenes
prefabs
component values
input maps
materials
abilities
skill trees
spawn points
```

### Phase 4: Registry Refresh

Add:

```text
component registry
system registry
plugin registry
reflection/schema metadata
editor refresh after cargo build
play-session restart on code changes
```

### Phase 5: Rhai Scripting

Add:

```text
language-neutral script runtime
Rhai adapter
typed script command API
Rhai AST/module cache
script hot reload
diagnostics for script failures
safe host API bindings
```

### Phase 6: External Runtime Process

Add:

```text
editor-runtime protocol
runtime process restart
state handoff/reload
crash isolation
```

## Invariants

The architecture should preserve these invariants:

1. ECS stores live world state, not authored definitions.
2. Authored assets are validated before runtime instantiation.
3. Runtime may spawn entities and add/remove known components.
4. Runtime should not bypass schema, validation, ownership, or authority rules.
5. Rust defines core component types, systems, scheduling, validation, and engine behavior.
6. Editor composes content from registered capabilities.
7. Scripts request commands through safe APIs instead of directly mutating ECS internals.
8. Multiplayer-authoritative gameplay should be validated server-side.
9. Structural Rust changes require registry refresh and usually play-session restart.
10. Data/assets/scripts should be hot-reloadable where safe.
11. Game-specific behavior belongs in game crates/plugins, not engine internals.
12. Engine crates provide reusable capability; game crates compose and extend it.

## Non-Goals

This design does not require:

```text
arbitrary runtime-defined component types for core gameplay
hot-swapping Rust memory layouts inside a running ECS world
making Rhai the owner of game state
making every authored graph an ECS entity
hardcoding Player as an engine primitive
embedding editor UI state into runtime gameplay components
starting with WASM-based modding
starting with dynamic Rust plugin hot-swap
```

These can be revisited later if there is a strong product requirement.

## Final Mental Model

```text
Rust code:
  defines what is possible.

Editor:
  builds concrete game content from known possibilities.

Assets:
  store authored definitions.

Runtime:
  validates, loads, instantiates, and executes.

ECS:
  stores live state only.

Scripts:
  request behavior through safe commands.

Hot reload:
  instant for data/scripts,
  rebuild-refresh-restart for Rust code.
```

The ideal workflow:

```text
Programmer adds a capability in Rust.
Editor refreshes and exposes that capability.
Designer composes content using it.
Runtime executes it safely.
```
