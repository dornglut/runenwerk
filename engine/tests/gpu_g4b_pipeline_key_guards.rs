use std::fs;
use std::path::{Path, PathBuf};

const FLOW_KEYS: &str = "src/plugins/render/pipelines/flow_keys.rs";
const BINDINGS: &str = "src/plugins/render/renderer/render_flow/bindings.rs";
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
        flow_keys.contains("pub primary_bind_group_layout: GpuBindGroupLayoutDescriptor"),
        "renderer pipeline keys must retain the complete typed primary bind-group layout"
    );
    for forbidden in [
        "pub shader_identity: String",
        "pub shader_revision: u64",
        "pub program_source_key: GpuProgramSourceKey",
        "pub program_source_revision: GpuProgramSourceRevision",
        "bind_group_layout_signature_hash",
    ] {
        assert!(
            !flow_keys.contains(forbidden),
            "duplicate source identity authority returned to FlowPassPipelineKey: {forbidden}"
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
fn primary_bind_group_layout_is_typed_before_wgpu_realization() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bindings = read(&manifest_dir, BINDINGS);

    assert!(
        bindings.contains("kind: GpuBindingKind"),
        "resolved bindings must retain typed G4B binding kinds"
    );
    assert_eq!(
        bindings
            .matches("GpuBindGroupLayoutDescriptor::new(0, binding_declarations)?")
            .count(),
        1,
        "binding resolution must construct one complete typed primary bind-group layout"
    );
    assert_eq!(
        bindings.matches("primary_bind_group_layout,").count(),
        1,
        "the typed primary layout must enter the pipeline key exactly once"
    );
    assert!(
        bindings.contains(".map(wgpu_bind_group_layout_entry)"),
        "WGPU layout realization must consume the typed G4B declarations"
    );
    for forbidden in [
        "layout_ty: BindingType",
        "hash_bind_group_layout_entries(",
        "bind_group_layout_signature_hash:",
    ] {
        assert!(
            !bindings.contains(forbidden),
            "superseded raw or hash-only primary-layout authority returned: {forbidden}"
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
