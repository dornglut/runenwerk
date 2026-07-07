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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiMountConfig {
    report_label: Option<String>,
}

impl UiMountConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_report_label(mut self, label: impl Into<String>) -> Self {
        self.report_label = Some(label.into());
        self
    }

    pub fn report_label(&self) -> Option<&str> {
        self.report_label.as_deref()
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountReport {
    screen_identity: String,
    mount_source: UiMountSource,
    report_label: Option<String>,
    accepted: bool,
    failure_reason: Option<UiMountFailureReason>,
}

impl UiMountReport {
    pub(crate) fn accepted(record: &UiMountRecord) -> Self {
        Self {
            screen_identity: record.screen_identity().to_string(),
            mount_source: record.mount_source(),
            report_label: record.request().config().report_label().map(str::to_string),
            accepted: true,
            failure_reason: None,
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
            accepted: false,
            failure_reason: Some(failure_reason),
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

    pub fn failure_reason(&self) -> Option<UiMountFailureReason> {
        self.failure_reason
    }
}
