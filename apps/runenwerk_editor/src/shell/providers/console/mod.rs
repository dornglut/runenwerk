//! File: apps/runenwerk_editor/src/shell/providers/console/mod.rs
//! Purpose: Console surface provider.

use super::*;

pub struct ConsoleProvider;

impl EditorSurfaceProvider for ConsoleProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            CONSOLE_PROVIDER_ID,
            "Console",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_support(request, EDITOR_CONSOLE_SURFACE_KEY)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model: ConsoleViewModel = build_console_view_model(context.app.console_lines());
        let root = build_console_panel(
            &view_model,
            context.theme,
            request.panel_instance_id,
            Some(request.tool_surface_instance_id),
        );
        Ok(ProviderSurfaceFrame {
            title: "Console".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes: SurfaceRouteTable::empty(),
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        Ok(None)
    }
}
