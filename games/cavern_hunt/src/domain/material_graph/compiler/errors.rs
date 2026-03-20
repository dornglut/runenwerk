use super::*;

// Owner: Cavern Hunt Domain - Material Graph
#[derive(Debug, Clone, PartialEq, Eq, ecs::Resource)]
pub struct MaterialCompileError {
    pub node: Option<String>,
    pub message: String,
}

impl MaterialCompileError {
    pub(super) fn graph(message: impl Into<String>) -> Self {
        Self {
            node: None,
            message: message.into(),
        }
    }

    pub(super) fn node(node: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node: Some(node.into()),
            message: message.into(),
        }
    }
}
