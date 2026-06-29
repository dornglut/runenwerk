use runenwerk_editor::editor_features::Phase12aInteractionProofHost;

#[test]
fn phase12a_interaction_proof_host_smoke() {
    let host = Phase12aInteractionProofHost::new();
    assert!(host.boundary_assertions().no_bypass_evidence());
}
