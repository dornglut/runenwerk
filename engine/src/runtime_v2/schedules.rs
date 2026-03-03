use scheduler::ScheduleLabel;

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
