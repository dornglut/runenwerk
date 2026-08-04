use std::fs;
use std::path::{Path, PathBuf};

const FLOW_KEYS: &str = "src/plugins/render/pipelines/flow_keys.rs";
const BINDINGS: &str = "src/plugins/render/renderer/render_flow/bindings.rs";

#[test]
fn renderer_pipeline_key_uses_typed_g4b_source_components_and_explicit_variant() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let flow_keys = read(&manifest_dir, FLOW_KEYS);

    assert!(
        flow_keys.contains("pub program_source_key: GpuProgramSourceKey"),
        "renderer pipeline keys must use the bounded G4B source key"
    );
    assert!(
        flow_keys.contains("pub program_source_revision: GpuProgramSourceRevision"),
        "renderer pipeline keys must use the nonzero G4B source revision"
    );
    assert!(
        flow_keys.contains("pub pipeline_variant: FlowPassPipelineVariant"),
        "renderer-local pipeline variation must remain separate from source identity"
    );
    for forbidden in ["pub shader_identity: String", "pub shader_revision: u64"] {
        assert!(
            !flow_keys.contains(forbidden),
            "legacy free-form source authority returned to FlowPassPipelineKey: {forbidden}"
        );
    }
    assert!(
        flow_keys.contains("ComputeSpecialization(String)"),
        "compute specialization must remain an explicit renderer-local pipeline variant"
    );
}

#[test]
fn binding_resolution_normalizes_the_remaining_combined_identity_boundary_once() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bindings = read(&manifest_dir, BINDINGS);

    assert_eq!(
        bindings
            .matches("split_shader_pipeline_identity(shader_identity)?")
            .count(),
        1,
        "the remaining combined renderer identity boundary must normalize exactly once"
    );
    assert_eq!(
        bindings
            .matches("const COMPUTE_SPECIALIZATION_SEPARATOR: &str = \"|constants:\";")
            .count(),
        1,
        "compute specialization splitting must have one canonical separator"
    );
    assert_eq!(
        bindings.matches("GpuProgramSourceKey::new(").count(),
        1,
        "binding resolution must construct one bounded source key"
    );
    assert_eq!(
        bindings
            .matches("GpuProgramSourceRevision::try_from_raw(")
            .count(),
        1,
        "binding resolution must construct one nonzero source revision"
    );
    assert!(
        bindings.contains("shader_revision.checked_add(1)"),
        "zero-based renderer revisions must normalize into the same one-based G4B domain as admission"
    );
}

fn read(manifest_dir: &Path, relative: &str) -> String {
    fs::read_to_string(manifest_dir.join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}
