use crate::{OperationId, OperationRecord, WorldRevision};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct OperationLog {
    pub operations: Vec<OperationRecord>,
    pub by_id: BTreeMap<OperationId, usize>,
    pub next_op_id: u64,
}

impl OperationLog {
    pub fn append(&mut self, mut record: OperationRecord) -> OperationId {
        if self.next_op_id == 0 {
            self.next_op_id = 1;
        }
        record.op_id = OperationId(self.next_op_id);
        self.next_op_id = self.next_op_id.saturating_add(1);
        let op_id = record.op_id;
        self.by_id.insert(op_id, self.operations.len());
        self.operations.push(record);
        op_id
    }

    pub fn latest_world_revision_hint(&self) -> WorldRevision {
        self.operations
            .last()
            .map(|op| WorldRevision(op.base_world_revision.0.saturating_add(1)))
            .unwrap_or_default()
    }
}
