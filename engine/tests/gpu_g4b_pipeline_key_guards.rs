use std::fs;
use std::path::{Path, PathBuf};

const FLOW_KEYS: &str = "src/plugins/render/pipelines/flow_keys.rs";
const BINDINGS: &str = "src/plugins/render/renderer/render_flow/bindings.rs";
const EXECUTION_PLAN: &str = "src/plugins/render/graph/execution_plan.rs";
const PIPELINE_REALIZATION: &str =
    "src/plugins/render/renderer/render_flow/pipeline_realization.rs";
const LOGICAL_OPERATIONS: &str = "src/plugins/render/renderer/render_flow/logical_operations.rs";
const EXECUTE: &str = "src/plugins/render/renderer/render_flow/execute.rs";
const RENDER_FLOW_MOD: &str = "src/plugins/render/renderer/render_flow/mod.rs";
const PIPELINE_CACHE: &str = "src/plugins/render/renderer/pipeline_cache.rs";
const PROGRAM_SOURCES: &str = "src/plugins/render/renderer/render_flow/program_sources.rs";
const PIPELINE_COMPUTE_REALIZATION: &str =
    "src/plugins/gpu/backend/wgpu/pipeline_realization/compute.rs";
const PIPELINE_RENDER_REALIZATION: &str =
    "src/plugins/gpu/backend/wgpu/pipeline_realization/render.rs";
const MATERIAL_COMPILER_BINDINGS: &str = "src/plugins/render/material_compiler/bindings.rs";
const MATERIAL_COMPILER_TYPES: &str = "src/plugins/render/material_compiler/types.rs";
const MATERIAL_WGSL_PROGRAM: &str = "src/plugins/render/material_compiler/wgsl/program.rs";
const MATERIAL_WGSL_PREVIEW: &str = "src/plugins/render/material_compiler/wgsl/preview.rs";
const MATERIAL_WGSL_SCENE: &str = "src/plugins/render/material_compiler/wgsl/scene.rs";
const MATERIAL_WGPU_PREPARE: &str = "src/plugins/render/renderer/prepare.rs";
const MATERIAL_HANDOFF: &str = "../apps/runenwerk_editor/src/material_lab/renderer_handoff.rs";

#[test]
fn renderer_pipeline_key_owns_one_complete_g4b_pipeline_descriptor() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);

    for required in [
        "pub enum FlowPassPipelineDescriptor",
        "Compute(GpuComputePipelineDescriptor)",
        "Render(GpuRenderPipelineDescriptor)",
        "pub pipeline_descriptor: FlowPassPipelineDescriptor",
        "pub fn program(&self) -> &GpuProgramDescriptor",
        "pub fn layout(&self) -> &GpuPipelineLayoutDescriptor",
        "pub fn specialization(&self) -> &GpuSpecializationValueSet",
        "pub fn render_state(&self) -> Option<&GpuRenderPipelineStateDescriptor>",
    ] {
        assert!(
            flow_keys.contains(required),
            "renderer pipeline keys must retain complete generic G4B descriptor authority: {required}"
        );
    }
    assert_eq!(
        flow_keys
            .matches("pub fn pipeline_descriptor_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "complete pipeline diagnostics must derive through one descriptor accessor"
    );
    assert_eq!(
        flow_keys
            .matches("pub fn pipeline_layout_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "pipeline-layout diagnostics must derive through one descriptor-backed accessor"
    );
    assert_eq!(
        flow_keys
            .matches("pub fn primary_bind_group_layout_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "legacy group-0 provenance diagnostics must derive through the complete descriptor"
    );
    assert_eq!(
        flow_keys
            .matches("pub fn render_pipeline_state_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "render-state diagnostics must remain derived through the complete descriptor"
    );
    for forbidden in [
        "pub shader_identity: String",
        "pub shader_revision: u64",
        "pub program_source_key: GpuProgramSourceKey",
        "pub program_source_revision: GpuProgramSourceRevision",
        "pub program_source_identity:",
        "pub pipeline_variant:",
        "FlowPassPipelineVariant",
        "ComputeSpecialization(",
        "pub pipeline_layout:",
        "pub render_pipeline_state:",
        "bind_group_layout_signature_hash",
        "pub primary_bind_group_layout:",
        "pub vertex_layout_signature_hash: u64",
        "pub vertex_input_state: GpuVertexInputStateDescriptor",
        "pub color_formats:",
        "pub depth_format:",
        "pub raster_state_signature_hash:",
        "pub sample_count:",
        "pub primitive_topology_class:",
        "FlowPrimitiveTopologyClass",
        "pub material_specialization_fragment_hash:",
        "pub view_signature_hash:",
        "pub feature_runtime_version:",
        "use wgpu::TextureFormat",
    ] {
        assert!(
            !flow_keys.contains(forbidden),
            "parallel, runtime-only, or untyped pipeline correctness authority returned to FlowPassPipelineKey: {forbidden}"
        );
    }
}

#[test]
fn binding_resolution_constructs_complete_descriptor_from_admitted_source() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bindings = read(&manifest_dir, BINDINGS);
    let resolver = section(
        &bindings,
        "pub(super) fn resolve_compiled_bind_group(",
        ") -> Result<RealizedFlowProgramBindings>",
        BINDINGS,
    );

    assert_eq!(
        resolver
            .matches("program_source: &GpuAdmittedProgramSource")
            .count(),
        1,
        "binding resolution must accept the admitted source record itself"
    );
    assert_eq!(
        resolver
            .matches("specialization: GpuSpecializationValueSet")
            .count(),
        1,
        "binding resolution must accept one normalized typed specialization set"
    );
    assert_eq!(
        bindings.matches("GpuProgramDescriptor::new(").count(),
        2,
        "compute and render paths must construct admitted generic program descriptors"
    );
    assert_eq!(
        bindings
            .matches("GpuComputePipelineDescriptor::new(")
            .count(),
        1,
        "compute passes must publish one complete generic compute descriptor"
    );
    assert_eq!(
        bindings
            .matches("GpuRenderPipelineDescriptor::new(")
            .count(),
        1,
        "render passes must publish one complete generic render descriptor"
    );
    assert_eq!(
        bindings
            .matches("GpuPipelineLayoutDescriptor::from_interface(program.interface())?")
            .count(),
        2,
        "compute and render layouts must derive from the admitted program interface"
    );
    assert!(
        bindings.contains("pipeline_descriptor,"),
        "the complete generic descriptor must enter the renderer key exactly once"
    );
    for forbidden in [
        "GpuProgramInterfaceDescriptor",
        "GpuBindingDeclaration",
        "GpuBindGroupLayoutDescriptor::new(",
        "GpuPipelineLayoutDescriptor::new(",
        "program_source_identity: &GpuProgramSourceIdentity",
        "program_source_identity: program_source_identity.clone(),",
        "pipeline_variant: FlowPassPipelineVariant",
        ".program_source_identity(",
        "GpuProgramSourceKey::new(",
        "shader_identity: &str",
        "shader_revision: u64",
        "split_shader_pipeline_identity(",
        "COMPUTE_SPECIALIZATION_SEPARATOR",
    ] {
        assert!(
            !bindings.contains(forbidden),
            "pre-admission or parallel pipeline authority returned to binding resolution: {forbidden}"
        );
    }
}

#[test]
fn renderer_runtime_hashes_are_diagnostic_only() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);
    let bindings = read(&manifest_dir, BINDINGS);
    let execute = read(&manifest_dir, EXECUTE);

    for forbidden in [
        "material_specialization_fragment_hash",
        "view_signature_hash",
        "feature_runtime_version",
    ] {
        assert!(
            !flow_keys.contains(forbidden),
            "renderer runtime diagnostic partition returned to pipeline-key identity: {forbidden}"
        );
    }
    for forbidden in [
        "material_specialization_fragment_hash(",
        "hash_view_signature(",
        "feature_runtime_version(",
    ] {
        assert!(
            !bindings.contains(forbidden),
            "binding resolution must not use renderer runtime diagnostics to partition backend artifacts: {forbidden}"
        );
    }
    for required in [
        "material_specialization_fragment_hash(",
        "hash_view_signature(",
        "feature_runtime_version(",
    ] {
        assert!(
            execute.contains(required),
            "renderer provenance must retain the runtime diagnostic projection: {required}"
        );
    }
    for forbidden in [
        "key.material_specialization_fragment_hash",
        "key.view_signature_hash",
        "key.feature_runtime_version",
    ] {
        assert!(
            !execute.contains(forbidden),
            "renderer provenance must not recover runtime diagnostics from pipeline-key authority: {forbidden}"
        );
    }
}

#[test]
fn admitted_program_interface_owns_pipeline_layout_before_descriptor_publication() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);
    let bindings = read(&manifest_dir, BINDINGS);
    let execute = read(&manifest_dir, EXECUTE);
    let render_flow_mod = read(&manifest_dir, RENDER_FLOW_MOD);

    for required in [
        "key: GpuBindingKey",
        "refinement: Option<GpuBindingLayoutRefinement>",
        "GpuBindingLayoutRefinement::new(*key)",
        ".with_texture_sample_class(GpuTextureSampleClass::FloatFilterable)",
        ".with_sampler_class(GpuSamplerClass::Filtering)",
        "gpu_material_binding_refinements_for_pass(",
    ] {
        assert!(
            bindings.contains(required),
            "renderer binding resolution must retain typed keys and sparse host/layout policy: {required}"
        );
    }
    assert_eq!(
        bindings
            .matches("GpuPipelineLayoutDescriptor::from_interface(program.interface())?")
            .count(),
        2,
        "compute and render pipeline layouts must derive from admitted program interfaces"
    );
    for forbidden in [
        "GpuProgramInterfaceDescriptor",
        "GpuBindingDeclaration",
        "GpuBindingKind",
        "GpuBindGroupLayoutDescriptor::new(",
        "GpuPipelineLayoutDescriptor::new(",
        "gpu_program_interface_for_layout(",
    ] {
        assert!(
            !bindings.contains(forbidden),
            "renderer must not regain caller-authored shader-interface/layout authority through {forbidden}"
        );
    }
    for required in [
        "GpuBindingKey::try_new(",
        "u64::from(binding.bind_group)",
        "u64::from(binding.texture_binding)",
        "u64::from(binding.sampler_binding)",
        "GpuTextureSampleClass::FloatFilterable",
        "GpuSamplerClass::Filtering",
        "GpuTextureViewDimension::D2",
        "GpuTextureViewDimension::D3",
    ] {
        assert!(
            bindings.contains(required),
            "material runtime/refinement lowering must consume compiler-published typed coordinates: {required}"
        );
    }
    for required in [
        "context.realize_program(pipeline_key.pipeline_descriptor.program())",
        "context.realize_pipeline_layout(pipeline_key.pipeline_descriptor.layout())",
        "GpuRuntimeBindingSet::new(",
        "pipeline_key.pipeline_descriptor.layout().clone()",
        "runtime_bindings,",
    ] {
        assert!(
            bindings.contains(required),
            "G4C2 descriptor realization/runtime validation must consume the admitted program-derived layout: {required}"
        );
    }
    for retired_physical_sidecar in [
        "context.realize_bind_group_layout(",
        "context.realize_bind_group(",
    ] {
        assert!(
            !bindings.contains(retired_physical_sidecar),
            "renderer must leave execution-owned binding materialization to G5 preparation: {retired_physical_sidecar}"
        );
    }
    for forbidden in [
        "layout_ty: BindingType",
        "hash_bind_group_layout_entries(",
        "bind_group_layout_signature_hash:",
        "primary_bind_group_layout: primary_bind_group_layout",
    ] {
        assert!(
            !bindings.contains(forbidden),
            "superseded raw or group-0-only pipeline-layout authority returned: {forbidden}"
        );
    }
    assert!(
        !flow_keys.contains("pub primary_bind_group_layout:"),
        "group-0-only layout authority must not return to FlowPassPipelineKey"
    );
    assert!(
        flow_keys.contains("diagnostic_hash(&self.pipeline_descriptor.layout().group(0))"),
        "legacy group-0 provenance diagnostics must be projections of the complete descriptor"
    );
    assert_eq!(
        execute
            .matches("FlowPassPipelineKey::primary_bind_group_layout_diagnostic_hash")
            .count(),
        1,
        "execution provenance must derive its group-0 diagnostic hash from descriptor-backed layout truth"
    );
    assert!(
        !execute.contains(".bind_group_layout_signature_hash"),
        "execution must not read a removed raw layout-hash field from the pipeline key"
    );
    for forbidden in [
        "compiled_storage_access_to_storage_texture_access,",
        "hash_bind_group_layout_entries,",
    ] {
        assert!(
            !render_flow_mod.contains(forbidden),
            "superseded render-flow re-export returned: {forbidden}"
        );
    }
}

#[test]
fn material_shader_binding_coordinates_have_one_compiler_allocation_owner() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let compiler = read(&manifest_dir, MATERIAL_COMPILER_BINDINGS);
    let compiler_types = read(&manifest_dir, MATERIAL_COMPILER_TYPES);
    let wgsl_program = read(&manifest_dir, MATERIAL_WGSL_PROGRAM);
    let wgsl_preview = read(&manifest_dir, MATERIAL_WGSL_PREVIEW);
    let wgsl_scene = read(&manifest_dir, MATERIAL_WGSL_SCENE);
    let handoff = read(&manifest_dir, MATERIAL_HANDOFF);
    let g4b = read(&manifest_dir, BINDINGS);
    let renderer_prepare = read(&manifest_dir, MATERIAL_WGPU_PREPARE);

    let compact = |source: &str| {
        source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
    };
    let handoff_production = handoff
        .split_once("#[cfg(test)]\nmod tests {")
        .map(|(production, _)| production)
        .expect("Material Lab handoff must keep its test module separate from production code");
    let handoff_compact = compact(handoff_production);
    let g4b_refinements = section(
        &g4b,
        "fn gpu_material_binding_refinements(",
        "fn gpu_render_pipeline_state_for_pass(",
        BINDINGS,
    );
    let g4b_compact = compact(g4b_refinements);
    let material_prepare = section(
        &renderer_prepare,
        "fn prepare_material_gpu_resources(",
        "fn resolve_ui_prepared_with_gate(",
        MATERIAL_WGPU_PREPARE,
    );
    let material_prepare_compact = compact(material_prepare);

    for required in [
        "pub bind_group: u32",
        "pub texture_binding: u32",
        "pub sampler_binding: u32",
    ] {
        assert!(
            compiler_types.contains(required),
            "compiler output must publish exact material shader coordinates: {required}"
        );
    }
    assert!(
        compiler.contains("binding.bind_group")
            && compiler.contains("binding.texture_binding")
            && compiler.contains("binding.sampler_binding"),
        "compiler WGSL declarations must consume published material binding coordinates"
    );
    assert!(
        wgsl_program.contains("texture_binding_variable(binding)")
            && wgsl_program.contains("sampler_binding_variable(binding)"),
        "material WGSL expressions must consume the compiler-published binding record"
    );
    assert!(
        wgsl_preview.contains("material_resource_declarations(&program.resource_bindings)")
            && wgsl_scene.contains("material_resource_declarations(&program.resource_bindings)"),
        "preview and scene WGSL generation must consume compiler-published resource bindings"
    );
    assert!(
        handoff.contains("compiler_resource_bindings")
            && handoff.contains("compiler_binding_for_resolved_resource"),
        "Material Lab must transport and semantically join compiler binding evidence"
    );
    assert!(
        g4b_compact.contains(
            "GpuBindingLayoutRefinement::new(GpuBindingKey::try_new(u64::from(binding.bind_group),u64::from(binding.texture_binding),)?)"
        ) && g4b_compact.contains(
            "GpuBindingLayoutRefinement::new(GpuBindingKey::try_new(u64::from(binding.bind_group),u64::from(binding.sampler_binding),)?)"
        ),
        "renderer material refinements must target exact compiler-published binding coordinates"
    );
    assert!(
        g4b_compact.contains(
            ".with_texture_sample_class(GpuTextureSampleClass::FloatFilterable)"
        ) && g4b_compact.contains(".with_sampler_class(GpuSamplerClass::Filtering)"),
        "renderer material policy must remain sparse filterability/sampler refinements"
    );
    for forbidden in [
        "material_wgpu_binding_indices",
        "GpuBindGroupLayoutDescriptor::new(",
        "GpuRuntimeBindingValue::",
        "context.realize_bind_group_layout(",
        "context.realize_bind_group(",
    ] {
        assert!(
            !material_prepare.contains(forbidden),
            "renderer material preparation must not restore duplicate G4B/G4C2 binding authority through {forbidden:?}"
        );
    }

    for (path, source) in [
        (MATERIAL_HANDOFF, handoff_compact.as_str()),
        (BINDINGS, g4b_compact.as_str()),
        (MATERIAL_WGPU_PREPARE, material_prepare_compact.as_str()),
    ] {
        assert_no_resource_slot_coordinate_arithmetic(path, source);
    }
}

#[test]
fn render_pipeline_state_is_typed_before_complete_descriptor_publication() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);
    let bindings = read(&manifest_dir, BINDINGS);
    let execution_plan = read(&manifest_dir, EXECUTION_PLAN);
    let pipeline_realization = read(&manifest_dir, PIPELINE_REALIZATION);
    let execute = read(&manifest_dir, EXECUTE);
    let render_flow_mod = read(&manifest_dir, RENDER_FLOW_MOD);

    assert_eq!(
        bindings
            .matches(
                "gpu_render_pipeline_state_for_pass(flow, pass_id, &color_formats, depth_format)?"
            )
            .count(),
        1,
        "binding resolution must normalize one aggregate render-pipeline state before publishing the descriptor"
    );
    assert!(
        bindings.contains("GpuRenderPipelineDescriptor::new("),
        "aggregate render state must enter the complete generic render descriptor"
    );
    for required in [
        "GpuRenderPipelineStateDescriptor::new(",
        "GpuColorTargetStateDescriptor::new(",
        "GpuPrimitiveStateDescriptor::new(",
        "GpuDepthStencilStateDescriptor::new(",
        "GpuMultisampleStateDescriptor::default()",
        "GpuVertexBufferLayoutDescriptor::new(",
        "GpuVertexAttribute::new(",
        "GpuVertexInputStateDescriptor::new(layouts)",
    ] {
        assert!(
            bindings.contains(required),
            "renderer render state is not normalized through complete typed G4B state: {required}"
        );
    }
    for forbidden in [
        "pub render_pipeline_state:",
        "pub vertex_input_state: GpuVertexInputStateDescriptor",
        "pub color_formats:",
        "pub depth_format:",
        "pub raster_state_signature_hash:",
        "pub sample_count:",
        "pub primitive_topology_class:",
        "FlowPrimitiveTopologyClass",
    ] {
        assert!(
            !flow_keys.contains(forbidden),
            "separate render-state correctness authority returned to the pipeline key: {forbidden}"
        );
    }
    for (path, source) in [
        (BINDINGS, bindings.as_str()),
        (EXECUTION_PLAN, execution_plan.as_str()),
        (PIPELINE_REALIZATION, pipeline_realization.as_str()),
        (RENDER_FLOW_MOD, render_flow_mod.as_str()),
    ] {
        for forbidden in [
            "raster_state_signature_hash",
            "primitive_topology_class",
            "FlowPrimitiveTopologyClass",
        ] {
            assert!(
                !source.contains(forbidden),
                "superseded raw render-state authority returned in {path}: {forbidden}"
            );
        }
    }
    assert!(
        !execution_plan.contains("pub fn signature_hash(self)"),
        "compiled raster state must not retain a naked hash producer after aggregate-state adoption"
    );

    assert_eq!(
        execute
            .matches("FlowPassPipelineKey::render_pipeline_state")
            .count(),
        4,
        "execution provenance must derive each render-state diagnostic from complete descriptor authority"
    );
    for forbidden in [
        "key.render_pipeline_state.as_ref()",
        "key.color_formats",
        "key.depth_format",
        "key.sample_count",
        "key.primitive_topology_class",
        "FlowPrimitiveTopologyClass",
    ] {
        assert!(
            !execute.contains(forbidden),
            "execution provenance returned to removed raw render-state key authority: {forbidden}"
        );
    }

    for (path, source) in [
        (FLOW_KEYS, flow_keys.as_str()),
        (BINDINGS, bindings.as_str()),
        (EXECUTION_PLAN, execution_plan.as_str()),
        (PIPELINE_REALIZATION, pipeline_realization.as_str()),
    ] {
        assert!(
            !source.contains("vertex_layout_signature_hash"),
            "superseded naked vertex-layout hash authority returned in {path}"
        );
    }
}

#[test]
fn wgpu_pipeline_semantics_project_from_complete_g4b_descriptors() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pipeline_passes = read(&manifest_dir, PIPELINE_REALIZATION);
    let logical_operations = read(&manifest_dir, LOGICAL_OPERATIONS);
    let bindings = read(&manifest_dir, BINDINGS);
    let compute_realization = read(&manifest_dir, PIPELINE_COMPUTE_REALIZATION);
    let render_realization = read(&manifest_dir, PIPELINE_RENDER_REALIZATION);
    let realization = section(
        &pipeline_passes,
        "pub(in crate::plugins::render::renderer::render_flow) fn realize_compiled_pass(",
        "fn resolve_color_target_format_from_plan(",
        PIPELINE_REALIZATION,
    );
    let paths = [
        (
            "compute",
            "CompiledPassExecutionPlan::Compute(value) => {",
            "CompiledPassExecutionPlan::Fullscreen(value) => {",
            "context.realize_compute_pipeline(",
            "fn encode_compute_pass(",
            "fn encode_fullscreen_pass(",
            ".for_compute_pipeline(",
            "PreparedFlowPipeline::Compute",
        ),
        (
            "fullscreen",
            "CompiledPassExecutionPlan::Fullscreen(value) => {",
            "CompiledPassExecutionPlan::Graphics(value) => {",
            "context.realize_render_pipeline(",
            "fn encode_fullscreen_pass(",
            "fn encode_graphics_pass(",
            ".for_render_pipeline(",
            "PreparedFlowPipeline::Render",
        ),
        (
            "graphics",
            "CompiledPassExecutionPlan::Graphics(value) => {",
            "CompiledPassExecutionPlan::Copy(_)",
            "context.realize_render_pipeline(",
            "fn encode_graphics_pass(",
            "struct EncodeComputePipeline",
            ".for_render_pipeline(",
            "PreparedFlowPipeline::Render",
        ),
    ];
    for (
        label,
        realization_start,
        realization_end,
        pipeline_realization_token,
        _encode_start,
        _encode_end,
        _execution_bridge_token,
        prepared_pipeline_kind,
    ) in paths
    {
        let realized = section(
            realization,
            realization_start,
            realization_end,
            PIPELINE_REALIZATION,
        );
        assert!(
            realized.contains(".resolve_compiled_bind_group("),
            "{label} must resolve its complete G4B/G4C2 descriptor dependencies before pipeline realization"
        );
        assert!(
            realized.contains(pipeline_realization_token),
            "{label} must delegate pipeline realization to the private G4C3 authority"
        );
        for forbidden in [
            ".for_pipeline_creation(",
            ".create_compute_pipeline(",
            ".create_render_pipeline(",
            "ShaderSource::Wgsl(",
        ] {
            assert!(
                !realized.contains(forbidden),
                "{label} renderer realization must not regain private backend pipeline authority through {forbidden:?}"
            );
        }
        let canonical_operation = if label == "compute" {
            "GpuWorkOperation::Compute(operation)"
        } else {
            "GpuWorkOperation::Render(GpuRenderOperation::new("
        };
        assert!(
            logical_operations.contains(prepared_pipeline_kind)
                && logical_operations.contains(canonical_operation),
            "{label} canonical G5 operation must consume the opaque pipeline realized by G4C3"
        );
        for forbidden in [
            ".resolve_compiled_bind_group(",
            "context.realize_compute_pipeline(",
            "context.realize_render_pipeline(",
            ".for_pipeline_creation(",
            ".create_compute_pipeline(",
            ".create_render_pipeline(",
        ] {
            assert!(
                !logical_operations.contains(forbidden),
                "{label} G5 operation projection must not re-enter G4 realization through {forbidden:?}"
            );
        }
    }
    for required in [
        "context.realize_program(pipeline_key.pipeline_descriptor.program())",
        "context.realize_pipeline_layout(",
    ] {
        assert!(
            bindings.contains(required),
            "G4C2 binding resolution must own {required:?}"
        );
    }
    for retired_physical_sidecar in [
        "context.realize_bind_group_layout(",
        "context.realize_bind_group(",
    ] {
        assert!(
            !bindings.contains(retired_physical_sidecar),
            "G5 preparation must own execution-time binding materialization: {retired_physical_sidecar}"
        );
    }

    for required in [
        ".create_compute_pipeline(&ComputePipelineDescriptor",
        "entry_point: Some(descriptor.entry_point().as_str())",
        "wgpu_specialization_constants(descriptor)",
        "layout: Some(layout.record.wgpu_object())",
        "module: program.record.wgpu_object()",
    ] {
        assert!(
            compute_realization.contains(required),
            "private G4C3 compute realization must project complete descriptor semantics: {required}"
        );
    }
    for required in [
        ".create_render_pipeline(&RenderPipelineDescriptor",
        "entry_point: Some(descriptor.entry_points().vertex().as_str())",
        "descriptor.entry_points()",
        ".fragment()",
        "wgpu_specialization_constants(descriptor)",
        "layout: Some(layout.record.wgpu_object())",
        "module: program.record.wgpu_object()",
        "targets: lowered.color_targets.as_slice()",
        "primitive: lowered.primitive",
        "depth_stencil: lowered.depth_stencil.clone()",
        "multisample: lowered.multisample",
    ] {
        assert!(
            render_realization.contains(required),
            "private G4C3 render realization must project complete descriptor semantics: {required}"
        );
    }
    for forbidden in [
        "CurrentRenderPipelineCreationTerminal",
        ".for_pipeline_creation(",
        ".create_compute_pipeline(",
        ".create_render_pipeline(",
        "ShaderSource::Wgsl(",
    ] {
        assert!(
            !pipeline_passes.contains(forbidden),
            "renderer pipeline consumer must not regain private or predecessor pipeline authority through {forbidden:?}"
        );
    }
}

#[test]
fn renderer_source_authority_normalizes_identity_only_during_admission() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = read(&manifest_dir, PIPELINE_CACHE);
    let authority = read(&manifest_dir, PROGRAM_SOURCES);

    assert!(
        !cache.contains("pub(crate) fn program_source_identity("),
        "the renderer cache must not expose a pre-admission identity gateway"
    );
    assert_eq!(
        cache.matches("pub(crate) fn admit_program_source(").count(),
        1,
        "the renderer cache must expose one admitted-source gateway"
    );
    let cache_gateway = section(
        &cache,
        "pub(crate) fn admit_program_source(",
        "pub fn retain_flows(",
        PIPELINE_CACHE,
    );
    assert_eq!(
        cache_gateway.matches(".admit_wgsl(").count(),
        1,
        "the cache admission gateway must delegate to the one source authority"
    );
    for forbidden in [
        "admit_and_retain_wgsl",
        "retained_sources",
        "program_source_retentions",
    ] {
        assert!(
            !cache.contains(forbidden) && !authority.contains(forbidden),
            "renderer-lifetime source retention must not return: {forbidden}"
        );
    }
    let retirement = section(
        &cache,
        "pub fn retain_flows(",
        "fn admit_builtin_program_source(",
        PIPELINE_CACHE,
    );
    assert_eq!(
        retirement
            .matches("self.program_sources.collect_unretained()")
            .count(),
        1,
        "flow retirement must collect lookup-only source records after cache-key removal"
    );

    assert_eq!(
        authority.matches("pub(crate) fn identity(").count(),
        1,
        "the renderer source authority must define one identity-normalization operation"
    );
    let authority_identity = section(
        &authority,
        "pub(crate) fn identity(",
        "pub(crate) fn admit_wgsl(",
        PROGRAM_SOURCES,
    );
    assert_eq!(
        authority_identity
            .matches("renderer_revision.checked_add(1)")
            .count(),
        1,
        "zero-based renderer revisions must normalize exactly once in the authority"
    );
    let admission = section(
        &authority,
        "pub(crate) fn admit_wgsl(",
        "pub(crate) fn collect_unretained(",
        PROGRAM_SOURCES,
    );
    assert_eq!(
        admission
            .matches("self.identity(key, renderer_revision)?")
            .count(),
        1,
        "source admission must use the one identity-normalization operation"
    );
}

fn read(manifest_dir: &Path, relative: &str) -> String {
    fs::read_to_string(manifest_dir.join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

fn section<'a>(source: &'a str, start: &str, end: &str, path: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("{path} no longer contains start marker {start:?}"));
    let tail = &source[start_index..];
    let end_index = tail
        .find(end)
        .unwrap_or_else(|| panic!("{path} no longer contains end marker {end:?} after {start:?}"));
    &tail[..end_index]
}

fn assert_no_resource_slot_coordinate_arithmetic(path: &str, source: &str) {
    for operand in [
        "resource_slot_index",
        "binding.resource_slot_index",
        "compiler_binding.resource_slot_index",
    ] {
        for operator in ["*", "+", "-", "/", "%"] {
            for forbidden in [
                format!("{operand}{operator}"),
                format!("{operator}{operand}"),
            ] {
                assert!(
                    !source.contains(&forbidden),
                    "downstream material shader binding identity must not be reconstructed from resource slots in {path}: {forbidden}"
                );
            }
        }
        for method_prefix in ["saturating_", "checked_", "wrapping_", "overflowing_"] {
            let forbidden = format!("{operand}.{method_prefix}");
            assert!(
                !source.contains(&forbidden),
                "downstream material shader binding identity must not be reconstructed from resource slots in {path}: {forbidden}"
            );
        }
    }
}
