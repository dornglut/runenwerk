use crate::runtime_v2::param::SystemParam;
use anyhow::{Result, anyhow};
use ecs_v2::World;
use scheduler::{RegisteredSystem, ScheduleLabel, SystemAccess};

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
                    let mut commands = ecs_v2::Commands::new();
                    $(
                        let $param = unsafe {
                            <$param as SystemParam>::get_param(
                                &mut states.$index,
                                world as *mut World,
                                &mut commands as *mut ecs_v2::Commands,
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
