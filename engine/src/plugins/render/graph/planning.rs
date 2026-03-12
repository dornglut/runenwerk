use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::frame_graph::{
    OwnerRenderGraphRegistration, RegisteredPassDescriptor, RegisteredPassKind,
    RegisteredPipelineDescriptor, RegisteredPipelineRef,
};
use crate::plugins::render::pipelines::PipelineKey;
use crate::plugins::render::{RenderFlowValidationError, RenderPassKind};

pub fn compile_flow_to_owner_registration(
    flow: &RenderFlow,
    owner: impl Into<String>,
) -> Result<OwnerRenderGraphRegistration, RenderFlowValidationError> {
    flow.validate()?;

    let mut registration = OwnerRenderGraphRegistration::new(owner);
    let mut pipelines = Vec::<RegisteredPipelineDescriptor>::new();
    let mut passes = Vec::<RegisteredPassDescriptor>::new();

    for pass in &flow.graph().passes.passes {
        let pipeline = pass.shader.as_ref().map(|shader| {
            let pipeline_id = format!("{}.pipeline", pass.id.as_str());
            pipelines.push(RegisteredPipelineDescriptor::new(
                pipeline_id.clone(),
                PipelineKey::from(shader.clone()),
            ));
            RegisteredPipelineRef::Named(pipeline_id)
        });

        let kind = match pass.kind {
            RenderPassKind::Compute => RegisteredPassKind::Compute,
            _ => RegisteredPassKind::Render,
        };

        let executor = match pass.kind {
            RenderPassKind::BuiltinUiComposite => "builtin_ui_composite".to_string(),
            _ => pass.id.as_str().to_string(),
        };

        passes.push(RegisteredPassDescriptor {
            id: pass.id.as_str().to_string(),
            kind,
            reads: pass
                .reads
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            writes: pass
                .writes
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            depends_on: pass
                .depends_on
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            pipeline,
            executor: Some(executor),
        });
    }

    registration.pipelines = pipelines;
    registration.passes = passes;
    Ok(registration)
}
