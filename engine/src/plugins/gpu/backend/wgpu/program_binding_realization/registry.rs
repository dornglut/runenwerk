//! Private ready-plus-in-flight single-flight registries for G4C2 authority.

use super::records::{
    BindGroupLayoutRealizationRecord, BindGroupRealizationRecord, PipelineLayoutRealizationRecord,
    ProgramRealizationRecord, StaticBindGroupValue, static_bind_group_values,
};
use crate::plugins::gpu::{
    GpuBindGroupLayoutDescriptor, GpuContextAffinity, GpuPipelineLayoutDescriptor,
    GpuProgramBindingRealizationError, GpuProgramBindingRealizationPolicy,
    GpuProgramBindingRealizationStats, GpuProgramDescriptor, GpuProgramSourceDigest,
    GpuRuntimeBindingValue,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

const G4C2_WGPU_REALIZATION_COMPATIBILITY_REVISION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ProgramRequestKey {
    affinity: GpuContextAffinity,
    descriptor: GpuProgramDescriptor,
    source_digest: GpuProgramSourceDigest,
    wgpu_realization_compatibility_revision: u32,
}

impl ProgramRequestKey {
    pub(super) fn new(affinity: GpuContextAffinity, descriptor: GpuProgramDescriptor) -> Self {
        Self {
            affinity,
            source_digest: descriptor.source().digest(),
            descriptor,
            wgpu_realization_compatibility_revision: G4C2_WGPU_REALIZATION_COMPATIBILITY_REVISION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct BindGroupLayoutRequestKey {
    affinity: GpuContextAffinity,
    descriptor: GpuBindGroupLayoutDescriptor,
}

impl BindGroupLayoutRequestKey {
    pub(super) fn new(
        affinity: GpuContextAffinity,
        descriptor: GpuBindGroupLayoutDescriptor,
    ) -> Self {
        Self {
            affinity,
            descriptor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PipelineLayoutRequestKey {
    affinity: GpuContextAffinity,
    descriptor: GpuPipelineLayoutDescriptor,
}

impl PipelineLayoutRequestKey {
    pub(super) fn new(
        affinity: GpuContextAffinity,
        descriptor: GpuPipelineLayoutDescriptor,
    ) -> Self {
        Self {
            affinity,
            descriptor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct BindGroupRequestKey {
    affinity: GpuContextAffinity,
    layout: GpuBindGroupLayoutDescriptor,
    static_values: Vec<StaticBindGroupValue>,
}

impl BindGroupRequestKey {
    pub(super) fn new(
        affinity: GpuContextAffinity,
        layout: GpuBindGroupLayoutDescriptor,
        values: Vec<GpuRuntimeBindingValue>,
    ) -> Self {
        Self {
            affinity,
            layout,
            static_values: static_bind_group_values(values),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum InFlightOutcome<Record> {
    Pending,
    Complete(Result<Arc<Record>, GpuProgramBindingRealizationError>),
    Abandoned,
}

#[derive(Debug)]
struct InFlightAttempt<Record> {
    outcome: Mutex<InFlightOutcome<Record>>,
    notify: Notify,
}

impl<Record> InFlightAttempt<Record> {
    fn new() -> Self {
        Self {
            outcome: Mutex::new(InFlightOutcome::Pending),
            notify: Notify::new(),
        }
    }

    fn finish(&self, outcome: Result<Arc<Record>, GpuProgramBindingRealizationError>) {
        *self
            .outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            InFlightOutcome::Complete(outcome);
        self.notify.notify_waiters();
    }

    fn abandon(&self) {
        *self
            .outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = InFlightOutcome::Abandoned;
        self.notify.notify_waiters();
    }

    async fn wait(&self) -> InFlightOutcome<Record> {
        loop {
            let notified = self.notify.notified();
            let outcome = self
                .outcome
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            match outcome {
                InFlightOutcome::Pending => notified.await,
                terminal => return terminal,
            }
        }
    }
}

pub(super) struct ReservationOwner<Key, Record> {
    registries: Arc<Mutex<ProgramBindingRegistries>>,
    key: Option<Key>,
    attempt: Arc<InFlightAttempt<Record>>,
    family: RegistryFamily,
}

impl<Key, Record> ReservationOwner<Key, Record>
where
    Key: Eq + Hash + Clone + Send + 'static,
    Record: Send + Sync + 'static,
    ProgramBindingRegistries: RegistryAccess<Key, Record>,
{
    pub(super) fn finish(
        mut self,
        outcome: Result<Arc<Record>, GpuProgramBindingRealizationError>,
    ) -> Result<Arc<Record>, GpuProgramBindingRealizationError> {
        let key = self
            .key
            .take()
            .expect("single-flight owner retains one key until finish");
        {
            let mut registries = self
                .registries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            <ProgramBindingRegistries as RegistryAccess<Key, Record>>::finish_owner(
                &mut registries,
                &key,
                &self.attempt,
                outcome.as_ref().ok(),
                self.family,
            );
        }
        self.attempt.finish(outcome.clone());
        outcome
    }
}

impl<Key, Record> Drop for ReservationOwner<Key, Record>
where
    Key: Eq + Hash + Clone + Send + 'static,
    Record: Send + Sync + 'static,
    ProgramBindingRegistries: RegistryAccess<Key, Record>,
{
    fn drop(&mut self) {
        let Some(key) = self.key.take() else {
            return;
        };
        let mut registries = self
            .registries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        <ProgramBindingRegistries as RegistryAccess<Key, Record>>::abandon_owner(
            &mut registries,
            &key,
            &self.attempt,
        );
        drop(registries);
        self.attempt.abandon();
    }
}

pub(super) enum Reservation<Key, Record> {
    Ready(Arc<Record>),
    Waiter(Arc<InFlightAttempt<Record>>),
    Owner(ReservationOwner<Key, Record>),
}

#[derive(Debug, Clone, Copy)]
enum RegistryFamily {
    Program,
    BindGroupLayout,
    PipelineLayout,
    BindGroup,
}

#[derive(Debug, Default)]
pub(super) struct ProgramBindingRegistries {
    programs: Registry<ProgramRequestKey, ProgramRealizationRecord>,
    bind_group_layouts: Registry<BindGroupLayoutRequestKey, BindGroupLayoutRealizationRecord>,
    pipeline_layouts: Registry<PipelineLayoutRequestKey, PipelineLayoutRealizationRecord>,
    bind_groups: Registry<BindGroupRequestKey, BindGroupRealizationRecord>,
}

impl ProgramBindingRegistries {
    pub(super) fn reserve_program(
        registries: &Arc<Mutex<Self>>,
        policy: GpuProgramBindingRealizationPolicy,
        key: ProgramRequestKey,
        request: String,
    ) -> Result<Reservation<ProgramRequestKey, ProgramRealizationRecord>, GpuProgramBindingRealizationError>
    {
        Self::reserve(registries, policy.max_programs(), key, request, RegistryFamily::Program)
    }

    pub(super) fn reserve_bind_group_layout(
        registries: &Arc<Mutex<Self>>,
        policy: GpuProgramBindingRealizationPolicy,
        key: BindGroupLayoutRequestKey,
        request: String,
    ) -> Result<
        Reservation<BindGroupLayoutRequestKey, BindGroupLayoutRealizationRecord>,
        GpuProgramBindingRealizationError,
    > {
        Self::reserve(
            registries,
            policy.max_bind_group_layouts(),
            key,
            request,
            RegistryFamily::BindGroupLayout,
        )
    }

    pub(super) fn reserve_pipeline_layout(
        registries: &Arc<Mutex<Self>>,
        policy: GpuProgramBindingRealizationPolicy,
        key: PipelineLayoutRequestKey,
        request: String,
    ) -> Result<
        Reservation<PipelineLayoutRequestKey, PipelineLayoutRealizationRecord>,
        GpuProgramBindingRealizationError,
    > {
        Self::reserve(
            registries,
            policy.max_pipeline_layouts(),
            key,
            request,
            RegistryFamily::PipelineLayout,
        )
    }

    pub(super) fn reserve_bind_group(
        registries: &Arc<Mutex<Self>>,
        policy: GpuProgramBindingRealizationPolicy,
        key: BindGroupRequestKey,
        request: String,
    ) -> Result<Reservation<BindGroupRequestKey, BindGroupRealizationRecord>, GpuProgramBindingRealizationError>
    {
        Self::reserve(registries, policy.max_bind_groups(), key, request, RegistryFamily::BindGroup)
    }

    fn reserve<Key, Record>(
        registries: &Arc<Mutex<Self>>,
        capacity: usize,
        key: Key,
        request: String,
        family: RegistryFamily,
    ) -> Result<Reservation<Key, Record>, GpuProgramBindingRealizationError>
    where
        Key: Eq + Hash + Clone + Send + 'static,
        Record: Send + Sync + 'static,
        Self: RegistryAccess<Key, Record>,
    {
        let mut registries = registries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registry = <Self as RegistryAccess<Key, Record>>::registry(&mut registries);
        if let Some(record) = registry.ready.get(&key) {
            return Ok(Reservation::Ready(Arc::clone(record)));
        }
        if let Some(attempt) = registry.in_flight.get(&key) {
            return Ok(Reservation::Waiter(Arc::clone(attempt)));
        }
        if registry.ready.len() + registry.in_flight.len() >= capacity {
            return Err(GpuProgramBindingRealizationError::capacity(request));
        }
        let attempt = Arc::new(InFlightAttempt::new());
        registry.in_flight.insert(key.clone(), Arc::clone(&attempt));
        Ok(Reservation::Owner(ReservationOwner {
            registries: Arc::clone(registries),
            key: Some(key),
            attempt,
            family,
        }))
    }

    pub(super) fn contains_program(&self, record: &Arc<ProgramRealizationRecord>) -> bool {
        self.programs
            .ready
            .values()
            .any(|candidate| Arc::ptr_eq(candidate, record))
    }

    pub(super) fn contains_pipeline_layout(
        &self,
        record: &Arc<PipelineLayoutRealizationRecord>,
    ) -> bool {
        self.pipeline_layouts
            .ready
            .values()
            .any(|candidate| Arc::ptr_eq(candidate, record))
    }

    pub(super) fn contains_bind_group(&self, record: &Arc<BindGroupRealizationRecord>) -> bool {
        self.bind_groups
            .ready
            .values()
            .any(|candidate| Arc::ptr_eq(candidate, record))
    }

    pub(super) fn stats(
        &self,
        policy: GpuProgramBindingRealizationPolicy,
    ) -> GpuProgramBindingRealizationStats {
        GpuProgramBindingRealizationStats::new(
            self.programs.ready.len(),
            self.programs.in_flight.len(),
            policy.max_programs(),
            self.bind_group_layouts.ready.len(),
            self.bind_group_layouts.in_flight.len(),
            policy.max_bind_group_layouts(),
            self.pipeline_layouts.ready.len(),
            self.pipeline_layouts.in_flight.len(),
            policy.max_pipeline_layouts(),
            self.bind_groups.ready.len(),
            self.bind_groups.in_flight.len(),
            policy.max_bind_groups(),
        )
    }
}

#[derive(Debug, Default)]
struct Registry<Key, Record> {
    ready: HashMap<Key, Arc<Record>>,
    in_flight: HashMap<Key, Arc<InFlightAttempt<Record>>>,
}

trait RegistryAccess<Key, Record> {
    fn registry(&mut self) -> &mut Registry<Key, Record>;

    fn finish_owner(
        &mut self,
        key: &Key,
        attempt: &Arc<InFlightAttempt<Record>>,
        record: Option<&Arc<Record>>,
        _family: RegistryFamily,
    );

    fn abandon_owner(&mut self, key: &Key, attempt: &Arc<InFlightAttempt<Record>>);
}

macro_rules! impl_registry_access {
    ($key:ty, $record:ty, $field:ident) => {
        impl RegistryAccess<$key, $record> for ProgramBindingRegistries {
            fn registry(&mut self) -> &mut Registry<$key, $record> {
                &mut self.$field
            }

            fn finish_owner(
                &mut self,
                key: &$key,
                attempt: &Arc<InFlightAttempt<$record>>,
                record: Option<&Arc<$record>>,
                _family: RegistryFamily,
            ) {
                let registry = &mut self.$field;
                let Some(current) = registry.in_flight.get(key) else {
                    return;
                };
                if !Arc::ptr_eq(current, attempt) {
                    return;
                }
                registry.in_flight.remove(key);
                if let Some(record) = record {
                    registry.ready.insert(key.clone(), Arc::clone(record));
                }
            }

            fn abandon_owner(
                &mut self,
                key: &$key,
                attempt: &Arc<InFlightAttempt<$record>>,
            ) {
                let registry = &mut self.$field;
                if registry
                    .in_flight
                    .get(key)
                    .is_some_and(|current| Arc::ptr_eq(current, attempt))
                {
                    registry.in_flight.remove(key);
                }
            }
        }
    };
}

impl_registry_access!(ProgramRequestKey, ProgramRealizationRecord, programs);
impl_registry_access!(
    BindGroupLayoutRequestKey,
    BindGroupLayoutRealizationRecord,
    bind_group_layouts
);
impl_registry_access!(
    PipelineLayoutRequestKey,
    PipelineLayoutRealizationRecord,
    pipeline_layouts
);
impl_registry_access!(BindGroupRequestKey, BindGroupRealizationRecord, bind_groups);
