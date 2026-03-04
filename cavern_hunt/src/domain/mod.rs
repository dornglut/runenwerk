pub mod components;
pub mod events;
pub mod loot;
pub mod resources;
pub mod snapshot;
pub mod worldgen;

pub use components::*;
pub use events::*;
pub use loot::*;
pub use resources::*;
pub use snapshot::*;
pub use worldgen::*;

use engine::prelude::{Entity, World};

pub fn is_active_player_entity(world: &World, entity: Entity) -> bool {
    world.get::<Player>(entity).is_some() && world.get::<PlayerActive>(entity).is_some()
}
