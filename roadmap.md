# ECS Target API Roadmap

This roadmap defines the target API, migration phases, and acceptance criteria for `foundation/ecs`.
It is intended as an implementation spec for Codex.

## 1. Objectives

The target API must satisfy these rules:

- One derive for ECS-managed data: `#[derive(ecs::Component)]`
- Tags are empty components
- Resources are globally stored singleton components
- One query type: `Query<Q, F = ()>`
- One iteration method: `iter()`
- System params are primary for gameplay code
- `World` remains the low-level runtime API
- Old borrow-wrapper query API is internal-only or removed

## 2. Canonical Gameplay Shape

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
```

## 3. Public API Target

### 3.1 Gameplay-facing

- `Component`
- `Bundle`
- `Entity`
- `Query<Q, F = ()>`
- `Res<T>`
- `ResMut<T>`
- `With<T>`
- `Without<T>`
- `Commands`
- `EventReader<T>`
- `EventWriter<T>`

### 3.2 Runtime-facing

- `World`
- `EntityRef`
- `EntityMut`
- `QueryState<Q, F = ()>`
- `QueryAccess`
- Event channel APIs
- Change tracking APIs
- Secondary index APIs

### 3.3 Internal-only

- `QueryBorrow`
- `QueryBorrowMut`
- Read/write fetch internals
- Store access helpers


## 4. Design Decisions

### 4.1 Resource model

Resources are components stored globally.

Target world signatures:

```rust
pub fn insert_resource<R: Component>(&mut self, resource: R);
pub fn has_resource<R: Component>(&self) -> bool;
pub fn resource<R: Component>(&self) -> Result<&R, ResourceError>;
pub fn resource_mut<R: Component>(&mut self) -> Result<ResMut<'_, R>, ResourceError>;
pub fn remove_resource<R: Component>(&mut self) -> Option<R>;
```

`Resource` as a public concept should be removed from docs and prelude.

### 4.2 Query model

Use one query type and one iteration method.

- Keep: `Query<Q, F>::iter()`
- Remove from public surface: `query_mut`, `iter_mut`, `single_mut`, `get_mut_on`

Mutability must come from `Q`, not from separate wrapper types.

## 5. Query Support Matrix

Required:

- `Query<&T>`
- `Query<&mut T>`
- `Query<(Entity, &T)>`
- `Query<(Entity, &mut T)>`
- `Query<(&A, &B)>`
- `Query<(&mut A, &B)>`
- `Query<(&A, &mut B)>`
- `Query<(&mut A, &mut B)>`
- `Query<Option<&T>>`
- `Query<Option<&mut T>>`
- `Query<(&mut A, Option<&B>)>`
- `Query<(Entity, Option<&T>)>`

Recommended next:

- `Query<(&A, &B, &C)>`
- `Query<(&mut A, &B, &C)>`
- `Query<(&mut A, &mut B, &C)>`

## 6. Filter API

Required:

- `With<T>`
- `Without<T>`
- Tuple composition: `(With<A>, Without<B>)`

Examples:

```rust
Query<&Position, With<Player>>
Query<&Position, Without<Disabled>>
Query<&Position, (With<Player>, Without<Disabled>)>
```

## 7. Runtime World API Target

```rust
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
```

## 8. Events, Commands, Change Tracking, Indexes

### 8.1 Events

Runtime:

```rust
pub fn configure_event_channel<T: 'static>(&mut self, config: EventChannelConfig);
pub fn emit_event<T: 'static>(&mut self, event: T);
pub fn read_events<T: 'static>(&self) -> &[T];
pub fn drain_events<T: 'static>(&mut self) -> Vec<T>;
pub fn clear_events<T: 'static>(&mut self) -> usize;
pub fn event_count<T: 'static>(&self) -> usize;
pub fn event_channel_stats<T: 'static>(&self) -> Option<EventChannelStats>;
```

Gameplay params:

- `EventReader<T>`
- `EventWriter<T>`

### 8.2 Commands

Keep queue model:

```rust
impl Commands {
    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B);
    pub fn despawn(&mut self, entity: Entity);
    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B);
    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity);
    pub fn apply(self, world: &mut World) -> Result<(), CommandError>;
}
```

Scheduler should flush at a deterministic boundary (preferred: end of stage).

### 8.3 Change tracking

```rust
pub fn current_change_tick(&self) -> u64;
pub fn component_changed_since<T: Component>(&self, tick: u64) -> bool;
pub fn resource_changed_since<R: Component>(&self, tick: u64) -> bool;
pub fn component_changes_since(&self, tick: u64) -> Vec<ComponentChangeRecord>;
pub fn resource_changes_since(&self, tick: u64) -> Vec<ResourceChangeRecord>;
```

### 8.4 Secondary indexes

Keep existing runtime APIs intact for now.
Long-term improvement: move read lookups from `&mut self` to `&self` where possible.

## 9. Migration Phases

### Phase 0: Freeze target

- Stop expanding old public query wrappers
- Treat this document as source of truth

### Phase 1: Resource alignment

Files:

- `foundation/ecs/src/resource.rs`
- `foundation/ecs/src/world/world_core_impl.rs`
- `foundation/ecs/src/lib.rs`
- `foundation/ecs/src/prelude.rs`

Tasks:

- Switch resource bounds from `R: Resource` to `R: Component`
- Remove `Resource` from public prelude/docs

### Phase 2: Unify query surface

Files:

- `foundation/ecs/src/query/mod.rs`
- `foundation/ecs/src/query/traits_and_state.rs`
- `foundation/ecs/src/query/query_data_impls.rs`
- `foundation/ecs/src/world/world_core_impl.rs`

Tasks:

- Public `Query<Q, F = ()>`
- Hide/remove `QueryBorrow`, `QueryBorrowMut`
- Collapse mut/read naming split

### Phase 3: QueryState naming cleanup

Files:

- `foundation/ecs/src/query/traits_and_state.rs`

Tasks:

- `iter_on` -> `iter`
- `get_on` -> `get`
- `single_on` -> `single`
- Remove `iter_mut_on`/`get_mut_on`/`single_mut_on` public usage

### Phase 4: System parameter extraction

Files:

- `foundation/ecs/src/system/mod.rs`
- `foundation/ecs/src/system/params.rs`
- `foundation/ecs/src/system/extract.rs`
- Scheduler integration points

Tasks:

- Add `SystemParam` extraction for `Query`, `Res`, `ResMut`
- Expose access metadata for scheduling

### Phase 5: Commands and events as params

Files:

- `foundation/ecs/src/system/params.rs`
- `foundation/ecs/src/world/*` (event + command runtime)
- Scheduler integration

Tasks:

- Inject `Commands` into systems
- Add `EventReader<T>` and `EventWriter<T>`

### Phase 6: Docs and exports

Files:

- `foundation/ecs/src/lib.rs`
- `foundation/ecs/src/prelude.rs`
- `foundation/ecs/README.md`
- `foundation/ecs/USAGE_GUIDE.md`

Tasks:

- Export only target public API
- Remove old wrapper API from docs
- Ensure examples compile

### Phase 7: Final cleanup

Tasks:

- Remove obsolete public APIs
- Remove transition shims
- Re-run acceptance checklist

## 10. Access Metadata Requirements

Scheduler conflict analysis requires:

- Component reads
- Component writes
- Resource reads
- Resource writes
- Deferred structural mutation (commands)

Expectations:

- `Query<(&mut Position, &Velocity)>` reports write/read correctly
- `Res<T>` reports resource read
- `ResMut<T>` reports resource write
- `Commands` reports deferred structural mutation
- Event params report consistent event-channel access semantics

## 11. Testing Plan

### 11.1 Query correctness

Add tests for:

- `(&A, &mut B)`
- `(&mut A, &mut B)`
- `(Entity, &mut T)`
- `Option<&T>`
- `Option<&mut T>`
- Optional tuple variants

### 11.2 Runtime behavior

Keep/add tests for:

- Entity lifecycle
- Bundle insert/remove
- Resource lifecycle
- Commands apply/flush behavior
- Event channel and observer behavior
- Change tracking
- Secondary indexes

### 11.3 Access metadata

Add tests verifying scheduler metadata for:

- Query read/write sets
- `Res` vs `ResMut`
- `Commands`
- Event params

### 11.4 Documentation

- Ensure README and usage-guide snippets compile in CI

## 12. Final Acceptance Checklist

API:

- Only `#[derive(ecs::Component)]` needed
- `Query<Q, F = ()>` is primary query API
- `Res<T>` and `ResMut<T>` are real system params
- Commands and events are available as system params
- `QueryBorrow`/`QueryBorrowMut` are not public
- `World::query_mut` is removed
- Public docs teach only the new model

Queries:

- All required query patterns compile and pass tests

Runtime:

- World lifecycle, events, commands, change tracking, indexes remain functional

Docs:

- Gameplay docs are param-first
- Runtime docs are advanced/low-level
- Examples compile

## 13. Implementation Order

Implement in this order:

1. Resource bounds (`Resource` -> `Component`)
2. System param extraction skeleton
3. Public `Query<Q, F>` surface
4. Query naming unification
5. `Res` and `ResMut` extraction
6. Missing query data impls
7. Commands/Event params
8. Prelude/root export cleanup
9. Docs alignment
10. Remove obsolete public API

## 14. Risks and Mitigations

- Query refactor complexity:
  - Keep internals temporarily split, hide behind unified public API
- Scheduler coupling:
  - Land `Query`/`Res`/`ResMut` access metadata first
- Aliasing hazards:
  - Add focused tests for mutable tuple/optional forms
- Doc drift:
  - Compile docs in CI

## 15. Final Target Summary

Gameplay code should center around:

- `Query<Q, F = ()>`
- `Res<T>`
- `ResMut<T>`
- `Commands`
- `EventReader<T>`
- `EventWriter<T>`

Runtime code should continue to support:

- `World`
- `EntityRef` and `EntityMut`
- `QueryState`
- Events
- Commands
- Change tracking
- Secondary indexes

Do not preserve old public query wrappers for compatibility if they conflict with the target API.

## Addendum: Required Clarifications

### Query ownership model

`Query<Q, F>` is a **system parameter** backed by cached internal query state.
It is not the old borrowed wrapper API and must not expose separate read/mut variants.

### Event param semantics

- `EventWriter<T>::send(event)` appends an event of type `T` to the channel for `T`
- `EventReader<T>::iter()` reads currently visible events without draining
- Draining remains part of the `World` runtime API unless explicitly promoted later

### Command flush semantics

Deferred `Commands` are flushed at the **end of the current scheduler stage**.
This is required behavior.

### World query policy

`World` retains only the advanced reusable query API through `QueryState`.
Do not keep `World::query_mut`.

If a direct world query constructor remains, it must return `QueryState<Q, F>` and use unified naming.

### Compatibility policy

Do not preserve obsolete public query wrappers for compatibility in the final API.

Temporary transition shims are allowed during implementation, but the final public API must remove:

- `QueryBorrow`
- `QueryBorrowMut`
- `World::query_mut`
- `iter_mut`
- `single_mut`