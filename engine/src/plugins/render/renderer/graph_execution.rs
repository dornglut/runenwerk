use super::*;
use anyhow::{Result, bail};

impl Renderer {
    pub(super) fn ensure_surface_color_write(
        &self,
        pass_id: &str,
        writes: &[crate::plugins::render::RenderResourceId],
    ) -> Result<()> {
        if writes.iter().any(|id| id.as_str() == "surface.color") {
            return Ok(());
        }
        bail!(
            "pass '{}' does not write 'surface.color'; only surface output is currently supported in core runtime path",
            pass_id
        );
    }
}
