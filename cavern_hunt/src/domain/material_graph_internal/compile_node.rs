// Owner: Cavern Hunt Domain - Material Graph
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
