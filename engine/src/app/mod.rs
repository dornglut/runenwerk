use crate::plugin::Plugin;
use crate::runtime_v2::schedules::{RenderPrepare, RenderSubmit, Startup, Update};
use crate::runtime_v2::system::IntoSystem;
use anyhow::{Result, anyhow};
use ecs_v2::{Resource, World};
use scheduler::{ExecutionScheduler, ScheduleLabel};

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

pub struct App {
    world: World,
    scheduler: ExecutionScheduler<World>,
    runner: Box<dyn AppRunner>,
    build_errors: Vec<anyhow::Error>,
    startup_ran: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            scheduler: ExecutionScheduler::new(),
            runner: Box::new(FixedFramesRunner::new(1)),
            build_errors: Vec::new(),
            startup_ran: false,
        }
    }

    pub fn add_plugin<P>(&mut self, plugin: P) -> &mut Self
    where
        P: Plugin + 'static,
    {
        plugin.build(self);
        self
    }

    pub fn add_systems<L, S, Marker>(&mut self, _schedule: L, system: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystem<Marker>,
    {
        match system.into_registered_system::<L>(&mut self.world) {
            Ok(registered) => {
                self.scheduler.add_system(registered);
            }
            Err(err) => self.build_errors.push(err),
        }
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

    pub fn run(mut self) -> Result<()> {
        self.run_internal()?;
        Ok(())
    }

    pub fn run_for_frames(mut self, frame_count: usize) -> Result<Self> {
        self.set_runner(FixedFramesRunner::new(frame_count));
        self.run_internal()?;
        Ok(self)
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    fn run_internal(&mut self) -> Result<()> {
        if !self.build_errors.is_empty() {
            let messages: Vec<_> = self.build_errors.iter().map(ToString::to_string).collect();
            return Err(anyhow!("app setup failed:\n{}", messages.join("\n")));
        }

        if !self.startup_ran {
            self.scheduler.run_schedule::<Startup>(&mut self.world)?;
            self.startup_ran = true;
        }

        let mut completed_frames = 0usize;
        while self.runner.next_frame(completed_frames) {
            self.scheduler.run_schedule::<Update>(&mut self.world)?;
            self.scheduler
                .run_schedule::<RenderPrepare>(&mut self.world)?;
            self.scheduler
                .run_schedule::<RenderSubmit>(&mut self.world)?;
            completed_frames = completed_frames.saturating_add(1);
        }

        Ok(())
    }
}
