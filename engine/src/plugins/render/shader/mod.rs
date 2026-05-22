pub use crate::plugins::shared::ReloadStatusPayload;
use crate::plugins::shared::{file_modified, should_poll, should_reload, watch_status_line};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

mod helpers;
mod hot_reload;
mod registry;
mod types;

pub use hot_reload::*;
pub use types::*;

// Owner: Engine Render Shader Registry - Tests
#[cfg(test)]
mod tests {
    use super::{ShaderRegistryEventKind, ShaderRegistryResource, ShaderReloadPollStatus};
    use crate::plugins::render::shader::helpers::normalize_shader_id;
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> String {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{unique}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn normalize_id_uses_relative_style_segments() {
        let id = normalize_shader_id("assets/shaders/ui/panel_text.wgsl");
        assert_eq!(id, "assets.shaders.ui.panel_text.wgsl");
    }

    #[test]
    fn poll_updates_discovers_and_loads_shader_files() {
        let root = temp_dir("shader_registry_discovery");
        let file = Path::new(&root).join("ui_rect.wgsl");
        fs::write(
            &file,
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
        )
        .expect("shader should be written");

        let mut registry = ShaderRegistryResource::with_roots([root.clone()]);
        let lines = registry.poll_updates();
        assert!(!lines.is_empty());
        assert_eq!(registry.shader_count(), 1);
        assert!(registry.handle("ui_rect").is_some());
        let src = registry.source_or("ui_rect", "fallback");
        assert_ne!(src, "fallback");
        let path_lookup = file.to_string_lossy().to_string();
        assert_ne!(
            registry.source_or(path_lookup.as_str(), "fallback"),
            "fallback"
        );
        assert!(
            registry.revision_for(path_lookup.as_str()) > 0,
            "path-based shader lookup should carry loaded revisions"
        );

        let events = registry.drain_messages();
        assert!(
            events
                .iter()
                .any(|event| event.kind == ShaderRegistryEventKind::Discovered)
        );
        assert!(
            events
                .iter()
                .any(|event| event.kind == ShaderRegistryEventKind::Reloaded)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn register_shader_returns_stable_handle_for_same_id() {
        let mut registry = ShaderRegistryResource::with_roots(["assets/shaders"]);
        let first =
            registry.register_shader_with_id("custom.main", "assets/shaders/custom_main.wgsl");
        let second =
            registry.register_shader_with_id("custom.main", "assets/shaders/custom_main_v2.wgsl");
        assert_eq!(first, second);
        assert_eq!(registry.shader_count(), 1);
    }

    #[test]
    fn shader_reload_poll_first_poll_is_immediate_then_throttled() {
        let root = temp_dir("shader_registry_throttle");
        let file = Path::new(&root).join("main.wgsl");
        fs::write(
            &file,
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
        )
        .expect("shader should be written");

        let now = Instant::now();
        let mut registry = ShaderRegistryResource::with_roots([root.clone()]);
        registry.set_reload_poll_interval(Duration::from_millis(500));

        let first = registry.poll_updates_at(now);
        assert!(!first.is_empty());
        assert_eq!(
            registry.last_reload_poll_report().status,
            ShaderReloadPollStatus::Polled
        );

        let second = registry.poll_updates_at(now + Duration::from_millis(100));
        assert!(second.is_empty());
        assert_eq!(
            registry.last_reload_poll_report().status,
            ShaderReloadPollStatus::Throttled
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shader_reload_poll_force_reload_bypasses_throttle() {
        let root = temp_dir("shader_registry_force_reload");
        let file = Path::new(&root).join("main.wgsl");
        fs::write(
            &file,
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
        )
        .expect("shader should be written");

        let now = Instant::now();
        let mut registry = ShaderRegistryResource::with_roots([root.clone()]);
        registry.set_reload_poll_interval(Duration::from_millis(500));
        let _ = registry.poll_updates_at(now);
        registry.request_reload();

        let forced = registry.poll_updates_at(now + Duration::from_millis(100));
        assert!(!forced.is_empty());
        let report = registry.last_reload_poll_report();
        assert_eq!(report.status, ShaderReloadPollStatus::Polled);
        assert!(report.force_reload);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shader_reload_poll_elapsed_interval_allows_next_poll() {
        let root = temp_dir("shader_registry_elapsed_reload");
        let file = Path::new(&root).join("main.wgsl");
        fs::write(
            &file,
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
        )
        .expect("shader should be written");

        let now = Instant::now();
        let mut registry = ShaderRegistryResource::with_roots([root.clone()]);
        registry.set_reload_poll_interval(Duration::from_millis(500));
        let _ = registry.poll_updates_at(now);

        let elapsed = registry.poll_updates_at(now + Duration::from_millis(500));
        assert!(elapsed.is_empty());
        assert_eq!(
            registry.last_reload_poll_report().status,
            ShaderReloadPollStatus::Polled
        );

        let _ = fs::remove_dir_all(root);
    }
}
