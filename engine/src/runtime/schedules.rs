use crate::runtime::system::IntoSystemSetKey;
use scheduler::label::{ScheduleLabel, SystemSetKey};

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
pub struct PreUpdate;

impl ScheduleLabel for PreUpdate {
    fn name() -> &'static str {
        "PreUpdate"
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct FixedUpdate;

impl ScheduleLabel for FixedUpdate {
    fn name() -> &'static str {
        "FixedUpdate"
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

#[derive(Debug, Copy, Clone, Default)]
pub struct FrameEnd;

impl ScheduleLabel for FrameEnd {
    fn name() -> &'static str {
        "FrameEnd"
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoreSet {
    NetReceive,
    Input,
    Time,
    Scene,
    Simulation,
    Replication,
    FrameEnd,
}

impl IntoSystemSetKey for CoreSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::NetReceive => SystemSetKey::of::<CoreSet>("CoreSet::NetReceive"),
            Self::Input => SystemSetKey::of::<CoreSet>("CoreSet::Input"),
            Self::Time => SystemSetKey::of::<CoreSet>("CoreSet::Time"),
            Self::Scene => SystemSetKey::of::<CoreSet>("CoreSet::Scene"),
            Self::Simulation => SystemSetKey::of::<CoreSet>("CoreSet::Simulation"),
            Self::Replication => SystemSetKey::of::<CoreSet>("CoreSet::Replication"),
            Self::FrameEnd => SystemSetKey::of::<CoreSet>("CoreSet::FrameEnd"),
        }
    }
}
