#![doc = include_str!("../README.md")]
#![doc = include_str!("../docs/reference/usage-guide.md")]
#![doc = include_str!("../docs/reference/architecture.md")]

extern crate self as ecs;

mod bundle;
mod component;
mod entity;
mod errors;
pub mod prelude;
pub mod query;
mod storage;
pub mod system;
pub mod telemetry;
mod world;

pub use bundle::Bundle;
pub use component::{Component, Resource};
pub use ecs_macros::{Bundle, Component, Resource};
pub use entity::{Entity, EntityAllocator};
pub use errors::{CommandError, EntityError, QueryError, ResourceError};
pub use query::{Added, Changed, Query, QueryAccess, QueryState, QueryTypeAccess, With, Without};
pub use system::{
    ConfiguredSystem, EventReader, EventWriter, IntoSystem, IntoSystemConfigs, IntoSystemSetKey,
    Res, ResMut, Runtime, SystemConfigExt, SystemParam, SystemParamError,
};
pub use world::{
    Commands, ComponentChangeKind, ComponentChangeRecord, EntityDespawnedEvent, EntityMut,
    EntityRef, EntitySpawnedEvent, EventChannelConfig, EventChannelStats, EventLifetime,
    EventObserverNotification, EventTracingPolicy, Mut, ObserverTrigger, OverflowPolicy,
    ResourceChangeKind, ResourceChangeRecord, World,
};
