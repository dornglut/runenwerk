#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct R0ReplicatedId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct R0ReplicatedPosition {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct R0ReplicatedHealth(u16);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct R0ActorView {
    replicated_id: u64,
    x: i32,
    y: i32,
    health: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct R0ReplicatedView {
    actors: Vec<R0ActorView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct R0ReplicatedStateProduct {
    bytes: Vec<u8>,
}

impl R0ReplicatedStateProduct {
    fn prepare(view: &R0ReplicatedView) -> Result<Self, R0FormationError> {
        let bytes = postcard::to_allocvec(view)
            .map_err(|error| R0FormationError::Encoding(error.to_string()))?;
        Ok(Self { bytes })
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn accounted_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn decode(&self) -> R0ReplicatedView {
        postcard::from_bytes(&self.bytes).expect("formed R0 product must decode")
    }
}

#[derive(Debug, Default)]
struct R0ActiveReplicatedStateProduct {
    active: Option<R0ReplicatedStateProduct>,
}

impl R0ActiveReplicatedStateProduct {
    fn active(&self) -> Option<&R0ReplicatedStateProduct> {
        self.active.as_ref()
    }

    fn activate(&mut self, candidate: R0ReplicatedStateProduct) {
        let _previous = std::mem::replace(&mut self.active, Some(candidate));
    }
}

#[derive(Debug, PartialEq, Eq)]
enum R0FormationError {
    DuplicateReplicatedId(u64),
    Encoding(String),
}

fn r0_prepare_from_world(world: &World) -> Result<R0ReplicatedStateProduct, R0FormationError> {
    let query = world.query::<(
        &R0ReplicatedId,
        &R0ReplicatedPosition,
        &R0ReplicatedHealth,
    )>();
    let mut actors = query
        .iter(world)
        .map(|(id, position, health)| R0ActorView {
            replicated_id: id.0,
            x: position.x,
            y: position.y,
            health: health.0,
        })
        .collect::<Vec<_>>();
    actors.sort_by_key(|actor| actor.replicated_id);

    for pair in actors.windows(2) {
        if pair[0].replicated_id == pair[1].replicated_id {
            return Err(R0FormationError::DuplicateReplicatedId(
                pair[0].replicated_id,
            ));
        }
    }

    R0ReplicatedStateProduct::prepare(&R0ReplicatedView { actors })
}

fn r0_spawn_actor(
    world: &mut World,
    replicated_id: u64,
    x: i32,
    y: i32,
    health: u16,
) -> runen_ecs::Entity {
    world
        .spawn((
            R0ReplicatedId(replicated_id),
            R0ReplicatedPosition { x, y },
            R0ReplicatedHealth(health),
        ))
        .expect("R0 proof actor should spawn")
}

#[test]
fn replicated_view_r0_forms_complete_order_independent_product() {
    let mut first_world = World::new();
    r0_spawn_actor(&mut first_world, 20_002, 20, 2, 80);
    r0_spawn_actor(&mut first_world, 10_001, 10, 1, 90);

    let mut second_world = World::new();
    r0_spawn_actor(&mut second_world, 10_001, 10, 1, 90);
    r0_spawn_actor(&mut second_world, 20_002, 20, 2, 80);

    let first_product = r0_prepare_from_world(&first_world).expect("first full view should form");
    let second_product =
        r0_prepare_from_world(&second_world).expect("second full view should form");

    assert_eq!(
        first_product.bytes(),
        second_product.bytes(),
        "formed product must depend on stable replicated identity and state, not ECS row/spawn order"
    );

    let decoded = first_product.decode();
    assert_eq!(
        decoded.actors,
        vec![
            R0ActorView {
                replicated_id: 10_001,
                x: 10,
                y: 1,
                health: 90,
            },
            R0ActorView {
                replicated_id: 20_002,
                x: 20,
                y: 2,
                health: 80,
            },
        ]
    );
}

#[test]
fn replicated_view_r0_prepare_failure_preserves_active_and_activation_is_complete() {
    let mut world = World::new();
    let first = r0_spawn_actor(&mut world, 10_001, 10, 1, 90);
    r0_spawn_actor(&mut world, 20_002, 20, 2, 80);

    let initial = r0_prepare_from_world(&world).expect("initial full view should form");
    let initial_bytes = initial.bytes().to_vec();
    let mut active = R0ActiveReplicatedStateProduct::default();
    active.activate(initial);

    let duplicate = r0_spawn_actor(&mut world, 10_001, 99, 99, 1);
    assert_eq!(
        r0_prepare_from_world(&world),
        Err(R0FormationError::DuplicateReplicatedId(10_001))
    );
    assert_eq!(
        active.active().expect("initial product remains active").bytes(),
        initial_bytes.as_slice(),
        "failed preparation must not mutate the active product"
    );

    world
        .despawn(duplicate)
        .expect("duplicate proof entity should despawn");

    let cursor = world.current_change_cursor();
    world
        .require_mut::<R0ReplicatedPosition>(first)
        .expect("proof actor position should exist")
        .x = 42;
    assert!(
        world
            .component_changed_since::<R0ReplicatedPosition>(cursor)
            .expect("current-world change cursor must be valid"),
        "change observation may signal rebuild work"
    );

    let updated = r0_prepare_from_world(&world).expect("updated full view should form");
    assert_eq!(
        active.active().expect("old product stays active").bytes(),
        initial_bytes.as_slice(),
        "candidate formation alone must not publish partial state"
    );

    let accounted = runen_net::replication::AccountedState::new(
        updated.clone(),
        updated.accounted_bytes(),
    );
    assert_eq!(
        accounted.accounted_bytes(),
        accounted.state().bytes().len(),
        "R0 accounting is the exact retained encoded product byte length"
    );

    active.activate(updated);
    let decoded = active
        .active()
        .expect("updated product should be active")
        .decode();
    assert_eq!(decoded.actors.len(), 2);
    assert_eq!(decoded.actors[0].replicated_id, 10_001);
    assert_eq!(decoded.actors[0].x, 42);
    assert_eq!(decoded.actors[1].replicated_id, 20_002);
}
