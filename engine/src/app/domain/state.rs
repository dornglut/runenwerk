use crate::app::domain::lifecycle::AppLifecycle;
use runen_ecs::World;

pub(crate) struct WindowedAppState {
    pub(crate) world: World,
    pub(crate) scheduler: runen_ecs::Runtime,
    pub(crate) startup_ran: AppLifecycle,
    pub(crate) title: String,
}
