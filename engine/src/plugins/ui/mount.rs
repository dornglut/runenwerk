use ui_surface::{
    MountedSurfaceInstance, SessionRetentionClass, SessionScopeHandle, SurfaceDefinitionId,
    SurfaceHostInstanceId, SurfaceInstanceId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiMountScreenId(String);

impl UiMountScreenId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl From<&str> for UiMountScreenId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for UiMountScreenId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountConfig {
    report_label: Option<String>,
    retention_class: SessionRetentionClass,
}

impl Default for UiMountConfig {
    fn default() -> Self {
        Self {
            report_label: None,
            retention_class: SessionRetentionClass::Restorable,
        }
    }
}

impl UiMountConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_report_label(mut self, label: impl Into<String>) -> Self {
        self.report_label = Some(label.into());
        self
    }

    pub fn with_retention_class(mut self, retention_class: SessionRetentionClass) -> Self {
        self.retention_class = retention_class;
        self
    }

    pub fn report_label(&self) -> Option<&str> {
        self.report_label.as_deref()
    }

    pub fn retention_class(&self) -> SessionRetentionClass {
        self.retention_class
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountRequest {
    screen_identity: UiMountScreenId,
    config: UiMountConfig,
}

impl UiMountRequest {
    pub fn new(screen_identity: impl Into<UiMountScreenId>) -> Self {
        Self {
            screen_identity: screen_identity.into(),
            config: UiMountConfig::default(),
        }
    }

    pub fn with_config(mut self, config: UiMountConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_report_label(mut self, label: impl Into<String>) -> Self {
        self.config = self.config.with_report_label(label);
        self
    }

    pub fn with_retention_class(mut self, retention_class: SessionRetentionClass) -> Self {
        self.config = self.config.with_retention_class(retention_class);
        self
    }

    pub fn screen_identity(&self) -> &str {
        self.screen_identity.as_str()
    }

    pub fn config(&self) -> &UiMountConfig {
        &self.config
    }

    pub(crate) fn failure_reason(&self) -> Option<UiMountFailureReason> {
        if self.screen_identity.is_blank() {
            Some(UiMountFailureReason::MissingScreenIdentity)
        } else {
            None
        }
    }
}

impl From<&str> for UiMountRequest {
    fn from(screen_identity: &str) -> Self {
        Self::new(screen_identity)
    }
}

impl From<String> for UiMountRequest {
    fn from(screen_identity: String) -> Self {
        Self::new(screen_identity)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiMountSource {
    AppMountUi,
    AppUiMount,
}

impl UiMountSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppMountUi => "app.mount_ui",
            Self::AppUiMount => "app.ui().mount",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiMountFailureReason {
    MissingScreenIdentity,
}

impl UiMountFailureReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingScreenIdentity => "missing_screen_identity",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::MissingScreenIdentity => "UI mount request is missing a screen identity",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountRecord {
    request: UiMountRequest,
    mount_source: UiMountSource,
}

impl UiMountRecord {
    pub(crate) fn new(request: UiMountRequest, mount_source: UiMountSource) -> Self {
        Self {
            request,
            mount_source,
        }
    }

    pub fn request(&self) -> &UiMountRequest {
        &self.request
    }

    pub fn mount_source(&self) -> UiMountSource {
        self.mount_source
    }

    pub fn screen_identity(&self) -> &str {
        self.request.screen_identity()
    }

    pub fn report_label(&self) -> Option<&str> {
        self.request.config().report_label()
    }

    pub fn retention_class(&self) -> SessionRetentionClass {
        self.request.config().retention_class()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountedSessionRecord {
    screen_identity: String,
    mount_source: UiMountSource,
    report_label: Option<String>,
    mounted_surface: MountedSurfaceInstance,
    session: SessionScopeHandle,
}

impl UiMountedSessionRecord {
    pub(crate) fn new(
        record: &UiMountRecord,
        mounted_surface: MountedSurfaceInstance,
        session: SessionScopeHandle,
    ) -> Self {
        Self {
            screen_identity: record.screen_identity().to_string(),
            mount_source: record.mount_source(),
            report_label: record.report_label().map(str::to_string),
            mounted_surface,
            session,
        }
    }

    pub(crate) fn replace_mounted_surface(&mut self, mounted_surface: MountedSurfaceInstance) {
        self.mounted_surface = mounted_surface;
    }

    pub fn screen_identity(&self) -> &str {
        &self.screen_identity
    }

    pub fn mount_source(&self) -> UiMountSource {
        self.mount_source
    }

    pub fn report_label(&self) -> Option<&str> {
        self.report_label.as_deref()
    }

    pub fn mounted_surface(&self) -> MountedSurfaceInstance {
        self.mounted_surface
    }

    pub fn session(&self) -> SessionScopeHandle {
        self.session
    }

    pub fn surface_instance_id(&self) -> SurfaceInstanceId {
        self.mounted_surface.surface_instance_id
    }

    pub fn definition_id(&self) -> SurfaceDefinitionId {
        self.mounted_surface.definition_id
    }

    pub fn host_instance_id(&self) -> SurfaceHostInstanceId {
        self.mounted_surface.host_instance_id
    }

    pub fn generation(&self) -> u64 {
        self.mounted_surface.generation
    }

    pub fn retention_class(&self) -> SessionRetentionClass {
        self.session.retention_class
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountReport {
    screen_identity: String,
    mount_source: UiMountSource,
    report_label: Option<String>,
    retention_class: SessionRetentionClass,
    accepted: bool,
    failure_reason: Option<UiMountFailureReason>,
    mounted_surface: Option<MountedSurfaceInstance>,
    session: Option<SessionScopeHandle>,
}

impl UiMountReport {
    pub(crate) fn accepted(
        record: &UiMountRecord,
        mounted_session: &UiMountedSessionRecord,
    ) -> Self {
        Self {
            screen_identity: record.screen_identity().to_string(),
            mount_source: record.mount_source(),
            report_label: record.report_label().map(str::to_string),
            retention_class: record.retention_class(),
            accepted: true,
            failure_reason: None,
            mounted_surface: Some(mounted_session.mounted_surface()),
            session: Some(mounted_session.session()),
        }
    }

    pub(crate) fn rejected(
        request: &UiMountRequest,
        mount_source: UiMountSource,
        failure_reason: UiMountFailureReason,
    ) -> Self {
        Self {
            screen_identity: request.screen_identity().to_string(),
            mount_source,
            report_label: request.config().report_label().map(str::to_string),
            retention_class: request.config().retention_class(),
            accepted: false,
            failure_reason: Some(failure_reason),
            mounted_surface: None,
            session: None,
        }
    }

    pub fn is_accepted(&self) -> bool {
        self.accepted
    }

    pub fn screen_identity(&self) -> &str {
        &self.screen_identity
    }

    pub fn mount_source(&self) -> UiMountSource {
        self.mount_source
    }

    pub fn report_label(&self) -> Option<&str> {
        self.report_label.as_deref()
    }

    pub fn retention_class(&self) -> SessionRetentionClass {
        self.retention_class
    }

    pub fn failure_reason(&self) -> Option<UiMountFailureReason> {
        self.failure_reason
    }

    pub fn mounted_surface(&self) -> Option<MountedSurfaceInstance> {
        self.mounted_surface
    }

    pub fn session(&self) -> Option<SessionScopeHandle> {
        self.session
    }

    pub fn surface_instance_id(&self) -> Option<SurfaceInstanceId> {
        self.mounted_surface
            .map(|surface| surface.surface_instance_id)
    }

    pub fn definition_id(&self) -> Option<SurfaceDefinitionId> {
        self.mounted_surface.map(|surface| surface.definition_id)
    }

    pub fn host_instance_id(&self) -> Option<SurfaceHostInstanceId> {
        self.mounted_surface.map(|surface| surface.host_instance_id)
    }

    pub fn generation(&self) -> Option<u64> {
        self.mounted_surface.map(|surface| surface.generation)
    }

    pub fn session_scope_id(&self) -> Option<u64> {
        self.session.map(|session| session.scope_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiUnmountReport {
    surface_instance_id: SurfaceInstanceId,
    removed: bool,
    generation: u64,
    remaining_mounted_surfaces: usize,
}

impl UiUnmountReport {
    pub(crate) fn new(
        surface_instance_id: SurfaceInstanceId,
        removed: bool,
        generation: u64,
        remaining_mounted_surfaces: usize,
    ) -> Self {
        Self {
            surface_instance_id,
            removed,
            generation,
            remaining_mounted_surfaces,
        }
    }

    pub fn surface_instance_id(&self) -> SurfaceInstanceId {
        self.surface_instance_id
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn remaining_mounted_surfaces(&self) -> usize {
        self.remaining_mounted_surfaces
    }
}
