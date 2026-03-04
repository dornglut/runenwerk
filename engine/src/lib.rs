pub mod app;
mod plugin;
pub mod plugins;
pub mod prelude;
mod runtime;
pub mod state;
pub mod utils;

pub use app::{App, AppRunner, FixedFramesRunner, FixedTicksRunner};
pub use plugin::Plugin;
pub use plugins::fixed_step::FixedStepPlugin;
pub use plugins::input::domain::InputState;
pub use plugins::net::{
    NetworkClientInbox, NetworkClientOutbox, NetworkClientPlugin, NetworkDiagnostics,
    NetworkServerInbox, NetworkServerOutbox, NetworkServerPlugin, PredictionDiagnostics,
    PredictionPlugin, ReplicationDiagnostics, ReplicationPlugin,
};
pub use plugins::time::domain::Time;
pub use runtime::{
    CatchupBudget, Commands, CoreSet, FixedTimeConfig, FixedTimeState, FixedUpdate, FrameEnd,
    PreUpdate, Query, RenderPrepare, RenderSubmit, Res, ResMut, SimulationTick, Startup,
    SystemConfigExt, Update, WindowState,
};
pub use runtime::{WorldMut, WorldRef};
pub use scheduler::SystemSet;
pub use state::{
    DebugMetricsState, GameplayRuntimeConfig, OverlayDrawCmd, OverlayDrawList, RegisteredScene,
    SceneCatalog, SceneHandle, SceneRegistration, SceneRuntimeState, StartupPhase, StartupState,
    UiOverlayState,
};
