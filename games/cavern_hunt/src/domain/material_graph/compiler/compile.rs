use super::*;

// Owner: Cavern Hunt Domain - Material Graph
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
