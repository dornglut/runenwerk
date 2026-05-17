//! File: domain/editor/editor_scene/src/model/material.rs
//! Purpose: Source-backed scene material palette and primitive slot contracts.

use asset::{ArtifactCacheKey, AssetArtifactId, AssetId};
use editor_core::EntityId;

pub const DEFAULT_SCENE_MATERIAL_SLOT_ID: SceneMaterialSlotId = SceneMaterialSlotId(1);
pub const MAX_PORTABLE_SCENE_MATERIAL_SLOTS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneMaterialSlotId(pub u64);

impl SceneMaterialSlotId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialSlot {
    pub slot_id: SceneMaterialSlotId,
    pub display_name: String,
    pub material_asset_id: Option<AssetId>,
    pub formed_material_artifact_id: Option<AssetArtifactId>,
    pub shader_artifact_id: Option<AssetArtifactId>,
    pub material_cache_key: Option<ArtifactCacheKey>,
    pub shader_cache_key: Option<ArtifactCacheKey>,
    pub prior_valid: bool,
    pub is_default: bool,
}

impl SceneMaterialSlot {
    pub fn new(slot_id: SceneMaterialSlotId, display_name: impl Into<String>) -> Self {
        Self {
            slot_id,
            display_name: display_name.into(),
            material_asset_id: None,
            formed_material_artifact_id: None,
            shader_artifact_id: None,
            material_cache_key: None,
            shader_cache_key: None,
            prior_valid: false,
            is_default: false,
        }
    }

    pub fn default_generated() -> Self {
        Self::new(DEFAULT_SCENE_MATERIAL_SLOT_ID, "Default Material").with_default(true)
    }

    pub fn with_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    pub fn with_material_asset(mut self, asset_id: AssetId) -> Self {
        self.material_asset_id = Some(asset_id);
        self
    }

    pub fn with_formed_artifact(mut self, artifact_id: AssetArtifactId) -> Self {
        self.formed_material_artifact_id = Some(artifact_id);
        self
    }

    pub fn with_shader_artifact(mut self, artifact_id: AssetArtifactId) -> Self {
        self.shader_artifact_id = Some(artifact_id);
        self
    }

    pub fn with_cache_keys(
        mut self,
        material_cache_key: ArtifactCacheKey,
        shader_cache_key: ArtifactCacheKey,
    ) -> Self {
        self.material_cache_key = Some(material_cache_key);
        self.shader_cache_key = Some(shader_cache_key);
        self
    }

    pub fn with_prior_valid(mut self, prior_valid: bool) -> Self {
        self.prior_valid = prior_valid;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialPalette {
    pub slots: Vec<SceneMaterialSlot>,
}

impl Default for SceneMaterialPalette {
    fn default() -> Self {
        Self {
            slots: vec![SceneMaterialSlot::default_generated()],
        }
    }
}

impl SceneMaterialPalette {
    pub fn new(slots: impl IntoIterator<Item = SceneMaterialSlot>) -> Result<Self, String> {
        let slots = slots.into_iter().collect::<Vec<_>>();
        if slots.is_empty() {
            return Err("scene material palette requires a default material slot".to_string());
        }
        if slots.len() > MAX_PORTABLE_SCENE_MATERIAL_SLOTS {
            return Err(format!(
                "scene material palette has {} slots, portable limit is {}",
                slots.len(),
                MAX_PORTABLE_SCENE_MATERIAL_SLOTS
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut default_count = 0;
        for slot in &slots {
            if slot.slot_id.raw() == 0 {
                return Err("scene material slot id must be nonzero".to_string());
            }
            if !ids.insert(slot.slot_id) {
                return Err(format!(
                    "duplicate scene material slot {}",
                    slot.slot_id.raw()
                ));
            }
            if slot.is_default {
                default_count += 1;
            }
        }
        if default_count != 1 {
            return Err("scene material palette requires exactly one default slot".to_string());
        }
        Ok(Self { slots })
    }

    pub fn default_slot(&self) -> &SceneMaterialSlot {
        self.slots
            .iter()
            .find(|slot| slot.is_default)
            .expect("palette constructor enforces a default slot")
    }

    pub fn contains_slot(&self, slot_id: SceneMaterialSlotId) -> bool {
        self.slots.iter().any(|slot| slot.slot_id == slot_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveMaterialSlotAssignment {
    pub entity_id: EntityId,
    pub slot_id: SceneMaterialSlotId,
}

impl PrimitiveMaterialSlotAssignment {
    pub const fn new(entity_id: EntityId, slot_id: SceneMaterialSlotId) -> Self {
        Self { entity_id, slot_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::asset_id;

    #[test]
    fn palette_requires_exactly_one_default_slot() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Rock")
                .with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");

        assert_eq!(
            palette.default_slot().slot_id,
            DEFAULT_SCENE_MATERIAL_SLOT_ID
        );
        assert!(palette.contains_slot(SceneMaterialSlotId::new(2)));
    }

    #[test]
    fn palette_rejects_duplicate_slot_ids() {
        let error = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(DEFAULT_SCENE_MATERIAL_SLOT_ID, "Duplicate"),
        ])
        .expect_err("duplicate slot id should fail");

        assert!(error.contains("duplicate"));
    }
}
