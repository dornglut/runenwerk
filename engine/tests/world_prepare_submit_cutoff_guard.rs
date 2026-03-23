use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn render_submit_consumes_prepared_world_resources_only() {
    let submit_source = read("src/plugins/render/renderer/submit.rs");

    assert!(
        submit_source.contains("PreparedWorldFeatureResource"),
        "submit path must ingest world prepared contributions"
    );
    assert!(
        submit_source.contains("PreparedCaveFeatureResource"),
        "submit path must ingest cave prepared contributions"
    );
    assert!(
        submit_source.contains("PreparedDetailFeatureResource"),
        "submit path must ingest detail prepared contributions"
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
