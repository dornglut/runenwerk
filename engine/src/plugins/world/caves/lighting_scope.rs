use super::sectors::CaveSectorId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CaveLightVolumeScope {
    pub sector_id: CaveSectorId,
    pub local_center: [f32; 3],
    pub local_extents: [f32; 3],
    pub intensity_scale: f32,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldCaveLightingScopeResource {
    pub scopes: BTreeMap<CaveSectorId, Vec<CaveLightVolumeScope>>,
}

impl WorldCaveLightingScopeResource {
    pub fn scopes_for_sector(&self, sector_id: CaveSectorId) -> &[CaveLightVolumeScope] {
        self.scopes
            .get(&sector_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
