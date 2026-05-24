use super::*;

pub(super) struct ImportInspectorProvider;

impl EditorSurfaceProvider for ImportInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            IMPORT_INSPECTOR_PROVIDER_ID,
            "Import Inspector",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_support(request, IMPORT_INSPECTOR_SURFACE_KEY)
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
            .import_inspector_view_model(context.app.asset_catalog_status_lines());
        let mut lines = vec![format!(
            "dirty assets: {}",
            view_model.pending_dirty_asset_ids.len()
        )];
        lines.extend(view_model.catalog_status_lines.clone());
        lines.extend(view_model.plan_lines.clone());
        lines.extend(view_model.diagnostic_lines.clone());
        lines.extend(view_model.prior_valid_lines.clone());
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
            (
                "Clear diagnostics".to_string(),
                SurfaceLocalAction::Asset(AssetSurfaceAction::ClearDiagnostics),
            ),
        ];
        if view_model.selected_asset_id.is_some() {
            actions.push((
                "Reimport selected".to_string(),
                SurfaceLocalAction::Asset(AssetSurfaceAction::ReimportSelectedAsset),
            ));
        }
        actions.extend(view_model.pending_dirty_asset_ids.iter().map(|asset_id| {
            (
                format!("Reimport dirty {}", asset_id.raw()),
                SurfaceLocalAction::Asset(AssetSurfaceAction::ReimportAsset {
                    asset_id: *asset_id,
                }),
            )
        }));
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );
        Ok(ProviderSurfaceFrame {
            title: "Import Inspector".to_string(),
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
            super::asset_browser::asset_surface_action_command(action, context.projection_epoch)
                .map(SurfaceCommandProposal::Shell),
        )
    }
}
