// engine/systems/render_system.rs
use crate::engine::EngineData;
use anyhow::Result;

pub fn render_system(data: &mut EngineData) -> Result<()> {
	if !data.gfx.headless {
		// Render might return a Result; propagate errors if needed
		data.gfx.render(&data.gpu_resources.camera)?;
	}
	Ok(())
}
