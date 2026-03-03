use crate::access::AccessConflict;
use crate::label::{ScheduleKey, ScheduleLabel};
use crate::system::RegisteredSystem;
use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionConflict {
    pub first_system: String,
    pub second_system: String,
    pub conflict: AccessConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionStage {
    pub index: usize,
    pub system_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub label: ScheduleKey,
    pub stages: Vec<ExecutionStage>,
    pub conflicts: Vec<ExecutionConflict>,
}

pub struct ExecutionScheduler<C> {
    systems: Vec<RegisteredSystem<C>>,
    plans: Vec<ExecutionPlan>,
    dirty: bool,
}

impl<C> Default for ExecutionScheduler<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ExecutionScheduler<C> {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            plans: Vec::new(),
            dirty: true,
        }
    }

    pub fn add_system(&mut self, system: RegisteredSystem<C>) -> usize {
        let index = self.systems.len();
        self.systems.push(system);
        self.dirty = true;
        index
    }

    pub fn systems(&self) -> &[RegisteredSystem<C>] {
        &self.systems
    }

    pub fn plans(&mut self) -> &[ExecutionPlan] {
        self.rebuild_if_dirty()
            .expect("execution plan rebuild failed");
        &self.plans
    }

    pub fn plan_for<L: ScheduleLabel>(&mut self) -> Option<&ExecutionPlan> {
        self.rebuild_if_dirty().ok()?;
        self.plans.iter().find(|plan| plan.label == L::key())
    }

    pub fn run_schedule<L: ScheduleLabel>(&mut self, ctx: &mut C) -> Result<()> {
        self.rebuild_if_dirty()?;
        let Some(plan) = self
            .plans
            .iter()
            .find(|plan| plan.label == L::key())
            .cloned()
        else {
            return Ok(());
        };

        for stage in plan.stages {
            for system_index in stage.system_indices {
                let system = self
                    .systems
                    .get_mut(system_index)
                    .context("execution plan referenced missing system")?;
                system
                    .run(ctx)
                    .with_context(|| format!("failed to run system '{}'", system.name()))?;
            }
        }

        Ok(())
    }

    fn rebuild_if_dirty(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let mut labels: Vec<ScheduleKey> = Vec::new();
        for system in &self.systems {
            if !labels.iter().any(|label| *label == system.label()) {
                labels.push(system.label());
            }
        }

        self.plans = labels
            .into_iter()
            .map(|label| self.build_plan(label))
            .collect();
        self.dirty = false;
        Ok(())
    }

    fn build_plan(&self, label: ScheduleKey) -> ExecutionPlan {
        let scheduled_indices: Vec<_> = self
            .systems
            .iter()
            .enumerate()
            .filter_map(|(index, system)| (system.label() == label).then_some(index))
            .collect();

        let stages = scheduled_indices
            .iter()
            .enumerate()
            .map(|(stage_index, system_index)| ExecutionStage {
                index: stage_index,
                system_indices: vec![*system_index],
            })
            .collect();

        let mut conflicts = Vec::new();
        for (left_pos, left_index) in scheduled_indices.iter().enumerate() {
            let left = &self.systems[*left_index];
            for right_index in scheduled_indices.iter().skip(left_pos + 1) {
                let right = &self.systems[*right_index];
                for conflict in left.access().conflicts_with(right.access()) {
                    conflicts.push(ExecutionConflict {
                        first_system: left.name().to_string(),
                        second_system: right.name().to_string(),
                        conflict,
                    });
                }
            }
        }

        ExecutionPlan {
            label,
            stages,
            conflicts,
        }
    }
}
