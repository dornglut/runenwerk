use crate::plugins::render::renderer::GfxFrameTimings;
use runen_ecs::Component;

#[derive(Debug, Clone, Copy, Component, runen_ecs::Resource)]
pub struct DebugMetricsState {
    pub visible: bool,
    pub fps_ema: f32,
    pub frame_ms_ema: f32,
    pub last_timings: Option<GfxFrameTimings>,
}

impl Default for DebugMetricsState {
    fn default() -> Self {
        Self {
            visible: false,
            fps_ema: 0.0,
            frame_ms_ema: 0.0,
            last_timings: None,
        }
    }
}

impl DebugMetricsState {
    pub fn observe_frame_delta(&mut self, delta_seconds: f32) {
        let safe_dt = delta_seconds.max(1.0 / 1000.0);
        let fps = (1.0 / safe_dt).clamp(0.0, 2000.0);
        let frame_ms = (safe_dt * 1000.0).clamp(0.0, 1000.0);
        let alpha = 0.12;
        if self.fps_ema <= f32::EPSILON {
            self.fps_ema = fps;
        } else {
            self.fps_ema = self.fps_ema + (fps - self.fps_ema) * alpha;
        }
        if self.frame_ms_ema <= f32::EPSILON {
            self.frame_ms_ema = frame_ms;
        } else {
            self.frame_ms_ema = self.frame_ms_ema + (frame_ms - self.frame_ms_ema) * alpha;
        }
    }
}
