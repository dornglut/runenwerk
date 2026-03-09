// src/domain/mod.rs

pub mod loot;
pub mod material_graph;
pub mod material_runtime;
pub mod resources;
pub mod gameplay;
pub mod world;
pub mod render_sdf;
pub mod snapshot;

pub use gameplay::events::*;
pub use loot::*;
pub use material_graph::*;
pub use material_runtime::*;
pub use resources::*;
pub use snapshot::*;
pub use world::*;
pub use gameplay::*;
pub use render_sdf::*;

use engine::prelude::{Entity, World};

pub fn is_active_player_entity(world: &World, entity: Entity) -> bool {
    world.get::<Player>(entity).is_some() && world.get::<PlayerActive>(entity).is_some()
}
