use crate::access::{AccessConflict, SystemAccess};
use crate::label::{ScheduleKey, ScheduleLabel, SystemSet, SystemSetKey};
use anyhow::Result;

pub type RunnableSystemFn<C> = Box<dyn FnMut(&mut C) -> Result<()>>;

pub struct RegisteredSystem<C> {
    name: String,
    label: ScheduleKey,
    sets: Vec<SystemSetKey>,
    access: SystemAccess,
    run: RunnableSystemFn<C>,
}

impl<C> RegisteredSystem<C> {
    pub fn new<L>(
        name: impl Into<String>,
        access: SystemAccess,
        run: impl FnMut(&mut C) -> Result<()> + 'static,
    ) -> Result<Self>
    where
        L: ScheduleLabel,
    {
        let name = name.into();
        access
            .validate_internal()
            .map_err(|conflict| internal_access_error(&name, &conflict))?;

        Ok(Self {
            name,
            label: L::key(),
            sets: Vec::new(),
            access,
            run: Box::new(run),
        })
    }

    pub fn with_set<S: SystemSet>(mut self) -> Self {
        self.sets.push(S::key());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn label(&self) -> ScheduleKey {
        self.label
    }

    pub fn sets(&self) -> &[SystemSetKey] {
        &self.sets
    }

    pub fn access(&self) -> &SystemAccess {
        &self.access
    }

    pub fn run(&mut self, ctx: &mut C) -> Result<()> {
        (self.run)(ctx)
    }
}

fn internal_access_error(system_name: &str, conflict: &AccessConflict) -> anyhow::Error {
    anyhow::anyhow!(
        "system '{system_name}' has conflicting access to '{}' ({:?})",
        conflict.key.name(),
        conflict.kind
    )
}
