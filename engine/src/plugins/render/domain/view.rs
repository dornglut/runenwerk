#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSurfaceView {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

impl RenderSurfaceView {
    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            scale_factor: 1.0,
        }
    }
}
