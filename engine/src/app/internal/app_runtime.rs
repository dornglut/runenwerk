// Owner: Engine Runtime - App Runtime
impl App {
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

    pub fn run_for_ticks(mut self, tick_count: u64) -> Result<Self> {
        self.set_runner(FixedTicksRunner::new(tick_count));
        self.run_headless()?;
        Ok(self)
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
        if !self.world.has_resource::<SceneCatalog>() {
            self.world.insert_resource(SceneCatalog::default());
        }
        if !self.world.has_resource::<StartupState>() {
            self.world.insert_resource(StartupState::default());
        }
        if !self.world.has_resource::<SceneRuntimeState>() {
            self.world.insert_resource(SceneRuntimeState::default());
        }
        if !self.world.has_resource::<UiOverlayState>() {
            self.world.insert_resource(UiOverlayState::default());
        }
        if !self.world.has_resource::<GameplayRuntimeConfig>() {
            self.world.insert_resource(GameplayRuntimeConfig::default());
        }
        if !self.world.has_resource::<SessionRuntimeState>() {
            self.world.insert_resource(SessionRuntimeState::default());
        }
        if !self.world.has_resource::<FixedTimeConfig>() {
            self.world.insert_resource(FixedTimeConfig::default());
        }
        if !self.world.has_resource::<CatchupBudget>() {
            self.world.insert_resource(CatchupBudget::default());
        }
        if !self.world.has_resource::<FixedTimeState>() {
            self.world.insert_resource(FixedTimeState::default());
        }
        if !self.world.has_resource::<SimulationTick>() {
            self.world.insert_resource(SimulationTick::default());
        }
        if !self.world.has_resource::<SimulationProfileConfig>() {
            self.world.insert_resource(SimulationProfileConfig::default());
        }
        if !self.world.has_resource::<SimulationSessionId>() {
            self.world.insert_resource(SimulationSessionId::default());
        }
        if !self.world.has_resource::<SimulationSeed>() {
            let seed = SimulationSeed::default();
            self.world.insert_resource(seed);
            self.world.insert_resource(SimulationRng::from_seed(seed));
        } else if !self.world.has_resource::<SimulationRng>() {
            let seed = self
                .world
                .resource::<SimulationSeed>()
                .copied()
                .unwrap_or_default();
            self.world.insert_resource(SimulationRng::from_seed(seed));
        }
    }

    fn run_headless(&mut self) -> Result<()> {
        self.prepare_for_run(true)?;

        let mut completed_frames = 0usize;
        while self.runner.next_frame(completed_frames, &self.world) {
            self.runner.before_frame(&mut self.world);
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
        self.scheduler.run_schedule::<PreUpdate>(&mut self.world)?;
        self.run_fixed_update_schedule()?;
        self.scheduler.run_schedule::<Update>(&mut self.world)?;
        self.scheduler.run_schedule::<RenderPrepare>(&mut self.world)?;
        self.scheduler.run_schedule::<RenderSubmit>(&mut self.world)?;
        self.scheduler.run_schedule::<FrameEnd>(&mut self.world)?;
        Ok(())
    }

    fn ensure_build_ready(&self) -> Result<()> {
        if self.build_errors.is_empty() {
            return Ok(());
        }
        let messages: Vec<_> = self.build_errors.iter().map(ToString::to_string).collect();
        Err(anyhow!("app setup failed:\n{}", messages.join("\n")))
    }

    fn run_fixed_update_schedule(&mut self) -> Result<()> {
        let step_seconds = self
            .world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds)
            .unwrap_or(1.0 / 60.0)
            .clamp(1.0 / 240.0, 1.0 / 15.0);
        let delta_seconds = self
            .world
            .resource::<Time>()
            .map(|time| time.delta_seconds)
            .unwrap_or(step_seconds)
            .clamp(0.0, 0.25);
        let max_steps_per_frame = self
            .world
            .resource::<CatchupBudget>()
            .map(|budget| budget.max_steps_per_frame)
            .unwrap_or(4)
            .clamp(1, 16);

        {
            let mut fixed_state = self
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds =
                (fixed_state.accumulator_seconds + delta_seconds)
                    .min(step_seconds * max_steps_per_frame as f32);
            fixed_state.steps_ran_last_frame = 0;
        }

        let mut steps = 0u32;
        loop {
            let should_step = {
                let fixed_state = self
                    .world
                    .resource::<FixedTimeState>()
                    .expect("FixedTimeState should be installed");
                fixed_state.accumulator_seconds + f32::EPSILON >= step_seconds
                    && steps < max_steps_per_frame
            };
            if !should_step {
                break;
            }

            self.scheduler.run_schedule::<FixedUpdate>(&mut self.world)?;
            steps = steps.saturating_add(1);

            let mut fixed_state = self
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds -= step_seconds;
            fixed_state.steps_ran_last_frame = steps;
        }

        let saturated = {
            let fixed_state = self
                .world
                .resource::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds + f32::EPSILON >= step_seconds
        };
        if saturated {
            let mut fixed_state = self
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds = 0.0;
            fixed_state.saturated_frames = fixed_state.saturated_frames.saturating_add(1);
            tracing::warn!("fixed-step loop saturated, dropping accumulated time");
        }

        if steps > 0 {
            let mut tick = self
                .world
                .resource_mut::<SimulationTick>()
                .expect("SimulationTick should be installed");
            tick.0 = tick.0.saturating_add(steps as u64);
        }

        Ok(())
    }
}
