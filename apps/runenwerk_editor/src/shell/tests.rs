use editor_core::{
    ChangeOrigin, ComponentTypeId, EntityId, SelectionTarget, SessionChangeKind, WorkflowEventKind,
};
use editor_inspector::{InspectorEditValue, InspectorPath};
use editor_shell::{
    CENTER_RIGHT_SPLIT_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
    ENTITY_TABLE_LIST_WIDGET_ID, ENTITY_TABLE_PANEL_WIDGET_ID, EditorDomainMutation,
    EntityTableComponentFilter, EntityTableHierarchyFilter, EntityTableSessionMutation,
    FLOATING_DROP_ZONE_WIDGET_ID, InspectorSessionMutation, LEFT_RIGHT_SPLIT_WIDGET_ID,
    MODELLING_WORKSPACE_PROFILE_ID, OutlinerDomainMutation, PanelKind, SCENE_WORKSPACE_PROFILE_ID,
    ShellCommand, StructuralCommandTarget, SurfaceLocalAction, SurfaceProviderAvailability,
    SurfaceProviderId, SurfaceSessionMutation, ToolSurfaceKind, ToolbarCommandKind,
    ToolbarMenuKind, UiInteraction, UiInteractionResults, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
    VIEWPORT_PANEL_WIDGET_ID, ViewportDomainMutation, ViewportSessionMutation,
    ViewportSurfaceAction, WorkspaceMutation, WorkspaceSplitAxis,
    map_interactions_to_shell_commands, outliner_row_widget_id, workspace_split_host_widget_id,
};
use editor_viewport::{
    ArtifactObservationFrame, ExpressionProductId, ProducerHealth, ProductAvailabilityState,
    ViewportDebugStage, ViewportHitResult, ViewportId, ViewportPresentationState,
};
use engine::plugins::render::UiFontAtlasResource;
use ui_input::{Modifiers, PointerButton, PointerEvent, PointerEventKind, UiInputEvent};
use ui_math::{UiPoint, UiRect, UiVector};
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{
    EntityTablePanelPresenter, EntityTablePanelUiState, ViewportPanelCommand,
};
use crate::editor_runtime::select_single_entity;
use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRecord, ToolSurfaceRuntimeBindingRegistryResource,
    ViewportArtifactObservationResource, ViewportInstanceRegistryResource,
    ViewportPresentationStateResource, ViewportRenderStateCommand,
    ViewportRenderStateCommandQueueResource,
};
use crate::shell::{
    EditorSurfaceProviderRegistry, RunenwerkEditorShellController, RunenwerkEditorShellState,
    SELECT_TOOL_ID, TRANSLATE_TOOL_ID, active_document_context, build_editor_shell_frame_model,
    dispatch_shell_command, dispatch_shell_command_with_viewport_commands,
};

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct TestMarker;

#[derive(Debug, Clone, Default, ecs::Component, ecs::ReflectComponent)]
struct QueryMarker {
    value: i32,
}

fn simple_test_template(id: &str) -> ui_definition::AuthoredUiTemplate {
    ui_definition::AuthoredUiTemplate {
        id: id.into(),
        root: ui_definition::UiNodeDefinition::Panel {
            id: "root".into(),
            children: vec![ui_definition::UiNodeDefinition::Label {
                id: "label".into(),
                label: ui_definition::UiValueBinding::static_text("Test"),
                availability: None,
            }],
            availability: None,
        },
        templates: Vec::new(),
        menus: Vec::new(),
    }
}

fn test_tool_surface_binding_registry(
    tool_surface: editor_shell::ToolSurfaceInstanceId,
    panel: editor_shell::PanelInstanceId,
    tab_stack: editor_shell::TabStackId,
    viewport: ViewportId,
) -> ToolSurfaceRuntimeBindingRegistryResource {
    let mut registry = ToolSurfaceRuntimeBindingRegistryResource::default();
    registry.upsert_binding(ToolSurfaceRuntimeBindingRecord {
        tool_surface_id: tool_surface,
        panel_instance_id: panel,
        tab_stack_id: tab_stack,
        viewport_id: viewport,
        host_widget_id: editor_shell::WidgetId(999),
        bounds: UiRect::new(0.0, 0.0, 640.0, 360.0),
        generation: 1,
    });
    registry
}

fn surface_session_command(
    target: StructuralCommandTarget,
    mutation: SurfaceSessionMutation,
    projection_epoch: u64,
) -> ShellCommand {
    ShellCommand::ApplySurfaceSessionMutation {
        target,
        mutation,
        projection_epoch,
    }
}

fn editor_domain_command(
    target: StructuralCommandTarget,
    mutation: EditorDomainMutation,
    projection_epoch: u64,
) -> ShellCommand {
    ShellCommand::ApplyEditorDomainMutation {
        target,
        mutation,
        projection_epoch,
    }
}

#[test]
fn dispatch_shell_command_updates_active_tool() {
    let mut app = RunenwerkEditorApp::new();

    dispatch_shell_command(
        &mut app,
        None,
        ShellCommand::ActivateSelectTool,
        None,
        None,
        None,
        None,
    )
    .expect("select tool command should succeed");
    assert_eq!(app.runtime().session().active_tool(), Some(SELECT_TOOL_ID));

    dispatch_shell_command(
        &mut app,
        None,
        ShellCommand::ActivateTranslateTool,
        None,
        None,
        None,
        None,
    )
    .expect("translate tool command should succeed");
    assert_eq!(
        app.runtime().session().active_tool(),
        Some(TRANSLATE_TOOL_ID)
    );
}

#[test]
fn dispatch_shell_command_applies_and_rolls_back_selected_editor_definition() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ApplySelectedEditorDefinition,
        None,
        None,
        None,
        None,
    )
    .expect("selected definition fixture should apply");

    assert_eq!(shell_state.self_authoring().applied_count(), 1);

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::RollbackSelectedEditorDefinition,
        None,
        None,
        None,
        None,
    )
    .expect("selected applied definition should rollback");

    assert_eq!(shell_state.self_authoring().applied_count(), 0);
}

#[test]
fn dispatch_shell_command_edits_selected_ui_and_theme_definition_drafts() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let node_id = shell_state
        .self_authoring()
        .selected_ui_node_id()
        .expect("selected UI definition should expose an editable node")
        .to_string();

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SetSelectedEditorDefinitionUiNodeText {
            node_id,
            text: "Edited by command".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("selected UI definition node text edit should succeed");
    assert!(
        shell_state
            .self_authoring()
            .formed_selected_preview(&ThemeTokens::default())
            .is_some()
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SelectEditorDefinitionDocument {
            document_id: "runenwerk.editor.theme.default".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("theme document selection should succeed");
    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SetSelectedEditorThemeColor {
            token: "accent".to_string(),
            value: "#2244ff".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("theme color draft edit should succeed");
    let selected = shell_state
        .self_authoring()
        .selected_document()
        .expect("theme document should remain selected");
    let editor_definition::EditorDefinitionDocumentContent::Theme(theme) = &selected.content else {
        panic!("selected document should be a theme definition");
    };
    assert_eq!(
        theme.colors.get("accent").map(String::as_str),
        Some("#2244ff")
    );
}

#[test]
fn applying_selected_theme_definition_produces_live_theme_activation() {
    let mut host = EditorHostResource::default();

    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::SelectEditorDefinitionDocument {
            document_id: "runenwerk.editor.theme.default".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("theme document selection should succeed");
    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::SetSelectedEditorThemeColor {
            token: "accent".to_string(),
            value: "#3366ff".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("theme color draft edit should succeed");
    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::ApplySelectedEditorDefinition,
        None,
        None,
        None,
        None,
    )
    .expect("theme definition apply should succeed");

    assert_eq!(host.app.pending_editor_definition_activation_count(), 1);

    let activated = host.apply_pending_editor_definition_activations();

    assert_eq!(activated, 1);
    assert_eq!(
        host.theme.accent,
        ui_theme::UiColor::new(0.2, 0.4, 1.0, 1.0)
    );
}

#[test]
fn applying_selected_workspace_layout_definition_replaces_live_workspace() {
    let mut host = EditorHostResource::default();

    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::SelectEditorDefinitionDocument {
            document_id: "runenwerk.editor.layout.editor_design".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("workspace layout definition selection should succeed");
    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::AddSelectedEditorWorkspaceLayoutTab {
            label: "Validation".to_string(),
            tool_surface: "definition_validation".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("workspace layout tab edit should succeed");
    dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::ApplySelectedEditorDefinition,
        None,
        None,
        None,
        None,
    )
    .expect("workspace layout definition apply should succeed");

    assert_eq!(host.app.pending_editor_definition_activation_count(), 1);

    let activated = host.apply_pending_editor_definition_activations();

    assert_eq!(activated, 1);
    host.shell_state
        .workspace_state()
        .validate_integrity()
        .expect("activated authored workspace layout should remain structurally valid");
    assert!(
        host.shell_state
            .workspace_state()
            .panels()
            .any(|panel| { panel.panel_kind == editor_shell::PanelKind::DefinitionValidation })
    );
    assert!(
        !host
            .shell_state
            .workspace_state()
            .panels()
            .any(|panel| panel.panel_kind == editor_shell::PanelKind::Viewport),
        "live workspace layout activation should replace the previous scene layout"
    );
}

#[test]
fn applying_ui_template_definition_updates_live_template_catalog() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let document = editor_definition::EditorDefinitionDocument::current(
        editor_definition::EditorDefinitionId::from("runenwerk.editor.test.template"),
        "test_template.ron",
        editor_definition::EditorDefinitionDocumentKind::UiLayout,
        editor_definition::EditorDefinitionDocumentContent::UiTemplate(simple_test_template(
            "runenwerk.editor.test.template",
        )),
    );

    shell_state
        .self_authoring_mut()
        .create_document(document)
        .expect("test UI template document should be accepted");

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ApplySelectedEditorDefinition,
        None,
        None,
        None,
        None,
    )
    .expect("UI template definition apply should succeed");

    let mut host = EditorHostResource {
        app,
        shell_state,
        theme: ThemeTokens::default(),
    };
    assert_eq!(host.app.pending_editor_definition_activation_count(), 1);

    let activated = host.apply_pending_editor_definition_activations();

    assert_eq!(activated, 1);
    assert!(
        host.shell_state
            .active_editor_definitions()
            .templates()
            .contains_key(&"runenwerk.editor.test.template".into())
    );
}

#[test]
fn applying_menu_shortcut_and_command_binding_definitions_updates_live_catalogs() {
    let mut host = EditorHostResource::default();
    host.app.queue_editor_definition_activation(
        editor_definition::EditorDefinitionDocument::current(
            editor_definition::EditorDefinitionId::from("runenwerk.editor.test.menu"),
            "test_menu.ron",
            editor_definition::EditorDefinitionDocumentKind::Menu,
            editor_definition::EditorDefinitionDocumentContent::Menu(
                editor_definition::EditorMenuDefinition {
                    id: "runenwerk.editor.test.menu".to_string(),
                    label: "Test Menu".to_string(),
                    items: Vec::new(),
                },
            ),
        ),
    );
    host.app.queue_editor_definition_activation(
        editor_definition::EditorDefinitionDocument::current(
            editor_definition::EditorDefinitionId::from("runenwerk.editor.test.shortcuts"),
            "test_shortcuts.ron",
            editor_definition::EditorDefinitionDocumentKind::Shortcut,
            editor_definition::EditorDefinitionDocumentContent::Shortcuts(
                editor_definition::EditorShortcutSetDefinition {
                    id: "runenwerk.editor.test.shortcuts".to_string(),
                    label: "Test Shortcuts".to_string(),
                    shortcuts: vec![editor_definition::EditorShortcutDefinition {
                        id: "test_apply".to_string(),
                        command: "editor.definition.apply_selected".to_string(),
                        chord: "Cmd+Shift+T".to_string(),
                        context: Some("editor-design".to_string()),
                    }],
                },
            ),
        ),
    );
    host.app.queue_editor_definition_activation(
        editor_definition::EditorDefinitionDocument::current(
            editor_definition::EditorDefinitionId::from("runenwerk.editor.test.commands"),
            "test_commands.ron",
            editor_definition::EditorDefinitionDocumentKind::CommandBinding,
            editor_definition::EditorDefinitionDocumentContent::CommandBindings(
                editor_definition::EditorCommandBindingSetDefinition {
                    id: "runenwerk.editor.test.commands".to_string(),
                    label: "Test Commands".to_string(),
                    bindings: vec![editor_definition::EditorCommandBindingDefinition {
                        id: "test_apply".to_string(),
                        command: "editor.definition.apply_selected".to_string(),
                        route_target: "self-authoring.apply-selected".to_string(),
                        capability_requirements: Vec::new(),
                        undoable: true,
                    }],
                },
            ),
        ),
    );

    let activated = host.apply_pending_editor_definition_activations();

    assert_eq!(activated, 3);
    assert!(
        host.shell_state
            .active_editor_definitions()
            .menus()
            .contains_key("runenwerk.editor.test.menu")
    );
    assert!(
        host.shell_state
            .active_editor_definitions()
            .shortcuts()
            .contains_key("runenwerk.editor.test.shortcuts")
    );
    assert!(
        host.shell_state
            .active_editor_definitions()
            .command_bindings()
            .contains_key("runenwerk.editor.test.commands")
    );
    assert_eq!(
        host.shell_state
            .active_editor_definitions()
            .command_for_route_target("self-authoring.apply-selected"),
        Some("editor.definition.apply_selected"),
        "active command-binding catalogs should map authored route targets to app/domain command ids",
    );
    assert_eq!(
        host.shell_state
            .active_editor_definitions()
            .route_target_for_command("editor.definition.apply_selected"),
        Some("self-authoring.apply-selected")
    );
}

#[test]
fn panel_and_tool_surface_registry_activation_blocks_active_workspace_removals() {
    let mut host = EditorHostResource::default();
    let original_panel_registry = host
        .shell_state
        .active_editor_definitions()
        .panel_registry()
        .cloned();
    host.app.queue_editor_definition_activation(
        editor_definition::EditorDefinitionDocument::current(
            editor_definition::EditorDefinitionId::from("runenwerk.editor.test.panels.empty"),
            "empty_panels.ron",
            editor_definition::EditorDefinitionDocumentKind::PanelRegistry,
            editor_definition::EditorDefinitionDocumentContent::PanelRegistry(
                editor_definition::EditorPanelRegistryDefinition {
                    id: "runenwerk.editor.test.panels.empty".to_string(),
                    label: "Empty Panels".to_string(),
                    panels: Vec::new(),
                },
            ),
        ),
    );

    let activated = host.apply_pending_editor_definition_activations();

    assert_eq!(activated, 0);
    assert_eq!(
        host.shell_state
            .active_editor_definitions()
            .panel_registry()
            .cloned(),
        original_panel_registry
    );

    let original_tool_surface_registry = host
        .shell_state
        .active_editor_definitions()
        .tool_surface_registry()
        .cloned();
    host.app.queue_editor_definition_activation(
        editor_definition::EditorDefinitionDocument::current(
            editor_definition::EditorDefinitionId::from("runenwerk.editor.test.surfaces.empty"),
            "empty_surfaces.ron",
            editor_definition::EditorDefinitionDocumentKind::ToolSurfaceDefinition,
            editor_definition::EditorDefinitionDocumentContent::ToolSurfaceRegistry(
                editor_definition::EditorToolSurfaceRegistryDefinition {
                    id: "runenwerk.editor.test.surfaces.empty".to_string(),
                    label: "Empty Tool Surfaces".to_string(),
                    tool_surfaces: Vec::new(),
                },
            ),
        ),
    );

    let activated = host.apply_pending_editor_definition_activations();

    assert_eq!(activated, 0);
    assert_eq!(
        host.shell_state
            .active_editor_definitions()
            .tool_surface_registry()
            .cloned(),
        original_tool_surface_registry
    );
}

#[test]
fn tool_surface_registry_activation_updates_future_creation_surface_kinds() {
    let mut host = EditorHostResource::default();
    host.app.queue_editor_definition_activation(
        editor_definition::EditorDefinitionDocument::current(
            editor_definition::EditorDefinitionId::from("runenwerk.editor.test.surfaces.extended"),
            "extended_surfaces.ron",
            editor_definition::EditorDefinitionDocumentKind::ToolSurfaceDefinition,
            editor_definition::EditorDefinitionDocumentContent::ToolSurfaceRegistry(
                editor_definition::EditorToolSurfaceRegistryDefinition {
                    id: "runenwerk.editor.test.surfaces.extended".to_string(),
                    label: "Extended Tool Surfaces".to_string(),
                    tool_surfaces: vec![
                        test_tool_surface_definition("outliner", "Outliner"),
                        test_tool_surface_definition("entity_table", "Entity Table"),
                        test_tool_surface_definition("viewport", "Viewport"),
                        test_tool_surface_definition("inspector", "Inspector"),
                        test_tool_surface_definition("console", "Console"),
                        test_tool_surface_definition("definition_validation", "Validation"),
                    ],
                },
            ),
        ),
    );

    let activated = host.apply_pending_editor_definition_activations();
    let frame_model = build_editor_shell_frame_model(
        &host.app,
        &host.shell_state,
        host.app.surface_provider_registry(),
        &host.theme,
        None,
        None,
        None,
    );

    assert_eq!(activated, 1);
    assert_eq!(
        frame_model.available_tool_surface_kinds,
        vec![
            ToolSurfaceKind::Outliner,
            ToolSurfaceKind::EntityTable,
            ToolSurfaceKind::Viewport,
            ToolSurfaceKind::Inspector,
            ToolSurfaceKind::Console,
            ToolSurfaceKind::DefinitionValidation,
        ],
        "activated tool-surface registry should feed future switch/create choices",
    );
}

fn test_tool_surface_definition(
    id: &str,
    label: &str,
) -> editor_definition::EditorToolSurfaceDefinition {
    editor_definition::EditorToolSurfaceDefinition {
        id: id.to_string(),
        label: label.to_string(),
        provider_family: "runenwerk.editor".to_string(),
        required_capabilities: Vec::new(),
        allowed_document_kinds: Vec::new(),
        allowed_workspace_profiles: Vec::new(),
    }
}

#[test]
fn dispatch_shell_command_edits_authored_workspace_layout_drafts() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SelectEditorDefinitionDocument {
            document_id: "runenwerk.editor.layout.editor_design".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("workspace layout definition selection should succeed");
    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::AddSelectedEditorWorkspaceLayoutTab {
            label: "Validation".to_string(),
            tool_surface: "definition_validation".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("workspace layout tab edit should succeed");
    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SplitSelectedEditorWorkspaceLayoutRoot {
            axis: "horizontal".to_string(),
        },
        None,
        None,
        None,
        None,
    )
    .expect("workspace layout split edit should succeed");

    let selected = shell_state
        .self_authoring()
        .selected_document()
        .expect("workspace layout should remain selected");
    let editor_definition::EditorDefinitionDocumentContent::WorkspaceLayout(layout) =
        &selected.content
    else {
        panic!("selected document should be a workspace layout");
    };
    assert!(matches!(
        layout.root,
        editor_definition::EditorWorkspaceHostDefinition::Split { .. }
    ));
}

#[test]
fn dispatch_shell_command_handles_toolbar_menu_and_workspace_commands() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let tab_stack_id = editor_shell::TabStackId::try_from_raw(1).unwrap();
    assert!(
        !shell_state
            .open_workspace_profile_ids()
            .contains(&EDITOR_DESIGN_WORKSPACE_PROFILE_ID),
        "Editor Design should be added to the workspace row only after the plus-menu activation"
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ToggleToolbarMenu {
            menu: ToolbarMenuKind::File,
        },
        None,
        None,
        None,
        None,
    )
    .expect("toolbar menu command should succeed");
    assert_eq!(
        shell_state.active_toolbar_menu(),
        Some(ToolbarMenuKind::File)
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ToggleTabStackActionMenu {
            tab_stack_id,
            anchor_widget_id: editor_shell::WidgetId(1),
        },
        None,
        None,
        None,
        None,
    )
    .expect("tab stack action menu command should succeed");
    assert_eq!(shell_state.active_toolbar_menu(), None);
    assert_eq!(
        shell_state.active_tab_stack_action_menu(),
        Some(tab_stack_id)
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ToggleToolbarMenu {
            menu: ToolbarMenuKind::Workspace,
        },
        None,
        None,
        None,
        None,
    )
    .expect("workspace menu command should succeed");
    assert_eq!(
        shell_state.active_toolbar_menu(),
        Some(ToolbarMenuKind::Workspace)
    );
    assert_eq!(shell_state.active_tab_stack_action_menu(), None);

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::RunToolbarCommand {
            command: ToolbarCommandKind::NextWorkspace,
        },
        None,
        None,
        None,
        None,
    )
    .expect("next workspace command should succeed");
    assert_eq!(
        shell_state.active_workspace_profile_id(),
        MODELLING_WORKSPACE_PROFILE_ID
    );
    assert_eq!(shell_state.active_toolbar_menu(), None);

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::RunToolbarCommand {
            command: ToolbarCommandKind::PreviousWorkspace,
        },
        None,
        None,
        None,
        None,
    )
    .expect("previous workspace command should succeed");
    assert_eq!(
        shell_state.active_workspace_profile_id(),
        SCENE_WORKSPACE_PROFILE_ID
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SwitchWorkspaceProfile {
            profile_id: MODELLING_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("explicit workspace switch command should succeed");
    assert_eq!(
        shell_state.active_workspace_profile_id(),
        MODELLING_WORKSPACE_PROFILE_ID
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SwitchWorkspaceProfile {
            profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("editor design workspace switch command should succeed");
    assert_eq!(
        shell_state.active_workspace_profile_id(),
        EDITOR_DESIGN_WORKSPACE_PROFILE_ID
    );
    assert!(
        shell_state
            .open_workspace_profile_ids()
            .contains(&EDITOR_DESIGN_WORKSPACE_PROFILE_ID),
        "activating Editor Design from the plus menu should add it to the open workspace row"
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::CloseWorkspaceProfile {
            profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("closing the active editor design workspace should switch to a remaining workspace");
    assert_eq!(
        shell_state.active_workspace_profile_id(),
        MODELLING_WORKSPACE_PROFILE_ID,
        "closing the active workspace should select the nearest remaining workspace"
    );
    assert!(
        !shell_state
            .open_workspace_profile_ids()
            .contains(&EDITOR_DESIGN_WORKSPACE_PROFILE_ID),
        "closed workspace should be removed from the workspace row"
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::CloseWorkspaceProfile {
            profile_id: MODELLING_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("closing a workspace should keep one fallback open");
    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::CloseWorkspaceProfile {
            profile_id: SCENE_WORKSPACE_PROFILE_ID,
        },
        None,
        None,
        None,
        None,
    )
    .expect("closing the last workspace should be ignored");
    assert_eq!(
        shell_state.open_workspace_profile_ids(),
        &[SCENE_WORKSPACE_PROFILE_ID],
        "workspace close must never leave the shell with zero open workspaces"
    );
}

#[test]
fn default_startup_resolves_scene_surface_providers() {
    let app = RunenwerkEditorApp::new();
    let shell_state = RunenwerkEditorShellState::new();
    assert!(matches!(
        active_document_context(&app),
        editor_shell::SurfaceDocumentContext::Resolved {
            document_kind: editor_core::DocumentKind::Scene,
            ..
        }
    ));

    let frame_model = build_editor_shell_frame_model(
        &app,
        &shell_state,
        app.surface_provider_registry(),
        &ThemeTokens::default(),
        None,
        None,
        None,
    );

    for kind in [
        PanelKind::Outliner,
        PanelKind::EntityTable,
        PanelKind::Viewport,
        PanelKind::Inspector,
    ] {
        let surface = surface_id_by_kind(shell_state.workspace_state(), kind);
        let frame = frame_model
            .surface(surface)
            .expect("mounted scene surface should resolve a frame");
        assert_eq!(
            frame.availability,
            SurfaceProviderAvailability::Available,
            "{kind:?} should not render unsupported document on default startup",
        );
    }
}

#[test]
fn scene_load_reset_keeps_active_scene_document_for_provider_frames() {
    let app = {
        let mut app = RunenwerkEditorApp::new();
        app.runtime_mut().prepare_for_scene_load();
        app
    };
    let shell_state = RunenwerkEditorShellState::new();
    assert!(matches!(
        active_document_context(&app),
        editor_shell::SurfaceDocumentContext::Resolved {
            document_kind: editor_core::DocumentKind::Scene,
            ..
        }
    ));

    let frame_model = build_editor_shell_frame_model(
        &app,
        &shell_state,
        app.surface_provider_registry(),
        &ThemeTokens::default(),
        None,
        None,
        None,
    );
    let viewport_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    assert_eq!(
        frame_model
            .surface(viewport_surface)
            .map(|frame| frame.availability.clone()),
        Some(SurfaceProviderAvailability::Available),
        "scene load reset should not leave scene providers in no-active-document state",
    );
}

#[test]
fn dispatch_shell_command_selects_outliner_entity() {
    let mut app = RunenwerkEditorApp::new();
    let ecs_entity = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(1), ecs_entity, "Player", None);

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            StructuralCommandTarget {
                panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
                active_tool_surface: Some(
                    editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
                ),
                tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
            },
            EditorDomainMutation::Outliner(OutlinerDomainMutation::SelectEntity {
                entity: EntityId(1),
            }),
            0,
        ),
        None,
        None,
        None,
        None,
    )
    .expect("outliner select shell command should succeed");

    assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));
    assert_eq!(
        app.runtime().session().selection().primary(),
        Some(&SelectionTarget::Entity(EntityId(1)))
    );
    assert!(matches!(
        app.runtime()
            .session_change_log()
            .last()
            .map(|change| (change.origin, change.kind.clone())),
        Some((
            ChangeOrigin::OutlinerPanel,
            SessionChangeKind::SelectionSetSingle {
                target: SelectionTarget::Entity(EntityId(1))
            }
        ))
    ));
}

#[test]
fn entity_table_row_interaction_selects_entity_with_structural_target() {
    let mut app = RunenwerkEditorApp::new();
    let alpha = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(1), alpha, "Alpha", None);
    let beta = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(2), beta, "Beta", None);

    let mut shell_state = RunenwerkEditorShellState::new();
    let (entity_table_panel, entity_table_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::EntityTable);
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::SetTabStackActivePanel {
            tab_stack_id: entity_table_stack,
            active_panel: Some(entity_table_panel),
        })
        .expect("entity table tab should activate");

    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);

    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached")
        .clone();
    let context = artifacts
        .workspace
        .widget_context_by_id
        .get(&ENTITY_TABLE_PANEL_WIDGET_ID)
        .copied()
        .expect("entity table panel should have a structural context");
    assert_eq!(context.panel_instance_id, entity_table_panel);

    let interactions = UiInteractionResults {
        items: vec![UiInteraction::TableRowSelected {
            target: ENTITY_TABLE_LIST_WIDGET_ID,
            row_index: 0,
        }],
    };
    let commands = map_interactions_to_shell_commands(&interactions, &artifacts);
    assert!(matches!(
        commands.as_slice(),
        [ShellCommand::DispatchSurfaceLocalAction {
            tool_surface_instance_id,
            target,
            projection_epoch,
            ..
        }] if *tool_surface_instance_id == context.active_tool_surface.expect("active surface")
            && target.panel_instance_id == entity_table_panel
            && target.tab_stack_id == entity_table_stack
            && *projection_epoch == artifacts.projection_epoch
    ));

    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    RunenwerkEditorShellController::dispatch_commands(
        &mut app,
        &mut shell_state,
        commands,
        &registry,
        None,
        None,
        None,
    )
    .expect("entity table provider-local command should dispatch");

    assert_eq!(
        app.runtime().session().selection().primary(),
        Some(&SelectionTarget::Entity(EntityId(1)))
    );
    assert!(matches!(
        app.runtime()
            .session_change_log()
            .last()
            .map(|change| (change.origin, change.kind.clone())),
        Some((
            ChangeOrigin::EntityTablePanel,
            SessionChangeKind::SelectionSetSingle {
                target: SelectionTarget::Entity(EntityId(1))
            }
        ))
    ));
}

#[test]
fn entity_table_query_filters_and_sorts_rows() {
    let mut app = RunenwerkEditorApp::new();
    let marker_type = ComponentTypeId(7001);
    app.runtime_mut()
        .register_component_type::<QueryMarker>(marker_type);

    let zeta = app.runtime_mut().spawn_world_entity(QueryMarker::default());
    app.runtime_mut()
        .register_entity(EntityId(1), zeta, "ZetaRoot", None);
    let child = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(2), child, "ChildAlpha", Some(EntityId(1)));
    let alpha = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(3), alpha, "AlphaRoot", None);
    select_single_entity(app.runtime_mut(), EntityId(1)).expect("selection should succeed");

    let mut query = EntityTablePanelUiState::new();
    query.set_search_query("alpha");
    let state = EntityTablePanelPresenter::build_state(app.runtime(), &query);
    assert_eq!(
        state.rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![EntityId(3), EntityId(2)]
    );

    query.set_search_query("");
    query.set_selected_only(true);
    let state = EntityTablePanelPresenter::build_state(app.runtime(), &query);
    assert_eq!(
        state.rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![EntityId(1)]
    );

    query.set_selected_only(false);
    query.set_hierarchy_filter(EntityTableHierarchyFilter::RootsOnly);
    let state = EntityTablePanelPresenter::build_state(app.runtime(), &query);
    assert_eq!(
        state.rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![EntityId(3), EntityId(1)]
    );

    query.set_hierarchy_filter(EntityTableHierarchyFilter::All);
    query.set_component_filter(EntityTableComponentFilter::Has(marker_type));
    let state = EntityTablePanelPresenter::build_state(app.runtime(), &query);
    assert_eq!(
        state.rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![EntityId(1)]
    );

    query.set_component_filter(EntityTableComponentFilter::All);
    query.set_hierarchy_filter(EntityTableHierarchyFilter::RootsOnly);
    query.toggle_sort(editor_shell::EntityTableSortKey::DisplayName);
    let state = EntityTablePanelPresenter::build_state(app.runtime(), &query);
    assert_eq!(
        state
            .rows
            .iter()
            .map(|row| row.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["ZetaRoot", "AlphaRoot"]
    );
}

#[test]
fn stale_provider_local_action_fails_closed_after_rebuild() {
    let mut app = RunenwerkEditorApp::new();
    let entity = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(1), entity, "Alpha", None);
    let mut shell_state = RunenwerkEditorShellState::new();
    let (entity_table_panel, entity_table_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::EntityTable);
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::SetTabStackActivePanel {
            tab_stack_id: entity_table_stack,
            active_panel: Some(entity_table_panel),
        })
        .expect("entity table tab should activate");

    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let stale_artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached")
        .clone();
    let stale_commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::TableRowSelected {
                target: ENTITY_TABLE_LIST_WIDGET_ID,
                row_index: 0,
            }],
        },
        &stale_artifacts,
    );
    assert!(matches!(
        stale_commands.as_slice(),
        [ShellCommand::DispatchSurfaceLocalAction { .. }]
    ));

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    assert!(stale_artifacts.projection_epoch < shell_state.current_projection_epoch());
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();

    RunenwerkEditorShellController::dispatch_commands(
        &mut app,
        &mut shell_state,
        stale_commands,
        &registry,
        None,
        None,
        None,
    )
    .expect("stale provider-local action should fail closed without mutation error");

    assert_eq!(app.runtime().session().selection().primary(), None);
}

#[test]
fn provider_id_mismatch_on_local_action_is_rejected_without_mutation() {
    let mut app = RunenwerkEditorApp::new();
    let entity = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(1), entity, "Alpha", None);
    let mut shell_state = RunenwerkEditorShellState::new();
    let (entity_table_panel, entity_table_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::EntityTable);
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::SetTabStackActivePanel {
            tab_stack_id: entity_table_stack,
            active_panel: Some(entity_table_panel),
        })
        .expect("entity table tab should activate");

    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached")
        .clone();
    let mut commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::TableRowSelected {
                target: ENTITY_TABLE_LIST_WIDGET_ID,
                row_index: 0,
            }],
        },
        &artifacts,
    );
    let [ShellCommand::DispatchSurfaceLocalAction { provider_id, .. }] = commands.as_mut_slice()
    else {
        panic!("expected one provider-local action");
    };
    *provider_id = SurfaceProviderId::try_from_raw(999).unwrap();

    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let result = RunenwerkEditorShellController::dispatch_commands(
        &mut app,
        &mut shell_state,
        commands,
        &registry,
        None,
        None,
        None,
    );

    assert!(result.is_err());
    assert_eq!(app.runtime().session().selection().primary(), None);
}

#[test]
fn dispatch_shell_command_selects_viewport_product_when_available() {
    let mut app = RunenwerkEditorApp::new();
    let mut viewport_presentations = ViewportPresentationStateResource::default();
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let viewport_id = ViewportId(1);
    let product_id = ExpressionProductId(2);
    let target = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
        active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
    };
    let tool_surface_bindings = test_tool_surface_binding_registry(
        editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
        target.panel_instance_id,
        target.tab_stack_id,
        viewport_id,
    );
    let mut frame =
        ArtifactObservationFrame::new(viewport_id, app.runtime().current_scene_reality_version());
    frame
        .availability_by_product
        .insert(product_id, ProductAvailabilityState::Available);
    frame
        .producer_health_by_product
        .insert(product_id, ProducerHealth::Healthy);
    viewport_observations.upsert_frame(frame);

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                viewport_id,
                product_id,
            }),
            0,
        ),
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        None,
    )
    .expect("viewport product select shell command should succeed");

    assert_eq!(
        viewport_presentations
            .state_for(viewport_id)
            .map(|state| state.selected_primary_product_id),
        Some(product_id)
    );
}

#[test]
fn dispatch_shell_command_updates_only_target_viewport_product_selection() {
    let mut app = RunenwerkEditorApp::new();
    let mut viewport_presentations = ViewportPresentationStateResource::default();
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let viewport_a = ViewportId(1);
    let viewport_b = ViewportId(2);
    let product_scene = ExpressionProductId(1);
    let product_picking = ExpressionProductId(2);
    let target = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
        active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
    };
    let tool_surface_bindings = test_tool_surface_binding_registry(
        editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
        target.panel_instance_id,
        target.tab_stack_id,
        viewport_b,
    );
    viewport_presentations.upsert_state(ViewportPresentationState::new(viewport_a, product_scene));
    viewport_presentations.upsert_state(ViewportPresentationState::new(viewport_b, product_scene));

    for viewport_id in [viewport_a, viewport_b] {
        let mut frame = ArtifactObservationFrame::new(
            viewport_id,
            app.runtime().current_scene_reality_version(),
        );
        frame
            .availability_by_product
            .insert(product_picking, ProductAvailabilityState::Available);
        frame
            .producer_health_by_product
            .insert(product_picking, ProducerHealth::Healthy);
        viewport_observations.upsert_frame(frame);
    }

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                viewport_id: viewport_b,
                product_id: product_picking,
            }),
            0,
        ),
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        None,
    )
    .expect("viewport product select shell command should succeed");

    assert_eq!(
        viewport_presentations
            .state_for(viewport_a)
            .map(|state| state.selected_primary_product_id),
        Some(product_scene),
        "selection for viewport A should remain unchanged",
    );
    assert_eq!(
        viewport_presentations
            .state_for(viewport_b)
            .map(|state| state.selected_primary_product_id),
        Some(product_picking),
        "selection for viewport B should update independently",
    );
}

#[test]
fn dispatch_shell_command_viewport_product_fails_closed_without_runtime_binding() {
    let mut app = RunenwerkEditorApp::new();
    let mut viewport_presentations = ViewportPresentationStateResource::default();
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let viewport_id = ViewportId(1);
    let product_id = ExpressionProductId(2);
    let mut frame =
        ArtifactObservationFrame::new(viewport_id, app.runtime().current_scene_reality_version());
    frame
        .availability_by_product
        .insert(product_id, ProductAvailabilityState::Available);
    viewport_observations.upsert_frame(frame);

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            StructuralCommandTarget {
                panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
                active_tool_surface: Some(
                    editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
                ),
                tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
            },
            EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                viewport_id,
                product_id,
            }),
            0,
        ),
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
        None,
        None,
    )
    .expect("missing binding should fail closed without raising mutation error");

    assert!(
        viewport_presentations.state_for(viewport_id).is_none(),
        "without runtime binding registry, structural viewport command must not mutate selection",
    );
}

#[test]
fn dispatch_shell_command_viewport_product_rejects_stale_binding_viewport_mismatch() {
    let mut app = RunenwerkEditorApp::new();
    let mut viewport_presentations = ViewportPresentationStateResource::default();
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let requested_viewport = ViewportId(1);
    let rebound_viewport = ViewportId(2);
    let product_id = ExpressionProductId(2);
    let target = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
        active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
    };
    let tool_surface_bindings = test_tool_surface_binding_registry(
        editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
        target.panel_instance_id,
        target.tab_stack_id,
        rebound_viewport,
    );

    for viewport_id in [requested_viewport, rebound_viewport] {
        let mut frame = ArtifactObservationFrame::new(
            viewport_id,
            app.runtime().current_scene_reality_version(),
        );
        frame
            .availability_by_product
            .insert(product_id, ProductAvailabilityState::Available);
        viewport_observations.upsert_frame(frame);
    }

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                viewport_id: requested_viewport,
                product_id,
            }),
            0,
        ),
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        None,
    )
    .expect("stale binding mismatch should fail closed without raising mutation error");

    assert!(
        viewport_presentations
            .state_for(requested_viewport)
            .is_none(),
        "requested viewport selection should not be updated on stale binding mismatch",
    );
    assert!(
        viewport_presentations.state_for(rebound_viewport).is_none(),
        "rebound viewport should not be implicitly mutated by stale command",
    );
}

#[test]
fn dispatch_shell_command_viewport_product_requires_structural_tool_surface_target() {
    let mut app = RunenwerkEditorApp::new();
    let mut viewport_presentations = ViewportPresentationStateResource::default();
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let viewport_id = ViewportId(1);
    let product_id = ExpressionProductId(2);
    let mut frame =
        ArtifactObservationFrame::new(viewport_id, app.runtime().current_scene_reality_version());
    frame
        .availability_by_product
        .insert(product_id, ProductAvailabilityState::Available);
    viewport_observations.upsert_frame(frame);

    let tool_surface_bindings = test_tool_surface_binding_registry(
        editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
        editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
        editor_shell::TabStackId::try_from_raw(1).unwrap(),
        viewport_id,
    );

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            StructuralCommandTarget {
                panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
                active_tool_surface: None,
                tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
            },
            EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                viewport_id,
                product_id,
            }),
            0,
        ),
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        None,
    )
    .expect("missing structural tool-surface target should fail closed");

    assert!(
        viewport_presentations.state_for(viewport_id).is_none(),
        "viewport selection must not mutate when structural tool surface is absent",
    );
}

#[test]
fn dispatch_shell_command_viewport_product_rejects_structural_binding_mismatch() {
    let mut app = RunenwerkEditorApp::new();
    let mut viewport_presentations = ViewportPresentationStateResource::default();
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let viewport_id = ViewportId(1);
    let product_id = ExpressionProductId(2);
    let mut frame =
        ArtifactObservationFrame::new(viewport_id, app.runtime().current_scene_reality_version());
    frame
        .availability_by_product
        .insert(product_id, ProductAvailabilityState::Available);
    viewport_observations.upsert_frame(frame);

    let target = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(7).unwrap(),
        active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(8).unwrap(),
    };
    let tool_surface_bindings = test_tool_surface_binding_registry(
        editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
        editor_shell::PanelInstanceId::try_from_raw(99).unwrap(),
        editor_shell::TabStackId::try_from_raw(100).unwrap(),
        viewport_id,
    );

    dispatch_shell_command(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SelectProduct {
                viewport_id,
                product_id,
            }),
            0,
        ),
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        None,
    )
    .expect("structural binding mismatch should fail closed");

    assert!(
        viewport_presentations.state_for(viewport_id).is_none(),
        "viewport selection must not mutate when structural binding mismatches runtime mapping",
    );
}

#[test]
fn dispatch_shell_command_enqueues_viewport_state_commands_for_bound_viewport() {
    let mut app = RunenwerkEditorApp::new();
    let requested_viewport = ViewportId(1);
    let rebound_viewport = ViewportId(2);
    let target = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(1).unwrap(),
        active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(1).unwrap(),
    };
    let tool_surface_bindings = test_tool_surface_binding_registry(
        target.active_tool_surface.unwrap(),
        target.panel_instance_id,
        target.tab_stack_id,
        rebound_viewport,
    );
    let mut viewport_render_commands = ViewportRenderStateCommandQueueResource::default();

    dispatch_shell_command_with_viewport_commands(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::ResetCamera {
                viewport_id: rebound_viewport,
            }),
            0,
        ),
        None,
        None,
        Some(&tool_surface_bindings),
        Some(&mut viewport_render_commands),
        None,
    )
    .expect("bound viewport camera reset command should dispatch");
    dispatch_shell_command_with_viewport_commands(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SetDebugStage {
                viewport_id: rebound_viewport,
                debug_stage: ViewportDebugStage::PickingHitMiss,
            }),
            0,
        ),
        None,
        None,
        Some(&tool_surface_bindings),
        Some(&mut viewport_render_commands),
        None,
    )
    .expect("bound viewport debug command should dispatch");
    dispatch_shell_command_with_viewport_commands(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SetRootBackgroundOpaque {
                viewport_id: rebound_viewport,
                enabled: true,
            }),
            0,
        ),
        None,
        None,
        Some(&tool_surface_bindings),
        Some(&mut viewport_render_commands),
        None,
    )
    .expect("bound viewport root opacity command should dispatch");

    assert_eq!(
        viewport_render_commands.drain().collect::<Vec<_>>(),
        vec![
            ViewportRenderStateCommand::ResetCamera {
                viewport_id: rebound_viewport,
            },
            ViewportRenderStateCommand::SetDebugStage {
                viewport_id: rebound_viewport,
                debug_stage: ViewportDebugStage::PickingHitMiss,
            },
            ViewportRenderStateCommand::SetRootBackgroundOpaque {
                viewport_id: rebound_viewport,
                enabled: true,
            },
        ],
        "viewport state commands should be routed through the active runtime binding",
    );

    dispatch_shell_command_with_viewport_commands(
        &mut app,
        None,
        editor_domain_command(
            target,
            EditorDomainMutation::Viewport(ViewportDomainMutation::SetDebugStage {
                viewport_id: requested_viewport,
                debug_stage: ViewportDebugStage::PrimitiveAvailability,
            }),
            0,
        ),
        None,
        None,
        Some(&tool_surface_bindings),
        Some(&mut viewport_render_commands),
        None,
    )
    .expect("stale viewport command should fail closed without raising a mutation error");

    assert!(
        viewport_render_commands.is_empty(),
        "stale viewport command must not enqueue state changes for the rebound viewport",
    );
}

#[test]
fn dispatch_shell_command_toggles_viewport_details_visibility() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let (viewport_panel, viewport_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let viewport_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let target = StructuralCommandTarget {
        panel_instance_id: viewport_panel,
        active_tool_surface: Some(viewport_surface),
        tab_stack_id: viewport_stack,
    };
    assert!(
        !app.surface_sessions()
            .session(viewport_surface)
            .map(|session| session.viewport_details_visible)
            .unwrap_or(false)
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        surface_session_command(
            target,
            SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleDetails),
            0,
        ),
        None,
        None,
        None,
        None,
    )
    .expect("viewport details toggle shell command should succeed");
    assert!(
        app.surface_sessions()
            .session(viewport_surface)
            .map(|session| session.viewport_details_visible)
            .unwrap_or(false)
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        surface_session_command(
            target,
            SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleDetails),
            0,
        ),
        None,
        None,
        None,
        None,
    )
    .expect("viewport details toggle shell command should succeed");
    assert!(
        !app.surface_sessions()
            .session(viewport_surface)
            .map(|session| session.viewport_details_visible)
            .unwrap_or(false)
    );
}

#[test]
fn provider_local_viewport_details_toggle_uses_routed_surface_instance() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let viewport_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached")
        .clone();
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Toggled {
                target: VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
                checked: true,
            }],
        },
        &artifacts,
    );
    assert!(matches!(
        commands.as_slice(),
        [ShellCommand::DispatchSurfaceLocalAction {
            action: SurfaceLocalAction::Viewport(ViewportSurfaceAction::ToggleDetails),
            tool_surface_instance_id,
            ..
        }] if *tool_surface_instance_id == viewport_surface
    ));

    let registry = app.surface_provider_registry_handle();
    RunenwerkEditorShellController::dispatch_commands(
        &mut app,
        &mut shell_state,
        commands,
        registry.as_ref(),
        None,
        None,
        None,
    )
    .expect("provider-local viewport details toggle should dispatch");

    assert_eq!(
        app.surface_sessions()
            .session(viewport_surface)
            .map(|session| session.viewport_details_visible),
        Some(true)
    );
}

#[test]
fn editor_type_switch_replaces_mounted_surface_without_changing_panel_identity() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let (viewport_panel, _) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let before_surface = shell_state
        .workspace_state()
        .panel(viewport_panel)
        .and_then(|panel| panel.active_tool_surface)
        .expect("viewport panel should have active surface");

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SwitchPanelToolSurfaceKind {
            panel_instance_id: viewport_panel,
            tool_surface_kind: editor_shell::ToolSurfaceKind::Inspector,
            projection_epoch: 1,
        },
        None,
        None,
        None,
        Some(1),
    )
    .expect("editor type switch should dispatch");

    let panel = shell_state
        .workspace_state()
        .panel(viewport_panel)
        .expect("panel identity should remain");
    let after_surface = panel
        .active_tool_surface
        .expect("switched panel should mount new surface");
    assert_ne!(before_surface, after_surface);
    assert_eq!(
        shell_state
            .workspace_state()
            .tool_surface(after_surface)
            .map(|surface| surface.tool_surface_kind),
        Some(editor_shell::ToolSurfaceKind::Inspector)
    );
    assert_eq!(
        shell_state
            .workspace_state()
            .tool_surface(before_surface)
            .map(|surface| surface.mount),
        Some(editor_shell::ToolSurfaceMount::Unmounted)
    );
    assert!(
        app.surface_sessions().session(before_surface).is_none(),
        "switched-out surface session should be pruned"
    );
}

#[test]
fn editor_type_switch_uses_new_surface_identity_for_provider_artifacts_and_sessions() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let (viewport_panel, _) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let before_surface = shell_state
        .workspace_state()
        .panel(viewport_panel)
        .and_then(|panel| panel.active_tool_surface)
        .expect("viewport panel should have active surface");
    app.surface_sessions_mut()
        .session_mut(before_surface)
        .viewport_details_visible = true;

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SwitchPanelToolSurfaceKind {
            panel_instance_id: viewport_panel,
            tool_surface_kind: editor_shell::ToolSurfaceKind::Viewport,
            projection_epoch: 1,
        },
        None,
        None,
        None,
        Some(1),
    )
    .expect("same-kind editor type switch should dispatch through mounted surface seam");

    let after_surface = shell_state
        .workspace_state()
        .panel(viewport_panel)
        .and_then(|panel| panel.active_tool_surface)
        .expect("switched panel should mount a replacement surface");
    assert_ne!(before_surface, after_surface);
    assert!(
        app.surface_sessions().session(before_surface).is_none(),
        "old surface-local state should be pruned after replacement"
    );
    assert_eq!(
        app.surface_sessions()
            .session(after_surface)
            .map(|session| session.viewport_details_visible),
        None,
        "replacement surface should not inherit old viewport details state"
    );

    let frame_model = build_editor_shell_frame_model(
        &app,
        &shell_state,
        app.surface_provider_registry(),
        &ThemeTokens::default(),
        None,
        None,
        None,
    );
    let viewport_frame = frame_model
        .surface(after_surface)
        .expect("replacement viewport surface should resolve a provider frame");
    assert_eq!(viewport_frame.panel_instance_id, viewport_panel);
    assert_eq!(viewport_frame.title, "Viewport");
    assert_eq!(
        viewport_frame.availability,
        SurfaceProviderAvailability::Available
    );
    let mut viewport_observations = ViewportArtifactObservationResource::default();
    let observed_viewport_id = ViewportId(77);
    viewport_observations.upsert_frame(ArtifactObservationFrame::new(
        observed_viewport_id,
        app.runtime().current_scene_reality_version(),
    ));
    let mut viewport_instances = ViewportInstanceRegistryResource::default();
    viewport_instances.sync_from_workspace_state(shell_state.workspace_state());
    let frame_model = build_editor_shell_frame_model(
        &app,
        &shell_state,
        app.surface_provider_registry(),
        &ThemeTokens::default(),
        Some(&viewport_observations),
        None,
        Some(&viewport_instances),
    );
    let viewport_frame = frame_model
        .surface(after_surface)
        .expect("unbound replacement viewport surface should still resolve");
    let expected_viewport_id = viewport_instances
        .viewport_for_tool_surface(after_surface)
        .expect("replacement viewport surface should have explicit runtime viewport identity");
    assert!(
        ui_tree_contains_viewport_embed(&viewport_frame.artifact.root, expected_viewport_id),
        "unbound replacement viewport surface should use its own deterministic viewport id"
    );
    assert!(
        !ui_tree_contains_viewport_embed(&viewport_frame.artifact.root, observed_viewport_id),
        "unbound replacement viewport surface must not inherit an unrelated observed viewport id"
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::SwitchPanelToolSurfaceKind {
            panel_instance_id: viewport_panel,
            tool_surface_kind: editor_shell::ToolSurfaceKind::Placeholder,
            projection_epoch: 2,
        },
        None,
        None,
        None,
        Some(2),
    )
    .expect("unsupported editor type switch should still use mounted surface seam");

    let unsupported_surface = shell_state
        .workspace_state()
        .panel(viewport_panel)
        .and_then(|panel| panel.active_tool_surface)
        .expect("placeholder switch should mount a surface");
    let frame_model = build_editor_shell_frame_model(
        &app,
        &shell_state,
        app.surface_provider_registry(),
        &ThemeTokens::default(),
        None,
        None,
        None,
    );
    let unsupported_frame = frame_model
        .surface(unsupported_surface)
        .expect("unsupported mounted surface should still resolve a diagnostic frame");
    assert_eq!(unsupported_frame.panel_instance_id, viewport_panel);
    assert_eq!(
        unsupported_frame.availability,
        SurfaceProviderAvailability::Unsupported
    );
    assert!(unsupported_frame.routes.is_empty());
}

#[test]
fn stale_provider_local_viewport_details_toggle_fails_closed() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let viewport_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let stale_artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached")
        .clone();
    let stale_commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Toggled {
                target: VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
                checked: true,
            }],
        },
        &stale_artifacts,
    );
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);

    let registry = app.surface_provider_registry_handle();
    RunenwerkEditorShellController::dispatch_commands(
        &mut app,
        &mut shell_state,
        stale_commands,
        registry.as_ref(),
        None,
        None,
        None,
    )
    .expect("stale provider-local viewport details toggle should fail closed");

    assert!(
        !app.surface_sessions()
            .session(viewport_surface)
            .map(|session| session.viewport_details_visible)
            .unwrap_or(false)
    );
}

#[test]
fn provider_id_mismatch_on_viewport_details_toggle_is_rejected_without_mutation() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let viewport_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached")
        .clone();
    let mut commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Toggled {
                target: VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
                checked: true,
            }],
        },
        &artifacts,
    );
    let [ShellCommand::DispatchSurfaceLocalAction { provider_id, .. }] = commands.as_mut_slice()
    else {
        panic!("expected one provider-local action");
    };
    *provider_id = SurfaceProviderId::try_from_raw(999).unwrap();

    let registry = app.surface_provider_registry_handle();
    let result = RunenwerkEditorShellController::dispatch_commands(
        &mut app,
        &mut shell_state,
        commands,
        registry.as_ref(),
        None,
        None,
        None,
    );

    assert!(result.is_err());
    assert!(
        !app.surface_sessions()
            .session(viewport_surface)
            .map(|session| session.viewport_details_visible)
            .unwrap_or(false)
    );
}

#[test]
fn two_viewport_surfaces_keep_independent_details_state() {
    let mut app = RunenwerkEditorApp::new();
    let surface_a = editor_shell::ToolSurfaceInstanceId::try_from_raw(101).unwrap();
    let surface_b = editor_shell::ToolSurfaceInstanceId::try_from_raw(102).unwrap();
    let target_a = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(201).unwrap(),
        active_tool_surface: Some(surface_a),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(301).unwrap(),
    };

    dispatch_shell_command(
        &mut app,
        None,
        surface_session_command(
            target_a,
            SurfaceSessionMutation::Viewport(ViewportSessionMutation::ToggleDetails),
            7,
        ),
        None,
        None,
        None,
        Some(7),
    )
    .expect("targeted viewport details toggle should dispatch");

    assert_eq!(
        app.surface_sessions()
            .session(surface_a)
            .map(|session| session.viewport_details_visible),
        Some(true)
    );
    assert!(
        !app.surface_sessions()
            .session(surface_b)
            .map(|session| session.viewport_details_visible)
            .unwrap_or(false)
    );
}

#[test]
fn two_entity_table_surfaces_keep_independent_search_and_sort_state() {
    let mut app = RunenwerkEditorApp::new();
    let surface_a = editor_shell::ToolSurfaceInstanceId::try_from_raw(111).unwrap();
    let surface_b = editor_shell::ToolSurfaceInstanceId::try_from_raw(112).unwrap();
    let target_a = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(211).unwrap(),
        active_tool_surface: Some(surface_a),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(311).unwrap(),
    };
    let target_b = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(212).unwrap(),
        active_tool_surface: Some(surface_b),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(312).unwrap(),
    };

    dispatch_shell_command(
        &mut app,
        None,
        surface_session_command(
            target_a,
            SurfaceSessionMutation::EntityTable(EntityTableSessionMutation::AppendSearchText {
                text: "alpha".to_string(),
            }),
            1,
        ),
        None,
        None,
        None,
        Some(1),
    )
    .expect("entity-table search should dispatch for surface A");
    dispatch_shell_command(
        &mut app,
        None,
        surface_session_command(
            target_b,
            SurfaceSessionMutation::EntityTable(EntityTableSessionMutation::ToggleSort {
                sort_key: editor_shell::EntityTableSortKey::DisplayName,
            }),
            1,
        ),
        None,
        None,
        None,
        Some(1),
    )
    .expect("entity-table sort should dispatch for surface B");

    let session_a = app
        .surface_sessions()
        .session(surface_a)
        .expect("surface A session should exist");
    let session_b = app
        .surface_sessions()
        .session(surface_b)
        .expect("surface B session should exist");
    assert_eq!(session_a.entity_table_ui_state.search_query(), "alpha");
    assert_eq!(session_b.entity_table_ui_state.search_query(), "");
    assert!(session_a.entity_table_ui_state.sort_ascending());
    assert!(!session_b.entity_table_ui_state.sort_ascending());
}

#[test]
fn two_inspector_surfaces_keep_independent_draft_state() {
    let mut app = RunenwerkEditorApp::new();
    let surface_a = editor_shell::ToolSurfaceInstanceId::try_from_raw(121).unwrap();
    let surface_b = editor_shell::ToolSurfaceInstanceId::try_from_raw(122).unwrap();
    let target_a = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(221).unwrap(),
        active_tool_surface: Some(surface_a),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(321).unwrap(),
    };
    let _target_b = StructuralCommandTarget {
        panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(222).unwrap(),
        active_tool_surface: Some(surface_b),
        tab_stack_id: editor_shell::TabStackId::try_from_raw(322).unwrap(),
    };
    app.surface_sessions_mut()
        .session_mut(surface_a)
        .inspector_ui_state
        .begin_field_edit(
            EntityId(1),
            editor_core::ComponentTypeId(1),
            InspectorPath::root().child_field("x"),
            InspectorEditValue::Text("1".to_string()),
            "1",
        );
    app.surface_sessions_mut()
        .session_mut(surface_b)
        .inspector_ui_state
        .begin_field_edit(
            EntityId(2),
            editor_core::ComponentTypeId(1),
            InspectorPath::root().child_field("x"),
            InspectorEditValue::Text("2".to_string()),
            "2",
        );

    dispatch_shell_command(
        &mut app,
        None,
        surface_session_command(
            target_a,
            SurfaceSessionMutation::Inspector(InspectorSessionMutation::CancelFieldText {
                index: 0,
            }),
            1,
        ),
        None,
        None,
        None,
        Some(1),
    )
    .expect("inspector cancel should dispatch for surface A");

    assert!(
        app.surface_sessions()
            .session(surface_a)
            .expect("surface A session should exist")
            .inspector_ui_state
            .active_draft()
            .is_none()
    );
    assert!(
        app.surface_sessions()
            .session(surface_b)
            .expect("surface B session should exist")
            .inspector_ui_state
            .active_draft()
            .is_some()
    );
}

#[test]
fn two_viewport_surfaces_keep_independent_interaction_state() {
    let mut app = RunenwerkEditorApp::new();
    let surface_a = editor_shell::ToolSurfaceInstanceId::try_from_raw(131).unwrap();
    let surface_b = editor_shell::ToolSurfaceInstanceId::try_from_raw(132).unwrap();
    let entity = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(1), entity, "Alpha", None);
    app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
        entity: EntityId(1),
    })
    .expect("entity selection should succeed");

    app.dispatch_viewport_interaction_for_surface(
        surface_a,
        crate::editor_features::viewport::ViewportInteractionCommand::PointerDown {
            hit: ViewportHitResult::gizmo_axis("X", 0.0),
        },
    )
    .expect("surface A viewport interaction should start");

    assert!(
        app.surface_sessions()
            .viewport_interaction_state(surface_a)
            .expect("surface A viewport state should exist")
            .drag_in_progress()
    );
    assert!(
        !app.surface_sessions()
            .viewport_interaction_state(surface_b)
            .map(|state| state.drag_in_progress())
            .unwrap_or(false)
    );
    assert_eq!(
        app.surface_sessions().active_viewport_drag_surface(),
        Some(surface_a)
    );
}

#[test]
fn two_console_surfaces_keep_independent_follow_state() {
    let mut app = RunenwerkEditorApp::new();
    let surface_a = editor_shell::ToolSurfaceInstanceId::try_from_raw(141).unwrap();
    let surface_b = editor_shell::ToolSurfaceInstanceId::try_from_raw(142).unwrap();

    app.surface_sessions_mut()
        .session_mut(surface_a)
        .console_follow_enabled = false;
    app.surface_sessions_mut()
        .session_mut(surface_b)
        .console_follow_enabled = true;

    assert_eq!(
        app.surface_sessions()
            .session(surface_a)
            .map(|session| session.console_follow_enabled),
        Some(false)
    );
    assert_eq!(
        app.surface_sessions()
            .session(surface_b)
            .map(|session| session.console_follow_enabled),
        Some(true)
    );
}

#[test]
fn dispatch_shell_command_records_workflow_dispatch_event() {
    let mut app = RunenwerkEditorApp::new();

    dispatch_shell_command(&mut app, None, ShellCommand::NoOp, None, None, None, None)
        .expect("no-op shell command should succeed");

    assert!(matches!(
        app.runtime().workflow_log().last().map(|event| &event.kind),
        Some(WorkflowEventKind::ShellCommandDispatched { command: "NoOp" })
    ));
}

#[test]
fn console_follow_disengages_on_upward_scroll_and_reengages_at_bottom() {
    let mut app = RunenwerkEditorApp::new();
    for index in 0..220 {
        app.append_console_line(format!("[test] line {index}"));
    }
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let console_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Console);
    assert!(
        app.surface_sessions()
            .session(console_surface)
            .map(|session| session.console_follow_enabled)
            .unwrap_or(true)
    );

    let tree = shell_state
        .last_tree()
        .expect("shell tree should be cached")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let scroll_bounds = layouts
        .get(&CONSOLE_SCROLL_WIDGET_ID)
        .expect("console scroll layout should exist")
        .content_bounds;
    let pointer = UiPoint::new(
        scroll_bounds.x + scroll_bounds.width * 0.5,
        scroll_bounds.y + 8.0,
    );

    let scroll_up = UiInputEvent::Pointer(PointerEvent {
        kind: PointerEventKind::Scroll,
        position: pointer,
        delta: UiVector::new(0.0, 8.0),
        button: None,
        modifiers: Modifiers::default(),
        click_count: 0,
    });
    RunenwerkEditorShellController::dispatch_input(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        &scroll_up,
    )
    .expect("scroll input should succeed");
    assert!(
        !app.surface_sessions()
            .session(console_surface)
            .map(|session| session.console_follow_enabled)
            .unwrap_or(true),
        "upward scroll from bottom should disengage console follow"
    );

    let scroll_down = UiInputEvent::Pointer(PointerEvent {
        kind: PointerEventKind::Scroll,
        position: pointer,
        delta: UiVector::new(0.0, -8.0),
        button: None,
        modifiers: Modifiers::default(),
        click_count: 0,
    });
    for _ in 0..128 {
        RunenwerkEditorShellController::dispatch_input(
            &mut app,
            &mut shell_state,
            bounds,
            &theme,
            &scroll_down,
        )
        .expect("scroll input should succeed");
        if app
            .surface_sessions()
            .session(console_surface)
            .map(|session| session.console_follow_enabled)
            .unwrap_or(false)
        {
            break;
        }
    }
    assert!(
        app.surface_sessions()
            .session(console_surface)
            .map(|session| session.console_follow_enabled)
            .unwrap_or(false),
        "console follow should re-engage after returning to bottom",
    );
}

#[test]
fn console_follow_auto_scrolls_only_while_follow_enabled() {
    let mut app = RunenwerkEditorApp::new();
    for index in 0..200 {
        app.append_console_line(format!("[test] line {index}"));
    }
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let tree = shell_state
        .last_tree()
        .expect("shell tree should be cached")
        .clone();
    let initial_max = shell_state
        .runtime()
        .max_scroll_offset(&tree, bounds, CONSOLE_SCROLL_WIDGET_ID)
        .unwrap_or(0.0);
    let initial_offset = shell_state
        .runtime()
        .scroll_offset(CONSOLE_SCROLL_WIDGET_ID);
    assert!(
        (initial_offset - initial_max).abs() <= 1.0,
        "follow-enabled frame should pin console to bottom",
    );

    app.append_console_line("[test] new follow-on line");
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let tree_after_append = shell_state
        .last_tree()
        .expect("shell tree should remain cached")
        .clone();
    let max_after_append = shell_state
        .runtime()
        .max_scroll_offset(&tree_after_append, bounds, CONSOLE_SCROLL_WIDGET_ID)
        .unwrap_or(0.0);
    let offset_after_append = shell_state
        .runtime()
        .scroll_offset(CONSOLE_SCROLL_WIDGET_ID);
    assert!(
        (offset_after_append - max_after_append).abs() <= 1.0,
        "auto-follow should stay at bottom while enabled",
    );

    let console_surface = surface_id_by_kind(shell_state.workspace_state(), PanelKind::Console);
    app.surface_sessions_mut()
        .session_mut(console_surface)
        .console_follow_enabled = false;
    shell_state
        .runtime_mut()
        .set_scroll_offset(CONSOLE_SCROLL_WIDGET_ID, (max_after_append * 0.5).max(0.0));
    let previous_offset = shell_state
        .runtime()
        .scroll_offset(CONSOLE_SCROLL_WIDGET_ID);

    app.append_console_line("[test] new follow-off line");
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let offset_follow_disabled = shell_state
        .runtime()
        .scroll_offset(CONSOLE_SCROLL_WIDGET_ID);
    assert!(
        (offset_follow_disabled - previous_offset).abs() <= 1.0,
        "disabled follow should preserve user scroll position",
    );
}

#[test]
fn shell_identity_is_stable_across_rebuilds() {
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let workspace_before = shell_state.workspace_id();
    let workspace_state_before = shell_state.workspace_state().clone();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let projection_before = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be cached after frame build")
        .workspace
        .widget_context_by_id
        .clone();
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let projection_after = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should remain cached after rebuild")
        .workspace
        .widget_context_by_id
        .clone();

    assert_eq!(shell_state.workspace_id(), workspace_before);
    assert_eq!(*shell_state.workspace_state(), workspace_state_before);
    assert_eq!(projection_before, projection_after);
}

#[test]
fn shell_state_tracks_active_workspace_profile_separately_from_workspace_graph() {
    let mut shell_state = RunenwerkEditorShellState::new();
    let workspace_before = shell_state.workspace_state().clone();

    assert_eq!(
        shell_state.active_workspace_profile_id(),
        editor_shell::LAYOUT_WORKSPACE_PROFILE_ID,
    );

    shell_state.set_active_workspace_profile_id(
        editor_shell::WorkspaceProfileId::try_from_raw(99).unwrap(),
    );

    assert_eq!(
        shell_state.active_workspace_profile_id(),
        editor_shell::WorkspaceProfileId::try_from_raw(99).unwrap(),
    );
    assert_eq!(
        *shell_state.workspace_state(),
        workspace_before,
        "changing active profile identity should not mutate the workspace graph",
    );
}

#[test]
fn clear_cached_projection_keeps_shell_identity_unchanged() {
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let workspace_before = shell_state.workspace_id();
    let workspace_state_before = shell_state.workspace_state().clone();
    let atlas = UiFontAtlasResource::default();
    let _ = RunenwerkEditorShellController::build_frame(
        &app,
        &mut shell_state,
        UiRect::new(0.0, 0.0, 1280.0, 720.0),
        &ThemeTokens::default(),
        &atlas,
    );
    assert!(shell_state.last_projection_artifacts().is_some());

    shell_state.clear_cached_projection();

    assert_eq!(shell_state.workspace_id(), workspace_before);
    assert_eq!(*shell_state.workspace_state(), workspace_state_before);
    assert!(shell_state.last_projection_artifacts().is_none());
    assert!(shell_state.last_tree().is_none());
    assert!(shell_state.last_bounds().is_none());
}

#[test]
fn workspace_surface_remount_preserves_viewport_structural_identity_across_rebuilds() {
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let before = *shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .workspace
        .widget_context_by_id
        .get(&VIEWPORT_PANEL_WIDGET_ID)
        .expect("viewport panel structural context should exist");
    let viewport_surface = before
        .active_tool_surface
        .expect("viewport panel should start with an attached tool surface");

    shell_state
        .apply_workspace_mutation(WorkspaceMutation::DetachToolSurfaceFromPanel {
            panel_id: before.panel_instance_id,
        })
        .expect("detaching viewport tool surface should succeed");
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let detached = *shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist after detach")
        .workspace
        .widget_context_by_id
        .get(&VIEWPORT_PANEL_WIDGET_ID)
        .expect("viewport panel structural context should exist after detach");

    assert_eq!(detached.panel_instance_id, before.panel_instance_id);
    assert_eq!(detached.tab_stack_id, before.tab_stack_id);
    assert_eq!(detached.active_tool_surface, None);

    shell_state
        .apply_workspace_mutation(WorkspaceMutation::AttachToolSurfaceToPanel {
            panel_id: before.panel_instance_id,
            tool_surface_id: viewport_surface,
        })
        .expect("reattaching viewport tool surface should succeed");
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let reattached = *shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist after reattach")
        .workspace
        .widget_context_by_id
        .get(&VIEWPORT_PANEL_WIDGET_ID)
        .expect("viewport panel structural context should exist after reattach");

    assert_eq!(reattached.panel_instance_id, before.panel_instance_id);
    assert_eq!(reattached.tab_stack_id, before.tab_stack_id);
    assert_eq!(reattached.active_tool_surface, Some(viewport_surface));
}

#[test]
fn stale_projection_commands_fail_closed_after_rebuild() {
    let mut app = RunenwerkEditorApp::new();
    let ecs_entity = app.runtime_mut().spawn_world_entity(TestMarker);
    app.runtime_mut()
        .register_entity(EntityId(1), ecs_entity, "Player", None);
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let stale_artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should be present")
        .clone();
    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let current_epoch = shell_state.current_projection_epoch();
    assert!(
        stale_artifacts.projection_epoch < current_epoch,
        "second rebuild should invalidate older projection artifacts",
    );

    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(outliner_row_widget_id(0))],
    };
    let commands = map_interactions_to_shell_commands(&interactions, &stale_artifacts);
    assert_eq!(commands.len(), 1);

    let workflow_log_len_before = app.runtime().workflow_log().len();
    for command in commands {
        dispatch_shell_command(
            &mut app,
            None,
            command,
            None,
            None,
            None,
            Some(current_epoch),
        )
        .expect("stale command dispatch should fail closed without error");
    }

    assert_eq!(app.outliner_state().selected_entity, None);
    assert_eq!(app.runtime().workflow_log().len(), workflow_log_len_before);
}

#[test]
fn drag_drop_tab_rehomes_panel_with_stable_structural_identity() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();

    let workspace_before = shell_state.workspace_state().clone();
    let outliner_stack_id = workspace_before
        .tab_stacks()
        .find(|stack| {
            stack.ordered_panels.iter().any(|panel| {
                workspace_before
                    .panel(*panel)
                    .map(|value| value.panel_kind == editor_shell::PanelKind::Outliner)
                    .unwrap_or(false)
            })
        })
        .expect("outliner stack should exist")
        .id;
    let (viewport_panel_id, viewport_stack_id) = artifacts
        .workspace
        .tab_button_route_by_widget_id
        .values()
        .find_map(|route| {
            workspace_before
                .panel(route.panel_instance_id)
                .filter(|panel| panel.panel_kind == editor_shell::PanelKind::Viewport)
                .map(|_| (route.panel_instance_id, route.tab_stack_id))
        })
        .expect("viewport tab route should exist");
    let viewport_surface_before = workspace_before
        .panel(viewport_panel_id)
        .expect("viewport panel should exist")
        .active_tool_surface;

    let source_widget = artifacts
        .workspace
        .tab_button_route_by_widget_id
        .iter()
        .find_map(|(widget_id, route)| {
            (route.panel_instance_id == viewport_panel_id).then_some(*widget_id)
        })
        .expect("source tab widget should exist");
    let target_drop_widget = artifacts
        .workspace
        .tab_drop_route_by_widget_id
        .iter()
        .find_map(|(widget_id, route)| {
            matches!(
                route.target,
                editor_shell::ProjectedTabDropTarget::TabStack {
                    tab_stack_id,
                    insert_index: 1
                } if tab_stack_id == outliner_stack_id
            )
            .then_some(*widget_id)
        })
        .expect("target tab drop widget should exist");

    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let source_position = center_of_widget(&layouts, source_widget);
    let mid_position = UiPoint::new(source_position.x + 26.0, source_position.y + 5.0);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        source_position,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        mid_position,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        mid_position,
        None,
    );
    let drag_tree = shell_state
        .last_tree()
        .expect("shell tree should exist while drag preview is active")
        .clone();
    let drag_layouts = shell_state.runtime().compute_layout(&drag_tree, bounds);
    let target_position = center_of_widget(&drag_layouts, target_drop_widget);
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        target_position,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        target_position,
        Some(PointerButton::Primary),
    );

    let workspace_after = shell_state.workspace_state();
    let outliner_stack_after = workspace_after
        .tab_stack(outliner_stack_id)
        .expect("outliner stack should exist");
    assert!(
        outliner_stack_after
            .ordered_panels
            .contains(&viewport_panel_id),
        "viewport panel should be rehomed into outliner tab stack",
    );
    assert_eq!(
        workspace_after
            .panel(viewport_panel_id)
            .expect("viewport panel should exist")
            .active_tool_surface,
        viewport_surface_before,
        "tab drag/drop must preserve panel tool-surface identity",
    );
    assert!(
        !workspace_after
            .tab_stack(viewport_stack_id)
            .expect("source stack should exist")
            .ordered_panels
            .contains(&viewport_panel_id),
        "source stack should no longer contain moved panel",
    );
}

#[test]
fn tab_click_under_drag_threshold_activates_tab_without_reorder() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let (entity_table_panel_id, outliner_stack_id) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::EntityTable);
    let before_order = shell_state
        .workspace_state()
        .tab_stack(outliner_stack_id)
        .expect("outliner stack should exist")
        .ordered_panels
        .clone();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let source_widget = tab_widget_for_panel(
        &artifacts,
        shell_state.workspace_state(),
        PanelKind::EntityTable,
    );
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let source_position = center_of_widget(&layouts, source_widget);
    let near_position = UiPoint::new(source_position.x + 2.0, source_position.y + 1.0);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        source_position,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        near_position,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        near_position,
        Some(PointerButton::Primary),
    );

    let outliner_stack = shell_state
        .workspace_state()
        .tab_stack(outliner_stack_id)
        .expect("outliner stack should remain");
    assert_eq!(outliner_stack.ordered_panels, before_order);
    assert_eq!(outliner_stack.active_panel, Some(entity_table_panel_id));
    assert!(
        shell_state.workspace_state().hosts().all(|host| !matches!(
            host.kind,
            editor_shell::PanelHostKind::FloatingHostPlaceholder(_)
        )),
        "clicking a tab below the drag threshold must not create a floating host",
    );
}

#[test]
fn drag_drop_tab_reorders_within_same_stack() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let (outliner_panel_id, outliner_stack_id) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Outliner);
    let (entity_table_panel_id, _) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::EntityTable);

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let source_widget = tab_widget_for_panel(
        &artifacts,
        shell_state.workspace_state(),
        PanelKind::EntityTable,
    );
    let target_drop_widget = artifacts
        .workspace
        .tab_drop_route_by_widget_id
        .iter()
        .find_map(|(widget_id, route)| {
            matches!(
                route.target,
                editor_shell::ProjectedTabDropTarget::TabStack {
                    tab_stack_id,
                    insert_index: 0
                } if tab_stack_id == outliner_stack_id
            )
            .then_some(*widget_id)
        })
        .expect("target reorder drop slot should exist");
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let source_position = center_of_widget(&layouts, source_widget);
    let mid_position = UiPoint::new(source_position.x + 24.0, source_position.y + 4.0);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        source_position,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        mid_position,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        mid_position,
        None,
    );
    let drag_tree = shell_state
        .last_tree()
        .expect("shell tree should exist while drag preview is active")
        .clone();
    let drag_layouts = shell_state.runtime().compute_layout(&drag_tree, bounds);
    let target_position = center_of_widget(&drag_layouts, target_drop_widget);
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        target_position,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        target_position,
        Some(PointerButton::Primary),
    );

    let stack_after = shell_state
        .workspace_state()
        .tab_stack(outliner_stack_id)
        .expect("outliner stack should remain");
    assert_eq!(
        stack_after.ordered_panels,
        vec![entity_table_panel_id, outliner_panel_id]
    );
    assert_eq!(stack_after.active_panel, Some(entity_table_panel_id));
}

#[test]
fn drag_drop_tab_to_float_creates_editor_managed_floating_host() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let (viewport_panel_id, viewport_stack_id) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let viewport_surface_before = shell_state
        .workspace_state()
        .panel(viewport_panel_id)
        .expect("viewport panel should exist")
        .active_tool_surface;

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let source_widget = tab_widget_for_panel(
        &artifacts,
        shell_state.workspace_state(),
        PanelKind::Viewport,
    );
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let source_position = center_of_widget(&layouts, source_widget);
    let activation_position = UiPoint::new(source_position.x + 32.0, source_position.y + 4.0);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        source_position,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        activation_position,
        None,
    );
    assert!(
        shell_state.docking_visual_state().active_tab_drag.is_some(),
        "moving beyond the threshold should start a tab drag"
    );

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let tree_with_float_target = shell_state
        .last_tree()
        .expect("shell tree with float target should exist")
        .clone();
    let layouts_with_float_target = shell_state
        .runtime()
        .compute_layout(&tree_with_float_target, bounds);
    let float_position = center_of_widget(&layouts_with_float_target, FLOATING_DROP_ZONE_WIDGET_ID);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        float_position,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        float_position,
        Some(PointerButton::Primary),
    );

    let workspace_after = shell_state.workspace_state();
    let floating_hosts = workspace_after
        .hosts()
        .filter_map(|host| match host.kind {
            editor_shell::PanelHostKind::FloatingHostPlaceholder(placeholder) => {
                Some((host.id, placeholder))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(floating_hosts.len(), 1);
    let floating_stack_id = floating_hosts[0]
        .1
        .tab_stack_id
        .expect("floating host should own a tab stack");
    assert_eq!(
        workspace_after
            .tab_stack(floating_stack_id)
            .expect("floating stack should exist")
            .ordered_panels,
        vec![viewport_panel_id]
    );
    assert_eq!(
        workspace_after
            .panel(viewport_panel_id)
            .expect("viewport panel should remain")
            .active_tool_surface,
        viewport_surface_before
    );
    assert!(
        workspace_after
            .tab_stack(viewport_stack_id)
            .expect("source viewport stack should remain as an empty dock slot")
            .ordered_panels
            .is_empty()
    );
}

#[test]
fn dragging_left_right_split_border_updates_workspace_fraction() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let split_bounds = layouts
        .get(&LEFT_RIGHT_SPLIT_WIDGET_ID)
        .expect("left-right split layout should exist")
        .bounds;
    let before = artifacts
        .workspace
        .fixed_layout
        .as_ref()
        .expect("fixed layout should project")
        .left_right_fraction;
    let boundary_x = split_bounds.x + split_bounds.width * before;
    let pointer_down = UiPoint::new(boundary_x, split_bounds.y + split_bounds.height * 0.5);
    let pointer_move = UiPoint::new(pointer_down.x + 120.0, pointer_down.y);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        pointer_down,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        pointer_move,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        pointer_move,
        Some(PointerButton::Primary),
    );

    let after = left_right_split_fraction(shell_state.workspace_state());
    assert!(
        (after - before).abs() > 0.02,
        "dragging split border should mutate left-right split fraction",
    );
}

#[test]
fn secondary_clicking_tab_opens_area_action_menu_without_extra_button() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let (viewport_panel, viewport_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let tab_widget = artifacts
        .workspace
        .tab_button_route_by_widget_id
        .iter()
        .find_map(|(widget_id, route)| {
            (route.panel_instance_id == viewport_panel).then_some(*widget_id)
        })
        .expect("viewport tab widget should be projected");
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let pointer = center_of_widget(&layouts, tab_widget);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        pointer,
        Some(PointerButton::Secondary),
    );

    assert_eq!(
        shell_state.active_tab_stack_action_menu(),
        Some(viewport_stack)
    );
}

#[test]
fn shift_dragging_area_corner_splits_tab_stack_area() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();
    let (_, viewport_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let area_bounds = layouts
        .get(&editor_shell::tab_stack_container_widget_id(viewport_stack))
        .expect("viewport tab stack container should exist")
        .bounds;
    let pointer_down = UiPoint::new(area_bounds.x + 2.0, area_bounds.y + 2.0);
    let pointer_move = UiPoint::new(pointer_down.x + 80.0, pointer_down.y + 4.0);
    let before_count = shell_state.workspace_state().tab_stacks().count();
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };

    dispatch_pointer_with_modifiers(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        pointer_down,
        Some(PointerButton::Primary),
        shift,
    );
    dispatch_pointer_with_modifiers(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        pointer_move,
        None,
        shift,
    );
    dispatch_pointer_with_modifiers(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        pointer_move,
        Some(PointerButton::Primary),
        shift,
    );
    assert!(
        shell_state.workspace_state().tab_stacks().count() > before_count,
        "shift-dragging an area corner inward should create a split tab-stack area",
    );
}

#[test]
fn dragging_left_right_split_border_applies_multiple_pointer_moves() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let split_bounds = layouts
        .get(&LEFT_RIGHT_SPLIT_WIDGET_ID)
        .expect("left-right split layout should exist")
        .bounds;
    let before = artifacts
        .workspace
        .fixed_layout
        .as_ref()
        .expect("fixed layout should project")
        .left_right_fraction;
    let boundary_x = split_bounds.x + split_bounds.width * before;
    let pointer_down = UiPoint::new(boundary_x, split_bounds.y + split_bounds.height * 0.5);
    let pointer_move_a = UiPoint::new(pointer_down.x + 60.0, pointer_down.y);
    let pointer_move_b = UiPoint::new(pointer_down.x + 180.0, pointer_down.y);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        pointer_down,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        pointer_move_a,
        None,
    );
    let after_first_move = left_right_split_fraction(shell_state.workspace_state());
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        pointer_move_b,
        None,
    );
    let after_second_move = left_right_split_fraction(shell_state.workspace_state());
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        pointer_move_b,
        Some(PointerButton::Primary),
    );

    assert!(
        (after_first_move - before).abs() > 0.01,
        "first move should adjust split fraction",
    );
    assert!(
        (after_second_move - after_first_move).abs() > 0.01,
        "second move should continue adjusting split fraction in same drag session",
    );
}

#[test]
fn closing_last_tab_closes_the_empty_area() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let (viewport_panel, viewport_stack) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    assert_eq!(
        shell_state
            .workspace_state()
            .tab_stack(viewport_stack)
            .expect("viewport stack should exist")
            .ordered_panels
            .len(),
        1
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ClosePanelTab {
            tab_stack_id: viewport_stack,
            panel_instance_id: viewport_panel,
            projection_epoch: 0,
        },
        None,
        None,
        None,
        None,
    )
    .expect("closing the last tab should close its area");

    assert!(
        shell_state
            .workspace_state()
            .tab_stack(viewport_stack)
            .is_none(),
        "single-tab close should remove the now-empty tab stack area"
    );
    assert!(
        shell_state
            .workspace_state()
            .panel(viewport_panel)
            .is_none()
    );
}

#[test]
fn cursor_intent_at_split_corner_uses_diagonal_resize() {
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let fixed = artifacts
        .workspace
        .fixed_layout
        .as_ref()
        .expect("fixed layout should project");
    let center_right_bounds = layouts
        .get(&CENTER_RIGHT_SPLIT_WIDGET_ID)
        .expect("center-right split layout should exist")
        .bounds;
    let pointer = UiPoint::new(
        center_right_bounds.x + 1.0,
        center_right_bounds.y + center_right_bounds.height * fixed.center_right_fraction,
    );

    let intent = RunenwerkEditorShellController::cursor_intent_for_pointer(&shell_state, pointer);

    assert!(
        matches!(
            intent,
            crate::shell::ShellCursorIntent::ResizeNwse
                | crate::shell::ShellCursorIntent::ResizeNesw
        ),
        "split boundary intersections should expose a diagonal resize cursor",
    );
}

#[test]
fn shift_dragging_split_corner_updates_both_split_fractions() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let artifacts = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let fixed = artifacts
        .workspace
        .fixed_layout
        .as_ref()
        .expect("fixed layout should project");
    let center_right_bounds = layouts
        .get(&CENTER_RIGHT_SPLIT_WIDGET_ID)
        .expect("center-right split layout should exist")
        .bounds;
    let pointer_down = UiPoint::new(
        center_right_bounds.x + 1.0,
        center_right_bounds.y + center_right_bounds.height * fixed.center_right_fraction,
    );
    let pointer_move = UiPoint::new(pointer_down.x + 90.0, pointer_down.y + 70.0);
    let before_left_right = left_right_split_fraction(shell_state.workspace_state());
    let before_center_right = center_right_split_fraction(shell_state.workspace_state());
    let no_modifiers = Modifiers::default();
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };

    dispatch_pointer_with_modifiers(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        pointer_down,
        Some(PointerButton::Primary),
        no_modifiers,
    );
    dispatch_pointer_with_modifiers(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        pointer_move,
        None,
        shift,
    );
    dispatch_pointer_with_modifiers(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        pointer_move,
        Some(PointerButton::Primary),
        shift,
    );

    let after_left_right = left_right_split_fraction(shell_state.workspace_state());
    let after_center_right = center_right_split_fraction(shell_state.workspace_state());
    assert!(
        (after_left_right - before_left_right).abs() > 0.01,
        "corner drag should update horizontal split fraction",
    );
    assert!(
        (after_center_right - before_center_right).abs() > 0.01,
        "corner drag should update vertical split fraction",
    );
}

#[test]
fn dragging_dynamic_split_border_updates_workspace_fraction() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1400.0, 840.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();
    let (_, viewport_stack_id) =
        panel_and_stack_by_kind(shell_state.workspace_state(), PanelKind::Viewport);
    let split_host_id = shell_state
        .apply_workspace_mutation_with_allocations(|allocator| {
            let split_host_id = allocator.allocate_panel_host_id();
            let first_child_host_id = allocator.allocate_panel_host_id();
            let second_child_host_id = allocator.allocate_panel_host_id();
            let new_tab_stack_id = allocator.allocate_tab_stack_id();
            let new_panel_id = allocator.allocate_panel_instance_id();
            let new_tool_surface_id = allocator.allocate_tool_surface_instance_id();
            (
                WorkspaceMutation::SplitTabStackArea {
                    tab_stack_id: viewport_stack_id,
                    axis: WorkspaceSplitAxis::Horizontal,
                    split_host_id,
                    first_child_host_id,
                    second_child_host_id,
                    new_tab_stack_id,
                    new_panel_id,
                    new_panel_kind: PanelKind::Inspector,
                    new_tool_surface_id,
                    new_tool_surface_kind: ToolSurfaceKind::Inspector,
                    fraction: 0.45,
                },
                split_host_id,
            )
        })
        .expect("dynamic workspace split should be valid");

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let tree = shell_state
        .last_tree()
        .expect("shell tree should exist")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let split_widget_id = workspace_split_host_widget_id(split_host_id);
    let split_bounds = layouts
        .get(&split_widget_id)
        .expect("dynamic split layout should exist")
        .bounds;
    let before = split_host_fraction(shell_state.workspace_state(), split_host_id);
    let boundary_x = split_bounds.x + split_bounds.width * before;
    let pointer_down = UiPoint::new(boundary_x, split_bounds.y + split_bounds.height * 0.5);
    let pointer_move = UiPoint::new(pointer_down.x + 100.0, pointer_down.y);

    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Down,
        pointer_down,
        Some(PointerButton::Primary),
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Move,
        pointer_move,
        None,
    );
    dispatch_pointer(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        PointerEventKind::Up,
        pointer_move,
        Some(PointerButton::Primary),
    );

    let after = split_host_fraction(shell_state.workspace_state(), split_host_id);
    assert!(
        (after - before).abs() > 0.02,
        "dragging a dynamic split border should mutate that split host fraction",
    );
}

#[test]
fn workspace_layout_roundtrip_preserves_identity_after_float_cycle() {
    let mut shell_state = RunenwerkEditorShellState::new();
    let workspace_before = shell_state.workspace_state().clone();
    let viewport_stack_id = workspace_before
        .tab_stacks()
        .find(|stack| {
            stack.ordered_panels.iter().any(|panel| {
                workspace_before
                    .panel(*panel)
                    .map(|value| value.panel_kind == editor_shell::PanelKind::Viewport)
                    .unwrap_or(false)
            })
        })
        .expect("viewport stack should exist")
        .id;
    let viewport_panel_id = workspace_before
        .tab_stack(viewport_stack_id)
        .and_then(|stack| stack.ordered_panels.first().copied())
        .expect("viewport panel should exist");
    let viewport_surface_before = workspace_before
        .panel(viewport_panel_id)
        .expect("viewport panel should exist")
        .active_tool_surface;

    let floating_host_id = shell_state.allocate_panel_host_id();
    let floating_stack_id = shell_state.allocate_tab_stack_id();
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::MovePanelToNewFloatingHost {
            panel_id: viewport_panel_id,
            source_tab_stack_id: viewport_stack_id,
            floating_host_id,
            floating_tab_stack_id: floating_stack_id,
            bounds: editor_shell::FloatingHostBounds::new(120.0, 88.0, 540.0, 360.0),
        })
        .expect("floating move should succeed");

    let path = {
        let mut value = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        value.push(format!("runenwerk_shell_workspace_cycle_{nanos}.ron"));
        value
    };
    crate::persistence::write_workspace_layout(&path, shell_state.workspace_state())
        .expect("workspace layout should write");
    let loaded = crate::persistence::read_workspace_layout(&path).expect("workspace should load");
    let _ = std::fs::remove_file(path);

    let mut restored_shell_state = RunenwerkEditorShellState::new();
    restored_shell_state.replace_workspace_state(loaded);
    let restored = restored_shell_state.workspace_state();

    assert_eq!(
        restored.workspace_id(),
        shell_state.workspace_state().workspace_id(),
        "workspace id should survive workspace layout persistence",
    );
    assert_eq!(
        restored
            .panel(viewport_panel_id)
            .expect("viewport panel should remain")
            .active_tool_surface,
        viewport_surface_before,
        "panel/tool-surface identity should survive workspace layout persistence",
    );
    assert!(
        matches!(
            restored
                .host(floating_host_id)
                .expect("floating host should remain")
                .kind,
            editor_shell::PanelHostKind::FloatingHostPlaceholder(
                editor_shell::FloatingHostPlaceholderState {
                    tab_stack_id: Some(id),
                    ..
                }
            ) if id == floating_stack_id
        ),
        "floating host stack identity should survive workspace layout persistence",
    );
}

fn dispatch_pointer(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    bounds: UiRect,
    theme: &ThemeTokens,
    kind: PointerEventKind,
    position: UiPoint,
    button: Option<PointerButton>,
) {
    let event = UiInputEvent::Pointer(PointerEvent {
        kind,
        position,
        delta: UiVector::ZERO,
        button,
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    RunenwerkEditorShellController::dispatch_input(app, shell_state, bounds, theme, &event)
        .expect("pointer dispatch should succeed");
}

#[expect(
    clippy::too_many_arguments,
    reason = "test pointer helper keeps UI event fields explicit"
)]
fn dispatch_pointer_with_modifiers(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    bounds: UiRect,
    theme: &ThemeTokens,
    kind: PointerEventKind,
    position: UiPoint,
    button: Option<PointerButton>,
    modifiers: Modifiers,
) {
    let event = UiInputEvent::Pointer(PointerEvent {
        kind,
        position,
        delta: UiVector::ZERO,
        button,
        modifiers,
        click_count: 1,
    });
    RunenwerkEditorShellController::dispatch_input(app, shell_state, bounds, theme, &event)
        .expect("pointer dispatch should succeed");
}

fn ui_tree_contains_viewport_embed(node: &editor_shell::UiNode, viewport_id: ViewportId) -> bool {
    matches!(
        &node.kind,
        editor_shell::UiNodeKind::ViewportSurfaceEmbed(embed)
            if embed.viewport_id == viewport_id.0
    ) || node
        .children
        .iter()
        .any(|child| ui_tree_contains_viewport_embed(child, viewport_id))
}

fn center_of_widget(
    layouts: &editor_shell::ComputedLayoutMap,
    widget_id: editor_shell::WidgetId,
) -> UiPoint {
    let bounds = layouts
        .get(&widget_id)
        .expect("widget layout should exist")
        .bounds;
    UiPoint::new(
        bounds.x + bounds.width * 0.5,
        bounds.y + bounds.height * 0.5,
    )
}

fn panel_and_stack_by_kind(
    workspace: &editor_shell::WorkspaceState,
    kind: PanelKind,
) -> (editor_shell::PanelInstanceId, editor_shell::TabStackId) {
    let panel_id = workspace
        .panels()
        .find(|panel| panel.panel_kind == kind)
        .expect("panel kind should exist")
        .id;
    let tab_stack_id = workspace
        .tab_stacks()
        .find(|stack| stack.ordered_panels.contains(&panel_id))
        .expect("panel should be mounted in a tab stack")
        .id;
    (panel_id, tab_stack_id)
}

fn surface_id_by_kind(
    workspace: &editor_shell::WorkspaceState,
    kind: PanelKind,
) -> editor_shell::ToolSurfaceInstanceId {
    let (panel_id, _) = panel_and_stack_by_kind(workspace, kind);
    workspace
        .panel(panel_id)
        .and_then(|panel| panel.active_tool_surface)
        .expect("panel should have active tool surface")
}

fn left_right_split_fraction(workspace: &editor_shell::WorkspaceState) -> f32 {
    let root = workspace
        .host(workspace.root_host_id())
        .expect("root host should exist");
    let editor_shell::PanelHostKind::SplitHost(root_split) = root.kind else {
        panic!("root host should be split host");
    };
    let left_right = workspace
        .host(root_split.first_child)
        .expect("left-right host should exist");
    let editor_shell::PanelHostKind::SplitHost(left_right_split) = left_right.kind else {
        panic!("left-right host should be split host");
    };
    left_right_split.fraction
}

fn center_right_split_fraction(workspace: &editor_shell::WorkspaceState) -> f32 {
    let root = workspace
        .host(workspace.root_host_id())
        .expect("root host should exist");
    let editor_shell::PanelHostKind::SplitHost(root_split) = root.kind else {
        panic!("root host should be split host");
    };
    let left_right = workspace
        .host(root_split.first_child)
        .expect("left-right host should exist");
    let editor_shell::PanelHostKind::SplitHost(left_right_split) = left_right.kind else {
        panic!("left-right host should be split host");
    };
    let center_right = workspace
        .host(left_right_split.second_child)
        .expect("center-right host should exist");
    let editor_shell::PanelHostKind::SplitHost(center_right_split) = center_right.kind else {
        panic!("center-right host should be split host");
    };
    center_right_split.fraction
}

fn split_host_fraction(
    workspace: &editor_shell::WorkspaceState,
    split_host_id: editor_shell::PanelHostId,
) -> f32 {
    let host = workspace
        .host(split_host_id)
        .expect("split host should exist");
    let editor_shell::PanelHostKind::SplitHost(split) = host.kind else {
        panic!("host should be split host");
    };
    split.fraction
}

fn tab_widget_for_panel(
    artifacts: &editor_shell::ShellProjectionArtifacts,
    workspace: &editor_shell::WorkspaceState,
    kind: PanelKind,
) -> editor_shell::WidgetId {
    artifacts
        .workspace
        .tab_button_route_by_widget_id
        .iter()
        .find_map(|(widget_id, route)| {
            workspace
                .panel(route.panel_instance_id)
                .filter(|panel| panel.panel_kind == kind)
                .map(|_| *widget_id)
        })
        .expect("tab widget for panel kind should exist")
}
