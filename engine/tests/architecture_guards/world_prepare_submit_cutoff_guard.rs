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
fn retired_procedural_world_render_projection_does_not_return() {
    // These are predecessor implementation names, not reserved procedural
    // world concepts or restrictions on future consumed render contracts.
    for path in [
        "src/plugins/render/features/mod.rs",
        "src/plugins/render/frame/contributions.rs",
        "src/plugins/render/inspect/prepared_frame.rs",
        "src/plugins/render/plugin.rs",
        "src/plugins/render/renderer/prepare.rs",
        "src/plugins/render/runtime/frame_prepare.rs",
        "src/plugins/world/prepare/contributions.rs",
    ] {
        let source = read(path);
        for retired in [
            "PROCEDURAL_WORLD_RENDER_FEATURE_LABEL",
            "PROCEDURAL_WORLD_RENDER_FEATURE_ID",
            "PreparedProceduralWorldFeatureResource",
            "PreparedProceduralWorldFeatureContribution",
            "PreparedProceduralOverlayContribution",
            "PreparedFeaturePayload::ProceduralWorld",
            "insert_procedural_world",
        ] {
            assert!(
                !source.contains(retired),
                "retired ProceduralWorld render projection must not return (found '{retired}' in {path})"
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

#[test]
fn prepublication_wind_fields_complete_source_census() {
    // Temporary owner-gate proof. This test is removed after its exact-head CI
    // run establishes consumer closure, before the production-retirement commit.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("engine must be inside the repository");
    let former_owners = [
        "engine/src/plugins/render/features/mod.rs",
        "engine/src/plugins/render/plugin.rs",
        "engine/src/plugins/world/prepare/contributions.rs",
        "engine/src/plugins/render/frame/contributions.rs",
        "engine/src/plugins/render/runtime/frame_prepare.rs",
        "engine/src/plugins/render/renderer/prepare.rs",
        "engine/src/plugins/render/inspect/prepared_frame.rs",
    ];
    let markers = [
        "WIND_FIELDS_RENDER_FEATURE_ID",
        "WIND_FIELDS_RENDER_FEATURE_LABEL",
        "PreparedWindField",
        "PreparedFeaturePayload::WindFields",
        "insert_wind_fields",
        "wind.fields",
        "wind_fields",
        "WindFields",
    ];
    let mut pending = vec![root.to_path_buf()];
    let mut total_files = 0usize;
    let mut text_files = 0usize;
    let mut owner_files = 0usize;
    let mut unowned_references = Vec::<String>::new();

    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
            let entry = entry.expect("repository directory entry must be readable");
            let path = entry.path();
            let kind = entry
                .file_type()
                .expect("repository file type must be readable");
            if kind.is_dir() {
                let name = entry.file_name();
                if [
                    ".git",
                    "target",
                    "node_modules",
                    "dist",
                    ".astro",
                    ".cache",
                    ".venv",
                ]
                .iter()
                .any(|excluded| name == *excluded)
                {
                    continue;
                }
                pending.push(path);
                continue;
            }
            assert!(
                kind.is_file(),
                "unexpected repository entry: {}",
                path.display()
            );
            total_files += 1;
            let bytes = fs::read(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            let Ok(content) = std::str::from_utf8(&bytes) else {
                continue;
            };
            text_files += 1;
            let relative = path
                .strip_prefix(root)
                .expect("repository file path must be underneath root")
                .to_string_lossy()
                .replace('\\', "/");
            if relative == "engine/tests/architecture_guards/world_prepare_submit_cutoff_guard.rs" {
                continue;
            }
            if former_owners.contains(&relative.as_str()) {
                owner_files += 1;
                continue;
            }

            for marker in markers {
                if content.contains(marker) {
                    unowned_references.push(format!("{relative}: {marker}"));
                }
            }
            if content.contains("RenderFeatureId") || content.contains("render_feature_id") {
                let compact = content.split_whitespace().collect::<String>();
                for marker in [
                    "RenderFeatureId::try_from_raw(10)",
                    "RenderFeatureId::from_raw(10)",
                    "render_feature_id(10)",
                ] {
                    if compact.contains(marker) {
                        unowned_references.push(format!("{relative}: {marker}"));
                    }
                }
            }
        }
    }

    // GitHub's immutable accepted-source tree enumerates 2,486 blob files;
    // exact-head hosted CI must check out at least that complete source set.
    assert!(
        total_files >= 2_486,
        "incomplete repository traversal: read only {total_files} files"
    );
    assert!(
        text_files >= 2_460,
        "incomplete maintained text traversal: read only {text_files} files"
    );
    assert_eq!(
        owner_files,
        former_owners.len(),
        "missing owning source files"
    );
    assert!(
        unowned_references.is_empty(),
        "WindFields compatibility references outside seven known owners: {unowned_references:#?}"
    );
}
