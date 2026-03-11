// src/domain/mod.rs

pub mod gameplay;
pub mod loot;
pub mod material_graph;
pub mod material_runtime;
pub mod render_sdf;
pub mod resources;
pub mod snapshot;
pub mod world;

pub use gameplay::events::*;
pub use gameplay::*;
pub use loot::*;
pub use material_graph::*;
pub use material_runtime::*;
pub use render_sdf::*;
pub use resources::*;
pub use snapshot::*;
pub use world::*;

use engine::prelude::{Entity, World};

pub fn is_active_player_entity(world: &World, entity: Entity) -> bool {
    world.get::<Player>(entity).is_some() && world.get::<PlayerActive>(entity).is_some()
}
