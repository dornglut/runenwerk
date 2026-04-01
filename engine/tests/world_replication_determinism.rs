use engine::SimulationTick;
use engine::plugins::world::chunks::dirty::WorldDirtyChunkMapResource;
use engine::plugins::world::chunks::partition::WorldPartitionConfig;
use engine::plugins::world::edits::invalidation::invalidate_dirty_chunks_from_op_log;
use engine::plugins::world::edits::log::WorldOperationLog;
use engine::plugins::world::edits::operation::{
    WorldBrushShape, WorldOperation, WorldOperationRecord, quantize_aabb, quantize_position,
};
use engine::plugins::world::edits::replay::{WorldReplayWindow, operations_for_replay_window};
use engine::plugins::world::ids::{PlanetId, WorldOpId, WorldRevision};

fn build_test_log() -> WorldOperationLog {
    let mut log = WorldOperationLog::default();
    let operations = [
        WorldOperation::CsgSubtract {
            brush: WorldBrushShape::Sphere {
                center_q: quantize_position([2.0, 0.0, -1.0], 1024),
                radius_q: 1536,
            },
        },
        WorldOperation::Smooth {
            bounds_q: quantize_aabb([-6.0, -2.0, -6.0], [6.0, 2.0, 6.0], 1024),
            kernel_radius_q: 512,
            strength_q: 192,
        },
        WorldOperation::MaterialFieldEdit {
            bounds_q: quantize_aabb([-4.0, -1.0, -4.0], [4.0, 1.0, 4.0], 1024),
            channel_mask: 0b0011,
            payload: vec![1, 2, 3, 4],
        },
    ];

    for operation in operations {
        let bounds_q = match &operation {
            WorldOperation::CsgSubtract { .. } => {
                quantize_aabb([-3.0, -3.0, -3.0], [3.0, 3.0, 3.0], 1024)
            }
            WorldOperation::Smooth { bounds_q, .. } => *bounds_q,
            WorldOperation::MaterialFieldEdit { bounds_q, .. } => *bounds_q,
            _ => quantize_aabb([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0], 1024),
        };
        let _ = log.append(WorldOperationRecord {
            op_id: WorldOpId(0),
            base_world_revision: WorldRevision(1),
            operation,
            affected_bounds_q: bounds_q,
            deterministic_seed: 1337,
            server_tick: SimulationTick(12),
            author_connection_id: Some(7),
        });
    }
    log
}

#[test]
fn op_log_replay_and_invalidation_are_deterministic() {
    let log_a = build_test_log();
    let log_b = build_test_log();

    let replay_window = WorldReplayWindow {
        applied_op_exclusive: WorldOpId(0),
        target_op_inclusive: WorldOpId(3),
    };

    let replay_a = operations_for_replay_window(&log_a, replay_window);
    let replay_b = operations_for_replay_window(&log_b, replay_window);
    assert_eq!(replay_a, replay_b, "replay output must be deterministic");

    let partition = WorldPartitionConfig::default();
    let mut dirty_a = WorldDirtyChunkMapResource::default();
    let mut dirty_b = WorldDirtyChunkMapResource::default();
    invalidate_dirty_chunks_from_op_log(&mut dirty_a, &partition, &log_a, PlanetId(0), 1024);
    invalidate_dirty_chunks_from_op_log(&mut dirty_b, &partition, &log_b, PlanetId(0), 1024);
    assert_eq!(
        dirty_a.by_chunk, dirty_b.by_chunk,
        "dirty invalidation set must be deterministic for identical op logs"
    );
}
