// main.rs
use ecs::{query3, spawn_entity, World};

#[derive(Debug)]
struct Position { x: f32, y: f32 }

#[derive(Debug)]
struct Velocity { dx: f32, dy: f32 }

#[derive(Debug)]
struct Health { value: f32 }

fn main() {
	let mut world = World::new();

	// spawn entities directly via the world
	let e1 = spawn_entity!(
		world,
		Position { x: 0.0, y: 0.0 },
		Velocity { dx: 0.0, dy: 0.0 },
		Health { value: 100.0 },
	);

	let e2 = spawn_entity!(
		world,
		Position { x: 5.0, y: 10.0 },
		Velocity { dx: 4.0, dy: 2.0 },
		Health { value: 29.0 },
	);

	let e3 = spawn_entity!(
		world,
		Position { x: 12.0, y: 4.0 },
		Velocity { dx: 9.0, dy: 8.0 },
		Health { value: 80.0 },
	);

	let results = query3::<Position, Velocity, Health>(&world);

	for (entity, (pos, vel, health)) in results {
		println!("{:?} {:?} {:?} {:?}", entity, pos, vel, health);
	}
}
