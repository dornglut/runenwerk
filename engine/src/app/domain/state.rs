use crate::app::domain::lifecycle::AppLifecycle;
use runen_ecs::World;

/// Physical initial Native window selection, separate from Host selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeInitialWindow {
    Primary,
    None,
}

pub(crate) struct WindowedAppState {
    pub(crate) world: World,
    pub(crate) scheduler: runen_ecs::Runtime,
    pub(crate) startup_ran: AppLifecycle,
    pub(crate) title: String,
    pub(crate) initial_native_window: NativeInitialWindow,
}
