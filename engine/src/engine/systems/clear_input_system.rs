use std::error::Error;
// file: engine/systems/input_system.rs
use crate::engine::EngineData;
use anyhow::{ Result};

pub fn clear_input_system(data: &mut EngineData) -> Result<()> {
	data.input.clear_frame();
	Ok(())
}
