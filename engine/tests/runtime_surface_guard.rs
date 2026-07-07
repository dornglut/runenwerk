use std::fs;
use std::path::{Path, PathBuf};

const GUARDED_PATHS: &[&str] = &[
    "src/app",
    "src/state.rs",
    "src/runtime",
    "src/plugins/fixed_step.rs",
    "src/plugins/time/mod.rs",
    "src/plugins/input/mod.rs",
    "src/plugins/grid/mod.rs",
    "src/plugins/debug_metrics/mod.rs",
    "examples/runtime_minimal",
    "examples/window_input_demo",
    "tests/runtime_app.rs",
    "tests/network_plugins.rs",
];

const BANNED_PATTERNS: &[&str] = &[
    "EngineData",
    "EnginePlugin",
    "engine::runtime",
    "engine::platform",
    "engine::legacy",
];

const SUBSTRATE_OVERLAY_PATHS: &[&str] = &[
    "src/plugins/debug_metrics/mod.rs",
    "src/plugins/scene/runtime/overlay_ui.rs",
];

const RETIRED_RENDER_UI_COLLECTION_PATTERNS: &[(&str, &str)] = &[
    (
        "src/plugins/render/plugin.rs",
        "collect_runtime_ui_frame_submissions_system",
    ),
    (
        "src/plugins/render/runtime/mod.rs",
        "collect_runtime_ui_frame_submissions_system",
    ),
    ("src/plugins/render/runtime/mod.rs", "ui_submission"),
];

#[test]
fn runtime_surface_guard_stays_free_of_legacy_runtime_imports() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();

    for relative_path in GUARDED_PATHS {
        let path = manifest_dir.join(relative_path);
        collect_offenders(&path, &manifest_dir, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "runtime surface guard found legacy runtime usage in current runtime surfaces:\n{}",
        offenders.join("\n")
    );
}

fn collect_offenders(path: &Path, root: &Path, offenders: &mut Vec<String>) {
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        entries.sort();
        for entry in entries {
            collect_offenders(&entry, root, offenders);
        }
        return;
    }

    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return;
    }

    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    for pattern in BANNED_PATTERNS {
        if contents.contains(pattern) {
            let relative = path.strip_prefix(root).unwrap_or(path);
            offenders.push(format!("{} contains `{pattern}`", relative.display()));
        }
    }
}

#[test]
fn runtime_surface_guard_overlay_runtime_paths_route_through_ui_substrate_frame_builder() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut missing_substrate_calls = Vec::new();
    let mut banned_manual_primitives = Vec::new();

    for relative_path in SUBSTRATE_OVERLAY_PATHS {
        let path = manifest_dir.join(relative_path);
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

        if !contents.contains("build_ui_frame(") {
            missing_substrate_calls.push(relative_path.to_string());
        }
        if contents.contains("GlyphRunPrimitive::new(")
            || contents.contains("RectPrimitive::new(")
            || contents.contains("estimate_glyph_run(")
        {
            banned_manual_primitives.push(relative_path.to_string());
        }
    }

    assert!(
        missing_substrate_calls.is_empty(),
        "expected overlay runtime paths to call ui_runtime::build_ui_frame:\n{}",
        missing_substrate_calls.join("\n")
    );
    assert!(
        banned_manual_primitives.is_empty(),
        "expected overlay runtime paths to avoid ad-hoc primitive assembly:\n{}",
        banned_manual_primitives.join("\n")
    );
}

#[test]
fn runtime_surface_guard_render_plugin_no_longer_owns_scene_or_debug_ui_submission_collection() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let legacy_collector_path = manifest_dir.join("src/plugins/render/runtime/ui_submission.rs");
    assert!(
        !legacy_collector_path.exists(),
        "legacy RenderPlugin UI collector should stay retired: {}",
        legacy_collector_path.display()
    );

    let mut offenders = Vec::new();
    for (relative_path, pattern) in RETIRED_RENDER_UI_COLLECTION_PATTERNS {
        let path = manifest_dir.join(relative_path);
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if contents.contains(pattern) {
            offenders.push(format!("{relative_path} contains `{pattern}`"));
        }
    }

    assert!(
        offenders.is_empty(),
        "RenderPlugin must not reintroduce scene/debug UI producer collection:\n{}",
        offenders.join("\n")
    );
}
