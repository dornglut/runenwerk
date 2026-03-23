use super::super::ids::WorldOpId;
use super::log::WorldOperationLog;
use super::operation::WorldOperationRecord;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct WorldReplayWindow {
    pub applied_op_exclusive: WorldOpId,
    pub target_op_inclusive: WorldOpId,
}

pub fn operations_for_replay_window(
    log: &WorldOperationLog,
    window: WorldReplayWindow,
) -> Vec<WorldOperationRecord> {
    log.operations
        .iter()
        .filter(|record| {
            record.op_id.0 > window.applied_op_exclusive.0
                && record.op_id.0 <= window.target_op_inclusive.0
        })
        .cloned()
        .collect()
}
