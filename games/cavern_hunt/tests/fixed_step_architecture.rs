use std::fs;
use std::path::PathBuf;

const FIXED_SIMULATION_FILES: &[&str] = &[
    "src/features/combat/plugin.rs",
    "src/features/ai/plugin.rs",
    "src/features/loot/plugin/run_state.rs",
];

const FORBIDDEN_FRAME_DELTA_PATTERNS: &[&str] = &[
    "resource::<Time>()",
    "resource::<engine::prelude::Time>()",
    ".delta_seconds",
];

const FIXED_WORLD_ORDERING_FILES: &[(&str, &[&str])] = &[
    (
        "src/features/combat/runtime/plugin_aim.rs",
        &[
            "in_set(CoreSet::Simulation)",
            "after(WorldRuntimeSet::BuildIntegrate)",
            "before(CoreSet::Replication)",
        ],
    ),
    (
        "src/features/ai/plugin.rs",
        &[
            "in_set(CoreSet::Simulation)",
            "after(WorldRuntimeSet::BuildIntegrate)",
            "before(CoreSet::Replication)",
        ],
    ),
    (
        "src/features/loot/plugin/mod.rs",
        &[
            "in_set(CoreSet::Simulation)",
            "after(WorldRuntimeSet::BuildIntegrate)",
            "before(CoreSet::Replication)",
        ],
    ),
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

#[test]
fn gameplay_fixed_step_systems_are_explicitly_ordered_relative_to_world_runtime() {
    for (path, required_patterns) in FIXED_WORLD_ORDERING_FILES {
        let source = read_source(path);
        for pattern in *required_patterns {
            assert!(
                source.contains(pattern),
                "{path} is missing required fixed-step world ordering contract `{pattern}`"
            );
        }
    }
}
