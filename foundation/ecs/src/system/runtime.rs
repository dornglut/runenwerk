use super::extract::{SystemParam, SystemParamError};
use anyhow::{Result, anyhow};
use scheduler::access::{AccessKey, SystemAccess};
use scheduler::label::{ScheduleLabel, SystemSet, SystemSetKey};
use scheduler::plan::{ExecutionPlan, ExecutionScheduler};
use scheduler::system::RegisteredSystem;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;

use crate::query::QueryAccess;
use crate::telemetry;
use crate::{Commands, World};

type DeferredCommands = Rc<RefCell<Vec<Commands>>>;

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
            fn register<L: ScheduleLabel>(
                self,
                world: &mut World,
                scheduler: &mut ExecutionScheduler<World>,
                deferred_commands: DeferredCommands,
                build_errors: &mut Vec<anyhow::Error>,
            ) {
                $(
                    self.$index.register::<L>(
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

trait SystemParamState: Sized {
    type State: 'static;

    fn init_state(world: &mut World) -> std::result::Result<Self::State, SystemParamError>;
    fn access(state: &Self::State) -> QueryAccess;

    unsafe fn extract<'w>(
        state: &'w mut Self::State,
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

    unsafe fn extract<'w>(
        state: &'w mut Self::State,
        world: *mut World,
        commands: *mut Commands,
    ) -> std::result::Result<Self, SystemParamError> {
        // Safety: `SystemParam` implementors are required to keep `State` lifetime-independent.
        // This cast converts the cached `'static` state type view into the extraction lifetime view.
        let state_ptr = state as *mut Self::State as *mut <T as SystemParam<'w>>::State;
        unsafe { <T as SystemParam<'w>>::extract(&mut *state_ptr, world, commands) }
    }
}

fn merge_access(system_name: &str, access_parts: &[SystemAccess]) -> Result<SystemAccess> {
    let mut merged = SystemAccess::new();
    for access in access_parts {
        let conflicts = merged.conflicts_with(access);
        if let Some(conflict) = conflicts.first() {
            return Err(anyhow!(
                "system '{}' has conflicting param access to '{}' ({:?})",
                system_name,
                conflict.key.name(),
                conflict.kind
            ));
        }
        for read in access.reads() {
            merged.add_read(*read);
        }
        for write in access.writes() {
            merged.add_write(*write);
        }
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
            fn into_registered_system<L: ScheduleLabel>(
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
                let mut func = self;
                let deferred_commands_ref = deferred_commands.clone();

                RegisteredSystem::new::<L>(system_name, access, move |world| {
                    let mut commands = Commands::new();
                    $(
                        let $param = unsafe {
                            <$param as SystemParamState>::extract(
                                &mut states.$index,
                                world as *mut World,
                                &mut commands as *mut Commands,
                            )?
                        };
                    )*
                    let result = func($($param),*);
                    deferred_commands_ref.borrow_mut().push(commands);
                    result.into_result()
                })
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

    pub fn run_schedule<L: ScheduleLabel>(&mut self, world: &mut World) -> Result<()> {
        self.ensure_build_ready()?;
        let plan_start = Instant::now();
        let Some(plan) = self.scheduler.plan_for::<L>().cloned() else {
            telemetry::record_runtime_plan(plan_start.elapsed().as_nanos() as u64);
            return Ok(());
        };
        telemetry::record_runtime_plan(plan_start.elapsed().as_nanos() as u64);

        for stage in plan.stages {
            let stage_start = Instant::now();
            for system_index in stage.system_indices {
                let system = self
                    .scheduler
                    .systems_mut()
                    .get_mut(system_index)
                    .ok_or_else(|| anyhow!("execution plan referenced missing system"))?;
                system.run(world)?;
            }
            telemetry::record_runtime_stage(stage_start.elapsed().as_nanos() as u64);
            self.flush_stage_commands(world)?;
        }

        Ok(())
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
        let stage_commands = std::mem::take(&mut *self.deferred_commands.borrow_mut());
        let command_queue_count = stage_commands.len() as u64;
        for commands in stage_commands {
            commands.apply(world)?;
        }
        telemetry::record_runtime_flush(start.elapsed().as_nanos() as u64, command_queue_count);
        Ok(())
    }
}

fn query_access_to_system_access(access: QueryAccess) -> SystemAccess {
    let mut system_access = SystemAccess::new();
    for read in access.component_reads() {
        system_access.add_read(AccessKey::component_by_id(read.type_id(), read.name()));
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
    if access.deferred_structural_mutation() {
        system_access.add_write(AccessKey::structural("world_structure"));
    }
    system_access
}
