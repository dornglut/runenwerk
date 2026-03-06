use crate::plugin::Plugin;
use crate::plugins::input::domain::InputState;
use crate::plugins::replay as replay_runtime;
use crate::plugins::scene::SceneReplayArchive;
use crate::plugins::time::domain::Time;
use crate::runtime::fixed_time::{CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick};
use crate::runtime::schedules::{
    FixedUpdate, FrameEnd, PreUpdate, RenderPrepare, RenderSubmit, Startup, Update,
};
use crate::runtime::system::IntoSystemConfigs;
use crate::runtime::window::WindowState;
use crate::runtime::winit_runner;
use crate::state::{
    GameplayRuntimeConfig, SceneCatalog, SceneRegistration, SceneRuntimeState, SessionRuntimeState,
    StartupState, UiOverlayState,
};
use anyhow::{Result, anyhow};
use ecs::{Resource, World};
use engine_replay::ReplayValidationReport;
use engine_sim::{
    AuthorityRole, SimulationProfile, SimulationProfileConfig, SimulationRng, SimulationSeed,
    SimulationSessionId,
};
use scheduler::label::ScheduleLabel;
use scheduler::plan::ExecutionScheduler;
use winit::event_loop::ControlFlow;

include!("internal/runners.rs");
include!("internal/app_core.rs");
include!("internal/app_runtime.rs");
include!("internal/plugins_tuple.rs");
