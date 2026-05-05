use super::*;
use crate::plugins::render::{RenderFlowId, RenderPassId, RenderResourceId};
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
    pub is_depth: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeResourceKey {
    FlowOwned(RenderResourceId),
    SurfaceColor,
    SurfaceDepth,
}

impl fmt::Display for RuntimeResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlowOwned(id) => write!(f, "{}", id),
            Self::SurfaceColor => f.write_str(SURFACE_COLOR_RESOURCE_LABEL),
            Self::SurfaceDepth => f.write_str(SURFACE_DEPTH_RESOURCE_LABEL),
        }
    }
}

#[derive(Debug, Default)]
pub struct FlowRuntimeResources {
    pub textures: BTreeMap<RenderResourceId, RuntimeTextureResource>,
    pub buffers: BTreeMap<RenderResourceId, RuntimeBufferResource>,
    pub kinds: BTreeMap<RenderResourceId, RuntimeResourceKind>,
    pub descriptors: BTreeMap<RenderResourceId, RenderResourceDescriptor>,
    pub resource_ids_by_label: BTreeMap<String, RenderResourceId>,
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

    #[test]
    fn kind_of_resolves_label_alias_to_runtime_id() {
        let mut resources = FlowRuntimeResources::default();
        resources.kinds.insert(
            RenderResourceId::try_from_raw(42).unwrap(),
            RuntimeResourceKind::TextureLike,
        );
        resources.resource_ids_by_label.insert(
            "editor.viewport.v1.scene_color".to_string(),
            RenderResourceId::try_from_raw(42).unwrap(),
        );

        assert_eq!(
            resources.kind_of("editor.viewport.v1.scene_color"),
            Some(RuntimeResourceKind::TextureLike),
        );
    }

    #[test]
    fn capture_texture_class_resolves_label_alias_to_descriptor() {
        let mut resources = FlowRuntimeResources::default();
        resources.descriptors.insert(
            RenderResourceId::try_from_raw(7).unwrap(),
            RenderResourceDescriptor::color_target(RenderResourceId::try_from_raw(7).unwrap()),
        );
        resources.resource_ids_by_label.insert(
            "editor.viewport.v1.overlay".to_string(),
            RenderResourceId::try_from_raw(7).unwrap(),
        );

        assert_eq!(
            resources.capture_texture_class(
                "editor.viewport.v1.overlay",
                CaptureTextureClass::DepthTarget,
            ),
            CaptureTextureClass::ColorTarget,
        );
    }
}
