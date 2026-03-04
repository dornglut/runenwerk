pub mod collision_field;
pub mod components;
pub mod events;
pub mod geometry_graph;
pub mod loot;
pub mod resources;
pub mod snapshot;
pub mod worldgen;

pub use collision_field::*;
pub use components::*;
pub use events::*;
pub use geometry_graph::*;
pub use loot::*;
pub use resources::*;
pub use snapshot::*;
pub use worldgen::*;

use engine::prelude::{Entity, World};

pub fn is_active_player_entity(world: &World, entity: Entity) -> bool {
    world.get::<Player>(entity).is_some() && world.get::<PlayerActive>(entity).is_some()
}
