use crate::app::App;
use anyhow::Result;

impl App {
    pub(crate) fn run_headless(&mut self) -> Result<()> {
        self.run_headless_for_frames(1)
    }

    pub(crate) fn run_headless_for_frames(&mut self, frame_count: usize) -> Result<()> {
        self.require_headless_host("headless execution")?;
        self.prepare_for_run()?;

        for _ in 0..frame_count {
            self.run_frame()?;
        }

        Ok(())
    }
}
