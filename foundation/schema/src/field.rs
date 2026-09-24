use alloc::string::String;
use alloc::vec::Vec;

use crate::constraint::SchemaConstraint;
use crate::metadata::{SchemaMetadata, SchemaMetadataEntry, SchemaMetadataError};
use crate::shape::SchemaShape;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaField {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    shape: SchemaShape,
    constraints: Vec<SchemaConstraint>,
    metadata: SchemaMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaFieldError {
    EmptyName,
    Metadata(SchemaMetadataError),
}

impl SchemaField {
    pub fn new(name: impl Into<String>, shape: SchemaShape) -> Result<Self, SchemaFieldError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SchemaFieldError::EmptyName);
        }

        Ok(Self {
            name,
            display_name: None,
            description: None,
            shape,
            constraints: Vec::new(),
            metadata: SchemaMetadata::new(),
        })
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_constraint(mut self, constraint: SchemaConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_metadata_entry(
        mut self,
        entry: SchemaMetadataEntry,
    ) -> Result<Self, SchemaFieldError> {
        self.metadata
            .push(entry)
            .map_err(SchemaFieldError::Metadata)?;
        Ok(self)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn shape(&self) -> &SchemaShape {
        &self.shape
    }

    pub fn constraints(&self) -> &[SchemaConstraint] {
        &self.constraints
    }

    pub fn metadata(&self) -> &SchemaMetadata {
        &self.metadata
    }
}
