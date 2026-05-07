use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderDynamicTextureTargetKey {
    pub namespace: String,
    pub target_id: String,
}

impl RenderDynamicTextureTargetKey {
    pub fn new(namespace: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            target_id: target_id.into(),
        }
    }

    pub fn label(&self) -> String {
        format!("{}:{}", self.namespace, self.target_id)
    }
}

impl fmt::Display for RenderDynamicTextureTargetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.target_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderTextureTargetFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    R32Uint,
    Depth32Float,
}

impl RenderTextureTargetFormat {
    pub const fn is_depth(self) -> bool {
        matches!(self, Self::Depth32Float)
    }

    pub const fn is_uint(self) -> bool {
        matches!(self, Self::R32Uint)
    }

    pub const fn is_displayable(self) -> bool {
        matches!(self, Self::Rgba8Unorm | Self::Rgba8UnormSrgb)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderTextureSampleMode {
    FilterableFloat,
    NonFilterableFloat,
    Uint,
    Depth,
    NotSampled,
}

impl RenderTextureSampleMode {
    pub const fn is_sampled(self) -> bool {
        !matches!(self, Self::NotSampled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderTextureTargetUsage {
    pub color_attachment: bool,
    pub depth_attachment: bool,
    pub sampled: bool,
    pub storage: bool,
    pub copy_src: bool,
    pub copy_dst: bool,
}

impl RenderTextureTargetUsage {
    pub const fn color_sampled() -> Self {
        Self {
            color_attachment: true,
            depth_attachment: false,
            sampled: true,
            storage: false,
            copy_src: true,
            copy_dst: true,
        }
    }

    pub const fn color_attachment_only() -> Self {
        Self {
            color_attachment: true,
            depth_attachment: false,
            sampled: false,
            storage: false,
            copy_src: true,
            copy_dst: true,
        }
    }

    pub const fn sampled_only() -> Self {
        Self {
            color_attachment: false,
            depth_attachment: false,
            sampled: true,
            storage: false,
            copy_src: true,
            copy_dst: false,
        }
    }

    pub const fn storage_sampled() -> Self {
        Self {
            color_attachment: false,
            depth_attachment: false,
            sampled: true,
            storage: true,
            copy_src: true,
            copy_dst: true,
        }
    }

    pub const fn depth_sampled() -> Self {
        Self {
            color_attachment: false,
            depth_attachment: true,
            sampled: true,
            storage: false,
            copy_src: true,
            copy_dst: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderDynamicTextureRetention {
    RetainWhileRequested,
    RetainUntilViewportClose,
    RetainForFrames(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderDynamicTextureTargetDescriptor {
    pub key: RenderDynamicTextureTargetKey,
    pub width: u32,
    pub height: u32,
    pub format: RenderTextureTargetFormat,
    pub usage: RenderTextureTargetUsage,
    pub sample_mode: RenderTextureSampleMode,
    pub retention: RenderDynamicTextureRetention,
}

impl RenderDynamicTextureTargetDescriptor {
    pub fn new(
        key: RenderDynamicTextureTargetKey,
        width: u32,
        height: u32,
        format: RenderTextureTargetFormat,
        usage: RenderTextureTargetUsage,
        sample_mode: RenderTextureSampleMode,
        retention: RenderDynamicTextureRetention,
    ) -> Self {
        Self {
            key,
            width,
            height,
            format,
            usage,
            sample_mode,
            retention,
        }
    }

    pub fn signature(&self) -> RenderDynamicTextureTargetSignature {
        RenderDynamicTextureTargetSignature {
            width: self.width,
            height: self.height,
            format: self.format,
            usage: self.usage,
            sample_mode: self.sample_mode,
            retention: self.retention,
        }
    }

    pub fn validate(&self) -> Result<(), RenderDynamicTextureTargetDescriptorError> {
        if self.key.namespace.trim().is_empty() || self.key.target_id.trim().is_empty() {
            return Err(RenderDynamicTextureTargetDescriptorError::EmptyKey);
        }
        if self.width == 0 || self.height == 0 {
            return Err(RenderDynamicTextureTargetDescriptorError::ZeroDimensions {
                width: self.width,
                height: self.height,
            });
        }
        if self.format.is_depth() {
            if self.usage.color_attachment || self.usage.storage {
                return Err(RenderDynamicTextureTargetDescriptorError::InvalidDepthUsage);
            }
            if self.usage.depth_attachment && self.sample_mode != RenderTextureSampleMode::Depth {
                return Err(RenderDynamicTextureTargetDescriptorError::InvalidSampleModeForFormat);
            }
        } else if self.usage.depth_attachment {
            return Err(RenderDynamicTextureTargetDescriptorError::InvalidColorUsage);
        }
        if self.format.is_uint()
            && self.sample_mode.is_sampled()
            && self.sample_mode != RenderTextureSampleMode::Uint
        {
            return Err(RenderDynamicTextureTargetDescriptorError::InvalidSampleModeForFormat);
        }
        if self.format.is_displayable()
            && self.sample_mode.is_sampled()
            && !matches!(
                self.sample_mode,
                RenderTextureSampleMode::FilterableFloat
                    | RenderTextureSampleMode::NonFilterableFloat
            )
        {
            return Err(RenderDynamicTextureTargetDescriptorError::InvalidSampleModeForFormat);
        }
        if self.sample_mode.is_sampled() && !self.usage.sampled {
            return Err(RenderDynamicTextureTargetDescriptorError::SampleModeRequiresSampledUsage);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderDynamicTextureTargetSignature {
    pub width: u32,
    pub height: u32,
    pub format: RenderTextureTargetFormat,
    pub usage: RenderTextureTargetUsage,
    pub sample_mode: RenderTextureSampleMode,
    pub retention: RenderDynamicTextureRetention,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RenderDynamicTextureTargetDescriptorError {
    #[error("dynamic texture target key namespace and target_id must be non-empty")]
    EmptyKey,
    #[error("dynamic texture target dimensions must be non-zero, got {width}x{height}")]
    ZeroDimensions { width: u32, height: u32 },
    #[error("depth dynamic texture targets cannot be color attachments or storage textures")]
    InvalidDepthUsage,
    #[error("color dynamic texture targets cannot use depth attachment usage")]
    InvalidColorUsage,
    #[error("dynamic texture target sample mode is incompatible with its format")]
    InvalidSampleModeForFormat,
    #[error("dynamic texture target sample_mode requires sampled usage")]
    SampleModeRequiresSampledUsage,
}
