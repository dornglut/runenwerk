use crate::app::domain::lifecycle::AppLifecycle;
use runen_ecs::World;
use winit::event_loop::ControlFlow;

pub(crate) struct WindowedAppState {
    pub(crate) world: World,
    pub(crate) scheduler: runen_ecs::Runtime,
    pub(crate) startup_ran: AppLifecycle,
    pub(crate) title: String,
    pub(crate) control_flow: ControlFlow,
}
