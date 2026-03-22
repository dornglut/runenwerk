#[derive(Debug, Clone, Copy, Default)]
pub struct PreparedFrameContext {
    pub frame_index: u64,
    pub flow_registry_revision: u64,
    pub shader_registry_revision: u64,
    pub prepare_epoch: u64,
}
