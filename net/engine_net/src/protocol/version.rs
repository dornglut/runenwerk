use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub protocol_version: u32,
    pub game_content_version: u32,
    pub schema_version: u32,
}

impl ProtocolVersion {
    pub const fn new(
        protocol_version: u32,
        game_content_version: u32,
        schema_version: u32,
    ) -> Self {
        Self {
            protocol_version,
            game_content_version,
            schema_version,
        }
    }

    pub const fn is_compatible_with(self, other: Self) -> bool {
        self.protocol_version == other.protocol_version
            && self.game_content_version == other.game_content_version
            && self.schema_version == other.schema_version
    }
}
