use std::fs;
use std::path::PathBuf;

const FIXED_SIMULATION_FILES: &[&str] = &[
    "src/plugins/combat.rs",
    "src/plugins/ai.rs",
    "src/plugins/loot.rs",
];

const FORBIDDEN_FRAME_DELTA_PATTERNS: &[&str] = &[
    "resource::<Time>()",
    "resource::<engine::prelude::Time>()",
    ".delta_seconds",
];

fn read_source(path: &str) -> String {
    let mut absolute = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    absolute.push(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|error| panic!("failed to read {}: {error:#}", absolute.display()))
}

#[test]
fn fixed_simulation_modules_do_not_use_frame_delta_time() {
    for path in FIXED_SIMULATION_FILES {
        let source = read_source(path);
        for pattern in FORBIDDEN_FRAME_DELTA_PATTERNS {
            assert!(
                !source.contains(pattern),
                "{path} contains forbidden frame-delta pattern `{pattern}`; fixed simulation must use FixedTimeConfig-derived dt"
            );
        }
    }
}

#[test]
fn fixed_simulation_modules_use_shared_fixed_step_helper() {
    for path in FIXED_SIMULATION_FILES {
        let source = read_source(path);
        assert!(
            source.contains("fixed_step_seconds("),
            "{path} should use shared fixed_step_seconds helper for deterministic fixed-step timing"
        );
    }
}
