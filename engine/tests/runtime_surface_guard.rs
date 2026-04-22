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

#[test]
fn runtime_surface_stays_free_of_legacy_runtime_imports() {
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
fn overlay_runtime_paths_route_through_ui_substrate_frame_builder() {
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
