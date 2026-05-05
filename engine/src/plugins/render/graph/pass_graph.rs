use crate::plugins::render::api::ids::RenderFeatureId;
use crate::plugins::render::api::{ComputeDispatchDescriptor, PassParamBinding};
use crate::plugins::render::{RenderPassId, RenderResourceId, ShaderHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPassKind {
    Compute,
    Fullscreen,
    BuiltinUiComposite,
    Graphics,
    Copy,
    Present,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderShaderReference {
    AssetPath(String),
    RegistryHandle(ShaderHandle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderVertexStepMode {
    Vertex,
    Instance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderVertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
}

impl RenderVertexFormat {
    pub const fn size_bytes(self) -> u64 {
        match self {
            Self::Float32 | Self::Uint32 | Self::Sint32 => 4,
            Self::Float32x2 | Self::Uint32x2 | Self::Sint32x2 => 8,
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 12,
            Self::Float32x4 | Self::Uint32x4 | Self::Sint32x4 => 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderVertexAttribute {
    pub shader_location: u32,
    pub offset: u64,
    pub format: RenderVertexFormat,
}

impl RenderVertexAttribute {
    pub const fn new(shader_location: u32, offset: u64, format: RenderVertexFormat) -> Self {
        Self {
            shader_location,
            offset,
            format,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderVertexBufferLayout {
    pub slot: u32,
    pub array_stride: u64,
    pub step_mode: RenderVertexStepMode,
    pub attributes: Vec<RenderVertexAttribute>,
}

impl RenderVertexBufferLayout {
    pub fn vertex(slot: u32, array_stride: u64) -> Self {
        Self {
            slot,
            array_stride,
            step_mode: RenderVertexStepMode::Vertex,
            attributes: Vec::new(),
        }
    }

    pub fn instance(slot: u32, array_stride: u64) -> Self {
        Self {
            slot,
            array_stride,
            step_mode: RenderVertexStepMode::Instance,
            attributes: Vec::new(),
        }
    }

    pub fn attribute(
        mut self,
        shader_location: u32,
        offset: u64,
        format: RenderVertexFormat,
    ) -> Self {
        self.attributes
            .push(RenderVertexAttribute::new(shader_location, offset, format));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderDrawDescriptor {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

impl RenderDrawDescriptor {
    pub const fn new(vertex_count: u32, instance_count: u32) -> Self {
        Self {
            vertex_count,
            instance_count,
            first_vertex: 0,
            first_instance: 0,
        }
    }

    pub const fn with_offsets(
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Self {
        Self {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderPassNode {
    pub id: RenderPassId,
    pub label: String,
    pub kind: RenderPassKind,
    pub feature_id: Option<RenderFeatureId>,
    pub shader: Option<RenderShaderReference>,
    pub reads: Vec<RenderResourceId>,
    pub writes: Vec<RenderResourceId>,
    pub depends_on: Vec<RenderPassId>,
    pub workgroup_size: Option<[u32; 3]>,
    pub clear_color: Option<[f32; 4]>,
    pub compute_dispatch: Option<ComputeDispatchDescriptor>,
    pub sampled_textures: Vec<RenderResourceId>,
    pub write_textures: Vec<RenderResourceId>,
    pub vertex_buffers: Vec<RenderResourceId>,
    pub vertex_buffer_layouts: Vec<RenderVertexBufferLayout>,
    pub index_buffers: Vec<RenderResourceId>,
    pub instance_buffers: Vec<RenderResourceId>,
    pub instance_buffer_layouts: Vec<RenderVertexBufferLayout>,
    pub indirect_buffers: Vec<RenderResourceId>,
    pub depth_target: Option<RenderResourceId>,
    pub draw: Option<RenderDrawDescriptor>,
    pub uniform_bindings: Vec<PassParamBinding>,
}

impl RenderPassNode {
    pub fn new(
        id: impl Into<RenderPassId>,
        label: impl Into<String>,
        kind: RenderPassKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            feature_id: None,
            shader: None,
            reads: Vec::new(),
            writes: Vec::new(),
            depends_on: Vec::new(),
            workgroup_size: None,
            clear_color: None,
            compute_dispatch: None,
            sampled_textures: Vec::new(),
            write_textures: Vec::new(),
            vertex_buffers: Vec::new(),
            vertex_buffer_layouts: Vec::new(),
            index_buffers: Vec::new(),
            instance_buffers: Vec::new(),
            instance_buffer_layouts: Vec::new(),
            indirect_buffers: Vec::new(),
            depth_target: None,
            draw: None,
            uniform_bindings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PassGraph {
    pub passes: Vec<RenderPassNode>,
}

impl PassGraph {
    pub fn add_pass(&mut self, pass: RenderPassNode) {
        self.passes.push(pass);
    }
}
