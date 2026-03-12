// Owner: SDF Renderer Example - Tests
use super::*;

#[cfg(test)]
mod tests {
    use super::{
        SdfInputBindingsConfig, SdfParamsConfig, SdfWorldState, app_input_bindings, build_render_flow,
        parse_key_code,
    };
    use engine::plugins::render::GpuParams;
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
    fn app_layer_input_bindings_are_mapped_from_config() {
        let bindings = app_input_bindings(&SdfInputBindingsConfig::default());
        assert!(!bindings.is_empty());
        assert!(bindings.iter().any(|(action, key)| {
            *action == crate::runtime::ACTION_UP && *key == KeyCode::KeyR
        }));
    }

    #[test]
    fn default_render_flow_validates_and_contains_expected_pass_order() {
        let flow = build_render_flow();
        let report = flow
            .validate()
            .expect("sdf renderer flow should validate");

        assert_eq!(
            report.pass_order,
            vec!["sdf.compute", "sdf.compose", "ui.composite"]
        );
    }

    #[test]
    fn state_projection_methods_produce_gpu_params() {
        let state = SdfWorldState::default();
        let compute = state.compute_params_with_surface((1280, 720)).to_gpu();
        let compose = state.compose_params((1280, 720)).to_gpu();

        assert!(compute.screen_size[0] > 0.0);
        assert!(compute.screen_size[1] > 0.0);
        assert_eq!(compose.output_size, [1280.0, 720.0]);
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
