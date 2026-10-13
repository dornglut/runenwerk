use crate::app::App;
use crate::app::domain::host::AppHostSelection;
use crate::runtime::frame_pacing::{FramePacingPolicyResource, FramePacingRuntimeStateResource};
use crate::runtime::window::PrimaryWindowInitialSizePxResource;
use crate::runtime::winit_runner;
use anyhow::Result;

pub trait AppNativeHostExt {
    fn with_frame_pacing(&mut self, policy: FramePacingPolicyResource) -> &mut Self;
    fn with_primary_window_size_px(&mut self, size_px: (u32, u32)) -> &mut Self;
}

impl AppNativeHostExt for App {
    fn with_frame_pacing(&mut self, policy: FramePacingPolicyResource) -> &mut Self {
        if matches!(self.host_selection, AppHostSelection::Headless) {
            self.record_missing_capability("with_frame_pacing", "native-window Host");
            return self;
        }

        self.world.insert_resource(policy);
        if let Ok(runtime_state) = self.world.resource_mut::<FramePacingRuntimeStateResource>() {
            runtime_state.observe_policy(policy);
        }
        self
    }

    fn with_primary_window_size_px(&mut self, size_px: (u32, u32)) -> &mut Self {
        if matches!(self.host_selection, AppHostSelection::Headless) {
            self.record_missing_capability("with_primary_window_size_px", "native-window Host");
            return self;
        }

        self.world
            .insert_resource(PrimaryWindowInitialSizePxResource::new(size_px));
        self
    }
}

impl App {
    pub(crate) fn prepare_native_ui_host(&mut self) -> Result<()> {
        use crate::plugins::ui::UiRuntimeSlotsResource;
        if !self.world.has_resource::<UiRuntimeSlotsResource>() {
            return Ok(());
        }
        let clock = self.native_ui_clock();
        self.world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .configure_host_clock(clock)
            .map_err(|error| {
                anyhow::anyhow!("native UI registration rejected: {}", error.message())
            })
    }

    /// Host selection, not resource presence, determines native UI clock binding.
    pub(crate) fn native_ui_clock(
        &mut self,
    ) -> Option<crate::runtime::native_host_wake::NativeHostClock> {
        if matches!(self.host_selection, AppHostSelection::Headless) {
            return None;
        }
        use crate::runtime::native_host_wake::NativeHostClock;
        if !self.world.has_resource::<NativeHostClock>() {
            self.world.insert_resource(NativeHostClock::new());
        }
        Some(
            self.world
                .resource::<NativeHostClock>()
                .expect("selected native App owns its just-installed clock")
                .clone(),
        )
    }

    pub(crate) fn prepare_windowed_frame_pacing(&mut self) {
        let policy = self
            .world
            .resource::<FramePacingPolicyResource>()
            .ok()
            .copied()
            .unwrap_or_default();

        if !self.world.has_resource::<FramePacingRuntimeStateResource>() {
            self.world
                .insert_resource(FramePacingRuntimeStateResource::default());
        }

        if let Ok(runtime_state) = self.world.resource_mut::<FramePacingRuntimeStateResource>() {
            runtime_state.observe_policy(policy);
        }
    }

    pub(crate) fn run_windowed(mut self) -> Result<()> {
        self.admit_composition()?;
        self.prepare_lifecycle_for_execution()?;
        // Direct resource mounting is a real public ingress too. Never silently
        // run already-settled ManualClock slots through a native host.
        self.prepare_native_ui_host()?;
        winit_runner::run(self.into_windowed_state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame_pacing::FramePacingMode;

    #[test]
    fn explicit_primary_window_size_is_native_host_configuration() {
        let mut app = App::new();
        app.with_primary_window_size_px((1600, 1200));

        assert_eq!(
            app.world()
                .resource::<PrimaryWindowInitialSizePxResource>()
                .expect("native Host size request should be installed")
                .size_px(),
            (1600, 1200)
        );
    }

    #[test]
    fn headless_host_rejects_primary_window_size_configuration() {
        let mut app = App::headless();
        app.with_primary_window_size_px((1600, 1200));

        assert!(
            app.world()
                .resource::<PrimaryWindowInitialSizePxResource>()
                .is_err()
        );
        assert!(
            app.admit_composition().is_err(),
            "native-window configuration must reject headless Host selection"
        );
    }

    #[test]
    fn windowed_host_preparation_materializes_default_runtime_state_without_policy_resource() {
        let mut app = App::new();

        assert!(app.world().resource::<FramePacingPolicyResource>().is_err());
        assert!(
            app.world()
                .resource::<FramePacingRuntimeStateResource>()
                .is_err()
        );

        app.prepare_windowed_frame_pacing();

        assert!(
            app.world().resource::<FramePacingPolicyResource>().is_err(),
            "implicit default pacing should not manufacture explicit policy state"
        );
        assert_eq!(
            app.world()
                .resource::<FramePacingRuntimeStateResource>()
                .expect("windowed Host preparation should provision pacing observation state")
                .mode,
            FramePacingMode::ContinuousCapped { target_fps: 60 }
        );
    }

    #[test]
    fn windowed_host_preparation_observes_explicit_policy() {
        let mut app = App::new();
        app.with_frame_pacing(FramePacingPolicyResource::on_demand());

        app.prepare_windowed_frame_pacing();

        assert_eq!(
            app.world()
                .resource::<FramePacingRuntimeStateResource>()
                .expect("windowed Host preparation should provision pacing observation state")
                .mode,
            FramePacingMode::OnDemand
        );
    }

    #[test]
    fn explicit_native_host_policy_updates_existing_observation_mode_without_resetting_observations()
     {
        let mut app = App::new();
        app.insert_resource(FramePacingRuntimeStateResource {
            mode: FramePacingMode::ContinuousCapped { target_fps: 60 },
            last_frame_interval_ms: 12.5,
            next_frame_delay_ms: Some(7.25),
            redraw_requested: true,
        });

        app.with_frame_pacing(FramePacingPolicyResource::on_demand());

        let runtime_state = app
            .world()
            .resource::<FramePacingRuntimeStateResource>()
            .expect("explicit native Host policy should update existing observation state");
        assert_eq!(runtime_state.mode, FramePacingMode::OnDemand);
        assert_eq!(runtime_state.last_frame_interval_ms, 12.5);
        assert_eq!(runtime_state.next_frame_delay_ms, Some(7.25));
        assert!(runtime_state.redraw_requested);
    }

    #[test]
    fn windowed_host_preparation_preserves_existing_runtime_observations() {
        let mut app = App::new();
        app.insert_resource(FramePacingRuntimeStateResource {
            mode: FramePacingMode::OnDemand,
            last_frame_interval_ms: 12.5,
            next_frame_delay_ms: Some(7.25),
            redraw_requested: true,
        });

        app.prepare_windowed_frame_pacing();

        let runtime_state = app
            .world()
            .resource::<FramePacingRuntimeStateResource>()
            .expect("windowed Host preparation should retain pacing observation state");
        assert_eq!(
            runtime_state.mode,
            FramePacingMode::ContinuousCapped { target_fps: 60 }
        );
        assert_eq!(runtime_state.last_frame_interval_ms, 12.5);
        assert_eq!(runtime_state.next_frame_delay_ms, Some(7.25));
        assert!(runtime_state.redraw_requested);
    }
}
