use super::RenderPassMaterialBindingEvidence;
use crate::plugins::render::{
    PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT,
    PREPARED_MATERIAL_PARAMETER_PAYLOAD_V1_MAX_PARAMETERS, PreparedMaterialFeatureContribution,
    PreparedModelMeshMaterialSelection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMeshMaterialHandoffDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialHandoffDiagnostic {
    pub severity: RenderMeshMaterialHandoffDiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
}

impl RenderMeshMaterialHandoffDiagnostic {
    fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: RenderMeshMaterialHandoffDiagnosticSeverity::Error,
            code,
            message: message.into(),
        }
    }

    fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: RenderMeshMaterialHandoffDiagnosticSeverity::Warning,
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderMeshMaterialHandoffInspectionRequest<'a> {
    pub prepared_material: &'a PreparedMaterialFeatureContribution,
    pub material_passes: &'a [RenderPassMaterialBindingEvidence],
    pub require_model_mesh_selection: bool,
    pub require_material_consuming_pass: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderMeshMaterialHandoffCounts {
    pub material_instance_count: usize,
    pub texture_binding_count: usize,
    pub material_binding_slot_count: usize,
    pub model_mesh_selection_count: usize,
    pub material_consuming_pass_count: usize,
    pub pass_exposed_model_mesh_selection_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialHandoffInspection {
    pub counts: RenderMeshMaterialHandoffCounts,
    pub scene_shader_identity: Option<String>,
    pub scene_shader_path: Option<String>,
    pub shader_artifact_id: Option<String>,
    pub shader_cache_key: Option<String>,
    pub material_table_identity: Option<String>,
    pub resource_layout_identity: Option<String>,
    pub diagnostics: Vec<RenderMeshMaterialHandoffDiagnostic>,
}

impl RenderMeshMaterialHandoffInspection {
    pub fn is_ready(&self) -> bool {
        !self.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == RenderMeshMaterialHandoffDiagnosticSeverity::Error
        })
    }
}

pub fn inspect_render_mesh_material_handoff(
    request: RenderMeshMaterialHandoffInspectionRequest<'_>,
) -> RenderMeshMaterialHandoffInspection {
    let material = request.prepared_material;
    let mut diagnostics = Vec::new();

    if let Err(error) = material.validate_portable_limits() {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "portable_limits",
            format!("prepared material handoff violates portable limits: {error}"),
        ));
    }

    inspect_material_instances(material, &mut diagnostics);
    inspect_scene_bundle(material, &mut diagnostics);
    inspect_binding_slots(material, &mut diagnostics);
    inspect_model_mesh_selections(request, &mut diagnostics);
    inspect_material_passes(request, &mut diagnostics);

    let scene_bundle = material.scene_bundle.as_ref();
    RenderMeshMaterialHandoffInspection {
        counts: RenderMeshMaterialHandoffCounts {
            material_instance_count: material.instances.len(),
            texture_binding_count: material
                .instances
                .iter()
                .map(|instance| instance.texture_bindings.len())
                .sum(),
            material_binding_slot_count: material.binding_table.slots.len(),
            model_mesh_selection_count: material.model_mesh_material_selections.len(),
            material_consuming_pass_count: request
                .material_passes
                .iter()
                .filter(|pass| pass.consumes_material_resources)
                .count(),
            pass_exposed_model_mesh_selection_count: request
                .material_passes
                .iter()
                .map(|pass| pass.model_mesh_material_selections_available_to_pass.len())
                .sum(),
        },
        scene_shader_identity: scene_bundle.map(|bundle| bundle.shader_identity.clone()),
        scene_shader_path: scene_bundle.map(|bundle| bundle.shader_path.clone()),
        shader_artifact_id: scene_bundle.map(|bundle| bundle.shader_artifact_id.clone()),
        shader_cache_key: scene_bundle.map(|bundle| bundle.shader_cache_key.clone()),
        material_table_identity: scene_bundle.map(|bundle| bundle.material_table_identity.clone()),
        resource_layout_identity: scene_bundle
            .map(|bundle| bundle.resource_layout_identity.clone()),
        diagnostics,
    }
}

fn inspect_material_instances(
    material: &PreparedMaterialFeatureContribution,
    diagnostics: &mut Vec<RenderMeshMaterialHandoffDiagnostic>,
) {
    if material.instances.is_empty() {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_material_instance",
            "prepared material handoff has no material instances",
        ));
    }

    for (index, instance) in material.instances.iter().enumerate() {
        if instance.material_instance_id.trim().is_empty() {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "missing_material_instance_identity",
                format!("material instance {index} has no source-backed identity"),
            ));
        }
        if instance.specialization_key_fragment.trim().is_empty() {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::warning(
                "missing_specialization_fragment",
                format!("material instance {index} has no specialization fragment"),
            ));
        }
        if instance.parameter_payload.parameters.len()
            > PREPARED_MATERIAL_PARAMETER_PAYLOAD_V1_MAX_PARAMETERS
        {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "parameter_payload_limit",
                format!(
                    "material instance {index} has {} parameters, portable limit is {}",
                    instance.parameter_payload.parameters.len(),
                    PREPARED_MATERIAL_PARAMETER_PAYLOAD_V1_MAX_PARAMETERS
                ),
            ));
        }
    }
}

fn inspect_scene_bundle(
    material: &PreparedMaterialFeatureContribution,
    diagnostics: &mut Vec<RenderMeshMaterialHandoffDiagnostic>,
) {
    let Some(bundle) = material.scene_bundle.as_ref() else {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_scene_material_bundle",
            "prepared material handoff has no scene material shader bundle",
        ));
        return;
    };

    for (code, label, value) in [
        (
            "missing_shader_artifact",
            "shader artifact id",
            bundle.shader_artifact_id.as_str(),
        ),
        (
            "missing_shader_cache_key",
            "shader cache key",
            bundle.shader_cache_key.as_str(),
        ),
        (
            "missing_shader_path",
            "shader path",
            bundle.shader_path.as_str(),
        ),
        (
            "missing_shader_identity",
            "shader identity",
            bundle.shader_identity.as_str(),
        ),
        (
            "missing_material_table_identity",
            "material table identity",
            bundle.material_table_identity.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                code,
                format!("scene material bundle is missing {label}"),
            ));
        }
    }

    if bundle.resource_layout_identity.trim().is_empty() {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::warning(
            "missing_resource_layout_identity",
            "scene material bundle has no resource layout identity",
        ));
    }
}

fn inspect_binding_slots(
    material: &PreparedMaterialFeatureContribution,
    diagnostics: &mut Vec<RenderMeshMaterialHandoffDiagnostic>,
) {
    for slot in &material.binding_table.slots {
        for (code, label, value) in [
            (
                "missing_binding_material_instance",
                "material instance id",
                slot.material_instance_id.as_str(),
            ),
            (
                "missing_formed_material_artifact",
                "formed material artifact id",
                slot.formed_material_artifact_id.as_str(),
            ),
            (
                "missing_binding_shader_artifact",
                "shader artifact id",
                slot.shader_artifact_id.as_str(),
            ),
            (
                "missing_material_cache_key",
                "material cache key",
                slot.material_cache_key.as_str(),
            ),
            (
                "missing_binding_shader_cache_key",
                "shader cache key",
                slot.shader_cache_key.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                    code,
                    format!(
                        "material binding slot {} is missing {label}",
                        slot.slot_index
                    ),
                ));
            }
        }
    }
}

fn inspect_model_mesh_selections(
    request: RenderMeshMaterialHandoffInspectionRequest<'_>,
    diagnostics: &mut Vec<RenderMeshMaterialHandoffDiagnostic>,
) {
    let material = request.prepared_material;
    if request.require_model_mesh_selection && material.model_mesh_material_selections.is_empty() {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_model_mesh_selection",
            "model/mesh material handoff requires at least one source-backed material selection",
        ));
    }

    for selection in &material.model_mesh_material_selections {
        inspect_model_mesh_selection(selection, diagnostics);
    }
}

fn inspect_model_mesh_selection(
    selection: &PreparedModelMeshMaterialSelection,
    diagnostics: &mut Vec<RenderMeshMaterialHandoffDiagnostic>,
) {
    if selection.surface.source.asset_id == 0 {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_model_mesh_asset_identity",
            "model/mesh material selection has no source asset identity",
        ));
    }
    if selection.surface.source.source_id == 0 {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_model_mesh_source_identity",
            "model/mesh material selection has no source identity",
        ));
    }
    if selection.surface.region_key.trim().is_empty() {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_model_mesh_region",
            "model/mesh material selection has no source region key",
        ));
    }
    if is_transient_model_mesh_material_region_key(&selection.surface.region_key) {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "transient_model_mesh_region",
            format!(
                "model/mesh material selection uses transient renderer region '{}'",
                selection.surface.region_key
            ),
        ));
    }
    if selection.material_table_index as usize
        >= PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT
    {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "model_mesh_table_index_limit",
            format!(
                "model/mesh material table index {} exceeds portable limit {}",
                selection.material_table_index, PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT
            ),
        ));
    }
    if selection.used_default_fallback {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::warning(
            "model_mesh_default_fallback",
            format!(
                "model/mesh material selection '{}' used default fallback",
                selection.surface.identity_key()
            ),
        ));
    }
}

fn inspect_material_passes(
    request: RenderMeshMaterialHandoffInspectionRequest<'_>,
    diagnostics: &mut Vec<RenderMeshMaterialHandoffDiagnostic>,
) {
    let material = request.prepared_material;
    let material_consuming_pass_count = request
        .material_passes
        .iter()
        .filter(|pass| pass.consumes_material_resources)
        .count();
    if request.require_material_consuming_pass && material_consuming_pass_count == 0 {
        diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
            "missing_material_consuming_pass",
            "prepared material handoff is not consumed by any material pass evidence",
        ));
    }

    for (index, pass) in request
        .material_passes
        .iter()
        .filter(|pass| pass.consumes_material_resources)
        .enumerate()
    {
        if !pass.prepared_material_available {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "material_pass_without_prepared_material",
                format!("material-consuming pass {index} reports no prepared material"),
            ));
        }
        if pass.material_instance_count != material.instances.len() {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "material_pass_instance_count_drift",
                format!(
                    "material-consuming pass {index} reports {} material instances, prepared handoff has {}",
                    pass.material_instance_count,
                    material.instances.len()
                ),
            ));
        }
        if pass.material_binding_slot_count != material.binding_table.slots.len() {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "material_pass_binding_count_drift",
                format!(
                    "material-consuming pass {index} reports {} binding slots, prepared handoff has {}",
                    pass.material_binding_slot_count,
                    material.binding_table.slots.len()
                ),
            ));
        }
        if pass.prepared_model_mesh_material_selection_count
            != material.model_mesh_material_selections.len()
        {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "material_pass_model_mesh_count_drift",
                format!(
                    "material-consuming pass {index} reports {} model/mesh selections, prepared handoff has {}",
                    pass.prepared_model_mesh_material_selection_count,
                    material.model_mesh_material_selections.len()
                ),
            ));
        }
        if request.require_model_mesh_selection
            && pass
                .model_mesh_material_selections_available_to_pass
                .is_empty()
        {
            diagnostics.push(RenderMeshMaterialHandoffDiagnostic::error(
                "model_mesh_selection_not_exposed_to_pass",
                format!("material-consuming pass {index} exposes no model/mesh selections"),
            ));
        }
    }
}

fn is_transient_model_mesh_material_region_key(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    [
        "renderable_index:",
        "draw_order:",
        "mesh_table_index:",
        "residency_slot:",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}
