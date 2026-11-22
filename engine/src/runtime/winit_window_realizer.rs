use crate::app::WindowedAppState;
use crate::runtime::window::{
    NativeWindowCreationRequest, NativeWindowId, NativeWindowRecord,
    PrimaryWindowInitialSizePxResource, WindowCursorIcon, WindowStateRegistryResource,
};
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

/// The original Native/Render primary has a fixed identity. Physical
/// presence alone cannot tell whether ID 1 was permanently retired.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeOriginalPrimaryLifetime {
    #[default]
    NeverRealized,
    Realized,
    Retired,
}

impl NativeOriginalPrimaryLifetime {
    fn can_publish(self) -> bool {
        self == Self::NeverRealized
    }

    fn publish(&mut self) -> Result<()> {
        if !self.can_publish() {
            return Err(anyhow!(
                "original primary Native identity cannot be realized a second time"
            ));
        }
        *self = Self::Realized;
        Ok(())
    }

    fn retire(&mut self) {
        if *self == Self::Realized {
            *self = Self::Retired;
        }
    }
}

#[derive(Default)]
pub(crate) struct WinitWindowRealizer {
    original_primary_lifetime: NativeOriginalPrimaryLifetime,
    primary_window: Option<Arc<Window>>,
    windows: BTreeMap<WindowId, Arc<Window>>,
    native_windows_by_winit: BTreeMap<WindowId, NativeWindowId>,
}

impl WinitWindowRealizer {
    pub(crate) fn has_primary_window(&self) -> bool {
        self.primary_window.is_some()
    }

    /// Admission is one-shot even across Winit Suspended/Resumed transitions.
    pub(crate) fn may_realize_initial_primary(&self) -> bool {
        self.original_primary_lifetime.can_publish()
    }

    pub(crate) fn has_any_window(&self) -> bool {
        !self.windows.is_empty()
    }

    pub(crate) fn primary_window(&self) -> Option<Arc<Window>> {
        self.primary_window.clone()
    }

    pub(crate) fn window(&self, window_id: WindowId) -> Option<Arc<Window>> {
        self.windows.get(&window_id).cloned()
    }

    pub(crate) fn native_window_id(&self, window_id: WindowId) -> Option<NativeWindowId> {
        self.native_windows_by_winit.get(&window_id).copied()
    }

    pub(crate) fn realized_windows(&self) -> Vec<(WindowId, NativeWindowId, Arc<Window>)> {
        self.windows
            .iter()
            .filter_map(|(window_id, window)| {
                self.native_windows_by_winit
                    .get(window_id)
                    .copied()
                    .map(|native_window_id| (*window_id, native_window_id, Arc::clone(window)))
            })
            .collect()
    }

    pub(crate) fn realize_primary(
        &self,
        event_loop: &ActiveEventLoop,
        state: &WindowedAppState,
    ) -> Result<Arc<Window>> {
        event_loop
            .create_window(primary_window_attributes(state))
            .map(Arc::new)
            .map_err(|err| anyhow!("failed to create runtime window: {err}"))
    }

    pub(crate) fn realize_secondary(
        &self,
        event_loop: &ActiveEventLoop,
        request: &NativeWindowCreationRequest,
    ) -> Result<Arc<Window>> {
        let attrs = Window::default_attributes()
            .with_title(request.title.clone())
            .with_inner_size(winit::dpi::PhysicalSize::new(
                request.size_px.0,
                request.size_px.1,
            ));
        event_loop
            .create_window(attrs)
            .map(Arc::new)
            .map_err(|err| anyhow!("native window creation failed: {err}"))
    }

    pub(crate) fn project_primary_window(
        &self,
        registry: &mut WindowStateRegistryResource,
        window: &Window,
    ) -> ((u32, u32), f64) {
        let size = window.inner_size();
        let size_px = (size.width.max(1), size.height.max(1));
        let scale_factor = window.scale_factor();
        registry.register_primary_window(
            window.title().to_string(),
            size_px,
            scale_factor,
            window.has_focus(),
        );
        (size_px, scale_factor)
    }

    pub(crate) fn project_created_window(
        &self,
        registry: &mut WindowStateRegistryResource,
        native_window_id: NativeWindowId,
        window: &Window,
    ) {
        let size = window.inner_size();
        registry.register_created_window(
            native_window_id,
            window.title().to_string(),
            (size.width.max(1), size.height.max(1)),
            window.scale_factor(),
            window.has_focus(),
        );
    }

    pub(crate) fn mark_creation_failed(
        &self,
        registry: &mut WindowStateRegistryResource,
        native_window_id: NativeWindowId,
        reason: impl Into<String>,
    ) {
        if let Some(record) = registry.record_mut(native_window_id) {
            record.mark_creation_failed(reason);
        }
    }

    pub(crate) fn publish_window(
        &mut self,
        native_window_id: NativeWindowId,
        window: Arc<Window>,
    ) -> Result<()> {
        // Verify the fixed original-primary identity before mutating any map.
        if native_window_id == NativeWindowId::primary() {
            self.original_primary_lifetime.publish()?;
        }
        let window_id = window.id();
        self.publish_mapping(window_id, native_window_id);
        self.windows.insert(window_id, Arc::clone(&window));
        if native_window_id == NativeWindowId::primary() {
            self.primary_window = Some(window);
        }
        Ok(())
    }

    pub(crate) fn remove_window(&mut self, window_id: WindowId) {
        let native_window_id = self.retire_mapping(window_id);
        self.windows.remove(&window_id);
        if native_window_id == Some(NativeWindowId::primary()) {
            self.primary_window = None;
            self.original_primary_lifetime.retire();
        }
    }

    pub(crate) fn apply_physical_record(window: &Window, record: &NativeWindowRecord) {
        if window.title() != record.title {
            window.set_title(&record.title);
        }
        window.set_cursor(winit_cursor_icon(record.cursor_icon));
    }

    pub(crate) fn request_redraw(window: &Window) {
        window.request_redraw();
    }

    fn publish_mapping(&mut self, window_id: WindowId, native_window_id: NativeWindowId) {
        self.native_windows_by_winit
            .insert(window_id, native_window_id);
    }

    fn retire_mapping(&mut self, window_id: WindowId) -> Option<NativeWindowId> {
        self.native_windows_by_winit.remove(&window_id)
    }
}

fn requested_primary_window_size_px(state: &WindowedAppState) -> Option<(u32, u32)> {
    state
        .world
        .resource::<PrimaryWindowInitialSizePxResource>()
        .ok()
        .map(|request| request.size_px())
}

fn primary_window_attributes(state: &WindowedAppState) -> WindowAttributes {
    let attrs = Window::default_attributes().with_title(state.title.clone());
    let Some((width, height)) = requested_primary_window_size_px(state) else {
        return attrs;
    };
    attrs.with_inner_size(winit::dpi::PhysicalSize::new(width, height))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppNativeHostExt};

    #[test]
    fn original_primary_identity_never_reopens_after_retirement() {
        let mut lifetime = NativeOriginalPrimaryLifetime::default();
        assert!(lifetime.can_publish());
        lifetime.publish().unwrap();
        assert!(!lifetime.can_publish());
        assert!(lifetime.publish().is_err(), "double realization must fail closed");
        lifetime.retire();
        assert_eq!(lifetime, NativeOriginalPrimaryLifetime::Retired);
        assert!(!lifetime.can_publish());
        assert!(lifetime.publish().is_err(), "retired fixed ID cannot be recreated");
        lifetime.retire();
        assert_eq!(lifetime, NativeOriginalPrimaryLifetime::Retired);
    }

    #[test]
    fn correlation_routes_known_primary_and_secondary_windows() {
        let primary_window_id = WindowId::dummy();
        let secondary_window_id = WindowId::dummy();
        let secondary =
            NativeWindowId::try_from_raw(2).expect("secondary native window id should be valid");
        let mut realizer = WinitWindowRealizer::default();

        realizer.publish_mapping(primary_window_id, NativeWindowId::primary());
        assert_eq!(
            realizer.native_window_id(primary_window_id),
            Some(NativeWindowId::primary())
        );

        realizer.publish_mapping(secondary_window_id, secondary);
        assert_eq!(
            realizer.native_window_id(secondary_window_id),
            Some(secondary)
        );
    }

    #[test]
    fn correlation_rejects_unknown_and_retired_windows() {
        let window_id = WindowId::dummy();
        let secondary =
            NativeWindowId::try_from_raw(2).expect("secondary native window id should be valid");
        let mut realizer = WinitWindowRealizer::default();

        assert_eq!(realizer.native_window_id(window_id), None);
        realizer.publish_mapping(window_id, secondary);
        assert_eq!(realizer.retire_mapping(window_id), Some(secondary));
        assert_eq!(realizer.native_window_id(window_id), None);
    }

    #[test]
    fn primary_window_size_request_is_absent_without_explicit_host_configuration() {
        let app = App::new();
        let state = app.into_windowed_state();

        assert_eq!(requested_primary_window_size_px(&state), None);
    }

    #[test]
    fn primary_window_size_request_reaches_realization_boundary() {
        let mut app = App::new();
        app.with_primary_window_size_px((1600, 1200));
        let state = app.into_windowed_state();

        assert_eq!(requested_primary_window_size_px(&state), Some((1600, 1200)));
    }

    #[test]
    fn winit_driver_does_not_import_render_surface_implementation_types() {
        let driver = include_str!("winit_runner.rs");
        for forbidden in ["RenderSurfaceId", "RenderSurfaceRegistryResource", "Gfx"] {
            assert!(
                !driver.contains(forbidden),
                "winit driver must not name Render implementation type {forbidden}"
            );
        }
    }
}
