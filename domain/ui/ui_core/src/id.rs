//! File: domain/ui/ui_core/src/id.rs
//! Purpose: Stable widget identifiers.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetTypeId(pub u64);