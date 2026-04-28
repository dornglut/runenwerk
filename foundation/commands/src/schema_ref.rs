use schema::{SchemaId, SchemaVersion};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandSchemaRef {
    schema_id: SchemaId,
    schema_version: SchemaVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandResultSchemaRef {
    schema_id: SchemaId,
    schema_version: SchemaVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandSchemaRefError {
    InvalidVersion,
}

impl CommandSchemaRef {
    pub fn new(schema_id: SchemaId, schema_version: SchemaVersion) -> Self {
        Self {
            schema_id,
            schema_version,
        }
    }

    pub fn schema_id(&self) -> &SchemaId {
        &self.schema_id
    }

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }
}

impl CommandResultSchemaRef {
    pub fn new(schema_id: SchemaId, schema_version: SchemaVersion) -> Self {
        Self {
            schema_id,
            schema_version,
        }
    }

    pub fn schema_id(&self) -> &SchemaId {
        &self.schema_id
    }

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }
}
