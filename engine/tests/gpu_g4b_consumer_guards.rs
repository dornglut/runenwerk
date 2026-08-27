use std::fs;
use std::path::{Path, PathBuf};

const PASS_GRAPH: &str = "src/plugins/render/graph/pass_graph.rs";
const PASS_BINDINGS: &str = "src/plugins/render/api/bindings.rs";
const EXECUTION_PLAN: &str = "src/plugins/render/graph/execution_plan.rs";
const RUNTIME_BINDINGS: &str = "src/plugins/render/renderer/render_flow/bindings.rs";
const RENDERER_SETUP: &str = "src/plugins/render/renderer/setup.rs";
const PRIMITIVE_PLAN: &str = "src/plugins/render/gpu_primitives/plan.rs";
const PIPELINE_REALIZATION: &str =
    "src/plugins/render/renderer/render_flow/pipeline_realization.rs";
const LOGICAL_OPERATIONS: &str = "src/plugins/render/renderer/render_flow/logical_operations.rs";
const PIPELINE_CACHE: &str = "src/plugins/render/renderer/pipeline_cache.rs";
const PROGRAM_SOURCES: &str = "src/plugins/render/renderer/render_flow/program_sources.rs";
const PROGRAM_MODULE: &str = "src/plugins/gpu/api/program.rs";
const PROGRAM_ANALYSIS: &str = "src/plugins/gpu/api/program/analysis.rs";
const PROGRAM_DESCRIPTOR: &str = "src/plugins/gpu/api/program/descriptor.rs";
const PROGRAM_ENTRY_POINT: &str = "src/plugins/gpu/api/program/entry_point.rs";
const PROGRAM_INTERFACE: &str = "src/plugins/gpu/api/program/interface.rs";
const PROGRAM_STAGE_IO: &str = "src/plugins/gpu/api/program/stage_io.rs";
const PROGRAM_REALIZATION: &str = "src/plugins/gpu/backend/wgpu/program_binding_realization/mod.rs";
const PROGRAM_REALIZATION_RECORDS: &str =
    "src/plugins/gpu/backend/wgpu/program_binding_realization/records.rs";
const FRAGMENTS: &str = "src/plugins/render/composition/fragments.rs";
const FRAGMENT_VALIDATION: &str = "src/plugins/render/composition/fragment_validation.rs";
const FRAGMENT_MERGE: &str = "src/plugins/render/graph/merge.rs";
const FLOW_VALIDATION: &str = "src/plugins/render/graph/validation.rs";

#[test]
fn renderer_shader_binding_identity_is_explicit_and_never_vector_derived() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let authoring = read(&manifest_dir, PASS_BINDINGS);
    let execution = read(&manifest_dir, EXECUTION_PLAN);
    let runtime = read(&manifest_dir, RUNTIME_BINDINGS);
    let primitives = read(&manifest_dir, PRIMITIVE_PLAN);

    assert!(
        authoring.contains("key: GpuBindingKey"),
        "render shader bindings must retain typed G4B binding identity at the authoring boundary"
    );
    assert!(
        authoring.contains("pub const fn key(&self) -> GpuBindingKey"),
        "render shader bindings must expose their typed key without reconstructing it downstream"
    );

    let compile_bindings = section(
        &execution,
        "fn compile_pass_bindings(",
        "fn collect_storage_usage(",
        EXECUTION_PLAN,
    );
    assert!(
        compile_bindings.contains("entries.sort_by_key(CompiledBindingEntry::key)"),
        "compiled bind-group entries must normalize by retained typed binding key"
    );
    assert!(
        !compile_bindings.contains(".enumerate()"),
        "compiled shader binding identity must never derive from vector position"
    );
    assert!(
        compile_bindings.contains("let key = binding.key();"),
        "compiled shader bindings must retain the authoring key"
    );

    let runtime_bindings = section(
        &runtime,
        "pub(super) fn resolve_compiled_bind_group",
        "fn resolved_binding_texture_view(",
        RUNTIME_BINDINGS,
    );
    assert!(
        !runtime_bindings.contains(".enumerate()"),
        "runtime resources and G4C2 binding values must not rebuild binding identity from vector position"
    );
    assert!(
        runtime_bindings.contains("runtime_binding_value(value, sampler.as_ref())"),
        "runtime resources must pass their retained typed keys into G4C2 binding realization"
    );
    let runtime_binding_value = section(
        &runtime,
        "fn runtime_binding_value(",
        "fn gpu_pipeline_descriptor_for_pass(",
        RUNTIME_BINDINGS,
    );
    assert!(
        runtime_binding_value.contains("GpuRuntimeBindingValue::new(value.key, [resource])"),
        "G4C2 runtime binding values must retain the authored typed binding key"
    );
    assert!(
        !runtime.contains("BindGroupEntry"),
        "renderer runtime binding realization must not recreate raw WGPU bind-group entries"
    );

    assert!(
        primitives.contains("shader_bindings: Vec<GpuPrimitiveShaderBinding>"),
        "generated GPU primitive stages must retain explicit shader binding records"
    );
    assert!(
        primitives.contains("GpuBindingKey::try_new(0, binding)"),
        "built-in primitive WGSL bindings must be encoded as typed keys at stage construction"
    );
    assert!(
        !primitives.contains("fn stage_binding_resources("),
        "primitive runtime proof must not reconstruct shader bindings from read/write vector order"
    );
}

#[test]
fn public_program_api_has_one_compiler_derived_authority() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let program_module = read(&manifest_dir, PROGRAM_MODULE);
    let descriptor = read(&manifest_dir, PROGRAM_DESCRIPTOR);
    let entry_point = read(&manifest_dir, PROGRAM_ENTRY_POINT);
    let interface = read(&manifest_dir, PROGRAM_INTERFACE);
    let stage_io = read(&manifest_dir, PROGRAM_STAGE_IO);

    assert!(program_module.contains("mod analysis;"));
    assert!(!program_module.contains("pub mod analysis;"));
    assert!(!program_module.contains("pub use analysis"));

    let constructor = section(
        &descriptor,
        "    pub fn new(",
        "    pub fn source(",
        PROGRAM_DESCRIPTOR,
    );
    assert!(
        constructor.contains("selected_entry_points: impl IntoIterator<Item = GpuEntryPointName>")
    );
    assert!(
        constructor.contains("refinements: impl IntoIterator<Item = GpuBindingLayoutRefinement>")
    );
    assert!(!constructor.contains("GpuProgramInterfaceDescriptor"));
    assert!(!constructor.contains("GpuEntryPointDescriptor"));

    let entry_descriptor = entry_point
        .split_once("impl GpuEntryPointDescriptor {")
        .map(|(_, tail)| tail)
        .expect("entry-point inspection record implementation must remain present");
    assert!(entry_descriptor.contains("pub(crate) const fn derived("));
    assert!(!entry_descriptor.contains("pub fn new("));

    assert!(!interface.contains("mod observed;"));
    assert!(!interface.contains("mod comparison;"));
    assert!(stage_io.contains("pub(crate) use signature::{GpuObserved"));
    assert!(!stage_io.contains("pub use signature::{GpuObserved"));
    assert!(!stage_io.contains("pub use builtin::*"));
    assert!(!stage_io.contains("pub use comparison::*"));

    for (path, source) in [
        (PROGRAM_MODULE, &program_module),
        (PROGRAM_DESCRIPTOR, &descriptor),
        (PROGRAM_ENTRY_POINT, &entry_point),
        (PROGRAM_INTERFACE, &interface),
        (PROGRAM_STAGE_IO, &stage_io),
    ] {
        assert!(
            !source.contains("naga::") && !source.contains("wgpu::"),
            "{path} must not leak Naga or WGPU through the public program API surface"
        );
    }
}

#[test]
fn renderer_program_interface_authority_is_compiler_derived_once() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let runtime = read(&manifest_dir, RUNTIME_BINDINGS);
    let setup = read(&manifest_dir, RENDERER_SETUP);
    let analysis = read(&manifest_dir, PROGRAM_ANALYSIS);
    let realization = read(&manifest_dir, PROGRAM_REALIZATION);
    let realization_records = read(&manifest_dir, PROGRAM_REALIZATION_RECORDS);

    for (path, source) in [(RUNTIME_BINDINGS, &runtime), (RENDERER_SETUP, &setup)] {
        assert!(
            source.contains("GpuProgramDescriptor::new("),
            "{path} must admit programs from canonical WGSL plus selected entry names"
        );
        assert!(
            source.contains("GpuPipelineLayoutDescriptor::from_interface(program")
                || source
                    .contains("GpuPipelineLayoutDescriptor::from_interface(\n            program"),
            "{path} must derive pipeline layout from the admitted program interface"
        );
        for forbidden in [
            "GpuProgramInterfaceDescriptor",
            "GpuBindingDeclaration",
            "GpuEntryPointDescriptor",
            "gpu_shader_stages_from_wgpu",
            "GpuShaderStage::",
            "ShaderStages::COMPUTE",
            "ShaderStages::VERTEX_FRAGMENT",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} must not regain caller-authored shader-interface authority through {forbidden:?}"
            );
        }
    }

    assert!(
        runtime.contains("GpuBindingLayoutRefinement"),
        "render-flow runtime policy must use sparse host/layout refinements"
    );
    assert!(
        setup.contains("GpuBindingLayoutRefinement"),
        "built-in UI setup must use sparse host/layout refinements"
    );
    for required in [
        "naga::front::wgsl::parse_str",
        "module_info.get_entry_point",
        "GpuEntryPointDescriptor::derived",
    ] {
        assert!(
            analysis.contains(required),
            "logical program admission must own compiler analysis fact {required:?}"
        );
    }
    for forbidden in [
        "mod evidence;",
        "naga::front::wgsl::parse_str",
        "validate_and_normalize(",
        "observed_interface",
        "vertex_inputs",
        "fragment_outputs",
    ] {
        assert!(
            !realization.contains(forbidden) && !realization_records.contains(forbidden),
            "private WGPU realization must not regain logical/reflection authority through {forbidden:?}"
        );
    }
}

#[test]
fn fragment_composition_preserves_explicit_shader_binding_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fragments = read(&manifest_dir, FRAGMENTS);
    let validation = read(&manifest_dir, FRAGMENT_VALIDATION);
    let merge = read(&manifest_dir, FRAGMENT_MERGE);

    assert!(
        fragments.contains("pub shader_bindings: Vec<RenderFragmentShaderBinding>"),
        "fragment pass descriptors must retain explicit shader binding records"
    );
    for required in [
        "texture_binding: GpuBindingKey",
        "sampler_binding: GpuBindingKey",
        "binding: GpuBindingKey",
    ] {
        assert!(
            fragments.contains(required),
            "fragment shader binding identity is missing {required:?}"
        );
    }
    assert!(
        validation.contains("fn validate_shader_binding_identity("),
        "fragment validation must own shader-binding identity validation"
    );
    assert!(
        validation.contains("if key.group() != 0"),
        "fragment bindings must reject groups outside the current logical group-0 contract"
    );
    assert!(
        validation.contains("if !keys.insert(key)"),
        "fragment bindings must reject duplicate typed keys before merge"
    );
    assert!(
        validation.contains(
            "sampled-texture resource roles and explicit shader bindings must match exactly"
        ),
        "fragment sampled-texture role truth must not drift from explicit shader bindings"
    );
    assert!(
        validation.contains(
            "storage-texture resource roles and explicit shader bindings must match exactly"
        ),
        "fragment storage-texture role truth must not drift from explicit shader bindings"
    );

    let merge_pass = section(
        &merge,
        "fn merge_pass_into_flow(",
        "fn apply_compute_view_scope(",
        FRAGMENT_MERGE,
    );
    assert!(
        merge_pass.contains("for binding in &pass.shader_bindings"),
        "fragment merge must lower retained explicit shader binding records"
    );
    assert!(
        !merge_pass.contains("for sample in &pass.sample_textures"),
        "fragment merge must not reconstruct sampled-texture slot identity from resource vectors"
    );
    assert!(
        !merge_pass.contains("for write in &pass.write_textures"),
        "fragment merge must not reconstruct storage-texture slot identity from resource vectors"
    );
}

#[test]
fn render_flow_validates_explicit_shader_binding_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let validation = read(&manifest_dir, FLOW_VALIDATION);

    assert!(
        validation.contains("fn validate_pass_shader_binding_identity("),
        "direct RenderFlow validation must own explicit shader-binding identity checks"
    );
    assert!(
        validation.contains("if key.group() != 0"),
        "direct RenderFlow bindings must reject groups outside the current logical group-0 contract"
    );
    assert!(
        validation.contains("if !keys.insert(key)"),
        "direct RenderFlow bindings must reject duplicate typed keys before descriptor publication"
    );
}

#[test]
fn resolved_renderer_programs_admit_and_realize_before_g5_pipeline_consumption() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = read(&manifest_dir, PIPELINE_REALIZATION);
    let operations = read(&manifest_dir, LOGICAL_OPERATIONS);
    let bindings = read(&manifest_dir, RUNTIME_BINDINGS);
    for required in [
        "context.realize_program(",
        "context.realize_pipeline_layout(",
    ] {
        assert!(
            bindings.contains(required),
            "renderer binding resolution must delegate {required:?} to the G4C2 realization authority"
        );
    }
    for retired_physical_sidecar in [
        "context.realize_bind_group_layout(",
        "context.realize_bind_group(",
    ] {
        assert!(
            !bindings.contains(retired_physical_sidecar),
            "renderer must leave execution-owned binding materialization to G5 preparation: {retired_physical_sidecar:?}"
        );
    }

    let realization = section(
        &source,
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
            "PreparedFlowPipeline::Compute",
        ),
        (
            "fullscreen",
            "CompiledPassExecutionPlan::Fullscreen(value) => {",
            "CompiledPassExecutionPlan::Graphics(value) => {",
            "context.realize_render_pipeline(",
            "PreparedFlowPipeline::Render",
        ),
        (
            "graphics",
            "CompiledPassExecutionPlan::Graphics(value) => {",
            "CompiledPassExecutionPlan::Copy(_)",
            "context.realize_render_pipeline(",
            "PreparedFlowPipeline::Render",
        ),
    ];

    for (
        label,
        realization_start,
        realization_end,
        pipeline_realization_token,
        prepared_pipeline_kind,
    ) in paths
    {
        let realization_path = section(
            realization,
            realization_start,
            realization_end,
            PIPELINE_REALIZATION,
        );
        let admission = unique_position(
            realization_path,
            "admit_resolved_program_source(",
            label,
            "resolved source admission",
        );
        let binding_realization = unique_position(
            realization_path,
            ".resolve_compiled_bind_group(",
            label,
            "G4C2 pipeline-key and bind-group realization",
        );
        let pipeline_realization = unique_position(
            realization_path,
            pipeline_realization_token,
            label,
            "G4C3 pipeline realization",
        );
        assert!(
            admission < binding_realization && binding_realization < pipeline_realization,
            "{label} must admit source, realize G4C2 dependencies, then realize its G4C3 pipeline before G5"
        );
        assert_eq!(
            realization_path.matches("&admitted_source,").count(),
            1,
            "{label} must hand the retained admitted source record into descriptor construction exactly once"
        );
        for forbidden in [
            ".for_pipeline_creation(",
            ".create_compute_pipeline(",
            ".create_render_pipeline(",
            "ShaderSource::Wgsl(",
            "admitted_source.identity(),",
        ] {
            assert!(
                !realization_path.contains(forbidden),
                "{label} no-loan realization must not regain retired/raw pipeline authority through {forbidden:?}"
            );
        }
        assert!(
            realization_path.contains(prepared_pipeline_kind),
            "{label} realization must retain the opaque pipeline for canonical G5 work"
        );
    }

    for required in [
        "PreparedFlowPipeline::Compute(realized_pipeline)",
        "PreparedFlowPipeline::Render(realized_pipeline)",
        "GpuWorkOperation::Compute(operation)",
        "GpuWorkOperation::Render(GpuRenderOperation::new(",
    ] {
        assert!(
            operations.contains(required),
            "canonical G5 projection must consume realized pipeline authority through {required:?}"
        );
    }
    for forbidden in [
        "current_render_execution_bridge",
        "context.realize_compute_pipeline(",
        "context.realize_render_pipeline(",
        ".create_compute_pipeline(",
        ".create_render_pipeline(",
    ] {
        assert!(
            !operations.contains(forbidden),
            "canonical G5 projection must not re-enter G4/raw execution through {forbidden:?}"
        );
    }
}

#[test]
fn compute_specialization_is_typed_and_realized_before_g5_consumption() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let graph = read(&manifest_dir, PASS_GRAPH);
    let execution = read(&manifest_dir, PIPELINE_REALIZATION);
    let operations = read(&manifest_dir, LOGICAL_OPERATIONS);
    let realization = section(
        &execution,
        "pub(in crate::plugins::render::renderer::render_flow) fn realize_compiled_pass(",
        "fn resolve_color_target_format_from_plan(",
        PIPELINE_REALIZATION,
    );
    let compute = section(
        realization,
        "CompiledPassExecutionPlan::Compute(value) => {",
        "CompiledPassExecutionPlan::Fullscreen(value) => {",
        PIPELINE_REALIZATION,
    );

    assert!(
        graph.contains("pub value: GpuSpecializationValue"),
        "renderer compute constants must retain typed G4B specialization values"
    );
    assert!(
        !graph.contains("pub value: i64"),
        "renderer compute constants must not erase U32/I32 specialization type identity"
    );
    assert_eq!(
        compute
            .matches("compute_specialization_from_constants(")
            .count(),
        1,
        "compute realization must derive one normalized typed G4B specialization value set"
    );
    assert_eq!(
        compute.matches("specialization,").count(),
        1,
        "compute realization must pass typed specialization independently from source identity"
    );
    assert!(
        !compute.contains("shader_pipeline_identity_with_constants(")
            && !compute.contains("FlowPassPipelineVariant"),
        "compute specialization must not be recombined with source identity or renderer-local variant authority"
    );
    let specialization = unique_position(
        compute,
        "compute_specialization_from_constants(",
        "compute",
        "typed specialization lowering",
    );
    let binding_realization = unique_position(
        compute,
        ".resolve_compiled_bind_group(",
        "compute",
        "G4C2 binding realization",
    );
    let pipeline_realization = unique_position(
        compute,
        "context.realize_compute_pipeline(",
        "compute",
        "G4C3 compute-pipeline realization",
    );
    assert!(
        specialization < binding_realization && binding_realization < pipeline_realization,
        "typed specialization must enter the G4B/G4C2 descriptor before private G4C3 pipeline realization"
    );

    let project_compute = section(
        &operations,
        "pub(super) fn project_compute_operation(",
        "pub(super) fn project_render_operation(",
        LOGICAL_OPERATIONS,
    );
    assert!(
        project_compute.contains("pipeline.pipeline")
            && project_compute.contains("PreparedFlowPipeline::Compute")
            && project_compute.contains("GpuWorkOperation::Compute(operation)"),
        "G5 compute projection must consume the already-realized opaque compute pipeline"
    );
    for forbidden in [
        "compute_specialization_from_constants(",
        "context.realize_compute_pipeline(",
        ".for_pipeline_creation(",
        ".create_compute_pipeline(",
    ] {
        assert!(
            !project_compute.contains(forbidden),
            "G5 compute projection must not re-enter specialization or pipeline realization through {forbidden:?}"
        );
    }

    let specialization_helper = execution
        .split_once("fn compute_specialization_from_constants(")
        .map(|(_, tail)| tail)
        .expect("pipeline realization must retain typed specialization lowering");
    for required in [
        "GpuSpecializationKey::new(",
        "GpuSpecializationDeclaration::new(",
        "GpuSpecializationSchema::new(",
        "GpuSpecializationValueSet::new(",
    ] {
        assert!(
            specialization_helper.contains(required),
            "typed compute specialization lowering is missing {required:?}"
        );
    }
    for forbidden in [
        "FlowPassPipelineVariant",
        "ComputeSpecialization(signature)",
        "format!(\"{name}={value}\")",
        ".join(\",\")",
        "|constants:",
    ] {
        assert!(
            !specialization_helper.contains(forbidden),
            "string-encoded or renderer-local specialization authority returned: {forbidden}"
        );
    }
    for retired in [
        "CurrentRenderPipelineCreationTerminal",
        "wgpu_specialization_constants(",
        ".for_pipeline_creation(",
        ".create_compute_pipeline(",
    ] {
        assert!(
            !execution.contains(retired),
            "renderer pipeline consumer must not regain retired/private backend specialization authority through {retired:?}"
        );
    }
}

#[test]
fn renderer_program_source_admission_has_one_cache_gateway_without_renderer_lifetime_pins() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let execution = read(&manifest_dir, PIPELINE_REALIZATION);
    let cache = read(&manifest_dir, PIPELINE_CACHE);
    let authority = read(&manifest_dir, PROGRAM_SOURCES);

    let execution_gateway = section(
        &execution,
        "fn admit_resolved_program_source(",
        "fn reject_material_shader_fallback(",
        PIPELINE_REALIZATION,
    );
    assert!(
        execution_gateway.contains(") -> Result<GpuAdmittedProgramSource>"),
        "resolved renderer source admission must return the admitted record retained by live descriptors"
    );
    assert_eq!(
        execution_gateway
            .matches("cache.admit_program_source(")
            .count(),
        1,
        "resolved renderer source admission must delegate to the cache gateway exactly once"
    );
    assert!(
        !execution.contains("RendererProgramSourceAuthority::new("),
        "render execution must not create a parallel source authority"
    );
    assert!(
        !execution.contains(".admit_wgsl("),
        "render execution must not bypass the cache source-admission gateway"
    );

    assert_eq!(
        cache.matches("pub(crate) fn admit_program_source(").count(),
        1,
        "the renderer cache must expose exactly one source-admission gateway"
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
        "the renderer cache gateway must delegate exactly once to the one source authority"
    );
    assert_eq!(
        cache
            .matches("RendererProgramSourceAuthority::new(")
            .count(),
        1,
        "the renderer cache must construct exactly one source authority"
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
        "flow retirement must reclaim lookup-only source records after dropping cache keys"
    );
}

#[test]
fn builtin_program_sources_use_the_same_cache_gateway() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = read(&manifest_dir, PIPELINE_CACHE);
    let default_impl = section(
        &cache,
        "impl Default for FlowPipelineArtifactCache",
        "impl FlowPipelineArtifactCache",
        PIPELINE_CACHE,
    );
    assert_eq!(
        default_impl
            .matches("admit_builtin_program_source(")
            .count(),
        3,
        "renderer cache construction must admit compute, fullscreen, and graphics builtins"
    );

    let builtin_gateway = section(
        &cache,
        "fn admit_builtin_program_source(",
        "#[cfg(test)]",
        PIPELINE_CACHE,
    );
    assert_eq!(
        builtin_gateway.matches(".admit_program_source(").count(),
        1,
        "builtin sources must use the same cache gateway as resolved sources"
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

fn unique_position(section: &str, token: &str, path: &str, role: &str) -> usize {
    assert_eq!(
        section.matches(token).count(),
        1,
        "{path} must contain exactly one {role} token {token:?}"
    );
    section
        .find(token)
        .expect("counted token must remain findable")
}