extern crate self as ecs;

mod bundle;
mod commands;
mod component;
mod entity;
mod errors;
pub mod prelude;
pub mod query;
mod indexing;
mod storage;
pub mod system;
pub mod telemetry;
mod world;
pub mod reflect;
pub use reflect::*;
pub use bundle::Bundle;
pub use commands::{BatchCommands, Commands, DeferredCommand};
pub use component::{Component, ComponentState, Resource, StatefulComponent};
pub use ecs_macros::{
    Bundle,
    Component,
    Reflect,
    ReflectComponent,
    ReflectResource,
    Resource,
    StatefulComponent,
};
pub use entity::{Entity, EntityAllocator};
pub use errors::{CommandError, EntityError, QueryError, ResourceError, SpatialIndexError};
pub use query::{
    Added, Changed, Orphaned, Query, QueryAccess, QueryOrphaned, QueryOrphanedState, QueryState,
    QueryTypeAccess, With, Without,
};
pub use indexing::{DEFAULT_SPATIAL_INDEX_NAME, SpatialHashConfig, SpatialHashIndex, SpatialIndex};
pub use system::{
    ConfiguredSystem, EventChannel, EventReader, EventWriter, IntoSystem, IntoSystemConfigs,
    IntoSystemSetKey, Res, ResMut, ResView, Runtime, SystemConfigExt, SystemParam,
    SystemParamError,
};
pub use world::{
    ComponentChangeKind, ComponentChangeRecord, EntityDespawnedEvent, EntityMut, EntityRef,
    EntitySpawnedEvent, EventChannelConfig, EventChannelStats, EventLifetime,
    EventObserverNotification, EventTracingPolicy, Mut, ObserverTrigger, OverflowPolicy,
    ResourceChangeKind, ResourceChangeRecord, World,
};
