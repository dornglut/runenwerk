use crate::plugin::Plugin;
use crate::plugins::input::domain::InputState;
use crate::plugins::time::domain::Time;
use crate::runtime_v2::schedules::{RenderPrepare, RenderSubmit, Startup, Update};
use crate::runtime_v2::system::IntoSystemConfigs;
use crate::runtime_v2::window::WindowState;
use crate::runtime_v2::winit_runner;
use anyhow::{Result, anyhow};
use ecs_v2::{Resource, World};
use scheduler::{ExecutionScheduler, ScheduleLabel};
use winit::event_loop::ControlFlow;

const DEFAULT_WINDOW_TITLE: &str = "Grotto Quest - Engine";

pub trait AppRunner: Send {
    fn next_frame(&mut self, completed_frames: usize) -> bool;
}

#[derive(Debug, Clone)]
pub struct FixedFramesRunner {
    frames_remaining: usize,
}

impl FixedFramesRunner {
    pub fn new(frame_count: usize) -> Self {
        Self {
            frames_remaining: frame_count,
        }
    }
}

impl AppRunner for FixedFramesRunner {
    fn next_frame(&mut self, _completed_frames: usize) -> bool {
        if self.frames_remaining == 0 {
            return false;
        }
        self.frames_remaining -= 1;
        true
    }
}

enum AppMode {
    Windowed,
    Headless,
}

pub(crate) struct WindowedAppState {
    pub(crate) world: World,
    pub(crate) scheduler: ExecutionScheduler<World>,
    pub(crate) build_errors: Vec<anyhow::Error>,
    pub(crate) startup_ran: bool,
    pub(crate) title: String,
    pub(crate) control_flow: ControlFlow,
}

pub struct App {
    world: World,
    scheduler: ExecutionScheduler<World>,
    runner: Box<dyn AppRunner>,
    build_errors: Vec<anyhow::Error>,
    startup_ran: bool,
    mode: AppMode,
    title: String,
    control_flow: ControlFlow,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_mode(AppMode::Windowed)
    }

    pub fn headless() -> Self {
        Self::with_mode(AppMode::Headless)
    }

    fn with_mode(mode: AppMode) -> Self {
        let title = DEFAULT_WINDOW_TITLE.to_string();
        let mut app = Self {
            world: World::new(),
            scheduler: ExecutionScheduler::new(),
            runner: Box::new(FixedFramesRunner::new(1)),
            build_errors: Vec::new(),
            startup_ran: false,
            mode,
            title: title.clone(),
            control_flow: ControlFlow::Poll,
        };
        app.install_builtin_resources();
        app
    }

    pub fn add_plugin<P>(&mut self, plugin: P) -> &mut Self
    where
        P: Plugin + 'static,
    {
        plugin.build(self);
        self
    }

    pub fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_plugins<P>(&mut self, plugins: P) -> &mut Self
    where
        P: IntoPlugins,
    {
        plugins.add_to_app(self);
        self
    }

    pub fn add_systems<L, S, Marker>(&mut self, _schedule: L, systems: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystemConfigs<Marker>,
    {
        systems.register::<L>(&mut self.world, &mut self.scheduler, &mut self.build_errors);
        self
    }

    pub fn init_resource<R>(&mut self) -> &mut Self
    where
        R: Resource + Default,
    {
        if self.world.resource::<R>().is_err() {
            self.world.insert_resource(R::default());
        }
        self
    }

    pub fn insert_resource<R>(&mut self, value: R) -> &mut Self
    where
        R: Resource,
    {
        self.world.insert_resource(value);
        self
    }

    pub fn set_runner<R>(&mut self, runner: R) -> &mut Self
    where
        R: AppRunner + 'static,
    {
        self.runner = Box::new(runner);
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = title.into();
        if let Ok(mut window) = self.world.resource_mut::<WindowState>() {
            window.set_title(self.title.clone());
        }
        self
    }

    pub fn with_control_flow(&mut self, control_flow: ControlFlow) -> &mut Self {
        self.control_flow = control_flow;
        self
    }

    pub fn run(mut self) -> Result<()> {
        match self.mode {
            AppMode::Windowed => winit_runner::run(self.into_windowed_state()),
            AppMode::Headless => {
                self.run_headless()?;
                Ok(())
            }
        }
    }

    pub fn run_for_frames(mut self, frame_count: usize) -> Result<Self> {
        self.set_runner(FixedFramesRunner::new(frame_count));
        self.run_headless()?;
        Ok(self)
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub(crate) fn into_windowed_state(self) -> WindowedAppState {
        WindowedAppState {
            world: self.world,
            scheduler: self.scheduler,
            build_errors: self.build_errors,
            startup_ran: self.startup_ran,
            title: self.title,
            control_flow: self.control_flow,
        }
    }

    fn install_builtin_resources(&mut self) {
        if self.world.resource::<Time>().is_err() {
            self.world.insert_resource(Time::new());
        }
        if self.world.resource::<InputState>().is_err() {
            self.world.insert_resource(InputState::new());
        }
        if self.world.resource::<WindowState>().is_err() {
            let state = match self.mode {
                AppMode::Windowed => WindowState::windowed(self.title.clone()),
                AppMode::Headless => WindowState::headless(self.title.clone()),
            };
            self.world.insert_resource(state);
        }
    }

    fn run_headless(&mut self) -> Result<()> {
        self.prepare_for_run(true)?;

        let mut completed_frames = 0usize;
        while self.runner.next_frame(completed_frames) {
            self.run_frame()?;
            completed_frames = completed_frames.saturating_add(1);
        }

        Ok(())
    }

    pub(crate) fn prepare_for_run(&mut self, headless: bool) -> Result<()> {
        self.ensure_build_ready()?;
        if let Ok(mut window) = self.world.resource_mut::<WindowState>() {
            window.set_headless(headless);
            window.redraw_requested = false;
            window.close_requested = false;
            window.title = self.title.clone();
        }
        if !self.startup_ran {
            self.scheduler.run_schedule::<Startup>(&mut self.world)?;
            self.startup_ran = true;
        }
        Ok(())
    }

    pub(crate) fn run_frame(&mut self) -> Result<()> {
        self.scheduler.run_schedule::<Update>(&mut self.world)?;
        self.scheduler
            .run_schedule::<RenderPrepare>(&mut self.world)?;
        self.scheduler
            .run_schedule::<RenderSubmit>(&mut self.world)?;
        Ok(())
    }

    fn ensure_build_ready(&self) -> Result<()> {
        if self.build_errors.is_empty() {
            return Ok(());
        }
        let messages: Vec<_> = self.build_errors.iter().map(ToString::to_string).collect();
        Err(anyhow!("app setup failed:\n{}", messages.join("\n")))
    }
}

pub trait IntoPlugins {
    fn add_to_app(self, app: &mut App);
}

impl<P> IntoPlugins for P
where
    P: Plugin + 'static,
{
    fn add_to_app(self, app: &mut App) {
        app.add_plugin(self);
    }
}

impl IntoPlugins for Box<dyn Plugin> {
    fn add_to_app(self, app: &mut App) {
        app.add_boxed_plugin(self);
    }
}

impl IntoPlugins for Vec<Box<dyn Plugin>> {
    fn add_to_app(self, app: &mut App) {
        for plugin in self {
            app.add_boxed_plugin(plugin);
        }
    }
}

macro_rules! impl_into_plugins_tuple {
    ($(($name:ident, $index:tt)),+ $(,)?) => {
        impl<$($name),+> IntoPlugins for ($($name,)+)
        where
            $($name: IntoPlugins,)+
        {
            fn add_to_app(self, app: &mut App) {
                $(
                    self.$index.add_to_app(app);
                )+
            }
        }
    };
}

impl_into_plugins_tuple!((A, 0), (B, 1));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_into_plugins_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
