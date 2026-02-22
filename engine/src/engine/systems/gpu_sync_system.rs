use crate::engine::EngineData;
use glam::IVec3;

pub fn gpu_sync_system(engine: &mut EngineData) -> anyhow::Result<()> {
	// Update GPU camera uniform
	engine
		.gpu_resources
		.update_camera(&engine.gfx.ctx.queue, &engine.world.camera);

	// Update chunks around the player
	let player_pos: IVec3 = engine.world.camera.position.floor().as_ivec3();
	engine.gpu_resources.update_chunks(player_pos);

	Ok(())
}
