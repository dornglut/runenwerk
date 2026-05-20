use super::model_mesh_preview_projection::material_model_mesh_preview_view_model;
use super::*;

impl MaterialLabRuntime {
    pub fn preview_view_model(&self, catalog: &AssetCatalog) -> MaterialPreviewViewModel {
        self.preview_view_model_with_scene_material_assignments(catalog, None)
    }

    pub fn preview_view_model_with_scene_material_assignments(
        &self,
        catalog: &AssetCatalog,
        scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    ) -> MaterialPreviewViewModel {
        let preview_status_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No active material preview product".to_string()],
            |preview| {
                vec![
                    format!("material product: {}", preview.product.product_id.raw()),
                    format!("artifact: {}", preview.artifact_id.raw()),
                    format!("shader artifact: {}", preview.shader_artifact_id.raw()),
                    format!(
                        "scene shader artifact: {}",
                        preview.scene_shader_artifact_id.raw()
                    ),
                    format!("viewport product: {}", preview.viewport_product_id.0),
                    format!("cache: {}", preview.artifact_cache_key.as_str()),
                    format!("shader cache: {}", preview.shader_cache_key.as_str()),
                    format!(
                        "scene shader cache: {}",
                        preview.scene_shader_cache_key.as_str()
                    ),
                ]
            },
        );
        let mut diagnostic_rows = self.material_diagnostic_rows();
        diagnostic_rows.extend(self.preview_scene_product_diagnostic_rows());
        MaterialPreviewViewModel {
            selected_asset_id: self.selected_material_asset_id,
            active_product_id: self
                .active_preview
                .as_ref()
                .map(EditorMaterialPreviewProduct::product_id),
            artifact_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.artifact_id),
            viewport_product_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.viewport_product_id),
            specialization_fragment: self
                .active_preview
                .as_ref()
                .map(|preview| preview.product.specialization_fragment.0.clone()),
            prepared_parameter_payload_bytes: self
                .active_preview
                .as_ref()
                .map(|preview| material_parameter_payload(preview).encoded_len())
                .unwrap_or(0),
            preview_surface: self
                .active_preview
                .as_ref()
                .map(material_preview_scene_surface_view_model),
            preview_status: self.material_preview_status_view_model(),
            model_mesh_preview: material_model_mesh_preview_view_model(
                catalog,
                scene_material_assignments,
            ),
            diagnostic_rows,
            resource_binding_diagnostics: self.material_resource_binding_diagnostic_rows(catalog),
            preview_status_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    fn material_preview_status_view_model(&self) -> MaterialPreviewStatusViewModel {
        let product_for_lineage = self
            .current_preview_scene_product()
            .or_else(|| self.last_valid_preview_scene_product());
        let last_publication = self.publication_journal.last();
        let publication_status = last_publication
            .map(|entry| material_preview_publication_status_kind(entry.status))
            .unwrap_or(MaterialPreviewPublicationStatusKind::NoPublication);
        let active_preview_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("material product {}", preview.product_id().raw()));
        let failed_preserved_last_good = last_publication
            .is_some_and(|entry| entry.status == ProductPublicationStatus::FailedPreserved);
        let last_good_available = self.active_preview.is_some() || failed_preserved_last_good;
        let active_product_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("material product {}", preview.product_id().raw()));
        let material_artifact_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("material artifact {}", preview.artifact_id.raw()))
            .or_else(|| {
                last_publication
                    .map(|entry| format!("last publication artifact {}", entry.artifact_id.raw()))
            });
        let shader_artifact_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("shader artifact {}", preview.shader_artifact_id.raw()));
        let scene_shader_artifact_label = self.active_preview.as_ref().map(|preview| {
            format!(
                "scene shader artifact {}",
                preview.scene_shader_artifact_id.raw()
            )
        });
        let viewport_product_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("viewport product {}", preview.viewport_product_id.0));
        let last_publication_label = last_publication.map(|entry| {
            let product = entry
                .product_id
                .map(|product_id| product_id.raw().to_string())
                .unwrap_or_else(|| "none".to_string());
            format!(
                "{:?} artifact {} product {}",
                entry.status,
                entry.artifact_id.raw(),
                product
            )
        });
        let last_good_reason = if self.active_preview.is_some() {
            Some("active material preview product is available".to_string())
        } else if failed_preserved_last_good {
            Some("last publication preserved a prior valid material artifact".to_string())
        } else {
            None
        };

        let mut detail_lines = Vec::new();
        if let Some(asset_id) = self.selected_material_asset_id {
            detail_lines.push(format!("selected material asset: {}", asset_id.raw()));
        }
        if let Some(status) = &self.last_workflow_status {
            detail_lines.push(format!("last material workflow: {status}"));
        }
        if let Some(entry) = last_publication {
            detail_lines.push(format!(
                "last publication: {:?} artifact {} product {:?}",
                entry.status,
                entry.artifact_id.raw(),
                entry.product_id.map(|product_id| product_id.raw())
            ));
        }
        if let Some(preview) = &self.active_preview {
            detail_lines.push(format!("active artifact: {}", preview.artifact_id.raw()));
            detail_lines.push(format!(
                "viewport product: {}",
                preview.viewport_product_id.0
            ));
            detail_lines.push(format!(
                "shader artifact: {}",
                preview.shader_artifact_id.raw()
            ));
            detail_lines.push(format!(
                "scene shader artifact: {}",
                preview.scene_shader_artifact_id.raw()
            ));
        }

        let preview_scene_product_status_label = Some(preview_scene_product_status_label(
            self.preview_scene_product_status(),
        ));
        let preview_scene_product_identity =
            product_for_lineage.map(|product| product.product_identity.clone());
        let preview_scene_product_mode_label =
            product_for_lineage.map(|product| preview_scene_product_mode_label(product.mode));
        let material_table_identity_label = product_for_lineage
            .map(|product| format!("material table {}", product.material_table_identity));
        let resource_layout_identity_label = product_for_lineage
            .map(|product| format!("resource layout {}", product.resource_layout_identity));
        let preview_scene_product_shader_identity_label = product_for_lineage
            .map(|product| format!("shader identity {}", product.shader.shader_identity));
        let preview_scene_product_shader_artifact_label = product_for_lineage.map(|product| {
            format!(
                "shader artifact {} cache {}",
                product.shader.shader_artifact_id,
                product.shader.shader_cache_key.as_str()
            )
        });
        let slot_count = product_for_lineage.map(|product| product.slots.len());
        let resource_slot_count = product_for_lineage.map(|product| product.resources.len());
        let last_valid_preview_scene_product_identity = self
            .last_valid_preview_scene_product()
            .map(|product| product.product_identity.clone());
        let preview_scene_product_failure_reason = self.preview_scene_product_failure_reason();

        let (status, headline) = if self.selected_material_asset_id.is_none() {
            (
                MaterialPreviewStatusKind::NoSelection,
                "No material asset selected".to_string(),
            )
        } else if last_publication
            .is_some_and(|entry| entry.status == ProductPublicationStatus::FailedPreserved)
        {
            (
                MaterialPreviewStatusKind::FailedPreservedLastGood,
                "Preview build failed; prior valid material remains available".to_string(),
            )
        } else if self
            .last_workflow_status
            .as_deref()
            .is_some_and(|status| status.contains("publication queued"))
        {
            (
                MaterialPreviewStatusKind::Queued,
                "Material preview publication queued".to_string(),
            )
        } else if self
            .last_workflow_status
            .as_deref()
            .is_some_and(|status| status.contains("build blocked"))
        {
            (
                MaterialPreviewStatusKind::Blocked,
                "Material preview build is blocked".to_string(),
            )
        } else if self.active_preview.is_some() {
            (
                MaterialPreviewStatusKind::Published,
                "Material preview product is active".to_string(),
            )
        } else if self.active_source_document.is_none() {
            (
                MaterialPreviewStatusKind::NoSourceDocument,
                "No material source document is loaded".to_string(),
            )
        } else {
            (
                MaterialPreviewStatusKind::NoActivePreview,
                "No active material preview product".to_string(),
            )
        };

        MaterialPreviewStatusViewModel {
            status,
            headline,
            detail_lines,
            last_good_available,
            active_preview_label,
            publication_status,
            product_status_label: Some(material_preview_product_status_label(
                status,
                publication_status,
                self.active_preview.is_some(),
                failed_preserved_last_good,
            )),
            last_publication_label,
            last_good_reason,
            failed_preserved_last_good,
            active_product_label,
            material_artifact_label,
            shader_artifact_label,
            scene_shader_artifact_label,
            viewport_product_label,
            preview_scene_product_identity,
            preview_scene_product_mode_label,
            preview_scene_product_status_label,
            material_table_identity_label,
            resource_layout_identity_label,
            preview_scene_product_shader_identity_label,
            preview_scene_product_shader_artifact_label,
            slot_count,
            resource_slot_count,
            last_valid_preview_scene_product_identity,
            preview_scene_product_failure_reason,
            diagnostic_count: self.diagnostics.len(),
        }
    }
}

fn material_preview_publication_status_kind(
    status: ProductPublicationStatus,
) -> MaterialPreviewPublicationStatusKind {
    match status {
        ProductPublicationStatus::Ready => MaterialPreviewPublicationStatusKind::Ready,
        ProductPublicationStatus::FailedPreserved => {
            MaterialPreviewPublicationStatusKind::FailedPreserved
        }
        ProductPublicationStatus::Rejected => MaterialPreviewPublicationStatusKind::Rejected,
    }
}

fn material_preview_product_status_label(
    status: MaterialPreviewStatusKind,
    publication_status: MaterialPreviewPublicationStatusKind,
    active_preview_available: bool,
    failed_preserved_last_good: bool,
) -> String {
    if active_preview_available {
        "active material preview product ready".to_string()
    } else if failed_preserved_last_good {
        "prior valid material artifact preserved".to_string()
    } else if publication_status != MaterialPreviewPublicationStatusKind::NoPublication {
        format!("last publication status: {publication_status:?}")
    } else {
        format!("preview status: {status:?}")
    }
}

fn preview_scene_product_status_label(status: PreviewSceneProductRuntimeStatus) -> String {
    match status {
        PreviewSceneProductRuntimeStatus::Empty => "no preview scene product".to_string(),
        PreviewSceneProductRuntimeStatus::Current { product_identity } => {
            format!("current preview scene product available ({product_identity})")
        }
        PreviewSceneProductRuntimeStatus::LastValidPreserved { product_identity } => {
            format!("last-valid preview scene product preserved ({product_identity})")
        }
    }
}

fn preview_scene_product_mode_label(mode: PreviewSceneProductMode) -> String {
    match mode {
        PreviewSceneProductMode::SingleMaterial => "single material".to_string(),
        PreviewSceneProductMode::SceneMaterialTable => "scene material table".to_string(),
    }
}

pub fn material_artifact_lines(catalog: &AssetCatalog) -> Vec<String> {
    let mut lines = catalog
        .artifacts
        .values()
        .filter_map(|artifact| match &artifact.payload_kind {
            ArtifactPayloadKind::FormedMaterialProduct { product_id } => Some(format!(
                "formed material artifact {} product={} validity={:?} cache={}",
                artifact.artifact_id.raw(),
                product_id,
                artifact.validity,
                artifact.cache_key.as_str()
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push("No formed material artifacts".to_string());
    }
    lines
}

pub fn previous_valid_material_artifact<'a>(
    catalog: &'a AssetCatalog,
    asset_id: AssetId,
) -> Option<&'a asset::AssetArtifactDescriptor> {
    let record = catalog.asset(asset_id)?;
    record
        .artifact_ids
        .iter()
        .rev()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .find(|artifact| {
            artifact.kind == AssetKind::Material && artifact.validity == ArtifactValidity::Valid
        })
}
