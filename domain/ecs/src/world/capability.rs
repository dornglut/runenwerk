//! Invocation-scoped projections owned by the `World` implementation.
//!
//! These capabilities are deliberately not general world handles.  The only
//! raw `World` address is held by the invocation authority in `system::extract`;
//! this module immediately projects that authority to the concrete storage and
//! bookkeeping domains used by a parameter.  The projections are valid only
//! while the invocation's structural freeze is active.

use super::World;
use super::change_tracking::{
    ComponentChangeKind, ComponentChangeRecord, ComponentMeta, RemovedComponentRecord,
    ResourceChangeKind, ResourceChangeRecord, ResourceMeta,
};
use super::component_indexes::{ComponentIndexKey, ComponentIndexStorage};
use super::messaging::broadcast::{
    BroadcastObserver, BroadcastObserverNotification, BroadcastObserverTrigger,
    BroadcastStreamStorage,
};
use super::messaging::tick_buffer::TickBufferStorage;
use super::messaging::work_queue::WorkQueueStorage;
use super::messaging::{BroadcastKey, TickBufferKey, TickBufferProvenance, WorkQueueKey};
use crate::component::Component;
use crate::entity::Entity;
use crate::entity::WorldScopeId;
use crate::errors::ResourceError;
use crate::storage::{ArchetypeExecutionBinding, ArchetypeRegistry, EntityLocationMap};
use std::any::{TypeId, type_name};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::time::Instant;

/// The sole invocation-scoped authority from which narrow capabilities are
/// projected.  It is never stored in a user-facing parameter value.
#[derive(Copy, Clone)]
pub(crate) struct WorldAuthority<'world> {
    world: NonNull<World>,
    _marker: PhantomData<&'world mut World>,
}

impl<'world> WorldAuthority<'world> {
    pub(crate) fn new(world: &'world mut World) -> Self {
        Self {
            world: NonNull::from(world),
            _marker: PhantomData,
        }
    }

    pub(crate) fn query(self) -> QueryCapability<'world> {
        // Safety: the authority was constructed from the live invocation World;
        // the bridge immediately projects only owned query fields.
        unsafe { QueryCapability::from_world_ptr(self.world) }
    }

    pub(crate) fn messaging(self) -> MessagingCapability<'world> {
        unsafe { MessagingCapability::from_world_ptr(self.world) }
    }

    pub(crate) unsafe fn world_mut(mut self) -> &'world mut World {
        unsafe { self.world.as_mut() }
    }

    pub(crate) fn resource<T: crate::component::Resource>(
        self,
    ) -> Result<ResourceCapability<'world, T>, ResourceError> {
        // Safety: the authority lifetime is the invocation lifetime and the
        // resource bridge retains only the stable boxed payload address.
        unsafe { World::resource_capability_from_ptr(self.world, false) }
    }

    pub(crate) fn resource_mut<T: crate::component::Resource>(
        self,
    ) -> Result<ResourceCapability<'world, T>, ResourceError> {
        // Safety: access validation rejects overlapping resource borrows before
        // this projection is manufactured.
        unsafe { World::resource_capability_from_ptr(self.world, true) }
    }
}

/// Narrow query-domain authority.  It contains pointers only to the storage,
/// location, and change-bookkeeping fields needed by supported query forms.
#[doc(hidden)]
pub struct QueryCapability<'world> {
    world_scope: WorldScopeId,
    alive_entities: NonNull<BTreeSet<Entity>>,
    archetype_registry: NonNull<ArchetypeRegistry>,
    entity_locations: NonNull<EntityLocationMap>,
    component_type_registry: NonNull<HashMap<TypeId, ComponentMeta>>,
    component_indexes: NonNull<RefCell<HashMap<ComponentIndexKey, Box<dyn ComponentIndexStorage>>>>,
    change_tick: NonNull<u64>,
    current_frame_index: NonNull<u64>,
    component_change_ticks: NonNull<HashMap<TypeId, u64>>,
    component_change_log: NonNull<Vec<ComponentChangeRecord>>,
    removed_component_records: NonNull<HashMap<TypeId, Vec<RemovedComponentRecord>>>,
    _marker: PhantomData<&'world World>,
}

impl<'world> Copy for QueryCapability<'world> {}
impl<'world> Clone for QueryCapability<'world> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'world> QueryCapability<'world> {
    pub(super) fn from_world(world: &'world World) -> Self {
        // Safety: this is the one World-owned projection point.  All fields are
        // part of `world` and remain at stable addresses while the invocation's
        // structural freeze is active; no capability stores an archetype row or
        // movable map entry address.
        Self {
            world_scope: world.scope_id(),
            alive_entities: NonNull::from(&world.alive_entities),
            archetype_registry: NonNull::from(&world.archetype_registry),
            entity_locations: NonNull::from(&world.entity_locations),
            component_type_registry: NonNull::from(&world.component_type_registry),
            component_indexes: NonNull::from(&world.component_indexes),
            change_tick: NonNull::from(&world.change_tick),
            current_frame_index: NonNull::from(&world.current_frame_index),
            component_change_ticks: NonNull::from(&world.component_change_ticks),
            component_change_log: NonNull::from(&world.component_change_log),
            removed_component_records: NonNull::from(&world.removed_component_records),
            _marker: PhantomData,
        }
    }

    pub(super) fn from_world_mut(world: &'world mut World) -> Self {
        Self {
            world_scope: world.scope_id(),
            alive_entities: NonNull::from(&mut world.alive_entities),
            archetype_registry: NonNull::from(&mut world.archetype_registry),
            entity_locations: NonNull::from(&mut world.entity_locations),
            component_type_registry: NonNull::from(&mut world.component_type_registry),
            component_indexes: NonNull::from(&mut world.component_indexes),
            change_tick: NonNull::from(&mut world.change_tick),
            current_frame_index: NonNull::from(&mut world.current_frame_index),
            component_change_ticks: NonNull::from(&mut world.component_change_ticks),
            component_change_log: NonNull::from(&mut world.component_change_log),
            removed_component_records: NonNull::from(&mut world.removed_component_records),
            _marker: PhantomData,
        }
    }

    pub(super) unsafe fn from_world_ptr(world: NonNull<World>) -> Self {
        let world_ptr = world.as_ptr();
        Self {
            world_scope: unsafe { (*world_ptr).scope_id() },
            alive_entities: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).alive_entities))
            },
            archetype_registry: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).archetype_registry))
            },
            entity_locations: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).entity_locations))
            },
            component_type_registry: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).component_type_registry))
            },
            component_indexes: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).component_indexes))
            },
            change_tick: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).change_tick))
            },
            current_frame_index: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).current_frame_index))
            },
            component_change_ticks: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).component_change_ticks))
            },
            component_change_log: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).component_change_log))
            },
            removed_component_records: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!(
                    (*world_ptr).removed_component_records
                ))
            },
            _marker: PhantomData,
        }
    }

    pub(crate) fn current_change_tick(self) -> u64 {
        unsafe { *self.change_tick.as_ptr() }
    }

    pub(crate) fn world_scope(self) -> WorldScopeId {
        self.world_scope
    }

    pub(crate) fn matching_entities_into(
        self,
        required_present: &[TypeId],
        excluded: &[TypeId],
        out: &mut Vec<Entity>,
    ) {
        let start = Instant::now();
        unsafe {
            self.archetype_registry.as_ref().collect_matching_entities(
                required_present,
                excluded,
                out,
            )
        };
        let count = out.len() as u64;
        crate::telemetry::record_query_matching(start.elapsed().as_nanos() as u64, count, count);
    }

    pub(crate) fn matching_archetype_bindings_into(
        self,
        required_present: &[TypeId],
        excluded: &[TypeId],
        out: &mut Vec<ArchetypeExecutionBinding>,
    ) -> bool {
        unsafe {
            self.archetype_registry.as_ref().collect_matching_bindings(
                required_present,
                excluded,
                out,
            )
        }
    }

    pub(crate) fn archetype_entity_at(self, archetype_index: usize, row: usize) -> Option<Entity> {
        unsafe {
            self.archetype_registry
                .as_ref()
                .entity_at(archetype_index, row)
        }
    }

    pub(crate) fn entity_matches_component_constraints(
        self,
        entity: Entity,
        required_present: &[TypeId],
        excluded: &[TypeId],
    ) -> bool {
        self.contains(entity)
            && required_present
                .iter()
                .all(|type_id| self.has_component_by_type_id(entity, *type_id))
            && excluded
                .iter()
                .all(|type_id| !self.has_component_by_type_id(entity, *type_id))
    }

    pub(crate) fn contains(self, entity: Entity) -> bool {
        unsafe { self.alive_entities.as_ref().contains(&entity) }
    }

    pub(crate) fn has_component_by_type_id(self, entity: Entity, type_id: TypeId) -> bool {
        let locations = unsafe { self.entity_locations.as_ref() };
        let Some(location) = locations.get(entity) else {
            return false;
        };
        unsafe {
            self.archetype_registry
                .as_ref()
                .component_types(location.archetype_id)
        }
        .is_some_and(|types| types.binary_search(&type_id).is_ok())
    }

    pub(crate) fn component<T: Component>(self, entity: Entity) -> Option<&'world T> {
        let locations = unsafe { self.entity_locations.as_ref() };
        let ptr = unsafe {
            self.archetype_registry
                .as_ref()
                .component_ptr::<T>(entity, locations)
        }?;
        // Safety: the storage registry verified the typed column and row.  The
        // boxed payload allocation is stable across registry/container moves.
        Some(unsafe { &*ptr })
    }

    /// # Safety
    /// The query access contract must contain the exclusive component borrow and
    /// structural mutation must remain frozen for `'world`.
    pub(crate) unsafe fn component_mut<T: Component>(
        mut self,
        entity: Entity,
    ) -> Option<&'world mut T> {
        let locations = unsafe { self.entity_locations.as_ref() };
        let registry = unsafe { self.archetype_registry.as_mut() };
        let ptr = registry.component_mut_ptr::<T>(entity, locations)?;
        Some(unsafe { &mut *ptr })
    }

    pub(crate) fn component_metadata<T: Component>(self, entity: Entity) -> Option<(u64, u64)> {
        let locations = unsafe { self.entity_locations.as_ref() };
        let metadata = unsafe {
            self.archetype_registry
                .as_ref()
                .component_metadata::<T>(entity, locations)
        }?;
        Some((metadata.added_tick, metadata.changed_tick))
    }

    pub(crate) fn mark_component_modified_by_id(
        mut self,
        entity: Entity,
        component_type: TypeId,
        component_name: &'static str,
    ) {
        let tick = self.record_component_change(
            entity,
            component_type,
            component_name,
            ComponentChangeKind::Modified,
        );
        let locations = unsafe { self.entity_locations.as_ref() };
        let _ = unsafe {
            self.archetype_registry
                .as_mut()
                .mark_component_changed_by_id(entity, component_type, tick, locations)
        };
    }

    pub(crate) fn mark_component_modified<T: Component>(self, entity: Entity) {
        if self.component::<T>(entity).is_some() {
            self.mark_component_modified_by_id(entity, TypeId::of::<T>(), T::component_name());
        }
    }

    fn record_component_change(
        mut self,
        entity: Entity,
        component_type: TypeId,
        component_name: &'static str,
        kind: ComponentChangeKind,
    ) -> u64 {
        let tick = unsafe {
            let tick = self.change_tick.as_mut();
            *tick = tick.saturating_add(1);
            *tick
        };
        unsafe {
            self.component_change_ticks
                .as_mut()
                .insert(component_type, tick)
        };
        let component_key = unsafe { self.component_type_registry.as_ref().get(&component_type) }
            .map(|meta| meta.id)
            .unwrap_or_default();
        unsafe {
            self.component_change_log
                .as_mut()
                .push(ComponentChangeRecord {
                    tick,
                    frame: *self.current_frame_index.as_ptr(),
                    entity,
                    component_type,
                    component_key,
                    component_name,
                    kind,
                })
        };
        if matches!(kind, ComponentChangeKind::Removed) {
            unsafe {
                self.removed_component_records
                    .as_mut()
                    .entry(component_type)
                    .or_default()
                    .push(RemovedComponentRecord { tick, entity })
            };
        }
        self.mark_component_indexes_dirty(component_type);
        tick
    }

    fn mark_component_indexes_dirty(self, component_type: TypeId) {
        let mut indexes = unsafe { self.component_indexes.as_ref().borrow_mut() };
        for (index_key, index) in indexes.iter_mut() {
            if index_key.component_type == component_type {
                index.mark_dirty();
            }
        }
    }

    pub(crate) fn component_changed_for_entity_since<T: Component>(
        self,
        entity: Entity,
        tick: u64,
    ) -> bool {
        let start = Instant::now();
        let changed = self
            .component_metadata::<T>(entity)
            .is_some_and(|(_, changed_tick)| changed_tick > tick);
        crate::telemetry::record_changed_check(start.elapsed().as_nanos() as u64);
        changed
    }

    pub(crate) fn component_added_for_entity_since<T: Component>(
        self,
        entity: Entity,
        tick: u64,
    ) -> bool {
        let start = Instant::now();
        let added = self
            .component_metadata::<T>(entity)
            .is_some_and(|(added_tick, _)| added_tick > tick);
        crate::telemetry::record_added_check(start.elapsed().as_nanos() as u64);
        added
    }

    pub(crate) fn removed_component_records_current_window(
        self,
        component_type: TypeId,
        out: &mut Vec<(Entity, u64)>,
    ) {
        out.clear();
        let records = unsafe { self.removed_component_records.as_ref() };
        if let Some(records) = records.get(&component_type) {
            out.extend(records.iter().map(|record| (record.entity, record.tick)));
        }
    }
}

/// Stable typed resource payload plus the separate narrow change recorder used
/// by `ResMut`.  No resource parameter retains a world or registry-entry pointer.
#[doc(hidden)]
pub struct ResourceCapability<'world, T> {
    value: NonNull<T>,
    mutation: Option<ResourceMutationCapability<'world>>,
    _marker: PhantomData<&'world T>,
}

/// Invocation-scoped messaging projection.  Every operation looks up its
/// `TypeId` in the owning map; no movable `HashMap` entry address is retained.
#[doc(hidden)]
pub struct MessagingCapability<'world> {
    broadcast_streams: NonNull<HashMap<TypeId, BroadcastStreamStorage>>,
    work_queues: NonNull<HashMap<TypeId, WorkQueueStorage>>,
    tick_buffers: NonNull<HashMap<TypeId, TickBufferStorage>>,
    broadcast_observers: NonNull<HashMap<String, BroadcastObserver>>,
    broadcast_observer_notifications: NonNull<Vec<BroadcastObserverNotification>>,
    next_broadcast_key: NonNull<u64>,
    next_work_queue_key: NonNull<u64>,
    next_tick_buffer_key: NonNull<u64>,
    current_buffer_tick: NonNull<u64>,
    finalized_buffer_tick: NonNull<Option<u64>>,
    _marker: PhantomData<&'world mut World>,
}

impl<'world> Copy for MessagingCapability<'world> {}
impl<'world> Clone for MessagingCapability<'world> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'world> MessagingCapability<'world> {
    pub(super) unsafe fn from_world_ptr(world: NonNull<World>) -> Self {
        let world_ptr = world.as_ptr();
        Self {
            broadcast_streams: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).broadcast_streams))
            },
            work_queues: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).work_queues))
            },
            tick_buffers: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).tick_buffers))
            },
            broadcast_observers: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).broadcast_observers))
            },
            broadcast_observer_notifications: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!(
                    (*world_ptr).broadcast_observer_notifications
                ))
            },
            next_broadcast_key: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).next_broadcast_key))
            },
            next_work_queue_key: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).next_work_queue_key))
            },
            next_tick_buffer_key: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).next_tick_buffer_key))
            },
            current_buffer_tick: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).current_buffer_tick))
            },
            finalized_buffer_tick: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).finalized_buffer_tick))
            },
            _marker: PhantomData,
        }
    }

    fn broadcast_key(mut self) -> BroadcastKey {
        unsafe {
            *self.next_broadcast_key.as_mut() = self.next_broadcast_key.as_ref().saturating_add(1);
            BroadcastKey(*self.next_broadcast_key.as_ptr())
        }
    }
    fn work_queue_key(mut self) -> WorkQueueKey {
        unsafe {
            *self.next_work_queue_key.as_mut() =
                self.next_work_queue_key.as_ref().saturating_add(1);
            WorkQueueKey(*self.next_work_queue_key.as_ptr())
        }
    }
    fn tick_buffer_key(mut self) -> TickBufferKey {
        unsafe {
            *self.next_tick_buffer_key.as_mut() =
                self.next_tick_buffer_key.as_ref().saturating_add(1);
            TickBufferKey(*self.next_tick_buffer_key.as_ptr())
        }
    }

    pub(crate) fn broadcast_read<T: 'static>(self) -> &'world [T] {
        unsafe {
            self.broadcast_streams
                .as_ref()
                .get(&TypeId::of::<T>())
                .map(|stream| stream.messages_ref::<T>())
                .unwrap_or(&[])
        }
    }
    pub(crate) fn broadcast_read_since<T: 'static>(mut self, sequence: u64) -> (&'world [T], u64) {
        unsafe {
            let Some(stream) = self.broadcast_streams.as_mut().get_mut(&TypeId::of::<T>()) else {
                return (&[], 0);
            };
            let next = stream.next_sequence;
            stream.record_consumer_read_from(sequence);
            (stream.messages_ref_since::<T>(sequence), next)
        }
    }
    pub(crate) fn broadcast_publish<T: 'static>(mut self, message: T) {
        let (accepted, stream_type_name) = unsafe {
            let streams = self.broadcast_streams.as_mut();
            let type_id = TypeId::of::<T>();
            if let std::collections::hash_map::Entry::Vacant(entry) = streams.entry(type_id) {
                let key = self.broadcast_key();
                entry.insert(BroadcastStreamStorage::new::<T>(key));
            }
            let stream = streams.get_mut(&type_id).expect("broadcast stream exists");
            let stream_type_name = stream.stream_type_name;
            let config = stream.config;
            let messages = stream.messages_mut::<T>();
            let accepted = match config.capacity {
                None => {
                    messages.push(message);
                    true
                }
                Some(0) => false,
                Some(capacity) if messages.len() < capacity => {
                    messages.push(message);
                    true
                }
                Some(_) => match config.overflow {
                    super::messaging::BroadcastOverflowPolicy::DropOldest => {
                        messages.remove(0);
                        messages.push(message);
                        stream.advance_sequence_for_removed(1);
                        true
                    }
                    super::messaging::BroadcastOverflowPolicy::DropNewest => false,
                    super::messaging::BroadcastOverflowPolicy::Panic => {
                        panic!("broadcast stream overflow")
                    }
                },
            };
            stream.emitted = stream.emitted.saturating_add(1);
            if !accepted {
                stream.dropped = stream.dropped.saturating_add(1);
            }
            if accepted {
                stream.next_sequence = stream.next_sequence.saturating_add(1);
            }
            (accepted, stream_type_name)
        };
        if accepted {
            self.trigger_broadcast_observers(
                TypeId::of::<T>(),
                stream_type_name,
                BroadcastObserverTrigger::OnPublish,
                1,
            );
        }
    }

    pub(crate) fn work_queue_iter<T: 'static>(
        self,
    ) -> Box<dyn Iterator<Item = &'world T> + 'world> {
        unsafe {
            match self.work_queues.as_ref().get(&TypeId::of::<T>()) {
                Some(queue) => Box::new(queue.messages_ref::<T>().iter()),
                None => Box::new(std::iter::empty()),
            }
        }
    }
    pub(crate) fn work_queue_len<T: 'static>(self) -> usize {
        unsafe {
            self.work_queues
                .as_ref()
                .get(&TypeId::of::<T>())
                .map(|q| q.messages_len_any())
                .unwrap_or(0)
        }
    }
    pub(crate) fn work_queue_peek<T: 'static>(self) -> Option<&'world T> {
        unsafe {
            self.work_queues
                .as_ref()
                .get(&TypeId::of::<T>())
                .and_then(|q| q.messages_ref::<T>().front())
        }
    }
    pub(crate) fn work_queue_enqueue<T: 'static>(
        mut self,
        message: T,
    ) -> Result<(), super::messaging::WorkQueueEnqueueError> {
        unsafe {
            let queues = self.work_queues.as_mut();
            let id = TypeId::of::<T>();
            if let std::collections::hash_map::Entry::Vacant(entry) = queues.entry(id) {
                let key = self.work_queue_key();
                entry.insert(WorkQueueStorage::new::<T>(key));
            }
            let queue = queues.get_mut(&id).unwrap();
            if let Some(capacity) = queue.config.capacity
                && queue.messages_ref::<T>().len() >= capacity
            {
                queue.rejected += 1;
                return Err(super::messaging::WorkQueueEnqueueError::Backpressure {
                    work_queue_type: queue.work_queue_type_name,
                    capacity,
                });
            }
            queue.messages_mut::<T>().push_back(message);
            queue.enqueued += 1;
            Ok(())
        }
    }
    pub(crate) fn work_queue_drain<T: 'static>(mut self) -> Vec<T> {
        unsafe {
            let Some(q) = self.work_queues.as_mut().get_mut(&TypeId::of::<T>()) else {
                return Vec::new();
            };
            let out: Vec<_> = q.messages_mut::<T>().drain(..).collect();
            q.drained += out.len() as u64;
            out
        }
    }
    pub(crate) fn work_queue_clear<T: 'static>(mut self) -> usize {
        unsafe {
            let Some(q) = self.work_queues.as_mut().get_mut(&TypeId::of::<T>()) else {
                return 0;
            };
            let n = q.clear_any();
            q.drained += n as u64;
            n
        }
    }

    pub(crate) fn current_buffer_messages<T: 'static>(self) -> &'world [T] {
        unsafe {
            let tick = *self.current_buffer_tick.as_ptr();
            self.tick_buffers
                .as_ref()
                .get(&TypeId::of::<T>())
                .and_then(|b| b.buckets_ref::<T>().get(&tick).map(Vec::as_slice))
                .unwrap_or(&[])
        }
    }
    pub(crate) fn current_buffer_tick(self) -> u64 {
        unsafe { *self.current_buffer_tick.as_ptr() }
    }
    pub(crate) fn buffer_messages_at_tick<T: 'static>(self, tick: u64) -> &'world [T] {
        unsafe {
            self.tick_buffers
                .as_ref()
                .get(&TypeId::of::<T>())
                .and_then(|b| b.buckets_ref::<T>().get(&tick).map(Vec::as_slice))
                .unwrap_or(&[])
        }
    }
    pub(crate) fn push_buffer_message<T: 'static>(
        mut self,
        tick: u64,
        provenance: TickBufferProvenance,
        message: T,
    ) -> Result<super::messaging::TickBufferMeta, super::messaging::TickBufferPushError> {
        unsafe {
            let buffers = self.tick_buffers.as_mut();
            let id = TypeId::of::<T>();
            if let std::collections::hash_map::Entry::Vacant(entry) = buffers.entry(id) {
                let key = self.tick_buffer_key();
                entry.insert(TickBufferStorage::new::<T>(key));
            }
            let buffer = buffers.get_mut(&id).unwrap();
            if let Some(finalized) = *self.finalized_buffer_tick.as_ptr()
                && tick <= finalized
            {
                buffer.rejected += 1;
                return Err(super::messaging::TickBufferPushError::FinalizedTick {
                    buffer_type: buffer.buffer_type_name,
                    tick,
                    finalized_tick: finalized,
                });
            }
            if let Some(capacity) = buffer.config.capacity
                && buffer.pending_messages >= capacity
            {
                buffer.rejected += 1;
                return Err(super::messaging::TickBufferPushError::Backpressure {
                    buffer_type: buffer.buffer_type_name,
                    capacity,
                });
            }
            if buffer.is_duplicate::<T>(tick, &message) {
                buffer.dropped = buffer.dropped.saturating_add(1);
                return Err(super::messaging::TickBufferPushError::Deduplicated {
                    buffer_type: buffer.buffer_type_name,
                });
            }
            buffer.next_sequence += 1;
            let meta = super::messaging::TickBufferMeta {
                buffer_key: buffer.buffer_key,
                tick,
                sequence: buffer.next_sequence,
                provenance,
            };
            buffer
                .buckets_mut::<T>()
                .entry(tick)
                .or_default()
                .push(message);
            buffer.metadata.entry(tick).or_default().push(meta);
            buffer.pending_messages += 1;
            buffer.pushed += 1;
            Ok(meta)
        }
    }
    pub(crate) fn drain_buffer<T: 'static>(mut self, tick: u64) -> Vec<T> {
        unsafe {
            let Some(b) = self.tick_buffers.as_mut().get_mut(&TypeId::of::<T>()) else {
                return Vec::new();
            };
            let out = b.buckets_mut::<T>().remove(&tick).unwrap_or_default();
            b.metadata_remove(tick);
            b.pending_messages = b.pending_messages.saturating_sub(out.len());
            b.drained += out.len() as u64;
            out
        }
    }

    fn trigger_broadcast_observers(
        mut self,
        stream_type: TypeId,
        stream_type_name: &'static str,
        trigger: BroadcastObserverTrigger,
        message_count: usize,
    ) {
        unsafe {
            let observers = self.broadcast_observers.as_mut();
            let notifications = self.broadcast_observer_notifications.as_mut();
            for observer in observers.values_mut() {
                if observer.stream_type != stream_type || observer.trigger != trigger {
                    continue;
                }
                observer.invocations = observer.invocations.saturating_add(1);
                notifications.push(BroadcastObserverNotification {
                    observer_id: observer.observer_id.clone(),
                    trigger: trigger.clone(),
                    stream_type: stream_type_name,
                    message_count,
                });
            }
        }
    }
}

impl<'world, T> Copy for ResourceCapability<'world, T> {}
impl<'world, T> Clone for ResourceCapability<'world, T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ResourceMutationCapability<'world> {
    next_resource_id: NonNull<u32>,
    resource_type_registry: NonNull<HashMap<TypeId, ResourceMeta>>,
    change_tick: NonNull<u64>,
    current_frame_index: NonNull<u64>,
    resource_change_ticks: NonNull<HashMap<TypeId, u64>>,
    resource_change_log: NonNull<Vec<ResourceChangeRecord>>,
    _marker: PhantomData<&'world mut World>,
}

impl<'world> ResourceMutationCapability<'world> {
    pub(crate) fn mark_modified<T: 'static>(mut self) {
        let type_id = TypeId::of::<T>();
        let name = type_name::<T>();
        let registry = unsafe { self.resource_type_registry.as_mut() };
        let key = registry
            .entry(type_id)
            .or_insert_with(|| {
                let id = unsafe { *self.next_resource_id.as_ptr() };
                unsafe { *self.next_resource_id.as_mut() = id.saturating_add(1) };
                ResourceMeta {
                    id: super::change_tracking::ResourceTypeKey(id),
                    name,
                }
            })
            .id;
        let tick = unsafe {
            let tick = self.change_tick.as_mut();
            *tick = tick.saturating_add(1);
            *tick
        };
        unsafe { self.resource_change_ticks.as_mut().insert(type_id, tick) };
        unsafe {
            self.resource_change_log
                .as_mut()
                .push(ResourceChangeRecord {
                    tick,
                    frame: *self.current_frame_index.as_ptr(),
                    resource_type: type_id,
                    resource_key: key,
                    resource_name: name,
                    kind: ResourceChangeKind::Modified,
                })
        };
    }
}

impl World {
    /// Central World-owned bridge for ordinary query extraction.
    pub(crate) fn query_capability(&self) -> QueryCapability<'_> {
        QueryCapability::from_world(self)
    }

    pub(crate) fn query_capability_mut(&mut self) -> QueryCapability<'_> {
        QueryCapability::from_world_mut(self)
    }

    pub(crate) unsafe fn resource_capability_from_ptr<'world, T: crate::component::Resource>(
        world: NonNull<World>,
        mutable: bool,
    ) -> Result<ResourceCapability<'world, T>, ResourceError> {
        let world_ptr = world.as_ptr();
        let type_id = TypeId::of::<T>();
        if mutable {
            let value = unsafe {
                (*world_ptr)
                    .resources
                    .get_mut(&type_id)
                    .and_then(|resource| resource.downcast_mut::<T>())
            }
            .ok_or(ResourceError::Missing {
                resource: type_name::<T>(),
            })?;
            let mutation = ResourceMutationCapability {
                next_resource_id: unsafe {
                    NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).next_resource_id))
                },
                resource_type_registry: unsafe {
                    NonNull::new_unchecked(std::ptr::addr_of_mut!(
                        (*world_ptr).resource_type_registry
                    ))
                },
                change_tick: unsafe {
                    NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).change_tick))
                },
                current_frame_index: unsafe {
                    NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).current_frame_index))
                },
                resource_change_ticks: unsafe {
                    NonNull::new_unchecked(std::ptr::addr_of_mut!(
                        (*world_ptr).resource_change_ticks
                    ))
                },
                resource_change_log: unsafe {
                    NonNull::new_unchecked(std::ptr::addr_of_mut!((*world_ptr).resource_change_log))
                },
                _marker: PhantomData,
            };
            Ok(ResourceCapability {
                value: NonNull::from(&mut *value),
                mutation: Some(mutation),
                _marker: PhantomData,
            })
        } else {
            let value = unsafe {
                (*world_ptr)
                    .resources
                    .get(&type_id)
                    .and_then(|resource| resource.downcast_ref::<T>())
            }
            .ok_or(ResourceError::Missing {
                resource: type_name::<T>(),
            })?;
            Ok(ResourceCapability {
                value: NonNull::from(value),
                mutation: None,
                _marker: PhantomData,
            })
        }
    }
}

impl<'world, T> ResourceCapability<'world, T> {
    pub(crate) fn value(self) -> NonNull<T> {
        self.value
    }
    pub(crate) fn mutation(self) -> Option<ResourceMutationCapability<'world>> {
        self.mutation
    }
}
