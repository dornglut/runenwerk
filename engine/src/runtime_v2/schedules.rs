use crate::runtime_v2::system::IntoSystemSetKey;
use scheduler::{ScheduleLabel, SystemSetKey};

#[derive(Debug, Copy, Clone, Default)]
pub struct Startup;

impl ScheduleLabel for Startup {
    fn name() -> &'static str {
        "Startup"
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Update;

impl ScheduleLabel for Update {
    fn name() -> &'static str {
        "Update"
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct RenderPrepare;

impl ScheduleLabel for RenderPrepare {
    fn name() -> &'static str {
        "RenderPrepare"
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct RenderSubmit;

impl ScheduleLabel for RenderSubmit {
    fn name() -> &'static str {
        "RenderSubmit"
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoreSet {
    Input,
    Time,
    FrameEnd,
}

impl IntoSystemSetKey for CoreSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::Input => SystemSetKey::of::<CoreSet>("CoreSet::Input"),
            Self::Time => SystemSetKey::of::<CoreSet>("CoreSet::Time"),
            Self::FrameEnd => SystemSetKey::of::<CoreSet>("CoreSet::FrameEnd"),
        }
    }
}
