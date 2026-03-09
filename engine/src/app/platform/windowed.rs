use crate::app::App;
use crate::runtime::winit_runner;
use anyhow::Result;

impl App {
    pub(crate) fn run_windowed(self) -> Result<()> {
        winit_runner::run(self.into_windowed_state())
    }
}
