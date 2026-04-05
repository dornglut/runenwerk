use crate::{OperationId, OperationLog, OperationRecord};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct ReplayWindow {
    pub applied_op_exclusive: OperationId,
    pub target_op_inclusive: OperationId,
}

pub fn operations_for_replay_window(
    log: &OperationLog,
    window: ReplayWindow,
) -> Vec<OperationRecord> {
    log.operations
        .iter()
        .filter(|record| {
            record.op_id.0 > window.applied_op_exclusive.0
                && record.op_id.0 <= window.target_op_inclusive.0
        })
        .cloned()
        .collect()
}
