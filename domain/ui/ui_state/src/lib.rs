//! File: domain/ui/ui_state/src/lib.rs
//! Crate: ui_state

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use ui_schema::UiSchemaValue;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStateModel {
    pub transient: BTreeMap<String, UiSchemaValue>,
    pub preview: BTreeMap<String, UiSchemaValue>,
    pub committed: BTreeMap<String, UiSchemaValue>,
    pub focus: BTreeSet<String>,
    pub hover: BTreeSet<String>,
    pub drag: BTreeMap<String, UiSchemaValue>,
    pub animation: BTreeMap<String, UiSchemaValue>,
    pub host_fed: BTreeMap<String, UiSchemaValue>,
    pub package_owned: BTreeMap<String, UiSchemaValue>,
}

impl UiStateModel {
    pub fn preview_value(&mut self, key: impl Into<String>, value: UiSchemaValue) {
        self.preview.insert(key.into(), value);
    }

    pub fn commit_preview(&mut self, key: &str) -> Option<UiSchemaValue> {
        let value = self.preview.remove(key)?;
        self.committed.insert(key.to_owned(), value.clone());
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_contract_separates_preview_and_committed_state() {
        let mut state = UiStateModel::default();
        state.preview_value("opacity", UiSchemaValue::number(0.5));

        assert!(state.committed.get("opacity").is_none());
        assert_eq!(
            state.commit_preview("opacity"),
            Some(UiSchemaValue::number(0.5))
        );
        assert_eq!(
            state.committed.get("opacity"),
            Some(&UiSchemaValue::number(0.5))
        );
    }
}
