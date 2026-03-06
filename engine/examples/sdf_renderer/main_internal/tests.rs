// Owner: SDF Renderer Example - Tests
#[cfg(test)]
mod tests {
    use super::{SdfRenderGraphConfig, parse_builtin_executor, parse_key_code};
    use engine::plugins::render::domain::RenderPassExecutorRegistryResource;
    use winit::keyboard::KeyCode;

    #[test]
    fn key_code_parser_accepts_named_and_compact_forms() {
        assert_eq!(parse_key_code("KeyR"), Some(KeyCode::KeyR));
        assert_eq!(parse_key_code("R"), Some(KeyCode::KeyR));
        assert_eq!(parse_key_code("Digit2"), Some(KeyCode::Digit2));
        assert_eq!(parse_key_code("2"), Some(KeyCode::Digit2));
        assert_eq!(parse_key_code("F10"), Some(KeyCode::F10));
        assert_eq!(parse_key_code("ArrowLeft"), Some(KeyCode::ArrowLeft));
        assert_eq!(parse_key_code("Backquote"), Some(KeyCode::Backquote));
        assert_eq!(parse_key_code("unknown"), None);
    }

    #[test]
    fn default_render_graph_config_converts_to_spec() {
        let spec = SdfRenderGraphConfig::default()
            .to_spec()
            .expect("default render graph config should convert to a typed spec");
        assert_eq!(spec.feature.as_str(), "sdf_renderer_example");
        assert_eq!(spec.passes.len(), 3);
    }

    #[test]
    fn builtin_executor_parser_accepts_builtin_labels() {
        assert_eq!(
            parse_builtin_executor("builtin_compute"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::Compute)
        );
        assert_eq!(
            parse_builtin_executor("builtin_compose"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::Compose)
        );
        assert_eq!(
            parse_builtin_executor("builtin_mesh_overlay"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::MeshOverlay)
        );
        assert_eq!(
            parse_builtin_executor("builtin_ui_composite"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::UiComposite)
        );
        assert_eq!(parse_builtin_executor("unknown"), None);
    }

    #[test]
    fn default_render_graph_config_registers_custom_executors() {
        let config = SdfRenderGraphConfig::default();
        let mut registry = RenderPassExecutorRegistryResource::default();
        let count = config
            .register_custom_executors(&mut registry)
            .expect("default executor bindings should apply");
        assert_eq!(count, 3);
        assert!(registry.resolve_custom("sdf.compute").is_some());
        assert!(registry.resolve_custom("sdf.compose").is_some());
        assert!(registry.resolve_custom("ui_composite").is_some());
        assert_eq!(registry.resolve_builtin("sdf.compute"), None);
        assert_eq!(registry.resolve_builtin("sdf.compose"), None);
    }
}
