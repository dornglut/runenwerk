//! Generic value and collection slot values supplied during formation.

use crate::identity::{UiCollectionSlotId, UiSelectionSlotId, UiValueSlotId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiValue {
    Text(String),
    Bool(bool),
    Number(f64),
}

impl UiValue {
    pub fn as_text(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiValueBinding {
    Static(UiValue),
    Slot(UiValueSlotId),
}

impl UiValueBinding {
    pub fn static_text(value: impl Into<String>) -> Self {
        Self::Static(UiValue::Text(value.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiCollectionItem {
    pub key: String,
    pub label: String,
    pub selected: bool,
    pub enabled: bool,
    #[serde(default)]
    pub values: BTreeMap<UiValueSlotId, UiValue>,
}

impl UiCollectionItem {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            selected: false,
            enabled: true,
            values: BTreeMap::new(),
        }
    }

    pub fn with_value(mut self, slot: UiValueSlotId, value: UiValue) -> Self {
        self.values.insert(slot, value);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCollectionBinding {
    pub slot: UiCollectionSlotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSelectionBinding {
    pub slot: UiSelectionSlotId,
}
