use super::extract::{SystemParam, SystemParamError};
use anyhow::{Result, anyhow};
use scheduler::access::{AccessKey, SystemAccess};
use scheduler::label::{ScheduleLabel, SystemSet, SystemSetKey};
use scheduler::plan::{BarrierKind, ExecutionBarrier, ExecutionPlan, ExecutionScheduler};
use scheduler::system::{ParamSlotDescriptor, RegisteredSystem, SystemId};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;

use crate::query::QueryAccess;
use crate::telemetry;
use crate::{Commands, World};

type DeferredCommands = Rc<RefCell<Vec<Commands>>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParamSlotId {
    pub system_id: SystemId,
    pub slot_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSlotMetadata {
    pub id: ParamSlotId,
    pub kind: &'static str,
    pub label: &'static str,
    pub type_name: &'static str,
}

pub trait SystemOutput {
    fn into_result(self) -> Result<()>;
}

impl SystemOutput for () {
    fn into_result(self) -> Result<()> {
        Ok(())
    }
}

impl<E> SystemOutput for Result<(), E>
where
    E: Into<anyhow::Error>,
{
    fn into_result(self) -> Result<()> {
        self.map_err(Into::into)
    }
}

pub trait IntoSystem<Marker>: 'static {
    fn into_registered_system<L: ScheduleLabel>(
        self,
        world: &mut World,
        deferred_commands: DeferredCommands,
    ) -> Result<RegisteredSystem<World>>;
}

pub trait IntoSystemSetKey {
    fn system_set_key(&self) -> SystemSetKey;
}

impl<S> IntoSystemSetKey for S
where
    S: SystemSet,
{
    fn system_set_key(&self) -> SystemSetKey {
        S::key()
    }
}

#[derive(Debug, Clone, Default)]
struct SystemConfigMetadata {
    sets: Vec<SystemSetKey>,
    before_sets: Vec<SystemSetKey>,
    after_sets: Vec<SystemSetKey>,
}

impl SystemConfigMetadata {
    fn with_set(&mut self, key: SystemSetKey) {
        if !self.sets.contains(&key) {
            self.sets.push(key);
        }
    }

    fn before_set(&mut self, key: SystemSetKey) {
        if !self.before_sets.contains(&key) {
            self.before_sets.push(key);
        }
    }

    fn after_set(&mut self, key: SystemSetKey) {
        if !self.after_sets.contains(&key) {
            self.after_sets.push(key);
        }
    }

    fn apply(&self, system: &mut RegisteredSystem<World>) {
        for key in &self.sets {
            system.with_set_key(*key);
        }
        for key in &self.before_sets {
            system.before_set_key(*key);
        }
        for key in &self.after_sets {
            system.after_set_key(*key);
        }
    }
}

pub struct ConfiguredSystem<S, Marker> {
    system: S,
    config: SystemConfigMetadata,
    _marker: PhantomData<fn() -> Marker>,
}

impl<S, Marker> ConfiguredSystem<S, Marker> {
    fn new(system: S) -> Self {
        Self {
            system,
            config: SystemConfigMetadata::default(),
            _marker: PhantomData,
        }
    }

    pub fn in_set<Set>(mut self, set: Set) -> Self
    where
        Set: IntoSystemSetKey,
    {
        self.config.with_set(set.system_set_key());
        self
    }

    pub fn before<Set>(mut self, set: Set) -> Self
    where
        Set: IntoSystemSetKey,
    {
        self.config.before_set(set.system_set_key());
        self
    }

    pub fn after<Set>(mut self, set: Set) -> Self
    where
        Set: IntoSystemSetKey,
    {
        self.config.after_set(set.system_set_key());
        self
    }
}

pub trait SystemConfigExt<Marker>: IntoSystem<Marker> + Sized {
    fn in_set<Set>(self, set: Set) -> ConfiguredSystem<Self, Marker>
    where
        Set: IntoSystemSetKey,
    {
        ConfiguredSystem::new(self).in_set(set)
    }

    fn before<Set>(self, set: Set) -> ConfiguredSystem<Self, Marker>
    where
        Set: IntoSystemSetKey,
    {
        ConfiguredSystem::new(self).before(set)
    }

    fn after<Set>(self, set: Set) -> ConfiguredSystem<Self, Marker>
    where
        Set: IntoSystemSetKey,
    {
        ConfiguredSystem::new(self).after(set)
    }
}

impl<S, Marker> SystemConfigExt<Marker> for S where S: IntoSystem<Marker> + Sized {}

pub trait IntoSystemConfigs<Marker> {
    fn register<L: ScheduleLabel>(
        self,
        world: &mut World,
        scheduler: &mut ExecutionScheduler<World>,
        deferred_commands: DeferredCommands,
        build_errors: &mut Vec<anyhow::Error>,
    );
}

impl<S, Marker> IntoSystemConfigs<Marker> for S
where
    S: IntoSystem<Marker>,
{
    fn register<L: ScheduleLabel>(
        self,
        world: &mut World,
        scheduler: &mut ExecutionScheduler<World>,
        deferred_commands: DeferredCommands,
        build_errors: &mut Vec<anyhow::Error>,
    ) {
        match self.into_registered_system::<L>(world, deferred_commands) {
            Ok(registered) => {
                scheduler.add_system(registered);
            }
            Err(err) => build_errors.push(err),
        }
    }
}

impl<S, Marker> IntoSystem<Marker> for ConfiguredSystem<S, Marker>
where
    S: IntoSystem<Marker>,
    Marker: 'static,
{
    fn into_registered_system<L: ScheduleLabel>(
        self,
        world: &mut World,
        deferred_commands: DeferredCommands,
    ) -> Result<RegisteredSystem<World>> {
        let mut registered = self
            .system
            .into_registered_system::<L>(world, deferred_commands)?;
        self.config.apply(&mut registered);
        Ok(registered)
    }
}

macro_rules! impl_into_system_configs_tuple {
    ($(($name:ident, $marker:ident, $index:tt)),+ $(,)?) => {
        impl<$($name, $marker,)+> IntoSystemConfigs<($($marker,)+)> for ($($name,)+)
        where
            $($name: IntoSystemConfigs<$marker>,)+
        {
            fn register<Sched: ScheduleLabel>(
                self,
                world: &mut World,
                scheduler: &mut ExecutionScheduler<World>,
                deferred_commands: DeferredCommands,
                build_errors: &mut Vec<anyhow::Error>,
            ) {
                $(
                    self.$index.register::<Sched>(
                        world,
                        scheduler,
                        deferred_commands.clone(),
                        build_errors,
                    );
                )+
            }
        }
    };
}

impl_into_system_configs_tuple!((A, AMarker, 0), (B, BMarker, 1));
impl_into_system_configs_tuple!((A, AMarker, 0), (B, BMarker, 1), (C, CMarker, 2));
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9),
    (K, KMarker, 10)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9),
    (K, KMarker, 10),
    (L, LMarker, 11)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9),
    (K, KMarker, 10),
    (L, LMarker, 11),
    (M, MMarker, 12)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9),
    (K, KMarker, 10),
    (L, LMarker, 11),
    (M, MMarker, 12),
    (N, NMarker, 13)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9),
    (K, KMarker, 10),
    (L, LMarker, 11),
    (M, MMarker, 12),
    (N, NMarker, 13),
    (O, OMarker, 14)
);
impl_into_system_configs_tuple!(
    (A, AMarker, 0),
    (B, BMarker, 1),
    (C, CMarker, 2),
    (D, DMarker, 3),
    (E, EMarker, 4),
    (F, FMarker, 5),
    (G, GMarker, 6),
    (H, HMarker, 7),
    (I, IMarker, 8),
    (J, JMarker, 9),
    (K, KMarker, 10),
    (L, LMarker, 11),
    (M, MMarker, 12),
    (N, NMarker, 13),
    (O, OMarker, 14),
    (P, PMarker, 15)
);

trait SystemParamState: Sized {
    type State: 'static;

    fn init_state(world: &mut World) -> std::result::Result<Self::State, SystemParamError>;
    fn access(state: &Self::State) -> QueryAccess;
    fn slot_descriptor() -> ParamSlotDescriptor;

    unsafe fn extract(
        state: &mut Self::State,
        world: *mut World,
        commands: *mut Commands,
    ) -> std::result::Result<Self, SystemParamError>;
}

impl<T> SystemParamState for T
where
    T: for<'w> SystemParam<'w>,
{
    type State = <T as SystemParam<'static>>::State;

    fn init_state(world: &mut World) -> std::result::Result<Self::State, SystemParamError> {
        <T as SystemParam<'static>>::init_state(world)
    }

    fn access(state: &Self::State) -> QueryAccess {
        <T as SystemParam<'static>>::access(state)
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        <T as SystemParam<'static>>::slot_descriptor()
    }

    unsafe fn extract(
        state: &mut Self::State,
        world: *mut World,
        commands: *mut Commands,
    ) -> std::result::Result<Self, SystemParamError> {
        // Safety: `SystemParam` implementors are required to keep `State` lifetime-independent.
        // This cast converts the cached `'static` state type view into the extraction lifetime view.
        let state_ptr = state as *mut Self::State as *mut <T as SystemParam<'_>>::State;
        unsafe { <T as SystemParam<'_>>::extract(&mut *state_ptr, world, commands) }
    }
}

fn merge_access(system_name: &str, access_parts: &[SystemAccess]) -> Result<SystemAccess> {
    let mut merged = SystemAccess::new();
    for access in access_parts {
        for read in access.reads() {
            merged.add_read(*read);
        }
        for write in access.writes() {
            merged.add_write(*write);
        }
        for drain in access.drains() {
            merged.add_drain(*drain);
        }
    }
    if let Err(conflict) = merged.validate_internal() {
        return Err(anyhow!(
            "system '{}' has conflicting param access: {}",
            system_name,
            conflict.diagnostic_message()
        ));
    }
    Ok(merged)
}

macro_rules! impl_into_system {
    ($(($index:tt, $param:ident)),* $(,)?) => {
        #[allow(unused_mut, unused_variables, non_snake_case)]
        impl<Func, R, $($param),*> IntoSystem<fn($($param),*) -> R> for Func
        where
            Func: FnMut($($param),*) -> R + 'static,
            R: SystemOutput,
            $($param: SystemParamState,)*
        {
            fn into_registered_system<Sched: ScheduleLabel>(
                self,
                world: &mut World,
                deferred_commands: DeferredCommands,
            ) -> Result<RegisteredSystem<World>> {
                let system_name = std::any::type_name::<Func>().to_string();
                let mut states = (
                    $(
                        <$param as SystemParamState>::init_state(world)?,
                    )*
                );
                let access_parts = vec![
                    $(
                        query_access_to_system_access(<$param as SystemParamState>::access(&states.$index)),
                    )*
                ];
                let access = merge_access(&system_name, &access_parts)?;
                let param_slots = vec![
                    $(
                        <$param as SystemParamState>::slot_descriptor(),
                    )*
                ];
                let mut func = self;
                let deferred_commands_ref = deferred_commands.clone();

                let mut registered = RegisteredSystem::new::<Sched>(system_name, access, move |world| {
                    let mut commands = Commands::new_external_owner();
                    $(
                        let $param = unsafe {
                            <$param as SystemParamState>::extract(
                                &mut states.$index,
                                world as *mut World,
                                &mut commands as *mut Commands,
                            )?
                        };
                    )*
                    let result = func($($param),*).into_result();
                    let staged_commands = commands.finalize_external_owner();
                    if result.is_ok() {
                        deferred_commands_ref.borrow_mut().push(staged_commands);
                    }
                    result
                })?;
                registered.set_param_slots(param_slots);
                Ok(registered)
            }
        }
    };
}

impl_into_system!();
impl_into_system!((0, A));
impl_into_system!((0, A), (1, B));
impl_into_system!((0, A), (1, B), (2, C));
impl_into_system!((0, A), (1, B), (2, C), (3, D));
impl_into_system!((0, A), (1, B), (2, C), (3, D), (4, E));
impl_into_system!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
impl_into_system!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J),
    (10, K)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J),
    (10, K),
    (11, L)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J),
    (10, K),
    (11, L),
    (12, M)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J),
    (10, K),
    (11, L),
    (12, M),
    (13, N)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J),
    (10, K),
    (11, L),
    (12, M),
    (13, N),
    (14, O)
);
impl_into_system!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H),
    (8, I),
    (9, J),
    (10, K),
    (11, L),
    (12, M),
    (13, N),
    (14, O),
    (15, P)
);

pub struct Runtime {
    scheduler: ExecutionScheduler<World>,
    deferred_commands: DeferredCommands,
    build_errors: Vec<anyhow::Error>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            scheduler: ExecutionScheduler::new(),
            deferred_commands: Rc::new(RefCell::new(Vec::new())),
            build_errors: Vec::new(),
        }
    }

    pub fn add_systems<L, S, Marker>(&mut self, world: &mut World, systems: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystemConfigs<Marker>,
    {
        systems.register::<L>(
            world,
            &mut self.scheduler,
            self.deferred_commands.clone(),
            &mut self.build_errors,
        );
        self
    }

    pub fn plan_for<L: ScheduleLabel>(&mut self) -> Option<&ExecutionPlan> {
        self.scheduler.plan_for::<L>()
    }

    pub fn scheduler(&mut self) -> &mut ExecutionScheduler<World> {
        &mut self.scheduler
    }

    pub fn param_slots_for_system(&self, system_id: SystemId) -> Option<Vec<ParamSlotMetadata>> {
        let system = self
            .scheduler
            .systems()
            .iter()
            .find(|system| system.id() == system_id)?;

        Some(
            system
                .param_slots()
                .iter()
                .enumerate()
                .map(|(slot_index, slot)| ParamSlotMetadata {
                    id: ParamSlotId {
                        system_id,
                        slot_index,
                    },
                    kind: slot.kind,
                    label: slot.label,
                    type_name: slot.type_name,
                })
                .collect(),
        )
    }

    pub fn run_schedule<L: ScheduleLabel>(&mut self, world: &mut World) -> Result<()> {
        if let Err(err) = self.ensure_build_ready() {
            self.discard_deferred_commands();
            return Err(err);
        }
        let plan_start = Instant::now();
        let Some(plan) = self.scheduler.plan_for::<L>().cloned() else {
            telemetry::record_runtime_plan(plan_start.elapsed().as_nanos() as u64);
            return Ok(());
        };
        telemetry::record_runtime_plan(plan_start.elapsed().as_nanos() as u64);

        for wave in &plan.waves {
            let stage_start = Instant::now();
            for system_index in &wave.system_indices {
                let Some(system) = self.scheduler.systems_mut().get_mut(*system_index) else {
                    self.discard_deferred_commands();
                    return Err(anyhow!("execution plan referenced missing system"));
                };
                if let Err(err) = system.run(world) {
                    self.discard_deferred_commands();
                    return Err(err);
                }
            }
            telemetry::record_runtime_stage(stage_start.elapsed().as_nanos() as u64);
            for barrier in plan.barriers_after_wave(wave.index) {
                if let Err(err) = self.execute_barrier(barrier, world) {
                    self.discard_deferred_commands();
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    fn execute_barrier(&self, barrier: &ExecutionBarrier, world: &mut World) -> Result<()> {
        match barrier.kind {
            BarrierKind::ApplyDeferredCommands => self.flush_stage_commands(world),
            BarrierKind::ProductPublication
            | BarrierKind::QuerySnapshotPublication
            | BarrierKind::RenderSubmit
            | BarrierKind::GenerationFinalization
            | BarrierKind::ReplayNetworkCapture
            | BarrierKind::Custom(_) => Ok(()),
        }
    }

    fn ensure_build_ready(&self) -> Result<()> {
        if self.build_errors.is_empty() {
            return Ok(());
        }
        let messages: Vec<_> = self.build_errors.iter().map(ToString::to_string).collect();
        Err(anyhow!("runtime setup failed:\n{}", messages.join("\n")))
    }

    fn flush_stage_commands(&self, world: &mut World) -> Result<()> {
        let start = Instant::now();
        world.begin_stage_command_flush();
        let stage_commands = std::mem::take(&mut *self.deferred_commands.borrow_mut());
        let command_queue_count = stage_commands.len() as u64;
        for commands in stage_commands {
            commands.apply(world)?;
        }
        telemetry::record_runtime_flush(start.elapsed().as_nanos() as u64, command_queue_count);
        Ok(())
    }

    fn discard_deferred_commands(&self) {
        self.deferred_commands.borrow_mut().clear();
    }
}

fn query_access_to_system_access(access: QueryAccess) -> SystemAccess {
    let mut system_access = SystemAccess::new();
    for read in access.component_reads() {
        system_access.add_read(AccessKey::component_by_id(read.type_id(), read.name()));
    }
    for read in access.orphaned_component_reads() {
        system_access.add_read(AccessKey::orphaned_component_by_id(
            read.type_id(),
            read.name(),
        ));
    }
    for write in access.component_writes() {
        system_access.add_write(AccessKey::component_by_id(write.type_id(), write.name()));
    }
    for read in access.resource_reads() {
        system_access.add_read(AccessKey::resource_by_id(read.type_id(), read.name()));
    }
    for write in access.resource_writes() {
        system_access.add_write(AccessKey::resource_by_id(write.type_id(), write.name()));
    }
    for read in access.broadcast_reads() {
        system_access.add_read(AccessKey::broadcast_stream_by_id(
            read.type_id(),
            read.name(),
        ));
    }
    for write in access.broadcast_writes() {
        system_access.add_write(AccessKey::broadcast_stream_by_id(
            write.type_id(),
            write.name(),
        ));
    }
    for read in access.work_queue_reads() {
        system_access.add_read(AccessKey::work_queue_by_id(read.type_id(), read.name()));
    }
    for write in access.work_queue_writes() {
        system_access.add_write(AccessKey::work_queue_by_id(write.type_id(), write.name()));
    }
    for drain in access.work_queue_drains() {
        system_access.add_drain(AccessKey::work_queue_by_id(drain.type_id(), drain.name()));
    }
    for read in access.tick_buffer_reads() {
        system_access.add_read(AccessKey::tick_buffer_by_id(read.type_id(), read.name()));
    }
    for write in access.tick_buffer_writes() {
        system_access.add_write(AccessKey::tick_buffer_by_id(write.type_id(), write.name()));
    }
    for drain in access.tick_buffer_drains() {
        system_access.add_drain(AccessKey::tick_buffer_by_id(drain.type_id(), drain.name()));
    }
    if access.deferred_structural_mutation() {
        system_access.add_write(AccessKey::structural("world_structure"));
    }
    system_access
}

#[cfg(test)]
mod tests {
    use super::Runtime;
    use crate::{Res, ResMut, Resource, World};
    use scheduler::label::ScheduleLabel;

    macro_rules! define_u32_resource {
        ($name:ident) => {
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            struct $name(pub u32);
            impl Resource for $name {}
        };
    }

    define_u32_resource!(R0);
    define_u32_resource!(R1);
    define_u32_resource!(R2);
    define_u32_resource!(R3);
    define_u32_resource!(R4);
    define_u32_resource!(R5);
    define_u32_resource!(R6);
    define_u32_resource!(R7);
    define_u32_resource!(R8);
    define_u32_resource!(R9);
    define_u32_resource!(R10);
    define_u32_resource!(R11);
    define_u32_resource!(R12);
    define_u32_resource!(R13);
    define_u32_resource!(R14);

    define_u32_resource!(Sum);
    define_u32_resource!(Counter);

    #[derive(Debug, Copy, Clone)]
    struct MaxAritySchedule;
    impl ScheduleLabel for MaxAritySchedule {}

    #[derive(Debug, Copy, Clone)]
    struct MaxTupleSchedule;
    impl ScheduleLabel for MaxTupleSchedule {}

    #[allow(clippy::too_many_arguments)]
    fn max_arity_system(
        r0: Res<R0>,
        r1: Res<R1>,
        r2: Res<R2>,
        r3: Res<R3>,
        r4: Res<R4>,
        r5: Res<R5>,
        r6: Res<R6>,
        r7: Res<R7>,
        r8: Res<R8>,
        r9: Res<R9>,
        r10: Res<R10>,
        r11: Res<R11>,
        r12: Res<R12>,
        r13: Res<R13>,
        r14: Res<R14>,
        mut sum: ResMut<Sum>,
    ) {
        sum.0 = r0.0
            + r1.0
            + r2.0
            + r3.0
            + r4.0
            + r5.0
            + r6.0
            + r7.0
            + r8.0
            + r9.0
            + r10.0
            + r11.0
            + r12.0
            + r13.0
            + r14.0;
    }

    fn bump_counter(mut counter: ResMut<Counter>) {
        counter.0 = counter.0.saturating_add(1);
    }

    #[test]
    fn supports_max_function_system_arity_sixteen() {
        let mut world = World::new();
        world.insert_resource(R0(1));
        world.insert_resource(R1(2));
        world.insert_resource(R2(3));
        world.insert_resource(R3(4));
        world.insert_resource(R4(5));
        world.insert_resource(R5(6));
        world.insert_resource(R6(7));
        world.insert_resource(R7(8));
        world.insert_resource(R8(9));
        world.insert_resource(R9(10));
        world.insert_resource(R10(11));
        world.insert_resource(R11(12));
        world.insert_resource(R12(13));
        world.insert_resource(R13(14));
        world.insert_resource(R14(15));
        world.insert_resource(Sum(0));

        let mut runtime = Runtime::new();
        runtime.add_systems::<MaxAritySchedule, _, _>(&mut world, max_arity_system);
        runtime
            .run_schedule::<MaxAritySchedule>(&mut world)
            .expect("max-arity system should register and execute");

        let actual = world
            .resource::<Sum>()
            .expect("sum resource should exist after schedule")
            .0;
        assert_eq!(actual, 120);
    }

    #[test]
    fn supports_max_tuple_registration_arity_sixteen() {
        let mut world = World::new();
        world.insert_resource(Counter(0));

        let mut runtime = Runtime::new();
        runtime.add_systems::<MaxTupleSchedule, _, _>(
            &mut world,
            (
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
                bump_counter,
            ),
        );
        runtime
            .run_schedule::<MaxTupleSchedule>(&mut world)
            .expect("max-tuple system registration should execute");

        let actual = world
            .resource::<Counter>()
            .expect("counter resource should exist after schedule")
            .0;
        assert_eq!(actual, 16);
    }
}
