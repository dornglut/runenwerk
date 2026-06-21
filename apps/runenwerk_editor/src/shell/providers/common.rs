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

pub(super) fn diagnostic_frame(
    request: &SurfaceProviderRequest,
    title: impl Into<String>,
    availability: SurfaceProviderAvailability,
    kind: SurfacePresentationArtifactKind,
    content_liveness: ContentLiveness,
    content_fallback: ContentProjectionFallback,
    diagnostic: SurfaceProviderDiagnostic,
) -> ResolvedSurfaceFrame {
    let diagnostic = diagnostic.for_mounted_unit(request.mounted_unit_id);
    let root = diagnostic_surface_node(request, &diagnostic);
    ResolvedSurfaceFrame::diagnostic(
        request,
        title,
        availability,
        content_liveness,
        content_fallback,
        SurfacePresentationArtifact::diagnostic(kind, root, diagnostic),
    )
}

pub(super) fn unavailable_content_frame(
    request: &SurfaceProviderRequest,
    liveness: ContentLiveness,
    app_projection: Option<(SurfaceProviderDescriptor, ProviderSurfaceFrame)>,
) -> ResolvedSurfaceFrame {
    if let Some((descriptor, frame)) = app_projection {
        return ResolvedSurfaceFrame {
            mounted_unit_id: request.mounted_unit_id,
            content_liveness: liveness,
            content_fallback: ContentProjectionFallback::AppProvidedUnavailable,
            surface_instance_id: request.tool_surface_instance_id,
            panel_instance_id: request.panel_instance_id,
            tab_stack_id: request.tab_stack_id,
            stable_surface_key: request.stable_surface_key.clone(),
            surface_definition_id: request.surface_definition_id,
            provider_id: Some(descriptor.id),
            title: frame.title,
            artifact: frame.artifact,
            routes: frame.routes,
            availability: availability_for_liveness(liveness),
        };
    }
    unavailable_content_frame_with_diagnostic(
        request,
        liveness,
        SurfaceProviderDiagnostic::new(
            liveness_diagnostic_code(liveness),
            liveness_actionable_message(liveness),
        ),
    )
}

pub(super) fn unavailable_content_frame_with_diagnostic(
    request: &SurfaceProviderRequest,
    liveness: ContentLiveness,
    diagnostic: SurfaceProviderDiagnostic,
) -> ResolvedSurfaceFrame {
    diagnostic_frame(
        request,
        liveness_title(liveness),
        availability_for_liveness(liveness),
        if liveness == ContentLiveness::Crashed {
            SurfacePresentationArtifactKind::Error
        } else {
            SurfacePresentationArtifactKind::Unsupported
        },
        liveness,
        ContentProjectionFallback::NeutralDiagnosticPlaceholder,
        diagnostic,
    )
}

fn availability_for_liveness(liveness: ContentLiveness) -> SurfaceProviderAvailability {
    if liveness == ContentLiveness::Crashed {
        SurfaceProviderAvailability::Error
    } else {
        SurfaceProviderAvailability::Unsupported
    }
}

fn liveness_title(liveness: ContentLiveness) -> &'static str {
    match liveness {
        ContentLiveness::Resolved => "Content",
        ContentLiveness::Missing => "Missing Content",
        ContentLiveness::Loading => "Loading Content",
        ContentLiveness::Suspended => "Suspended Content",
        ContentLiveness::Denied => "Denied Content",
        ContentLiveness::UnsupportedProfile => "Unsupported Content",
        ContentLiveness::Crashed => "Crashed Content",
    }
}

fn liveness_diagnostic_code(liveness: ContentLiveness) -> &'static str {
    match liveness {
        ContentLiveness::Resolved => "editor_composition.content.resolved",
        ContentLiveness::Missing => "editor_composition.content.missing",
        ContentLiveness::Loading => "editor_composition.content.loading",
        ContentLiveness::Suspended => "editor_composition.content.suspended",
        ContentLiveness::Denied => "editor_composition.content.denied",
        ContentLiveness::UnsupportedProfile => "editor_composition.content.unsupported_profile",
        ContentLiveness::Crashed => "editor_composition.content.crashed",
    }
}

fn liveness_actionable_message(liveness: ContentLiveness) -> &'static str {
    match liveness {
        ContentLiveness::Resolved => "Use the resolved content projection.",
        ContentLiveness::Missing => "Restore or remap the missing mounted content binding.",
        ContentLiveness::Loading => "Wait for the mounted content provider to finish loading.",
        ContentLiveness::Suspended => "Resume the mounted content session.",
        ContentLiveness::Denied => "Grant the required app capability or select allowed content.",
        ContentLiveness::UnsupportedProfile => {
            "Install a provider that supports this mounted content profile."
        }
        ContentLiveness::Crashed => "Inspect the provider failure and restart the content session.",
    }
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
