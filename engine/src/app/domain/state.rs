use ecs::World;
use scheduler::ExecutionScheduler;
use winit::event_loop::ControlFlow;

pub(crate) struct WindowedAppState {
    pub(crate) world: World,
    pub(crate) scheduler: ExecutionScheduler<World>,
    pub(crate) build_errors: Vec<anyhow::Error>,
    pub(crate) startup_ran: bool,
    pub(crate) title: String,
    pub(crate) control_flow: ControlFlow,
}
