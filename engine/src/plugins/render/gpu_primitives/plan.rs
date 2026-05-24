use super::{
    CounterResetDescriptor, GpuPrimitiveValidationError, IndirectDrawArgsGenerationDescriptor,
    U32PrefixScanDescriptor, U32ScatterDescriptor,
};
use crate::plugins::render::RenderResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPrimitiveResourceAccessKind {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuPrimitiveResourceAccess {
    pub resource_id: RenderResourceId,
    pub kind: GpuPrimitiveResourceAccessKind,
}

impl GpuPrimitiveResourceAccess {
    pub const fn read(resource_id: RenderResourceId) -> Self {
        Self {
            resource_id,
            kind: GpuPrimitiveResourceAccessKind::Read,
        }
    }

    pub const fn write(resource_id: RenderResourceId) -> Self {
        Self {
            resource_id,
            kind: GpuPrimitiveResourceAccessKind::Write,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuPrimitiveStep {
    CounterReset(CounterResetDescriptor),
    U32PrefixScan(U32PrefixScanDescriptor),
    U32Scatter(U32ScatterDescriptor),
    IndirectDrawArgs(IndirectDrawArgsGenerationDescriptor),
}

impl GpuPrimitiveStep {
    pub fn label(&self) -> &str {
        match self {
            Self::CounterReset(step) => step.label.as_str(),
            Self::U32PrefixScan(step) => step.label.as_str(),
            Self::U32Scatter(step) => step.label.as_str(),
            Self::IndirectDrawArgs(step) => step.label.as_str(),
        }
    }

    pub fn validate(&self) -> Result<(), GpuPrimitiveValidationError> {
        match self {
            Self::CounterReset(step) => step.validate(),
            Self::U32PrefixScan(step) => step.validate(),
            Self::U32Scatter(step) => step.validate(),
            Self::IndirectDrawArgs(step) => step.validate(),
        }
    }

    pub fn resource_accesses(&self) -> Vec<GpuPrimitiveResourceAccess> {
        match self {
            Self::CounterReset(step) => vec![GpuPrimitiveResourceAccess::write(step.counters)],
            Self::U32PrefixScan(step) => vec![
                GpuPrimitiveResourceAccess::read(step.input),
                GpuPrimitiveResourceAccess::write(step.output),
            ],
            Self::U32Scatter(step) => vec![
                GpuPrimitiveResourceAccess::read(step.source_indices),
                GpuPrimitiveResourceAccess::read(step.prefix_offsets),
                GpuPrimitiveResourceAccess::write(step.output_indices),
            ],
            Self::IndirectDrawArgs(step) => vec![GpuPrimitiveResourceAccess::write(step.output)],
        }
    }
}

impl From<CounterResetDescriptor> for GpuPrimitiveStep {
    fn from(value: CounterResetDescriptor) -> Self {
        Self::CounterReset(value)
    }
}

impl From<U32PrefixScanDescriptor> for GpuPrimitiveStep {
    fn from(value: U32PrefixScanDescriptor) -> Self {
        Self::U32PrefixScan(value)
    }
}

impl From<U32ScatterDescriptor> for GpuPrimitiveStep {
    fn from(value: U32ScatterDescriptor) -> Self {
        Self::U32Scatter(value)
    }
}

impl From<IndirectDrawArgsGenerationDescriptor> for GpuPrimitiveStep {
    fn from(value: IndirectDrawArgsGenerationDescriptor) -> Self {
        Self::IndirectDrawArgs(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPrimitiveExecutionPlan {
    pub label: String,
    pub steps: Vec<GpuPrimitiveStep>,
}

impl GpuPrimitiveExecutionPlan {
    pub fn new(
        label: impl Into<String>,
        steps: impl IntoIterator<Item = GpuPrimitiveStep>,
    ) -> Result<Self, GpuPrimitiveValidationError> {
        let plan = Self {
            label: label.into(),
            steps: steps.into_iter().collect(),
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), GpuPrimitiveValidationError> {
        if self.label.trim().is_empty() {
            return Err(GpuPrimitiveValidationError::EmptyLabel {
                primitive: "gpu_primitive_execution_plan",
            });
        }
        if self.steps.is_empty() {
            return Err(GpuPrimitiveValidationError::EmptyExecutionPlan {
                label: self.label.clone(),
            });
        }
        for step in &self.steps {
            step.validate()?;
        }
        Ok(())
    }

    pub fn resource_accesses(&self) -> Vec<GpuPrimitiveResourceAccess> {
        self.steps
            .iter()
            .flat_map(GpuPrimitiveStep::resource_accesses)
            .collect()
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{
        CounterResetDescriptor, PrefixScanMode, RenderFlow, U32Counter, U32PrefixScanDescriptor,
        U32ScanElement,
    };

    #[test]
    fn gpu_primitives_execution_plan_rejects_empty_step_list() {
        assert!(matches!(
            GpuPrimitiveExecutionPlan::new("empty", []),
            Err(GpuPrimitiveValidationError::EmptyExecutionPlan { .. })
        ));
    }

    #[test]
    fn gpu_primitives_execution_plan_exposes_resource_accesses() {
        let (flow, counters) =
            RenderFlow::new("test.primitive.plan").storage_array::<U32Counter>("counts", 16);
        let (flow, offsets) = flow.storage_array::<U32ScanElement>("offsets", 16);
        let _flow = flow;
        let counters_id = *counters.id();
        let offsets_id = *offsets.id();

        let reset = CounterResetDescriptor::new("reset", counters.clone(), 16)
            .expect("valid counter reset descriptor");
        let scan =
            U32PrefixScanDescriptor::new("scan", counters, offsets, 16, PrefixScanMode::Exclusive)
                .expect("valid scan descriptor");
        let plan = GpuPrimitiveExecutionPlan::new(
            "grid.build",
            [GpuPrimitiveStep::from(reset), GpuPrimitiveStep::from(scan)],
        )
        .expect("valid primitive plan");

        assert_eq!(plan.step_count(), 2);
        let accesses = plan.resource_accesses();
        assert!(accesses.iter().any(|access| {
            access.kind == GpuPrimitiveResourceAccessKind::Write
                && access.resource_id == counters_id
        }));
        assert!(accesses.iter().any(|access| {
            access.kind == GpuPrimitiveResourceAccessKind::Write && access.resource_id == offsets_id
        }));
    }
}
