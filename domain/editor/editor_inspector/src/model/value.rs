//! File: domain/editor/editor_inspector/src/model/value.rs
//! Purpose: Inspector-facing value model.

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Enum {
        current: String,
        options: Vec<String>,
    },
    ReadOnlyText(String),
    Group,
    Unsupported {
        type_name: String,
    },
}
