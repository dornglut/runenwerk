use asset::{AssetDiagnosticCode, AssetDiagnosticRecord, AssetId};
use editor_scene::SceneMaterialSlotId;

use crate::editor_app::RunenwerkEditorApp;
use crate::material_lab::model_mesh_regions::resolve_catalog_model_mesh_material_region;

impl RunenwerkEditorApp {
    pub fn assign_model_mesh_material_region_slot(
        &mut self,
        model_asset_id: AssetId,
        material_region_key: &str,
        slot_id: SceneMaterialSlotId,
    ) -> Result<(), editor_core::EditorMutationError> {
        let catalog = self.asset_catalog_runtime().catalog();
        let region = match resolve_catalog_model_mesh_material_region(
            catalog,
            model_asset_id,
            material_region_key,
        ) {
            Ok(region) => region,
            Err(message) => {
                self.record_material_workflow_diagnostics([model_mesh_assignment_diagnostic(
                    model_asset_id,
                    material_region_key,
                    message,
                )]);
                return Err(editor_core::EditorMutationError::runtime_rejected(
                    "model mesh material region assignment is invalid",
                ));
            }
        };
        let scene_region = match region.scene_material_region() {
            Ok(scene_region) => scene_region,
            Err(message) => {
                self.record_material_workflow_diagnostics([model_mesh_assignment_diagnostic(
                    model_asset_id,
                    material_region_key,
                    message,
                )]);
                return Err(editor_core::EditorMutationError::runtime_rejected(
                    "model mesh material region identity is invalid",
                ));
            }
        };

        let mut assignments = self.runtime().scene_material_assignments().clone();
        if let Err(message) = assignments.assign_model_mesh_material_slot(scene_region, slot_id) {
            self.record_material_workflow_diagnostics([model_mesh_assignment_diagnostic(
                model_asset_id,
                material_region_key,
                message,
            )]);
            return Err(editor_core::EditorMutationError::runtime_rejected(
                "model mesh material region slot is invalid",
            ));
        }
        self.runtime_mut()
            .replace_scene_material_assignments(assignments);
        self.material_lab_runtime_mut().set_workflow_status(format!(
            "assigned mesh region {} to scene material slot {}",
            material_region_key,
            slot_id.raw()
        ));
        self.append_console_line(format!(
            "[material] assigned mesh asset {} region {} to slot {}",
            model_asset_id.raw(),
            material_region_key,
            slot_id.raw()
        ));
        Ok(())
    }
}

fn model_mesh_assignment_diagnostic(
    model_asset_id: AssetId,
    material_region_key: &str,
    message: impl Into<String>,
) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::error(AssetDiagnosticCode::RatificationRejected, message).with_subject(
        format!(
            "model_mesh.material_region:{}:{}",
            model_asset_id.raw(),
            material_region_key
        ),
    )
}
