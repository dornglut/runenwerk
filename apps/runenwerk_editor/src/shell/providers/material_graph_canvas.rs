use super::*;

pub(super) struct MaterialGraphCanvasProvider;

impl EditorSurfaceProvider for MaterialGraphCanvasProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
            "Material Graph Canvas",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::MaterialGraphCanvas
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model = context.app.material_lab_runtime().graph_canvas_view_model(
            context.app.asset_catalog_runtime().catalog(),
            context.app.asset_catalog_status_lines(),
        );
        let mut lines = vec![
            "material graph canvas: source-backed MaterialGraphDocument workflow".to_string(),
            surface_document_context_line(&request.document_context),
            "canvas state is projection only; domain/material_graph remains material truth"
                .to_string(),
            format!("material graph assets: {}", view_model.rows.len()),
        ];
        lines.extend(view_model.catalog_status_lines.clone());
        for row in &view_model.rows {
            let marker = if row.is_selected { "*" } else { " " };
            let preserved = if row.has_prior_valid_preservation {
                " prior-valid-preserved"
            } else {
                ""
            };
            lines.push(format!(
                "{marker} material asset {} {} [{}] source={:?} artifacts={}{}",
                row.asset_id.raw(),
                row.display_name,
                row.stable_name,
                row.source_id.map(|source_id| source_id.raw()),
                row.artifact_count,
                preserved
            ));
        }
        if let Some(selected) = &view_model.selected {
            lines.push(format!(
                "selected material asset {} source={:?} path={}",
                selected.asset_id.raw(),
                selected.source_id.map(|source_id| source_id.raw()),
                selected.source_path.as_deref().unwrap_or("none")
            ));
            if let Some(document_id) = selected.document_id {
                lines.push(format!("document id: {}", document_id.raw()));
            }
            if let Some(output_target) = selected.output_target {
                lines.push(format!("output target: {output_target:?}"));
            }
            lines.push(format!("source-map nodes: {}", selected.node_count));
        }
        lines.extend(view_model.diagnostic_lines.clone());
        lines.extend(crate::material_lab::material_artifact_lines(
            context.app.asset_catalog_runtime().catalog(),
        ));
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());

        let mut actions = Vec::new();
        actions.extend(view_model.rows.iter().flat_map(|row| {
            [
                (
                    format!("Select {}", row.display_name),
                    SurfaceLocalAction::Material(MaterialSurfaceAction::SelectMaterialAsset {
                        asset_id: row.asset_id,
                    }),
                ),
                (
                    format!("Build {}", row.display_name),
                    SurfaceLocalAction::Material(MaterialSurfaceAction::BuildMaterialPreview {
                        asset_id: row.asset_id,
                    }),
                ),
            ]
        }));
        if view_model.selected.is_some() {
            actions.push((
                "Build selected preview".to_string(),
                SurfaceLocalAction::Material(MaterialSurfaceAction::BuildSelectedMaterialPreview),
            ));
        }
        actions.push((
            "Clear material diagnostics".to_string(),
            SurfaceLocalAction::Material(MaterialSurfaceAction::ClearMaterialDiagnostics),
        ));

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );

        Ok(ProviderSurfaceFrame {
            title: "Material Graph Canvas".to_string(),
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
            material_surface_action_command(action, context.projection_epoch)
                .map(SurfaceCommandProposal::Shell),
        )
    }
}

pub(super) fn material_surface_action_command(
    action: MaterialSurfaceAction,
    projection_epoch: u64,
) -> Option<ShellCommand> {
    match action {
        MaterialSurfaceAction::SelectMaterialAsset { asset_id } => {
            Some(ShellCommand::SelectMaterialAsset {
                asset_id,
                projection_epoch,
            })
        }
        MaterialSurfaceAction::BuildMaterialPreview { asset_id } => {
            Some(ShellCommand::BuildMaterialPreview {
                asset_id,
                projection_epoch,
            })
        }
        MaterialSurfaceAction::BuildSelectedMaterialPreview => {
            Some(ShellCommand::BuildSelectedMaterialPreview { projection_epoch })
        }
        MaterialSurfaceAction::ClearMaterialDiagnostics => {
            Some(ShellCommand::ClearMaterialDiagnostics { projection_epoch })
        }
    }
}
