use super::{
    CounterResetDescriptor, GeneratedIndirectDrawArgs, GpuPrimitiveValidationError,
    IndirectDrawArgsGenerationDescriptor, PrefixScanMode, U32PrefixScanDescriptor,
    U32ScatterDescriptor,
};
use crate::plugins::gpu::GpuBufferHandle;
use crate::plugins::render::RenderShaderConstant;

pub const GPU_PRIMITIVE_WORKGROUP_SIZE: u32 = 64;
pub const GPU_PRIMITIVE_COUNTER_RESET_SHADER: &str =
    "assets/shaders/gpu_primitive_counter_reset.wgsl";
pub const GPU_PRIMITIVE_PREFIX_SCAN_SHADER: &str = "assets/shaders/gpu_primitive_prefix_scan.wgsl";
pub const GPU_PRIMITIVE_PREFIX_SCAN_APPLY_OFFSETS_SHADER: &str =
    "assets/shaders/gpu_primitive_prefix_scan_apply_offsets.wgsl";
pub const GPU_PRIMITIVE_U32_SCATTER_SHADER: &str = "assets/shaders/gpu_primitive_u32_scatter.wgsl";
pub const GPU_PRIMITIVE_INDIRECT_DRAW_ARGS_SHADER: &str =
    "assets/shaders/gpu_primitive_indirect_draw_args.wgsl";
pub const GPU_PRIMITIVE_INDEXED_INDIRECT_DRAW_ARGS_SHADER: &str =
    "assets/shaders/gpu_primitive_indexed_indirect_draw_args.wgsl";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPrimitiveDispatchStageKind {
    CounterReset,
    U32PrefixScanBlock { mode: PrefixScanMode },
    U32PrefixScanApplyBlockOffsets,
    U32Scatter,
    IndirectDrawArgs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPrimitiveDispatchStage {
    pub label: String,
    pub kind: GpuPrimitiveDispatchStageKind,
    pub shader_asset: &'static str,
    pub reads: Vec<GpuBufferHandle>,
    pub writes: Vec<GpuBufferHandle>,
    pub dispatch: [u32; 3],
    pub constants: Vec<RenderShaderConstant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPrimitiveDispatchPlan {
    pub label: String,
    pub temporary_storage: Vec<GpuBufferHandle>,
    pub stages: Vec<GpuPrimitiveDispatchStage>,
}

impl GpuPrimitiveDispatchPlan {
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn temporary_storage_count(&self) -> usize {
        self.temporary_storage.len()
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

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub(crate) fn dispatch_plan_with_temporary<F, E>(
        &self,
        mut allocate_temporary: F,
    ) -> Result<GpuPrimitiveDispatchPlan, E>
    where
        F: FnMut(String, u64) -> Result<GpuBufferHandle, E>,
        E: From<GpuPrimitiveValidationError>,
    {
        self.validate().map_err(E::from)?;
        let mut builder = GpuPrimitiveDispatchPlanBuilder::new(self.label.clone());
        for (step_index, step) in self.steps.iter().enumerate() {
            builder.push_step(step_index, step, &mut allocate_temporary)?;
        }
        Ok(builder.finish())
    }
}

#[derive(Debug, Clone)]
struct PrefixScanLevel {
    output: GpuBufferHandle,
    element_count: u32,
}

struct GpuPrimitiveDispatchPlanBuilder {
    label: String,
    temporary_storage: Vec<GpuBufferHandle>,
    stages: Vec<GpuPrimitiveDispatchStage>,
}

impl GpuPrimitiveDispatchPlanBuilder {
    fn new(label: String) -> Self {
        Self {
            label,
            temporary_storage: Vec::new(),
            stages: Vec::new(),
        }
    }

    fn push_step<F, E>(
        &mut self,
        step_index: usize,
        step: &GpuPrimitiveStep,
        allocate_temporary: &mut F,
    ) -> Result<(), E>
    where
        F: FnMut(String, u64) -> Result<GpuBufferHandle, E>,
    {
        match step {
            GpuPrimitiveStep::CounterReset(step) => self.push_counter_reset(step_index, step),
            GpuPrimitiveStep::U32PrefixScan(step) => {
                self.push_prefix_scan(step_index, step, allocate_temporary)?
            }
            GpuPrimitiveStep::U32Scatter(step) => self.push_scatter(step_index, step),
            GpuPrimitiveStep::IndirectDrawArgs(step) => {
                self.push_indirect_draw_args(step_index, step)
            }
        }
        Ok(())
    }

    fn push_counter_reset(&mut self, step_index: usize, step: &CounterResetDescriptor) {
        self.push_stage(GpuPrimitiveDispatchStage {
            label: self.stage_label(step_index, step.label.as_str(), "counter_reset"),
            kind: GpuPrimitiveDispatchStageKind::CounterReset,
            shader_asset: GPU_PRIMITIVE_COUNTER_RESET_SHADER,
            reads: Vec::new(),
            writes: vec![step.counters.clone()],
            dispatch: dispatch_for_count(step.counter_count),
            constants: vec![
                RenderShaderConstant::u32("ELEMENT_COUNT", step.counter_count),
                RenderShaderConstant::u32("RESET_VALUE", step.reset_value),
            ],
        });
    }

    fn push_prefix_scan<F, E>(
        &mut self,
        step_index: usize,
        step: &U32PrefixScanDescriptor,
        allocate_temporary: &mut F,
    ) -> Result<(), E>
    where
        F: FnMut(String, u64) -> Result<GpuBufferHandle, E>,
    {
        let mut levels = Vec::<PrefixScanLevel>::new();
        let mut input = step.input.clone();
        let mut output = step.output.clone();
        let mut element_count = step.total_count;
        let mut level_index = 0usize;

        loop {
            let block_count = block_count_for(element_count);
            let block_sums = self.register_temporary_scan_storage(
                step_index,
                step.label.as_str(),
                level_index,
                "block_sums",
                block_count,
                allocate_temporary,
            )?;
            let mode = if level_index == 0 {
                step.mode
            } else {
                PrefixScanMode::Exclusive
            };
            self.push_stage(GpuPrimitiveDispatchStage {
                label: self.stage_label(
                    step_index,
                    step.label.as_str(),
                    format!("scan_level_{level_index}"),
                ),
                kind: GpuPrimitiveDispatchStageKind::U32PrefixScanBlock { mode },
                shader_asset: GPU_PRIMITIVE_PREFIX_SCAN_SHADER,
                reads: vec![input.clone()],
                writes: vec![output.clone(), block_sums.clone()],
                dispatch: [block_count, 1, 1],
                constants: vec![
                    RenderShaderConstant::u32("ELEMENT_COUNT", element_count),
                    RenderShaderConstant::u32(
                        "INCLUSIVE",
                        if matches!(mode, PrefixScanMode::Inclusive) {
                            1
                        } else {
                            0
                        },
                    ),
                ],
            });
            levels.push(PrefixScanLevel {
                output: output.clone(),
                element_count,
            });

            if block_count <= 1 {
                break;
            }

            input = block_sums;
            output = self.register_temporary_scan_storage(
                step_index,
                step.label.as_str(),
                level_index,
                "block_offsets",
                block_count,
                allocate_temporary,
            )?;
            element_count = block_count;
            level_index = level_index.saturating_add(1);
        }

        if levels.len() <= 1 {
            return Ok(());
        }

        for level_index in (0..levels.len() - 1).rev() {
            let output = levels[level_index].output.clone();
            let offsets = levels[level_index + 1].output.clone();
            self.push_stage(GpuPrimitiveDispatchStage {
                label: self.stage_label(
                    step_index,
                    step.label.as_str(),
                    format!("apply_offsets_level_{level_index}"),
                ),
                kind: GpuPrimitiveDispatchStageKind::U32PrefixScanApplyBlockOffsets,
                shader_asset: GPU_PRIMITIVE_PREFIX_SCAN_APPLY_OFFSETS_SHADER,
                reads: vec![output.clone(), offsets],
                writes: vec![output],
                dispatch: dispatch_for_count(levels[level_index].element_count),
                constants: vec![RenderShaderConstant::u32(
                    "ELEMENT_COUNT",
                    levels[level_index].element_count,
                )],
            });
        }
        Ok(())
    }

    fn push_scatter(&mut self, step_index: usize, step: &U32ScatterDescriptor) {
        self.push_stage(GpuPrimitiveDispatchStage {
            label: self.stage_label(step_index, step.label.as_str(), "scatter"),
            kind: GpuPrimitiveDispatchStageKind::U32Scatter,
            shader_asset: GPU_PRIMITIVE_U32_SCATTER_SHADER,
            reads: vec![step.source_indices.clone(), step.prefix_offsets.clone()],
            writes: vec![step.output_indices.clone()],
            dispatch: dispatch_for_count(step.element_count),
            constants: vec![
                RenderShaderConstant::u32("ELEMENT_COUNT", step.element_count),
                RenderShaderConstant::u32("OUTPUT_CAPACITY", step.output_capacity),
            ],
        });
    }

    fn push_indirect_draw_args(
        &mut self,
        step_index: usize,
        step: &IndirectDrawArgsGenerationDescriptor,
    ) {
        let (shader_asset, constants) = match step.args {
            GeneratedIndirectDrawArgs::Draw(args) => (
                GPU_PRIMITIVE_INDIRECT_DRAW_ARGS_SHADER,
                vec![
                    RenderShaderConstant::u32("OUTPUT_INDEX", step.output_index),
                    RenderShaderConstant::u32("VERTEX_COUNT", args.vertex_count),
                    RenderShaderConstant::u32("INSTANCE_COUNT", args.instance_count),
                    RenderShaderConstant::u32("FIRST_VERTEX", args.first_vertex),
                    RenderShaderConstant::u32("FIRST_INSTANCE", args.first_instance),
                ],
            ),
            GeneratedIndirectDrawArgs::DrawIndexed(args) => (
                GPU_PRIMITIVE_INDEXED_INDIRECT_DRAW_ARGS_SHADER,
                vec![
                    RenderShaderConstant::u32("OUTPUT_INDEX", step.output_index),
                    RenderShaderConstant::u32("INDEX_COUNT", args.index_count),
                    RenderShaderConstant::u32("INSTANCE_COUNT", args.instance_count),
                    RenderShaderConstant::u32("FIRST_INDEX", args.first_index),
                    RenderShaderConstant::i32("BASE_VERTEX", args.base_vertex),
                    RenderShaderConstant::u32("FIRST_INSTANCE", args.first_instance),
                ],
            ),
        };
        self.push_stage(GpuPrimitiveDispatchStage {
            label: self.stage_label(step_index, step.label.as_str(), "indirect_draw_args"),
            kind: GpuPrimitiveDispatchStageKind::IndirectDrawArgs,
            shader_asset,
            reads: Vec::new(),
            writes: vec![step.output.clone()],
            dispatch: [1, 1, 1],
            constants,
        });
    }

    fn push_stage(&mut self, stage: GpuPrimitiveDispatchStage) {
        self.stages.push(stage);
    }

    fn register_temporary_scan_storage<F, E>(
        &mut self,
        step_index: usize,
        step_label: &str,
        level_index: usize,
        suffix: &str,
        element_count: u32,
        allocate_temporary: &mut F,
    ) -> Result<GpuBufferHandle, E>
    where
        F: FnMut(String, u64) -> Result<GpuBufferHandle, E>,
    {
        let label = self.temporary_label(step_index, step_label, level_index, suffix);
        if let Some(existing) = self
            .temporary_storage
            .iter()
            .find(|existing| existing.descriptor().common().label().as_str() == label)
        {
            return Ok(existing.clone());
        }
        let handle = allocate_temporary(label, u64::from(element_count.max(1)))?;
        self.temporary_storage.push(handle.clone());
        Ok(handle)
    }

    fn stage_label(
        &self,
        step_index: usize,
        step_label: &str,
        suffix: impl std::fmt::Display,
    ) -> String {
        format!("{}.{}.{}.{}", self.label, step_index, step_label, suffix)
    }

    fn temporary_label(
        &self,
        step_index: usize,
        step_label: &str,
        level_index: usize,
        suffix: &str,
    ) -> String {
        format!(
            "{}.{}.{}.level_{}.{}",
            self.label, step_index, step_label, level_index, suffix
        )
    }

    fn finish(self) -> GpuPrimitiveDispatchPlan {
        GpuPrimitiveDispatchPlan {
            label: self.label,
            temporary_storage: self.temporary_storage,
            stages: self.stages,
        }
    }
}

fn block_count_for(element_count: u32) -> u32 {
    element_count.div_ceil(GPU_PRIMITIVE_WORKGROUP_SIZE).max(1)
}

fn dispatch_for_count(element_count: u32) -> [u32; 3] {
    [block_count_for(element_count), 1, 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{GpuCapabilityProfile, GpuContext, GpuContextDescriptor};
    use crate::plugins::render::{
        CompiledDrawSource, CompiledPassExecutionPlan, CounterResetDescriptor, DrawIndirectArgs,
        IndirectDrawArgsGenerationDescriptor, PrefixScanMode, RenderFlow, RenderShaderReference,
        U32Counter, U32PrefixScanDescriptor, U32ScanElement, U32ScatterDescriptor,
        compile_flow_plan,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use wgpu::util::DeviceExt;

    fn dispatch_plan_for_test(plan: &GpuPrimitiveExecutionPlan) -> GpuPrimitiveDispatchPlan {
        let mut owner = Some(RenderFlow::new(format!("{}.test_temporaries", plan.label)));
        plan.dispatch_plan_with_temporary(|label, element_count| {
            let flow = owner
                .take()
                .expect("test temporary owner should be available");
            let (flow, handle) = flow
                .storage_array::<U32ScanElement>(label, element_count)
                .expect("typed primitive temporary should allocate through the flow owner");
            owner = Some(flow);
            Ok::<_, GpuPrimitiveValidationError>(handle)
        })
        .expect("primitive dispatch plan should lower")
    }

    #[test]
    fn gpu_primitives_execution_plan_rejects_empty_step_list() {
        assert!(matches!(
            GpuPrimitiveExecutionPlan::new("empty", []),
            Err(GpuPrimitiveValidationError::EmptyExecutionPlan { .. })
        ));
    }

    #[test]
    fn gpu_primitives_execution_plan_retains_typed_buffer_handles() {
        let (flow, counters) = RenderFlow::new("test.primitive.plan")
            .storage_array::<U32Counter>("counts", 16)
            .expect("render flow authoring should succeed");
        let (flow, offsets) = flow
            .storage_array::<U32ScanElement>("offsets", 16)
            .expect("render flow authoring should succeed");
        let _flow = flow;
        let counters_id = counters.diagnostic_identity();
        let offsets_id = offsets.diagnostic_identity();

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
        let GpuPrimitiveStep::CounterReset(reset) = &plan.steps[0] else {
            panic!("first primitive step should remain a counter reset");
        };
        let GpuPrimitiveStep::U32PrefixScan(scan) = &plan.steps[1] else {
            panic!("second primitive step should remain a prefix scan");
        };
        assert_eq!(reset.counters.diagnostic_identity(), counters_id);
        assert_eq!(scan.output.diagnostic_identity(), offsets_id);
    }

    #[test]
    fn gpu_primitives_prefix_scan_dispatch_covers_hierarchical_counts() {
        let stage_counts = [1_u32, 64, 65, 130, 4097]
            .into_iter()
            .map(prefix_scan_stage_count)
            .collect::<Vec<_>>();

        assert_eq!(stage_counts, vec![1, 1, 3, 3, 5]);
    }

    #[test]
    fn gpu_primitives_4097_scan_lowers_typed_temporaries_and_inferred_stage_order() {
        let (flow, input) = RenderFlow::new("test.primitive.4097")
            .storage_array::<U32ScanElement>("scan.input", 4097)
            .expect("scan input should be valid");
        let (flow, output) = flow
            .storage_array::<U32ScanElement>("scan.output", 4097)
            .expect("scan output should be valid");
        let scan =
            U32PrefixScanDescriptor::new("scan", input, output, 4097, PrefixScanMode::Exclusive)
                .expect("4097-element scan should be valid");
        let primitive =
            GpuPrimitiveExecutionPlan::new("primitive.4097", [GpuPrimitiveStep::from(scan)])
                .expect("4097-element primitive plan should be valid");
        let flow = flow
            .gpu_primitive_plan(&primitive)
            .expect("primitive temporaries should allocate through the flow owner")
            .validate()
            .expect("4097-element primitive flow should validate");
        let compiled =
            compile_flow_plan(&flow).expect("4097-element primitive flow should compile");
        let temporary_ids = compiled
            .resources
            .resources
            .iter()
            .filter_map(|resource| {
                let handle = resource.buffer_handle()?;
                handle
                    .descriptor()
                    .common()
                    .label()
                    .as_str()
                    .starts_with("primitive.4097.0.scan.level_")
                    .then(|| handle.diagnostic_identity())
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(temporary_ids.len(), 5);
        assert!(compiled.resources.resources.iter().all(|resource| {
            !temporary_ids.contains(resource.id())
                || resource.lifetime() == crate::plugins::gpu::GpuResourceLifetime::Transient
        }));

        let prepared = compiled
            .structural_work()
            .expect("compiled primitive flow should retain G3 work")
            .graph();
        assert_eq!(prepared.nodes().len(), 5);
        assert!(prepared.nodes().iter().all(|prepared_node| {
            matches!(
                prepared_node.node().operation(),
                crate::plugins::gpu::GpuWorkOperation::Compute(_)
            )
        }));
        let accessed_temporary_ids = prepared
            .nodes()
            .iter()
            .flat_map(|prepared_node| prepared_node.node().accesses())
            .filter_map(|access| match access {
                crate::plugins::gpu::GpuResourceAccess::Buffer(access)
                    if temporary_ids.contains(&access.resource_identity()) =>
                {
                    Some(access.resource_identity())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(accessed_temporary_ids, temporary_ids);
        assert!(prepared.dependencies().iter().all(|dependency| {
            dependency.reasons().iter().all(|reason| {
                !matches!(
                    reason,
                    crate::plugins::gpu::GpuDependencyReason::ExplicitNonData { .. }
                )
            })
        }));
        assert!(prepared.topological_order().windows(2).all(|pair| {
            prepared
                .dependencies()
                .iter()
                .any(|dependency| dependency.before() == pair[0] && dependency.after() == pair[1])
        }));
    }

    #[test]
    fn gpu_primitives_execution_plan_lowers_to_compute_dispatch_stages() {
        let (flow, counters) = RenderFlow::new("test.primitive.dispatch")
            .storage_array::<U32Counter>("counts", 130)
            .expect("render flow authoring should succeed");
        let (flow, offsets) = flow
            .storage_array::<U32ScanElement>("offsets", 130)
            .expect("render flow authoring should succeed");
        let (flow, source_indices) = flow
            .storage_array::<U32ScanElement>("source", 130)
            .expect("render flow authoring should succeed");
        let (flow, sorted_indices) = flow
            .storage_array::<U32ScanElement>("sorted", 130)
            .expect("render flow authoring should succeed");
        let (_flow, draw_args) = flow
            .storage_array::<DrawIndirectArgs>("draw.args", 1)
            .expect("render flow authoring should succeed");

        let reset = CounterResetDescriptor::new("reset", counters.clone(), 130)
            .expect("valid counter reset");
        let scan = U32PrefixScanDescriptor::new(
            "scan",
            counters,
            offsets.clone(),
            130,
            PrefixScanMode::Exclusive,
        )
        .expect("valid scan");
        let scatter =
            U32ScatterDescriptor::new("scatter", source_indices, offsets, sorted_indices, 130, 130)
                .expect("valid scatter");
        let args = IndirectDrawArgsGenerationDescriptor::draw(
            "draw_args",
            draw_args,
            0,
            DrawIndirectArgs::new(6, 130, 0, 0),
        )
        .expect("valid args generation");
        let primitive_plan = GpuPrimitiveExecutionPlan::new(
            "primitive.dispatch",
            [
                GpuPrimitiveStep::from(reset),
                GpuPrimitiveStep::from(scan),
                GpuPrimitiveStep::from(scatter),
                GpuPrimitiveStep::from(args),
            ],
        )
        .expect("valid primitive plan");
        let dispatch_plan = dispatch_plan_for_test(&primitive_plan);

        assert_eq!(dispatch_plan.stage_count(), 6);
        assert_eq!(dispatch_plan.temporary_storage_count(), 3);
        assert!(
            dispatch_plan.stages.iter().any(|stage| {
                stage.shader_asset == GPU_PRIMITIVE_PREFIX_SCAN_APPLY_OFFSETS_SHADER
            })
        );
        assert!(dispatch_plan.stages.iter().all(|stage| {
            stage
                .reads
                .iter()
                .chain(&stage.writes)
                .all(|resource| resource.descriptor().size_bytes() > 0)
        }));
    }

    #[test]
    fn gpu_primitives_append_to_render_flow_uses_normal_compute_passes() {
        let (flow, counters) = RenderFlow::new("test.primitive.flow")
            .with_color_target("test.primitive.color")
            .expect("render flow authoring should succeed")
            .storage_array::<U32Counter>("counts", 130)
            .expect("render flow authoring should succeed");
        let (flow, offsets) = flow
            .storage_array::<U32ScanElement>("offsets", 130)
            .expect("render flow authoring should succeed");
        let (flow, source_indices) = flow
            .storage_array::<U32ScanElement>("source", 130)
            .expect("render flow authoring should succeed");
        let (flow, sorted_indices) = flow
            .storage_array::<U32ScanElement>("sorted", 130)
            .expect("render flow authoring should succeed");
        let (flow, draw_args) = flow
            .storage_array::<DrawIndirectArgs>("draw.args", 1)
            .expect("render flow authoring should succeed");

        let reset = CounterResetDescriptor::new("reset", counters.clone(), 130)
            .expect("valid counter reset");
        let scan = U32PrefixScanDescriptor::new(
            "scan",
            counters,
            offsets.clone(),
            130,
            PrefixScanMode::Inclusive,
        )
        .expect("valid scan");
        let scatter =
            U32ScatterDescriptor::new("scatter", source_indices, offsets, sorted_indices, 130, 130)
                .expect("valid scatter");
        let args = IndirectDrawArgsGenerationDescriptor::draw(
            "draw_args",
            draw_args.clone(),
            0,
            DrawIndirectArgs::new(6, 130, 0, 0),
        )
        .expect("valid args generation");
        let primitive_plan = GpuPrimitiveExecutionPlan::new(
            "primitive.flow",
            [
                GpuPrimitiveStep::from(reset),
                GpuPrimitiveStep::from(scan),
                GpuPrimitiveStep::from(scatter),
                GpuPrimitiveStep::from(args),
            ],
        )
        .expect("valid primitive plan");
        let flow = flow
            .gpu_primitive_plan(&primitive_plan)
            .expect("primitive plan should append to render flow")
            .graphics_pass("primitive.draw")
            .write_color_target("test.primitive.color")
            .draw_indirect(draw_args.clone(), 6, 130)
            .expect("declared draw-argument layout should match")
            .finish()
            .validate()
            .expect("primitive flow should validate");
        let compiled = compile_flow_plan(&flow).expect("primitive flow should compile");
        let prepared = compiled
            .structural_work()
            .expect("compiled primitive flow should retain G3 work")
            .graph();
        assert!(prepared.dependencies().iter().all(|dependency| {
            dependency.reasons().iter().all(|reason| {
                !matches!(
                    reason,
                    crate::plugins::gpu::GpuDependencyReason::ExplicitNonData { .. }
                )
            })
        }));

        let compute_passes = compiled
            .execution
            .passes
            .iter()
            .filter_map(|pass| match pass {
                CompiledPassExecutionPlan::Compute(value) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(compute_passes.len(), 6);
        assert!(compute_passes.iter().any(|pass| {
            matches!(
                pass.shader.as_ref(),
                Some(RenderShaderReference::AssetPath(path))
                    if path == GPU_PRIMITIVE_INDIRECT_DRAW_ARGS_SHADER
            ) && pass
                .shader_constants
                .iter()
                .any(|constant| constant.name == "INSTANCE_COUNT" && constant.value == 130)
        }));

        let draw = compiled
            .execution
            .passes
            .iter()
            .find_map(|pass| match pass {
                CompiledPassExecutionPlan::Graphics(value) => value.draw,
                _ => None,
            })
            .expect("graphics pass should preserve indirect draw");
        assert!(matches!(
            draw.source,
            CompiledDrawSource::Indirect { args_buffer, .. }
                if args_buffer == draw_args.diagnostic_identity()
        ));
    }

    #[test]
    fn gpu_primitives_shader_assets_parse_as_wgsl() {
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under workspace root");
        for shader in [
            GPU_PRIMITIVE_COUNTER_RESET_SHADER,
            GPU_PRIMITIVE_PREFIX_SCAN_SHADER,
            GPU_PRIMITIVE_PREFIX_SCAN_APPLY_OFFSETS_SHADER,
            GPU_PRIMITIVE_U32_SCATTER_SHADER,
            GPU_PRIMITIVE_INDIRECT_DRAW_ARGS_SHADER,
            GPU_PRIMITIVE_INDEXED_INDIRECT_DRAW_ARGS_SHADER,
        ] {
            let source = std::fs::read_to_string(workspace_root.join(shader))
                .unwrap_or_else(|err| panic!("failed to read primitive shader '{shader}': {err}"));
            naga::front::wgsl::parse_str(source.as_str()).unwrap_or_else(|err| {
                panic!("primitive shader '{shader}' failed WGSL parse: {err}")
            });
        }
    }

    #[test]
    fn gpu_primitives_runtime_dispatch_writes_scan_scatter_and_draw_args_when_adapter_available() {
        let context = match pollster::block_on(GpuContext::request(GpuContextDescriptor::new(
            GpuCapabilityProfile::ComputeBaseline.requirements(),
        ))) {
            Ok(context) => context,
            Err(error) => {
                println!("gpu primitive runtime dispatch test skipped: {error}");
                return;
            }
        };
        let loan = context.current_render_device_queue();
        let (device, queue) = (loan.device, loan.queue);
        if !context
            .device_facts()
            .is_enabled(crate::plugins::gpu::GpuCapabilityFeature::Compute)
        {
            return;
        }
        let element_count = 130_u32;
        let (flow, scan_input) = RenderFlow::new("test.primitive.runtime")
            .storage_array::<U32ScanElement>("scan.input", u64::from(element_count))
            .expect("render flow authoring should succeed");
        let (flow, scan_output) = flow
            .storage_array::<U32ScanElement>("scan.output", u64::from(element_count))
            .expect("render flow authoring should succeed");
        let (flow, source_indices) = flow
            .storage_array::<U32ScanElement>("source.indices", u64::from(element_count))
            .expect("render flow authoring should succeed");
        let (flow, sorted_indices) = flow
            .storage_array::<U32ScanElement>("sorted.indices", u64::from(element_count))
            .expect("render flow authoring should succeed");
        let (_flow, draw_args) = flow
            .storage_array::<DrawIndirectArgs>("draw.args", 1)
            .expect("render flow authoring should succeed");

        let scan = U32PrefixScanDescriptor::new(
            "scan",
            scan_input.clone(),
            scan_output.clone(),
            element_count,
            PrefixScanMode::Exclusive,
        )
        .expect("runtime scan descriptor should be valid");
        let scatter = U32ScatterDescriptor::new(
            "scatter",
            source_indices.clone(),
            scan_output.clone(),
            sorted_indices.clone(),
            element_count,
            element_count,
        )
        .expect("runtime scatter descriptor should be valid");
        let args = IndirectDrawArgsGenerationDescriptor::draw(
            "draw_args",
            draw_args.clone(),
            0,
            DrawIndirectArgs::new(6, element_count, 0, 0),
        )
        .expect("runtime draw args descriptor should be valid");
        let primitive_plan = GpuPrimitiveExecutionPlan::new(
            "primitive.runtime",
            [
                GpuPrimitiveStep::from(scan),
                GpuPrimitiveStep::from(scatter),
                GpuPrimitiveStep::from(args),
            ],
        )
        .expect("runtime primitive plan should be valid");
        let dispatch_plan = dispatch_plan_for_test(&primitive_plan);

        let mut buffers = BTreeMap::<GpuBufferHandle, wgpu::Buffer>::new();
        insert_storage_buffer(
            device,
            &mut buffers,
            scan_input.clone(),
            &vec![1_u32; element_count as usize],
        );
        insert_storage_buffer(
            device,
            &mut buffers,
            scan_output.clone(),
            &vec![0_u32; element_count as usize],
        );
        insert_storage_buffer(
            device,
            &mut buffers,
            source_indices.clone(),
            &(0..element_count)
                .map(|index| 1000 + index)
                .collect::<Vec<_>>(),
        );
        insert_storage_buffer(
            device,
            &mut buffers,
            sorted_indices.clone(),
            &vec![0_u32; element_count as usize],
        );
        insert_storage_buffer(device, &mut buffers, draw_args.clone(), &[0_u32; 4]);
        for temporary in &dispatch_plan.temporary_storage {
            insert_storage_buffer(
                device,
                &mut buffers,
                temporary.clone(),
                &vec![0_u32; (temporary.descriptor().size_bytes() / 4) as usize],
            );
        }

        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under workspace root");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("gpu_primitive_runtime_test_encoder"),
        });
        for stage in &dispatch_plan.stages {
            encode_runtime_primitive_stage(device, &mut encoder, workspace_root, &buffers, stage);
        }
        let scan_readback = copy_storage_buffer_to_readback(
            device,
            &mut encoder,
            buffers
                .get(&scan_output)
                .expect("scan output buffer should exist"),
            u64::from(element_count) * 4,
        );
        let scatter_readback = copy_storage_buffer_to_readback(
            device,
            &mut encoder,
            buffers
                .get(&sorted_indices)
                .expect("scatter output buffer should exist"),
            u64::from(element_count) * 4,
        );
        let args_readback = copy_storage_buffer_to_readback(
            device,
            &mut encoder,
            buffers
                .get(&draw_args)
                .expect("draw args buffer should exist"),
            DrawIndirectArgs::BYTE_SIZE,
        );
        queue.submit(std::iter::once(encoder.finish()));

        let scan_values = read_u32_buffer(device, &scan_readback);
        let scatter_values = read_u32_buffer(device, &scatter_readback);
        let args_values = read_u32_buffer(device, &args_readback);

        assert_eq!(
            scan_values,
            (0..element_count).collect::<Vec<_>>(),
            "exclusive scan should produce 0..N offsets for all-one input"
        );
        assert_eq!(
            scatter_values,
            (0..element_count)
                .map(|index| 1000 + index)
                .collect::<Vec<_>>(),
            "scatter should use scanned offsets to preserve source order"
        );
        assert_eq!(args_values, vec![6, element_count, 0, 0]);
    }

    fn prefix_scan_stage_count(element_count: u32) -> usize {
        let (flow, input) = RenderFlow::new("test.primitive.scan.dispatch")
            .storage_array::<U32ScanElement>("scan.input", u64::from(element_count))
            .expect("render flow authoring should succeed");
        let (_flow, output) = flow
            .storage_array::<U32ScanElement>("scan.output", u64::from(element_count))
            .expect("render flow authoring should succeed");
        let scan = U32PrefixScanDescriptor::new(
            "scan",
            input,
            output,
            element_count,
            PrefixScanMode::Exclusive,
        )
        .expect("valid scan descriptor");
        let plan = GpuPrimitiveExecutionPlan::new("scan.plan", [GpuPrimitiveStep::from(scan)])
            .expect("valid primitive plan");
        dispatch_plan_for_test(&plan).stage_count()
    }

    fn insert_storage_buffer(
        device: &wgpu::Device,
        buffers: &mut BTreeMap<GpuBufferHandle, wgpu::Buffer>,
        resource: GpuBufferHandle,
        values: &[u32],
    ) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gpu_primitive_runtime_test_storage"),
            contents: crate::plugins::render::bytemuck::cast_slice(values),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        buffers.insert(resource, buffer);
    }

    fn encode_runtime_primitive_stage(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        workspace_root: &std::path::Path,
        buffers: &BTreeMap<GpuBufferHandle, wgpu::Buffer>,
        stage: &GpuPrimitiveDispatchStage,
    ) {
        let source = std::fs::read_to_string(workspace_root.join(stage.shader_asset))
            .unwrap_or_else(|err| panic!("failed to read shader '{}': {err}", stage.shader_asset));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu_primitive_runtime_test_shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let binding_resources = stage_binding_resources(stage);
        let layout_entries = binding_resources
            .iter()
            .enumerate()
            .map(
                |(index, (_resource, writable))| wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: !*writable,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            )
            .collect::<Vec<_>>();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu_primitive_runtime_test_bind_group_layout"),
            entries: layout_entries.as_slice(),
        });
        let bind_group_entries = binding_resources
            .iter()
            .enumerate()
            .map(|(index, (resource, _writable))| wgpu::BindGroupEntry {
                binding: index as u32,
                resource: buffers
                    .get(resource)
                    .expect("primitive runtime buffer should exist")
                    .as_entire_binding(),
            })
            .collect::<Vec<_>>();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu_primitive_runtime_test_bind_group"),
            layout: &bind_group_layout,
            entries: bind_group_entries.as_slice(),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gpu_primitive_runtime_test_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let constants = stage
            .constants
            .iter()
            .map(|constant| (constant.name.as_str(), constant.value as f64))
            .collect::<Vec<_>>();
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gpu_primitive_runtime_test_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: constants.as_slice(),
                ..wgpu::PipelineCompilationOptions::default()
            },
            cache: None,
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("gpu_primitive_runtime_test_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(stage.dispatch[0], stage.dispatch[1], stage.dispatch[2]);
    }

    fn stage_binding_resources(stage: &GpuPrimitiveDispatchStage) -> Vec<(GpuBufferHandle, bool)> {
        let writable = stage.writes.iter().cloned().collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::<GpuBufferHandle>::new();
        let mut resources = Vec::<(GpuBufferHandle, bool)>::new();
        for resource in stage.reads.iter().chain(stage.writes.iter()) {
            if seen.insert(resource.clone()) {
                resources.push((resource.clone(), writable.contains(resource)));
            }
        }
        resources
    }

    fn copy_storage_buffer_to_readback(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Buffer,
        size: u64,
    ) -> wgpu::Buffer {
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu_primitive_runtime_test_readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(source, 0, &readback, 0, size);
        readback
    }

    fn read_u32_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<u32> {
        let slice = buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device polling should complete primitive readback");
        receiver
            .recv()
            .expect("primitive readback channel should receive")
            .expect("primitive readback mapping should succeed");
        let data = slice.get_mapped_range();
        let values = crate::plugins::render::bytemuck::cast_slice::<u8, u32>(&data).to_vec();
        drop(data);
        buffer.unmap();
        values
    }
}
