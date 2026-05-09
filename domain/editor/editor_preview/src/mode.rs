use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewMode {
    Preview,
    Simulate,
    Play,
}

impl PreviewMode {
    pub const fn can_mutate_authored_documents(self) -> bool {
        false
    }
}
