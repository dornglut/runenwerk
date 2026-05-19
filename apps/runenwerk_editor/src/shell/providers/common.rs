use super::*;

pub(super) fn deterministic_provider(
    mut supported: Vec<&dyn EditorSurfaceProvider>,
) -> Option<&dyn EditorSurfaceProvider> {
    supported.sort_by_key(|provider| provider.descriptor().priority);
    let first = supported[0];
    if supported.len() == 1 {
        return Some(first);
    }
    if supported[0].descriptor().priority == supported[1].descriptor().priority {
        return None;
    }
    Some(first)
}

pub(super) fn unsupported_frame(
    request: &SurfaceProviderRequest,
    title: &str,
    code: &'static str,
    message: &str,
) -> ResolvedSurfaceFrame {
    diagnostic_frame(
        request,
        title,
        SurfaceProviderAvailability::Unsupported,
        SurfacePresentationArtifactKind::Unsupported,
        SurfaceProviderDiagnostic::new(code, message),
    )
}

pub(super) fn diagnostic_frame(
    request: &SurfaceProviderRequest,
    title: impl Into<String>,
    availability: SurfaceProviderAvailability,
    kind: SurfacePresentationArtifactKind,
    diagnostic: SurfaceProviderDiagnostic,
) -> ResolvedSurfaceFrame {
    let root = diagnostic_surface_node(request, &diagnostic);
    ResolvedSurfaceFrame::diagnostic(
        request,
        title,
        availability,
        SurfacePresentationArtifact::diagnostic(kind, root, diagnostic),
    )
}

pub(super) fn diagnostic_surface_node(
    request: &SurfaceProviderRequest,
    diagnostic: &SurfaceProviderDiagnostic,
) -> editor_shell::UiNode {
    let root_id = surface_widget_id(
        request.tool_surface_instance_id,
        editor_shell::WidgetId(900_000),
    );
    let label_id = surface_widget_id(
        request.tool_surface_instance_id,
        editor_shell::WidgetId(900_001),
    );
    editor_shell::panel(
        root_id,
        ThemeTokens::default(),
        vec![editor_shell::label(
            label_id,
            format!("Unsupported surface: {}", diagnostic.message),
            ThemeTokens::default().body_small_text_style(FontId(1)),
        )],
    )
}

