// Owner: SDF Renderer Example - Tests
use super::*;

#[cfg(test)]
mod tests {
    use super::{SdfParamsConfig, SdfRenderGraphConfig, parse_builtin_executor, parse_key_code};
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
    fn default_render_graph_config_registers_feature_executors() {
        let config = SdfRenderGraphConfig::default();
        let mut registry = RenderPassExecutorRegistryResource::default();
        let count = config
            .register_executor_bindings(&mut registry)
            .expect("default executor bindings should apply");
        assert_eq!(count, 2);
        assert!(registry.resolve_custom("sdf.compute").is_some());
        assert!(registry.resolve_custom("sdf.compose").is_some());
        assert_eq!(registry.resolve_builtin("sdf.compute"), None);
        assert_eq!(registry.resolve_builtin("sdf.compose"), None);
        assert_eq!(registry.resolve_builtin("builtin_ui_composite"), None);
    }

    #[test]
    fn params_config_parses_display_fit_mode() {
        let parsed: SdfParamsConfig = ron::from_str(
            r#"(
                display: (
                    fit_mode: contain,
                    target_aspect: 1.7777778,
                    render_scale: 1.5,
                    bar_color: (0.0, 0.0, 0.0, 1.0),
                ),
            )"#,
        )
        .expect("display config should parse");
        assert_eq!(parsed.display.fit_mode.as_shader_mode(), 1);
        assert!((parsed.display.target_aspect - 1.7777778).abs() < 0.0001);
        assert!((parsed.display.render_scale - 1.5).abs() < 0.0001);
    }

    #[test]
    fn params_asset_file_parses_display_config() {
        let parsed: SdfParamsConfig = ron::from_str(include_str!("assets/sdf_params.ron"))
            .expect("sdf_params.ron should parse");
        assert_eq!(parsed.display.fit_mode.as_shader_mode(), 0);
        assert!(parsed.display.target_aspect > 0.0001);
        assert!(parsed.display.render_scale > 1.0);
    }
}
