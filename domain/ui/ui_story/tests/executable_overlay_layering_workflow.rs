use ui_story::{UiStoryBuiltinWorkflowProfile, WORKFLOW_EXECUTABLE_INTERACTION_PROOF};

#[test]
fn executable_overlay_layering_consumes_existing_interaction_workflow_profile() {
    let graph = UiStoryBuiltinWorkflowProfile::ExecutableInteractionProof.graph();
    let order = graph
        .topological_nodes()
        .expect("overlay layering should reuse valid executable interaction workflow");

    assert_eq!(
        UiStoryBuiltinWorkflowProfile::ExecutableInteractionProof
            .profile_id()
            .as_str(),
        WORKFLOW_EXECUTABLE_INTERACTION_PROOF
    );
    assert!(order.iter().any(|node| node.node_id.as_str() == "interaction_story"));
    assert!(order.iter().any(|node| node.node_id.as_str() == "interaction_replay"));
    assert!(order.iter().any(|node| node.node_id.as_str() == "replay_live_parity"));
    assert!(order.iter().any(|node| node.node_id.as_str() == "interaction_static_mount"));
}
