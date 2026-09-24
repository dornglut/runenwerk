use id_macros::id;
use std::collections::BTreeMap;

#[id]
pub struct NativeWindowId;

impl NativeWindowId {
    pub fn primary() -> Self {
        Self::try_from_raw(1).expect("primary native window id must be non-zero")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCursorIcon {
    Default,
    ColResize,
    RowResize,
    NwseResize,
    NeswResize,
    Grab,
    Grabbing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeWindowLifecycleState {
    Requested,
    Created,
    CreationFailed,
    CloseIntentPending,
    CloseApproved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWindowCreationRequest {
    pub native_window_id: NativeWindowId,
    pub title: String,
    pub size_px: (u32, u32),
}

impl NativeWindowCreationRequest {
    pub fn new(
        native_window_id: NativeWindowId,
        title: impl Into<String>,
        size_px: (u32, u32),
    ) -> Self {
        Self {
            native_window_id,
            title: title.into(),
            size_px: normalize_extent(size_px),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWindowRecord {
    pub native_window_id: NativeWindowId,
    pub title: String,
    pub size_px: (u32, u32),
    pub scale_factor: f64,
    pub close_requested: bool,
    pub close_intent_pending: bool,
    pub focused: bool,
    pub redraw_requested: bool,
    pub cursor_icon: WindowCursorIcon,
    pub lifecycle_state: NativeWindowLifecycleState,
    pub failure_reason: Option<String>,
}

impl NativeWindowRecord {
    fn requested(request: &NativeWindowCreationRequest) -> Self {
        Self {
            native_window_id: request.native_window_id,
            title: request.title.clone(),
            size_px: request.size_px,
            scale_factor: 1.0,
            close_requested: false,
            close_intent_pending: false,
            focused: false,
            redraw_requested: true,
            cursor_icon: WindowCursorIcon::Default,
            lifecycle_state: NativeWindowLifecycleState::Requested,
            failure_reason: None,
        }
    }

    fn created(
        native_window_id: NativeWindowId,
        title: impl Into<String>,
        size_px: (u32, u32),
        scale_factor: f64,
        focused: bool,
    ) -> Self {
        Self {
            native_window_id,
            title: title.into(),
            size_px: normalize_extent(size_px),
            scale_factor,
            close_requested: false,
            close_intent_pending: false,
            focused,
            redraw_requested: true,
            cursor_icon: WindowCursorIcon::Default,
            lifecycle_state: NativeWindowLifecycleState::Created,
            failure_reason: None,
        }
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    pub fn request_close(&mut self) {
        self.approve_close();
    }

    pub fn receive_close_intent(&mut self) {
        self.close_requested = false;
        self.close_intent_pending = true;
        self.lifecycle_state = NativeWindowLifecycleState::CloseIntentPending;
    }

    pub fn approve_close(&mut self) {
        self.close_requested = true;
        self.close_intent_pending = false;
        self.lifecycle_state = NativeWindowLifecycleState::CloseApproved;
    }

    pub fn veto_close(&mut self) {
        self.close_requested = false;
        self.close_intent_pending = false;
        self.lifecycle_state = NativeWindowLifecycleState::Created;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.request_redraw();
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: WindowCursorIcon) {
        if self.cursor_icon != cursor_icon {
            self.cursor_icon = cursor_icon;
            self.request_redraw();
        }
    }

    pub fn mark_creation_failed(&mut self, reason: impl Into<String>) {
        self.close_requested = false;
        self.close_intent_pending = false;
        self.lifecycle_state = NativeWindowLifecycleState::CreationFailed;
        self.failure_reason = Some(reason.into());
    }
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct WindowStateRegistryResource {
    primary_window_id: Option<NativeWindowId>,
    next_window_raw: u64,
    records: BTreeMap<NativeWindowId, NativeWindowRecord>,
    pending_creation_requests: Vec<NativeWindowCreationRequest>,
}

impl WindowStateRegistryResource {
    pub fn primary_window_id(&self) -> Option<NativeWindowId> {
        self.primary_window_id
    }

    pub fn record(&self, native_window_id: NativeWindowId) -> Option<&NativeWindowRecord> {
        self.records.get(&native_window_id)
    }

    pub fn record_mut(
        &mut self,
        native_window_id: NativeWindowId,
    ) -> Option<&mut NativeWindowRecord> {
        self.records.get_mut(&native_window_id)
    }

    pub fn records(&self) -> impl Iterator<Item = &NativeWindowRecord> {
        self.records.values()
    }

    pub fn register_primary_window(
        &mut self,
        title: impl Into<String>,
        size_px: (u32, u32),
        scale_factor: f64,
        focused: bool,
    ) -> NativeWindowId {
        let primary = NativeWindowId::primary();
        self.primary_window_id = Some(primary);
        self.next_window_raw = self.next_window_raw.max(primary.raw().saturating_add(1));
        self.records.insert(
            primary,
            NativeWindowRecord::created(primary, title, size_px, scale_factor, focused),
        );
        primary
    }

    pub fn request_window(
        &mut self,
        title: impl Into<String>,
        size_px: (u32, u32),
    ) -> NativeWindowCreationRequest {
        let native_window_id = self.allocate_window_id();
        let request = NativeWindowCreationRequest::new(native_window_id, title, size_px);
        self.records
            .insert(native_window_id, NativeWindowRecord::requested(&request));
        self.pending_creation_requests.push(request.clone());
        request
    }

    pub fn register_created_window(
        &mut self,
        native_window_id: NativeWindowId,
        title: impl Into<String>,
        size_px: (u32, u32),
        scale_factor: f64,
        focused: bool,
    ) {
        self.records.insert(
            native_window_id,
            NativeWindowRecord::created(native_window_id, title, size_px, scale_factor, focused),
        );
        self.next_window_raw = self
            .next_window_raw
            .max(native_window_id.raw().saturating_add(1));
        if native_window_id == NativeWindowId::primary() {
            self.primary_window_id = Some(native_window_id);
        }
    }

    pub fn take_pending_creation_requests(&mut self) -> Vec<NativeWindowCreationRequest> {
        std::mem::take(&mut self.pending_creation_requests)
    }

    pub fn pending_creation_requests(&self) -> &[NativeWindowCreationRequest] {
        &self.pending_creation_requests
    }

    pub fn remove_window(
        &mut self,
        native_window_id: NativeWindowId,
    ) -> Option<NativeWindowRecord> {
        self.pending_creation_requests
            .retain(|request| request.native_window_id != native_window_id);
        let removed = self.records.remove(&native_window_id);
        if self.primary_window_id == Some(native_window_id) {
            self.primary_window_id = None;
        }
        removed
    }

    fn allocate_window_id(&mut self) -> NativeWindowId {
        loop {
            let raw = self
                .next_window_raw
                .max(NativeWindowId::primary().raw().saturating_add(1));
            self.next_window_raw = raw.saturating_add(1);
            if let Ok(id) = NativeWindowId::try_from_raw(raw)
                && !self.records.contains_key(&id)
            {
                return id;
            }
        }
    }
}

fn normalize_extent((width, height): (u32, u32)) -> (u32, u32) {
    (width.max(1), height.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_registry_registers_primary_without_legacy_mirror() {
        let mut registry = WindowStateRegistryResource::default();

        let primary = registry.register_primary_window("Runtime", (1440, 900), 1.5, true);
        let record = registry
            .record(primary)
            .expect("primary native window record should be present");

        assert_eq!(primary, NativeWindowId::primary());
        assert_eq!(record.title, "Runtime");
        assert_eq!(record.size_px, (1440, 900));
        assert_eq!(record.scale_factor, 1.5);
        assert!(record.focused);
        assert_eq!(record.lifecycle_state, NativeWindowLifecycleState::Created);
    }

    #[test]
    fn primary_registration_preserves_realized_focus_state() {
        let mut registry = WindowStateRegistryResource::default();

        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0, false);

        assert!(!registry.record(primary).unwrap().focused);
    }

    #[test]
    fn window_registry_tracks_pending_secondary_window_requests() {
        let mut registry = WindowStateRegistryResource::default();
        registry.register_primary_window("Runtime", (1280, 720), 1.0, true);

        let request = registry.request_window("Secondary", (640, 480));

        assert_ne!(request.native_window_id, NativeWindowId::primary());
        assert_eq!(request.title, "Secondary");
        assert_eq!(
            registry.pending_creation_requests(),
            std::slice::from_ref(&request)
        );
        assert_eq!(
            registry
                .record(request.native_window_id)
                .map(|record| record.lifecycle_state),
            Some(NativeWindowLifecycleState::Requested)
        );
    }

    #[test]
    fn created_secondary_replaces_requested_record_with_realized_native_facts() {
        let mut registry = WindowStateRegistryResource::default();
        registry.register_primary_window("Runtime", (1280, 720), 1.0, true);
        let request = registry.request_window("Secondary", (640, 480));

        registry.register_created_window(
            request.native_window_id,
            "Secondary",
            (800, 600),
            2.0,
            false,
        );

        let record = registry.record(request.native_window_id).unwrap();
        assert_eq!(record.size_px, (800, 600));
        assert_eq!(record.scale_factor, 2.0);
        assert_eq!(record.lifecycle_state, NativeWindowLifecycleState::Created);
    }
}
