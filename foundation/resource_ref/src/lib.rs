//! Portable external resource reference vocabulary.
//!
//! This crate owns stable references to resources that are resolved by higher
//! layers. It does not know about asset catalogs, renderer resources, editor
//! state, paths, IO, or runtime handles.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceRef {
    pub kind: ResourceRefKind,
    pub stable_id: ResourceStableId,
    pub revision: Option<ResourceRevisionRef>,
    pub artifact: Option<ResourceArtifactRef>,
}

impl ResourceRef {
    pub fn new(
        kind: impl Into<ResourceRefKind>,
        stable_id: impl Into<ResourceStableId>,
    ) -> Result<Self, ResourceRefError> {
        let reference = Self {
            kind: kind.into(),
            stable_id: stable_id.into(),
            revision: None,
            artifact: None,
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn with_revision(mut self, revision: impl Into<ResourceRevisionRef>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    pub fn with_artifact(mut self, artifact: impl Into<ResourceArtifactRef>) -> Self {
        self.artifact = Some(artifact.into());
        self
    }

    pub fn validate(&self) -> Result<(), ResourceRefError> {
        if self.kind.0.trim().is_empty() {
            return Err(ResourceRefError::EmptyKind);
        }
        if self.stable_id.0.trim().is_empty() {
            return Err(ResourceRefError::EmptyStableId);
        }
        if self
            .revision
            .as_ref()
            .is_some_and(|revision| revision.0.trim().is_empty())
        {
            return Err(ResourceRefError::EmptyRevision);
        }
        if self
            .artifact
            .as_ref()
            .is_some_and(|artifact| artifact.0.trim().is_empty())
        {
            return Err(ResourceRefError::EmptyArtifact);
        }
        Ok(())
    }

    pub fn canonical_component(&self) -> String {
        let mut encoder = ResourceRefCanonicalEncoder::default();
        encoder.field("kind", self.kind.as_str());
        encoder.field("stable_id", self.stable_id.as_str());
        encoder.optional_field(
            "revision",
            self.revision.as_ref().map(ResourceRevisionRef::as_str),
        );
        encoder.optional_field(
            "artifact",
            self.artifact.as_ref().map(ResourceArtifactRef::as_str),
        );
        encoder.finish_hex()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceRefKind(String);

impl ResourceRefKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ResourceRefKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ResourceRefKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceStableId(String);

impl ResourceStableId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ResourceStableId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ResourceStableId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceRevisionRef(String);

impl ResourceRevisionRef {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ResourceRevisionRef {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ResourceRevisionRef {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceArtifactRef(String);

impl ResourceArtifactRef {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ResourceArtifactRef {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ResourceArtifactRef {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRefError {
    EmptyKind,
    EmptyStableId,
    EmptyRevision,
    EmptyArtifact,
}

impl fmt::Display for ResourceRefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKind => formatter.write_str("resource reference kind must not be empty"),
            Self::EmptyStableId => {
                formatter.write_str("resource reference stable id must not be empty")
            }
            Self::EmptyRevision => {
                formatter.write_str("resource reference revision must not be empty")
            }
            Self::EmptyArtifact => {
                formatter.write_str("resource reference artifact must not be empty")
            }
        }
    }
}

impl Error for ResourceRefError {}

#[derive(Default)]
struct ResourceRefCanonicalEncoder {
    bytes: Vec<u8>,
}

impl ResourceRefCanonicalEncoder {
    fn field(&mut self, label: &str, value: &str) {
        self.bytes.extend_from_slice(label.as_bytes());
        self.bytes.push(b'=');
        self.bytes
            .extend_from_slice(value.as_bytes().len().to_string().as_bytes());
        self.bytes.push(b':');
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(b'\n');
    }

    fn optional_field(&mut self, label: &str, value: Option<&str>) {
        match value {
            Some(value) => self.field(label, value),
            None => self.field(label, "<none>"),
        }
    }

    fn finish_hex(self) -> String {
        self.bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_refs_reject_empty_identity() {
        assert_eq!(
            ResourceRef::new("", "texture.rock"),
            Err(ResourceRefError::EmptyKind)
        );
        assert_eq!(
            ResourceRef::new("asset.catalog", ""),
            Err(ResourceRefError::EmptyStableId)
        );
    }

    #[test]
    fn canonical_component_is_length_prefixed() {
        let left = ResourceRef::new("asset.catalog", "a:b")
            .expect("reference")
            .with_artifact("c");
        let right = ResourceRef::new("asset.catalog", "a")
            .expect("reference")
            .with_artifact("b:c");

        assert_ne!(left.canonical_component(), right.canonical_component());
    }
}
