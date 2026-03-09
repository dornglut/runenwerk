use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SnapshotCursor(pub u64);
