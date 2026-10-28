use crate::app::WindowedAppState;
use crate::plugins::InputState;
use crate::plugins::render::host::native as render_native_host;
use crate::plugins::ui::UiRuntimeSlotsResource;
use crate::runtime::PrimaryPresentationMetricsResource;
use crate::runtime::frame_lifecycle::{run_frame as run_runtime_frame, run_startup_if_needed};
use crate::runtime::frame_pacing::{
    FramePacingPolicyResource, FramePacingRuntimeStateResource, FramePacingSchedule,
};
use crate::runtime::native_window_hooks::with_native_window_hooks;
use crate::runtime::platform::{
    PlatformEvent, PlatformWindowEvent, PlatformWindowEventQueueResource,
    apply_native_window_event, apply_platform_input_event,
};
use crate::runtime::presentation::ensure_primary_presentation_metrics;
use crate::runtime::window::{
    NativeWindowCreationRequest, NativeWindowId, WindowStateRegistryResource,
};
use crate::runtime::winit_input::{
    WinitInputAdapter, contact_input, cursor_position, keyboard_input, pointer_button_input,
    scroll_input, text_input,
};
use crate::runtime::winit_window_realizer::WinitWindowRealizer;
use crate::runtime::{NativeHostEvent, NativeWakeTransport};
use anyhow::{Context, Result, anyhow};
use runen_input::{ContinuityLoss, InputContext};
use std::collections::HashSet;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub(crate) fn run(mut state: WindowedAppState) -> Result<()> {
    install_native_window_provider_resources(&mut state.world);

    let mut event_loop_builder = EventLoop::<NativeHostEvent>::with_user_event();
    with_native_window_hooks(&mut state.world, |registry, _world| {
        registry.configure_event_loop(&mut event_loop_builder);
    });
    let event_loop = match event_loop_builder.build() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            let cleanup = shutdown_native_ui(&mut state.world);
            return combine_execution_and_cleanup(Err(error.into()), cleanup);
        }
    };
    let transport = NativeWakeTransport::new(event_loop.create_proxy());
    if state.world.has_resource::<UiRuntimeSlotsResource>() {
        state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .attach_native_transport(transport.clone());
    }
    state.world.insert_resource(transport);
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut runner = WinitRunner {
        state,
        window_realizer: WinitWindowRealizer::default(),
        input_adapter: WinitInputAdapter::default(),
        last_primary_redraw_at: None,
        frame_pacing_schedule: FramePacingSchedule::default(),
        fatal_error: None,
        frame_requested: false,
        frame_in_progress: false,
        owned_redraws: HashSet::new(),
        shutdown_done: false,
    };
    let execution = event_loop.run_app(&mut runner).map_err(anyhow::Error::from);
    let execution = match (runner.fatal_error.take(), execution) {
        (Some(error), Err(loop_error)) => {
            Err(error.context(format!("native event loop also failed: {loop_error:#}")))
        }
        (Some(error), Ok(())) => Err(error),
        (None, execution) => execution,
    };
    let cleanup = runner.shutdown_native();
    combine_execution_and_cleanup(execution, cleanup)
}

fn combine_execution_and_cleanup(execution: Result<()>, cleanup: Result<()>) -> Result<()> {
    match (execution, cleanup) {
        (Err(error), Err(cleanup)) => {
            Err(error.context(format!("native cleanup also failed: {cleanup:#}")))
        }
        (Err(error), _) | (_, Err(error)) => Err(error),
        _ => Ok(()),
    }
}

fn shutdown_native_ui(world: &mut runen_ecs::World) -> Result<()> {
    if world.has_resource::<UiRuntimeSlotsResource>() {
        world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .shutdown_native_slots()?;
    }
    Ok(())
}

fn install_native_window_provider_resources(world: &mut runen_ecs::World) {
    ensure_primary_presentation_metrics(world);
    if !world.has_resource::<WindowStateRegistryResource>() {
        world.insert_resource(WindowStateRegistryResource::default());
    }
    if !world.has_resource::<PlatformWindowEventQueueResource>() {
        world.insert_resource(PlatformWindowEventQueueResource::default());
    }
}

struct WinitRunner {
    state: WindowedAppState,
    window_realizer: WinitWindowRealizer,
    input_adapter: WinitInputAdapter,
    last_primary_redraw_at: Option<Instant>,
    frame_pacing_schedule: FramePacingSchedule,
    fatal_error: Option<anyhow::Error>,
    frame_requested: bool,
    /// Suppresses reentrant ordinary frames across the schedule and window effects.
    frame_in_progress: bool,
    /// Acknowledges host-requested redraws without advancing the frame twice.
    owned_redraws: HashSet<NativeWindowId>,
    shutdown_done: bool,
}

impl WinitRunner {
    fn sync_window_state(&mut self, window: &Window) -> Result<()> {
        let (size_px, scale_factor) = {
            let registry = self
                .state
                .world
                .resource_mut::<WindowStateRegistryResource>()
                .context("native Host window registry is unavailable")?;
            self.window_realizer
                .project_primary_window(registry, window)
        };
        self.sync_primary_presentation_and_surface_extent(size_px, scale_factor)
    }

    fn apply_event(&mut self, event: PlatformEvent) -> Result<()> {
        self.apply_event_for_native_window(NativeWindowId::primary(), event)
    }

    fn apply_window_focus_event(
        &mut self,
        native_window_id: NativeWindowId,
        focused: bool,
    ) -> Result<()> {
        if !focused
            && let Some(context) = self.input_adapter.window_source_context(native_window_id)
        {
            return self.apply_event_group(
                native_window_id,
                [
                    PlatformEvent::Focused { focused },
                    PlatformEvent::InputContinuityLost {
                        context,
                        loss: ContinuityLoss::Source,
                    },
                ],
            );
        }
        self.apply_event_for_native_window(native_window_id, PlatformEvent::Focused { focused })
    }

    fn retire_window_input_source(&mut self, native_window_id: NativeWindowId) -> Result<bool> {
        let Some(context) = self.input_adapter.window_source_context(native_window_id) else {
            return Ok(false);
        };
        self.apply_event_for_native_window(
            native_window_id,
            PlatformEvent::InputContinuityLost {
                context,
                loss: ContinuityLoss::Source,
            },
        )?;
        let retired = self
            .input_adapter
            .retire_window_source(native_window_id)
            .expect("known window input source must remain mapped until retirement");
        debug_assert_eq!(retired, context);
        Ok(true)
    }

    fn apply_raw_device_removal(&mut self, device_id: winit::event::DeviceId) -> Result<bool> {
        let Some(context) = self.input_adapter.retire_raw_device(device_id)? else {
            return Ok(false);
        };
        let input = self
            .state
            .world
            .resource_mut::<InputState>()
            .context("missing InputState resource")?;
        input.handle_continuity_loss(context, ContinuityLoss::Source);
        Ok(true)
    }

    fn apply_event_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        event: PlatformEvent,
    ) -> Result<()> {
        self.apply_event_group(native_window_id, [event])
    }

    fn apply_event_group<const N: usize>(
        &mut self,
        native_window_id: NativeWindowId,
        events: [PlatformEvent; N],
    ) -> Result<()> {
        // Journal capacity is checked for the entire observed callback before
        // any physical state or native record receives its members.
        self.state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()?
            .reserve_native_group(&events)?;
        let is_input = |event: &PlatformEvent| {
            matches!(
                event,
                PlatformEvent::KeyboardInput { .. }
                    | PlatformEvent::InputContinuityLost { .. }
                    | PlatformEvent::TextInput { .. }
                    | PlatformEvent::MouseWheel { .. }
                    | PlatformEvent::CursorMoved { .. }
                    | PlatformEvent::MouseInput { .. }
                    | PlatformEvent::Touch { .. }
            )
        };
        if events.iter().any(is_input) {
            self.state
                .world
                .resource::<InputState>()
                .context("missing InputState resource")?;
        }
        if events.iter().any(|event| !is_input(event)) {
            self.state
                .world
                .resource::<WindowStateRegistryResource>()?
                .record(native_window_id)
                .context("native Host event targets an unknown window")?;
        }
        if native_window_id == NativeWindowId::primary()
            && events.iter().any(|event| {
                matches!(
                    event,
                    PlatformEvent::Resized { .. } | PlatformEvent::ScaleFactorChanged { .. }
                )
            })
        {
            self.state
                .world
                .resource::<PrimaryPresentationMetricsResource>()?;
        }
        // Move owned payloads once. Admission does not require infallible String
        // clones after the explicit allocation/capacity check.
        let journal = events.map(|event| PlatformWindowEvent::new(native_window_id, event));
        for entry in &journal {
            self.apply_admitted_event(native_window_id, &entry.event)?;
        }
        self.state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .context("native Host platform-window event queue is unavailable")?
            .publish_group(journal)?;
        Ok(())
    }

    fn apply_admitted_event(
        &mut self,
        native_window_id: NativeWindowId,
        event: &PlatformEvent,
    ) -> Result<()> {
        match event {
            PlatformEvent::KeyboardInput { .. }
            | PlatformEvent::InputContinuityLost { .. }
            | PlatformEvent::TextInput { .. }
            | PlatformEvent::MouseWheel { .. }
            | PlatformEvent::CursorMoved { .. }
            | PlatformEvent::MouseInput { .. }
            | PlatformEvent::Touch { .. } => {
                let input = self
                    .state
                    .world
                    .resource_mut::<InputState>()
                    .context("missing InputState resource")?;
                apply_platform_input_event(input, event);
            }
            PlatformEvent::Resumed
            | PlatformEvent::CloseRequested
            | PlatformEvent::Focused { .. }
            | PlatformEvent::Resized { .. }
            | PlatformEvent::ScaleFactorChanged { .. }
            | PlatformEvent::RedrawRequested => {
                let projected_primary_metrics = {
                    let registry = self
                        .state
                        .world
                        .resource_mut::<WindowStateRegistryResource>()
                        .context("native Host window registry is unavailable")?;
                    let record = registry
                        .record_mut(native_window_id)
                        .context("native Host event targets an unknown window")?;
                    apply_native_window_event(record, event);
                    if native_window_id == NativeWindowId::primary()
                        && matches!(
                            event,
                            PlatformEvent::Resized { .. }
                                | PlatformEvent::ScaleFactorChanged { .. }
                        )
                    {
                        Some((record.size_px, record.scale_factor))
                    } else {
                        None
                    }
                };
                if let Some((size_px, scale_factor)) = projected_primary_metrics {
                    self.sync_primary_presentation_and_surface_extent(size_px, scale_factor)?;
                }
            }
        }
        Ok(())
    }

    fn apply_raw_mouse_motion(&mut self, context: InputContext, dx: f32, dy: f32) -> Result<()> {
        let input = self
            .state
            .world
            .resource_mut::<InputState>()
            .context("missing InputState resource")?;
        input.handle_relative_motion(context, dx, dy);
        Ok(())
    }

    fn sync_primary_presentation_and_surface_extent(
        &mut self,
        size_px: (u32, u32),
        scale_factor: f64,
    ) -> Result<()> {
        self.state
            .world
            .resource_mut::<PrimaryPresentationMetricsResource>()
            .context("missing primary presentation metrics")?
            .update(size_px, scale_factor);
        render_native_host::project_primary_extent(&mut self.state.world, size_px);
        Ok(())
    }

    fn run_startup_if_needed(&mut self) -> Result<()> {
        run_startup_if_needed(
            &mut self.state.world,
            &mut self.state.scheduler,
            &mut self.state.startup_ran,
        )
    }

    fn run_frame(&mut self) -> Result<()> {
        if let Some(window) = self.window_realizer.primary_window() {
            with_native_window_hooks(&mut self.state.world, |registry, world| {
                registry.dispatch_frame(&window, world);
            });
        }
        if self.state.world.has_resource::<UiRuntimeSlotsResource>() {
            self.state
                .world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .settle_native_slots()?;
        }
        run_runtime_frame(&mut self.state.world, &mut self.state.scheduler)
    }

    fn shutdown_native(&mut self) -> Result<()> {
        if self.shutdown_done {
            return Ok(());
        }
        if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>() {
            transport.close();
        }
        let cleanup = shutdown_native_ui(&mut self.state.world);
        self.shutdown_done = cleanup.is_ok();
        cleanup
    }

    fn request_native_redraw(&mut self, id: NativeWindowId, window: &Window) {
        if self.owned_redraws.insert(id) {
            WinitWindowRealizer::request_redraw(window);
        }
    }

    fn acknowledge_native_redraw(&mut self, id: NativeWindowId) {
        if !self.owned_redraws.remove(&id) {
            self.frame_requested = true;
        }
    }

    fn attach_native_window_hooks(&mut self, window: &Window) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.attach_hooks(window, world);
        });
    }

    fn drain_pending_window_requests(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let requests = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .ok()
            .map(|registry| registry.take_pending_creation_requests())
            .unwrap_or_default();
        for request in requests {
            self.create_window_for_request(event_loop, request)?;
        }
        Ok(())
    }

    fn create_window_for_request(
        &mut self,
        event_loop: &ActiveEventLoop,
        request: NativeWindowCreationRequest,
    ) -> Result<()> {
        let window = match self.window_realizer.realize_secondary(event_loop, &request) {
            Ok(window) => window,
            Err(err) => {
                self.mark_window_creation_failed(request.native_window_id, format!("{err:#}"))?;
                return Ok(());
            }
        };

        let realized_size = window.inner_size();
        let realized_size_px = (realized_size.width.max(1), realized_size.height.max(1));
        if let Err(err) = render_native_host::attach_secondary(
            &mut self.state.world,
            request.native_window_id,
            window.clone(),
            request.size_px,
            realized_size_px,
        ) {
            self.mark_window_creation_failed(request.native_window_id, format!("{err:#}"))?;
            return Ok(());
        }

        let logical_commit = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .context("native Host window registry is unavailable")
            .map(|registry| {
                self.window_realizer.project_created_window(
                    registry,
                    request.native_window_id,
                    &window,
                );
            });
        if let Err(err) = logical_commit {
            render_native_host::try_rollback_secondary(
                &mut self.state.world,
                request.native_window_id,
            )
            .context("secondary correlation rollback was refused")?;
            return Err(err);
        }

        self.attach_native_window_hooks(&window);
        self.request_native_redraw(request.native_window_id, &window);
        self.window_realizer
            .publish_window(request.native_window_id, window);
        Ok(())
    }

    fn mark_window_creation_failed(
        &mut self,
        native_window_id: NativeWindowId,
        reason: impl Into<String>,
    ) -> Result<()> {
        render_native_host::try_rollback_secondary(&mut self.state.world, native_window_id)
            .context("secondary creation rollback was refused")?;
        let registry = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .context("native Host window registry is unavailable")?;
        self.window_realizer
            .mark_creation_failed(registry, native_window_id, reason);
        Ok(())
    }

    /// Explicit Render handoff must succeed before Input or native-window retirement.
    fn authorize_secondary_close(&mut self, native_window_id: NativeWindowId) -> Result<()> {
        render_native_host::try_retire_secondary(&mut self.state.world, native_window_id)
            .context("secondary physical Render surface retirement was refused")?;
        self.retire_window_input_source(native_window_id)?;
        Ok(())
    }

    fn dispatch_native_window_event(&mut self, window: &Window, event: &WindowEvent) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.dispatch_window_event(window, event, world);
        });
    }

    fn dispatch_native_device_event(&mut self, event: &DeviceEvent) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.dispatch_device_event(event, world);
        });
    }

    fn apply_window_effects(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        self.drain_pending_window_requests(event_loop)?;

        let windows = self.window_realizer.realized_windows();
        for (window_id, native_window_id, window) in windows {
            let record = self
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .ok()
                .and_then(|registry| registry.record(native_window_id).cloned());
            let Some(record) = record else {
                continue;
            };

            WinitWindowRealizer::apply_physical_record(&window, &record);

            if record.close_requested {
                if native_window_id == NativeWindowId::primary() {
                    // Windows winit must not enter a blocking wait after exit.
                    event_loop.set_control_flow(ControlFlow::Poll);
                    event_loop.exit();
                    return Ok(());
                }
                self.authorize_secondary_close(native_window_id)?;
                if let Ok(window_registry) = self
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()
                {
                    window_registry.remove_window(native_window_id);
                }
                self.window_realizer.remove_window(window_id);
                self.owned_redraws.remove(&native_window_id);
                continue;
            }

            if record.redraw_requested {
                let request_now = native_window_id != NativeWindowId::primary()
                    || self.should_request_primary_redraw_now(Instant::now());
                if request_now {
                    self.request_native_redraw(native_window_id, &window);
                    if let Ok(registry) = self
                        .state
                        .world
                        .resource_mut::<WindowStateRegistryResource>()
                        && let Some(record) = registry.record_mut(native_window_id)
                    {
                        record.redraw_requested = false;
                    }
                }
            }
        }

        Ok(())
    }

    fn should_request_primary_redraw_now(&mut self, now: Instant) -> bool {
        let policy = self.frame_pacing_policy();
        let decision = self.frame_pacing_schedule.decide(policy, now);
        policy.target_frame_interval().is_none() || decision.request_redraw
    }

    fn frame_pacing_policy(&self) -> FramePacingPolicyResource {
        self.state
            .world
            .resource::<FramePacingPolicyResource>()
            .ok()
            .copied()
            .unwrap_or_default()
    }

    fn observe_frame_pacing_decision(
        &mut self,
        policy: FramePacingPolicyResource,
        now: Instant,
        next_deadline: Option<Instant>,
        redraw_requested: bool,
    ) {
        if let Ok(state) = self
            .state
            .world
            .resource_mut::<FramePacingRuntimeStateResource>()
        {
            state.observe_policy(policy);
            state.observe_next_deadline(now, next_deadline);
            state.observe_redraw_requested(redraw_requested);
        }
    }

    fn request_redraw_for_native_window(&mut self, native_window_id: NativeWindowId) {
        self.frame_requested = true;
        if let Ok(registry) = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            && let Some(record) = registry.record_mut(native_window_id)
        {
            record.request_redraw();
        }
        if let Ok(state) = self
            .state
            .world
            .resource_mut::<FramePacingRuntimeStateResource>()
        {
            state.observe_redraw_requested(true);
        }
    }

    fn observe_primary_frame_started(&mut self, now: Instant) {
        if let Some(previous) = self.last_primary_redraw_at
            && let Ok(state) = self
                .state
                .world
                .resource_mut::<FramePacingRuntimeStateResource>()
        {
            state.observe_frame_interval(now.saturating_duration_since(previous));
        }
        self.last_primary_redraw_at = Some(now);
    }

    /// Drives one ordinary Engine frame from either normal event-loop delivery or
    /// a Windows resize/move callback. The caller already owns a fully admitted
    /// callback and has applied pacing. Neither opportunity owns a second
    /// schedule or a separate journal-retirement rule.
    fn run_requested_frame<F>(
        &mut self,
        now: Instant,
        primary_window_ready: bool,
        after_frame: F,
    ) -> Result<bool>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        if self.frame_in_progress
            || !self.frame_requested
            || !primary_window_ready
            || !self.frame_is_ready(now)
        {
            return Ok(false);
        }

        // Consume only this request. An actual request arising during the frame
        // remains pending for a later, separately paced opportunity.
        self.frame_requested = false;
        self.frame_in_progress = true;
        self.observe_primary_frame_started(now);
        let result = self.run_frame().and_then(|_| after_frame(self));
        self.frame_in_progress = false;
        result?;
        Ok(true)
    }

    fn progress_requested_frame(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        match self.run_requested_frame(
            Instant::now(),
            self.window_realizer.has_primary_window(),
            |runner| runner.apply_window_effects(event_loop),
        ) {
            Ok(false) => {}
            Err(error) => {
                self.exit_with_error(event_loop, anyhow!("runtime frame failed: {error:#}"))
            }
            Ok(true) => {
                self.pace_or_exit(event_loop);
                if !event_loop.exiting()
                    && self.frame_requested
                    && self.frame_is_ready(Instant::now())
                {
                    // Preserve the original bounded continuation: a fresh
                    // request requires its own native event-loop opportunity.
                    if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>()
                        && let Err(error) = transport.notify()
                    {
                        self.exit_with_error(event_loop, error.into());
                    }
                }
            }
        }
    }

    fn frame_is_ready(&self, now: Instant) -> bool {
        self.last_primary_redraw_at
            .zip(self.frame_pacing_policy().target_frame_interval())
            .is_none_or(|(previous, interval)| now.saturating_duration_since(previous) >= interval)
    }

    fn ui_readiness(&self) -> Result<(bool, Option<Instant>)> {
        if self.state.world.has_resource::<UiRuntimeSlotsResource>() {
            self.state
                .world
                .resource::<UiRuntimeSlotsResource>()?
                .native_readiness()
        } else {
            Ok((false, None))
        }
    }

    fn apply_frame_pacing(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if event_loop.exiting() {
            return Ok(());
        }
        if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>() {
            transport.ensure_delivery()?;
        }
        let now = Instant::now();
        let policy = self.frame_pacing_policy();
        let decision = self.frame_pacing_schedule.decide(policy, now);
        let (unfinished, ui_deadline) = self.ui_readiness()?;
        if unfinished
            || ui_deadline.is_some_and(|deadline| deadline <= now)
            || decision.request_redraw
        {
            self.frame_requested = true;
        }
        if decision.request_redraw
            && let Some(window) = self.window_realizer.primary_window()
        {
            self.request_native_redraw(NativeWindowId::primary(), &window);
        }
        let capped_deadline = if self.frame_requested && !self.frame_is_ready(now) {
            self.last_primary_redraw_at
                .zip(policy.target_frame_interval())
                .and_then(|(last, interval)| last.checked_add(interval))
        } else {
            None
        };
        // Due work is represented by the pending frame bit. Never schedule a
        // past WaitUntil that would spin while waiting for a continuous cap.
        let next_deadline = [
            decision.next_deadline,
            ui_deadline.filter(|deadline| *deadline > now),
            capped_deadline,
        ]
        .into_iter()
        .flatten()
        .min();
        match next_deadline {
            Some(deadline) => event_loop.set_control_flow(ControlFlow::WaitUntil(deadline)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
        self.observe_frame_pacing_decision(policy, now, next_deadline, decision.request_redraw);
        Ok(())
    }

    fn pace_or_exit(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.apply_frame_pacing(event_loop) {
            self.exit_with_error(event_loop, error);
        }
    }

    fn exit_with_error(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
        tracing::error!(error = %format!("{err:#}"), "runtime windowed execution failed");
        self.fatal_error = Some(err);
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.exit();
    }
}

impl Drop for WinitRunner {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown_native() {
            tracing::error!(%error, "native UI destructor cleanup failed");
        }
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.detach_hooks(world);
        });
    }
}

impl ApplicationHandler<NativeHostEvent> for WinitRunner {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: NativeHostEvent) {
        if event_loop.exiting() {
            return;
        }
        if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>() {
            transport.acknowledge();
        }
        self.frame_requested = true;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_realizer.has_primary_window() {
            return;
        }

        let window = match self
            .window_realizer
            .realize_primary(event_loop, &self.state)
        {
            Ok(window) => window,
            Err(err) => {
                self.exit_with_error(event_loop, err);
                return;
            }
        };

        if let Err(err) = self.sync_window_state(&window) {
            self.exit_with_error(
                event_loop,
                anyhow!("failed to sync initial window state: {err:#}"),
            );
            return;
        }

        let size = window.inner_size();
        if let Err(err) = render_native_host::initialize_primary(
            &mut self.state.world,
            window.clone(),
            (size.width, size.height),
        ) {
            self.exit_with_error(event_loop, err);
            return;
        }

        if let Err(err) = self.apply_event(PlatformEvent::Resumed) {
            self.exit_with_error(event_loop, anyhow!("failed to apply resume event: {err:#}"));
            return;
        }

        self.attach_native_window_hooks(&window);

        if let Err(err) = self.run_startup_if_needed() {
            self.exit_with_error(event_loop, anyhow!("runtime startup failed: {err:#}"));
            return;
        }

        self.frame_requested = true;
        self.request_native_redraw(NativeWindowId::primary(), &window);
        self.window_realizer
            .publish_window(NativeWindowId::primary(), window);
        self.pace_or_exit(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if event_loop.exiting() {
            return;
        }
        let Some(window) = self.window_realizer.window(window_id) else {
            tracing::debug!(
                target = "engine.runtime.window",
                ?window_id,
                "ignoring event for unknown native window"
            );
            return;
        };
        let Some(native_window_id) = self.window_realizer.native_window_id(window_id) else {
            tracing::debug!(
                target = "engine.runtime.window",
                ?window_id,
                "ignoring event for unregistered native window"
            );
            return;
        };

        // Win32's interactive sizing loop can dispatch resize callbacks
        // without delivering about_to_wait. A fully processed resize provides
        // a canonical frame opportunity; move-only notifications do not
        // manufacture OnDemand visual work.
        #[cfg(target_os = "windows")]
        let resize_progress_opportunity = matches!(event, WindowEvent::Resized(_));
        self.dispatch_native_window_event(&window, &event);
        if !matches!(event, WindowEvent::RedrawRequested) {
            self.frame_requested = true;
        }

        let result: Result<()> = (|| match event {
            WindowEvent::CloseRequested => {
                self.apply_event_for_native_window(native_window_id, PlatformEvent::CloseRequested)
            }
            WindowEvent::Focused(focused) => {
                self.apply_window_focus_event(native_window_id, focused)
            }
            WindowEvent::Resized(size) => self.apply_event_for_native_window(
                native_window_id,
                PlatformEvent::Resized {
                    width: size.width,
                    height: size.height,
                },
            ),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let size = window.inner_size();
                self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::ScaleFactorChanged {
                        scale_factor,
                        width: size.width,
                        height: size.height,
                    },
                )
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let text = text_input(&event, is_synthetic);
                let input = keyboard_input(&event, is_synthetic);
                let result = match text {
                    Some(text) => self.apply_event_group(
                        native_window_id,
                        [
                            PlatformEvent::KeyboardInput { context, input },
                            PlatformEvent::TextInput { text },
                        ],
                    ),
                    None => self.apply_event_for_native_window(
                        native_window_id,
                        PlatformEvent::KeyboardInput { context, input },
                    ),
                };
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let input = scroll_input(delta, phase);
                let result = self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::MouseWheel { context, input },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let position = cursor_position(position);
                let result = self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::CursorMoved { context, position },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let input = pointer_button_input(state, button);
                let result = self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::MouseInput { context, input },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::Touch(touch) => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, touch.device_id)?;
                let input = contact_input(touch);
                let result = self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::Touch { context, input },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::RedrawRequested => {
                self.acknowledge_native_redraw(native_window_id);
                self.apply_event_for_native_window(native_window_id, PlatformEvent::RedrawRequested)
            }
            _ => Ok(()),
        })();

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime window event failed: {err:#}"));
            return;
        }
        self.pace_or_exit(event_loop);
        #[cfg(target_os = "windows")]
        if resize_progress_opportunity {
            self.progress_requested_frame(event_loop);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.dispatch_native_device_event(&event);

        let result: Result<()> = (|| match event {
            DeviceEvent::MouseMotion { delta } => {
                let context = self.input_adapter.raw_device_context(device_id)?;
                let result = self.apply_raw_mouse_motion(context, delta.0 as f32, delta.1 as f32);
                if result.is_ok() {
                    self.request_redraw_for_native_window(NativeWindowId::primary());
                }
                result
            }
            DeviceEvent::Removed => {
                let result = self.apply_raw_device_removal(device_id);
                if result.as_ref().is_ok_and(|removed| *removed) {
                    self.request_redraw_for_native_window(NativeWindowId::primary());
                }
                result.map(|_| ())
            }
            _ => Ok(()),
        })();

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime device event failed: {err:#}"));
            return;
        }
        self.pace_or_exit(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.pace_or_exit(event_loop);
        if !event_loop.exiting() {
            self.progress_requested_frame(event_loop);
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Err(error) = self.shutdown_native() {
            self.fatal_error = Some(match self.fatal_error.take() {
                Some(previous) => {
                    previous.context(format!("native cleanup also failed: {error:#}"))
                }
                None => error,
            });
        }
    }
}

#[cfg(test)]
#[path = "winit_runner_retirement_tests.rs"]
mod retirement_tests;

#[cfg(test)]
#[path = "winit_runner_resize_progress_tests.rs"]
mod resize_progress_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppNativeHostExt};
    use crate::plugins::{FixedStepPlugin, SimulationPlugin, TimePlugin};
    use crate::runtime::fixed_time::{
        CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick,
    };
    use crate::runtime::schedules::{FixedUpdate, PreUpdate};
    use crate::runtime::{CoreSet, Res, ResMut, SystemConfigExt};

    #[derive(Debug, Default, runen_ecs::Component, runen_ecs::Resource)]
    struct FixedTickLog(Vec<u64>);

    fn configure_probe(app: &mut App) {
        app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin));
        app.init_resource::<FixedTickLog>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, set_frame_delta.after(CoreSet::Time));
        app.add_systems(FixedUpdate, log_tick);
    }

    pub(super) fn runner_with_frame_pacing(policy: FramePacingPolicyResource) -> WinitRunner {
        let mut app = App::new();
        app.with_frame_pacing(policy);
        install_native_window_provider_resources(app.world_mut());
        app.world_mut()
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native Host test fixture should install window registry")
            .register_primary_window("test primary", (1280, 720), 1.0, true);
        WinitRunner {
            state: app.into_windowed_state(),
            window_realizer: WinitWindowRealizer::default(),
            input_adapter: WinitInputAdapter::default(),
            last_primary_redraw_at: None,
            frame_pacing_schedule: FramePacingSchedule::default(),
            fatal_error: None,
            frame_requested: false,
            frame_in_progress: false,
            owned_redraws: HashSet::new(),
            shutdown_done: false,
        }
    }

    fn set_frame_delta(mut time: ResMut<crate::plugins::time::domain::Time>) {
        time.delta_seconds = 0.05;
    }

    fn log_tick(tick: Res<SimulationTick>, mut log: ResMut<FixedTickLog>) {
        log.0.push(tick.0);
    }

    #[test]
    fn headless_and_windowed_paths_share_fixed_step_semantics() {
        let mut headless = App::headless();
        configure_probe(&mut headless);
        headless
            .prepare_for_run()
            .expect("headless startup should run");
        headless.run_frame().expect("headless frame should run");

        let headless_log = headless
            .world()
            .resource::<FixedTickLog>()
            .expect("headless log resource should exist")
            .0
            .clone();
        let headless_tick = headless
            .world()
            .resource::<SimulationTick>()
            .expect("headless tick should exist")
            .0;
        let headless_fixed = *headless
            .world()
            .resource::<FixedTimeState>()
            .expect("headless fixed state should exist");

        let mut windowed = App::new();
        configure_probe(&mut windowed);
        let mut runner = WinitRunner {
            state: windowed.into_windowed_state(),
            window_realizer: WinitWindowRealizer::default(),
            input_adapter: WinitInputAdapter::default(),
            last_primary_redraw_at: None,
            frame_pacing_schedule: FramePacingSchedule::default(),
            fatal_error: None,
            frame_requested: false,
            frame_in_progress: false,
            owned_redraws: HashSet::new(),
            shutdown_done: false,
        };
        runner
            .run_startup_if_needed()
            .expect("windowed startup should run");
        runner.run_frame().expect("windowed frame should run");

        let windowed_log = runner
            .state
            .world
            .resource::<FixedTickLog>()
            .expect("windowed log resource should exist")
            .0
            .clone();
        let windowed_tick = runner
            .state
            .world
            .resource::<SimulationTick>()
            .expect("windowed tick should exist")
            .0;
        let windowed_fixed = *runner
            .state
            .world
            .resource::<FixedTimeState>()
            .expect("windowed fixed state should exist");

        assert_eq!(headless_log, vec![1, 2, 3]);
        assert_eq!(windowed_log, vec![1, 2, 3]);
        assert_eq!(headless_log, windowed_log);
        assert_eq!(headless_tick, windowed_tick);
        assert_eq!(headless_fixed.steps_ran_last_frame, 3);
        assert_eq!(windowed_fixed.steps_ran_last_frame, 3);
        assert_eq!(headless_fixed.saturated_frames, 0);
        assert_eq!(windowed_fixed.saturated_frames, 0);
    }

    fn normalized_shift_press(context: InputContext) -> PlatformEvent {
        PlatformEvent::KeyboardInput {
            context,
            input: runen_input::KeyboardInput {
                physical_key: runen_input::PhysicalKeyIdentity::code("ShiftLeft"),
                logical_key: runen_input::LogicalKey::Native(
                    runen_input::NativeLogicalKey::Unidentified,
                ),
                location: runen_input::KeyLocation::Left,
                state: runen_input::DigitalState::Pressed,
                repeat: false,
                origin: runen_input::ObservationOrigin::SourceReport,
            },
        }
    }

    #[test]
    fn focus_loss_invalidates_existing_window_source_without_retiring_its_identity() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let backend_device = winit::event::DeviceId::dummy();
        let context = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), backend_device)
            .unwrap();
        runner
            .apply_event(normalized_shift_press(context))
            .expect("normalized key press should apply");
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );

        runner
            .apply_window_focus_event(NativeWindowId::primary(), false)
            .expect("focus loss should apply");

        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );
        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary())
                .map(|value| value.source),
            Some(context.source)
        );
    }

    #[test]
    fn focus_loss_does_not_allocate_an_unused_window_source() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());

        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary()),
            None
        );
        runner
            .apply_window_focus_event(NativeWindowId::primary(), false)
            .expect("focus loss should apply");
        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary()),
            None
        );
    }

    #[test]
    fn explicit_window_source_retirement_invalidates_state_and_refreshes_source_identity() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let secondary =
            NativeWindowId::try_from_raw(2).expect("secondary native window id should be valid");
        let backend_device = winit::event::DeviceId::dummy();
        let context = runner
            .input_adapter
            .window_context(secondary, backend_device)
            .unwrap();
        runner
            .apply_event_for_native_window(secondary, normalized_shift_press(context))
            .expect("secondary key press should apply");

        assert!(
            runner
                .retire_window_input_source(secondary)
                .expect("window source retirement should apply")
        );
        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );
        assert_eq!(runner.input_adapter.window_source_context(secondary), None);

        let replacement = runner
            .input_adapter
            .window_context(secondary, backend_device)
            .unwrap();
        assert_ne!(replacement.source, context.source);
        assert_eq!(replacement.device, context.device);
    }

    #[test]
    fn raw_device_removal_retires_only_raw_identity() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let backend_device = winit::event::DeviceId::dummy();
        let window = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), backend_device)
            .unwrap();
        let raw = runner
            .input_adapter
            .raw_device_context(backend_device)
            .unwrap();

        assert!(
            runner
                .apply_raw_device_removal(backend_device)
                .expect("raw device removal should apply")
        );
        let replacement = runner
            .input_adapter
            .raw_device_context(backend_device)
            .unwrap();
        assert_ne!(replacement.source, raw.source);
        assert_ne!(replacement.device, raw.device);
        assert_eq!(
            runner
                .input_adapter
                .window_context(NativeWindowId::primary(), backend_device)
                .unwrap(),
            window
        );
    }

    #[test]
    fn native_host_provider_setup_installs_empty_window_registry_and_event_queue() {
        let mut app = App::new();
        assert!(
            app.world()
                .resource::<WindowStateRegistryResource>()
                .is_err()
        );
        assert!(
            app.world()
                .resource::<PlatformWindowEventQueueResource>()
                .is_err()
        );

        install_native_window_provider_resources(app.world_mut());

        let windows = app
            .world()
            .resource::<WindowStateRegistryResource>()
            .expect("native Host should install the native window registry");
        assert_eq!(windows.records().count(), 0);
        assert!(
            app.world()
                .resource::<PlatformWindowEventQueueResource>()
                .is_ok()
        );
    }

    #[test]
    fn primary_window_events_project_primary_presentation_metrics() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());

        runner
            .apply_event(PlatformEvent::Resized {
                width: 1600,
                height: 900,
            })
            .expect("primary resize should apply");
        runner
            .apply_event(PlatformEvent::ScaleFactorChanged {
                scale_factor: 2.0,
                width: 1600,
                height: 900,
            })
            .expect("primary scale change should apply");

        let presentation = runner
            .state
            .world
            .resource::<PrimaryPresentationMetricsResource>()
            .expect("primary presentation metrics should exist");
        assert_eq!(presentation.size_px(), (1600, 900));
        assert_eq!(presentation.scale_factor(), 2.0);
    }

    #[test]
    fn secondary_window_geometry_does_not_replace_primary_presentation_metrics() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let secondary = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .request_window("Secondary", (640, 480))
            .native_window_id;

        runner
            .apply_event_for_native_window(
                secondary,
                PlatformEvent::ScaleFactorChanged {
                    scale_factor: 1.75,
                    width: 900,
                    height: 600,
                },
            )
            .expect("secondary scale event should apply");

        let presentation = runner
            .state
            .world
            .resource::<PrimaryPresentationMetricsResource>()
            .expect("primary presentation metrics should exist");
        assert_eq!(presentation, &PrimaryPresentationMetricsResource::default());
        let secondary_record = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .record(secondary)
            .expect("secondary record should exist");
        assert_eq!(secondary_record.size_px, (900, 600));
        assert_eq!(secondary_record.scale_factor, 1.75);
    }

    #[test]
    fn secondary_creation_failure_marks_only_the_requested_logical_window() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let request = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .request_window("Secondary", (900, 600));

        runner
            .mark_window_creation_failed(request.native_window_id, "test realization failure")
            .unwrap();

        let windows = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window registry should exist");
        assert_eq!(
            windows
                .record(request.native_window_id)
                .map(|record| record.lifecycle_state),
            Some(crate::runtime::NativeWindowLifecycleState::CreationFailed)
        );
        assert!(
            runner.window_realizer.realized_windows().is_empty(),
            "failed realization must not publish physical correlation"
        );
    }

    #[test]
    fn explicit_primary_redraw_request_wakes_on_demand_pacing() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());

        runner.request_redraw_for_native_window(NativeWindowId::primary());

        let registry = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window registry should exist");
        assert!(
            registry
                .record(NativeWindowId::primary())
                .expect("primary should exist")
                .redraw_requested
        );
        let pacing = runner
            .state
            .world
            .resource::<FramePacingRuntimeStateResource>()
            .expect("pacing state should exist");
        assert!(pacing.redraw_requested);
    }

    #[test]
    fn primary_input_redraws_wait_for_the_continuous_deadline() {
        let policy = FramePacingPolicyResource::continuous_capped(60);
        let interval = policy.target_frame_interval().expect("frame interval");
        let mut runner = runner_with_frame_pacing(policy);
        let start = Instant::now();

        assert!(runner.should_request_primary_redraw_now(start));
        assert!(!runner.should_request_primary_redraw_now(start + interval / 2));
        assert!(runner.should_request_primary_redraw_now(start + interval));
    }

    #[test]
    fn continuous_uncapped_keeps_host_turns_ready_without_a_timer() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::continuous_uncapped());
        let start = Instant::now();
        runner.last_primary_redraw_at = Some(start);
        assert!(runner.frame_is_ready(start));
        assert!(runner.should_request_primary_redraw_now(start));
        assert!(
            runner.should_request_primary_redraw_now(start + std::time::Duration::from_nanos(1))
        );
        let policy = runner.frame_pacing_policy();
        let next = runner.frame_pacing_schedule.decide(policy, start);
        assert!(next.request_redraw);
        assert!(
            next.next_deadline.is_none(),
            "uncapped Host never creates a timer deadline"
        );
    }

    #[test]
    fn live_uncapped_mode_switches_preserve_explicit_redraw_and_capped_readiness() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::continuous_uncapped());
        let now = Instant::now();
        runner.last_primary_redraw_at = Some(now);
        assert!(runner.frame_is_ready(now));
        let policy = runner.frame_pacing_policy();
        let uncapped = runner.frame_pacing_schedule.decide(policy, now);
        assert!(uncapped.request_redraw);
        assert_eq!(uncapped.next_deadline, None);

        // A secondary logical window's explicit redraw intent belongs to the
        // window registry, not to the primary continuous pacing policy.
        let secondary = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry")
            .request_window("secondary", (640, 480))
            .native_window_id;
        runner.request_redraw_for_native_window(secondary);
        assert!(runner.frame_requested);

        *runner
            .state
            .world
            .resource_mut::<FramePacingPolicyResource>()
            .expect("frame pacing policy") = FramePacingPolicyResource::on_demand();
        let policy = runner.frame_pacing_policy();
        let on_demand = runner.frame_pacing_schedule.decide(policy, now);
        assert!(!on_demand.request_redraw);
        assert_eq!(on_demand.next_deadline, None);
        assert!(runner.frame_is_ready(now));
        assert!(runner.should_request_primary_redraw_now(now));
        assert!(
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .expect("window registry")
                .record(secondary)
                .expect("secondary record")
                .redraw_requested,
            "switching to on-demand must preserve an explicit secondary redraw"
        );
        runner.frame_requested = false;
        assert!(
            !runner
                .frame_pacing_schedule
                .decide(policy, now)
                .request_redraw,
            "on-demand must not keep the former uncapped automatic requests alive"
        );

        let capped = FramePacingPolicyResource::continuous_capped(30);
        *runner
            .state
            .world
            .resource_mut::<FramePacingPolicyResource>()
            .expect("frame pacing policy") = capped;
        let interval = capped.target_frame_interval().expect("30 FPS interval");
        assert!(!runner.frame_is_ready(now));
        let resumed = runner.frame_pacing_schedule.decide(capped, now);
        assert!(resumed.request_redraw);
        assert_eq!(resumed.next_deadline, Some(now + interval));
        assert!(
            !runner
                .frame_pacing_schedule
                .decide(capped, now + interval / 2)
                .request_redraw
        );
        assert!(runner.frame_is_ready(now + interval));
        assert!(
            runner
                .frame_pacing_schedule
                .decide(capped, now + interval)
                .request_redraw
        );
    }

    #[test]
    fn primary_input_redraws_stay_immediate_on_demand() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let now = Instant::now();

        assert!(runner.should_request_primary_redraw_now(now));
        assert!(runner.should_request_primary_redraw_now(now + std::time::Duration::from_secs(1)));
    }

    #[test]
    fn any_window_readiness_respects_continuous_cap_and_owned_redraw_does_not_duplicate() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
        let start = Instant::now();
        runner.last_primary_redraw_at = Some(start);
        let secondary = NativeWindowId::try_from_raw(2).unwrap();
        runner.request_redraw_for_native_window(secondary);
        assert!(runner.frame_requested);
        assert!(!runner.frame_is_ready(start));
        assert!(runner.frame_is_ready(start + std::time::Duration::from_millis(17)));
        runner.frame_requested = false;
        runner.owned_redraws.insert(secondary);
        runner.acknowledge_native_redraw(secondary);
        assert!(!runner.frame_requested);
        runner.acknowledge_native_redraw(secondary);
        assert!(
            runner.frame_requested,
            "an independent native redraw requests an Engine turn"
        );
    }

    #[test]
    fn native_group_overflow_changes_neither_physical_state_nor_journal() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let context = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), winit::event::DeviceId::dummy())
            .unwrap();
        for _ in 0..4095 {
            runner
                .state
                .world
                .resource_mut::<PlatformWindowEventQueueResource>()
                .unwrap()
                .publish(PlatformWindowEvent::new(
                    NativeWindowId::primary(),
                    PlatformEvent::RedrawRequested,
                ))
                .unwrap();
        }
        assert!(
            runner
                .apply_event_group(
                    NativeWindowId::primary(),
                    [
                        normalized_shift_press(context),
                        PlatformEvent::TextInput { text: "x".into() },
                    ]
                )
                .is_err()
        );
        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .shift_down()
        );
        assert_eq!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .unwrap()
                .events()
                .len(),
            4095
        );
    }

    #[test]
    fn missing_physical_owner_rejects_focus_group_before_window_or_journal_mutation() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.remove_resource::<InputState>();
        let primary = NativeWindowId::primary();
        runner
            .input_adapter
            .window_context(primary, winit::event::DeviceId::dummy())
            .unwrap();
        let before = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap()
            .record(primary)
            .unwrap()
            .focused;
        assert!(runner.apply_window_focus_event(primary, false).is_err());
        assert_eq!(
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .unwrap()
                .record(primary)
                .unwrap()
                .focused,
            before
        );
        assert!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .unwrap()
                .events()
                .is_empty()
        );
    }
}
