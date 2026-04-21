use editor_core::{ChangeOrigin, EntityId, SelectionTarget, SessionChangeKind, WorkflowEventKind};
use editor_shell::{
    CONSOLE_SCROLL_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID, PanelHostId,
    PanelHostKind, PanelKind, ShellCommand, StructuralCommandTarget, TabStackHostState, TabStackId,
    UiInteraction, UiInteractionResults, VIEWPORT_PANEL_WIDGET_ID, WorkspaceMutation,
    WorkspaceSplitAxis, map_interactions_to_shell_commands, outliner_row_widget_id,
};
use editor_viewport::{
    ArtifactObservationFrame, ExpressionProductId, ProducerHealth, ProductAvailabilityState,
    ViewportId, ViewportPresentationState,
};
use engine::plugins::render::UiFontAtlasResource;
use ui_input::{Modifiers, PointerEvent, PointerEventKind, UiInputEvent};
use ui_math::{UiPoint, UiRect, UiVector};
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ViewportArtifactObservationResource, ViewportPresentationStateResource,
};
use crate::shell::{
    RunenwerkEditorShellController, RunenwerkEditorShellState, SELECT_TOOL_ID, TRANSLATE_TOOL_ID,
    build_editor_shell_view_model, dispatch_shell_command,
};

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct TestMarker;

fn center_for_widget(
    shell_state: &RunenwerkEditorShellState,
    bounds: UiRect,
    widget_id: editor_shell::WidgetId,
) -> UiPoint {
    let tree = shell_state
        .last_tree()
        .expect("shell tree should be cached")
        .clone();
    let layouts = shell_state.runtime().compute_layout(&tree, bounds);
    let layout = layouts
        .get(&widget_id)
        .expect("widget layout should exist in shell tree");
    UiPoint::new(
        layout.content_bounds.x + layout.content_bounds.width * 0.5,
        layout.content_bounds.y + layout.content_bounds.height * 0.5,
    )
}

fn pointer_event(
    kind: PointerEventKind,
    position: UiPoint,
    button: Option<ui_input::PointerButton>,
) -> UiInputEvent {
    UiInputEvent::Pointer(PointerEvent {
        kind,
        position,
        delta: UiVector::ZERO,
        button,
        modifiers: Modifiers::default(),
        click_count: 1,
    })
}

#[test]
fn build_editor_shell_view_model_reflects_current_outliner_selection() {
    let mut app = RunenwerkEditorApp::new();

    let ecs_entity = app.runtime_mut().spawn_world_entity(TestMarker);

    app.runtime_mut()
        .register_entity(EntityId(1), ecs_entity, "Player", None);

    app.runtime_mut().set_selection_single_with_origin(
        SelectionTarget::Entity(EntityId(1)),
        ChangeOrigin::Runtime,
    );

    let shell = build_editor_shell_view_model(&app);

    assert_eq!(shell.outliner.rows.len(), 1);
    assert_eq!(shell.outliner.rows[0].entity, EntityId(1));
    assert!(shell.outliner.rows[0].is_selected);
}

#[test]
fn build_editor_shell_view_model_reflects_active_tool_and_viewport_state() {
    let mut app = RunenwerkEditorApp::new();

    app.runtime_mut()
        .set_active_tool_with_origin(Some(TRANSLATE_TOOL_ID), ChangeOrigin::Runtime);

    app.tool_runtime_state_mut()
        .set_hovered_entity(Some(EntityId(7)));

    let shell = build_editor_shell_view_model(&app);

    assert_eq!(shell.toolbar.buttons.len(), 7);
    assert!(
        shell
            .toolbar
            .buttons
            .iter()
            .any(|button| { button.id == TRANSLATE_TOOL_ID && button.is_active })
    );
    assert_eq!(shell.viewport.hovered_entity, Some(EntityId(7)));
    assert!(!shell.viewport.preview_active);
}

#[test]
fn build_editor_shell_view_model_has_no_implicit_main_viewport_without_products() {
    let app = RunenwerkEditorApp::new();

    let shell = build_editor_shell_view_model(&app);

    assert!(
        shell.viewport.viewport_id.is_none(),
        "shell view model must not synthesize an implicit main viewport id when runtime has no viewport products",
    );
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
    )
    .expect("translate tool command should succeed");
    assert_eq!(
        app.runtime().session().active_tool(),
        Some(TRANSLATE_TOOL_ID)
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
        ShellCommand::SelectOutlinerEntity {
            entity: EntityId(1),
            target: StructuralCommandTarget {
                panel_instance_id: editor_shell::PanelInstanceId::new(1),
                active_tool_surface: None,
                tab_stack_id: editor_shell::TabStackId::new(1),
            },
            projection_epoch: 0,
        },
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
fn dispatch_shell_command_selects_viewport_product_when_available() {
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
    frame
        .producer_health_by_product
        .insert(product_id, ProducerHealth::Healthy);
    viewport_observations.upsert_frame(frame);

    dispatch_shell_command(
        &mut app,
        None,
        ShellCommand::SelectViewportProduct {
            viewport_id,
            product_id,
            target: StructuralCommandTarget {
                panel_instance_id: editor_shell::PanelInstanceId::new(1),
                active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::new(1)),
                tab_stack_id: editor_shell::TabStackId::new(1),
            },
            projection_epoch: 0,
        },
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
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
        ShellCommand::SelectViewportProduct {
            viewport_id: viewport_b,
            product_id: product_picking,
            target: StructuralCommandTarget {
                panel_instance_id: editor_shell::PanelInstanceId::new(1),
                active_tool_surface: Some(editor_shell::ToolSurfaceInstanceId::new(1)),
                tab_stack_id: editor_shell::TabStackId::new(1),
            },
            projection_epoch: 0,
        },
        Some(&mut viewport_presentations),
        Some(&viewport_observations),
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
fn dispatch_shell_command_toggles_viewport_details_visibility() {
    let mut app = RunenwerkEditorApp::new();
    assert!(!app.viewport_details_visible());

    dispatch_shell_command(
        &mut app,
        None,
        ShellCommand::ToggleViewportDetails,
        None,
        None,
        None,
    )
    .expect("viewport details toggle shell command should succeed");
    assert!(app.viewport_details_visible());

    dispatch_shell_command(
        &mut app,
        None,
        ShellCommand::ToggleViewportDetails,
        None,
        None,
        None,
    )
    .expect("viewport details toggle shell command should succeed");
    assert!(!app.viewport_details_visible());
}

#[test]
fn dispatch_shell_command_records_workflow_dispatch_event() {
    let mut app = RunenwerkEditorApp::new();

    dispatch_shell_command(&mut app, None, ShellCommand::NoOp, None, None, None)
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
    assert!(app.console_follow_enabled());

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
        !app.console_follow_enabled(),
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
        if app.console_follow_enabled() {
            break;
        }
    }
    assert!(
        app.console_follow_enabled(),
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

    app.set_console_follow_enabled(false);
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
fn clear_cached_projection_keeps_shell_identity_unchanged() {
    let mut shell_state = RunenwerkEditorShellState::new();
    let workspace_before = shell_state.workspace_id();
    let workspace_state_before = shell_state.workspace_state().clone();
    shell_state.set_last_projection_artifacts(
        editor_shell::build_editor_shell(
            &build_editor_shell_view_model(&RunenwerkEditorApp::new()),
            &ThemeTokens::default(),
            shell_state.workspace_state(),
        )
        .projection_artifacts,
    );

    shell_state.clear_cached_projection();

    assert_eq!(shell_state.workspace_id(), workspace_before);
    assert_eq!(*shell_state.workspace_state(), workspace_state_before);
    assert!(shell_state.last_projection_artifacts().is_none());
    assert!(shell_state.last_tree().is_none());
    assert!(shell_state.last_bounds().is_none());
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
        dispatch_shell_command(&mut app, None, command, None, None, Some(current_epoch))
            .expect("stale command dispatch should fail closed without error");
    }

    assert_eq!(app.outliner_state().selected_entity, None);
    assert_eq!(app.runtime().workflow_log().len(), workflow_log_len_before);
}

#[test]
fn docking_tab_move_keeps_panel_and_surface_identity_stable() {
    let app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let before = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner panel context should exist");
    let root_split = match shell_state
        .workspace_state()
        .host(shell_state.workspace_state().root_host_id())
        .expect("root host should exist")
        .kind
    {
        PanelHostKind::SplitHost(split) => split,
        _ => panic!("bootstrap root host should be split"),
    };

    let new_host_id = PanelHostId::new(970);
    let new_stack_id = TabStackId::new(980);
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::CreateTabStack {
            tab_stack_id: new_stack_id,
        })
        .expect("creating tab stack should succeed");
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::CreateHostNode {
            host_id: new_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: new_stack_id,
            }),
        })
        .expect("creating host should succeed");
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::SetHostToSplit {
            host_id: shell_state.workspace_state().root_host_id(),
            axis: WorkspaceSplitAxis::Horizontal,
            fraction: 0.5,
            first_child: new_host_id,
            second_child: root_split.first_child,
        })
        .expect("split host rehome should succeed");
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::MovePanelToTabStack {
            panel_id: before.panel_instance_id,
            source_tab_stack_id: before.tab_stack_id,
            destination_tab_stack_id: new_stack_id,
            destination_index: Some(0),
            activate_in_destination: true,
        })
        .expect("moving outliner panel into new stack should succeed");

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let after = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist after rebuild")
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner panel context should exist");

    assert_eq!(after.panel_instance_id, before.panel_instance_id);
    assert_eq!(after.active_tool_surface, before.active_tool_surface);
    assert_eq!(after.tab_stack_id, new_stack_id);
}

#[test]
fn stale_projection_commands_fail_closed_after_docking_mutation() {
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
        .expect("projection artifacts should exist")
        .clone();
    let outliner_context = stale_artifacts
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner panel context should exist");
    let root_split = match shell_state
        .workspace_state()
        .host(shell_state.workspace_state().root_host_id())
        .expect("root host should exist")
        .kind
    {
        PanelHostKind::SplitHost(split) => split,
        _ => panic!("bootstrap root host should be split"),
    };

    let new_host_id = PanelHostId::new(990);
    let new_stack_id = TabStackId::new(995);
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::CreateTabStack {
            tab_stack_id: new_stack_id,
        })
        .expect("creating tab stack should succeed");
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::CreateHostNode {
            host_id: new_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: new_stack_id,
            }),
        })
        .expect("creating host should succeed");
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::SetHostToSplit {
            host_id: shell_state.workspace_state().root_host_id(),
            axis: WorkspaceSplitAxis::Horizontal,
            fraction: 0.5,
            first_child: new_host_id,
            second_child: root_split.first_child,
        })
        .expect("split host rehome should succeed");
    shell_state
        .apply_workspace_mutation(WorkspaceMutation::MovePanelToTabStack {
            panel_id: outliner_context.panel_instance_id,
            source_tab_stack_id: outliner_context.tab_stack_id,
            destination_tab_stack_id: new_stack_id,
            destination_index: Some(0),
            activate_in_destination: true,
        })
        .expect("moving panel into new stack should succeed");

    let current_epoch = shell_state.current_projection_epoch();
    assert!(stale_artifacts.projection_epoch < current_epoch);

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(outliner_row_widget_id(0))],
        },
        &stale_artifacts,
    );
    let workflow_log_len_before = app.runtime().workflow_log().len();
    for command in commands {
        dispatch_shell_command(&mut app, None, command, None, None, Some(current_epoch))
            .expect("stale command should fail closed");
    }

    assert_eq!(app.outliner_state().selected_entity, None);
    assert_eq!(app.runtime().workflow_log().len(), workflow_log_len_before);
}

#[test]
fn dispatch_shell_command_activate_tab_updates_workspace_active_panel() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let projection = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let viewport_context = projection
        .workspace
        .widget_context_by_id
        .get(&VIEWPORT_PANEL_WIDGET_ID)
        .copied()
        .expect("viewport context should exist");
    let inspector_context = projection
        .workspace
        .widget_context_by_id
        .get(&INSPECTOR_PANEL_WIDGET_ID)
        .copied()
        .expect("inspector context should exist");
    assert_ne!(
        viewport_context.tab_stack_id,
        inspector_context.tab_stack_id
    );

    shell_state
        .apply_workspace_mutation(WorkspaceMutation::MovePanelToTabStack {
            panel_id: inspector_context.panel_instance_id,
            source_tab_stack_id: inspector_context.tab_stack_id,
            destination_tab_stack_id: viewport_context.tab_stack_id,
            destination_index: None,
            activate_in_destination: false,
        })
        .expect("moving inspector panel into viewport stack should succeed");

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::ActivateTab {
            tab_stack_id: viewport_context.tab_stack_id,
            panel_instance_id: inspector_context.panel_instance_id,
            projection_epoch: 0,
        },
        None,
        None,
        None,
    )
    .expect("activate-tab command should succeed");

    let viewport_stack = shell_state
        .workspace_state()
        .tab_stack(viewport_context.tab_stack_id)
        .expect("viewport stack should exist");
    assert_eq!(
        viewport_stack.active_panel,
        Some(inspector_context.panel_instance_id),
        "activate-tab command must set active panel by explicit structural ids",
    );
}

#[test]
fn dispatch_shell_command_float_panel_preserves_panel_and_surface_identity() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();
    let atlas = UiFontAtlasResource::default();

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let before = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner context should exist");

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::FloatPanel {
            tab_stack_id: before.tab_stack_id,
            panel_instance_id: before.panel_instance_id,
            projection_epoch: 0,
        },
        None,
        None,
        None,
    )
    .expect("float-panel command should succeed");

    let _ =
        RunenwerkEditorShellController::build_frame(&app, &mut shell_state, bounds, &theme, &atlas);
    let after = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist after float")
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner context should exist after float");

    assert_eq!(after.panel_instance_id, before.panel_instance_id);
    assert_eq!(after.active_tool_surface, before.active_tool_surface);
    assert_ne!(after.tab_stack_id, before.tab_stack_id);

    let root = shell_state
        .workspace_state()
        .host(shell_state.workspace_state().root_host_id())
        .expect("root host should exist");
    let floating_host_id = match root.kind {
        PanelHostKind::SplitHost(split) => split.second_child,
        _ => panic!("float command should install split root host"),
    };
    let floating_host = shell_state
        .workspace_state()
        .host(floating_host_id)
        .expect("floating host should exist");
    assert!(
        matches!(
            floating_host.kind,
            PanelHostKind::FloatingHostPlaceholder(_)
        ),
        "split root second child must be floating host placeholder",
    );
}

#[test]
fn tab_drag_drop_moves_panel_between_tab_stacks_with_stable_identity() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
    let theme = ThemeTokens::default();

    let _ = RunenwerkEditorShellController::rebuild_tree(&app, &mut shell_state, &theme);
    let projection = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .clone();
    let outliner_context = projection
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner context should exist");
    let inspector_context = projection
        .workspace
        .widget_context_by_id
        .get(&INSPECTOR_PANEL_WIDGET_ID)
        .copied()
        .expect("inspector context should exist");

    let outliner_tab_button = projection
        .workspace
        .tab_button_by_widget_id
        .iter()
        .find_map(|(widget_id, tab)| {
            if tab.panel_kind == PanelKind::Outliner {
                Some(*widget_id)
            } else {
                None
            }
        })
        .expect("outliner tab button should exist");

    let down_position = center_for_widget(&shell_state, bounds, outliner_tab_button);
    let up_position = center_for_widget(&shell_state, bounds, INSPECTOR_PANEL_WIDGET_ID);

    RunenwerkEditorShellController::dispatch_input(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        &pointer_event(
            PointerEventKind::Down,
            down_position,
            Some(ui_input::PointerButton::Primary),
        ),
    )
    .expect("tab pointer-down should succeed");
    RunenwerkEditorShellController::dispatch_input(
        &mut app,
        &mut shell_state,
        bounds,
        &theme,
        &pointer_event(
            PointerEventKind::Up,
            up_position,
            Some(ui_input::PointerButton::Primary),
        ),
    )
    .expect("tab pointer-up should succeed");

    let _ = RunenwerkEditorShellController::rebuild_tree(&app, &mut shell_state, &theme);
    let moved_context = shell_state
        .last_projection_artifacts()
        .expect("projection artifacts should exist")
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner context should exist after drag-drop");

    assert_eq!(
        moved_context.panel_instance_id,
        outliner_context.panel_instance_id
    );
    assert_eq!(
        moved_context.active_tool_surface,
        outliner_context.active_tool_surface
    );
    assert_eq!(moved_context.tab_stack_id, inspector_context.tab_stack_id);
}
