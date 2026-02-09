// main.rs
use ecs::{World, Component};

fn main() {
	let mut world = World::new();

	// Define your components
	#[derive(Debug)]
	pub struct Position { x: i32, y: i32 }
	#[derive(Debug)]
	pub struct Velocity { x: i32, y: i32 }
	#[derive(Debug)]
	pub struct Health { value: i32 }
	#[derive(Debug)]
	pub struct Attack { value: i32 }

	// Spawn entities
	let player = world.spawn();
	let enemy = world.spawn();

	// Add components
	world.add_component(player, Position { x: 0, y: 0 });
	world.add_component(player, Velocity { x: 3, y: 1 });
	world.add_component(enemy, Position { x: 10, y: 2 });
	world.add_component(enemy, Velocity { x: 2, y: -4 });

	// --- Movement system ---
	// Iterate over entities that have both Position and Velocity
	let mut movers = Vec::new();

	for archetype in &mut world.archetypes {
		if let (Some(pos_vec), Some(vel_vec)) = (
			archetype.get_component_vec_mut::<Position>(),
			archetype.get_component_vec::<Velocity>(), // only need immutable
		) {
			for i in 0..pos_vec.len() {
				movers.push((archetype.entities[i], &mut pos_vec[i], &vel_vec[i]));
			}
		}
	}

	// Apply movement
	for (entity, pos, vel) in movers {
		pos.x += vel.x;
		pos.y += vel.y;
		println!("Entity {:?} moved to ({}, {})", entity, pos.x, pos.y);
	}
}
