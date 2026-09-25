use crate::app::App;
use crate::app::domain::mode::AppMode;
use crate::runtime::frame_lifecycle::{run_frame as run_runtime_frame, run_startup_if_needed};
use anyhow::{Result, anyhow};

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
        self.require_headless_host("run_for_frames")?;
        self.run_headless_for_frames(frame_count)?;
        Ok(self)
    }

    pub(crate) fn require_headless_host(&self, operation: &str) -> Result<()> {
        if matches!(self.mode, AppMode::Windowed) {
            return Err(anyhow!(
                "{operation} requires App::headless(); Host selection is stable and advancement does not change it"
            ));
        }
        Ok(())
    }

    pub(crate) fn prepare_for_run(&mut self) -> Result<()> {
        self.admit_composition()?;
        self.prepare_lifecycle_for_execution()?;
        run_startup_if_needed(&mut self.world, &mut self.scheduler, &mut self.lifecycle)
    }

    pub(crate) fn run_frame(&mut self) -> Result<()> {
        self.lifecycle.require_running()?;
        run_runtime_frame(&mut self.world, &mut self.scheduler)
    }
}
