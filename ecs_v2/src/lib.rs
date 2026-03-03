extern crate self as ecs_v2;

mod bundle;
mod component;
mod entity;
mod errors;
pub mod prelude;
pub mod query;
mod resource;
mod world;

pub use bundle::Bundle;
pub use component::Component;
pub use ecs_v2_macros::{Bundle, Component};
pub use entity::{Entity, EntityAllocator};
pub use errors::{CommandError, EntityError, QueryError, ResourceError};
pub use query::{
    QueryAccess, QueryBorrow, QueryBorrowMut, QueryData, QueryFilter, QueryState, QueryTypeAccess,
    With, Without,
};
pub use resource::Resource;
pub use world::{
    Commands, EntityDespawnedEvent, EntityMut, EntityRef, EntitySpawnedEvent, EventChannelConfig,
    EventChannelStats, EventLifetime, EventTracingPolicy, Mut, OverflowPolicy, Res, ResMut, World,
};
