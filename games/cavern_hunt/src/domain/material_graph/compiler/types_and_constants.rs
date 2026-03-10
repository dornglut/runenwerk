use super::*;

// Owner: Cavern Hunt Domain - Material Graph
pub const MATERIAL_GRAPH_VERSION: u32 = 1;
pub const MATERIAL_REGISTER_LIMIT: u32 = 64;
pub const MATERIAL_OP_CONST_SCALAR: u32 = 0;
pub const MATERIAL_OP_CONST_VEC3: u32 = 1;
pub const MATERIAL_OP_WORLD_POS: u32 = 2;
pub const MATERIAL_OP_WORLD_NORMAL: u32 = 3;
pub const MATERIAL_OP_TRIPLANAR_NOISE: u32 = 4;
pub const MATERIAL_OP_FBM_NOISE: u32 = 5;
pub const MATERIAL_OP_SLOPE_MASK: u32 = 6;
pub const MATERIAL_OP_HEIGHT_MASK: u32 = 7;
pub const MATERIAL_OP_ADD: u32 = 8;
pub const MATERIAL_OP_MULTIPLY: u32 = 9;
pub const MATERIAL_OP_BLEND: u32 = 10;
pub const MATERIAL_OP_CLAMP01: u32 = 11;
pub const MATERIAL_OP_TO_VEC3: u32 = 12;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialPortTypeV1 {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphAssetV1 {
    pub version: u32,
    pub id: String,
    pub nodes: Vec<MaterialNodeV1>,
    pub outputs: MaterialGraphOutputsV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphOutputsV1 {
    pub base_color: String,
    pub roughness: String,
    pub metallic: String,
    pub normal_perturb: Option<String>,
    pub ao: Option<String>,
    pub emissive: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialNodeV1 {
    pub id: String,
    pub kind: MaterialNodeKindV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialNodeKindV1 {
    ConstantScalar {
        value: f32,
    },
    ConstantVec3 {
        value: [f32; 3],
    },
    WorldPos,
    WorldNormal,
    TriplanarNoise {
        position: String,
        normal: String,
        scale: f32,
        sharpness: f32,
        seed: f32,
    },
    FbmNoise {
        position: String,
        scale: f32,
        octaves: u8,
        lacunarity: f32,
        gain: f32,
        seed: f32,
    },
    SlopeMask {
        normal: String,
        power: f32,
        invert: bool,
    },
    HeightMask {
        position: String,
        min: f32,
        max: f32,
        falloff: f32,
    },
    Add {
        a: String,
        b: String,
    },
    Multiply {
        a: String,
        b: String,
    },
    Blend {
        a: String,
        b: String,
        mask: String,
    },
    Clamp01 {
        input: String,
    },
    ToVec3 {
        input: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialOpCodeV1 {
    pub op: u32,
    pub dst: u32,
    pub src_a: u32,
    pub src_b: u32,
    pub src_c: u32,
    pub const_idx: u32,
    pub flags: u32,
    pub output_type: MaterialPortTypeV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialProgramOutputsV1 {
    pub base_color: u32,
    pub roughness: u32,
    pub metallic: u32,
    pub normal_perturb: u32,
    pub ao: u32,
    pub emissive: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialProgramV1 {
    pub version: u32,
    pub graph_id: String,
    pub ops: Vec<MaterialOpCodeV1>,
    pub constants: Vec<[f32; 4]>,
    pub outputs: MaterialProgramOutputsV1,
    pub register_count: u32,
}
