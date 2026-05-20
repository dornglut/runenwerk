use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForeignMeshMaterialRegionKeyError {
    Empty,
    NotNormalized,
    TransientRuntimeIdentity,
}

impl ForeignMeshMaterialRegionKeyError {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            Self::Empty => "foreign mesh material region key must not be empty",
            Self::NotNormalized => "foreign mesh material region key must already be normalized",
            Self::TransientRuntimeIdentity => {
                "foreign mesh material region key must not use transient runtime identity"
            }
        }
    }
}

impl fmt::Display for ForeignMeshMaterialRegionKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_static_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ForeignMeshMaterialRegionKey(String);

impl ForeignMeshMaterialRegionKey {
    pub fn new(value: impl Into<String>) -> Result<Self, ForeignMeshMaterialRegionKeyError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ForeignMeshMaterialRegionKeyError::Empty);
        }
        if trimmed != value {
            return Err(ForeignMeshMaterialRegionKeyError::NotNormalized);
        }
        if is_transient_foreign_mesh_region_key(trimmed) {
            return Err(ForeignMeshMaterialRegionKeyError::TransientRuntimeIdentity);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ForeignMeshMaterialRegionKeySource {
    SourceMaterialSlot {
        slot_index: u32,
        slot_name: Option<String>,
    },
    ImporterAuthored {
        key: String,
    },
    DeterministicFallback {
        topology_hash: String,
        slot_index: u32,
    },
}

impl ForeignMeshMaterialRegionKeySource {
    pub const fn requires_weak_identity_diagnostic(&self) -> bool {
        matches!(self, Self::DeterministicFallback { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignMeshMaterialRegionDescriptor {
    pub key: ForeignMeshMaterialRegionKey,
    pub display_name: String,
    pub key_source: ForeignMeshMaterialRegionKeySource,
}

impl ForeignMeshMaterialRegionDescriptor {
    pub fn source_material_slot(slot_index: u32, slot_name: Option<&str>) -> Self {
        let display_name = slot_name
            .filter(|name| !name.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("Material slot {slot_index}"));
        Self {
            key: ForeignMeshMaterialRegionKey(format!("source_material_slot:{slot_index}")),
            display_name,
            key_source: ForeignMeshMaterialRegionKeySource::SourceMaterialSlot {
                slot_index,
                slot_name: slot_name.map(str::to_string),
            },
        }
    }

    pub fn importer_authored(
        key: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<Self, ForeignMeshMaterialRegionKeyError> {
        let key = ForeignMeshMaterialRegionKey::new(key)?;
        Ok(Self {
            display_name: normalized_display_name(display_name),
            key_source: ForeignMeshMaterialRegionKeySource::ImporterAuthored {
                key: key.as_str().to_string(),
            },
            key,
        })
    }

    pub fn deterministic_fallback(
        topology_hash: impl Into<String>,
        slot_index: u32,
        display_name: impl Into<String>,
    ) -> Result<Self, ForeignMeshMaterialRegionKeyError> {
        let topology_hash = topology_hash.into();
        let topology_hash = topology_hash.trim();
        if topology_hash.is_empty() {
            return Err(ForeignMeshMaterialRegionKeyError::Empty);
        }
        let key = ForeignMeshMaterialRegionKey::new(format!(
            "topology_hash:{topology_hash}:material_slot:{slot_index}"
        ))?;
        Ok(Self {
            display_name: normalized_display_name(display_name),
            key_source: ForeignMeshMaterialRegionKeySource::DeterministicFallback {
                topology_hash: topology_hash.to_string(),
                slot_index,
            },
            key,
        })
    }
}

fn normalized_display_name(display_name: impl Into<String>) -> String {
    let display_name = display_name.into();
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        "Material region".to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_transient_foreign_mesh_region_key(region_key: &str) -> bool {
    let normalized = region_key.to_ascii_lowercase();
    [
        "entity:",
        "ecs:",
        "renderable:",
        "renderable_index:",
        "renderer:",
        "draw:",
        "residency:",
        "palette:",
        "display:",
        "ui:",
        "artifact:",
        "generated:",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_material_slot_region_uses_stable_source_slot_identity() {
        let region = ForeignMeshMaterialRegionDescriptor::source_material_slot(3, Some("Body"));

        assert_eq!(region.key.as_str(), "source_material_slot:3");
        assert_eq!(region.display_name, "Body");
        assert!(!region.key_source.requires_weak_identity_diagnostic());
    }

    #[test]
    fn importer_authored_region_rejects_transient_runtime_identity() {
        let error =
            ForeignMeshMaterialRegionDescriptor::importer_authored("renderable_index:7", "Body")
                .expect_err("renderer index must not become authored region truth");

        assert_eq!(
            error,
            ForeignMeshMaterialRegionKeyError::TransientRuntimeIdentity
        );
    }

    #[test]
    fn deterministic_fallback_requires_diagnostic() {
        let region =
            ForeignMeshMaterialRegionDescriptor::deterministic_fallback("topology-abc", 2, "Body")
                .expect("deterministic fallback should be representable");

        assert_eq!(
            region.key.as_str(),
            "topology_hash:topology-abc:material_slot:2"
        );
        assert!(region.key_source.requires_weak_identity_diagnostic());
    }
}
