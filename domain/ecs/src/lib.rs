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
    BroadcastReader, BroadcastReaderState, BroadcastWriter, ConfiguredSystem, IntoSystem,
    IntoSystemConfigs, IntoSystemSetKey, ParamSlotId, ParamSlotMetadata, Res, ResMut, ResView,
    Runtime, SystemConfigExt, SystemId, SystemParam, SystemParamError, TickBufferDrainer,
    TickBufferReader, TickBufferWriter, WorkQueueDrainer, WorkQueueReader, WorkQueueWriter,
};
pub use world::{
    BroadcastDiagnosticsSnapshot, BroadcastKey, BroadcastLifetime, BroadcastObserverNotification,
    BroadcastObserverTrigger, BroadcastOverflowPolicy, BroadcastStreamConfig, BroadcastStreamStats,
    BroadcastTracingPolicy, ChangeExtractionFilter, ChangeExtractionWindow, ComponentChangeKind,
    ComponentChangeRecord, ComponentStructuralDelta, ComponentTypeKey, OwnerId,
    OwnerRole, EntityDespawnedEvent, EntityMut, EntityRef, EntitySpawnedEvent,
    MessagingDiagnosticsSnapshot, MessagingFinalizationCounters, Mut, OwnerState, OwnershipTarget,
    OwnershipTransferRecord, ResourceChangeKind, ResourceChangeRecord, ResourceOwnerKey,
    ResourceOwnershipDescriptor, ResourceStructuralDelta, ResourceTypeKey, StructuralDeltaBatch,
    StructuralDeltaRef, TickBufferConfig, TickBufferDiagnosticsSnapshot, TickBufferKey,
    TickBufferMeta, TickBufferProvenance, TickBufferPushError, TickBufferRecord,
    TickBufferRecordRef, TickBufferStats, WorkQueueConfig, WorkQueueDiagnosticsSnapshot,
    WorkQueueEnqueueError, WorkQueueKey, WorkQueueStats, World,
};
