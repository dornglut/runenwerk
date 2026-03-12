use crate::plugins::render::resources::RenderFrameDataRegistry;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SdfExtractedFrame {
    pub field_count: u32,
}

pub fn extract_sdf_frame(_frame_data: &RenderFrameDataRegistry<'_>) -> SdfExtractedFrame {
    SdfExtractedFrame::default()
}
