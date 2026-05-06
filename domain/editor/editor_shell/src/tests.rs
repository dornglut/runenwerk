use std::collections::BTreeMap;

use editor_core::EntityId;
use ui_theme::ThemeTokens;

use crate::{
    EditorShellFrameModel, PanelInstanceId, PanelKind, ResolvedSurfaceFrame, ShellCommand,
    SurfaceLocalAction, SurfaceLocalRoute, SurfacePresentationArtifact,
    SurfaceProviderAvailability, SurfaceProviderId, SurfaceRouteTable, ToolSurfaceKind,
    ToolbarButtonViewModel, ToolbarViewModel, UiInteraction, UiInteractionResults, WidgetId,
    WorkspaceIdentityAllocator, WorkspaceMutation, WorkspaceSplitAxis, WorkspaceState,
    build_editor_shell_frame, label, map_interactions_to_shell_commands, reduce_workspace,
    tab_close_button_widget_id, tab_stack_kind_select_widget_id,
    tab_stack_new_tab_button_widget_id, tab_stack_split_horizontal_button_widget_id,
    tool_surface_definition_id, workspace_split_host_widget_id,
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
fn top_bar_menu_and_workspace_buttons_map_to_shell_commands() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_001),
                    stable_name: "menu_file",
                    label: "File".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_100),
                    stable_name: "file_save",
                    label: "Save".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_101),
                    stable_name: "file_save_as",
                    label: "Save As".to_string(),
                    is_active: false,
                    enabled: false,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_002),
                    stable_name: "workspace_modelling",
                    label: "Modelling".to_string(),
                    is_active: false,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(crate::TOOLBAR_FILE_MENU_WIDGET_ID),
                UiInteraction::Activated(crate::toolbar_menu_item_widget_id(1)),
                UiInteraction::Activated(crate::toolbar_menu_item_widget_id(2)),
                UiInteraction::Activated(crate::TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID),
            ],
        },
        &build.projection_artifacts,
    );

    assert_eq!(
        commands,
        vec![
            ShellCommand::ToggleToolbarMenu {
                menu: crate::ToolbarMenuKind::File,
            },
            ShellCommand::RunToolbarCommand {
                command: crate::ToolbarCommandKind::SaveScene,
            },
            ShellCommand::NoOp,
            ShellCommand::SwitchWorkspaceProfile {
                profile_id: crate::MODELLING_WORKSPACE_PROFILE_ID,
            },
        ]
    );
}

#[test]
fn default_scene_workspace_uses_viewport_left_and_hierarchy_over_inspector_right() {
    let workspace = sample_workspace_state();
    let projection =
        crate::project_fixed_layout(&workspace).expect("default layout should project");

    assert_eq!(projection.left_right_fraction, 0.72);
    assert_eq!(projection.center_right_fraction, 0.56);
    assert_eq!(
        projection
            .viewport
            .active_panel
            .map(|panel| panel.panel_kind),
        Some(PanelKind::Viewport)
    );
    assert_eq!(
        projection
            .outliner
            .active_panel
            .map(|panel| panel.panel_kind),
        Some(PanelKind::Outliner)
    );
    assert_eq!(
        projection
            .inspector
            .active_panel
            .map(|panel| panel.panel_kind),
        Some(PanelKind::Inspector)
    );
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
fn tab_chrome_maps_shell_owned_controls_to_structural_commands() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let projection_epoch = build.projection_artifacts.projection_epoch;

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::SelectChanged {
                    target: tab_stack_kind_select_widget_id(viewport_stack),
                    index: 3,
                },
                UiInteraction::Activated(tab_stack_new_tab_button_widget_id(viewport_stack)),
                UiInteraction::Activated(tab_stack_split_horizontal_button_widget_id(
                    viewport_stack,
                )),
                UiInteraction::Activated(tab_close_button_widget_id(viewport_stack, 0)),
            ],
        },
        &build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::SwitchPanelToolSurfaceKind {
                panel_instance_id,
                tool_surface_kind: ToolSurfaceKind::Inspector,
                projection_epoch: select_epoch,
            },
            ShellCommand::CreatePanelTab {
                tab_stack_id: create_stack,
                ..
            },
            ShellCommand::SplitTabStackArea {
                tab_stack_id: split_stack,
                ..
            },
            ShellCommand::ClosePanelTab {
                tab_stack_id: close_stack,
                panel_instance_id: close_panel,
                projection_epoch: close_epoch,
            },
        ] if *panel_instance_id == viewport_panel
            && *select_epoch == projection_epoch
            && *create_stack == viewport_stack
            && *split_stack == viewport_stack
            && *close_stack == viewport_stack
            && *close_panel == viewport_panel
            && *close_epoch == projection_epoch
    ));
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

#[test]
fn shell_frame_renders_dynamic_split_workspace_after_area_split() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
    let split_host_id = allocator.allocate_panel_host_id();
    let first_child_host_id = allocator.allocate_panel_host_id();
    let second_child_host_id = allocator.allocate_panel_host_id();
    let new_tab_stack_id = allocator.allocate_tab_stack_id();
    let new_panel_id = allocator.allocate_panel_instance_id();
    let new_surface_id = allocator.allocate_tool_surface_instance_id();

    let split_workspace = reduce_workspace(
        &workspace,
        WorkspaceMutation::SplitTabStackArea {
            tab_stack_id: viewport_stack,
            axis: WorkspaceSplitAxis::Horizontal,
            split_host_id,
            first_child_host_id,
            second_child_host_id,
            new_tab_stack_id,
            new_panel_id,
            new_panel_kind: PanelKind::Inspector,
            new_tool_surface_id: new_surface_id,
            new_tool_surface_kind: ToolSurfaceKind::Inspector,
            fraction: 0.5,
        },
    )
    .expect("split area should produce a valid workspace graph");
    let frame_model = frame_model_for_workspace(&split_workspace);

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &split_workspace);

    assert!(
        build.projection_artifacts.workspace.fixed_layout.is_none(),
        "arbitrary split workspace should leave the legacy fixed projection path",
    );
    assert!(
        build
            .projection_artifacts
            .workspace
            .tab_button_route_by_widget_id
            .values()
            .any(|route| route.tab_stack_id == new_tab_stack_id
                && route.panel_instance_id == new_panel_id),
        "dynamic projection should route tabs in newly split areas",
    );
    assert!(
        ui_tree_contains_widget(
            &build.tree.root,
            workspace_split_host_widget_id(split_host_id)
        ),
        "dynamic composition should render the newly inserted split host",
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

fn tab_stack_by_panel(workspace: &WorkspaceState, panel_id: PanelInstanceId) -> crate::TabStackId {
    workspace
        .tab_stacks()
        .find(|stack| stack.ordered_panels.contains(&panel_id))
        .map(|stack| stack.id)
        .expect("panel should belong to a tab stack")
}

fn ui_tree_contains_widget(node: &crate::UiNode, widget_id: WidgetId) -> bool {
    node.id == widget_id
        || node
            .children
            .iter()
            .any(|child| ui_tree_contains_widget(child, widget_id))
}
