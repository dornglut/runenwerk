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
    fn shader_path_replacement_retains_generation_and_rejects_unloaded_source() {
        let root = temp_dir("shader_registry_path_revision");
        let first_path = Path::new(&root).join("first.wgsl");
        let second_path = Path::new(&root).join("second.wgsl");
        let missing_path = Path::new(&root).join("missing.wgsl");
        fs::write(&first_path, "fn first_source() {}\n").unwrap();
        fs::write(&second_path, "fn second_source() {}\n").unwrap();
        let mut registry = ShaderRegistryResource::with_roots(std::iter::empty::<String>());
        let handle =
            registry.register_shader_with_id("stable.preview", first_path.to_string_lossy());
        registry.poll_updates();
        let first_revision = registry.revision_for_handle(handle);
        assert!(first_revision > 0);

        assert_eq!(
            registry.register_shader_with_id("stable.preview", second_path.to_string_lossy()),
            handle
        );
        assert_eq!(registry.revision_for_handle(handle), 0);
        assert!(!registry.is_loaded("stable.preview"));
        assert_eq!(registry.source_or("stable.preview", "fallback"), "fallback");
        registry.poll_updates();
        let second_revision = registry.revision_for_handle(handle);
        assert!(second_revision > first_revision);
        assert_eq!(
            registry.loaded_source("stable.preview"),
            Some("fn second_source() {}\n")
        );

        // A failed edit of the current path preserves the last valid source.
        registry.drain_messages();
        fs::write(&second_path, "").unwrap();
        registry.request_reload();
        registry.poll_updates();
        assert_eq!(registry.revision_for_handle(handle), second_revision);
        assert_eq!(
            registry.loaded_source("stable.preview"),
            Some("fn second_source() {}\n")
        );
        assert_eq!(
            registry
                .read_messages()
                .iter()
                .find(|event| event.kind == ShaderRegistryEventKind::SkippedEmpty)
                .unwrap()
                .revision,
            second_revision
        );

        // A replacement path has no valid source until its own load succeeds.
        registry.drain_messages();
        registry.register_shader_with_id("stable.preview", missing_path.to_string_lossy());
        registry.poll_updates();
        assert_eq!(registry.revision_for_handle(handle), 0);
        assert!(!registry.is_loaded("stable.preview"));
        assert_eq!(
            registry
                .read_messages()
                .iter()
                .find(|event| event.kind == ShaderRegistryEventKind::Failed)
                .unwrap()
                .revision,
            0
        );
        registry.drain_messages();
        fs::write(&missing_path, "").unwrap();
        registry.request_reload();
        registry.poll_updates();
        assert_eq!(
            registry
                .read_messages()
                .iter()
                .find(|event| event.kind == ShaderRegistryEventKind::SkippedEmpty)
                .unwrap()
                .revision,
            0
        );
        assert_eq!(registry.revision_for_handle(handle), 0);
        fs::write(&missing_path, "fn third_source() {}\n").unwrap();
        registry.request_reload();
        registry.poll_updates();
        assert!(registry.revision_for_handle(handle) > second_revision);
        assert_eq!(registry.shader_count(), 1);
        let _ = fs::remove_dir_all(root);
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
