use crate::plugins::render::api::PassParamBinding;
use crate::plugins::render::{RenderPassId, RenderResourceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPassKind {
    Compute,
    Fullscreen,
    BuiltinUiComposite,
    Graphics,
    Copy,
    Present,
}

#[derive(Debug, Clone)]
pub struct RenderPassNode {
    pub id: RenderPassId,
    pub kind: RenderPassKind,
    pub shader: Option<String>,
    pub reads: Vec<RenderResourceId>,
    pub writes: Vec<RenderResourceId>,
    pub depends_on: Vec<RenderPassId>,
    pub workgroup_size: Option<[u32; 3]>,
    pub clear_color: Option<[f32; 4]>,
    pub sampled_textures: Vec<RenderResourceId>,
    pub write_textures: Vec<RenderResourceId>,
    pub vertex_buffers: Vec<RenderResourceId>,
    pub index_buffers: Vec<RenderResourceId>,
    pub instance_buffers: Vec<RenderResourceId>,
    pub indirect_buffers: Vec<RenderResourceId>,
    pub depth_target: Option<RenderResourceId>,
    pub uniform_bindings: Vec<PassParamBinding>,
}

impl RenderPassNode {
    pub fn new(id: impl Into<RenderPassId>, kind: RenderPassKind) -> Self {
        Self {
            id: id.into(),
            kind,
            shader: None,
            reads: Vec::new(),
            writes: Vec::new(),
            depends_on: Vec::new(),
            workgroup_size: None,
            clear_color: None,
            sampled_textures: Vec::new(),
            write_textures: Vec::new(),
            vertex_buffers: Vec::new(),
            index_buffers: Vec::new(),
            instance_buffers: Vec::new(),
            indirect_buffers: Vec::new(),
            depth_target: None,
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
