use crate::runtime::EngineData;
use wgpu::SurfaceError;

pub fn ui_render_submit_system(data: &mut EngineData) -> anyhow::Result<()> {
    match data.gfx.render(&data.ui.draw_list) {
        Ok(()) => Ok(()),
        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
            let (w, h) = data.ui.screen_size;
            data.gfx.resize(w as u32, h as u32);
            Ok(())
        }
        Err(SurfaceError::Timeout) => Ok(()),
        Err(SurfaceError::OutOfMemory) => anyhow::bail!("surface out of memory"),
        Err(SurfaceError::Other) => Ok(()),
    }
}
