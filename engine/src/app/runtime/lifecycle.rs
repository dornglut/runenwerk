use crate::app::App;
use crate::app::domain::mode::AppMode;
use crate::app::domain::runner::{FixedFramesRunner, FixedTicksRunner};
use crate::runtime::schedules::{
    FrameEnd, PreUpdate, RenderPrepare, RenderSubmit, Startup, Update,
};
use crate::runtime::window::WindowState;
use anyhow::Result;

impl App {
    pub fn run(self) -> Result<()> {
        match self.mode {
            AppMode::Windowed => self.run_windowed(),
            AppMode::Headless => {
                let mut app = self;
                app.run_headless()?;
                Ok(())
            }
        }
    }

    pub fn run_for_frames(mut self, frame_count: usize) -> Result<Self> {
        self.set_runner(FixedFramesRunner::new(frame_count));
        self.run_headless()?;
        Ok(self)
    }

    pub fn run_for_ticks(mut self, tick_count: u64) -> Result<Self> {
        self.set_runner(FixedTicksRunner::new(tick_count));
        self.run_headless()?;
        Ok(self)
    }

    pub(crate) fn prepare_for_run(&mut self, headless: bool) -> Result<()> {
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
        self.scheduler.run_schedule::<PreUpdate>(&mut self.world)?;
        self.run_fixed_update_schedule()?;
        self.scheduler.run_schedule::<Update>(&mut self.world)?;
        self.scheduler
            .run_schedule::<RenderPrepare>(&mut self.world)?;
        self.scheduler
            .run_schedule::<RenderSubmit>(&mut self.world)?;
        self.scheduler.run_schedule::<FrameEnd>(&mut self.world)?;
        Ok(())
    }
}
