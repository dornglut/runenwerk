extern crate self as ecs;

mod bundle;
mod commands;
mod component;
mod entity;
mod errors;
mod indexing;
pub mod prelude;
pub mod query;
pub mod reflect;
mod storage;
pub mod system;
pub mod telemetry;
mod world;
pub use bundle::Bundle;
pub use commands::{BatchCommands, Commands, DeferredCommand};
pub use component::{Component, ComponentState, Resource, StatefulComponent};
pub use ecs_macros::{
    Bundle, Component, Reflect, ReflectComponent, ReflectResource, Resource, StatefulComponent,
};
pub use entity::{Entity, EntityAllocator};
pub use errors::{CommandError, EntityError, QueryError, ResourceError, SpatialIndexError};
pub use indexing::{DEFAULT_SPATIAL_INDEX_NAME, SpatialHashConfig, SpatialHashIndex, SpatialIndex};
pub use query::{
    Added, Changed, Orphaned, Query, QueryAccess, QueryOrphaned, QueryOrphanedState, QueryState,
    QueryTypeAccess, With, Without,
};
pub use reflect::*;
pub use system::{
    BroadcastReader, BroadcastReaderState, BroadcastWriter, ConfiguredSystem, InputStreamDrainer,
    InputStreamReader, InputStreamWriter, IntoSystem, IntoSystemConfigs, IntoSystemSetKey,
    ParamSlotId, ParamSlotMetadata, QueueDrainer, QueueReader, QueueWriter, Res, ResMut, ResView,
    Runtime, SystemConfigExt, SystemId, SystemParam, SystemParamError,
};
pub use world::{
    BroadcastLifetime, BroadcastObserverNotification, BroadcastObserverTrigger,
    BroadcastOverflowPolicy, BroadcastStreamConfig, BroadcastStreamStats, BroadcastTracingPolicy,
    ComponentChangeKind, ComponentChangeRecord, EntityDespawnedEvent, EntityMut, EntityRef,
    EntitySpawnedEvent, InputStreamConfig, InputStreamPushError, InputStreamStats,
    MessagingFinalizationCounters, Mut, QueueConfig, QueueEnqueueError, QueueStats,
    ResourceChangeKind, ResourceChangeRecord, World,
};
