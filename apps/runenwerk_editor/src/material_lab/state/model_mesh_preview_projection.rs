use super::*;
use crate::material_lab::model_mesh_regions::catalog_model_mesh_material_regions;

pub(super) fn material_model_mesh_preview_view_model(
    catalog: &AssetCatalog,
    assignments: Option<&SceneMaterialAssignmentState>,
) -> MaterialModelMeshPreviewViewModel {
    let regions = catalog_model_mesh_material_regions(catalog)
        .into_iter()
        .map(|region| {
            let source_region = region.scene_material_region();
            let assignment = assignments.and_then(|assignments| {
                let source_region = source_region.as_ref().ok()?;
                assignments
                    .model_mesh_assignments()
                    .find(|assignment| assignment.material_region == *source_region)
            });
            let assigned_slot_id = assignment.as_ref().map(|assignment| assignment.slot_id);
            let assigned_slot_label = assigned_slot_id.and_then(|slot_id| {
                assignments?
                    .palette()
                    .slots
                    .iter()
                    .find(|slot| slot.slot_id == slot_id)
                    .map(|slot| slot.display_name.clone())
            });
            let resolution = assignments.and_then(|assignments| {
                let source_region = source_region.as_ref().ok()?;
                Some(assignments.resolve_material_binding_for_model_mesh_region(source_region))
            });
            let resolution_diagnostic = resolution.as_ref().and_then(|(_, diagnostics)| {
                diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.as_str())
                    .next()
                    .map(str::to_string)
            });
            let source_identity_diagnostic = source_region.as_ref().err().cloned();
            let diagnostic = region
                .display_diagnostic()
                .or(source_identity_diagnostic)
                .or(resolution_diagnostic);
            let valid = region.is_assignable() && source_region.is_ok();
            let (requested_slot_id, resolved_slot_id, material_table_index, used_default_fallback) =
                resolution
                    .map(|(resolution, _)| {
                        (
                            Some(resolution.requested_slot_id),
                            Some(resolution.resolved_slot_id),
                            Some(resolution.material_table_index),
                            resolution.used_default_fallback,
                        )
                    })
                    .unwrap_or((None, None, None, false));

            MaterialModelMeshPreviewRegionViewModel {
                asset_id: region.asset_id,
                stable_name: region.asset_stable_name,
                asset_display_name: region.asset_display_name,
                artifact_id: region.artifact_id,
                source_id: region.source_id,
                source_revision_id: region.source_revision_id,
                source_revision: region.source_revision,
                material_region_key: region.material_region_key,
                material_region_label: region.display_name,
                assigned_slot_id,
                assigned_slot_label,
                requested_slot_id,
                resolved_slot_id,
                material_table_index,
                used_default_fallback,
                valid,
                diagnostic,
            }
        })
        .collect::<Vec<_>>();

    let source_backed_region_count = regions
        .iter()
        .filter(|region| region.source_id.is_some())
        .count();
    let assignable_region_count = regions.iter().filter(|region| region.valid).count();
    let prepared_region_count = regions
        .iter()
        .filter(|region| region.material_table_index.is_some())
        .count();
    let assigned_region_count = regions
        .iter()
        .filter(|region| region.assigned_slot_id.is_some())
        .count();
    let diagnostic_count = regions
        .iter()
        .filter(|region| region.diagnostic.is_some())
        .count();

    let (status, headline) = if regions.is_empty() {
        (
            MaterialModelMeshPreviewStatusKind::NoModelMeshRegions,
            "No source-backed model/mesh material regions are available".to_string(),
        )
    } else if assignments.is_none() {
        (
            MaterialModelMeshPreviewStatusKind::WaitingForSceneMaterialAssignments,
            "Model/mesh regions found; scene material assignments are not available to prepare preview selections".to_string(),
        )
    } else if prepared_region_count > 0 {
        (
            MaterialModelMeshPreviewStatusKind::Ready,
            "Source-backed Material Lab Mesh Preview product surface can prepare renderer selections".to_string(),
        )
    } else {
        (
            MaterialModelMeshPreviewStatusKind::Blocked,
            "Model/mesh regions are present but none can prepare renderer selections".to_string(),
        )
    };

    MaterialModelMeshPreviewViewModel {
        status,
        headline,
        source_backed_region_count,
        assignable_region_count,
        prepared_region_count,
        assigned_region_count,
        diagnostic_count,
        regions,
    }
}
