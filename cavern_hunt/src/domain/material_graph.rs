use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCompileError {
    pub node: Option<String>,
    pub message: String,
}

impl MaterialCompileError {
    fn graph(message: impl Into<String>) -> Self {
        Self {
            node: None,
            message: message.into(),
        }
    }

    fn node(node: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node: Some(node.into()),
            message: message.into(),
        }
    }
}

pub fn compile_material_graph(
    asset: &MaterialGraphAssetV1,
) -> Result<MaterialProgramV1, MaterialCompileError> {
    if asset.version != MATERIAL_GRAPH_VERSION {
        return Err(MaterialCompileError::graph(format!(
            "unsupported material graph version {}; expected {}",
            asset.version, MATERIAL_GRAPH_VERSION
        )));
    }
    let graph_id = asset.id.trim();
    if graph_id.is_empty() {
        return Err(MaterialCompileError::graph("graph id cannot be empty"));
    }

    let mut seen = BTreeSet::new();
    let mut slots = BTreeMap::<String, (u32, MaterialPortTypeV1)>::new();
    let mut ops = Vec::new();
    let mut constants = Vec::new();
    let mut next_slot = 0_u32;

    for node in &asset.nodes {
        let node_id = node.id.trim();
        if node_id.is_empty() {
            return Err(MaterialCompileError::graph("node id cannot be empty"));
        }
        if !seen.insert(node_id.to_string()) {
            return Err(MaterialCompileError::node(
                node_id,
                "duplicate node id in material graph",
            ));
        }

        let dst = next_slot;
        next_slot = next_slot.saturating_add(1);
        if next_slot > MATERIAL_REGISTER_LIMIT {
            return Err(MaterialCompileError::node(
                node_id,
                format!(
                    "register limit exceeded (>{}); reduce node count",
                    MATERIAL_REGISTER_LIMIT
                ),
            ));
        }

        let (mut op, ty) = compile_node(&node.kind, node_id, dst, &slots, &mut constants)?;
        if ty == MaterialPortTypeV1::Vec3 {
            op.flags |= 0x8000_0000;
        }
        op.output_type = ty;
        ops.push(op);
        slots.insert(node_id.to_string(), (dst, ty));
    }

    let mut defaults = MaterialDefaults::default();
    let base_color = resolve_slot(
        &asset.outputs.base_color,
        MaterialPortTypeV1::Vec3,
        &slots,
        "outputs.base_color",
    )?;
    let roughness = resolve_slot(
        &asset.outputs.roughness,
        MaterialPortTypeV1::Scalar,
        &slots,
        "outputs.roughness",
    )?;
    let metallic = resolve_slot(
        &asset.outputs.metallic,
        MaterialPortTypeV1::Scalar,
        &slots,
        "outputs.metallic",
    )?;
    let normal_perturb = resolve_optional_slot(
        asset.outputs.normal_perturb.as_deref(),
        MaterialPortTypeV1::Vec3,
        &slots,
        "outputs.normal_perturb",
        &mut defaults,
        &mut ops,
        &mut constants,
        &mut next_slot,
        [0.0, 0.0, 0.0, 0.0],
        MaterialPortTypeV1::Vec3,
    )?;
    let ao = resolve_optional_slot(
        asset.outputs.ao.as_deref(),
        MaterialPortTypeV1::Scalar,
        &slots,
        "outputs.ao",
        &mut defaults,
        &mut ops,
        &mut constants,
        &mut next_slot,
        [1.0, 0.0, 0.0, 0.0],
        MaterialPortTypeV1::Scalar,
    )?;
    let emissive = resolve_optional_slot(
        asset.outputs.emissive.as_deref(),
        MaterialPortTypeV1::Vec3,
        &slots,
        "outputs.emissive",
        &mut defaults,
        &mut ops,
        &mut constants,
        &mut next_slot,
        [0.0, 0.0, 0.0, 0.0],
        MaterialPortTypeV1::Vec3,
    )?;

    Ok(MaterialProgramV1 {
        version: MATERIAL_GRAPH_VERSION,
        graph_id: graph_id.to_string(),
        ops,
        constants,
        outputs: MaterialProgramOutputsV1 {
            base_color,
            roughness,
            metallic,
            normal_perturb,
            ao,
            emissive,
        },
        register_count: next_slot,
    })
}

#[derive(Default)]
struct MaterialDefaults {
    count: u32,
}

fn resolve_optional_slot(
    candidate: Option<&str>,
    expected: MaterialPortTypeV1,
    slots: &BTreeMap<String, (u32, MaterialPortTypeV1)>,
    label: &str,
    defaults: &mut MaterialDefaults,
    ops: &mut Vec<MaterialOpCodeV1>,
    constants: &mut Vec<[f32; 4]>,
    next_slot: &mut u32,
    default_value: [f32; 4],
    default_ty: MaterialPortTypeV1,
) -> Result<u32, MaterialCompileError> {
    if let Some(candidate) = candidate {
        return resolve_slot(candidate, expected, slots, label);
    }
    let dst = *next_slot;
    *next_slot = next_slot.saturating_add(1);
    defaults.count = defaults.count.saturating_add(1);
    if *next_slot > MATERIAL_REGISTER_LIMIT {
        return Err(MaterialCompileError::graph(format!(
            "register limit exceeded while allocating default for {label}"
        )));
    }
    let const_idx = constants.len() as u32;
    constants.push(default_value);
    ops.push(MaterialOpCodeV1 {
        op: match default_ty {
            MaterialPortTypeV1::Scalar => MATERIAL_OP_CONST_SCALAR,
            _ => MATERIAL_OP_CONST_VEC3,
        },
        dst,
        src_a: 0,
        src_b: 0,
        src_c: 0,
        const_idx,
        flags: 0,
        output_type: default_ty,
    });
    Ok(dst)
}

fn resolve_slot(
    id: &str,
    expected: MaterialPortTypeV1,
    slots: &BTreeMap<String, (u32, MaterialPortTypeV1)>,
    label: &str,
) -> Result<u32, MaterialCompileError> {
    let key = id.trim();
    let Some((slot, ty)) = slots.get(key).copied() else {
        return Err(MaterialCompileError::graph(format!(
            "{label} references unknown node '{key}'"
        )));
    };
    if ty != expected {
        return Err(MaterialCompileError::graph(format!(
            "{label} expected {:?} but '{key}' is {:?}",
            expected, ty
        )));
    }
    Ok(slot)
}

fn resolve_input(
    slots: &BTreeMap<String, (u32, MaterialPortTypeV1)>,
    input: &str,
    expected: MaterialPortTypeV1,
    node_id: &str,
    label: &str,
) -> Result<u32, MaterialCompileError> {
    let key = input.trim();
    let Some((slot, ty)) = slots.get(key).copied() else {
        return Err(MaterialCompileError::node(
            node_id,
            format!("{label} references unknown node '{key}'"),
        ));
    };
    if ty != expected {
        return Err(MaterialCompileError::node(
            node_id,
            format!("{label} expected {:?} but '{key}' is {:?}", expected, ty),
        ));
    }
    Ok(slot)
}

fn resolve_numeric_input(
    slots: &BTreeMap<String, (u32, MaterialPortTypeV1)>,
    input: &str,
    node_id: &str,
    label: &str,
) -> Result<(u32, MaterialPortTypeV1), MaterialCompileError> {
    let key = input.trim();
    let Some((slot, ty)) = slots.get(key).copied() else {
        return Err(MaterialCompileError::node(
            node_id,
            format!("{label} references unknown node '{key}'"),
        ));
    };
    if matches!(ty, MaterialPortTypeV1::Scalar | MaterialPortTypeV1::Vec3) {
        Ok((slot, ty))
    } else {
        Err(MaterialCompileError::node(
            node_id,
            format!("{label} must be Scalar or Vec3, got {:?}", ty),
        ))
    }
}

fn compile_node(
    node: &MaterialNodeKindV1,
    node_id: &str,
    dst: u32,
    slots: &BTreeMap<String, (u32, MaterialPortTypeV1)>,
    constants: &mut Vec<[f32; 4]>,
) -> Result<(MaterialOpCodeV1, MaterialPortTypeV1), MaterialCompileError> {
    let mut op = MaterialOpCodeV1 {
        op: MATERIAL_OP_CONST_SCALAR,
        dst,
        src_a: 0,
        src_b: 0,
        src_c: 0,
        const_idx: 0,
        flags: 0,
        output_type: MaterialPortTypeV1::Scalar,
    };

    let ty = match node {
        MaterialNodeKindV1::ConstantScalar { value } => {
            let const_idx = constants.len() as u32;
            constants.push([*value, 0.0, 0.0, 0.0]);
            op.op = MATERIAL_OP_CONST_SCALAR;
            op.const_idx = const_idx;
            MaterialPortTypeV1::Scalar
        }
        MaterialNodeKindV1::ConstantVec3 { value } => {
            let const_idx = constants.len() as u32;
            constants.push([value[0], value[1], value[2], 0.0]);
            op.op = MATERIAL_OP_CONST_VEC3;
            op.const_idx = const_idx;
            MaterialPortTypeV1::Vec3
        }
        MaterialNodeKindV1::WorldPos => {
            op.op = MATERIAL_OP_WORLD_POS;
            MaterialPortTypeV1::Vec3
        }
        MaterialNodeKindV1::WorldNormal => {
            op.op = MATERIAL_OP_WORLD_NORMAL;
            MaterialPortTypeV1::Vec3
        }
        MaterialNodeKindV1::TriplanarNoise {
            position,
            normal,
            scale,
            sharpness,
            seed,
        } => {
            op.op = MATERIAL_OP_TRIPLANAR_NOISE;
            op.src_a = resolve_input(
                slots,
                position,
                MaterialPortTypeV1::Vec3,
                node_id,
                "TriplanarNoise.position",
            )?;
            op.src_b = resolve_input(
                slots,
                normal,
                MaterialPortTypeV1::Vec3,
                node_id,
                "TriplanarNoise.normal",
            )?;
            op.const_idx = constants.len() as u32;
            constants.push([*scale, *sharpness, *seed, 0.0]);
            MaterialPortTypeV1::Scalar
        }
        MaterialNodeKindV1::FbmNoise {
            position,
            scale,
            octaves,
            lacunarity,
            gain,
            seed,
        } => {
            op.op = MATERIAL_OP_FBM_NOISE;
            op.src_a = resolve_input(
                slots,
                position,
                MaterialPortTypeV1::Vec3,
                node_id,
                "FbmNoise.position",
            )?;
            op.const_idx = constants.len() as u32;
            constants.push([*scale, *lacunarity, *gain, *seed]);
            op.flags = (*octaves).max(1) as u32;
            MaterialPortTypeV1::Scalar
        }
        MaterialNodeKindV1::SlopeMask {
            normal,
            power,
            invert,
        } => {
            op.op = MATERIAL_OP_SLOPE_MASK;
            op.src_a = resolve_input(
                slots,
                normal,
                MaterialPortTypeV1::Vec3,
                node_id,
                "SlopeMask.normal",
            )?;
            op.const_idx = constants.len() as u32;
            constants.push([*power, if *invert { 1.0 } else { 0.0 }, 0.0, 0.0]);
            MaterialPortTypeV1::Scalar
        }
        MaterialNodeKindV1::HeightMask {
            position,
            min,
            max,
            falloff,
        } => {
            op.op = MATERIAL_OP_HEIGHT_MASK;
            op.src_a = resolve_input(
                slots,
                position,
                MaterialPortTypeV1::Vec3,
                node_id,
                "HeightMask.position",
            )?;
            op.const_idx = constants.len() as u32;
            constants.push([*min, *max, *falloff, 0.0]);
            MaterialPortTypeV1::Scalar
        }
        MaterialNodeKindV1::Add { a, b } => {
            let (src_a, ty_a) = resolve_numeric_input(slots, a, node_id, "Add.a")?;
            let (src_b, ty_b) = resolve_numeric_input(slots, b, node_id, "Add.b")?;
            op.op = MATERIAL_OP_ADD;
            op.src_a = src_a;
            op.src_b = src_b;
            if ty_a == MaterialPortTypeV1::Vec3 || ty_b == MaterialPortTypeV1::Vec3 {
                MaterialPortTypeV1::Vec3
            } else {
                MaterialPortTypeV1::Scalar
            }
        }
        MaterialNodeKindV1::Multiply { a, b } => {
            let (src_a, ty_a) = resolve_numeric_input(slots, a, node_id, "Multiply.a")?;
            let (src_b, ty_b) = resolve_numeric_input(slots, b, node_id, "Multiply.b")?;
            op.op = MATERIAL_OP_MULTIPLY;
            op.src_a = src_a;
            op.src_b = src_b;
            if ty_a == MaterialPortTypeV1::Vec3 || ty_b == MaterialPortTypeV1::Vec3 {
                MaterialPortTypeV1::Vec3
            } else {
                MaterialPortTypeV1::Scalar
            }
        }
        MaterialNodeKindV1::Blend { a, b, mask } => {
            let (src_a, ty_a) = resolve_numeric_input(slots, a, node_id, "Blend.a")?;
            let (src_b, ty_b) = resolve_numeric_input(slots, b, node_id, "Blend.b")?;
            if ty_a != ty_b {
                return Err(MaterialCompileError::node(
                    node_id,
                    "Blend input types must match",
                ));
            }
            op.op = MATERIAL_OP_BLEND;
            op.src_a = src_a;
            op.src_b = src_b;
            op.src_c = resolve_input(
                slots,
                mask,
                MaterialPortTypeV1::Scalar,
                node_id,
                "Blend.mask",
            )?;
            ty_a
        }
        MaterialNodeKindV1::Clamp01 { input } => {
            let (src, ty) = resolve_numeric_input(slots, input, node_id, "Clamp01.input")?;
            op.op = MATERIAL_OP_CLAMP01;
            op.src_a = src;
            ty
        }
        MaterialNodeKindV1::ToVec3 { input } => {
            op.op = MATERIAL_OP_TO_VEC3;
            op.src_a = resolve_input(
                slots,
                input,
                MaterialPortTypeV1::Scalar,
                node_id,
                "ToVec3.input",
            )?;
            MaterialPortTypeV1::Vec3
        }
    };

    Ok((op, ty))
}

include!("material_graph_internal/tests.rs");
