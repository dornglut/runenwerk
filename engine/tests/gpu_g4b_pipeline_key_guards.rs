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
    for forbidden in [
        "pub shader_identity: String",
        "pub shader_revision: u64",
        "pub program_source_key: GpuProgramSourceKey",
        "pub program_source_revision: GpuProgramSourceRevision",
    ] {
        assert!(
            !flow_keys.contains(forbidden),
            "duplicate source identity authority returned to FlowPassPipelineKey: {forbidden}"
        );
    }
}

#[test]
fn binding_resolution_obtains_owner_scoped_identity_from_the_cache_authority() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bindings = read(&manifest_dir, BINDINGS);

    assert_eq!(
        bindings.matches(".program_source_identity(").count(),
        1,
        "binding resolution must obtain one owner-scoped identity from the cache authority"
    );
    assert_eq!(
        bindings.matches("program_source_identity,").count(),
        1,
        "the owner-scoped identity must enter the pipeline key exactly once"
    );
    assert!(
        !bindings.contains("GpuProgramSourceRevision::try_from_raw("),
        "binding resolution must not own revision normalization"
    );
    assert!(
        !bindings.contains("shader_revision.checked_add(1)"),
        "binding resolution must not own zero-to-one-based revision normalization"
    );
}

#[test]
fn renderer_source_authority_owns_identity_normalization_once() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = read(&manifest_dir, PIPELINE_CACHE);
    let authority = read(&manifest_dir, PROGRAM_SOURCES);

    assert_eq!(
        cache
            .matches("pub(crate) fn program_source_identity(")
            .count(),
        1,
        "the renderer cache must expose one owner-scoped identity gateway"
    );
    let cache_gateway = section(
        &cache,
        "pub(crate) fn program_source_identity(",
        "pub(crate) fn admit_program_source(",
        PIPELINE_CACHE,
    );
    assert_eq!(
        cache_gateway
            .matches("self.program_sources.identity(")
            .count(),
        1,
        "the cache identity gateway must delegate to the one renderer source authority"
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
        "source admission must use the same identity-normalization operation as cache keys"
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
