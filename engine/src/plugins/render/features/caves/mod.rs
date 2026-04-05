use std::collections::BTreeSet;
use world_sdf::CaveSectorId;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct CaveRenderVisibilityResource {
    pub visible_sectors: BTreeSet<CaveSectorId>,
}
