use engine::plugins::render::graph::merge_flow_with_contributions;
use engine::plugins::render::{
    FragmentPassKind, FragmentPassSpec, FragmentReloadOutcome, FragmentResourceSpec, RenderFlow,
    RenderFlowFragmentHotReloadState, RenderFlowFragmentSpec, RenderFlowVariant,
    parse_fragment_ron,
};

#[test]
fn fragment_spec_parses_compiles_and_validates() {
    let ron = r#"
(
    namespace: "post",
    resources: [
        (kind: "imported_texture", id: "surface.color"),
        (kind: "color_target", id: "post.output"),
    ],
    passes: [
        (
            id: "post.tonemap",
            kind: fullscreen,
            reads: ["surface.color"],
            writes: ["post.output"],
        ),
        (
            id: "post.copy",
            kind: copy,
            reads: ["post.output"],
            writes: ["surface.color"],
            depends_on: ["post.tonemap"],
        ),
        (
            id: "post.present",
            kind: present,
            reads: ["surface.color"],
            depends_on: ["post.copy"],
        ),
    ],
)
"#;

    let spec = parse_fragment_ron(ron).expect("spec should parse");
    let contribution = spec
        .to_contribution()
        .expect("spec should compile into a contribution");
    assert_eq!(contribution.namespace(), "post");

    let merged =
        merge_flow_with_contributions(&RenderFlow::new("main.flow"), &[contribution]).unwrap();
    let order = merged.pass_order().expect("merged flow should validate");
    assert_eq!(order, vec!["post.tonemap", "post.copy", "post.present"]);
}

#[test]
fn fragment_spec_rejects_ids_outside_fragment_namespace() {
    let ron = r#"
(
    namespace: "post",
    resources: [(kind: "color_target", id: "post.output")],
    passes: [
        (
            id: "boids.simulate",
            kind: compute,
            writes: ["post.output"],
        ),
    ],
)
"#;

    let spec = parse_fragment_ron(ron).expect("spec should parse");
    let err = spec
        .to_contribution()
        .expect_err("foreign namespace ids must fail fragment validation");
    assert!(
        err.to_string()
            .contains("must stay in fragment namespace 'post'")
    );
}

#[test]
fn hot_reload_state_reports_updated_unchanged_and_failed_transitions() {
    let source_ok = r#"
(
    namespace: "post",
    resources: [(kind: "color_target", id: "post.output")],
    passes: [
        (
            id: "post.copy",
            kind: copy,
            reads: ["post.output"],
            writes: ["post.output"],
        ),
    ],
)
"#;
    let source_bad = "(namespace: \"post\", passes: [";

    let mut state = RenderFlowFragmentHotReloadState::default();

    let first = state.apply_source("assets/post_flow.ron", source_ok);
    match first {
        FragmentReloadOutcome::Updated {
            source_id,
            revision,
            ..
        } => {
            assert_eq!(source_id, "assets/post_flow.ron");
            assert_eq!(revision, 1);
        }
        _ => panic!("expected updated outcome"),
    }
    assert_eq!(state.revision(), 1);

    let second = state.apply_source("assets/post_flow.ron", source_ok);
    assert!(matches!(second, FragmentReloadOutcome::Unchanged));
    assert_eq!(state.revision(), 1);

    let third = state.apply_source("assets/post_flow.ron", source_bad);
    match third {
        FragmentReloadOutcome::Failed {
            source_id,
            revision,
            error,
        } => {
            assert_eq!(source_id, "assets/post_flow.ron");
            assert_eq!(revision, 2);
            assert!(error.contains("failed to parse render flow fragment"));
        }
        _ => panic!("expected failed outcome"),
    }
    assert_eq!(state.revision(), 2);
    assert!(
        state
            .last_error("assets/post_flow.ron")
            .is_some_and(|error| error.contains("failed to parse render flow fragment"))
    );
}

#[test]
fn variant_specific_contribution_ids_are_namespaced_for_editor_views() {
    let spec = RenderFlowFragmentSpec {
        namespace: "post".to_string(),
        variant: RenderFlowVariant::EditorViewport("scene-view".to_string()),
        resources: vec![FragmentResourceSpec::ColorTarget {
            id: "post.output".to_string(),
            transient: false,
        }],
        passes: vec![FragmentPassSpec {
            id: "post.compose".to_string(),
            kind: FragmentPassKind::Fullscreen,
            shader: None,
            reads: vec![],
            writes: vec!["post.output".to_string()],
            depends_on: vec![],
            sampled_textures: vec![],
            write_textures: vec![],
            vertex_buffers: vec![],
            index_buffers: vec![],
            instance_buffers: vec![],
            indirect_buffers: vec![],
            depth_target: None,
            workgroup_size: None,
            clear_color: None,
        }],
    };

    let contribution = spec
        .to_contribution()
        .expect("editor viewport variant should compile");
    assert_eq!(contribution.namespace(), "post_editor_scene_view");
}
