use crate::app::App;
use crate::app::domain::mode::AppMode;
use crate::app::domain::runner::{FixedFramesRunner, FixedStepsRunner};
use crate::plugins::fixed_step::fixed_step_is_active;
use crate::runtime::frame_lifecycle::{
    prepare_world_for_run, run_frame as run_runtime_frame, run_startup_if_needed,
};
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
        self.set_runner(FixedFramesRunner::new(frame_count));
        self.run_headless()?;
        Ok(self)
    }

    pub fn run_for_fixed_steps(mut self, step_count: u64) -> Result<Self> {
        self.require_headless_host("run_for_fixed_steps")?;
        if !fixed_step_is_active(&self.world) {
            return Err(anyhow!(
                "run_for_fixed_steps requires FixedStepPlugin to select fixed cadence"
            ));
        }
        self.set_runner(FixedStepsRunner::new(step_count));
        self.run_headless()?;
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

    pub(crate) fn prepare_for_run(&mut self, headless: bool) -> Result<()> {
        self.admit_composition()?;
        prepare_world_for_run(&mut self.world, &self.title, headless);
        run_startup_if_needed(&mut self.world, &mut self.scheduler, &mut self.startup_ran)
    }

    pub(crate) fn run_frame(&mut self) -> Result<()> {
        run_runtime_frame(&mut self.world, &mut self.scheduler)
    }
}
