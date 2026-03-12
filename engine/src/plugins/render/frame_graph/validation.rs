use super::{RegisteredPassDescriptor, RenderFeatureGraphSpec, RenderPassSpec, RenderPipelineSpec};
use anyhow::{Result, bail};

pub fn validate_feature_graph(spec: &RenderFeatureGraphSpec) -> Result<()> {
    spec.validate()
}

pub fn validate_pipeline_specs(pipelines: &[RenderPipelineSpec]) -> Result<()> {
    if pipelines
        .iter()
        .any(|pipeline| pipeline.id.as_str().trim().is_empty())
    {
        bail!("pipeline id must not be empty");
    }
    Ok(())
}

pub fn validate_pass_specs(passes: &[RenderPassSpec]) -> Result<()> {
    if passes.iter().any(|pass| pass.id.as_str().trim().is_empty()) {
        bail!("pass id must not be empty");
    }
    Ok(())
}

pub fn validate_registered_passes(passes: &[RegisteredPassDescriptor]) -> Result<()> {
    if passes.iter().any(|pass| pass.id.trim().is_empty()) {
        bail!("registered pass id must not be empty");
    }
    Ok(())
}
