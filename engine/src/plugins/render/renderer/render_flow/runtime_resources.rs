use super::*;
use crate::plugins::gpu::{GpuWorkResourceId, PreparedGpuData, UniformData};
use crate::plugins::render::{
    PreparedTargetBinding, RenderDynamicTextureTargetKey, RenderFlowId, RenderPassId,
    prepare_projected_uniform_bytes,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeResourceKind {
    TextureLike,
    BufferLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeBufferKind {
    Uniform,
    Storage,
}

#[derive(Debug)]
pub struct RuntimeTextureResource {
    pub texture: Texture,
    pub format: TextureFormat,
    pub size: (u32, u32),
    pub usage: TextureUsages,
    pub is_depth: bool,
    pub history_signature: Option<String>,
    pub generation: u64,
    pub reused_last_frame: bool,
}

#[derive(Debug)]
pub struct RuntimeBufferResource {
    pub buffer: Buffer,
    pub size: u64,
    pub kind: RuntimeBufferKind,
    pub generation: u64,
    pub reused_last_frame: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureAllocationSpec {
    pub size: (u32, u32),
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub is_depth: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BufferAllocationSpec {
    pub size: u64,
    pub usage: BufferUsages,
    pub kind: RuntimeBufferKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeResourceKey {
    FlowOwned(GpuWorkResourceId),
    InvocationUniform {
        invocation_id: String,
        resource_id: GpuWorkResourceId,
    },
    InvocationHistory {
        invocation_id: String,
        resource_id: GpuWorkResourceId,
    },
    DynamicTexture(RenderDynamicTextureTargetKey),
    SurfaceColor,
    SurfaceDepth,
}

impl fmt::Display for RuntimeResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlowOwned(id) => write!(f, "{}", id),
            Self::InvocationUniform {
                invocation_id,
                resource_id,
            } => write!(f, "{}@{}", resource_id, invocation_id),
            Self::InvocationHistory {
                invocation_id,
                resource_id,
            } => write!(f, "{}@history:{}", resource_id, invocation_id),
            Self::DynamicTexture(key) => write!(f, "{}", key),
            Self::SurfaceColor => f.write_str(SURFACE_COLOR_RESOURCE_LABEL),
            Self::SurfaceDepth => f.write_str(SURFACE_DEPTH_RESOURCE_LABEL),
        }
    }
}

#[derive(Debug, Default)]
pub struct FlowRuntimeResources {
    pub textures: BTreeMap<GpuWorkResourceId, RuntimeTextureResource>,
    pub buffers: BTreeMap<GpuWorkResourceId, RuntimeBufferResource>,
    pub invocation_uniform_buffers: BTreeMap<(String, GpuWorkResourceId), RuntimeBufferResource>,
    pub invocation_history_textures: BTreeMap<(String, GpuWorkResourceId), RuntimeTextureResource>,
    pub active_invocation_uniform_scope: Option<String>,
    pub kinds: BTreeMap<GpuWorkResourceId, RuntimeResourceKind>,
    pub descriptors: BTreeMap<GpuWorkResourceId, RenderResourceDeclaration>,
    pub resource_ids_by_label: BTreeMap<String, GpuWorkResourceId>,
    pub target_alias_bindings: BTreeMap<String, PreparedTargetBinding>,
}

impl FlowRuntimeResources {
    pub(super) fn prepare_uniform_upload(
        &self,
        resource_id: GpuWorkResourceId,
        bytes: &[u8],
    ) -> anyhow::Result<PreparedGpuData<UniformData>> {
        let declaration = self.descriptors.get(&resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "uniform upload references undeclared logical resource '{}'",
                resource_id
            )
        })?;
        let RenderResourceDeclaration::Uniform(uniform) = declaration else {
            anyhow::bail!(
                "uniform upload references non-uniform logical resource '{}'",
                resource_id
            );
        };
        let common = uniform.handle().descriptor().common();
        Ok(prepare_projected_uniform_bytes(
            common.label().as_str(),
            bytes.to_vec(),
            uniform.layout(),
            common.provenance().clone(),
        )?)
    }
}

#[derive(Debug)]
pub struct ResolvedTextureRef<'a> {
    pub id: RuntimeResourceKey,
    pub texture: &'a Texture,
    pub format: TextureFormat,
    pub size: (u32, u32),
    pub is_depth: bool,
    pub generation: Option<u64>,
}

#[derive(Debug)]
pub struct ResolvedBufferRef<'a> {
    pub id: RuntimeResourceKey,
    pub buffer: &'a Buffer,
    pub size: u64,
    pub kind: RuntimeBufferKind,
    pub generation: Option<u64>,
}

#[derive(Debug)]
pub enum RuntimeTextureView<'a> {
    Borrowed(&'a TextureView),
    Owned(TextureView),
}

impl<'a> RuntimeTextureView<'a> {
    pub fn as_ref(&self) -> &TextureView {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }
}

#[derive(Debug)]
pub struct ResolvedColorTargetView<'a> {
    pub view: RuntimeTextureView<'a>,
    pub format: TextureFormat,
}

#[derive(Debug)]
pub struct ResolvedDepthTargetView {
    pub view: TextureView,
    pub format: TextureFormat,
}

mod inspect;
mod realize;
mod resolve;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{GpuResourceLifetime, GpuWorkResourceIdAllocator};
    use crate::plugins::render::RenderTextureIntent;
    use std::num::NonZeroU64;

    fn resource(local: u64) -> GpuWorkResourceId {
        let mut allocator = GpuWorkResourceIdAllocator::for_owner_scope(
            NonZeroU64::new(1).expect("test owner scope is nonzero"),
        );
        (1..=local)
            .map(|_| {
                allocator
                    .allocate()
                    .expect("test allocation should succeed")
            })
            .last()
            .expect("test local value is nonzero")
    }

    #[test]
    fn kind_of_resolves_label_alias_to_runtime_id() {
        let mut resources = FlowRuntimeResources::default();
        let id = resource(42);
        resources.kinds.insert(id, RuntimeResourceKind::TextureLike);
        resources
            .resource_ids_by_label
            .insert("editor.viewport.v1.scene_color".to_string(), id);

        assert_eq!(
            resources.kind_of("editor.viewport.v1.scene_color"),
            Some(RuntimeResourceKind::TextureLike),
        );
    }

    #[test]
    fn capture_texture_class_resolves_label_alias_to_descriptor() {
        let mut resources = FlowRuntimeResources::default();
        resources.descriptors.insert(
            resource(7),
            RenderResourceDeclaration::declare_color_attachment(resource(7), "overlay"),
        );
        resources
            .resource_ids_by_label
            .insert("editor.viewport.v1.overlay".to_string(), resource(7));

        assert_eq!(
            resources.capture_texture_class(
                "editor.viewport.v1.overlay",
                CaptureTextureClass::DepthTarget,
            ),
            CaptureTextureClass::ColorTarget,
        );
    }

    #[test]
    fn flow_owned_texture_allocation_honors_fixed_size_exact_format_and_usage() {
        let id = resource(11);
        let descriptor = RenderResourceDeclaration::StorageImage(RenderTextureIntent {
            id,
            label: "fixed storage".to_string(),
            lifetime: GpuResourceLifetime::Retained,
            texture: crate::plugins::render::RenderTextureDescriptor {
                size: crate::plugins::render::RenderTextureSizePolicy::Fixed {
                    width: 320,
                    height: 180,
                },
                format: crate::plugins::render::RenderTextureFormatPolicy::Exact(
                    crate::plugins::render::RenderTextureTargetFormat::R32Uint,
                ),
                usage: crate::plugins::render::RenderTextureTargetUsage {
                    color_attachment: false,
                    depth_attachment: false,
                    sampled: true,
                    storage: true,
                    copy_src: false,
                    copy_dst: true,
                },
                sample_mode: crate::plugins::render::RenderTextureSampleMode::Uint,
            },
        });

        let spec = FlowRuntimeResources::texture_allocation_spec(
            &descriptor,
            (1920, 1080),
            TextureFormat::Bgra8UnormSrgb,
        )
        .expect("normalized descriptor lowering should succeed")
        .expect("storage texture should allocate");

        assert_eq!(spec.size, (320, 180));
        assert_eq!(spec.format, TextureFormat::R32Uint);
        assert!(spec.usage.contains(TextureUsages::TEXTURE_BINDING));
        assert!(spec.usage.contains(TextureUsages::STORAGE_BINDING));
        assert!(spec.usage.contains(TextureUsages::COPY_DST));
        assert!(!spec.usage.contains(TextureUsages::RENDER_ATTACHMENT));
        assert!(!spec.usage.contains(TextureUsages::COPY_SRC));
    }

    #[test]
    fn flow_owned_texture_allocation_resolves_surface_policy_from_frame() {
        let id = resource(12);
        let descriptor = RenderResourceDeclaration::declare_color_attachment(id, "color");

        let spec = FlowRuntimeResources::texture_allocation_spec(
            &descriptor,
            (1280, 720),
            TextureFormat::Rgba8UnormSrgb,
        )
        .expect("normalized descriptor lowering should succeed")
        .expect("color target should allocate");

        assert_eq!(spec.size, (1280, 720));
        assert_eq!(spec.format, TextureFormat::Rgba8UnormSrgb);
        assert!(spec.usage.contains(TextureUsages::RENDER_ATTACHMENT));
        assert!(spec.usage.contains(TextureUsages::TEXTURE_BINDING));
    }

    #[test]
    fn exact_color_target_allocation_ignores_surface_format_policy() {
        let id = resource(13);
        let descriptor = RenderResourceDeclaration::declare_color_attachment_exact(
            id,
            "exact color",
            crate::plugins::render::RenderTextureTargetFormat::Rgba8Unorm,
        );

        let spec = FlowRuntimeResources::texture_allocation_spec(
            &descriptor,
            (1280, 720),
            TextureFormat::Rgba8UnormSrgb,
        )
        .expect("normalized descriptor lowering should succeed")
        .expect("exact color target should allocate");

        assert_eq!(spec.size, (1280, 720));
        assert_eq!(spec.format, TextureFormat::Rgba8Unorm);
        assert!(spec.usage.contains(TextureUsages::RENDER_ATTACHMENT));
        assert!(spec.usage.contains(TextureUsages::TEXTURE_BINDING));
    }
}
