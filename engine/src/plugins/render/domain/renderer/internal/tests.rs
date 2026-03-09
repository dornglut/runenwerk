// Owner: Engine Renderer - Tests
#[cfg(test)]
mod tests {
    use super::{
        PassKind, PipelineKey, RenderGraphRegistryResource, Renderer, ResolvedFramePassDescriptor,
    };

    #[test]
    fn clip_to_scissor_clamps_and_rejects_empty() {
        let clipped = Renderer::clip_to_scissor([-10.0, 4.0, 20.0, 10.0], 100, 80)
            .expect("clip should intersect");
        assert_eq!(clipped, (0, 4, 10, 10));

        let none = Renderer::clip_to_scissor([200.0, 200.0, 10.0, 10.0], 100, 80);
        assert!(none.is_none());
    }

    #[test]
    fn build_frame_graph_from_descriptors_collects_diagnostics() {
        let renderer = Renderer::new();
        let descriptors = vec![
            ResolvedFramePassDescriptor {
                name: "".to_string(),
                kind: PassKind::Render,
                pipeline: PipelineKey::from("world_compose_fullscreen"),
                reads: Vec::new(),
                writes: Vec::new(),
                depends_on: Vec::new(),
                executor: "ui_composite".to_string(),
            },
            ResolvedFramePassDescriptor {
                name: "builtin_compute".to_string(),
                kind: PassKind::Compute,
                pipeline: PipelineKey::from("world_compute_basic"),
                reads: vec!["world_params".to_string()],
                writes: vec!["world_color".to_string()],
                depends_on: Vec::new(),
                executor: "builtin_compute".to_string(),
            },
            ResolvedFramePassDescriptor {
                name: "builtin_compute".to_string(),
                kind: PassKind::Compute,
                pipeline: PipelineKey::from("world_compute_high_contrast"),
                reads: vec!["world_params".to_string()],
                writes: vec!["world_color".to_string()],
                depends_on: Vec::new(),
                executor: "builtin_compute".to_string(),
            },
            ResolvedFramePassDescriptor {
                name: "builtin_compose".to_string(),
                kind: PassKind::Render,
                pipeline: PipelineKey::from("world_compose_fullscreen"),
                reads: vec!["world_color".to_string()],
                writes: vec!["surface_color".to_string()],
                depends_on: vec!["missing_pass".to_string()],
                executor: "builtin_compose".to_string(),
            },
        ];

        let output = renderer.build_frame_graph_from_descriptors(&descriptors);
        assert_eq!(output.handles.len(), 2);
        assert_eq!(output.diagnostics.empty_pass_name_count, 1);
        assert_eq!(
            output.diagnostics.duplicate_pass_names,
            vec!["builtin_compute".to_string()]
        );
        assert_eq!(
            output.diagnostics.missing_dependencies,
            vec![("builtin_compose".to_string(), "missing_pass".to_string())]
        );
    }

    #[test]
    fn build_frame_graph_reports_when_no_feature_graph_is_registered() {
        let renderer = Renderer::new();
        let output = renderer.build_frame_graph(&RenderGraphRegistryResource::default());
        assert!(output.handles.is_empty());
        assert!(output.diagnostics.no_registered_passes);
    }
}
