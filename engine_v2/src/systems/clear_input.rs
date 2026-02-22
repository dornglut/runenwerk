use crate::runtime::EngineData;

pub fn clear_input_system(data: &mut EngineData) -> anyhow::Result<()> {
    data.input.clear_frame();
    Ok(())
}
