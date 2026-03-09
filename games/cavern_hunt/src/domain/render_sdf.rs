use crate::CavernCameraState;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfAgent {
	pub pos: [f32; 2],
	pub radius: f32,
	pub health_ratio: f32,
	pub team: u32,
	pub kind: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfGeometryPrimitive {
	pub shape_kind: u32,
	pub op_kind: u32,
	pub material_class: u32,
	pub material_instance: u32,
	pub p0: [f32; 4],
	pub p1: [f32; 4],
	pub p2: [f32; 4],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfMaterialProgramHeader {
	pub class_id: u32,
	pub op_offset: u32,
	pub op_count: u32,
	pub const_offset: u32,
	pub const_count: u32,
	pub base_color_slot: u32,
	pub roughness_slot: u32,
	pub metallic_slot: u32,
	pub normal_perturb_slot: u32,
	pub ao_slot: u32,
	pub emissive_slot: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CavernSdfMaterialOp {
	pub op: u32,
	pub dst: u32,
	pub src_a: u32,
	pub src_b: u32,
	pub src_c: u32,
	pub const_idx: u32,
	pub flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CavernSdfWorldFrame {
	pub world_bounds: [f32; 4],
	pub floor_height: f32,
	pub rock_height: f32,
	pub camera: CavernCameraState,
	pub render_mode: u32,
	pub gi_mode: u32,
	pub gi_quality: u32,
	pub gi_sample_budget: u32,
	pub material_program_headers: Vec<CavernSdfMaterialProgramHeader>,
	pub material_ops: Vec<CavernSdfMaterialOp>,
	pub material_constants: Vec<[f32; 4]>,
	pub geometry_primitives: Vec<CavernSdfGeometryPrimitive>,
	pub agents: Vec<CavernSdfAgent>,
}

impl Default for CavernSdfWorldFrame {
	fn default() -> Self {
		Self {
			world_bounds: [-24.0, -24.0, 24.0, 24.0],
			floor_height: 0.0,
			rock_height: 3.8,
			camera: CavernCameraState::default(),
			render_mode: crate::CAVERN_RENDER_MODE_MATERIAL_GRAPH,
			gi_mode: crate::CAVERN_GI_MODE_AO_BENT,
			gi_quality: 1,
			gi_sample_budget: 14,
			material_program_headers: Vec::new(),
			material_ops: Vec::new(),
			material_constants: Vec::new(),
			geometry_primitives: Vec::new(),
			agents: Vec::new(),
		}
	}
}