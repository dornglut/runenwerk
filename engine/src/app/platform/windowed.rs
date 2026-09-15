use crate::app::App;
use crate::runtime::winit_runner;
use anyhow::Result;

impl App {
    pub(crate) fn run_windowed(mut self) -> Result<()> {
        self.admit_composition()?;
        winit_runner::run(self.into_windowed_state())
    }
}
