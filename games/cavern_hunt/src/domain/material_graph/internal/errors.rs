// Owner: Cavern Hunt Domain - Material Graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCompileError {
    pub node: Option<String>,
    pub message: String,
}

impl MaterialCompileError {
    fn graph(message: impl Into<String>) -> Self {
        Self {
            node: None,
            message: message.into(),
        }
    }

    fn node(node: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node: Some(node.into()),
            message: message.into(),
        }
    }
}
