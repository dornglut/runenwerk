use super::*;

pub(super) struct MaterialPreviewProvider;

impl EditorSurfaceProvider for MaterialPreviewProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            MATERIAL_PREVIEW_PROVIDER_ID,
            "Material Preview",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_or_legacy_kind_support(
            request,
            MATERIAL_PREVIEW_SURFACE_KEY,
            ToolSurfaceKind::MaterialPreview,
        )
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model = context
            .app
            .material_lab_runtime()
            .preview_view_model(context.app.asset_catalog_runtime().catalog());
        let mut lines = vec![
            "material preview: catalog-backed product and prepared renderer handoff".to_string(),
            surface_document_context_line(&request.document_context),
            "preview output fails closed when no formed material product is available".to_string(),
        ];
        if let Some(product_id) = view_model.active_product_id {
            lines.push(format!("active material product: {}", product_id.raw()));
        }
        if let Some(artifact_id) = view_model.artifact_id {
            lines.push(format!("artifact: {}", artifact_id.raw()));
        }
        if let Some(viewport_product_id) = view_model.viewport_product_id {
            lines.push(format!("viewport product: {}", viewport_product_id.0));
        }
        if let Some(fragment) = &view_model.specialization_fragment {
            lines.push(format!("specialization: {fragment}"));
        }
        lines.push(format!(
            "prepared parameter payload bytes: {}",
            view_model.prepared_parameter_payload_bytes
        ));
        lines.extend(material_preview_status_lines(&view_model.preview_status));
        lines.extend(super::material_graph_canvas::material_diagnostic_row_lines(
            &view_model.diagnostic_rows,
        ));
        lines.extend(
            super::material_graph_canvas::material_resource_binding_diagnostic_lines(
                &view_model.resource_binding_diagnostics,
            ),
        );
        lines.extend(view_model.preview_status_lines.clone());
        lines.extend(view_model.diagnostic_lines.clone());
        lines.extend(crate::material_lab::material_artifact_lines(
            context.app.asset_catalog_runtime().catalog(),
        ));
        lines.extend(context.app.asset_catalog_runtime().texture_product_lines());
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        let mut actions = vec![(
            "Clear material diagnostics".to_string(),
            SurfaceLocalAction::Material(MaterialSurfaceAction::ClearMaterialDiagnostics),
        )];
        if view_model.selected_asset_id.is_some() {
            actions.push((
                "Build selected preview".to_string(),
                SurfaceLocalAction::Material(MaterialSurfaceAction::BuildSelectedMaterialPreview),
            ));
        }

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );

        Ok(ProviderSurfaceFrame {
            title: "Material Preview".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let SurfaceLocalAction::Material(action) = action else {
            return Ok(None);
        };
        Ok(
            super::material_graph_canvas::material_surface_action_command(
                action,
                context.projection_epoch,
            )
            .map(SurfaceCommandProposal::Shell),
        )
    }
}

fn material_preview_status_lines(status: &MaterialPreviewStatusViewModel) -> Vec<String> {
    let mut lines = vec![format!(
        "material preview status [{:?}]: {}",
        status.status, status.headline
    )];
    lines.push(format!(
        "last good material preview available: {}",
        status.last_good_available
    ));
    lines.push(format!(
        "material preview publication status: {:?}",
        status.publication_status
    ));
    if let Some(label) = &status.product_status_label {
        lines.push(format!("material preview product status: {label}"));
    }
    if let Some(label) = &status.last_publication_label {
        lines.push(format!("last material preview publication: {label}"));
    }
    if let Some(reason) = &status.last_good_reason {
        lines.push(format!("last good material preview reason: {reason}"));
    }
    lines.push(format!(
        "failed preview preserved last good: {}",
        status.failed_preserved_last_good
    ));
    lines.push(format!(
        "material preview diagnostic count: {}",
        status.diagnostic_count
    ));
    lines.push("Preview Scene Product".to_string());
    if let Some(label) = &status.preview_scene_product_status_label {
        lines.push(format!("preview scene product status: {label}"));
    }
    if let Some(label) = &status.preview_scene_product_mode_label {
        lines.push(format!("preview scene product mode: {label}"));
    }
    if let Some(identity) = &status.preview_scene_product_identity {
        lines.push(format!("preview scene product: {identity}"));
    }
    if let Some(label) = &status.material_table_identity_label {
        lines.push(format!("preview scene product material table: {label}"));
    }
    if let Some(label) = &status.resource_layout_identity_label {
        lines.push(format!("preview scene product resource layout: {label}"));
    }
    if let Some(label) = &status.preview_scene_product_shader_identity_label {
        lines.push(format!("preview scene product shader: {label}"));
    }
    if let Some(label) = &status.preview_scene_product_shader_artifact_label {
        lines.push(format!("preview scene product shader artifact: {label}"));
    }
    if let Some(slot_count) = status.slot_count {
        lines.push(format!("preview scene product slots: {slot_count}"));
    }
    if let Some(resource_slot_count) = status.resource_slot_count {
        lines.push(format!(
            "preview scene product resources: {resource_slot_count}"
        ));
    }
    if let Some(identity) = &status.last_valid_preview_scene_product_identity {
        lines.push(format!("preview scene product last valid: {identity}"));
    }
    if let Some(reason) = &status.preview_scene_product_failure_reason {
        lines.push(format!("preview scene product failure: {reason}"));
    }
    if let Some(label) = &status.active_preview_label {
        lines.push(format!("active preview: {label}"));
    }
    if let Some(label) = &status.active_product_label {
        lines.push(format!("active material product label: {label}"));
    }
    if let Some(label) = &status.material_artifact_label {
        lines.push(format!("material preview artifact label: {label}"));
    }
    if let Some(label) = &status.shader_artifact_label {
        lines.push(format!("material preview shader artifact label: {label}"));
    }
    if let Some(label) = &status.scene_shader_artifact_label {
        lines.push(format!(
            "material preview scene shader artifact label: {label}"
        ));
    }
    if let Some(label) = &status.viewport_product_label {
        lines.push(format!("material preview viewport product label: {label}"));
    }
    lines.extend(
        status
            .detail_lines
            .iter()
            .map(|line| format!("preview status detail: {line}")),
    );
    lines
}
