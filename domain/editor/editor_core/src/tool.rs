//! File: domain/editor/editor_core/src/tool.rs
//! Purpose: Tool id and tool descriptor contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToolId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolDescriptor {
    pub id: ToolId,
    pub stable_name: &'static str,
    pub display_name: String,
}

impl ToolDescriptor {
    pub fn new(id: ToolId, stable_name: &'static str, display_name: impl Into<String>) -> Self {
        Self {
            id,
            stable_name,
            display_name: display_name.into(),
        }
    }
}
