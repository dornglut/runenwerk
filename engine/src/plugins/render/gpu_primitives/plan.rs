use super::{
    CounterResetDescriptor, GeneratedIndirectDrawArgs, GpuPrimitiveValidationError,
    IndirectDrawArgsGenerationDescriptor, PrefixScanMode, U32PrefixScanDescriptor,
    U32ScatterDescriptor,
};
use crate::plugins::render::{RenderResourceId, RenderShaderConstant};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPrimitiveTemporaryStorageKind {
    U32ScanElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPrimitiveTemporaryStorage {
    pub label: String,
    pub kind: GpuPrimitiveTemporaryStorageKind,
    pub element_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuPrimitiveDispatchResource {
    Existing(RenderResourceId),
    Temporary(String),
}

impl GpuPrimitiveDispatchResource {
    pub const fn existing(resource_id: RenderResourceId) -> Self {
        Self::Existing(resource_id)
    }

    pub fn temporary(label: impl Into<String>) -> Self {
        Self::Temporary(label.into())
    }
}

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
    pub reads: Vec<GpuPrimitiveDispatchResource>,
    pub writes: Vec<GpuPrimitiveDispatchResource>,
    pub dispatch: [u32; 3],
    pub constants: Vec<RenderShaderConstant>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPrimitiveDispatchPlan {
    pub label: String,
    pub temporary_storage: Vec<GpuPrimitiveTemporaryStorage>,
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

    pub fn dispatch_plan(&self) -> Result<GpuPrimitiveDispatchPlan, GpuPrimitiveValidationError> {
        self.validate()?;
        let mut builder = GpuPrimitiveDispatchPlanBuilder::new(self.label.clone());
        for (step_index, step) in self.steps.iter().enumerate() {
            builder.push_step(step_index, step);
        }
        Ok(builder.finish())
    }
}

#[derive(Debug, Clone)]
struct PrefixScanLevel {
    output: GpuPrimitiveDispatchResource,
    element_count: u32,
}

struct GpuPrimitiveDispatchPlanBuilder {
    label: String,
    temporary_storage: Vec<GpuPrimitiveTemporaryStorage>,
    stages: Vec<GpuPrimitiveDispatchStage>,
    previous_stage_label: Option<String>,
}

impl GpuPrimitiveDispatchPlanBuilder {
    fn new(label: String) -> Self {
        Self {
            label,
            temporary_storage: Vec::new(),
            stages: Vec::new(),
            previous_stage_label: None,
        }
    }

    fn push_step(&mut self, step_index: usize, step: &GpuPrimitiveStep) {
        match step {
            GpuPrimitiveStep::CounterReset(step) => self.push_counter_reset(step_index, step),
            GpuPrimitiveStep::U32PrefixScan(step) => self.push_prefix_scan(step_index, step),
            GpuPrimitiveStep::U32Scatter(step) => self.push_scatter(step_index, step),
            GpuPrimitiveStep::IndirectDrawArgs(step) => {
                self.push_indirect_draw_args(step_index, step)
            }
        }
    }

    fn push_counter_reset(&mut self, step_index: usize, step: &CounterResetDescriptor) {
        self.push_stage(GpuPrimitiveDispatchStage {
            label: self.stage_label(step_index, step.label.as_str(), "counter_reset"),
            kind: GpuPrimitiveDispatchStageKind::CounterReset,
            shader_asset: GPU_PRIMITIVE_COUNTER_RESET_SHADER,
            reads: Vec::new(),
            writes: vec![GpuPrimitiveDispatchResource::existing(step.counters)],
            dispatch: dispatch_for_count(step.counter_count),
            constants: vec![
                RenderShaderConstant::u32("ELEMENT_COUNT", step.counter_count),
                RenderShaderConstant::u32("RESET_VALUE", step.reset_value),
            ],
            depends_on: Vec::new(),
        });
    }

    fn push_prefix_scan(&mut self, step_index: usize, step: &U32PrefixScanDescriptor) {
        let mut levels = Vec::<PrefixScanLevel>::new();
        let mut input = GpuPrimitiveDispatchResource::existing(step.input);
        let mut output = GpuPrimitiveDispatchResource::existing(step.output);
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
            );
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
                depends_on: Vec::new(),
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
            );
            element_count = block_count;
            level_index = level_index.saturating_add(1);
        }

        if levels.len() <= 1 {
            return;
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
                depends_on: Vec::new(),
            });
        }
    }

    fn push_scatter(&mut self, step_index: usize, step: &U32ScatterDescriptor) {
        self.push_stage(GpuPrimitiveDispatchStage {
            label: self.stage_label(step_index, step.label.as_str(), "scatter"),
            kind: GpuPrimitiveDispatchStageKind::U32Scatter,
            shader_asset: GPU_PRIMITIVE_U32_SCATTER_SHADER,
            reads: vec![
                GpuPrimitiveDispatchResource::existing(step.source_indices),
                GpuPrimitiveDispatchResource::existing(step.prefix_offsets),
            ],
            writes: vec![GpuPrimitiveDispatchResource::existing(step.output_indices)],
            dispatch: dispatch_for_count(step.element_count),
            constants: vec![
                RenderShaderConstant::u32("ELEMENT_COUNT", step.element_count),
                RenderShaderConstant::u32("OUTPUT_CAPACITY", step.output_capacity),
            ],
            depends_on: Vec::new(),
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
            writes: vec![GpuPrimitiveDispatchResource::existing(step.output)],
            dispatch: [1, 1, 1],
            constants,
            depends_on: Vec::new(),
        });
    }

    fn push_stage(&mut self, mut stage: GpuPrimitiveDispatchStage) {
        if let Some(previous) = self.previous_stage_label.as_ref() {
            stage.depends_on.push(previous.clone());
        }
        self.previous_stage_label = Some(stage.label.clone());
        self.stages.push(stage);
    }

    fn register_temporary_scan_storage(
        &mut self,
        step_index: usize,
        step_label: &str,
        level_index: usize,
        suffix: &str,
        element_count: u32,
    ) -> GpuPrimitiveDispatchResource {
        let label = self.temporary_label(step_index, step_label, level_index, suffix);
        if self
            .temporary_storage
            .iter()
            .all(|existing| existing.label != label)
        {
            self.temporary_storage.push(GpuPrimitiveTemporaryStorage {
                label: label.clone(),
                kind: GpuPrimitiveTemporaryStorageKind::U32ScanElement,
                element_count: u64::from(element_count.max(1)),
            });
        }
        GpuPrimitiveDispatchResource::temporary(label)
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
    use crate::plugins::render::{
        CompiledDrawSource, CompiledPassExecutionPlan, CounterResetDescriptor, DrawIndirectArgs,
        IndirectDrawArgsGenerationDescriptor, PrefixScanMode, RenderFlow, RenderShaderReference,
        U32Counter, U32PrefixScanDescriptor, U32ScanElement, U32ScatterDescriptor,
        compile_flow_plan,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;
    use wgpu::util::DeviceExt;

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

    #[test]
    fn gpu_primitives_prefix_scan_dispatch_covers_hierarchical_counts() {
        let stage_counts = [1_u32, 64, 65, 130, 4097]
            .into_iter()
            .map(prefix_scan_stage_count)
            .collect::<Vec<_>>();

        assert_eq!(stage_counts, vec![1, 1, 3, 3, 5]);
    }

    #[test]
    fn gpu_primitives_execution_plan_lowers_to_compute_dispatch_stages() {
        let (flow, counters) =
            RenderFlow::new("test.primitive.dispatch").storage_array::<U32Counter>("counts", 130);
        let (flow, offsets) = flow.storage_array::<U32ScanElement>("offsets", 130);
        let (flow, source_indices) = flow.storage_array::<U32ScanElement>("source", 130);
        let (flow, sorted_indices) = flow.storage_array::<U32ScanElement>("sorted", 130);
        let (_flow, draw_args) = flow.storage_array::<DrawIndirectArgs>("draw.args", 1);

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
        let dispatch_plan = primitive_plan
            .dispatch_plan()
            .expect("dispatch plan should lower");

        assert_eq!(dispatch_plan.stage_count(), 6);
        assert_eq!(dispatch_plan.temporary_storage_count(), 3);
        assert!(
            dispatch_plan.stages.iter().any(|stage| {
                stage.shader_asset == GPU_PRIMITIVE_PREFIX_SCAN_APPLY_OFFSETS_SHADER
            })
        );
        assert!(
            dispatch_plan
                .stages
                .windows(2)
                .all(|pair| pair[1].depends_on == vec![pair[0].label.clone()])
        );
    }

    #[test]
    fn gpu_primitives_append_to_render_flow_uses_normal_compute_passes() {
        let (flow, counters) = RenderFlow::new("test.primitive.flow")
            .with_color_target("test.primitive.color")
            .storage_array::<U32Counter>("counts", 130);
        let (flow, offsets) = flow.storage_array::<U32ScanElement>("offsets", 130);
        let (flow, source_indices) = flow.storage_array::<U32ScanElement>("source", 130);
        let (flow, sorted_indices) = flow.storage_array::<U32ScanElement>("sorted", 130);
        let (flow, draw_args) = flow.storage_array::<DrawIndirectArgs>("draw.args", 1);

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
        let final_primitive_pass = primitive_plan
            .dispatch_plan()
            .expect("dispatch plan should lower")
            .stages
            .last()
            .expect("primitive plan should emit at least one stage")
            .label
            .clone();

        let flow = flow
            .gpu_primitive_plan(&primitive_plan)
            .expect("primitive plan should append to render flow")
            .graphics_pass("primitive.draw")
            .write_color_target("test.primitive.color")
            .draw_indirect(draw_args.clone(), 6, 130)
            .depends_on(final_primitive_pass)
            .finish()
            .validate()
            .expect("primitive flow should validate");
        let compiled = compile_flow_plan(&flow).expect("primitive flow should compile");

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
            CompiledDrawSource::Indirect { args_buffer, .. } if args_buffer == *draw_args.id()
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
        let Some((device, queue)) = runtime_primitive_device() else {
            return;
        };
        let element_count = 130_u32;
        let (flow, scan_input) = RenderFlow::new("test.primitive.runtime")
            .storage_array::<U32ScanElement>("scan.input", u64::from(element_count));
        let (flow, scan_output) =
            flow.storage_array::<U32ScanElement>("scan.output", u64::from(element_count));
        let (flow, source_indices) =
            flow.storage_array::<U32ScanElement>("source.indices", u64::from(element_count));
        let (flow, sorted_indices) =
            flow.storage_array::<U32ScanElement>("sorted.indices", u64::from(element_count));
        let (_flow, draw_args) = flow.storage_array::<DrawIndirectArgs>("draw.args", 1);

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
        let dispatch_plan = primitive_plan
            .dispatch_plan()
            .expect("runtime primitive dispatch plan should lower");

        let mut buffers = BTreeMap::<GpuPrimitiveDispatchResource, wgpu::Buffer>::new();
        insert_storage_buffer(
            &device,
            &mut buffers,
            GpuPrimitiveDispatchResource::existing(*scan_input.id()),
            &vec![1_u32; element_count as usize],
        );
        insert_storage_buffer(
            &device,
            &mut buffers,
            GpuPrimitiveDispatchResource::existing(*scan_output.id()),
            &vec![0_u32; element_count as usize],
        );
        insert_storage_buffer(
            &device,
            &mut buffers,
            GpuPrimitiveDispatchResource::existing(*source_indices.id()),
            &(0..element_count)
                .map(|index| 1000 + index)
                .collect::<Vec<_>>(),
        );
        insert_storage_buffer(
            &device,
            &mut buffers,
            GpuPrimitiveDispatchResource::existing(*sorted_indices.id()),
            &vec![0_u32; element_count as usize],
        );
        insert_storage_buffer(
            &device,
            &mut buffers,
            GpuPrimitiveDispatchResource::existing(*draw_args.id()),
            &[0_u32; 4],
        );
        for temporary in &dispatch_plan.temporary_storage {
            insert_storage_buffer(
                &device,
                &mut buffers,
                GpuPrimitiveDispatchResource::temporary(temporary.label.clone()),
                &vec![0_u32; temporary.element_count as usize],
            );
        }

        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under workspace root");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("gpu_primitive_runtime_test_encoder"),
        });
        for stage in &dispatch_plan.stages {
            encode_runtime_primitive_stage(&device, &mut encoder, workspace_root, &buffers, stage);
        }
        let scan_readback = copy_storage_buffer_to_readback(
            &device,
            &mut encoder,
            buffers
                .get(&GpuPrimitiveDispatchResource::existing(*scan_output.id()))
                .expect("scan output buffer should exist"),
            u64::from(element_count) * 4,
        );
        let scatter_readback = copy_storage_buffer_to_readback(
            &device,
            &mut encoder,
            buffers
                .get(&GpuPrimitiveDispatchResource::existing(
                    *sorted_indices.id(),
                ))
                .expect("scatter output buffer should exist"),
            u64::from(element_count) * 4,
        );
        let args_readback = copy_storage_buffer_to_readback(
            &device,
            &mut encoder,
            buffers
                .get(&GpuPrimitiveDispatchResource::existing(*draw_args.id()))
                .expect("draw args buffer should exist"),
            DrawIndirectArgs::BYTE_SIZE,
        );
        queue.submit(std::iter::once(encoder.finish()));

        let scan_values = read_u32_buffer(&device, &scan_readback);
        let scatter_values = read_u32_buffer(&device, &scatter_readback);
        let args_values = read_u32_buffer(&device, &args_readback);

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
            .storage_array::<U32ScanElement>("scan.input", u64::from(element_count));
        let (_flow, output) =
            flow.storage_array::<U32ScanElement>("scan.output", u64::from(element_count));
        let scan = U32PrefixScanDescriptor::new(
            "scan",
            input,
            output,
            element_count,
            PrefixScanMode::Exclusive,
        )
        .expect("valid scan descriptor");
        GpuPrimitiveExecutionPlan::new("scan.plan", [GpuPrimitiveStep::from(scan)])
            .expect("valid primitive plan")
            .dispatch_plan()
            .expect("dispatch plan should lower")
            .stage_count()
    }

    fn runtime_primitive_device() -> Option<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..wgpu::InstanceDescriptor::default()
            }
            .with_env(),
        );
        let adapter =
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })) {
                Ok(adapter) => adapter,
                Err(err) => {
                    println!("gpu primitive runtime dispatch test skipped: no adapter: {err}");
                    return None;
                }
            };
        match pollster::block_on(crate::plugins::render::backend::request_device_and_queue(
            &adapter,
        )) {
            Ok((device, queue, _timing)) => Some((device, queue)),
            Err(err) => {
                println!(
                    "gpu primitive runtime dispatch test skipped: device request failed: {err}"
                );
                None
            }
        }
    }

    fn insert_storage_buffer(
        device: &wgpu::Device,
        buffers: &mut BTreeMap<GpuPrimitiveDispatchResource, wgpu::Buffer>,
        resource: GpuPrimitiveDispatchResource,
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
        buffers: &BTreeMap<GpuPrimitiveDispatchResource, wgpu::Buffer>,
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

    fn stage_binding_resources(
        stage: &GpuPrimitiveDispatchStage,
    ) -> Vec<(GpuPrimitiveDispatchResource, bool)> {
        let writable = stage.writes.iter().cloned().collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::<GpuPrimitiveDispatchResource>::new();
        let mut resources = Vec::<(GpuPrimitiveDispatchResource, bool)>::new();
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
