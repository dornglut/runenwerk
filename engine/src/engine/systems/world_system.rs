use crate::engine::EngineData;
use anyhow::Result;
use tracing::info;

pub fn world_update_system(engine: &mut EngineData) -> Result<()> {
	let dt = engine.time.delta_seconds();
	engine.world.camera.update(&engine.input, dt);
	Ok(())
}