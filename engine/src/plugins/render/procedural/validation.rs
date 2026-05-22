use super::descriptors::{ProceduralPassDescriptor, ProceduralVisualDescriptor};
use crate::plugins::render::{
    RenderDepthPolicy, RenderPrimitiveTopology, RenderVertexBufferLayout, RenderVertexStepMode,
};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProceduralValidationError {
    #[error("procedural pass label must not be empty")]
    EmptyLabel,
    #[error("procedural pass '{pass_label}' must declare a shader")]
    MissingShader { pass_label: String },
    #[error("procedural pass '{pass_label}' must declare a color target")]
    MissingColorTarget { pass_label: String },
    #[error("procedural pass '{pass_label}' has empty color target label")]
    EmptyColorTarget { pass_label: String },
    #[error("procedural pass '{pass_label}' must draw at least one instance")]
    EmptyInstanceDraw { pass_label: String },
    #[error("procedural pass '{pass_label}' must draw at least one vertex")]
    EmptyVertexDraw { pass_label: String },
    #[error(
        "procedural pass '{pass_label}' {role} layout slot {slot} must use {expected} step mode"
    )]
    InvalidStepMode {
        pass_label: String,
        role: &'static str,
        slot: u32,
        expected: &'static str,
    },
    #[error("procedural pass '{pass_label}' {role} layout slot {slot} must have a non-zero stride")]
    EmptyStride {
        pass_label: String,
        role: &'static str,
        slot: u32,
    },
    #[error("procedural pass '{pass_label}' {role} layout slot {slot} must declare attributes")]
    MissingAttributes {
        pass_label: String,
        role: &'static str,
        slot: u32,
    },
    #[error(
        "procedural pass '{pass_label}' {role} layout slot {slot} has an attribute outside the stride"
    )]
    AttributeOutsideStride {
        pass_label: String,
        role: &'static str,
        slot: u32,
    },
    #[error(
        "procedural pass '{pass_label}' depth policy {policy} requires an explicit depth target"
    )]
    MissingDepthTargetForPolicy {
        pass_label: String,
        policy: &'static str,
    },
    #[error(
        "procedural pass '{pass_label}' declares a depth target while depth policy is disabled"
    )]
    DisabledDepthWithTarget { pass_label: String },
    #[error(
        "procedural pass '{pass_label}' generated sprite/impostor visuals require triangle-list topology"
    )]
    UnsupportedGeneratedSpriteTopology { pass_label: String },
}

pub fn validate_procedural_pass(
    descriptor: &ProceduralPassDescriptor,
) -> Result<(), ProceduralValidationError> {
    let pass_label = descriptor.label.trim();
    if pass_label.is_empty() {
        return Err(ProceduralValidationError::EmptyLabel);
    }
    let pass_label = pass_label.to_string();

    if descriptor.shader.is_none() {
        return Err(ProceduralValidationError::MissingShader { pass_label });
    }
    let Some(target) = descriptor.target.as_ref() else {
        return Err(ProceduralValidationError::MissingColorTarget { pass_label });
    };
    if target.color_target.trim().is_empty() {
        return Err(ProceduralValidationError::EmptyColorTarget { pass_label });
    }
    if descriptor.instance_count == 0 {
        return Err(ProceduralValidationError::EmptyInstanceDraw { pass_label });
    }
    if descriptor.visual.vertex_count() == 0 {
        return Err(ProceduralValidationError::EmptyVertexDraw { pass_label });
    }

    validate_layout(
        &pass_label,
        "instance",
        &descriptor.instance_buffer.layout,
        RenderVertexStepMode::Instance,
    )?;
    if let ProceduralVisualDescriptor::MeshSprite { vertex_buffer, .. } = &descriptor.visual {
        validate_layout(
            &pass_label,
            "vertex",
            &vertex_buffer.layout,
            RenderVertexStepMode::Vertex,
        )?;
    }

    match descriptor.policy.depth_policy {
        RenderDepthPolicy::ReadOnly | RenderDepthPolicy::ReadWrite
            if target.depth_target.is_none() =>
        {
            return Err(ProceduralValidationError::MissingDepthTargetForPolicy {
                pass_label,
                policy: depth_policy_name(descriptor.policy.depth_policy),
            });
        }
        RenderDepthPolicy::Disabled if target.depth_target.is_some() => {
            return Err(ProceduralValidationError::DisabledDepthWithTarget { pass_label });
        }
        RenderDepthPolicy::Default
        | RenderDepthPolicy::ReadOnly
        | RenderDepthPolicy::ReadWrite
        | RenderDepthPolicy::Disabled => {}
    }

    if matches!(
        descriptor.visual,
        ProceduralVisualDescriptor::QuadSprite { .. }
            | ProceduralVisualDescriptor::LocalSdf2dImpostor { .. }
    ) && descriptor.policy.primitive_topology != RenderPrimitiveTopology::TriangleList
    {
        return Err(ProceduralValidationError::UnsupportedGeneratedSpriteTopology { pass_label });
    }

    Ok(())
}

fn validate_layout(
    pass_label: &str,
    role: &'static str,
    layout: &RenderVertexBufferLayout,
    expected_step: RenderVertexStepMode,
) -> Result<(), ProceduralValidationError> {
    if layout.step_mode != expected_step {
        return Err(ProceduralValidationError::InvalidStepMode {
            pass_label: pass_label.to_string(),
            role,
            slot: layout.slot,
            expected: vertex_step_mode_name(expected_step),
        });
    }
    if layout.array_stride == 0 {
        return Err(ProceduralValidationError::EmptyStride {
            pass_label: pass_label.to_string(),
            role,
            slot: layout.slot,
        });
    }
    if layout.attributes.is_empty() {
        return Err(ProceduralValidationError::MissingAttributes {
            pass_label: pass_label.to_string(),
            role,
            slot: layout.slot,
        });
    }
    if layout.attributes.iter().any(|attribute| {
        attribute
            .offset
            .saturating_add(attribute.format.size_bytes())
            > layout.array_stride
    }) {
        return Err(ProceduralValidationError::AttributeOutsideStride {
            pass_label: pass_label.to_string(),
            role,
            slot: layout.slot,
        });
    }
    Ok(())
}

fn vertex_step_mode_name(value: RenderVertexStepMode) -> &'static str {
    match value {
        RenderVertexStepMode::Vertex => "vertex",
        RenderVertexStepMode::Instance => "instance",
    }
}

fn depth_policy_name(value: RenderDepthPolicy) -> &'static str {
    match value {
        RenderDepthPolicy::Default => "default",
        RenderDepthPolicy::Disabled => "disabled",
        RenderDepthPolicy::ReadOnly => "read_only",
        RenderDepthPolicy::ReadWrite => "read_write",
    }
}
