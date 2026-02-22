use ecs::{World, WorldQueryExt, init_tracing};
use glam::Vec3;
use tracing::info;

#[derive(Debug, Copy, Clone)]
struct Position(Vec3);

#[derive(Debug, Copy, Clone)]
struct Velocity(Vec3);

fn movement_system(world: &mut World, dt: f32) {
    world.query_mut::<Position, Velocity, _>(|_entity, position, velocity| {
        position.0 += velocity.0 * dt;
    });
}

fn main() {
    init_tracing();

    let mut world = World::new();

    world.spawn_many(12, |builder| {
        builder
            .with(Position(Vec3::new(
                rand::random::<f32>() * 50.0,
                rand::random::<f32>() * 50.0,
                0.0,
            )))
            .with(Velocity(Vec3::new(
                rand::random::<f32>() - 0.5,
                rand::random::<f32>() - 0.5,
                0.0,
            )))
    });

    info!("spawned {} entities", world.entity_locations.len());

    let dt = 0.16_f32;
    for tick in 0..5 {
        movement_system(&mut world, dt);

        let visible_count = world
            .query()
            .with::<Position>()
            .with::<Velocity>()
            .iter()
            .count();
        info!(tick, visible_count, "simulation step");
    }

    info!("final sample (first 5 entities)");
    for (entity, (position, velocity)) in world
        .query()
        .with::<Position>()
        .with::<Velocity>()
        .iter()
        .take(5)
    {
        info!(
            id = entity.id,
            generation = entity.generation,
            ?position,
            ?velocity,
            "entity"
        );
    }
}
