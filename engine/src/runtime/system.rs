use crate::runtime::param::SystemParam;
use anyhow::{Result, anyhow};
use ecs::World;
use scheduler::access::SystemAccess;
use scheduler::label::{ScheduleLabel, SystemSet, SystemSetKey};
use scheduler::plan::ExecutionScheduler;
use scheduler::system::RegisteredSystem;
use std::marker::PhantomData;

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

pub trait IntoSystem<Marker>: Send + 'static {
    fn into_registered_system<L: ScheduleLabel>(
        self,
        world: &mut World,
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
        build_errors: &mut Vec<anyhow::Error>,
    ) {
        match self.into_registered_system::<L>(world) {
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
    ) -> Result<RegisteredSystem<World>> {
        let mut registered = self.system.into_registered_system::<L>(world)?;
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
                build_errors: &mut Vec<anyhow::Error>,
            ) {
                $(
                    self.$index.register::<L>(world, scheduler, build_errors);
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
            Func: FnMut($($param),*) -> R + Send + 'static,
            R: SystemOutput,
            $($param: SystemParam,)*
        {
            fn into_registered_system<L: ScheduleLabel>(
                self,
                world: &mut World,
            ) -> Result<RegisteredSystem<World>> {
                let system_name = std::any::type_name::<Func>().to_string();
                let mut states = (
                    $(
                        <$param as SystemParam>::init_state(world)?,
                    )*
                );
                let access_parts = vec![
                    $(
                        <$param as SystemParam>::access(&states.$index),
                    )*
                ];
                let access = merge_access(&system_name, &access_parts)?;
                let mut func = self;

                RegisteredSystem::new::<L>(system_name, access, move |world| {
                    let mut commands = ecs::Commands::new();
                    $(
                        let $param = unsafe {
                            <$param as SystemParam>::get_param(
                                &mut states.$index,
                                world as *mut World,
                                &mut commands as *mut ecs::Commands,
                            )?
                        };
                    )*
                    let result = func($($param),*);
                    commands.apply(world)?;
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
