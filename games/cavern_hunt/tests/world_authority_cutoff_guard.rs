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
        "pub(crate) fn constrained_move(",
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
    for forbidden in ["CavernGeometrySnapshotV1", "geometry_revision", "geometry_edits"] {
        assert!(
            !snapshot_types.contains(forbidden),
            "snapshot wire schema must not include legacy geometry payload field '{forbidden}'"
        );
    }
    for required in [
        "pub struct CavernRunSnapshotV3",
        "pub struct CavernRunDeltaV3",
        "type Snapshot = CavernRunSnapshotV3",
        "type Delta = CavernRunDeltaV3",
        "unsupported cavern snapshot version: V1 (expected V3)",
        "unsupported cavern delta version: V1 (expected V3)",
    ] {
        assert!(
            snapshot_types.contains(required) || net_driver.contains(required),
            "hard-cut snapshot schema and net driver must include '{required}'"
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
    let render_plugin = read("src/features/render_sdf/plugin.rs");
    let geometry_projection = read("src/features/render_sdf/runtime/world_frame_and_geometry/geometry_projection.rs");

    for required in [
        "init_resource::<CavernSdfWorldFrame>()",
        "render_sdf::build_sdf_world_frame_system",
    ] {
        assert!(
            plugin_wiring.contains(required),
            "runtime wiring must include sdf frame adapter '{required}' until world feature fully replaces it"
        );
    }
    for forbidden in [
        "init_resource::<CavernGeometryGraph>()",
        "init_resource::<CavernCollisionField>()",
        "init_resource::<CavernGeometryRuntimeState>()",
    ] {
        assert!(
            !plugin_wiring.contains(forbidden),
            "runtime wiring must not initialize legacy runtime authority resource '{forbidden}'"
        );
    }

    assert!(
        !world_frame_builder.contains("resource::<CavernGeometryGraph>()"),
        "sdf frame adapter must not query legacy geometry graph authority"
    );
    for source in [&render_plugin, &geometry_projection] {
        for forbidden in ["CavernGeometryGraph", "geometry_primitives_from_graph("] {
            assert!(
                !source.contains(forbidden),
                "sdf runtime path must not retain legacy geometry graph authority symbol '{forbidden}'"
            );
        }
    }

    assert!(
        !setup_and_slots.contains("render_sdf::setup_render_resources"),
        "client setup must not perform ad hoc legacy sdf frame bootstrap"
    );
}

#[test]
fn world_edit_paths_use_central_ingress_without_direct_dirty_map_mutation() {
    let worldgen_init = read("src/features/worldgen/plugin/init.rs");
    let geometry_edits = read("src/features/worldgen/plugin/geometry_edits.rs");

    assert!(
        !worldgen_init.contains("resource::<CavernGeometryGraph>()"),
        "worldgen init should not query legacy geometry graph resource for runtime ingress/bootstrap"
    );

    for source in [&worldgen_init, &geometry_edits] {
        assert!(
            source.contains("submit_world_operation("),
            "runtime world edits must flow through world ingress submit API"
        );
        for forbidden in [
            "mark_dirty(",
            "WorldDirtyChunkMapResource",
            "resource_mut::<WorldDirtyChunkMapResource>()",
        ] {
            assert!(
                !source.contains(forbidden),
                "world edit runtime path must not bypass ingress via '{forbidden}'"
            );
        }
    }
    for forbidden in [
        "resource_mut::<CavernGeometryGraph>()",
        "WorldOperation::StructureRemove",
        "WorldOperation::MaterialFieldEdit",
        "WorldOperation::DensityFieldDeform",
    ] {
        assert!(
            !geometry_edits.contains(forbidden),
            "runtime world edits must not map legacy graph primitive mutation paths via '{forbidden}'"
        );
    }
}

#[test]
fn world_checkpoint_capture_reads_world_replication_state() {
    let capture = read("src/domain/snapshot/capture_and_delta.rs");

    assert!(
        capture.contains("resource::<ReplicationStateResource>()"),
        "world checkpoint capture should consume world-owned replication state resource"
    );
    for forbidden in [
        "resource::<WorldChunkRuntimeMapResource>()",
        "resource::<WorldSdfChunkStoreResource>()",
        "resource::<WorldOperationLog>()",
    ] {
        assert!(
            !capture.contains(forbidden),
            "world checkpoint capture should not rebuild deltas ad hoc via '{forbidden}'"
        );
    }
}

#[test]
fn materials_gi_scaffold_reads_world_authority_revision_only() {
    let materials = read("src/features/materials/plugin.rs");

    assert!(
        materials.contains("resource::<WorldAuthorityState>()"),
        "materials gi scaffold should read world authority revision"
    );
    assert!(
        !materials.contains("resource::<CavernGeometryGraph>()"),
        "materials gi scaffold must not fall back to legacy geometry graph revision"
    );
}

#[test]
fn legacy_collision_field_is_not_registered_as_runtime_resource_component() {
    let resource_markers = read("src/ecs_resource_components.rs");
    assert!(
        !resource_markers.contains("collision_field::CavernCollisionField"),
        "legacy collision field must not be registered as a runtime resource component"
    );
}

#[test]
fn snapshot_restore_does_not_reintroduce_legacy_collision_field_authority() {
    let restore = read("src/domain/snapshot/restore.rs");

    for forbidden in [
        "resource_mut::<CavernCollisionField>()",
        "insert_resource(CavernCollisionField::from_graph",
        "field.sync_revision(&geometry)",
    ] {
        assert!(
            !restore.contains(forbidden),
            "snapshot restore must not reintroduce legacy collision authority via '{forbidden}'"
        );
    }
}

#[test]
fn snapshot_restore_does_not_reintroduce_legacy_geometry_graph_resources() {
    let restore = read("src/domain/snapshot/restore.rs");
    for forbidden in [
        "insert_resource(CavernGeometryGraph",
        "insert_resource(CavernGeometryRuntimeState",
    ] {
        assert!(
            !restore.contains(forbidden),
            "snapshot restore must not reintroduce legacy geometry runtime resources via '{forbidden}'"
        );
    }
}

#[test]
fn legacy_collision_field_module_is_deleted_from_world_domain_surface() {
    let world_mod = read("src/domain/world/mod.rs");
    assert!(
        !world_mod.contains("collision_field"),
        "world domain surface must no longer expose legacy collision_field module"
    );
    assert!(
        !Path::new("src/domain/world/collision_field").exists(),
        "legacy collision_field module directory should be removed in hard-cut mode"
    );
}
