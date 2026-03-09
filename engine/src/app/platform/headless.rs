use crate::app::App;
use anyhow::Result;

impl App {
    pub(crate) fn run_headless(&mut self) -> Result<()> {
        self.prepare_for_run(true)?;

        let mut completed_frames = 0usize;
        while self.runner.next_frame(completed_frames, &self.world) {
            self.runner.before_frame(&mut self.world);
            self.run_frame()?;
            completed_frames = completed_frames.saturating_add(1);
        }

        Ok(())
    }
}
