use std::fs;
use std::path::{Path, PathBuf};

const FLOW_KEYS: &str = "src/plugins/render/pipelines/flow_keys.rs";
const BINDINGS: &str = "src/plugins/render/renderer/render_flow/bindings.rs";
const EXECUTION_PLAN: &str = "src/plugins/render/graph/execution_plan.rs";
const EXECUTE_PASSES: &str = "src/plugins/render/renderer/render_flow/execute_passes.rs";
const EXECUTE: &str = "src/plugins/render/renderer/render_flow/execute.rs";
const RENDER_FLOW_MOD: &str = "src/plugins/render/renderer/render_flow/mod.rs";
const PIPELINE_CACHE: &str = "src/plugins/render/renderer/pipeline_cache.rs";
const PROGRAM_SOURCES: &str = "src/plugins/render/renderer/render_flow/program_sources.rs";

#[test]
fn renderer_pipeline_key_uses_one_owner_scoped_g4b_source_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);

    assert!(
        flow_keys.contains("pub program_source_identity: GpuProgramSourceIdentity"),
        "renderer pipeline keys must retain one owner-scoped G4B source identity"
    );
    assert!(
        flow_keys.contains("pub pipeline_variant: FlowPassPipelineVariant"),
        "renderer-local pipeline variation must remain separate from source identity"
    );
    assert!(
        flow_keys.contains("ComputeSpecialization(GpuSpecializationValueSet)"),
        "compute pipeline variation must retain complete typed G4B specialization values"
    );
    assert!(
        flow_keys.contains("pub fn specialization(&self) -> Option<&GpuSpecializationValueSet>"),
        "backend lowering must have one typed specialization accessor"
    );
    assert!(
        flow_keys.contains("pub pipeline_layout: GpuPipelineLayoutDescriptor"),
        "renderer pipeline keys must retain one complete typed G4B pipeline layout"
    );
    assert!(
        flow_keys.contains("pub render_pipeline_state: Option<GpuRenderPipelineStateDescriptor>"),
        "renderer pipeline keys must retain one complete typed G4B render-pipeline state"
    );
    assert_eq!(
        flow_keys
            .matches("pub fn pipeline_layout_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "pipeline-layout diagnostics must derive through one typed-layout accessor"
    );
    assert_eq!(
        flow_keys
            .matches("pub fn primary_bind_group_layout_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "legacy group-0 provenance diagnostics must derive through the typed pipeline layout"
    );
    assert_eq!(
        flow_keys
            .matches("pub fn render_pipeline_state_diagnostic_hash(&self) -> u64")
            .count(),
        1,
        "render-state diagnostics must remain derived through one typed aggregate-state accessor"
    );
    for forbidden in [
        "pub shader_identity: String",
        "pub shader_revision: u64",
        "pub program_source_key: GpuProgramSourceKey",
        "pub program_source_revision: GpuProgramSourceRevision",
        "ComputeSpecialization(String)",
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
        "use wgpu::TextureFormat",
    ] {
        assert!(
            !flow_keys.contains(forbidden),
            "duplicate or untyped pipeline correctness authority returned to FlowPassPipelineKey: {forbidden}"
        );
    }
}

#[test]
fn binding_resolution_consumes_admitted_source_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bindings = read(&manifest_dir, BINDINGS);

    assert_eq!(
        bindings
            .matches("program_source_identity: &GpuProgramSourceIdentity")
            .count(),
        1,
        "binding resolution must accept one admitted owner-scoped source identity"
    );
    assert_eq!(
        bindings
            .matches("program_source_identity: program_source_identity.clone(),")
            .count(),
        1,
        "the admitted owner-scoped identity must enter the pipeline key exactly once"
    );
    assert_eq!(
        bindings
            .matches("pipeline_variant: FlowPassPipelineVariant")
            .count(),
        1,
        "renderer-local pipeline variation must be passed independently"
    );
    for forbidden in [
        ".program_source_identity(",
        "GpuProgramSourceKey::new(",
        "shader_identity: &str",
        "shader_revision: u64",
        "split_shader_pipeline_identity(",
        "COMPUTE_SPECIALIZATION_SEPARATOR",
    ] {
        assert!(
            !bindings.contains(forbidden),
            "pre-admission or combined source identity authority returned to binding resolution: {forbidden}"
        );
    }
}

#[test]
fn complete_pipeline_layout_is_typed_before_pipeline_key_publication() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);
    let bindings = read(&manifest_dir, BINDINGS);
    let execute = read(&manifest_dir, EXECUTE);
    let render_flow_mod = read(&manifest_dir, RENDER_FLOW_MOD);

    assert!(
        bindings.contains("kind: GpuBindingKind"),
        "resolved primary bindings must retain typed G4B binding kinds"
    );
    assert_eq!(
        bindings
            .matches("GpuBindGroupLayoutDescriptor::new(0, binding_declarations)?")
            .count(),
        1,
        "binding resolution must construct one complete typed group-0 layout"
    );
    assert_eq!(
        bindings
            .matches("GpuBindGroupLayoutDescriptor::new(1, declarations)?")
            .count(),
        1,
        "material resource declarations must normalize to one typed group-1 layout"
    );
    assert_eq!(
        bindings
            .matches("GpuPipelineLayoutDescriptor::new(groups)?")
            .count(),
        1,
        "binding resolution must construct one complete logical typed pipeline layout"
    );
    assert_eq!(
        bindings.matches("pipeline_layout,").count(),
        1,
        "the complete typed pipeline layout must enter the pipeline key exactly once"
    );
    for required in [
        "GpuBindingKey::try_new(1, texture_binding_identity)?",
        "GpuBindingKind::sampled_texture(",
        "GpuTextureSampleClass::FloatFilterable",
        "GpuBindingKind::sampler(GpuSamplerClass::Filtering)",
        "GpuTextureViewDimension::D2",
        "GpuTextureViewDimension::D3",
    ] {
        assert!(
            bindings.contains(required),
            "material group-1 layout is not normalized through the expected typed G4B vocabulary: {required}"
        );
    }
    assert!(
        bindings.contains(".map(wgpu_bind_group_layout_entry)"),
        "current group-0 WGPU layout realization must consume typed G4B declarations"
    );
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
        flow_keys.contains("diagnostic_hash(&self.pipeline_layout.group(0))"),
        "legacy group-0 provenance diagnostics must be projections of the typed pipeline layout"
    );
    assert_eq!(
        execute
            .matches("FlowPassPipelineKey::primary_bind_group_layout_diagnostic_hash")
            .count(),
        1,
        "execution provenance must derive its group-0 diagnostic hash from the typed pipeline layout"
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
fn render_pipeline_state_is_typed_before_pipeline_key_publication() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);
    let bindings = read(&manifest_dir, BINDINGS);
    let execution_plan = read(&manifest_dir, EXECUTION_PLAN);
    let execute_passes = read(&manifest_dir, EXECUTE_PASSES);
    let execute = read(&manifest_dir, EXECUTE);
    let render_flow_mod = read(&manifest_dir, RENDER_FLOW_MOD);

    assert_eq!(
        bindings
            .matches(
                "gpu_render_pipeline_state_for_pass(flow, pass_id, &color_formats, depth_format)?"
            )
            .count(),
        1,
        "binding resolution must normalize one aggregate render-pipeline state before publishing the key"
    );
    assert_eq!(
        bindings.matches("render_pipeline_state,").count(),
        1,
        "the aggregate render-pipeline state must enter the pipeline key exactly once"
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
        (EXECUTE_PASSES, execute_passes.as_str()),
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

    assert!(
        execute.contains("key.render_pipeline_state.as_ref()"),
        "execution provenance must derive render-state diagnostics from the typed aggregate"
    );
    for forbidden in [
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
        (EXECUTE_PASSES, execute_passes.as_str()),
    ] {
        assert!(
            !source.contains("vertex_layout_signature_hash"),
            "superseded naked vertex-layout hash authority returned in {path}"
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
        "pub fn get_or_create_shader_module<",
        PIPELINE_CACHE,
    );
    assert_eq!(
        cache_gateway.matches(".admit_and_retain_wgsl(").count(),
        1,
        "the cache admission gateway must delegate to the one retaining source authority"
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
        "pub(crate) fn admit_and_retain_wgsl(",
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
