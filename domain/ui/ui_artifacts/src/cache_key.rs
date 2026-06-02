use super::*;

pub(crate) fn stable_cache_key(
    program: &UiProgram,
    target_profile: UiRuntimeTargetProfile,
    target_profile_version: u32,
    package_ids: &[String],
    control_kind_ids: &[String],
    schema_ids: &[RuntimeSchemaRef],
    route_ids: &[RuntimeRouteRef],
    kernel_ids: &[String],
    capability_ids: &[String],
    source_map: &CompiledSourceMap,
) -> ArtifactCacheKey {
    let mut parts = Vec::new();
    parts.push(format!(
        "target:{target_profile:?}:{target_profile_version}"
    ));
    parts.extend(package_ids.iter().map(|value| format!("package:{value}")));
    parts.extend(
        control_kind_ids
            .iter()
            .map(|value| format!("control-kind:{value}")),
    );
    parts.extend(
        schema_ids
            .iter()
            .map(|schema| format!("schema:{}@{}", schema.schema_id, schema.schema_version)),
    );
    parts.extend(route_ids.iter().map(|route| {
        format!(
            "route:{}:{}@{}",
            route.route_id, route.payload_schema.schema_id, route.payload_schema.schema_version
        )
    }));
    parts.extend(kernel_ids.iter().map(|value| format!("kernel:{value}")));
    parts.extend(
        capability_ids
            .iter()
            .map(|value| format!("capability:{value}")),
    );
    parts.extend(source_map.entries.iter().map(|entry| {
        format!(
            "source-map:{:?}:{}:{}:{}",
            entry.table, entry.row, entry.source_id, entry.target_id
        )
    }));
    parts.push(format!(
        "tables:controls={}:layout={}:state={}:binding={}:visual={}",
        program.graphs.control.nodes.len(),
        program.graphs.layout.constraints.len(),
        program.graphs.state.requirements.len(),
        program.graphs.binding.bindings.len(),
        program.graphs.visual.operators.len()
    ));

    ArtifactCacheKey::new(format!(
        "ui-program:{}:{}:{:016x}",
        program.id.as_str(),
        program.version.value(),
        stable_hash(parts.iter().map(String::as_str))
    ))
}

fn stable_hash<'a>(parts: impl Iterator<Item = &'a str>) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
