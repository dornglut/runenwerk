use ecs::*;
use glam::Vec3;
use tracing::{info, debug};

#[derive(Debug)]
struct Position(Vec3);

#[derive(Debug)]
struct Velocity(Vec3);

fn main() {
	init_tracing();

	// 1️⃣ Create world
	let mut world = World::new();

	// 2️⃣ Register component types
	world.register_component::<Position>();
	world.register_component::<Velocity>();

	// 3️⃣ Spawn initial entities manually
	let e1 = world
		.entity()
		.with(Position(Vec3::new(0.0, 0.0, 0.0)))
		.with(Velocity(Vec3::new(1.0, 0.0, 0.0)))
		.build();

	let e2 = world
		.entity()
		.with(Position(Vec3::new(10.0, 5.0, 0.0)))
		.with(Velocity(Vec3::new(0.0, -1.0, 0.0)))
		.build();

	info!("Created initial entities: {:?}, {:?}", e1, e2);

	// 4️⃣ Query before any updates
	info!("=== Initial ECS state ===");
	for (entity, (pos, vel)) in world.query()
		.with::<Position>()
		.with::<Velocity>()
		.iter()
	{
		info!("Entity {:?}: Position={:?}, Velocity={:?}", entity, pos, vel);
	}

	world.spawn_many(5, |builder| {
		builder
			.with(Position(Vec3::new(
				rand::random::<f32>() * 10.0,
				rand::random::<f32>() * 10.0,
				0.0,
			)))
			.with(Velocity(Vec3::new(
				rand::random::<f32>() - 0.5,
				rand::random::<f32>() - 0.5,
				0.0,
			)))
	});


	// 6️⃣ Query after spawning
	info!("=== After spawn_many ===");
	for (entity, (pos, vel)) in world.query()
		.with::<Position>()
		.with::<Velocity>()
		.iter()
	{
		info!("Entity {:?}: Position={:?}, Velocity={:?}", entity, pos, vel);
	}

	// 7️⃣ Remove a component
	if let Some(removed_velocity) = world.remove_component::<Velocity>(e1) {
		info!(?e1, ?removed_velocity, "Removed Velocity component");
	}

	// 8️⃣ Query after removal
	info!("=== After removing Velocity from e1 ===");
	for (entity, (pos, vel)) in world.query()
		.with::<Position>()
		.with::<Velocity>()
		.iter()
	{
		info!("Entity {:?}: Position={:?}, Velocity={:?}", entity, pos, vel);
	}

	// 9️⃣ Final query
	info!("=== Final ECS state ===");
	for (entity, (pos, vel)) in world.query()
		.with::<Position>()
		.with::<Velocity>()
		.iter()
	{
		info!("Entity {:?}: Position={:?}, Velocity={:?}", entity, pos, vel);
	}
}
