use super::*;
use crate::texture_preview::{TexturePreviewViewModel, texture_preview_view_model};

const TEXTURE_PREVIEW_ROOT_WIDGET_ID: WidgetId = WidgetId(52_000);
const TEXTURE_PREVIEW_SCROLL_WIDGET_ID: WidgetId = WidgetId(52_001);
const TEXTURE_PREVIEW_BODY_WIDGET_ID: WidgetId = WidgetId(52_002);
const TEXTURE_PREVIEW_SURFACE_WIDGET_ID: WidgetId = WidgetId(52_003);
const TEXTURE_PREVIEW_CONTROLS_WIDGET_ID: WidgetId = WidgetId(52_004);
const TEXTURE_PREVIEW_LINE_WIDGET_ID_BASE: u64 = 52_100;
const TEXTURE_PREVIEW_ACTION_WIDGET_ID_BASE: u64 = 52_300;

pub(super) struct TextureViewerProvider;

impl EditorSurfaceProvider for TextureViewerProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            TEXTURE_VIEWER_PROVIDER_ID,
            "Texture Viewer",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_support(request, TEXTURE_VIEWER_2D_SURFACE_KEY)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model = texture_preview_view_model(
            context.app.asset_catalog_runtime().catalog(),
            context.app.asset_catalog_runtime().selected_asset_id(),
            context.app.texture_preview_runtime(),
            TextureViewerSurfaceKind::Texture2D,
        );
        let (root, routes) = build_texture_preview_panel(
            context.theme,
            request,
            &view_model,
            vec![(
                "Reset".to_string(),
                TextureSurfaceAction::ResetPreview {
                    surface: TextureViewerSurfaceKind::Texture2D,
                },
            )],
        );

        Ok(ProviderSurfaceFrame {
            title: "Texture Viewer".to_string(),
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
        texture_surface_action_command(action, context.projection_epoch)
    }
}

pub(super) fn build_texture_preview_panel(
    theme: &ThemeTokens,
    request: &SurfaceProviderRequest,
    view_model: &TexturePreviewViewModel,
    actions: Vec<(String, TextureSurfaceAction)>,
) -> (UiNode, SurfaceRouteTable) {
    let scope = editor_shell::SurfaceWidgetScope::new(request.tool_surface_instance_id);
    let text_style = theme.body_small_text_style(FontId(1));
    let mut routes = SurfaceRouteTable::empty();
    let mut body_children = Vec::new();

    if let Some(surface) = &view_model.product_surface {
        body_children.push(editor_shell::product_surface(
            scope.widget_id(TEXTURE_PREVIEW_SURFACE_WIDGET_ID),
            surface.source.clone(),
            ui_math::UiSize::new(surface.width.max(1) as f32, surface.height.max(1) as f32),
        ));
    }

    let lines = texture_preview_lines(view_model);
    for (index, line) in lines.into_iter().enumerate() {
        body_children.push(editor_shell::label(
            scope.widget_id(WidgetId(TEXTURE_PREVIEW_LINE_WIDGET_ID_BASE + index as u64)),
            line,
            text_style.clone(),
        ));
    }

    if !actions.is_empty() {
        let mut action_nodes = Vec::with_capacity(actions.len());
        for (index, (label, action)) in actions.into_iter().enumerate() {
            let widget_id = scope.widget_id(WidgetId(
                TEXTURE_PREVIEW_ACTION_WIDGET_ID_BASE + index as u64,
            ));
            action_nodes.push(editor_shell::compact_surface_action_button(
                widget_id, label, false, true, theme,
            ));
            routes.insert(
                widget_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Texture(action)),
            );
        }
        body_children.push(editor_shell::hstack(
            scope.widget_id(TEXTURE_PREVIEW_CONTROLS_WIDGET_ID),
            theme.spacing.xs,
            action_nodes,
        ));
    }

    let body = editor_shell::vstack(
        scope.widget_id(TEXTURE_PREVIEW_BODY_WIDGET_ID),
        theme.spacing.xs,
        body_children,
    );
    let scroll = editor_shell::vscroll(
        scope.widget_id(TEXTURE_PREVIEW_SCROLL_WIDGET_ID),
        theme.clone(),
        vec![body],
    );
    (
        editor_shell::panel(
            scope.widget_id(TEXTURE_PREVIEW_ROOT_WIDGET_ID),
            theme.clone(),
            vec![scroll],
        ),
        routes,
    )
}

pub(super) fn texture_preview_lines(view_model: &TexturePreviewViewModel) -> Vec<String> {
    let mut lines = vec![
        "texture viewer: rendered GPU product-surface preview".to_string(),
        "descriptor text is diagnostics only; completion evidence is the product surface and proof metadata".to_string(),
    ];
    if let Some(descriptor) = view_model.descriptor {
        lines.push(format!(
            "preview descriptor: product={} mip={} slice={} channel={:?} color_space={:?}",
            descriptor.product_id.raw(),
            descriptor.mip_level,
            descriptor.slice_index,
            descriptor.channel,
            descriptor.color_space_override
        ));
    } else {
        lines.push(
            "preview descriptor: unavailable until a typed texture product is selected".to_string(),
        );
    }
    if let Some(proof) = &view_model.proof {
        lines.extend([
            format!("texture product id: {}", proof.texture_product_id),
            format!("descriptor hash: {}", proof.descriptor_hash),
            format!("artifact URI: {}", proof.artifact_uri),
            format!("upload format: {}", proof.upload_format),
            format!("mip count: {}", proof.mip_count),
            format!("selected mip: {}", proof.selected_mip),
            format!("selected slice: {}", proof.selected_slice),
            format!("selected channel: {}", proof.selected_channel),
            format!("sampler identity: {}", proof.sampler_identity),
            format!("bind group identity: {}", proof.bind_group_identity),
            format!("residency state: {}", proof.residency_state),
            format!("residency class: {}", proof.residency_class),
            format!("preview target: {}", proof.target_key.label()),
        ]);
    }
    for diagnostic in &view_model.diagnostics {
        lines.push(format!(
            "texture preview diagnostic {:?}: {}",
            diagnostic.code, diagnostic.message
        ));
    }
    lines
}

pub(super) fn texture_surface_action_command(
    action: SurfaceLocalAction,
    projection_epoch: u64,
) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
    let SurfaceLocalAction::Texture(action) = action else {
        return Ok(None);
    };
    Ok(Some(SurfaceCommandProposal::Shell(
        ShellCommand::ApplyTextureSurfaceAction {
            action,
            projection_epoch,
        },
    )))
}
