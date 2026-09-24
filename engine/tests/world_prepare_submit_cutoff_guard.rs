use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn render_submit_consumes_prepared_world_resources_only() {
    let prepare_source = read("src/plugins/render/runtime/frame_prepare.rs");
    let submit_source = read("src/plugins/render/runtime/frame_submit.rs");

    assert!(
        prepare_source.contains("PreparedWorldFeatureResource"),
        "prepare path must ingest world prepared contributions"
    );
    assert!(
        prepare_source.contains("PreparedCaveFeatureResource"),
        "prepare path must ingest cave prepared contributions"
    );
    assert!(
        prepare_source.contains("PreparedDetailFeatureResource"),
        "prepare path must ingest detail prepared contributions"
    );
    assert!(
        submit_source.contains("PreparedRenderFrameResource"),
        "submit path must consume the prepared render frame"
    );

    for forbidden in [
        "WorldOperationLog",
        "WorldChunkRuntimeMapResource",
        "dispatch_world_build_jobs_system",
        "integrate_completed_build_outputs_system",
    ] {
        assert!(
            !submit_source.contains(forbidden),
            "submit must not pull authoritative world/build runtime state directly (found '{forbidden}')"
        );
    }
}
