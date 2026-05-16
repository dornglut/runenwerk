use super::*;

pub(super) struct AssetBrowserProvider;

impl EditorSurfaceProvider for AssetBrowserProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            ASSET_BROWSER_PROVIDER_ID,
            "Asset Browser",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::AssetBrowser
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model = context
            .app
            .asset_catalog_runtime()
            .asset_browser_view_model(context.app.asset_catalog_status_lines());
        let mut lines = vec![
            format!("assets: {}", view_model.rows.len()),
            format!("dirty assets: {}", view_model.dirty_asset_count),
        ];
        lines.extend(view_model.catalog_status_lines.clone());
        for row in &view_model.rows {
            let marker = if row.is_selected { "*" } else { " " };
            let dirty = if row.is_dirty { "dirty" } else { "current" };
            let preserved = if row.has_prior_valid_preservation {
                " prior-valid-preserved"
            } else {
                ""
            };
            lines.push(format!(
                "{marker} asset {} {} [{:?}] source={:?} artifacts={} {dirty}{preserved}",
                row.asset_id.raw(),
                row.display_name,
                row.kind,
                row.source_id.map(|id| id.raw()),
                row.artifact_count
            ));
        }
        if let Some(selected) = &view_model.selected {
            lines.push(format!(
                "selected: {} [{}] {:?}",
                selected.display_name, selected.stable_name, selected.kind
            ));
            lines.extend(selected.source_lines.iter().cloned());
            lines.extend(selected.artifact_lines.iter().cloned());
            lines.extend(selected.dependency_lines.iter().cloned());
        }
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        let mut actions = vec![
            (
                "Load catalog".to_string(),
                SurfaceLocalAction::Asset(AssetSurfaceAction::LoadProjectCatalog),
            ),
            (
                "Save catalog".to_string(),
                SurfaceLocalAction::Asset(AssetSurfaceAction::SaveProjectCatalog),
            ),
        ];
        actions.extend(view_model.rows.iter().map(|row| {
            (
                format!("Select {}", row.display_name),
                SurfaceLocalAction::Asset(AssetSurfaceAction::SelectAsset {
                    asset_id: row.asset_id,
                }),
            )
        }));
        if view_model.selected.is_some() {
            actions.push((
                "Reimport selected".to_string(),
                SurfaceLocalAction::Asset(AssetSurfaceAction::ReimportSelectedAsset),
            ));
        }
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );
        Ok(ProviderSurfaceFrame {
            title: "Asset Browser".to_string(),
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
        let SurfaceLocalAction::Asset(action) = action else {
            return Ok(None);
        };
        Ok(
            asset_surface_action_command(action, context.projection_epoch)
                .map(SurfaceCommandProposal::Shell),
        )
    }
}

pub(super) fn asset_surface_action_command(
    action: AssetSurfaceAction,
    projection_epoch: u64,
) -> Option<ShellCommand> {
    match action {
        AssetSurfaceAction::SelectAsset { asset_id } => Some(ShellCommand::SelectAsset {
            asset_id,
            projection_epoch,
        }),
        AssetSurfaceAction::LoadProjectCatalog => {
            Some(ShellCommand::LoadAssetCatalog { projection_epoch })
        }
        AssetSurfaceAction::SaveProjectCatalog => {
            Some(ShellCommand::SaveAssetCatalog { projection_epoch })
        }
        AssetSurfaceAction::ReimportAsset { asset_id } => Some(ShellCommand::ReimportAsset {
            asset_id,
            projection_epoch,
        }),
        AssetSurfaceAction::ReimportSelectedAsset => {
            Some(ShellCommand::ReimportSelectedAsset { projection_epoch })
        }
        AssetSurfaceAction::ClearDiagnostics => {
            Some(ShellCommand::ClearAssetDiagnostics { projection_epoch })
        }
    }
}
