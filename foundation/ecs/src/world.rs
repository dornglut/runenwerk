use crate::bundle::Bundle;
use crate::component::Component;
use crate::entity::{Entity, EntityAllocator};
use crate::errors::{CommandError, EntityError, ResourceError};
use crate::query::{QueryBorrow, QueryBorrowMut, QueryData};
use crate::resource::Resource;
use std::any::{Any, TypeId, type_name};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::{Deref, DerefMut};

include!("world/internal/events_and_indexes.rs");

include!("world/internal/handles_and_commands.rs");

include!("world/internal/world_struct.rs");

include!("world/internal/world_core_impl.rs");

include!("world/internal/world_index_and_events_impl.rs");

include!("world/internal/world_internal_impl.rs");
