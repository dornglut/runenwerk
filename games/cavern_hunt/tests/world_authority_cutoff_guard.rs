use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn gameplay_runtime_no_longer_falls_back_to_legacy_geometry_collision_authority() {
    let projectiles = read("src/features/combat/runtime/projectiles.rs");
    let movement = read("src/features/combat/runtime/movement_fire.rs");
    let ai = read("src/features/ai/plugin.rs");

    for forbidden in [
        "constrained_move_legacy",
        "sweep_world_collision_legacy",
        "resource::<CavernGeometryGraph>()",
        "resource_mut::<CavernCollisionField>()",
    ] {
        assert!(
            !projectiles.contains(forbidden),
            "projectiles runtime must not contain legacy authority fallback '{forbidden}'"
        );
    }

    for forbidden in ["CavernGeometryGraph", "CavernCollisionField"] {
        assert!(
            !movement.contains(forbidden),
            "movement runtime must not depend on legacy world authority '{forbidden}'"
        );
        assert!(
            !ai.contains(forbidden),
            "ai runtime must not depend on legacy world authority '{forbidden}'"
        );
    }
}

#[test]
fn snapshot_wire_schema_is_world_checkpoint_driven() {
    let snapshot_types = read("src/domain/snapshot/types_and_bundles.rs");
    let net_driver = read("src/net/driver.rs");

    assert!(
        snapshot_types.contains("pub struct CavernWorldCheckpointV1"),
        "snapshot schema must include world checkpoint payload"
    );
    for required in [
        "chunk_headers",
        "chunk_contents",
        "op_windows",
        "residency_hints",
    ] {
        assert!(
            snapshot_types.contains(required),
            "world checkpoint schema must include '{required}'"
        );
    }
    for forbidden in [
        "CavernGeometrySnapshotV1",
        "geometry_revision",
        "geometry_edits",
    ] {
        assert!(
            !snapshot_types.contains(forbidden),
            "snapshot wire schema must not include legacy geometry payload field '{forbidden}'"
        );
    }

    assert!(
        net_driver.contains("capture_world_checkpoint(world, Some(connection_id))"),
        "connection-aware snapshot capture must include per-connection world checkpoint"
    );
}

#[test]
fn runtime_wiring_keeps_frame_builder_adapter_without_legacy_geometry_authority() {
    let plugin_wiring = read("src/app/runtime/plugin_wiring.rs");
    let setup_and_slots = read("src/app/runtime/setup_and_slots.rs");
    let world_frame_builder =
        read("src/features/render_sdf/runtime/world_frame_and_geometry/world_frame.rs");

    for required in [
        "init_resource::<CavernSdfWorldFrame>()",
        "render_sdf::build_sdf_world_frame_system",
    ] {
        assert!(
            plugin_wiring.contains(required),
            "runtime wiring must include sdf frame adapter '{required}' until world feature fully replaces it"
        );
    }

    assert!(
        !world_frame_builder.contains("resource::<CavernGeometryGraph>()"),
        "sdf frame adapter must not query legacy geometry graph authority"
    );

    assert!(
        !setup_and_slots.contains("render_sdf::setup_render_resources"),
        "client setup must not perform ad hoc legacy sdf frame bootstrap"
    );
}
