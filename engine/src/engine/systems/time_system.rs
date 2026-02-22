use crate::engine::EngineData;
use anyhow::Result;

pub fn time_system(engine: &mut EngineData) -> Result<()> {
	let dt = engine.time.tick();
	engine.metrics.update(dt);
	Ok(())
}
