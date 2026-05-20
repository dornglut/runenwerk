use super::*;

const MATERIAL_PREVIEW_ROOT_WIDGET_ID: WidgetId = WidgetId(53_000);
const MATERIAL_PREVIEW_SCROLL_WIDGET_ID: WidgetId = WidgetId(53_001);
const MATERIAL_PREVIEW_BODY_WIDGET_ID: WidgetId = WidgetId(53_002);
const MATERIAL_PREVIEW_SURFACE_WIDGET_ID: WidgetId = WidgetId(53_003);
const MATERIAL_PREVIEW_CONTROLS_WIDGET_ID: WidgetId = WidgetId(53_004);
const MATERIAL_PREVIEW_LINE_WIDGET_ID_BASE: u64 = 53_100;
const MATERIAL_PREVIEW_ACTION_WIDGET_ID_BASE: u64 = 53_500;

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
            .preview_view_model_with_scene_material_assignments(
                context.app.asset_catalog_runtime().catalog(),
                Some(context.app.runtime().scene_material_assignments()),
            );
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
        lines.push("scene material binding: SDF primitives use scene material slots".to_string());
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

        let (root, routes) =
            build_material_preview_panel(context.theme, request, &view_model, lines, actions);

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

fn build_material_preview_panel(
    theme: &ThemeTokens,
    request: &SurfaceProviderRequest,
    view_model: &MaterialPreviewViewModel,
    mut lines: Vec<String>,
    actions: Vec<(String, SurfaceLocalAction)>,
) -> (UiNode, SurfaceRouteTable) {
    let scope = editor_shell::SurfaceWidgetScope::new(request.tool_surface_instance_id);
    let text_style = theme.body_small_text_style(FontId(1));
    let mut routes = SurfaceRouteTable::empty();
    let mut body_children = Vec::new();

    if let Some(surface) = &view_model.preview_surface {
        body_children.push(editor_shell::product_surface(
            scope.widget_id(MATERIAL_PREVIEW_SURFACE_WIDGET_ID),
            surface.source.clone(),
            ui_math::UiSize::new(surface.width.max(1) as f32, surface.height.max(1) as f32),
        ));
        lines.push("material preview scene: rendered GPU product-surface preview".to_string());
        lines.push(format!(
            "material preview scene target: {}",
            surface.target_label
        ));
        lines.push(format!(
            "material preview scene bind group identity: {}",
            surface.bind_group_identity
        ));
    } else {
        lines.push(
            "material preview scene: unavailable until a material preview product is active"
                .to_string(),
        );
    }

    for (index, line) in lines.into_iter().enumerate() {
        body_children.push(editor_shell::label(
            scope.widget_id(WidgetId(
                MATERIAL_PREVIEW_LINE_WIDGET_ID_BASE + index as u64,
            )),
            line,
            text_style.clone(),
        ));
    }

    if !actions.is_empty() {
        let mut action_nodes = Vec::with_capacity(actions.len());
        for (index, (label, action)) in actions.into_iter().enumerate() {
            let widget_id = scope.widget_id(WidgetId(
                MATERIAL_PREVIEW_ACTION_WIDGET_ID_BASE + index as u64,
            ));
            action_nodes.push(editor_shell::compact_surface_action_button(
                widget_id, label, false, true, theme,
            ));
            routes.insert(widget_id, SurfaceLocalRoute::new(action));
        }
        body_children.push(editor_shell::hstack(
            scope.widget_id(MATERIAL_PREVIEW_CONTROLS_WIDGET_ID),
            theme.spacing.xs,
            action_nodes,
        ));
    }

    let body = editor_shell::vstack(
        scope.widget_id(MATERIAL_PREVIEW_BODY_WIDGET_ID),
        theme.spacing.xs,
        body_children,
    );
    let scroll = editor_shell::vscroll(
        scope.widget_id(MATERIAL_PREVIEW_SCROLL_WIDGET_ID),
        theme.clone(),
        vec![body],
    );
    (
        editor_shell::panel(
            scope.widget_id(MATERIAL_PREVIEW_ROOT_WIDGET_ID),
            theme.clone(),
            vec![scroll],
        ),
        routes,
    )
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
