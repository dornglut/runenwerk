use crate::plugins::shared::{
    ReloadStatusPayload, file_modified, should_poll, should_reload, watch_status_line,
};
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
    use super::{ShaderRegistryEventKind, ShaderRegistryResource};
    use crate::plugins::render::shader::helpers::normalize_shader_id;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

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

        let events = registry.drain_events();
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
        let first = registry.register_shader("custom.main", "assets/shaders/custom_main.wgsl");
        let second = registry.register_shader("custom.main", "assets/shaders/custom_main_v2.wgsl");
        assert_eq!(first, second);
        assert_eq!(registry.shader_count(), 1);
    }
}
