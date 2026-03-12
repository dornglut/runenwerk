use crate::plugins::render::domain::GfxFrameTimings;

#[derive(Debug, Clone, Copy, Default, ecs::Component)]
pub struct RenderDebugTimingsState {
    pub workload_ms: f32,
    pub total_ms: f32,
}

impl RenderDebugTimingsState {
    pub fn observe_frame_timings(&mut self, timings: GfxFrameTimings) {
        self.workload_ms = timings.renderer.prepare_ui_ms
            + timings.renderer.prepare_mesh_ms
            + timings.renderer.world_prepare_ms
            + timings.renderer.encode_submit_ms;
        self.total_ms = timings.acquire_ms + self.workload_ms + timings.present_ms;
    }
}
