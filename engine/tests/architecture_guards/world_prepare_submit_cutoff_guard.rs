use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn retired_cave_render_projection_does_not_return() {
    let feature_source = read("src/plugins/render/features/mod.rs");
    let contribution_source = read("src/plugins/render/frame/contributions.rs");
    let prepare_source = read("src/plugins/render/runtime/frame_prepare.rs");
    let world_prepare_source = read("src/plugins/world/prepare/contributions.rs");

    let predecessor_sources = [
        ("render/features/mod.rs", feature_source.as_str()),
        (
            "render/frame/contributions.rs",
            contribution_source.as_str(),
        ),
        ("render/runtime/frame_prepare.rs", prepare_source.as_str()),
        (
            "world/prepare/contributions.rs",
            world_prepare_source.as_str(),
        ),
    ];

    for retired in [
        "CaveRenderVisibilityResource",
        "PreparedCaveFeatureResource",
        "PreparedCaveFeatureContribution",
        "PreparedFeaturePayload::Caves",
        "insert_caves",
    ] {
        for (path, source) in predecessor_sources {
            assert!(
                !source.contains(retired),
                "retired Cave render projection must not return (found '{retired}' in {path})"
            );
        }
    }
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
