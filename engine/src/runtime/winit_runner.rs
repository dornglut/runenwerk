use crate::app::WindowedAppState;
use crate::plugins::InputState;
use crate::plugins::render::backend::{RenderSurfaceId, RenderSurfaceRegistryResource};
use crate::plugins::render::render_integration_is_active;
use crate::plugins::render::renderer::Gfx;
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
    NativeWindowCreationRequest, NativeWindowId, WindowCursorIcon, WindowStateRegistryResource,
};
use crate::runtime::winit_input::{
    WinitInputAdapter, contact_input, cursor_position, keyboard_input, pointer_button_input,
    scroll_input, text_input,
};
use anyhow::{Context, Result, anyhow};
use runen_input::InputContext;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

pub(crate) fn run(mut state: WindowedAppState) -> Result<()> {
    install_native_window_provider_resources(&mut state.world);

    let mut event_loop_builder = EventLoop::builder();
    with_native_window_hooks(&mut state.world, |registry, _world| {
        registry.configure_event_loop(&mut event_loop_builder);
    });
    let event_loop = event_loop_builder.build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut runner = WinitRunner {
        state,
        window: None,
        windows: BTreeMap::new(),
        native_windows_by_winit: BTreeMap::new(),
        input_adapter: WinitInputAdapter::default(),
        last_primary_redraw_at: None,
        frame_pacing_schedule: FramePacingSchedule::default(),
        fatal_error: None,
    };
    event_loop
        .run_app(&mut runner)
        .map_err(anyhow::Error::from)?;
    if let Some(err) = runner.fatal_error.take() {
        Err(err)
    } else {
        Ok(())
    }
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
    window: Option<Arc<Window>>,
    windows: BTreeMap<WindowId, Arc<Window>>,
    native_windows_by_winit: BTreeMap<WindowId, NativeWindowId>,
    input_adapter: WinitInputAdapter,
    last_primary_redraw_at: Option<Instant>,
    frame_pacing_schedule: FramePacingSchedule,
    fatal_error: Option<anyhow::Error>,
}

impl WinitRunner {
    fn sync_window_state(&mut self, window: &Window) -> Result<()> {
        let size = window.inner_size();
        let size_px = (size.width.max(1), size.height.max(1));
        let scale_factor = window.scale_factor();
        self.state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .context("native Host window registry is unavailable")?
            .register_primary_window(
                window.title().to_string(),
                size_px,
                scale_factor,
                window.has_focus(),
            );
        self.sync_primary_presentation_and_surface_extent(size_px, scale_factor)
    }

    fn apply_event(&mut self, event: PlatformEvent) -> Result<()> {
        self.apply_event_for_native_window(NativeWindowId::primary(), event)
    }

    fn apply_event_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        event: PlatformEvent,
    ) -> Result<()> {
        self.state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .context("native Host platform-window event queue is unavailable")?
            .publish(PlatformWindowEvent::new(native_window_id, event.clone()));

        match &event {
            PlatformEvent::KeyboardInput { .. }
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
                apply_platform_input_event(input, &event);
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
                    apply_native_window_event(record, &event);
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
        if let Ok(surface_registry) = self
            .state
            .world
            .resource_mut::<RenderSurfaceRegistryResource>()
        {
            surface_registry
                .update_surface_extent_for_native_window(NativeWindowId::primary(), size_px);
        }
        Ok(())
    }

    fn confirm_primary_render_surface_attachment(
        &mut self,
        target_size_px: (u32, u32),
    ) -> Result<()> {
        let surface = RenderSurfaceId::primary();
        if !self
            .state
            .world
            .resource::<Gfx>()
            .context("runtime gfx is unavailable")?
            .has_surface(surface)
        {
            return Err(anyhow!(
                "runtime gfx does not own the primary render surface after initialization"
            ));
        }
        self.state
            .world
            .resource_mut::<RenderSurfaceRegistryResource>()
            .context("render surface registry is unavailable")?
            .confirm_surface_attachment(surface, NativeWindowId::primary(), target_size_px)
    }

    fn run_startup_if_needed(&mut self) -> Result<()> {
        run_startup_if_needed(
            &mut self.state.world,
            &mut self.state.scheduler,
            &mut self.state.startup_ran,
        )
    }

    fn run_frame(&mut self) -> Result<()> {
        if let Some(window) = self.window.clone() {
            with_native_window_hooks(&mut self.state.world, |registry, world| {
                registry.dispatch_frame(&window, world);
            });
        }
        run_runtime_frame(&mut self.state.world, &mut self.state.scheduler)
    }

    fn attach_native_window_hooks(&mut self, window: &Window) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.attach_hooks(window, world);
        });
    }

    fn register_runtime_window(&mut self, native_window_id: NativeWindowId, window: Arc<Window>) {
        self.native_windows_by_winit
            .insert(window.id(), native_window_id);
        self.windows.insert(window.id(), window);
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
        let attrs: WindowAttributes = Window::default_attributes()
            .with_title(request.title.clone())
            .with_inner_size(winit::dpi::PhysicalSize::new(
                request.size_px.0,
                request.size_px.1,
            ));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                self.mark_window_creation_failed(
                    request.native_window_id,
                    format!("native window creation failed: {err}"),
                );
                return Ok(());
            }
        };

        let realized_size = window.inner_size();
        let realized_size_px = (realized_size.width.max(1), realized_size.height.max(1));
        let realized_scale_factor = window.scale_factor();
        let realized_title = window.title().to_string();
        if render_integration_is_active(&self.state.world) {
            let render_surface_id = self
                .state
                .world
                .resource_mut::<RenderSurfaceRegistryResource>()
                .ok()
                .map(|registry| {
                    registry
                        .surface_for_native_window(request.native_window_id)
                        .unwrap_or_else(|| {
                            registry.reserve_surface_for_native_window(
                                request.native_window_id,
                                request.size_px,
                            )
                        })
                });
            let Some(render_surface_id) = render_surface_id else {
                self.mark_window_creation_failed(
                    request.native_window_id,
                    "render surface registry is unavailable",
                );
                return Ok(());
            };
            let attach_result = self
                .state
                .world
                .resource_mut::<Gfx>()
                .context("runtime gfx is unavailable")
                .and_then(|gfx| {
                    gfx.attach_surface(render_surface_id, Arc::clone(&window), realized_size_px)
                });
            if let Err(err) = attach_result {
                self.mark_window_creation_failed(
                    request.native_window_id,
                    format!("GPU surface attachment failed: {err:#}"),
                );
                return Ok(());
            }
            let confirm_result = self
                .state
                .world
                .resource_mut::<RenderSurfaceRegistryResource>()
                .context("render surface registry is unavailable")
                .and_then(|registry| {
                    registry.confirm_surface_attachment(
                        render_surface_id,
                        request.native_window_id,
                        realized_size_px,
                    )
                });
            if let Err(err) = confirm_result {
                if let Ok(gfx) = self.state.world.resource_mut::<Gfx>() {
                    gfx.detach_surface(render_surface_id);
                }
                self.mark_window_creation_failed(
                    request.native_window_id,
                    format!("render surface correlation failed: {err:#}"),
                );
                return Ok(());
            }
        }
        if let Ok(registry) = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
        {
            registry.register_created_window(
                request.native_window_id,
                realized_title,
                realized_size_px,
                realized_scale_factor,
                window.has_focus(),
            );
        }

        self.attach_native_window_hooks(&window);
        window.request_redraw();
        self.register_runtime_window(request.native_window_id, window);
        Ok(())
    }

    fn mark_window_creation_failed(
        &mut self,
        native_window_id: NativeWindowId,
        reason: impl Into<String>,
    ) {
        if let Ok(surface_registry) = self
            .state
            .world
            .resource_mut::<RenderSurfaceRegistryResource>()
        {
            surface_registry.retire_surface_for_native_window(native_window_id);
        }
        if let Ok(registry) = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            && let Some(record) = registry.record_mut(native_window_id)
        {
            record.mark_creation_failed(reason);
        }
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

        let windows = self.windows.values().cloned().collect::<Vec<_>>();
        for window in windows {
            let Some(native_window_id) =
                native_window_id_for_winit_event(&self.native_windows_by_winit, window.id())
            else {
                tracing::debug!(
                    target = "engine.runtime.window",
                    ?window,
                    "skipping effects for unregistered native window"
                );
                continue;
            };
            let record = self
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .ok()
                .and_then(|registry| registry.record(native_window_id).cloned());
            let Some(record) = record else {
                continue;
            };

            if window.title() != record.title {
                window.set_title(&record.title);
            }
            window.set_cursor(winit_cursor_icon(record.cursor_icon));

            if record.close_requested {
                if native_window_id == NativeWindowId::primary() {
                    event_loop.exit();
                    return Ok(());
                }
                let render_surface_id = self
                    .state
                    .world
                    .resource::<RenderSurfaceRegistryResource>()
                    .ok()
                    .and_then(|registry| registry.surface_for_native_window(native_window_id));
                if let Some(render_surface_id) = render_surface_id
                    && let Ok(gfx) = self.state.world.resource_mut::<Gfx>()
                {
                    gfx.detach_surface(render_surface_id);
                }
                if let Ok(surface_registry) = self
                    .state
                    .world
                    .resource_mut::<RenderSurfaceRegistryResource>()
                {
                    surface_registry.retire_surface_for_native_window(native_window_id);
                }
                if let Ok(window_registry) = self
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()
                {
                    window_registry.remove_window(native_window_id);
                }
                self.windows.remove(&window.id());
                self.native_windows_by_winit.remove(&window.id());
                continue;
            }

            if record.redraw_requested {
                let request_now = native_window_id != NativeWindowId::primary()
                    || self.should_request_primary_redraw_now(Instant::now());
                if request_now {
                    window.request_redraw();
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

    fn observe_primary_frame_started(&mut self) {
        let now = Instant::now();
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

    fn apply_frame_pacing(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let policy = self.frame_pacing_policy();
        let decision = self.frame_pacing_schedule.decide(policy, now);
        if decision.request_redraw
            && let Some(window) = self.window.as_ref()
        {
            window.request_redraw();
        }
        match decision.next_deadline {
            Some(deadline) => event_loop.set_control_flow(ControlFlow::wait_duration(
                deadline.saturating_duration_since(Instant::now()),
            )),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
        self.observe_frame_pacing_decision(
            policy,
            now,
            decision.next_deadline,
            decision.request_redraw,
        );
    }

    fn exit_with_error(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
        tracing::error!(error = %format!("{err:#}"), "runtime windowed execution failed");
        self.fatal_error = Some(err);
        event_loop.exit();
    }
}

impl Drop for WinitRunner {
    fn drop(&mut self) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.detach_hooks(world);
        });
    }
}

fn winit_cursor_icon(cursor_icon: WindowCursorIcon) -> CursorIcon {
    match cursor_icon {
        WindowCursorIcon::Default => CursorIcon::Default,
        WindowCursorIcon::ColResize => CursorIcon::ColResize,
        WindowCursorIcon::RowResize => CursorIcon::RowResize,
        WindowCursorIcon::NwseResize => CursorIcon::NwseResize,
        WindowCursorIcon::NeswResize => CursorIcon::NeswResize,
        WindowCursorIcon::Grab => CursorIcon::Grab,
        WindowCursorIcon::Grabbing => CursorIcon::Grabbing,
    }
}

impl ApplicationHandler for WinitRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs: WindowAttributes =
            Window::default_attributes().with_title(self.state.title.clone());
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                self.exit_with_error(
                    event_loop,
                    anyhow!("failed to create runtime window: {err}"),
                );
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

        if render_integration_is_active(&self.state.world) {
            if self.state.world.resource::<Gfx>().is_err() {
                let gfx = match Gfx::new(window.clone()) {
                    Ok(gfx) => gfx,
                    Err(err) => {
                        self.exit_with_error(
                            event_loop,
                            anyhow!("failed to initialize runtime gfx: {err:#}"),
                        );
                        return;
                    }
                };
                self.state.world.insert_resource(gfx);
                let size = window.inner_size();
                if let Err(err) =
                    self.confirm_primary_render_surface_attachment((size.width, size.height))
                {
                    self.exit_with_error(
                        event_loop,
                        anyhow!("failed to confirm primary render surface attachment: {err:#}"),
                    );
                    return;
                }
            } else {
                self.exit_with_error(
                    event_loop,
                    anyhow!(
                        "preexisting runtime gfx cannot be proven attached to the newly created primary window"
                    ),
                );
                return;
            }
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

        window.request_redraw();
        self.register_runtime_window(NativeWindowId::primary(), window.clone());
        self.window = Some(window);
        self.apply_frame_pacing(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.windows.get(&window_id).cloned() else {
            tracing::debug!(
                target = "engine.runtime.window",
                ?window_id,
                "ignoring event for unknown native window"
            );
            return;
        };
        let Some(native_window_id) =
            native_window_id_for_winit_event(&self.native_windows_by_winit, window_id)
        else {
            tracing::debug!(
                target = "engine.runtime.window",
                ?window_id,
                "ignoring event for unregistered native window"
            );
            return;
        };

        self.dispatch_native_window_event(&window, &event);

        let result = match event {
            WindowEvent::CloseRequested => {
                self.apply_event_for_native_window(native_window_id, PlatformEvent::CloseRequested)
            }
            WindowEvent::Focused(focused) => self.apply_event_for_native_window(
                native_window_id,
                PlatformEvent::Focused { focused },
            ),
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
                    .window_context(native_window_id, device_id);
                let text = text_input(&event, is_synthetic);
                let input = keyboard_input(&event, is_synthetic);
                let mut result = self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::KeyboardInput { context, input },
                );
                if result.is_ok()
                    && let Some(text) = text
                {
                    result = self.apply_event_for_native_window(
                        native_window_id,
                        PlatformEvent::TextInput { text },
                    );
                }
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
                    .window_context(native_window_id, device_id);
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
                    .window_context(native_window_id, device_id);
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
                    .window_context(native_window_id, device_id);
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
                    .window_context(native_window_id, touch.device_id);
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
                if native_window_id != NativeWindowId::primary() {
                    let frame_result = self
                        .apply_event_for_native_window(
                            native_window_id,
                            PlatformEvent::RedrawRequested,
                        )
                        .and_then(|_| self.apply_window_effects(event_loop));
                    if let Err(err) = frame_result {
                        self.exit_with_error(
                            event_loop,
                            anyhow!("runtime secondary-window redraw failed: {err:#}"),
                        );
                    }
                    return;
                }
                let frame_result = self
                    .apply_event(PlatformEvent::RedrawRequested)
                    .and_then(|_| {
                        self.observe_primary_frame_started();
                        self.run_frame()
                    })
                    .and_then(|_| self.apply_window_effects(event_loop));
                if let Err(err) = frame_result {
                    self.exit_with_error(event_loop, anyhow!("runtime frame failed: {err:#}"));
                    return;
                }
                self.apply_frame_pacing(event_loop);
                return;
            }
            _ => Ok(()),
        };

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime window event failed: {err:#}"));
            return;
        }
        self.apply_frame_pacing(event_loop);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.dispatch_native_device_event(&event);

        let result = match event {
            DeviceEvent::MouseMotion { delta } => {
                let context = self.input_adapter.raw_device_context(device_id);
                let result = self.apply_raw_mouse_motion(context, delta.0 as f32, delta.1 as f32);
                if result.is_ok() {
                    self.request_redraw_for_native_window(NativeWindowId::primary());
                }
                result
            }
            _ => Ok(()),
        };

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime device event failed: {err:#}"));
            return;
        }
        self.apply_frame_pacing(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.apply_frame_pacing(event_loop);
    }
}

fn native_window_id_for_winit_event(
    native_windows_by_winit: &BTreeMap<WindowId, NativeWindowId>,
    window_id: WindowId,
) -> Option<NativeWindowId> {
    native_windows_by_winit.get(&window_id).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppNativeHostExt};
    use crate::plugins::render::backend::RenderSurfaceLifecycleState;
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

    fn runner_with_frame_pacing(policy: FramePacingPolicyResource) -> WinitRunner {
        let mut app = App::new();
        app.with_frame_pacing(policy);
        install_native_window_provider_resources(app.world_mut());
        app.world_mut()
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native Host test fixture should install window registry")
            .register_primary_window("test primary", (1280, 720), 1.0, true);
        WinitRunner {
            state: app.into_windowed_state(),
            window: None,
            windows: BTreeMap::new(),
            native_windows_by_winit: BTreeMap::new(),
            input_adapter: WinitInputAdapter::default(),
            last_primary_redraw_at: None,
            frame_pacing_schedule: FramePacingSchedule::default(),
            fatal_error: None,
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
            window: None,
            windows: BTreeMap::new(),
            native_windows_by_winit: BTreeMap::new(),
            input_adapter: WinitInputAdapter::default(),
            last_primary_redraw_at: None,
            frame_pacing_schedule: FramePacingSchedule::default(),
            fatal_error: None,
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

    #[test]
    fn winit_window_event_route_rejects_unknown_window_id() {
        let native_windows_by_winit = BTreeMap::new();
        assert_eq!(
            native_window_id_for_winit_event(&native_windows_by_winit, WindowId::dummy()),
            None
        );
    }

    #[test]
    fn winit_window_event_route_maps_known_primary_window() {
        let mut native_windows_by_winit = BTreeMap::new();
        native_windows_by_winit.insert(WindowId::dummy(), NativeWindowId::primary());
        assert_eq!(
            native_window_id_for_winit_event(&native_windows_by_winit, WindowId::dummy()),
            Some(NativeWindowId::primary())
        );
    }

    #[test]
    fn winit_window_event_route_maps_known_secondary_window() {
        let secondary_window =
            NativeWindowId::try_from_raw(2).expect("test native window id should be non-zero");
        let mut native_windows_by_winit = BTreeMap::new();
        native_windows_by_winit.insert(WindowId::dummy(), secondary_window);
        assert_eq!(
            native_window_id_for_winit_event(&native_windows_by_winit, WindowId::dummy()),
            Some(secondary_window)
        );
    }

    #[test]
    fn winit_window_event_route_rejects_retired_window_id() {
        let secondary_window =
            NativeWindowId::try_from_raw(2).expect("test native window id should be non-zero");
        let mut native_windows_by_winit = BTreeMap::new();
        native_windows_by_winit.insert(WindowId::dummy(), secondary_window);
        native_windows_by_winit.remove(&WindowId::dummy());
        assert_eq!(
            native_window_id_for_winit_event(&native_windows_by_winit, WindowId::dummy()),
            None
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
    fn primary_presentation_sync_does_not_manufacture_render_attachment() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner
            .state
            .world
            .insert_resource(RenderSurfaceRegistryResource::default());
        runner
            .sync_primary_presentation_and_surface_extent((1440, 900), 1.0)
            .expect("primary presentation sync should succeed");
        let surfaces = runner
            .state
            .world
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap();
        assert_eq!(surfaces.records().count(), 0);
        assert_eq!(surfaces.primary_surface_id(), None);
    }

    #[test]
    fn secondary_creation_failure_retires_reserved_render_surface() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner
            .state
            .world
            .insert_resource(RenderSurfaceRegistryResource::default());

        let request = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .request_window("Secondary", (900, 600));
        let render_surface_id = runner
            .state
            .world
            .resource_mut::<RenderSurfaceRegistryResource>()
            .expect("render surface registry should exist")
            .reserve_surface_for_native_window(request.native_window_id, request.size_px);

        runner.mark_window_creation_failed(request.native_window_id, "test attachment failure");

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

        let surfaces = runner
            .state
            .world
            .resource::<RenderSurfaceRegistryResource>()
            .expect("render surface registry should exist");
        assert_eq!(
            surfaces.surface_for_native_window(request.native_window_id),
            None
        );
        assert_eq!(
            surfaces
                .record(render_surface_id)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Retired)
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
    fn primary_input_redraws_stay_immediate_on_demand() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let now = Instant::now();

        assert!(runner.should_request_primary_redraw_now(now));
        assert!(runner.should_request_primary_redraw_now(now + std::time::Duration::from_secs(1)));
    }
}
