// main.rs
use ecs::{AIState, BehaviorFn, Chunk, ChunkBuilder};

// src/main.rs
use std::sync::Arc;
use glam::Vec3;

// Example behavior functions
fn player_input(chunk: &mut Chunk, i: usize, dt: f32) {
	if let Some(input) = chunk.input[i] {
		chunk.velocities[i] += input * dt * 5.0;
	}
}

fn enemy_ai(chunk: &mut Chunk, i: usize, dt: f32) {
	if let Some(ai) = &mut chunk.ai_state[i] {
		let target_pos = chunk.positions[ai.target_idx];
		let dir = (target_pos - chunk.positions[i]).normalize();
		chunk.velocities[i] = dir * dt * 2.0; // simple chase
		ai.cooldown -= dt;
	}
}

fn main() {
	let mut world = Chunk::new();

	// Shared pipelines
	let player_pipeline: Arc<Vec<BehaviorFn>> = Arc::new(vec![player_input]);
	let enemy_pipeline: Arc<Vec<BehaviorFn>> = Arc::new(vec![enemy_ai]);

	// Spawn player at the center
	let player_idx = ChunkBuilder::new(&mut world)
		.position(Vec3::new(50.0, 50.0, 0.0))
		.pipeline(player_pipeline.clone())
		.input(Vec3::new(1.0, 0.0, 0.0)) // example input vector
		.build();

	// Spawn 5 enemies around the player
	for i in 0..5 {
		ChunkBuilder::new(&mut world)
			.position(Vec3::new(50.0 + i as f32 * 5.0, 60.0, 0.0))
			.pipeline(enemy_pipeline.clone())
			.ai_state(AIState { target_idx: player_idx, cooldown: 0.0 })
			.build();
	}

	// Run update loop for 10 frames
	let dt = 0.2; // ~60 FPS
	for frame in 0..10 {
		world.update(dt);

		// Print positions after this frame
		println!("Frame {}:", frame + 1);
		println!("  Player position: {:?}", world.positions[player_idx]);
		for i in 1..=5 {
			println!("  Enemy {} position: {:?}", i, world.positions[i]);
		}
	}
}
