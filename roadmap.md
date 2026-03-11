Foundation Domain — Clean Complete Target

This is the API I would target for foundation/ecs.

1. Core rules

One derive only: #[derive(ecs::Component)]

Tags are empty components

Resources are global singleton components

One query type: Query<Q, F = ()>

One query iteration method: iter()

System params are the primary API

World remains the low-level runtime API

Borrow-wrapper types are internal only

2. Public gameplay-facing API
   System params

File: foundation/ecs/src/system/params.rs
Types: Query, Res, ResMut, Commands, EventReader, EventWriter

use ecs::prelude::*;

#[derive(ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Velocity { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Simulated;

#[derive(ecs::Component)]
struct DeltaTime(pub f32);

#[derive(ecs::Component)]
struct Frame(pub u64);

fn tick(
mut query: Query<(&mut Position, &Velocity), With<Simulated>>,
dt: Res<DeltaTime>,
mut frame: ResMut<Frame>,
) {
for (pos, vel) in query.iter() {
pos.x += vel.x * dt.0;
pos.y += vel.y * dt.0;
}

    frame.0 += 1;
}

This is the canonical target shape.

3. Supported query patterns

File: foundation/ecs/src/query/data.rs
Trait: QueryData

Target support should include:

Query<&Position>
Query<&mut Position>
Query<(Entity, &Position)>
Query<(Entity, &mut Position)>

Query<(&Position, &Velocity)>
Query<(&mut Position, &Velocity)>
Query<(&Position, &mut Velocity)>
Query<(&mut Position, &mut Velocity)>

Query<(&Position, &Velocity, &Health)>
Query<(&mut Position, &Velocity, &Health)>
Query<(&mut Position, &mut Velocity, &Health)>

Query<Option<&Velocity>>
Query<Option<&mut Velocity>>
Query<(&mut Position, Option<&Velocity>)>
Query<(Entity, Option<&Velocity>)>

That is the minimum “complete modern ECS” set.

4. Filters

File: foundation/ecs/src/query/filter.rs
Types: With, Without

Query<&Position, With<Player>>
Query<&Position, Without<Disabled>>
Query<&Position, (With<Player>, Without<Disabled>)>

Target filter API:

With<T>

Without<T>

tuples for composition

Nothing more is needed initially.

5. Public World runtime API

This is the low-level API that powers systems and engine/runtime code.

File: foundation/ecs/src/world/mod.rs
Impl block: impl World

impl World {
pub fn new() -> Self;

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity;
    pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError>;
    pub fn contains(&self, entity: Entity) -> bool;

    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EntityError>;
    pub fn remove<B: Bundle>(&mut self, entity: Entity) -> Result<B, EntityError>;

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<'_, T>>;

    pub fn require<T: Component>(&self, entity: Entity) -> Result<&T, EntityError>;
    pub fn require_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<'_, T>, EntityError>;

    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, EntityError>;
    pub fn entity_mut(&mut self, entity: Entity) -> Result<EntityMut<'_>, EntityError>;

    pub fn insert_resource<R: Component>(&mut self, resource: R);
    pub fn has_resource<R: Component>(&self) -> bool;
    pub fn resource<R: Component>(&self) -> Result<&R, ResourceError>;
    pub fn resource_mut<R: Component>(&mut self) -> Result<ResMut<'_, R>, ResourceError>;
    pub fn remove_resource<R: Component>(&mut self) -> Option<R>;

    pub fn commands(&self) -> Commands;
}
Important target decision

Use Component for resources too.

So this should go away as a public concept:

File to remove: foundation/ecs/src/resource.rs

pub trait Resource: 'static {}
impl<T: 'static> Resource for T {}

If you keep it at all, keep it internal or as a blanket alias over Component.

6. Entity handles

File: foundation/ecs/src/world/handles.rs
Types: EntityRef, EntityMut

impl<'w> EntityRef<'w> {
pub fn entity(&self) -> Entity;
pub fn contains<T: Component>(&self) -> bool;
pub fn get<T: Component>(&self) -> Option<&T>;
pub fn require<T: Component>(&self) -> Result<&T, EntityError>;
}

impl<'w> EntityMut<'w> {
pub fn entity(&self) -> Entity;
pub fn contains<T: Component>(&self) -> bool;
pub fn get<T: Component>(&self) -> Option<&T>;
pub fn get_mut<T: Component>(&mut self) -> Option<Mut<'_, T>>;
pub fn require<T: Component>(&self) -> Result<&T, EntityError>;
pub fn require_mut<T: Component>(&mut self) -> Result<Mut<'_, T>, EntityError>;
pub fn insert<B: Bundle>(&mut self, bundle: B) -> Result<(), EntityError>;
pub fn remove<B: Bundle>(&mut self) -> Result<B, EntityError>;
pub fn despawn(self) -> Result<(), EntityError>;
}

This part is already close to good.

7. Bundles

File: foundation/ecs/src/bundle.rs
Trait: Bundle

Keep:

single component as bundle

tuple bundles up to reasonable arity

#[derive(ecs::Bundle)]

Example:

#[derive(ecs::Bundle)]
struct CombatBundle {
health: Health,
name: Name,
}

This is good and should stay.

8. Events — complete target

Events should exist in both:

low-level World API

ergonomic system-param API

Gameplay-facing target

File: foundation/ecs/src/system/params.rs
Types: EventReader<T>, EventWriter<T>

use ecs::prelude::*;

#[derive(Clone)]
struct DamageEvent {
amount: i32,
}

fn emit_damage(mut events: EventWriter<DamageEvent>) {
events.send(DamageEvent { amount: 5 });
}

fn read_damage(events: EventReader<DamageEvent>) {
for event in events.iter() {
let _ = event.amount;
}
}
Runtime-facing target

File: foundation/ecs/src/world/events.rs
Impl block: impl World

impl World {
pub fn configure_event_channel<T: 'static>(&mut self, config: EventChannelConfig);
pub fn emit_event<T: 'static>(&mut self, event: T);
pub fn read_events<T: 'static>(&self) -> &[T];
pub fn drain_events<T: 'static>(&mut self) -> Vec<T>;
pub fn clear_events<T: 'static>(&mut self) -> usize;
pub fn event_count<T: 'static>(&self) -> usize;
pub fn event_channel_stats<T: 'static>(&self) -> Option<EventChannelStats>;

    pub fn observe_events<T: 'static>(
        &mut self,
        observer_id: impl Into<String>,
        trigger: ObserverTrigger,
    ) -> bool;

    pub fn remove_event_observer(&mut self, observer_id: &str) -> bool;
    pub fn event_observer_invocations(&self, observer_id: &str) -> Option<u64>;
    pub fn drain_event_observer_notifications(&mut self) -> Vec<EventObserverNotification>;
    pub fn finish_event_frame(&mut self);
}

This part of your runtime is already strong.

9. Commands — complete target
   Gameplay-facing target

File: foundation/ecs/src/system/params.rs
Type: Commands

use ecs::prelude::*;

fn spawn_enemy(mut commands: Commands) {
commands.spawn((Enemy, Position { x: 0.0, y: 0.0 }));
}
Runtime-facing target

File: foundation/ecs/src/world/commands.rs
Type: Commands

Keep the queue model, but make the scheduler flush it automatically after the system/stage.

Publicly this is enough:

impl Commands {
pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B);
pub fn despawn(&mut self, entity: Entity);
pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B);
pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity);
pub fn apply(self, world: &mut World) -> Result<(), CommandError>;
}

This is already basically good.

10. Change tracking — complete target

This should remain runtime-facing first.

File: foundation/ecs/src/world/change_tracking.rs
Impl block: impl World

impl World {
pub fn current_change_tick(&self) -> u64;
pub fn component_changed_since<T: Component>(&self, tick: u64) -> bool;
pub fn resource_changed_since<R: Component>(&self, tick: u64) -> bool;

    pub fn component_changes_since(&self, tick: u64) -> Vec<ComponentChangeRecord>;
    pub fn resource_changes_since(&self, tick: u64) -> Vec<ResourceChangeRecord>;
}

Optional later ergonomic addition:

Changed<T> query filter

Added<T> query filter

RemovedComponents<T> event-like tracking

But not required for a first clean target.

11. Secondary indexes — complete target

Keep them runtime-facing.

File: foundation/ecs/src/world/indexes.rs
Impl block: impl World

impl World {
pub fn ensure_component_index<T: Component, K: Ord + Clone + 'static>(
&mut self,
extractor: fn(&T) -> K,
) -> bool;

    pub fn ensure_component_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        extractor: fn(&T) -> K,
    ) -> bool;

    pub fn find_entity_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<Entity>;

    pub fn find_entity_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<Entity>;

    pub fn find_entities_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Vec<Entity>;

    pub fn find_entities_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Vec<Entity>;

    pub fn find_component_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<&T>;
}

Only note: long term, these should probably become &self lookups instead of &mut self.

But for target completeness, this is okay.

12. Query internals — what should be public vs internal
    Public

Query<Q, F = ()>

QueryState<Q, F = ()> maybe, but advanced/runtime-facing

QueryAccess

With<T>

Without<T>

Internal only

QueryBorrow

QueryBorrowMut

ReadOnlyQueryData

MutableQueryData

store access helpers

So this should change.

File: foundation/ecs/src/prelude.rs

Target prelude
pub use crate::{
Bundle, CommandError, Commands, Component, ComponentChangeKind, ComponentChangeRecord, Entity,
EntityDespawnedEvent, EntityError, EntityMut, EntityRef, EntitySpawnedEvent,
EventChannelConfig, EventChannelStats, EventLifetime, EventObserverNotification,
EventTracingPolicy, ObserverTrigger, OverflowPolicy, Query, QueryAccess, QueryError,
QueryFilter, QueryState, QueryTypeAccess, Res, ResMut, ResourceChangeKind,
ResourceChangeRecord, ResourceError, With, Without, World,
};

And remove from prelude:

QueryBorrow
QueryBorrowMut
QueryData

Those are too implementation-shaped for the target API.

13. Exact target docs coverage

Your final target docs should cover these features:

Gameplay-facing

components

tags

resources

Query

Res

ResMut

filters

optional components

entity ids

events via EventReader/EventWriter

commands injection

Runtime-facing

World

spawn/despawn

direct access

entity handles

bundles

QueryState

event channels

observers

change tracking

indexes

manual commands apply

That is the complete target.

What I would delete from the current design
Delete as public API

separate Resource mental model

QueryBorrow

QueryBorrowMut

World::query_mut()

iter_mut()

single_mut()

get_mut_on() naming split in docs

Keep only internally if needed

borrow wrapper machinery

mutable/read-only fetch traits

The final clean public target

If I compress it to the cleanest final form, it is this:

Gameplay systems

File: foundation/ecs/examples/system_api.rs
Function: tick

use ecs::prelude::*;

#[derive(ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Velocity { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Simulated;

#[derive(ecs::Component)]
struct DeltaTime(pub f32);

#[derive(ecs::Component)]
struct Frame(pub u64);

fn tick(
mut query: Query<(&mut Position, &Velocity), With<Simulated>>,
dt: Res<DeltaTime>,
mut frame: ResMut<Frame>,
) {
for (pos, vel) in query.iter() {
pos.x += vel.x * dt.0;
pos.y += vel.y * dt.0;
}

    frame.0 += 1;
}
Runtime world

File: foundation/ecs/src/world/mod.rs

impl World {
pub fn new() -> Self;
pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity;
pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError>;
pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EntityError>;
pub fn remove<B: Bundle>(&mut self, entity: Entity) -> Result<B, EntityError>;

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<'_, T>>;
    pub fn require<T: Component>(&self, entity: Entity) -> Result<&T, EntityError>;
    pub fn require_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<'_, T>, EntityError>;

    pub fn insert_resource<R: Component>(&mut self, resource: R);
    pub fn resource<R: Component>(&self) -> Result<&R, ResourceError>;
    pub fn resource_mut<R: Component>(&mut self) -> Result<ResMut<'_, R>, ResourceError>;
    pub fn remove_resource<R: Component>(&mut self) -> Option<R>;

    pub fn commands(&self) -> Commands;
}
Query support

File: foundation/ecs/src/query/mod.rs

Query<&T>
Query<&mut T>
Query<(Entity, &T)>
Query<(Entity, &mut T)>
Query<(&A, &B)>
Query<(&mut A, &B)>
Query<(&A, &mut B)>
Query<(&mut A, &mut B)>
Query<Option<&T>>
Query<Option<&mut T>>
Query<(&mut A, Option<&B>)>
Filters
With<T>
Without<T>
(With<A>, Without<B>)
Events
EventReader<T>
EventWriter<T>
world.emit_event(T)
world.read_events::<T>()
world.drain_events::<T>()
Commands
Commands::spawn
Commands::despawn
Commands::insert
Commands::remove
Commands::apply
Bottom line

Your clean and complete target should be:

one derive: Component

one query type: Query<Q, F>

one iteration method: iter()

system params as the primary API

world API as the advanced/runtime API

events, commands, change tracking, and indexes included

old borrow-wrapper query API fully hidden or removed

That is the design I would target before touching implementation.

can u make one extensive roadmap? it should be 10 out of 10 rating. i will feed it to codex for implementation. give me a .md file

I’m turning your target API and current code into an implementation roadmap that is organized by file, phase, and acceptance criteria so it can be handed directly to Codex.

Nachgedacht für 4m 5s

Done: ecs_target_api_roadmap.md

i cant download the file.

provide me the full contents via other means
Nachgedacht für ein paar Sekunden
# ECS Target API Roadmap

This document defines the **clean target API**, the **migration plan**, and the **acceptance criteria** for evolving `foundation/ecs` from the current runtime-oriented API into a cleaner, system-parameter-first ECS.

It is written to be handed directly to an implementation agent.

---

# 1. Goals

The target API must satisfy these design goals:

- **One derive only** for ECS-managed data:
  - `#[derive(ecs::Component)]`
- **Tags are empty components**
- **Resources are global singleton components**
- **One query type**
  - `Query<Q, F = ()>`
- **One iteration method**
  - `iter()`
- **System parameters are the primary API**
  - `Query<Q, F>`
  - `Res<T>`
  - `ResMut<T>`
  - later: `Commands`, `EventReader<T>`, `EventWriter<T>`
- **`World` remains the low-level runtime API**
- **Old borrow-wrapper query types become internal or are removed**
- **The public API must match the documentation target**
- **The result must be simpler than the current implementation surface**

---

# 2. Final Target API

## 2.1 Canonical gameplay system shape

```rust
use ecs::prelude::*;

#[derive(ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Velocity { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Simulated;

#[derive(ecs::Component)]
struct DeltaTime(pub f32);

#[derive(ecs::Component)]
struct Frame(pub u64);

fn tick(
    mut query: Query<(&mut Position, &Velocity), With<Simulated>>,
    dt: Res<DeltaTime>,
    mut frame: ResMut<Frame>,
) {
    for (pos, vel) in query.iter() {
        pos.x += vel.x * dt.0;
        pos.y += vel.y * dt.0;
    }

    frame.0 += 1;
}
2.2 Target public concepts
Public gameplay-facing concepts

Component

Bundle

Entity

Query<Q, F = ()>

Res<T>

ResMut<T>

With<T>

Without<T>

Commands

later: EventReader<T>, EventWriter<T>

Public runtime-facing concepts

World

EntityRef

EntityMut

QueryState<Q, F = ()>

QueryAccess

event APIs

change tracking APIs

index APIs

Internal-only concepts

old borrowed query wrappers

mutable/read-only fetch internals

store access helpers

query aliasing plumbing

3. Non-Goals

These items are explicitly out of scope for the first target milestone unless later phases say otherwise:

archetype migration

sparse set redesign

scheduler rewrite beyond what is needed for system param injection

adding every Bevy-style query filter immediately

adding every possible query tuple arity immediately

automatic reflection/editor tooling

hot reloading as an ECS core concern

4. Key Design Decisions
4.1 One derive only

All ECS-managed data derives ecs::Component.

This includes:

regular components

tag components

resources

Example:

#[derive(ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Simulated;

#[derive(ecs::Component)]
struct DeltaTime(pub f32);
4.2 No public Resource mental model

The current code has:

pub trait Resource: 'static {}
impl<T: 'static> Resource for T {}

This should not remain a first-class public concept.

Preferred target:

resources are stored globally in World

resource types are still just Component

Possible implementation choices:

remove the public Resource trait entirely

keep it as an internal compatibility alias

keep it as a blanket alias over Component

Preferred choice:

remove it from public docs and public API thinking

signatures should prefer R: Component

4.3 One query type

Target:

Query<Q, F = ()>

Not target:

QueryBorrow

QueryBorrowMut

World::query_mut()

Mutability must be expressed by Q, not by a parallel API tree.

4.4 One iteration method

Target:

query.iter()

Not target:

iter_mut()

single_mut()

get_mut() on query wrappers as a separate naming tree

Read/write distinction comes from Q.

4.5 System params first, World second

Most gameplay code should use:

Query

Res

ResMut

World should still exist, but as the lower-level runtime interface.

5. Current State Summary

The current codebase already has strong foundations:

entity allocator

bundles

component storage

resources

commands

entity handles

event channels

observers

change tracking

component indexes

detached query state

access metadata

The current surface is still shaped around:

World::query()

World::query_mut()

QueryBorrow

QueryBorrowMut

The largest mismatch is that the docs target a system-param API that is not yet represented cleanly in the implementation.

6. High-Level Migration Strategy

This migration should happen in the following order:

Freeze the target API

Refactor query internals to support one unified query abstraction

Introduce system parameter extraction

Convert public docs and prelude exports

Complete query data support to match docs

Wire scheduler/runtime integration

Add event/command system-param story

Delete obsolete public API

Polish naming and acceptance tests

Do not start by patching the docs alone. The runtime surface must be reshaped first.

7. File-Level Target Architecture

This section describes the recommended target module layout. Exact file names may vary, but the public responsibilities should match this split.

7.1 Crate root

File: foundation/ecs/src/lib.rs

Target responsibilities

define clean public exports

expose only the intended surface

remove old query wrapper types from root exports

keep implementation internals unexported where possible

Target export shape
extern crate self as ecs;

mod bundle;
mod component;
mod entity;
mod errors;
pub mod prelude;
pub mod query;
mod world;
mod system;

pub use bundle::Bundle;
pub use component::Component;
pub use ecs_macros::{Bundle, Component};
pub use entity::{Entity, EntityAllocator};
pub use errors::{CommandError, EntityError, QueryError, ResourceError};

pub use query::{
    Query,
    QueryAccess,
    QueryFilter,
    QueryState,
    QueryTypeAccess,
    With,
    Without,
};

pub use system::{
    Commands,
    EventReader,
    EventWriter,
    Res,
    ResMut,
    SystemParam,
};

pub use world::{
    ComponentChangeKind,
    ComponentChangeRecord,
    EntityDespawnedEvent,
    EntityMut,
    EntityRef,
    EntitySpawnedEvent,
    EventChannelConfig,
    EventChannelStats,
    EventLifetime,
    EventObserverNotification,
    EventTracingPolicy,
    Mut,
    ObserverTrigger,
    OverflowPolicy,
    ResourceChangeKind,
    ResourceChangeRecord,
    World,
};
Remove from root exports

QueryBorrow

QueryBorrowMut

raw QueryData if it is purely internal

ReadOnlyQueryData

MutableQueryData

7.2 Prelude

File: foundation/ecs/src/prelude.rs

Target responsibilities

export the clean day-to-day API

prioritize gameplay-facing ergonomics

avoid exporting implementation detail types

Target prelude
pub use crate::{
    Bundle,
    CommandError,
    Commands,
    Component,
    ComponentChangeKind,
    ComponentChangeRecord,
    Entity,
    EntityDespawnedEvent,
    EntityError,
    EntityMut,
    EntityRef,
    EntitySpawnedEvent,
    EventChannelConfig,
    EventChannelStats,
    EventLifetime,
    EventObserverNotification,
    EventReader,
    EventTracingPolicy,
    EventWriter,
    Mut,
    ObserverTrigger,
    OverflowPolicy,
    Query,
    QueryAccess,
    QueryError,
    QueryFilter,
    QueryState,
    QueryTypeAccess,
    Res,
    ResMut,
    ResourceChangeKind,
    ResourceChangeRecord,
    With,
    Without,
    World,
};
Remove from prelude

QueryBorrow

QueryBorrowMut

old low-level query internals

7.3 Query module

File: foundation/ecs/src/query/mod.rs

Target responsibilities

define the unified public Query

define QueryState

define filters

define access metadata

hide read/mut query implementation splits internally

Suggested internal split
query/
  mod.rs
  access.rs
  filter.rs
  data.rs
  state.rs
  runtime.rs
  iter.rs
7.4 System module

File: foundation/ecs/src/system/mod.rs

Target responsibilities

define system parameter types:

Query

Res

ResMut

Commands

later EventReader, EventWriter

define extraction traits

define access metadata contribution for params

Suggested internal split
system/
  mod.rs
  params.rs
  extract.rs
  access.rs
7.5 World module

File: foundation/ecs/src/world/mod.rs

Target responsibilities

low-level entity/component/resource runtime API

events

indexes

change tracking

commands queue construction

entity handles

Suggested internal split
world/
  mod.rs
  world_struct.rs
  lifecycle.rs
  internal.rs
  events.rs
  indexes.rs
  handles.rs
  commands.rs
  change_tracking.rs
8. Exact Public API Target
8.1 Component

File: foundation/ecs/src/component.rs

Keep:

pub trait Component: 'static {
    fn component_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

No separate Tag derive.

No separate Resource derive.

8.2 Bundle

File: foundation/ecs/src/bundle.rs

Keep current shape, but expand tuple arity if needed.

Target support:

single component bundle

tuple bundles

custom #[derive(ecs::Bundle)]

No major conceptual changes needed.

8.3 World

File: foundation/ecs/src/world/mod.rs

Target runtime surface
impl World {
    pub fn new() -> Self;

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity;
    pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError>;
    pub fn contains(&self, entity: Entity) -> bool;

    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EntityError>;
    pub fn remove<B: Bundle>(&mut self, entity: Entity) -> Result<B, EntityError>;

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<'_, T>>;

    pub fn require<T: Component>(&self, entity: Entity) -> Result<&T, EntityError>;
    pub fn require_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<'_, T>, EntityError>;

    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, EntityError>;
    pub fn entity_mut(&mut self, entity: Entity) -> Result<EntityMut<'_>, EntityError>;

    pub fn insert_resource<R: Component>(&mut self, resource: R);
    pub fn has_resource<R: Component>(&self) -> bool;
    pub fn resource<R: Component>(&self) -> Result<&R, ResourceError>;
    pub fn resource_mut<R: Component>(&mut self) -> Result<ResMut<'_, R>, ResourceError>;
    pub fn remove_resource<R: Component>(&mut self) -> Option<R>;

    pub fn commands(&self) -> Commands;
}
Remove from public World surface

query_mut

if World::query remains, it should be transitional or advanced-only

the primary query entry point should become the system param API

If World::query is kept at all, the preferred target shape is:

pub fn query<Q, F = ()>(&mut self) -> QueryState<Q, F>
where
    Q: QueryData,
    F: QueryFilter;

But for the cleanest final public model, gameplay docs should not center on World::query.

8.4 Entity handles

File: foundation/ecs/src/world/handles.rs

Keep current structure, but stay consistent.

Target API
impl<'w> EntityRef<'w> {
    pub fn entity(&self) -> Entity;
    pub fn contains<T: Component>(&self) -> bool;
    pub fn get<T: Component>(&self) -> Option<&T>;
    pub fn require<T: Component>(&self) -> Result<&T, EntityError>;
}

impl<'w> EntityMut<'w> {
    pub fn entity(&self) -> Entity;
    pub fn contains<T: Component>(&self) -> bool;
    pub fn get<T: Component>(&self) -> Option<&T>;
    pub fn get_mut<T: Component>(&mut self) -> Option<Mut<'_, T>>;
    pub fn require<T: Component>(&self) -> Result<&T, EntityError>;
    pub fn require_mut<T: Component>(&mut self) -> Result<Mut<'_, T>, EntityError>;
    pub fn insert<B: Bundle>(&mut self, bundle: B) -> Result<(), EntityError>;
    pub fn remove<B: Bundle>(&mut self) -> Result<B, EntityError>;
    pub fn despawn(self) -> Result<(), EntityError>;
}

No major redesign needed here.

8.5 Query

File: foundation/ecs/src/query/runtime.rs

Target public type
pub struct Query<Q, F = ()> { ... }

This is a system parameter, not just a raw world borrow wrapper.

Target public methods
impl<Q, F> Query<Q, F>
where
    Q: QueryData,
    F: QueryFilter,
{
    pub fn iter(&mut self) -> Q::Iter<'_>;
    pub fn get(&mut self, entity: Entity) -> Option<Q::Item<'_>>;
    pub fn single(&mut self) -> Result<Q::Item<'_>, QueryError>;
    pub fn access(&self) -> &QueryAccess;
}
Optional builder-like methods

These are acceptable but should be secondary to type-level filters:

pub fn with<T: Component>(self) -> Query<Q, (F, With<T>)>;
pub fn without<T: Component>(self) -> Query<Q, (F, Without<T>)>;

Primary docs should still prefer type-level filters.

8.6 QueryState

File: foundation/ecs/src/query/state.rs

This remains the advanced runtime reusable query type.

Target public methods
impl<Q, F> QueryState<Q, F>
where
    Q: QueryData,
    F: QueryFilter,
{
    pub fn new(world: &mut World) -> Self;
    pub fn access(&self) -> &QueryAccess;

    pub fn with<T: Component>(self) -> Self;
    pub fn without<T: Component>(self) -> Self;

    pub fn iter<'w>(&mut self, world: &'w mut World) -> Q::Iter<'w>;
    pub fn get<'w>(&mut self, world: &'w mut World, entity: Entity) -> Option<Q::Item<'w>>;
    pub fn single<'w>(&mut self, world: &'w mut World) -> Result<Q::Item<'w>, QueryError>;
}
Target naming

Rename:

iter_on -> iter

get_on -> get

single_on -> single

iter_mut_on -> fold into iter

get_mut_on -> fold into get

single_mut_on -> fold into single

Mutability should come from Q.

8.7 QueryData support

File: foundation/ecs/src/query/data.rs

Required support for first complete target

Must support at least:

&T
&mut T
(Entity, &T)
(Entity, &mut T)

(&A, &B)
(&mut A, &B)
(&A, &mut B)
(&mut A, &mut B)

Option<&T>
Option<&mut T>

(&mut A, Option<&B>)
(Entity, Option<&T>)
Strongly recommended next step

Support common triples:

(&A, &B, &C)
(&mut A, &B, &C)
(&mut A, &mut B, &C)
Optional later

larger tuple arities

Has<T>

Changed<T> query wrappers

removed-component trackers

8.8 Filters

File: foundation/ecs/src/query/filter.rs

Keep:

With<T>

Without<T>

tuple composition

Target examples
Query<&Position, With<Player>>
Query<&Position, Without<Disabled>>
Query<&Position, (With<Player>, Without<Disabled>)>

No separate builder-only filtering model should be required.

8.9 Res / ResMut

File: foundation/ecs/src/system/params.rs

Keep the wrapper types but promote them to real system params.

Target definitions
pub struct Res<'a, T> {
    value: &'a T,
}

pub struct ResMut<'a, T> {
    value: &'a mut T,
}
Target behavior

Res<T> contributes read access to scheduler metadata

ResMut<T> contributes write access

Res<T> should be first-class in docs and extraction

Currently Res exists but is not wired into the runtime access API. That must be fixed.

8.10 Commands

File: foundation/ecs/src/world/commands.rs and foundation/ecs/src/system/params.rs

Runtime-facing target

Keep the queue model.

impl Commands {
    pub fn new() -> Self;
    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B);
    pub fn despawn(&mut self, entity: Entity);
    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B);
    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity);
    pub fn apply(self, world: &mut World) -> Result<(), CommandError>;
}
Gameplay-facing target

Allow scheduler injection of Commands.

Docs should show:

fn spawn_enemy(mut commands: Commands) {
    commands.spawn((Enemy, Position { x: 0.0, y: 0.0 }));
}

Scheduler/stage should flush commands automatically after systems.

8.11 Events

File: foundation/ecs/src/world/events.rs and foundation/ecs/src/system/params.rs

Runtime-facing world API

Keep current event channel capabilities.

Gameplay-facing target

Add:

pub struct EventReader<'a, T> { ... }
pub struct EventWriter<'a, T> { ... }
Target example
#[derive(Clone)]
struct DamageEvent {
    amount: i32,
}

fn emit_damage(mut events: EventWriter<DamageEvent>) {
    events.send(DamageEvent { amount: 5 });
}

fn read_damage(events: EventReader<DamageEvent>) {
    for event in events.iter() {
        let _ = event.amount;
    }
}

Event system params are not required for the first query migration phase, but they are required for the complete target.

8.12 Change tracking

File: foundation/ecs/src/world/change_tracking.rs

Keep current runtime API.

Target surface
impl World {
    pub fn current_change_tick(&self) -> u64;
    pub fn component_changed_since<T: Component>(&self, tick: u64) -> bool;
    pub fn resource_changed_since<R: Component>(&self, tick: u64) -> bool;

    pub fn component_changes_since(&self, tick: u64) -> Vec<ComponentChangeRecord>;
    pub fn resource_changes_since(&self, tick: u64) -> Vec<ResourceChangeRecord>;
}

Do not block the main migration on adding Changed<T> filters. Those can be later.

8.13 Secondary indexes

File: foundation/ecs/src/world/indexes.rs

Keep current functionality.

Target runtime API
impl World {
    pub fn ensure_component_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        extractor: fn(&T) -> K,
    ) -> bool;

    pub fn ensure_component_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        extractor: fn(&T) -> K,
    ) -> bool;

    pub fn find_entity_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<Entity>;

    pub fn find_entity_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<Entity>;

    pub fn find_entities_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Vec<Entity>;

    pub fn find_entities_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Vec<Entity>;

    pub fn find_component_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<&T>;
}

Long term improvement:

make lookup &self rather than &mut self

Do not block the main migration on that change.

9. Major API Deletions

The following public API should be removed or hidden by the end of the migration.

9.1 Remove from public API

QueryBorrow

QueryBorrowMut

World::query_mut

iter_mut

single_mut

public split between read-only and mutable borrowed query wrappers

public Resource as a distinct mental model

9.2 Keep internal only if needed

low-level fetch traits

query aliasing helpers

store access helpers

query wrapper glue used during transition

10. Migration Phases
Phase 0 — Lock the target
Goal

Freeze the design before implementation churn.

Tasks

Accept this target doc as source of truth.

Stop adding new features to the old public query API.

Mark old query wrapper API as transitional internally.

Acceptance criteria

team agrees on the final public gameplay-facing API

no further expansion of QueryBorrow/QueryBorrowMut

Phase 1 — Remove public Resource distinction
Goal

Make resources conceptually just globally stored components.

Files

foundation/ecs/src/resource.rs

foundation/ecs/src/world/world_core_impl.rs

foundation/ecs/src/lib.rs

foundation/ecs/src/prelude.rs

Tasks

Change world resource APIs from R: Resource to R: Component.

Remove Resource from public prelude and public docs.

Either:

delete resource.rs

or keep it internal with pub(crate) visibility

Update all internal references accordingly.

Preferred end state
pub fn insert_resource<R: Component>(&mut self, resource: R);
pub fn has_resource<R: Component>(&self) -> bool;
pub fn resource<R: Component>(&self) -> Result<&R, ResourceError>;
pub fn resource_mut<R: Component>(&mut self) -> Result<ResMut<'_, R>, ResourceError>;
pub fn remove_resource<R: Component>(&mut self) -> Option<R>;
Acceptance criteria

public docs use only Component

resource APIs compile with Component bounds

no user-facing mention of a separate resource derive or trait

Phase 2 — Unify query internals around one conceptual query model
Goal

Replace the public split between borrowed read and borrowed mutable queries.

Files

foundation/ecs/src/query/traits_and_state.rs

foundation/ecs/src/query/mod.rs

foundation/ecs/src/query/data.rs or equivalent

foundation/ecs/src/world/world_core_impl.rs

Tasks

Introduce a single public Query<Q, F = ()> type.

Make QueryBorrow and QueryBorrowMut internal or remove them.

Collapse external method naming:

iter_mut -> iter

single_mut -> single

get_mut -> get on query object

Keep mutability encoded in Q.

Implementation guidance

It is acceptable to internally keep separate code paths for readonly and mutable data. The public API must not expose that split.

Acceptance criteria

no public QueryBorrow

no public QueryBorrowMut

no public iter_mut

Query<Q, F>::iter() works for both readonly and mutable query types

Phase 3 — Rename QueryState API to match unified query semantics
Goal

Make reusable query state consistent with Query.

Files

foundation/ecs/src/query/traits_and_state.rs

Tasks

Rename:

iter_on -> iter

get_on -> get

single_on -> single

remove public iter_mut_on

remove public get_mut_on

remove public single_mut_on

Target
let mut movement = QueryState::<(&mut Position, &Velocity), With<Simulated>>::new(&mut world);

for (pos, vel) in movement.iter(&mut world) {
    pos.x += vel.x;
    pos.y += vel.y;
}
Acceptance criteria

QueryState API no longer exposes a split read/mut naming tree

docs for advanced/runtime queries use unified method names

Phase 4 — Add system parameter extraction
Goal

Make Query, Res, and ResMut real system parameters.

Files

foundation/ecs/src/system/mod.rs

foundation/ecs/src/system/params.rs

foundation/ecs/src/system/extract.rs

scheduler/runtime integration files outside ECS crate as needed

Tasks

Define a SystemParam trait or equivalent extraction interface.

Implement extraction for:

Query<Q, F>

Res<T>

ResMut<T>

Make each param contribute scheduler access metadata.

Ensure lifetimes and borrowing remain sound.

Suggested trait shape
pub trait SystemParam {
    type State;
    type Item<'w, 's>;

    fn init_state(world: &mut World) -> Self::State;
    fn access(state: &Self::State) -> QueryAccess;
    fn get_param<'w, 's>(state: &'s mut Self::State, world: &'w mut World) -> Self::Item<'w, 's>;
}

Exact shape may vary.

Acceptance criteria

a system function can receive Query, Res, ResMut

scheduler/runtime can construct those params

param access metadata is available for conflict analysis

Phase 5 — Wire Res<T> properly
Goal

Make Res<T> a real read param, not just a dormant wrapper type.

Files

foundation/ecs/src/system/params.rs

foundation/ecs/src/world/world_core_impl.rs

scheduler integration

Tasks

Ensure Res<T> is extractable from World.

Ensure scheduler access metadata marks it as a resource read.

Keep ResMut<T> as a resource write.

Acceptance criteria

gameplay systems can use Res<T>

access metadata distinguishes Res<T> vs ResMut<T>

Phase 6 — Complete query pattern support to match docs
Goal

Make implementation support the query patterns shown in target docs.

Files

foundation/ecs/src/query/query_data_impls.rs

foundation/ecs/src/query/data.rs

related iterator/runtime files

Required implementations
Add mutable entity query

(Entity, &mut T)

Add symmetric mutable/read tuple

(&A, &mut B)

Add double mutable tuple

(&mut A, &mut B)

Add optionals

Option<&T>

Option<&mut T>

(&mut A, Option<&B>)

(Entity, Option<&T>)

Strongly recommended

triples:

(&A, &B, &C)

(&mut A, &B, &C)

(&mut A, &mut B, &C)

Acceptance criteria

every query pattern shown in target docs compiles and passes tests

aliasing constraints remain enforced

mutable distinct-type assertions are correct where needed

Phase 7 — Convert prelude and root exports
Goal

Make the clean target actually visible to users.

Files

foundation/ecs/src/lib.rs

foundation/ecs/src/prelude.rs

Tasks

Export Query, Res, ResMut.

Remove QueryBorrow, QueryBorrowMut from prelude.

Remove public Resource export if still present.

Ensure docs/examples compile against the new prelude.

Acceptance criteria

users can write the target example with use ecs::prelude::*;

no internal-only types appear in the main docs path

Phase 8 — Add gameplay-facing commands injection
Goal

Support systems receiving Commands directly.

Files

foundation/ecs/src/system/params.rs

scheduler/runtime integration

foundation/ecs/src/world/commands.rs

Tasks

Add Commands as a system param.

Ensure command buffers are merged/flushed predictably.

Define flush timing clearly:

end of system

end of stage

end of schedule phase

Preferred:

flush at end of stage or equivalent deterministic boundary

Acceptance criteria

systems can declare mut commands: Commands

deferred structural changes apply at a deterministic point

command ordering remains stable and documented

Phase 9 — Add gameplay-facing event params
Goal

Provide clean system-param event API.

Files

foundation/ecs/src/system/params.rs

foundation/ecs/src/world/events.rs

scheduler/runtime integration

Tasks

Add EventReader<T>

Add EventWriter<T>

Define semantics:

writer sends into event channel

reader iterates current visible events

Keep world-level event APIs intact for advanced/runtime usage

Acceptance criteria

systems can read and write events without using raw World

world-level event channels remain available

Phase 10 — Make docs match the real API
Goal

Eliminate documentation/API mismatch.

Files

foundation/ecs/README.md

foundation/ecs/USAGE_GUIDE.md

optionally foundation/ecs/ARCHITECTURE.md

Tasks

Make README center on:

Component

Query

Res

ResMut

tags

filters

Move advanced runtime APIs to USAGE_GUIDE.md

Ensure all examples compile

Remove references to:

query_mut

QueryBorrow

QueryBorrowMut

separate public Resource concept

Acceptance criteria

README matches gameplay-facing target

USAGE_GUIDE documents runtime APIs

all doc examples compile in CI

Phase 11 — Delete obsolete API
Goal

Finish the cleanup.

Tasks

delete QueryBorrow public exposure

delete QueryBorrowMut public exposure

delete World::query_mut

delete obsolete doc references

delete transitional compatibility shims if no longer needed

Acceptance criteria

no old query split remains in public API

codebase no longer teaches the wrong model anywhere

11. Scheduler Integration Requirements

The target docs say:

The scheduler can use this to run systems in parallel.

This means the implementation must support access extraction.

Required metadata

QueryAccess must track:

component reads

component writes

resource reads

resource writes

deferred structural mutation

Required param access support
Query

derive access from Q

Res<T>

add resource read for T

ResMut<T>

add resource write for T

Commands

mark deferred structural mutation

EventWriter<T>

Recommended:

treat as resource-like write to event channel T

EventReader<T>`

Recommended:

treat as resource-like read to event channel T

Acceptance criteria

scheduler can determine conflicts from declared params

two systems that only read the same resource can run in parallel

systems writing the same component/resource cannot

systems mutating commands are treated according to scheduler policy

12. Testing Roadmap

Every migration phase must land with tests.

12.1 Compile tests / example tests

Add examples or doctests for:

canonical system example

readonly query

mutable query

filtered query

optional component query

entity-id query

resource params

commands param

event params

12.2 Unit tests

Add/keep unit coverage for:

entity lifecycle

component insertion/removal

resource insertion/removal

commands queue execution

event channel behavior

observer behavior

change tracking

index rebuild and lookup

query aliasing edge cases

12.3 Query correctness tests

Specifically add tests for:

(&A, &mut B)

(&mut A, &mut B)

(Entity, &mut T)

Option<&T>

Option<&mut T>

optional tuple patterns

12.4 Access metadata tests

Add tests that verify:

Query<(&mut Position, &Velocity)> reports write/read correctly

Res<Frame> reports resource read

ResMut<Frame> reports resource write

Commands reports deferred structural mutation

event params report expected access

12.5 Docs compile tests

All README and usage guide code examples should compile in CI.

13. Acceptance Checklist

This is the final checklist for “target reached”.

API

 Only #[derive(ecs::Component)] is needed for ECS-managed data

 Tags are empty components

 Resources are globally stored components

 Query<Q, F = ()> is the primary query API

 Res<T> and ResMut<T> are real system params

 Commands can be injected into systems

 event params exist or are clearly staged for the next milestone

 QueryBorrow and QueryBorrowMut are not public

 World::query_mut is gone

 public docs do not teach the old API

Queries

 Query<&T>

 Query<&mut T>

 Query<(Entity, &T)>

 Query<(Entity, &mut T)>

 Query<(&A, &B)>

 Query<(&mut A, &B)>

 Query<(&A, &mut B)>

 Query<(&mut A, &mut B)>

 Query<Option<&T>>

 Query<Option<&mut T>>

 Query<(&mut A, Option<&B>)>

Filters

 With<T>

 Without<T>

 tuple-composed filters

Runtime features

 world lifecycle APIs remain intact

 commands queue remains intact

 events remain intact

 change tracking remains intact

 indexes remain intact

Docs

 README is gameplay-facing

 USAGE_GUIDE is runtime-facing

 all examples compile

14. Recommended Order of Implementation

If this is being fed to an implementation agent, use this exact order:

refactor resource bounds from Resource to Component

create new system module with param extraction traits

add public Query<Q, F> param type

unify query iteration naming

implement param access metadata

implement Res<T> extraction

implement ResMut<T> extraction

add missing query data impls

update prelude and root exports

add injected Commands

add event params

update docs

delete obsolete public API

This ordering minimizes wasted work and avoids documenting an API that is not actually implemented.

15. Risks and Mitigations
Risk: query refactor becomes tangled

Mitigation:

keep old split internals temporarily

wrap them behind the new public Query

Risk: scheduler extraction gets blocked

Mitigation:

land Query, Res, ResMut extraction before commands/events params

stage commands/events later

Risk: optional mutable queries introduce aliasing bugs

Mitigation:

add focused tests

assert distinct component types where necessary

keep unsafe blocks localized and documented

Risk: docs drift from implementation again

Mitigation:

enforce docs compile in CI

make target examples part of tests

16. Final Implementation Target Summary

The final shipped user-facing model should feel like this:

use ecs::prelude::*;

#[derive(ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Velocity { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Simulated;

#[derive(ecs::Component)]
struct DeltaTime(pub f32);

#[derive(ecs::Component)]
struct Frame(pub u64);

fn tick(
    mut query: Query<(&mut Position, &Velocity), With<Simulated>>,
    dt: Res<DeltaTime>,
    mut frame: ResMut<Frame>,
) {
    for (pos, vel) in query.iter() {
        pos.x += vel.x * dt.0;
        pos.y += vel.y * dt.0;
    }

    frame.0 += 1;
}

And the advanced runtime model should still allow:

World

EntityRef

EntityMut

QueryState

events

commands

indexes

change tracking

without leaking the old query wrapper ergonomics into the main public API.

17. Final Instruction to Implementation Agent

Implement the target API exactly as described here.

Priorities:

correctness

public API cleanliness

docs/API alignment

deletion of obsolete public surface

maintain runtime features already present

Do not preserve the old public query API just for compatibility.
Compatibility is less important than landing the clean final model.