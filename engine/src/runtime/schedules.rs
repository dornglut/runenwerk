use runen_ecs::{ScheduleLabel, SystemSet};

/// Core runtime schedules.
///
/// Frame contract (see `runtime::frame_lifecycle::run_frame`):
/// `PreUpdate` -> (`FixedStepBegin` -> `FixedUpdate`) (0..N) -> `Update` -> `RenderPrepare` -> `RenderSubmit` -> `FrameEnd`.

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct Startup;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct Update;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct PreUpdate;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct FixedStepBegin;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct FixedUpdate;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct RenderPrepare;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct RenderSubmit;

#[derive(Debug, Copy, Clone, Default, ScheduleLabel)]
pub struct FrameEnd;

#[derive(Debug, Copy, Clone, PartialEq, Eq, SystemSet)]
pub enum CoreSet {
    Input,
    Time,
    Scene,
    Simulation,
    FrameEnd,
}
