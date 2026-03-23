use crate::plugins::world::caves::sectors::CaveSectorId;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct CaveRenderVisibilityResource {
    pub visible_sectors: BTreeSet<CaveSectorId>,
}
