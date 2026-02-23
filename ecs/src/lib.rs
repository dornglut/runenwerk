extern crate self as ecs;

mod bundle;
mod component;
mod component_registry;
mod entity;
mod entity_builder;
pub mod prelude;
mod query;
mod resource;
mod table;
mod utils;
mod world;

pub use bundle::ComponentBundle;
pub use component::Component;
pub use component_registry::{ComponentKey, ComponentRegistry};
pub use ecs_macros::{Component, ComponentBundle};
pub use entity::{EntityAllocator, EntityHandle};
pub use entity_builder::{EntityBuilder, WorldBuilderExt};
pub use query::{ComponentTuple, QueryBuilder, TypedQueryIterator, WorldQueryExt};
pub use resource::Resource;
pub use table::{Archetype, ArchetypeKey};
pub use utils::init_tracing;
pub use world::{
    EventChannelConfig, EventChannelStats, EventLifetime, EventObserverNotification,
    EventTracingPolicy, ObserverTrigger, OverflowPolicy, World,
};

pub(crate) use table::{AnyStorage, Column, Row};
