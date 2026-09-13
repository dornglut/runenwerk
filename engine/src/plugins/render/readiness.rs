use runen_ecs::Component;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderReadinessPhase {
    Loading,
    Ready,
}

#[derive(Debug, Copy, Clone, Component, runen_ecs::Resource)]
pub struct RenderReadinessState {
    pub phase: RenderReadinessPhase,
    pub stable_frames: u32,
    pub required_stable_frames: u32,
    pub elapsed_loading_seconds: f32,
    pub max_loading_seconds: f32,
}

impl Default for RenderReadinessState {
    fn default() -> Self {
        Self::loading()
    }
}

impl RenderReadinessState {
    pub const DEFAULT_REQUIRED_STABLE_FRAMES: u32 = 8;
    pub const DEFAULT_MAX_LOADING_SECONDS: f32 = 5.0;

    pub fn loading() -> Self {
        Self {
            phase: RenderReadinessPhase::Loading,
            stable_frames: 0,
            required_stable_frames: Self::DEFAULT_REQUIRED_STABLE_FRAMES,
            elapsed_loading_seconds: 0.0,
            max_loading_seconds: Self::DEFAULT_MAX_LOADING_SECONDS,
        }
    }

    pub fn ready() -> Self {
        let mut state = Self::loading();
        state.phase = RenderReadinessPhase::Ready;
        state.stable_frames = state.required_stable_frames;
        state
    }

    pub fn is_loading(&self) -> bool {
        self.phase == RenderReadinessPhase::Loading
    }

    pub fn is_ready(&self) -> bool {
        self.phase == RenderReadinessPhase::Ready
    }

    pub fn observe_render_warm_frame(&mut self, warm_frame: bool, delta_seconds: f32) -> bool {
        if self.is_ready() {
            return false;
        }

        self.elapsed_loading_seconds += delta_seconds.max(0.0);
        if warm_frame {
            self.stable_frames = self.stable_frames.saturating_add(1);
        } else {
            self.stable_frames = 0;
        }

        let stable_target = self.required_stable_frames.max(1);
        let timeout_limit = self.max_loading_seconds.max(0.0);
        let timed_out = self.elapsed_loading_seconds + 1.0e-6 >= timeout_limit;
        if self.stable_frames >= stable_target || timed_out {
            self.phase = RenderReadinessPhase::Ready;
            return true;
        }

        false
    }
}
