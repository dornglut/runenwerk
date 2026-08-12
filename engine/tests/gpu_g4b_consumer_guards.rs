use std::fs;
use std::path::{Path, PathBuf};

const PASS_GRAPH: &str = "src/plugins/render/graph/pass_graph.rs";
const PASS_BINDINGS: &str = "src/plugins/render/api/bindings.rs";
const EXECUTION_PLAN: &str = "src/plugins/render/graph/execution_plan.rs";
const RUNTIME_BINDINGS: &str = "src/plugins/render/renderer/render_flow/bindings.rs";
const PRIMITIVE_PLAN: &str = "src/plugins/render/gpu_primitives/plan.rs";
const EXECUTE_PASSES: &str = "src/plugins/render/renderer/render_flow/execute_passes.rs";
const PIPELINE_CACHE: &str = "src/plugins/render/renderer/pipeline_cache.rs";
const PROGRAM_SOURCES: &str = "src/plugins/render/renderer/render_flow/program_sources.rs";
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
        "fn gpu_pipeline_layout_for_pass(",
        RUNTIME_BINDINGS,
    );
    assert!(
        !runtime_bindings.contains(".enumerate()"),
        "runtime G4B declarations and WGPU bind-group entries must not rebuild binding identity from vector position"
    );
    assert!(
        runtime_bindings.contains("value.key,") || runtime_bindings.contains("value.key"),
        "runtime G4B declarations must use the retained typed key"
    );
    assert!(
        runtime_bindings.contains("runtime_binding_value(value, sampler.as_ref())"),
        "runtime G4B declarations must pass their retained typed keys into G4C2 binding realization"
    );
    assert!(
        runtime.contains("GpuRuntimeBindingValue::new(value.key, [resource])"),
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
fn resolved_renderer_programs_admit_before_g4c2_realization_and_g4c3_pipeline_creation() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = read(&manifest_dir, EXECUTE_PASSES);
    let bindings = read(&manifest_dir, RUNTIME_BINDINGS);
    for required in [
        "context.realize_program(",
        "context.realize_bind_group_layout(",
        "context.realize_pipeline_layout(",
        "context.realize_bind_group(",
    ] {
        assert!(
            bindings.contains(required),
            "renderer binding resolution must delegate {required:?} to the G4C2 realization authority"
        );
    }
    let realization = section(
        &source,
        "pub(super) fn realize_compiled_pass(",
        "pub(super) fn encode_compiled_pass(",
        EXECUTE_PASSES,
    );
    let paths = [
        (
            "compute",
            "CompiledPassExecutionPlan::Compute(value) => {",
            "CompiledPassExecutionPlan::Fullscreen(value) => {",
            "fn encode_compute_pass(",
            "fn encode_fullscreen_pass(",
        ),
        (
            "fullscreen",
            "CompiledPassExecutionPlan::Fullscreen(value) => {",
            "CompiledPassExecutionPlan::Graphics(value) => {",
            "fn encode_fullscreen_pass(",
            "fn encode_graphics_pass(",
        ),
        (
            "graphics",
            "CompiledPassExecutionPlan::Graphics(value) => {",
            "CompiledPassExecutionPlan::Copy(_)",
            "fn encode_graphics_pass(",
            "fn encode_copy_pass(",
        ),
    ];

    for (label, realization_start, realization_end, raw_start, raw_end) in paths {
        let realization_path = section(
            realization,
            realization_start,
            realization_end,
            EXECUTE_PASSES,
        );
        let admission = unique_position(
            realization_path,
            "admit_resolved_program_source(",
            label,
            "resolved source admission",
        );
        let key_resolution = unique_position(
            realization_path,
            ".resolve_compiled_bind_group(",
            label,
            "G4C2 pipeline-key and bind-group realization",
        );
        assert!(
            admission < key_resolution,
            "{label} constructs renderer cache identity before admitting its exact resolved canonical WGSL source"
        );
        assert!(
            !realization_path.contains(".for_pipeline_creation("),
            "{label} must finish G4C2 realization before G4C3 pipeline creation begins"
        );
        assert_eq!(
            realization_path.matches("&admitted_source,").count(),
            1,
            "{label} must hand the retained admitted source record into complete descriptor construction exactly once"
        );
        assert!(
            !realization_path.contains("admitted_source.identity(),"),
            "{label} must not collapse the retained admitted source record back to identity-only key authority"
        );
        assert!(
            !realization_path.contains("ShaderSource::Wgsl("),
            "{label} must not retain a parallel raw WGPU WGSL realization path after G4C2"
        );

        let raw_path = section(&source, raw_start, raw_end, EXECUTE_PASSES);
        unique_position(
            raw_path,
            ".for_pipeline_creation(",
            label,
            "temporary G4C3 pipeline creation",
        );
        for forbidden in [
            "admit_resolved_program_source(",
            ".resolve_compiled_bind_group(",
            "context.realize_",
        ] {
            assert!(
                !raw_path.contains(forbidden),
                "{label} raw G4C3/G5 phase must not re-enter G4C2 through {forbidden:?}"
            );
        }
        for required in [
            "prepared.bindings.program",
            "prepared.bindings.pipeline_layout",
        ] {
            assert!(
                raw_path.contains(required),
                "{label} G4C3 creation must consume the completed G4C2 artifact {required:?}"
            );
        }
    }
}

#[test]
fn compute_specialization_is_typed_and_separate_from_source_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let graph = read(&manifest_dir, PASS_GRAPH);
    let execution = read(&manifest_dir, EXECUTE_PASSES);
    let realization = section(
        &execution,
        "pub(super) fn realize_compiled_pass(",
        "pub(super) fn encode_compiled_pass(",
        EXECUTE_PASSES,
    );
    let compute = section(
        realization,
        "CompiledPassExecutionPlan::Compute(value) => {",
        "CompiledPassExecutionPlan::Fullscreen(value) => {",
        EXECUTE_PASSES,
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
        "compute execution must derive one normalized typed G4B specialization value set"
    );
    assert_eq!(
        compute.matches("specialization,").count(),
        1,
        "compute execution must pass typed specialization independently from the admitted source record"
    );
    assert!(
        !compute.contains("shader_pipeline_identity_with_constants("),
        "compute specialization must not be recombined with source identity"
    );
    assert!(
        !compute.contains("FlowPassPipelineVariant"),
        "renderer-local specialization wrapper authority must remain retired"
    );
    let key_resolution = unique_position(
        compute,
        ".resolve_compiled_bind_group(",
        "compute",
        "G4C2 pipeline-key resolution",
    );
    let specialization = unique_position(
        compute,
        "compute_specialization_from_constants(",
        "compute",
        "typed specialization lowering",
    );
    assert!(
        specialization < key_resolution,
        "the typed G4B specialization descriptor must feed G4C2 realization before descriptor publication"
    );
    assert!(
        !compute.contains(".for_pipeline_creation("),
        "typed G4B specialization must complete in the no-loan G4C2 phase"
    );

    let raw_compute = section(
        &execution,
        "fn encode_compute_pass(",
        "fn encode_fullscreen_pass(",
        EXECUTE_PASSES,
    );
    unique_position(
        raw_compute,
        ".for_pipeline_creation(",
        "compute",
        "temporary G4C3 pipeline creation",
    );
    assert!(
        !raw_compute.contains("compute_specialization_from_constants("),
        "raw G4C3 compute creation must consume, not recompute, the G4B specialization descriptor"
    );
    assert!(
        raw_compute.contains("prepared.bindings.pipeline_key"),
        "raw G4C3 compute creation must consume the completed G4C2 pipeline descriptor"
    );

    let specialization_helper = section(
        &execution,
        "fn compute_specialization_from_constants(",
        "fn wgpu_specialization_constants(",
        EXECUTE_PASSES,
    );
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

    let backend_helper = section(
        &execution,
        "fn wgpu_specialization_constants(",
        "fn render_vertex_format_to_wgpu(",
        EXECUTE_PASSES,
    );
    assert!(
        backend_helper.contains("values: &GpuSpecializationValueSet"),
        "backend specialization lowering must accept the typed G4B value set directly"
    );
    assert!(
        backend_helper.contains(".entries()"),
        "backend specialization lowering must project constants from typed G4B entries"
    );
    assert!(
        !backend_helper.contains("RenderShaderConstant"),
        "WGPU specialization lowering must not bypass typed G4B specialization authority"
    );
    let compute_terminal = section(
        &execution,
        "impl CurrentRenderPipelineCreationTerminal for CreateFlowComputePipeline",
        "/// G4C3's temporary fullscreen render-pipeline creation terminal.",
        EXECUTE_PASSES,
    );
    assert!(
        compute_terminal
            .contains("wgpu_specialization_constants(self.descriptor.specialization())"),
        "the temporary G4C3 compute terminal must project specialization from the completed typed descriptor"
    );
}

#[test]
fn renderer_program_source_admission_has_one_cache_gateway_without_renderer_lifetime_pins() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let execution = read(&manifest_dir, EXECUTE_PASSES);
    let cache = read(&manifest_dir, PIPELINE_CACHE);
    let authority = read(&manifest_dir, PROGRAM_SOURCES);

    let execution_gateway = section(
        &execution,
        "fn admit_resolved_program_source(",
        "fn reject_material_shader_fallback(",
        EXECUTE_PASSES,
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
        "pub fn compute_pipeline(",
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
