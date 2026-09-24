use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ReloadStatusPayload {
    pub kind: &'static str,
    pub id: String,
    pub state: String,
    pub source_path: String,
    pub revision: u64,
    pub watch_enabled: bool,
    pub modified: Option<SystemTime>,
    pub error: Option<String>,
    pub details: Option<String>,
}

impl ReloadStatusPayload {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: &'static str,
        id: impl Into<String>,
        state: impl Into<String>,
        source_path: impl Into<String>,
        revision: u64,
        watch_enabled: bool,
        modified: Option<SystemTime>,
        error: Option<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            kind,
            id: id.into(),
            state: state.into(),
            source_path: source_path.into(),
            revision,
            watch_enabled,
            modified,
            error,
            details,
        }
    }

    pub fn line(&self) -> String {
        let mut line = format!(
            "reload kind={} id={} state={} rev={} watch={} path={}",
            self.kind,
            self.id,
            self.state,
            self.revision,
            on_off(self.watch_enabled),
            self.source_path,
        );
        if let Some(modified_unix) = self.modified.and_then(system_time_to_unix_seconds) {
            line.push_str(&format!(" modified_unix={modified_unix}"));
        }
        if let Some(details) = &self.details {
            line.push_str(&format!(" details={details}"));
        }
        if let Some(error) = &self.error {
            line.push_str(&format!(" error={error}"));
        }
        line
    }
}

pub fn file_modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

pub fn should_poll(watch_enabled: bool, force_reload: bool) -> bool {
    watch_enabled || force_reload
}

pub fn should_reload(
    watch_enabled: bool,
    force_reload: bool,
    previous_modified: Option<SystemTime>,
    current_modified: Option<SystemTime>,
) -> bool {
    should_poll(watch_enabled, force_reload)
        && (force_reload || previous_modified != current_modified)
}

pub fn watch_status_line(kind: &'static str, watch_enabled: bool, source: &str) -> String {
    format!(
        "reload kind={} watch={} source={}",
        kind,
        on_off(watch_enabled),
        source
    )
}

pub fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn system_time_to_unix_seconds(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{ReloadStatusPayload, should_reload};
    use std::time::SystemTime;

    #[test]
    fn should_reload_obeys_watch_or_force() {
        let now = Some(SystemTime::now());
        assert!(!should_reload(false, false, now, now));
        assert!(should_reload(false, true, now, now));
        assert!(should_reload(true, false, None, now));
    }

    #[test]
    fn status_payload_line_contains_core_fields() {
        let payload = ReloadStatusPayload::new(
            "shader",
            "ui_rect",
            "reloaded",
            "assets/shaders/ui_rect.wgsl",
            3,
            true,
            None,
            None,
            None,
        );
        let line = payload.line();
        assert!(line.contains("kind=shader"));
        assert!(line.contains("id=ui_rect"));
        assert!(line.contains("state=reloaded"));
        assert!(line.contains("rev=3"));
    }
}
