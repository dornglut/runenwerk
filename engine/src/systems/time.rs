use crate::runtime::EngineData;

pub fn time_system(data: &mut EngineData) -> anyhow::Result<()> {
    data.time.tick();
    Ok(())
}
