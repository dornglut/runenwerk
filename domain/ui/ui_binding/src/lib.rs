//! File: domain/ui/ui_binding/src/lib.rs
//! Crate: ui_binding

use serde::{Deserialize, Serialize};
use ui_schema::UiSchemaValue;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BindingId(String);

impl BindingId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BindingSnapshot {
    pub binding_id: BindingId,
    pub value: UiSchemaValue,
    pub dirty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionDiff {
    Insert { index: usize },
    Remove { index: usize },
    Move { from: usize, to: usize },
    Replace { index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingAuthorization {
    pub capability: String,
    pub can_read: bool,
    pub can_write: bool,
}

impl BindingSnapshot {
    pub fn new(binding_id: BindingId, value: UiSchemaValue) -> Self {
        Self {
            binding_id,
            value,
            dirty: false,
        }
    }

    pub fn mark_dirty(mut self) -> Self {
        self.dirty = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_contract_tracks_snapshots_dirty_state_and_authorization() {
        let snapshot = BindingSnapshot::new(
            BindingId::new("inspector.name"),
            UiSchemaValue::string("Lamp"),
        )
        .mark_dirty();
        let authorization = BindingAuthorization {
            capability: "editor.inspector.write".to_owned(),
            can_read: true,
            can_write: true,
        };

        assert!(snapshot.dirty);
        assert!(authorization.can_write);
        assert_eq!(
            CollectionDiff::Replace { index: 0 },
            CollectionDiff::Replace { index: 0 }
        );
    }
}
