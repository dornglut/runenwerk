//! File: domain/ui/ui_schema/src/value.rs
//! Crate: ui_schema

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiSchemaValue {
    Null,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(String),
    List(Vec<UiSchemaValue>),
    Object(BTreeMap<String, UiSchemaValue>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSchemaValueKind {
    Null,
    Bool,
    Integer,
    Number,
    String,
    List,
    Object,
}

impl UiSchemaValue {
    pub const fn null() -> Self {
        Self::Null
    }

    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub const fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub const fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn list(values: impl IntoIterator<Item = UiSchemaValue>) -> Self {
        Self::List(values.into_iter().collect())
    }

    pub fn object<const N: usize>(entries: [(&str, UiSchemaValue); N]) -> Self {
        Self::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }

    pub fn kind(&self) -> UiSchemaValueKind {
        match self {
            Self::Null => UiSchemaValueKind::Null,
            Self::Bool(_) => UiSchemaValueKind::Bool,
            Self::Integer(_) => UiSchemaValueKind::Integer,
            Self::Number(_) => UiSchemaValueKind::Number,
            Self::String(_) => UiSchemaValueKind::String,
            Self::List(_) => UiSchemaValueKind::List,
            Self::Object(_) => UiSchemaValueKind::Object,
        }
    }

    pub fn get(&self, key: &str) -> Option<&UiSchemaValue> {
        match self {
            Self::Object(values) => values.get(key),
            _ => None,
        }
    }

    pub fn get_path(&self, path: &[&str]) -> Option<&UiSchemaValue> {
        let mut current = self;
        for segment in path {
            current = current.get(segment)?;
        }
        Some(current)
    }
}
