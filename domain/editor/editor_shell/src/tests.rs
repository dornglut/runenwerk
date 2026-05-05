use std::collections::BTreeMap;

use editor_core::EntityId;
use ui_theme::ThemeTokens;

use crate::{
    EditorShellFrameModel, PanelInstanceId, PanelKind, ResolvedSurfaceFrame, ShellCommand,
    SurfaceLocalAction, SurfaceLocalRoute, SurfacePresentationArtifact,
    SurfaceProviderAvailability, SurfaceProviderId, SurfaceRouteTable, ToolbarButtonViewModel,
    ToolbarViewModel, UiInteraction, UiInteractionResults, WidgetId, WorkspaceIdentityAllocator,
    WorkspaceState, build_editor_shell_frame, label, map_interactions_to_shell_commands,
    tool_surface_definition_id,
};

#[test]
fn toolbar_activation_maps_to_shell_command() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(2),
                stable_name: "translate",
                label: "Translate".to_string(),
                is_active: false,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
            )],
        },
        &build.projection_artifacts,
    );

    assert_eq!(commands, vec![ShellCommand::ActivateTranslateTool]);
}

#[test]
fn provider_route_activation_maps_to_surface_local_dispatch_command() {
    let workspace = sample_workspace_state();
    let (panel_id, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let frame_model = frame_model_with_surface_route(
        &workspace,
        surface_id,
        WidgetId(50_000),
        SurfaceLocalAction::SelectOutlinerEntity {
            entity: EntityId(42),
        },
    );

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(WidgetId(50_000))],
        },
        &build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [ShellCommand::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            target,
            action: SurfaceLocalAction::SelectOutlinerEntity { entity },
            projection_epoch,
        }] if *provider_id == SurfaceProviderId::try_from_raw(77).unwrap()
            && *tool_surface_instance_id == surface_id
            && target.panel_instance_id == panel_id
            && target.active_tool_surface == Some(surface_id)
            && *entity == EntityId(42)
            && *projection_epoch == build.projection_artifacts.projection_epoch
    ));
}

#[test]
fn provider_route_rejects_mismatched_structural_context() {
    let workspace = sample_workspace_state();
    let (_, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let frame_model = frame_model_with_surface_route(
        &workspace,
        surface_id,
        WidgetId(50_001),
        SurfaceLocalAction::SelectOutlinerEntity {
            entity: EntityId(42),
        },
    );
    let mut build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    build
        .projection_artifacts
        .widget_structural_context_by_id
        .insert(
            WidgetId(50_001),
            crate::StructuralWidgetRoutingContext {
                panel_instance_id: PanelInstanceId::try_from_raw(999).unwrap(),
                active_tool_surface: None,
                tab_stack_id: crate::TabStackId::try_from_raw(999).unwrap(),
            },
        );

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(WidgetId(50_001))],
        },
        &build.projection_artifacts,
    );

    assert_eq!(commands, vec![ShellCommand::NoOp]);
}

#[test]
fn frame_model_surfaces_are_artifact_lookup_not_layout_authority() {
    let workspace = sample_workspace_state();
    let (_, outliner_surface) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let (_, viewport_surface) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let frame_model = frame_model_with_only_surface(&workspace, viewport_surface);

    assert!(frame_model.surface(viewport_surface).is_some());
    assert!(frame_model.surface(outliner_surface).is_none());

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    assert!(
        build
            .projection_artifacts
            .workspace
            .widget_context_by_id
            .values()
            .any(|context| context.active_tool_surface == Some(outliner_surface)),
        "workspace projection still owns mounted surface layout even when the frame lookup lacks an artifact"
    );
}

fn frame_model_with_surface_route(
    workspace: &WorkspaceState,
    routed_surface: crate::ToolSurfaceInstanceId,
    widget_id: WidgetId,
    action: SurfaceLocalAction,
) -> EditorShellFrameModel {
    let mut frame_model = frame_model_for_workspace(workspace);
    let frame = frame_model
        .surfaces
        .get_mut(&routed_surface)
        .expect("routed surface should exist in frame model");
    frame
        .routes
        .insert(widget_id, SurfaceLocalRoute::new(action));
    frame.artifact.root = label(
        widget_id,
        frame.title.clone(),
        ThemeTokens::default().body_small_text_style(ui_text::FontId(1)),
    );
    frame_model
}

fn frame_model_with_only_surface(
    workspace: &WorkspaceState,
    surface_id: crate::ToolSurfaceInstanceId,
) -> EditorShellFrameModel {
    let mut frame_model = EditorShellFrameModel::new(ToolbarViewModel::default(), BTreeMap::new());
    let panel = workspace
        .panels()
        .find(|panel| panel.active_tool_surface == Some(surface_id))
        .expect("surface should be mounted");
    let tab_stack_id = workspace
        .tab_stacks()
        .find(|stack| stack.ordered_panels.contains(&panel.id))
        .map(|stack| stack.id)
        .expect("mounted panel should belong to a tab stack");
    let surface = workspace
        .tool_surface(surface_id)
        .expect("surface should exist");
    frame_model.surfaces.insert(
        surface_id,
        surface_frame(
            panel.id,
            tab_stack_id,
            surface,
            WidgetId(surface_id.raw() + 10_000),
        ),
    );
    frame_model
}

fn frame_model_for_workspace(workspace: &WorkspaceState) -> EditorShellFrameModel {
    let mut surfaces = BTreeMap::new();
    for panel in workspace.panels() {
        let Some(surface_id) = panel.active_tool_surface else {
            continue;
        };
        let Some(surface) = workspace.tool_surface(surface_id) else {
            continue;
        };
        let Some(tab_stack_id) = workspace
            .tab_stacks()
            .find(|stack| stack.ordered_panels.contains(&panel.id))
            .map(|stack| stack.id)
        else {
            continue;
        };
        surfaces.insert(
            surface_id,
            surface_frame(
                panel.id,
                tab_stack_id,
                surface,
                WidgetId(surface_id.raw() + 10_000),
            ),
        );
    }
    EditorShellFrameModel::new(ToolbarViewModel::default(), surfaces)
}

fn surface_frame(
    panel_instance_id: PanelInstanceId,
    tab_stack_id: crate::TabStackId,
    surface: &crate::ToolSurfaceState,
    root_widget_id: WidgetId,
) -> ResolvedSurfaceFrame {
    ResolvedSurfaceFrame {
        surface_instance_id: surface.id,
        panel_instance_id,
        tab_stack_id,
        surface_kind: surface.tool_surface_kind,
        surface_definition_id: tool_surface_definition_id(surface.tool_surface_kind),
        provider_id: Some(SurfaceProviderId::try_from_raw(77).unwrap()),
        title: format!("{:?}", surface.tool_surface_kind),
        artifact: SurfacePresentationArtifact::provider(label(
            root_widget_id,
            "surface",
            ThemeTokens::default().body_small_text_style(ui_text::FontId(1)),
        )),
        routes: SurfaceRouteTable::empty(),
        availability: SurfaceProviderAvailability::Available,
    }
}

fn sample_workspace_state() -> WorkspaceState {
    let mut allocator = WorkspaceIdentityAllocator::new();
    let workspace_id = allocator.allocate_workspace_id();
    WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
}

fn panel_and_surface_by_kind(
    workspace: &WorkspaceState,
    panel_kind: PanelKind,
) -> (PanelInstanceId, crate::ToolSurfaceInstanceId) {
    workspace
        .panels()
        .find(|panel| panel.panel_kind == panel_kind)
        .and_then(|panel| {
            panel
                .active_tool_surface
                .map(|surface_id| (panel.id, surface_id))
        })
        .expect("expected mounted surface for panel kind")
}
