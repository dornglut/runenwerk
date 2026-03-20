use super::*;

// Owner: Cavern Hunt Domain - Material Graph
#[derive(Default, ecs::Resource)]
pub(super) struct MaterialDefaults {
    count: u32,
}

pub(super) fn resolve_optional_slot(
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

pub(super) fn resolve_slot(
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

pub(super) fn resolve_input(
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

pub(super) fn resolve_numeric_input(
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
