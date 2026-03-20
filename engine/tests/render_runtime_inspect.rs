use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, resource_kind_name, summarize_pass_timings,
};
use engine::plugins::render::RenderResourceDescriptor;

#[derive(Debug, Clone, Copy, engine::plugins::render::GpuStorage)]
struct InspectStorage {
    value: u32,
}

#[test]
fn runtime_timing_snapshot_preserves_flow_pass_kind_and_dispatch_metadata() {
    let samples = vec![
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compute".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.6,
            dispatch_workgroups: Some([20, 12, 1]),
        },
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 1.1,
            dispatch_workgroups: None,
        },
    ];
    let snapshot = summarize_pass_timings(&samples);

    assert_eq!(snapshot.per_pass.len(), 2);
    assert_eq!(snapshot.per_pass[0].flow_id, "flow.a");
    assert_eq!(snapshot.per_pass[0].pass_kind, "compute");
    assert_eq!(snapshot.per_pass[0].dispatch_workgroups, Some([20, 12, 1]));
    assert_eq!(snapshot.slowest_pass_id.as_deref(), Some("a.compose"));
}

#[test]
fn debug_timing_state_extracts_compute_dispatch_samples() {
    let mut state = RenderDebugTimingsState::default();
    state.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compute".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.6,
            dispatch_workgroups: Some([10, 4, 1]),
        },
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.2,
            dispatch_workgroups: None,
        },
    ]);

    assert_eq!(state.pass_sample_count, 2);
    assert_eq!(state.compute_dispatches.len(), 1);
    assert_eq!(state.compute_dispatches[0].flow_id, "flow.a");
    assert_eq!(state.compute_dispatches[0].pass_id, "a.compute");
    assert_eq!(state.compute_dispatches[0].workgroups, [10, 4, 1]);
}

#[test]
fn resource_kind_label_matches_descriptor_kind() {
    let descriptor = RenderResourceDescriptor::storage_buffer::<InspectStorage>("inspect.cells");
    assert_eq!(resource_kind_name(&descriptor), "storage_buffer");
}
