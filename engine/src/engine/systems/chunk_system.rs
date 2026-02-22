use crate::engine::EngineData;
use anyhow::Result;
use glam::IVec3;

/// Scheduler node to update chunks around the player
pub fn chunk_system(data: &mut EngineData) -> Result<()> {
	// Determine player's current chunk coordinate
	let player_pos = IVec3::new(
		data.world.camera.position.x.floor() as i32,
		data.world.camera.position.y.floor() as i32,
		data.world.camera.position.z.floor() as i32,
	);

	// Update sliding window (allocates/releases chunks)
	data.gpu_resources.update_chunks(player_pos);

	Ok(())
}
