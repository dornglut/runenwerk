use super::{
    render_pass_usage::validate_render_pass_usage_scope, GpuCapabilityFeature,
    GpuCapabilityRequirement, GpuCapabilityRequirementError, GpuCapabilityRequirements,
    GpuDepthStencilAccess, GpuQueryAccess, GpuQueryAccessKind, GpuReadbackOperation, GpuRenderDraw,
    GpuRenderPassSignature, GpuResourceAccess, GpuUploadOperation, GpuWorkOperationCause,
    GpuWorkOperationError,
};
use super::work::{
    GpuClearOperation, GpuColorAttachmentLoad, GpuComputeOperation, GpuCopyOperation,
    GpuDepthAttachmentLoad, GpuDrawIntent, GpuPresentOperation, GpuQueryResolveOperation,
    GpuRenderColorAttachment, GpuRenderDepthStencilAttachment,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuWorkNodeKind {
    Compute,
    Render,
    Copy,
    Clear,
    Resolve,
    Present,
    Upload,
    Readback,
}

/// One logical render pass with ordered execution-complete draws.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GpuRenderOperation {
    color_attachments: Vec<GpuRenderColorAttachment>,
    depth_stencil_attachment: Option<GpuRenderDepthStencilAttachment>,
    draws: Vec<GpuRenderDraw>,
    timestamp_writes: Vec<GpuQueryAccess>,
    signature: Option<GpuRenderPassSignature>,
    accesses: Vec<GpuResourceAccess>,
}

impl GpuRenderOperation {
    pub fn new(
        color_attachments: impl IntoIterator<Item = GpuRenderColorAttachment>,
        depth_stencil_attachment: Option<GpuRenderDepthStencilAttachment>,
        draws: impl IntoIterator<Item = GpuRenderDraw>,
        timestamp_writes: impl IntoIterator<Item = GpuQueryAccess>,
    ) -> Result<Self, GpuWorkOperationError> {
        let color_attachments = color_attachments.into_iter().collect::<Vec<_>>();
        let draws = draws.into_iter().collect::<Vec<_>>();
        let timestamp_writes = timestamp_writes.into_iter().collect::<Vec<_>>();
        validate_timestamp_writes(&timestamp_writes)?;

        let clears_color = color_attachments
            .iter()
            .any(|attachment| matches!(attachment.load(), GpuColorAttachmentLoad::Clear(_)));
        let clears_depth = depth_stencil_attachment.as_ref().is_some_and(|attachment| {
            matches!(attachment.load(), GpuDepthAttachmentLoad::Clear(_))
        });
        if draws.is_empty() && !clears_color && !clears_depth && timestamp_writes.is_empty() {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU render operation",
                "render",
                None,
                GpuWorkOperationCause::ZeroWork,
                "add a draw, attachment clear, or timestamp write",
            ));
        }

        let signature = if color_attachments.is_empty() && depth_stencil_attachment.is_none() {
            if !draws.is_empty() {
                return Err(GpuWorkOperationError::invalid(
                    "construct GPU render operation",
                    "attachmentless draw",
                    None,
                    GpuWorkOperationCause::InvalidAttachment,
                    "provide a color or depth/stencil attachment for render-pass draw execution",
                ));
            }
            None
        } else {
            Some(GpuRenderPassSignature::from_attachments(
                &color_attachments,
                depth_stencil_attachment.as_ref(),
            )?)
        };

        if let Some(signature) = &signature {
            for draw in &draws {
                signature.validate_draw(draw)?;
                validate_depth_access_for_draw(depth_stencil_attachment.as_ref(), draw)?;
            }
        }

        let mut accesses = Vec::new();
        for attachment in &color_attachments {
            accesses.push(GpuResourceAccess::Texture(
                attachment.source_access().clone(),
            ));
            if let Some(resolve) = attachment.resolve_target() {
                accesses.push(GpuResourceAccess::Texture(resolve.access().clone()));
            }
        }
        if let Some(attachment) = &depth_stencil_attachment {
            accesses.push(GpuResourceAccess::Texture(
                attachment.source_access().clone(),
            ));
        }
        for draw in &draws {
            accesses.extend(draw.accesses().iter().cloned());
        }
        accesses.extend(
            timestamp_writes
                .iter()
                .cloned()
                .map(GpuResourceAccess::Query),
        );

        validate_render_pass_usage_scope(&accesses)?;
        validate_timestamp_write_aliases(&timestamp_writes)?;

        Ok(Self {
            color_attachments,
            depth_stencil_attachment,
            draws,
            timestamp_writes,
            signature,
            accesses,
        })
    }

    pub fn color_attachments(&self) -> &[GpuRenderColorAttachment] {
        &self.color_attachments
    }

    pub fn depth_stencil_attachment(&self) -> Option<&GpuRenderDepthStencilAttachment> {
        self.depth_stencil_attachment.as_ref()
    }

    pub fn draws(&self) -> &[GpuRenderDraw] {
        &self.draws
    }

    pub fn timestamp_writes(&self) -> &[GpuQueryAccess] {
        &self.timestamp_writes
    }

    pub fn signature(&self) -> Option<&GpuRenderPassSignature> {
        self.signature.as_ref()
    }

    pub fn accesses(&self) -> &[GpuResourceAccess] {
        &self.accesses
    }
}

fn validate_timestamp_writes(accesses: &[GpuQueryAccess]) -> Result<(), GpuWorkOperationError> {
    if accesses
        .iter()
        .any(|access| access.kind() != GpuQueryAccessKind::WriteTimestamp)
    {
        return Err(GpuWorkOperationError::invalid(
            "construct GPU render operation",
            "timestamp writes",
            accesses.first().map(GpuQueryAccess::resource_identity),
            GpuWorkOperationCause::OperationAccessContradiction,
            "provide only WriteTimestamp query accesses as render-side timestamp writes",
        ));
    }
    Ok(())
}

fn validate_timestamp_write_aliases(accesses: &[GpuQueryAccess]) -> Result<(), GpuWorkOperationError> {
    for left_index in 0..accesses.len() {
        let left = &accesses[left_index];
        for right in &accesses[(left_index + 1)..] {
            if left.resource_identity() == right.resource_identity()
                && left.range().overlaps(right.range())
            {
                return Err(GpuWorkOperationError::invalid(
                    "construct GPU render operation",
                    "timestamp write overlap",
                    Some(left.resource_identity()),
                    GpuWorkOperationCause::OperationAccessContradiction,
                    "write each query slot at most once in one logical render pass",
                ));
            }
        }
    }
    Ok(())
}

fn validate_depth_access_for_draw(
    attachment: Option<&GpuRenderDepthStencilAttachment>,
    draw: &GpuRenderDraw,
) -> Result<(), GpuWorkOperationError> {
    let Some(attachment) = attachment else {
        return Ok(());
    };
    if attachment.access() == GpuDepthStencilAccess::ReadOnly
        && draw
            .pipeline()
            .state()
            .depth_stencil()
            .is_some_and(|depth| depth.depth_write_enabled())
    {
        return Err(GpuWorkOperationError::invalid(
            "validate GPU render draw depth access",
            "read-only depth attachment with depth-writing pipeline",
            Some(attachment.source().parent_texture().diagnostic_identity()),
            GpuWorkOperationCause::InvalidAttachment,
            "disable pipeline depth writes when the render pass uses a read-only depth attachment",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuWorkOperation {
    Compute(GpuComputeOperation),
    Render(GpuRenderOperation),
    Copy(GpuCopyOperation),
    Clear(GpuClearOperation),
    Resolve(GpuQueryResolveOperation),
    Present(GpuPresentOperation),
    Upload(GpuUploadOperation),
    Readback(GpuReadbackOperation),
}

impl GpuWorkOperation {
    pub const fn kind(&self) -> GpuWorkNodeKind {
        match self {
            Self::Compute(_) => GpuWorkNodeKind::Compute,
            Self::Render(_) => GpuWorkNodeKind::Render,
            Self::Copy(_) => GpuWorkNodeKind::Copy,
            Self::Clear(_) => GpuWorkNodeKind::Clear,
            Self::Resolve(_) => GpuWorkNodeKind::Resolve,
            Self::Present(_) => GpuWorkNodeKind::Present,
            Self::Upload(_) => GpuWorkNodeKind::Upload,
            Self::Readback(_) => GpuWorkNodeKind::Readback,
        }
    }

    pub fn derived_accesses(&self) -> Result<Vec<GpuResourceAccess>, GpuWorkOperationError> {
        match self {
            Self::Compute(operation) => {
                let mut accesses = operation.bindings().accesses().to_vec();
                if let Some(access) = operation.dispatch().indirect_access() {
                    accesses.push(GpuResourceAccess::Buffer(access.clone()));
                }
                accesses.extend(
                    operation
                        .timestamp_writes()
                        .iter()
                        .cloned()
                        .map(GpuResourceAccess::Query),
                );
                Ok(accesses)
            }
            Self::Render(operation) => Ok(operation.accesses().to_vec()),
            Self::Copy(operation) => operation.derived_accesses(),
            Self::Clear(operation) => operation.derived_accesses(),
            Self::Resolve(operation) => Ok(vec![
                GpuResourceAccess::Query(operation.source_access().clone()),
                GpuResourceAccess::Buffer(operation.destination_access().clone()),
            ]),
            Self::Present(operation) => Ok(vec![GpuResourceAccess::Texture(
                operation.source_access().clone(),
            )]),
            Self::Upload(operation) => Ok(vec![operation.destination_access().clone()]),
            Self::Readback(operation) => Ok(vec![operation.source_access().clone()]),
        }
    }

    pub(crate) fn validate_shape(&self) -> Result<(), GpuWorkOperationError> {
        match self {
            Self::Compute(_)
            | Self::Render(_)
            | Self::Resolve(_)
            | Self::Present(_)
            | Self::Upload(_)
            | Self::Readback(_) => self.derived_accesses().map(|_| ()),
            Self::Copy(operation) => match operation {
                GpuCopyOperation::BufferToBuffer {
                    source,
                    destination,
                } => GpuCopyOperation::buffer_to_buffer(source.clone(), destination.clone())
                    .map(|_| ()),
                GpuCopyOperation::BufferToTexture {
                    source,
                    destination,
                } => GpuCopyOperation::buffer_to_texture(source.clone(), destination.clone())
                    .map(|_| ()),
                GpuCopyOperation::TextureToBuffer {
                    source,
                    destination,
                } => GpuCopyOperation::texture_to_buffer(source.clone(), destination.clone())
                    .map(|_| ()),
                GpuCopyOperation::TextureToTexture {
                    source,
                    destination,
                } => GpuCopyOperation::texture_to_texture(source.clone(), destination.clone())
                    .map(|_| ()),
            },
            Self::Clear(GpuClearOperation::BufferZero(region)) => {
                GpuClearOperation::buffer_zero(region.clone()).map(|_| ())
            }
        }
    }

    pub fn derived_requirements(
        &self,
    ) -> Result<GpuCapabilityRequirements, GpuCapabilityRequirementError> {
        let mut requirements = GpuCapabilityRequirements::new();
        let primary = match self {
            Self::Compute(_) => GpuCapabilityFeature::Compute,
            Self::Render(_) => GpuCapabilityFeature::RenderPipeline,
            Self::Copy(_) | Self::Clear(_) | Self::Upload(_) | Self::Readback(_) => {
                GpuCapabilityFeature::Copy
            }
            Self::Resolve(_) => GpuCapabilityFeature::TimestampQuery,
            Self::Present(_) => GpuCapabilityFeature::Presentation,
        };
        requirements.insert(GpuCapabilityRequirement::Required(primary))?;

        match self {
            Self::Compute(operation) => {
                requirements = requirements.merge(operation.pipeline().requirements())?;
                if operation.dispatch().is_indirect() {
                    requirements.insert(GpuCapabilityRequirement::Required(
                        GpuCapabilityFeature::IndirectExecution,
                    ))?;
                }
                if !operation.timestamp_writes().is_empty() {
                    requirements.insert(GpuCapabilityRequirement::Required(
                        GpuCapabilityFeature::TimestampQuery,
                    ))?;
                }
            }
            Self::Render(operation) => {
                for draw in operation.draws() {
                    requirements = requirements.merge(draw.requirements())?;
                    if matches!(draw.draw(), GpuDrawIntent::Indirect { .. }) {
                        requirements.insert(GpuCapabilityRequirement::Required(
                            GpuCapabilityFeature::IndirectExecution,
                        ))?;
                    }
                }
                if operation.depth_stencil_attachment().is_some() {
                    requirements.insert(GpuCapabilityRequirement::Required(
                        GpuCapabilityFeature::DepthAttachment,
                    ))?;
                }
                if !operation.timestamp_writes().is_empty() {
                    requirements.insert(GpuCapabilityRequirement::Required(
                        GpuCapabilityFeature::TimestampQuery,
                    ))?;
                }
            }
            Self::Copy(_)
            | Self::Clear(_)
            | Self::Resolve(_)
            | Self::Present(_)
            | Self::Upload(_)
            | Self::Readback(_) => {}
        }

        for access in self.derived_accesses().map_err(|_| {
            GpuCapabilityRequirementError::Invalid {
                operation: "derive GPU operation capability requirements",
                label: format!("{:?}", self.kind()),
                cause: super::GpuCapabilityRequirementCause::ConflictingStrength,
                correction: "retain checked operation accesses before deriving capability requirements",
            }
        })? {
            requirements = requirements.merge(&access.derived_requirements()?)?;
        }
        Ok(requirements)
    }
}
