use crate::app::App;
use crate::app::domain::host::AppHostSelection;
use crate::app::domain::state::NativeInitialWindow;
use crate::runtime::app_shutdown::{AppShutdownError, AppShutdownHandle, AppShutdownOwner};
use crate::runtime::frame_pacing::{FramePacingPolicyResource, FramePacingRuntimeStateResource};
use crate::runtime::native_window_requests::{
    NativeWindowRequestError, NativeWindowRequestOwner, NativeWindowRequests,
};
use crate::runtime::window::PrimaryWindowInitialSizePxResource;
use crate::runtime::winit_runner;
use anyhow::Result;

use crate::runtime::schedules::FrameEnd;
use crate::runtime::window::WindowStateRegistryResource;
use anyhow::{anyhow, ensure};
use runen_ecs::ResMut;

/// Policy selected for primary native-window close intents.
///
/// The absence of this resource preserves the existing explicit approval
/// contract. Automatic approval is only for ordinary applications without an
/// asynchronous confirmation or unsaved-work veto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, runen_ecs::Resource)]
pub enum NativePrimaryClosePolicy {
    /// The application must explicitly approve or veto each close intent.
    Explicit,
    /// Approve a primary close intent still pending after product Update.
    ApproveUnhandled,
}

fn approve_unhandled_primary_close_intent(mut windows: ResMut<WindowStateRegistryResource>) {
    if let Some(primary) = windows.primary_window_id()
        && let Some(record) = windows.record_mut(primary)
        && record.close_intent_pending
    {
        record.approve_close();
    }
}

pub trait AppNativeHostExt {
    fn with_frame_pacing(&mut self, policy: FramePacingPolicyResource) -> &mut Self;
    fn with_primary_window_size_px(&mut self, size_px: (u32, u32)) -> &mut Self;
    /// Select an explicit primary close policy while App is configuring.
    ///
    /// Only ApproveUnhandled installs a FrameEnd close handler. An application
    /// requiring unsaved-work confirmation must use Explicit instead.
    fn with_primary_close_policy(&mut self, policy: NativePrimaryClosePolicy) -> Result<&mut Self>;
}

impl AppNativeHostExt for App {
    fn with_primary_close_policy(&mut self, policy: NativePrimaryClosePolicy) -> Result<&mut Self> {
        ensure!(
            self.allow_topology_mutation("with_primary_close_policy", None),
            "primary close policy must be selected during App configuration"
        );
        if matches!(self.host_selection, AppHostSelection::Headless) {
            self.record_missing_capability("with_primary_close_policy", "native-window Host");
            return Err(anyhow!(
                "primary close policy requires App::new() native-window Host"
            ));
        }
        ensure!(
            self.initial_native_window == NativeInitialWindow::Primary,
            "primary close policy requires an initially declared native primary window"
        );
        ensure!(
            !self.world.has_resource::<NativePrimaryClosePolicy>(),
            "primary close policy has already been selected"
        );
        if policy == NativePrimaryClosePolicy::ApproveUnhandled {
            self.add_systems(FrameEnd, approve_unhandled_primary_close_intent);
        }
        self.insert_resource(policy);
        Ok(self)
    }

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
        if self.initial_native_window == NativeInitialWindow::None {
            self.record_missing_capability(
                "with_primary_window_size_px",
                "initial native primary window",
            );
            return self;
        }
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
    /// Select the existing Native event loop with no mandatory initial window.
    ///
    /// App::new() retains its native-primary behavior until the separate
    /// breaking constructor migration. No logical primary, UI, GPU or
    /// Render surface is implicitly selected.
    pub fn enable_windowing(&mut self) -> Result<&mut Self> {
        ensure!(
            self.is_native_window_host(),
            "enable_windowing requires a Native-window Host; App::headless() is stable"
        );
        ensure!(
            self.lifecycle.is_configuring(),
            "enable_windowing must be selected before App admission"
        );
        ensure!(
            !self
                .world
                .has_resource::<PrimaryWindowInitialSizePxResource>()
                && !self.world.has_resource::<NativePrimaryClosePolicy>(),
            "zero initial windows conflict with selected initial-primary configuration"
        );
        self.initial_native_window = NativeInitialWindow::None;
        Ok(self)
    }

    /// Obtain an App-owned shutdown handle before native run consumes App.
    /// A Headless App still has only bounded execution; the future shared
    /// long-running Headless driver is a separate delivery.
    pub fn shutdown_handle(
        &mut self,
    ) -> std::result::Result<AppShutdownHandle, AppShutdownError> {
        if !self.is_native_window_host() {
            return Err(AppShutdownError::UnsupportedHost);
        }
        if !self.lifecycle.is_configuring() {
            return Err(AppShutdownError::SealedComposition);
        }
        if !self.world.has_resource::<NativeWindowRequestOwner>() {
            self.world
                .insert_resource(NativeWindowRequestOwner::default());
        }
        if !self.world.has_resource::<AppShutdownOwner>() {
            let windows = self
                .world
                .resource::<NativeWindowRequestOwner>()
                .map_err(|_| AppShutdownError::MissingOwner)?
                .handle();
            self.world.insert_resource(AppShutdownOwner::new(windows));
        }
        self.world
            .resource::<AppShutdownOwner>()
            .map(|owner| owner.handle())
            .map_err(|_| AppShutdownError::MissingOwner)
    }

    /// Acquire Native Host window commands before run consumes the App.
    /// The handle stays NotReady until the original Winit Host can receive
    /// requests. Queueing is not a physical creation acknowledgement.
    pub fn native_window_requests(
        &mut self,
    ) -> std::result::Result<NativeWindowRequests, NativeWindowRequestError> {
        if !self.is_native_window_host() {
            return Err(NativeWindowRequestError::UnsupportedHost);
        }
        if !self.lifecycle.is_configuring() {
            return Err(NativeWindowRequestError::SealedComposition);
        }
        if !self.world.has_resource::<NativeWindowRequestOwner>() {
            self.world
                .insert_resource(NativeWindowRequestOwner::default());
        }
        self.world
            .resource::<NativeWindowRequestOwner>()
            .map(|owner| owner.handle())
            .map_err(|_| NativeWindowRequestError::MissingOwner)
    }

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
        if self.initial_native_window == NativeInitialWindow::None {
            ensure!(
                !crate::plugins::render::render_integration_is_active(&self.world),
                "zero-initial-window Native startup with RenderPlugin is not yet supported: no primary Gfx owner exists for secondary attachment"
            );
            if let Ok(slots) = self
                .world
                .resource::<crate::plugins::ui::UiRuntimeSlotsResource>()
            {
                ensure!(
                    slots.is_empty(),
                    "zero-initial-window Native startup does not yet support pre-mounted UI without an explicit presentation target"
                );
            }
        }
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

    #[test]
    fn primary_close_default_keeps_explicit_intent_pending() {
        use crate::runtime::window::{NativeWindowId, WindowStateRegistryResource};

        let mut app = App::new();
        let mut windows = WindowStateRegistryResource::default();
        let id = windows.register_primary_window("Manual", (800, 600), 1.0, true);
        windows.record_mut(id).unwrap().receive_close_intent();
        app.insert_resource(windows);

        let mut state = app.into_windowed_state();
        crate::runtime::frame_lifecycle::run_frame(&mut state.world, &mut state.scheduler).unwrap();
        let windows = state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap();
        let primary = windows.record(NativeWindowId::primary()).unwrap();
        assert!(primary.close_intent_pending);
        assert!(!primary.close_requested);
        assert!(!state.world.has_resource::<NativePrimaryClosePolicy>());
    }

    #[test]
    fn opted_in_primary_close_is_approved_at_frame_end_without_secondary_change() {
        use crate::runtime::window::WindowStateRegistryResource;

        let mut app = App::new();
        app.with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
            .unwrap();
        let mut windows = WindowStateRegistryResource::default();
        let primary = windows.register_primary_window("Primary", (800, 600), 1.0, true);
        let secondary = windows.request_window("Secondary", (640, 480));
        windows.register_created_window(
            secondary.native_window_id,
            "Secondary",
            (640, 480),
            1.0,
            true,
        );
        windows.record_mut(primary).unwrap().receive_close_intent();
        windows
            .record_mut(secondary.native_window_id)
            .unwrap()
            .receive_close_intent();
        app.insert_resource(windows);

        let mut state = app.into_windowed_state();
        crate::runtime::frame_lifecycle::run_frame(&mut state.world, &mut state.scheduler).unwrap();
        let windows = state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap();
        assert!(windows.record(primary).unwrap().close_requested);
        assert!(!windows.record(primary).unwrap().close_intent_pending);
        assert!(
            windows
                .record(secondary.native_window_id)
                .unwrap()
                .close_intent_pending
        );
        assert!(
            !windows
                .record(secondary.native_window_id)
                .unwrap()
                .close_requested
        );
        assert_eq!(
            *state.world.resource::<NativePrimaryClosePolicy>().unwrap(),
            NativePrimaryClosePolicy::ApproveUnhandled
        );
    }

    #[test]
    fn product_update_veto_wins_over_selected_frame_end_close() {
        use crate::runtime::schedules::Update;
        use crate::runtime::window::WindowStateRegistryResource;

        fn veto_primary_close(mut windows: ResMut<WindowStateRegistryResource>) {
            if let Some(primary) = windows.primary_window_id()
                && let Some(record) = windows.record_mut(primary)
                && record.close_intent_pending
            {
                record.veto_close();
            }
        }

        let mut app = App::new();
        app.with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
            .unwrap();
        app.add_systems(Update, veto_primary_close);
        let mut windows = WindowStateRegistryResource::default();
        let primary = windows.register_primary_window("Veto", (800, 600), 1.0, true);
        windows.record_mut(primary).unwrap().receive_close_intent();
        app.insert_resource(windows);

        let mut state = app.into_windowed_state();
        crate::runtime::frame_lifecycle::run_frame(&mut state.world, &mut state.scheduler).unwrap();
        let windows = state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap();
        let record = windows.record(primary).unwrap();
        assert!(!record.close_requested);
        assert!(!record.close_intent_pending);
    }

    #[test]
    fn explicit_selected_close_remains_manual_and_no_intent_is_inert() {
        use crate::runtime::window::WindowStateRegistryResource;

        for policy in [
            NativePrimaryClosePolicy::Explicit,
            NativePrimaryClosePolicy::ApproveUnhandled,
        ] {
            let mut app = App::new();
            app.with_primary_close_policy(policy).unwrap();
            let mut windows = WindowStateRegistryResource::default();
            let id = windows.register_primary_window("No intent", (800, 600), 1.0, true);
            app.insert_resource(windows);
            let mut state = app.into_windowed_state();
            crate::runtime::frame_lifecycle::run_frame(&mut state.world, &mut state.scheduler)
                .unwrap();
            assert!(
                !state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap()
                    .record(id)
                    .unwrap()
                    .close_requested
            );
            if policy == NativePrimaryClosePolicy::Explicit {
                let record = state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap()
                    .record(id)
                    .unwrap();
                assert!(!record.close_intent_pending);
            }
        }
    }

    #[test]
    fn native_shutdown_handle_is_pre_run_and_closes_when_app_is_dropped() {
        use crate::runtime::app_shutdown::{AppShutdownError, AppShutdownReadiness};

        let mut app = App::new();
        let handle = app.shutdown_handle().unwrap();
        assert_eq!(handle.readiness().unwrap(), AppShutdownReadiness::NotReady);
        assert!(matches!(
            handle.request_shutdown(),
            Err(AppShutdownError::NotReady)
        ));
        drop(app);
        assert_eq!(handle.readiness().unwrap(), AppShutdownReadiness::Closed);
        assert!(matches!(
            handle.request_shutdown(),
            Err(AppShutdownError::Closed)
        ));

        let mut headless = App::headless();
        assert!(matches!(
            headless.shutdown_handle(),
            Err(AppShutdownError::UnsupportedHost)
        ));
        let mut prepared = App::new();
        prepared.prepare_lifecycle_for_execution().unwrap();
        assert!(matches!(
            prepared.shutdown_handle(),
            Err(AppShutdownError::SealedComposition)
        ));
    }

    #[test]
    fn zero_initial_window_selection_preserves_default_and_requires_native() {
        let default = App::new();
        assert_eq!(default.initial_native_window, NativeInitialWindow::Primary);

        let mut zero_window = App::new();
        zero_window.enable_windowing().unwrap();
        assert_eq!(zero_window.initial_native_window, NativeInitialWindow::None);
        assert_eq!(
            zero_window.into_windowed_state().initial_native_window,
            NativeInitialWindow::None
        );

        let mut headless = App::headless();
        assert!(headless.enable_windowing().is_err());

        let mut late = App::new();
        late.prepare_lifecycle_for_execution().unwrap();
        assert!(late.enable_windowing().is_err());
        assert_eq!(late.initial_native_window, NativeInitialWindow::Primary);
    }

    #[test]
    fn zero_initial_window_rejects_primary_only_configuration_in_either_order() {
        let mut primary_size = App::new();
        primary_size.with_primary_window_size_px((1600, 900));
        assert!(primary_size.enable_windowing().is_err());
        assert_eq!(
            primary_size.initial_native_window,
            NativeInitialWindow::Primary
        );

        let mut windowless = App::new();
        windowless.enable_windowing().unwrap();
        windowless.with_primary_window_size_px((1600, 900));
        assert!(
            windowless.admit_composition().is_err(),
            "an invalid initial-primary size must fail at App admission"
        );

        let mut close = App::new();
        close.enable_windowing().unwrap();
        assert!(
            close
                .with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
                .is_err()
        );
    }

    #[test]
    fn windowless_render_selection_rejects_before_winit_or_startup() {
        let mut app = App::new();
        app.enable_windowing().unwrap();
        app.add_plugins(crate::plugins::default_plugins());
        app.add_plugin(crate::plugins::RenderPlugin);
        let error = app
            .run()
            .expect_err("unsupported secondary Render attachment needs an explicit failure");
        assert!(
            format!("{error:#}").contains("no primary Gfx owner"),
            "unsupported selection silently started the host: {error:#}"
        );
    }

    #[test]
    fn primary_close_policy_rejects_headless_duplicate_and_late_selection() {
        let mut headless = App::headless();
        assert!(
            headless
                .with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
                .is_err()
        );
        assert!(!headless.world().has_resource::<NativePrimaryClosePolicy>());
        assert!(headless.admit_composition().is_err());

        let mut native = App::new();
        native
            .with_primary_close_policy(NativePrimaryClosePolicy::Explicit)
            .unwrap();
        assert!(
            native
                .with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
                .is_err()
        );
        assert_eq!(
            *native
                .world()
                .resource::<NativePrimaryClosePolicy>()
                .unwrap(),
            NativePrimaryClosePolicy::Explicit
        );

        let mut late = App::new();
        late.prepare_lifecycle_for_execution().unwrap();
        assert!(
            late.with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
                .is_err()
        );
        assert!(!late.world().has_resource::<NativePrimaryClosePolicy>());
        assert!(late.admit_composition().is_err());
    }
}
