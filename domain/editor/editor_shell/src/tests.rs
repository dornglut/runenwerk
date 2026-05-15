use std::collections::BTreeMap;

use editor_core::{ComponentTypeId, EntityId};
use ui_input::{
    Key, KeyState, KeyboardEvent, Modifiers, PointerEvent, PointerEventKind, TextInputEvent,
    UiInputEvent,
};
use ui_math::{Axis, UiRect};
use ui_theme::ThemeTokens;

use crate::{
    ActiveTabDragVisualState, ActiveTabStackPopupMenu, DockDropCandidate, DockDropScope,
    DockingInteractionVisualState, DockingPreviewDropTarget,
    ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID, EditorShellFrameModel,
    EntityTableComponentFilter, EntityTableHierarchyFilter, EntityTableSurfaceAction,
    InspectorSurfaceAction, OutlinerSurfaceAction, PanelInstanceId, PanelKind,
    ResolvedSurfaceFrame, RoutedShellAction, ShellCommand, SurfaceLocalAction, SurfaceLocalRoute,
    SurfacePresentationArtifact, SurfaceProviderAvailability, SurfaceProviderId, SurfaceRouteTable,
    TabStackPopupMenuKind, ToolSurfaceKind, ToolbarButtonViewModel, ToolbarViewModel,
    UiInteraction, UiInteractionResults, ViewportSurfaceAction, WidgetId,
    WorkspaceIdentityAllocator, WorkspaceMutation, WorkspaceSplitAxis, WorkspaceState,
    build_editor_shell_frame, build_editor_shell_frame_with_docking_visual_state,
    build_entity_table_panel, dock_split_preview_label_widget_id,
    dock_split_preview_overlay_widget_id, dock_split_preview_panel_widget_id, label,
    map_interactions_to_shell_commands, panel_kind_definition_key, reduce_workspace,
    tab_close_button_widget_id, tab_stack_action_menu_popup_widget_id,
    tab_stack_container_widget_id, tab_stack_new_surface_menu_item_widget_id,
    tab_stack_new_surface_menu_popup_widget_id, tab_stack_new_tab_button_widget_id,
    tab_stack_split_horizontal_button_widget_id, tab_stack_surface_submenu_anchor_widget_id,
    tool_surface_definition_id, tool_surface_kind_definition_key,
    toolbar_workspace_close_widget_id, workspace_split_host_widget_id,
};

#[test]
fn toolbar_omits_global_transform_tool_buttons() {
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
                    id: editor_core::ToolId(3_001),
                    stable_name: "workspace_scene",
                    label: "Scene".to_string(),
                    is_active: true,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    for removed_widget in [
        crate::TOOLBAR_SELECT_BUTTON_WIDGET_ID,
        crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
        crate::TOOLBAR_ROTATE_BUTTON_WIDGET_ID,
        crate::TOOLBAR_SCALE_BUTTON_WIDGET_ID,
    ] {
        assert!(
            build.tree.walk().all(|node| node.id != removed_widget),
            "global transform tool buttons should not be projected in the top toolbar",
        );
    }
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
                    is_active: true,
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
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_003),
                    stable_name: "workspace_editor_design",
                    label: "Editor Design".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_004),
                    stable_name: "workspace_plus",
                    label: "+".to_string(),
                    is_active: true,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_402),
                    stable_name: "workspace_menu_editor_design",
                    label: "Editor Design".to_string(),
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
                UiInteraction::Activated(toolbar_workspace_close_widget_id(
                    crate::MODELLING_WORKSPACE_PROFILE_ID,
                )),
                UiInteraction::Activated(crate::TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID),
                UiInteraction::Activated(crate::TOOLBAR_ADD_WORKSPACE_WIDGET_ID),
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
            ShellCommand::CloseWorkspaceProfile {
                profile_id: crate::MODELLING_WORKSPACE_PROFILE_ID,
            },
            ShellCommand::SwitchWorkspaceProfile {
                profile_id: crate::EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            },
            ShellCommand::ToggleToolbarMenu {
                menu: crate::ToolbarMenuKind::Workspace,
            },
        ]
    );

    let workspace_menu_frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(3_004),
                stable_name: "workspace_plus",
                label: "+".to_string(),
                is_active: true,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    );
    let workspace_menu_build = build_editor_shell_frame(
        &workspace_menu_frame_model,
        &ThemeTokens::default(),
        &workspace,
    );
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                crate::toolbar_menu_item_widget_id(2),
            )],
        },
        &workspace_menu_build.projection_artifacts,
    );
    assert_eq!(
        commands,
        vec![ShellCommand::SwitchWorkspaceProfile {
            profile_id: crate::EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        }]
    );
}

#[test]
fn toolbar_route_slots_use_app_supplied_route_actions_before_fallback() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(2_001),
                stable_name: "menu_file",
                label: "File".to_string(),
                is_active: true,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    )
    .with_active_ui_definitions(
        None,
        Some(editor_definition::EditorToolbarBinding {
            template: "unused.toolbar.template".into(),
            workspace_catalog: None,
            routes: Vec::new(),
            availability: Vec::new(),
            menus: Vec::new(),
            menu_items: vec![editor_definition::EditorToolbarMenuItemBinding {
                menu_id: "file".to_string(),
                item_id: "apply".to_string(),
                label: "Apply".to_string(),
                route: ui_definition::UiRouteSlotId::new("authored.apply-selected"),
                availability: None,
            }],
        }),
        None,
    )
    .with_route_actions(BTreeMap::from([(
        "authored.apply-selected".to_string(),
        RoutedShellAction::ApplySelectedEditorDefinition,
    )]));
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                crate::toolbar_menu_item_widget_id(0),
            )],
        },
        &build.projection_artifacts,
    );

    assert_eq!(commands, vec![ShellCommand::ApplySelectedEditorDefinition]);
}

#[test]
fn active_top_bar_menu_projects_as_popup_without_pushing_content_down() {
    let active_frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_001),
                    stable_name: "menu_file",
                    label: "File".to_string(),
                    is_active: true,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_100),
                    stable_name: "file_save",
                    label: "Save".to_string(),
                    is_active: false,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let inactive_frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(2_001),
                stable_name: "menu_file",
                label: "File".to_string(),
                is_active: false,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let theme = ThemeTokens::default();
    let active = build_editor_shell_frame(&active_frame_model, &theme, &workspace);
    let inactive = build_editor_shell_frame(&inactive_frame_model, &theme, &workspace);
    let active_layouts = ui_runtime::compute_tree_layout(
        &active.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    let inactive_layouts = ui_runtime::compute_tree_layout(
        &inactive.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );

    assert!(
        active_layouts.contains_key(&crate::TOOLBAR_MENU_POPUP_WIDGET_ID),
        "active menu should project a retained popup"
    );
    assert!(
        !inactive_layouts.contains_key(&crate::TOOLBAR_MENU_POPUP_WIDGET_ID),
        "inactive menu should not project popup layout"
    );
    assert_eq!(
        active_layouts
            .get(&crate::BODY_ROOT_WIDGET_ID)
            .expect("active body root")
            .bounds,
        inactive_layouts
            .get(&crate::BODY_ROOT_WIDGET_ID)
            .expect("inactive body root")
            .bounds,
        "opening a menu must not add a second toolbar row or push content down"
    );
}

#[test]
fn entity_table_control_rail_overflows_and_scrolls_from_child_controls() {
    let theme = ThemeTokens::default();
    let tree = ui_tree::UiTree::new(build_entity_table_panel(
        &crate::EntityTableViewModel::default(),
        &theme,
        PanelInstanceId::try_from_raw(1).unwrap(),
        None,
    ));
    let bounds = ui_math::UiRect::new(0.0, 0.0, 260.0, 240.0);
    let mut runtime = ui_runtime::UiRuntime::new();
    let layouts = runtime.compute_layout(&tree, bounds);
    let max_offset = runtime
        .max_scroll_offset_for_layout_axis(
            &tree,
            &layouts,
            ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID,
            Axis::Horizontal,
        )
        .expect("entity controls rail should be a scroll node");

    assert!(
        max_offset > 0.0,
        "entity table controls must measure to content width and overflow when narrow"
    );

    let search_bounds = layouts
        .get(&ENTITY_TABLE_SEARCH_WIDGET_ID)
        .expect("entity search layout should exist")
        .bounds;
    let pointer = ui_math::UiPoint::new(
        search_bounds.x + search_bounds.width * 0.5,
        search_bounds.y + search_bounds.height * 0.5,
    );
    assert_eq!(
        ui_runtime::hit_test_widget(&tree, &layouts, pointer),
        Some(ENTITY_TABLE_SEARCH_WIDGET_ID),
        "the controls rail row must have stable identity so child controls win hit testing",
    );
    let _ = runtime.dispatch_input(
        &tree,
        &layouts,
        &UiInputEvent::Pointer(PointerEvent {
            kind: PointerEventKind::Scroll,
            position: pointer,
            delta: ui_math::UiVector::new(0.0, -8.0),
            button: None,
            modifiers: Modifiers::default(),
            click_count: 0,
            ..Default::default()
        }),
    );

    assert!(
        runtime.scroll_offset_for_axis(ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID, Axis::Horizontal)
            > 0.0,
        "wheel input over the search child should scroll the controls rail owner"
    );
}

#[test]
fn toolbar_separator_projects_as_centered_visible_divider() {
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
                    id: editor_core::ToolId(2_002),
                    stable_name: "menu_edit",
                    label: "Edit".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_003),
                    stable_name: "menu_window",
                    label: "Window".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_001),
                    stable_name: "workspace_scene",
                    label: "Scene".to_string(),
                    is_active: true,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    let separator_node = build
        .tree
        .walk()
        .find(|node| node.id == crate::TOOLBAR_SEPARATOR_WIDGET_ID)
        .expect("toolbar separator should exist");
    assert!(matches!(
        &separator_node.kind,
        crate::UiNodeKind::Divider(_)
    ));

    let row = layouts
        .get(&crate::TOOLBAR_ROW_WIDGET_ID)
        .expect("toolbar row should have layout");
    let separator = layouts
        .get(&crate::TOOLBAR_SEPARATOR_WIDGET_ID)
        .expect("toolbar separator should have layout");

    assert!((separator.bounds.width - 1.0).abs() < 0.001);
    assert!((separator.bounds.height - 14.0).abs() < 0.001);
    assert!(
        (separator.bounds.y - (row.bounds.y + (row.bounds.height - separator.bounds.height) * 0.5))
            .abs()
            < 0.001,
        "toolbar separator should be centered vertically in the row"
    );
}

#[test]
fn default_scene_workspace_uses_viewport_left_and_hierarchy_over_inspector_right() {
    let workspace = sample_workspace_state();
    let projection =
        crate::project_workspace_for_shell(&workspace).expect("default layout should project");
    let crate::ProjectedWorkspaceHostSlot::Split {
        axis: WorkspaceSplitAxis::Vertical,
        fraction: body_console_fraction,
        first_child: left_right,
        ..
    } = &projection.root_host
    else {
        panic!("default root host should be a vertical graph split");
    };
    let crate::ProjectedWorkspaceHostSlot::Split {
        axis: WorkspaceSplitAxis::Horizontal,
        fraction: left_right_fraction,
        first_child: viewport,
        second_child: right_sidebar,
        ..
    } = left_right.as_ref()
    else {
        panic!("default upper body should be a horizontal graph split");
    };
    let crate::ProjectedWorkspaceHostSlot::Split {
        axis: WorkspaceSplitAxis::Vertical,
        fraction: center_right_fraction,
        first_child: outliner,
        second_child: inspector,
        ..
    } = right_sidebar.as_ref()
    else {
        panic!("default right sidebar should be a vertical graph split");
    };
    let crate::ProjectedWorkspaceHostSlot::TabStack {
        tab_stack: viewport,
        ..
    } = viewport.as_ref()
    else {
        panic!("default viewport slot should be a tab stack");
    };
    let crate::ProjectedWorkspaceHostSlot::TabStack {
        tab_stack: outliner,
        ..
    } = outliner.as_ref()
    else {
        panic!("default outliner slot should be a tab stack");
    };
    let crate::ProjectedWorkspaceHostSlot::TabStack {
        tab_stack: inspector,
        ..
    } = inspector.as_ref()
    else {
        panic!("default inspector slot should be a tab stack");
    };

    assert_eq!(*body_console_fraction, 0.78);
    assert_eq!(*left_right_fraction, 0.72);
    assert_eq!(*center_right_fraction, 0.56);
    assert_eq!(
        viewport.active_panel.map(|panel| panel.panel_kind),
        Some(PanelKind::Viewport)
    );
    assert_eq!(
        outliner.active_panel.map(|panel| panel.panel_kind),
        Some(PanelKind::Outliner)
    );
    assert_eq!(
        inspector.active_panel.map(|panel| panel.panel_kind),
        Some(PanelKind::Inspector)
    );
}

#[test]
fn panel_and_tool_surface_definition_keys_share_workspace_vocabulary() {
    for (panel_kind, tool_surface_kind, expected_key) in [
        (PanelKind::Outliner, ToolSurfaceKind::Outliner, "outliner"),
        (
            PanelKind::EntityTable,
            ToolSurfaceKind::EntityTable,
            "entity_table",
        ),
        (PanelKind::Viewport, ToolSurfaceKind::Viewport, "viewport"),
        (
            PanelKind::Inspector,
            ToolSurfaceKind::Inspector,
            "inspector",
        ),
        (PanelKind::Console, ToolSurfaceKind::Console, "console"),
        (
            PanelKind::EditorDesignOutliner,
            ToolSurfaceKind::EditorDesignOutliner,
            "editor_design_outliner",
        ),
        (
            PanelKind::UiHierarchy,
            ToolSurfaceKind::UiHierarchy,
            "ui_hierarchy",
        ),
        (PanelKind::UiCanvas, ToolSurfaceKind::UiCanvas, "ui_canvas"),
        (
            PanelKind::StyleInspector,
            ToolSurfaceKind::StyleInspector,
            "style_inspector",
        ),
        (PanelKind::Bindings, ToolSurfaceKind::Bindings, "bindings"),
        (
            PanelKind::DockLayoutPreview,
            ToolSurfaceKind::DockLayoutPreview,
            "dock_layout_preview",
        ),
        (
            PanelKind::ThemeEditor,
            ToolSurfaceKind::ThemeEditor,
            "theme_editor",
        ),
        (
            PanelKind::ShortcutEditor,
            ToolSurfaceKind::ShortcutEditor,
            "shortcut_editor",
        ),
        (
            PanelKind::MenuEditor,
            ToolSurfaceKind::MenuEditor,
            "menu_editor",
        ),
        (
            PanelKind::DefinitionValidation,
            ToolSurfaceKind::DefinitionValidation,
            "definition_validation",
        ),
        (
            PanelKind::CommandDiff,
            ToolSurfaceKind::CommandDiff,
            "command_diff",
        ),
        (
            PanelKind::AssetBrowser,
            ToolSurfaceKind::AssetBrowser,
            "asset_browser",
        ),
        (
            PanelKind::ImportInspector,
            ToolSurfaceKind::ImportInspector,
            "import_inspector",
        ),
        (
            PanelKind::FieldProductViewer,
            ToolSurfaceKind::FieldProductViewer,
            "field_product_viewer",
        ),
        (
            PanelKind::SdfBrushBrowser,
            ToolSurfaceKind::SdfBrushBrowser,
            "sdf_brush_browser",
        ),
        (
            PanelKind::GraphCanvas,
            ToolSurfaceKind::GraphCanvas,
            "graph_canvas",
        ),
        (
            PanelKind::Diagnostics,
            ToolSurfaceKind::Diagnostics,
            "diagnostics",
        ),
        (
            PanelKind::RuntimeDebug,
            ToolSurfaceKind::RuntimeDebug,
            "runtime_debug",
        ),
        (
            PanelKind::FieldLayerStack,
            ToolSurfaceKind::FieldLayerStack,
            "field_layer_stack",
        ),
        (
            PanelKind::SdfGraphCanvas,
            ToolSurfaceKind::SdfGraphCanvas,
            "sdf_graph_canvas",
        ),
        (
            PanelKind::MaterialGraphCanvas,
            ToolSurfaceKind::MaterialGraphCanvas,
            "material_graph_canvas",
        ),
        (
            PanelKind::MaterialInspector,
            ToolSurfaceKind::MaterialInspector,
            "material_inspector",
        ),
        (
            PanelKind::MaterialPreview,
            ToolSurfaceKind::MaterialPreview,
            "material_preview",
        ),
        (
            PanelKind::TextureViewer,
            ToolSurfaceKind::TextureViewer,
            "texture_viewer",
        ),
        (
            PanelKind::VolumeTextureViewer,
            ToolSurfaceKind::VolumeTextureViewer,
            "volume_texture_viewer",
        ),
        (
            PanelKind::ProcgenGraphCanvas,
            ToolSurfaceKind::ProcgenGraphCanvas,
            "procgen_graph_canvas",
        ),
        (
            PanelKind::ProcgenPreview,
            ToolSurfaceKind::ProcgenPreview,
            "procgen_preview",
        ),
        (
            PanelKind::GameplayGraphCanvas,
            ToolSurfaceKind::GameplayGraphCanvas,
            "gameplay_graph_canvas",
        ),
        (
            PanelKind::GameplayCompilerDiagnostics,
            ToolSurfaceKind::GameplayCompilerDiagnostics,
            "gameplay_compiler_diagnostics",
        ),
        (
            PanelKind::ParticleGraphCanvas,
            ToolSurfaceKind::ParticleGraphCanvas,
            "particle_graph_canvas",
        ),
        (
            PanelKind::ParticlePreview,
            ToolSurfaceKind::ParticlePreview,
            "particle_preview",
        ),
        (
            PanelKind::PhysicsAuthoring,
            ToolSurfaceKind::PhysicsAuthoring,
            "physics_authoring",
        ),
        (
            PanelKind::PhysicsDebug,
            ToolSurfaceKind::PhysicsDebug,
            "physics_debug",
        ),
        (PanelKind::Timeline, ToolSurfaceKind::Timeline, "timeline"),
        (
            PanelKind::CurveEditor,
            ToolSurfaceKind::CurveEditor,
            "curve_editor",
        ),
        (
            PanelKind::AnimationGraphCanvas,
            ToolSurfaceKind::AnimationGraphCanvas,
            "animation_graph_canvas",
        ),
        (
            PanelKind::SimulationPreview,
            ToolSurfaceKind::SimulationPreview,
            "simulation_preview",
        ),
        (
            PanelKind::SimulationDiagnostics,
            ToolSurfaceKind::SimulationDiagnostics,
            "simulation_diagnostics",
        ),
        (
            PanelKind::Placeholder,
            ToolSurfaceKind::Placeholder,
            "placeholder",
        ),
    ] {
        assert_eq!(panel_kind_definition_key(panel_kind), expected_key);
        assert_eq!(
            tool_surface_kind_definition_key(tool_surface_kind),
            expected_key
        );
    }
}

#[test]
fn provider_route_activation_maps_to_surface_local_dispatch_command() {
    let workspace = sample_workspace_state();
    let (panel_id, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let frame_model = frame_model_with_surface_route(
        &workspace,
        surface_id,
        WidgetId(50_000),
        SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity {
            entity: EntityId(42),
        }),
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
            action: SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity { entity }),
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
        SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity {
            entity: EntityId(42),
        }),
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
fn surface_text_and_keyboard_input_map_to_typed_entity_table_actions() {
    let widget_id = WidgetId(50_100);
    let actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        widget_id,
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText {
            text: String::new(),
        }),
        vec![
            UiInteraction::TextInput {
                target: widget_id,
                event: TextInputEvent {
                    text: "alpha".to_string(),
                },
            },
            UiInteraction::KeyboardInput {
                target: widget_id,
                event: keyboard_event(Key::Backspace),
            },
        ],
    );

    assert_eq!(
        actions,
        vec![
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText {
                text: "alpha".to_string(),
            }),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::BackspaceSearch),
        ]
    );
}

#[test]
fn surface_toggle_select_and_table_row_input_map_to_typed_entity_table_actions() {
    let selected_only_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_110),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetSelectedOnly {
            selected_only: false,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_110),
            checked: true,
        }],
    );
    assert_eq!(
        selected_only_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetSelectedOnly {
                selected_only: true,
            }
        )]
    );

    let roots_only_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_111),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetHierarchyFilter {
            filter: EntityTableHierarchyFilter::All,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_111),
            checked: true,
        }],
    );
    assert_eq!(
        roots_only_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetHierarchyFilter {
                filter: EntityTableHierarchyFilter::RootsOnly,
            }
        )]
    );

    let component_filter = EntityTableComponentFilter::Has(ComponentTypeId(9));
    let select_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_112),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectComponentFilter {
            filters: vec![EntityTableComponentFilter::All, component_filter],
        }),
        vec![UiInteraction::SelectChanged {
            target: WidgetId(50_112),
            index: 1,
        }],
    );
    assert_eq!(
        select_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetComponentFilter {
                filter: component_filter,
            }
        )]
    );

    let row_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_113),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectRow {
            entities: vec![EntityId(7), EntityId(8)],
        }),
        vec![UiInteraction::TableRowSelected {
            target: WidgetId(50_113),
            row_index: 1,
        }],
    );
    assert_eq!(
        row_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SelectEntity {
                entity: EntityId(8),
            }
        )]
    );
}

#[test]
fn surface_text_keyboard_toggle_and_numeric_input_map_to_typed_inspector_actions() {
    let text_widget_id = WidgetId(50_120);
    let text_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        text_widget_id,
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
            index: 3,
            text: String::new(),
        }),
        vec![
            UiInteraction::TextInput {
                target: text_widget_id,
                event: TextInputEvent {
                    text: "Beta".to_string(),
                },
            },
            UiInteraction::KeyboardInput {
                target: text_widget_id,
                event: keyboard_event(Key::Backspace),
            },
            UiInteraction::KeyboardInput {
                target: text_widget_id,
                event: keyboard_event(Key::Enter),
            },
            UiInteraction::KeyboardInput {
                target: text_widget_id,
                event: keyboard_event(Key::Escape),
            },
        ],
    );
    assert_eq!(
        text_actions,
        vec![
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                index: 3,
                text: "Beta".to_string(),
            }),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::BackspaceFieldText { index: 3 }),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::CommitFieldText { index: 3 }),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::CancelFieldText { index: 3 }),
        ]
    );

    let bool_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        WidgetId(50_121),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldBool {
            index: 4,
            value: false,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_121),
            checked: true,
        }],
    );
    assert_eq!(
        bool_actions,
        vec![SurfaceLocalAction::Inspector(
            InspectorSurfaceAction::SetFieldBool {
                index: 4,
                value: true,
            }
        )]
    );

    let numeric_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        WidgetId(50_122),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber {
            index: 5,
            value: 0.0,
        }),
        vec![UiInteraction::NumericStepped {
            target: WidgetId(50_122),
            value: 12.5,
        }],
    );
    assert_eq!(
        numeric_actions,
        vec![SurfaceLocalAction::Inspector(
            InspectorSurfaceAction::SetFieldNumber {
                index: 5,
                value: 12.5,
            }
        )]
    );

    let enum_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        WidgetId(50_123),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SelectFieldEnum {
            index: 6,
            options: vec!["Nearest".to_string(), "Linear".to_string()],
        }),
        vec![UiInteraction::SelectChanged {
            target: WidgetId(50_123),
            index: 1,
        }],
    );
    assert_eq!(
        enum_actions,
        vec![SurfaceLocalAction::Inspector(
            InspectorSurfaceAction::SetFieldEnum {
                index: 6,
                value: "Linear".to_string(),
            }
        )]
    );

    let root_opaque_actions = mapped_surface_actions_for_route(
        PanelKind::Viewport,
        WidgetId(50_124),
        SurfaceLocalAction::Viewport(ViewportSurfaceAction::SetRootBackgroundOpaque {
            viewport_id: editor_viewport::ViewportId(4),
            enabled: false,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_124),
            checked: true,
        }],
    );
    assert_eq!(
        root_opaque_actions,
        vec![SurfaceLocalAction::Viewport(
            ViewportSurfaceAction::SetRootBackgroundOpaque {
                viewport_id: editor_viewport::ViewportId(4),
                enabled: true,
            }
        )]
    );
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
            ShellCommand::ToggleTabStackCreateSurfaceMenu {
                tab_stack_id: create_stack,
                anchor_widget_id,
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
        ] if *create_stack == viewport_stack
            && *anchor_widget_id == tab_stack_new_tab_button_widget_id(viewport_stack)
            && *split_stack == viewport_stack
            && *close_stack == viewport_stack
            && *close_panel == viewport_panel
            && *close_epoch == projection_epoch
    ));
}

#[test]
fn tab_stack_area_actions_are_projected_as_popup_menu() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let inactive_frame_model = frame_model_for_workspace(&workspace);
    let inactive_build =
        build_editor_shell_frame(&inactive_frame_model, &ThemeTokens::default(), &workspace);

    assert!(!ui_tree_contains_widget(
        &inactive_build.tree.root,
        tab_stack_action_menu_popup_widget_id(viewport_stack)
    ));
    assert!(!ui_tree_contains_widget(
        &inactive_build.tree.root,
        tab_stack_split_horizontal_button_widget_id(viewport_stack)
    ));

    let active_frame_model = frame_model_for_workspace(&workspace)
        .with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::AreaActions,
            tab_stack_id: viewport_stack,
            anchor_widget_id: WidgetId(99_001),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);

    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_action_menu_popup_widget_id(viewport_stack)
    ));
    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_split_horizontal_button_widget_id(viewport_stack)
    ));

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(tab_stack_surface_submenu_anchor_widget_id(
                    viewport_stack,
                )),
                UiInteraction::Activated(tab_stack_split_horizontal_button_widget_id(
                    viewport_stack,
                )),
            ],
        },
        &active_build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::ToggleTabStackSurfaceMenu {
                tab_stack_id,
                anchor_widget_id,
            },
            ShellCommand::SplitTabStackArea {
                tab_stack_id: split_stack,
                ..
            },
        ] if *tab_stack_id == viewport_stack
            && *anchor_widget_id == tab_stack_surface_submenu_anchor_widget_id(viewport_stack)
            && *split_stack == viewport_stack
    ));
}

#[test]
fn tab_plus_projects_create_surface_menu_and_routes_selected_kind() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let available_kinds = vec![
        ToolSurfaceKind::Outliner,
        ToolSurfaceKind::EntityTable,
        ToolSurfaceKind::Viewport,
        ToolSurfaceKind::Inspector,
        ToolSurfaceKind::Console,
    ];
    let frame_model = frame_model_for_workspace(&workspace)
        .with_available_tool_surface_kinds(available_kinds.clone());
    let inactive_build =
        build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);

    assert!(!ui_tree_contains_widget(
        &inactive_build.tree.root,
        tab_stack_new_surface_menu_popup_widget_id(viewport_stack)
    ));

    let plus_commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                tab_stack_new_tab_button_widget_id(viewport_stack),
            )],
        },
        &inactive_build.projection_artifacts,
    );
    assert_eq!(
        plus_commands,
        vec![ShellCommand::ToggleTabStackCreateSurfaceMenu {
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(viewport_stack),
        }]
    );

    let active_frame_model =
        frame_model.with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::CreateSurface,
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(viewport_stack),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);
    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_new_surface_menu_popup_widget_id(viewport_stack)
    ));

    let inspector_index = available_kinds
        .iter()
        .position(|kind| *kind == ToolSurfaceKind::Inspector)
        .expect("inspector kind should be present");
    let create_commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                tab_stack_new_surface_menu_item_widget_id(viewport_stack, inspector_index),
            )],
        },
        &active_build.projection_artifacts,
    );
    assert!(matches!(
        create_commands.as_slice(),
        [
            ShellCommand::CreatePanelTab {
                tab_stack_id,
                tool_surface_kind: ToolSurfaceKind::Inspector,
                ..
            }
        ] if *tab_stack_id == viewport_stack
    ));
}

#[test]
fn locked_tab_plus_menu_shows_all_kinds_but_routes_only_locked_kind() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let workspace = reduce_workspace(
        &workspace,
        WorkspaceMutation::LockTabStackAreaType {
            tab_stack_id: viewport_stack,
            locked_tool_surface_kind: Some(ToolSurfaceKind::Viewport),
        },
    )
    .expect("locking viewport tab stack should succeed");
    let available_kinds = vec![
        ToolSurfaceKind::Outliner,
        ToolSurfaceKind::EntityTable,
        ToolSurfaceKind::Viewport,
        ToolSurfaceKind::Inspector,
        ToolSurfaceKind::Console,
    ];
    let active_frame_model = frame_model_for_workspace(&workspace)
        .with_available_tool_surface_kinds(available_kinds.clone())
        .with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::CreateSurface,
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(viewport_stack),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);
    let viewport_index = available_kinds
        .iter()
        .position(|kind| *kind == ToolSurfaceKind::Viewport)
        .expect("viewport kind should be present");
    let console_index = available_kinds
        .iter()
        .position(|kind| *kind == ToolSurfaceKind::Console)
        .expect("console kind should be present");

    assert!(button_enabled(
        &active_build.tree.root,
        tab_stack_new_surface_menu_item_widget_id(viewport_stack, viewport_index)
    ));
    assert!(!button_enabled(
        &active_build.tree.root,
        tab_stack_new_surface_menu_item_widget_id(viewport_stack, console_index)
    ));

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(tab_stack_new_surface_menu_item_widget_id(
                    viewport_stack,
                    console_index,
                )),
                UiInteraction::Activated(tab_stack_new_surface_menu_item_widget_id(
                    viewport_stack,
                    viewport_index,
                )),
            ],
        },
        &active_build.projection_artifacts,
    );
    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::NoOp,
            ShellCommand::CreatePanelTab {
                tab_stack_id,
                tool_surface_kind: ToolSurfaceKind::Viewport,
                ..
            },
        ] if *tab_stack_id == viewport_stack
    ));
}

#[test]
fn dock_split_preview_projects_side_slice_without_consuming_layout() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let theme = ThemeTokens::default();
    let anchor_widget_id = tab_stack_container_widget_id(viewport_stack);
    let preview_target = DockingPreviewDropTarget::SplitIntoArea {
        target_tab_stack_id: viewport_stack,
        side: crate::DockSplitSide::Left,
    };
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(preview_target),
            preview_candidates: vec![DockDropCandidate {
                target: preview_target,
                scope: DockDropScope::Area,
                side: crate::DockSplitSide::Left,
                anchor_widget_id,
                active: true,
            }],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &theme,
        &workspace,
        Some(&docking_visual_state),
    );
    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_overlay_widget_id(anchor_widget_id)
    ));

    let bounds = UiRect::new(0.0, 0.0, 960.0, 640.0);
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        bounds,
        &ui_runtime::UiRuntimeState::default(),
    );
    let anchor = layouts
        .get(&anchor_widget_id)
        .expect("target tab stack should have layout");
    let preview = layouts
        .get(&dock_split_preview_panel_widget_id(anchor_widget_id))
        .expect("preview slice should have layout");
    assert!((preview.bounds.x - anchor.bounds.x).abs() <= 0.001);
    assert!((preview.bounds.height - anchor.bounds.height).abs() <= 0.001);
    assert!(preview.bounds.width > 0.0 && preview.bounds.width <= 132.0);
}

#[test]
fn dock_root_split_preview_spans_target_root_edge() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let theme = ThemeTokens::default();
    let anchor_widget_id = workspace_split_host_widget_id(workspace.root_host_id());
    let preview_target = DockingPreviewDropTarget::SplitIntoRoot {
        side: crate::DockSplitSide::Bottom,
    };
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(preview_target),
            preview_candidates: vec![DockDropCandidate {
                target: preview_target,
                scope: DockDropScope::Workspace,
                side: crate::DockSplitSide::Bottom,
                anchor_widget_id,
                active: true,
            }],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &theme,
        &workspace,
        Some(&docking_visual_state),
    );
    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_overlay_widget_id(anchor_widget_id)
    ));

    let bounds = UiRect::new(0.0, 0.0, 960.0, 640.0);
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        bounds,
        &ui_runtime::UiRuntimeState::default(),
    );
    let anchor = layouts
        .get(&anchor_widget_id)
        .expect("root target should have layout");
    let preview = layouts
        .get(&dock_split_preview_panel_widget_id(anchor_widget_id))
        .expect("root preview slice should have layout");
    assert!(
        (preview.bounds.y + preview.bounds.height - (anchor.bounds.y + anchor.bounds.height)).abs()
            <= 0.001
    );
    assert!((preview.bounds.width - anchor.bounds.width).abs() <= 0.001);
    assert!(preview.bounds.height > 0.0 && preview.bounds.height <= 42.0);
}

#[test]
fn dock_scope_previews_render_all_candidates_and_active_label() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let theme = ThemeTokens::default();
    let area_anchor = tab_stack_container_widget_id(viewport_stack);
    let group_anchor = crate::CENTER_RIGHT_SPLIT_WIDGET_ID;
    let workspace_anchor = crate::BODY_ROOT_WIDGET_ID;
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(DockingPreviewDropTarget::SplitIntoHost {
                target_host_id: workspace.root_host_id(),
                side: crate::DockSplitSide::Right,
            }),
            preview_candidates: vec![
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoArea {
                        target_tab_stack_id: viewport_stack,
                        side: crate::DockSplitSide::Right,
                    },
                    scope: DockDropScope::Area,
                    side: crate::DockSplitSide::Right,
                    anchor_widget_id: area_anchor,
                    active: false,
                },
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoHost {
                        target_host_id: workspace.root_host_id(),
                        side: crate::DockSplitSide::Right,
                    },
                    scope: DockDropScope::Group,
                    side: crate::DockSplitSide::Right,
                    anchor_widget_id: group_anchor,
                    active: true,
                },
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoRoot {
                        side: crate::DockSplitSide::Right,
                    },
                    scope: DockDropScope::Workspace,
                    side: crate::DockSplitSide::Right,
                    anchor_widget_id: workspace_anchor,
                    active: false,
                },
            ],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &theme,
        &workspace,
        Some(&docking_visual_state),
    );

    for anchor in [area_anchor, group_anchor, workspace_anchor] {
        assert!(ui_tree_contains_widget(
            &build.tree.root,
            dock_split_preview_overlay_widget_id(anchor)
        ));
    }
    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_label_widget_id(group_anchor)
    ));
    assert!(!ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_label_widget_id(area_anchor)
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

fn mapped_surface_actions_for_route(
    panel_kind: PanelKind,
    widget_id: WidgetId,
    action: SurfaceLocalAction,
    interactions: Vec<UiInteraction>,
) -> Vec<SurfaceLocalAction> {
    let workspace = sample_workspace_state();
    let (_, surface_id) = panel_and_surface_by_kind(&workspace, panel_kind);
    let frame_model = frame_model_with_surface_route(&workspace, surface_id, widget_id, action);
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);

    map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: interactions,
        },
        &build.projection_artifacts,
    )
    .into_iter()
    .map(|command| match command {
        ShellCommand::DispatchSurfaceLocalAction { action, .. } => action,
        other => panic!("expected surface dispatch command, got {other:?}"),
    })
    .collect()
}

fn keyboard_event(key: Key) -> KeyboardEvent {
    KeyboardEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
    }
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

fn button_enabled(node: &crate::UiNode, widget_id: WidgetId) -> bool {
    if node.id == widget_id {
        if let crate::UiNodeKind::Button(button) = &node.kind {
            return button.enabled;
        }
        return false;
    }
    node.children
        .iter()
        .any(|child| button_enabled(child, widget_id))
}
