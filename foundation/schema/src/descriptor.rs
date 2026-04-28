use alloc::string::String;
use alloc::vec::Vec;

use crate::compatibility::SchemaCompatibility;
use crate::id::SchemaId;
use crate::issue::SchemaIssue;
use crate::metadata::{SchemaMetadata, SchemaMetadataEntry, SchemaMetadataError};
use crate::shape::SchemaShape;
use crate::version::SchemaVersion;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaDescriptor {
    id: SchemaId,
    version: SchemaVersion,
    display_name: Option<String>,
    description: Option<String>,
    root_shape: SchemaShape,
    metadata: SchemaMetadata,
    compatibility: SchemaCompatibility,
    issues: Vec<SchemaIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaDescriptorError {
    Metadata(SchemaMetadataError),
}

impl SchemaDescriptor {
    pub fn new(id: SchemaId, version: SchemaVersion, root_shape: SchemaShape) -> Self {
        Self {
            id,
            version,
            display_name: None,
            description: None,
            root_shape,
            metadata: SchemaMetadata::new(),
            compatibility: SchemaCompatibility::Unknown,
            issues: Vec::new(),
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_metadata_entry(
        mut self,
        entry: SchemaMetadataEntry,
    ) -> Result<Self, SchemaDescriptorError> {
        self.metadata
            .push(entry)
            .map_err(SchemaDescriptorError::Metadata)?;
        Ok(self)
    }

    pub fn with_compatibility(mut self, compatibility: SchemaCompatibility) -> Self {
        self.compatibility = compatibility;
        self
    }

    pub fn with_issue(mut self, issue: SchemaIssue) -> Self {
        self.issues.push(issue);
        self
    }

    pub fn id(&self) -> &SchemaId {
        &self.id
    }

    pub fn version(&self) -> SchemaVersion {
        self.version
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn root_shape(&self) -> &SchemaShape {
        &self.root_shape
    }

    pub fn metadata(&self) -> &SchemaMetadata {
        &self.metadata
    }

    pub fn compatibility(&self) -> SchemaCompatibility {
        self.compatibility
    }

    pub fn issues(&self) -> &[SchemaIssue] {
        &self.issues
    }

    pub fn highest_issue(&self) -> Option<&SchemaIssue> {
        self.issues.iter().max_by_key(|issue| issue.severity_rank())
    }
}
