use editor_shell::{
    BODY_CONSOLE_SPLIT_WIDGET_ID, CENTER_RIGHT_SPLIT_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID,
    ComputedLayoutMap, DockingPreviewDropTarget, EditorDomainMutation, EditorShellFrameModel,
    LEFT_RIGHT_SPLIT_WIDGET_ID, PanelHostId, ProjectedTabStackSlot, ProjectedWorkspaceHostSlot,
    ShellCommand, ShellUiExpressionFrame, SurfaceCommandProposal, SurfaceSessionMutation,
    TOOLBAR_ADD_WORKSPACE_WIDGET_ID, TOOLBAR_EDIT_MENU_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID,
    TOOLBAR_MENU_POPUP_WIDGET_ID, TOOLBAR_WINDOW_MENU_WIDGET_ID, TabDropDestination,
    ToolSurfaceKind, UiInputOutcome, UiInteractionResults, UiTree, WidgetId, WorkspaceSplitAxis,
    build_editor_shell_frame_with_docking_visual_state, map_interactions_to_shell_commands,
    tab_stack_action_menu_popup_widget_id, tab_stack_container_widget_id,
    tab_stack_surface_menu_popup_widget_id,
};
use editor_viewport::ArtifactObservationFrame;
use ui_input::{
    EventPropagation, FocusChange, InputResponse, PointerCapture, PointerEvent, PointerEventKind,
    UiInputEvent,
};
use ui_math::UiRect;
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource,
};
use crate::shell::{
    CornerAreaSplitSession, CornerSplitResizeSession, EditorSurfaceProviderRegistry,
    RunenwerkEditorShellState, SurfaceProviderDispatchContext, active_document_context,
    build_editor_shell_frame_model, dispatch_shell_command, mounted_surface_requests,
};
use editor_shell::TabStackPopupMenuKind;

const CONSOLE_FOLLOW_BOTTOM_EPSILON: f32 = 1.0;
const SPLIT_HIT_SLOP_PX: f32 = 12.0;
const SPLIT_MIN_FRACTION: f32 = 0.08;
const SPLIT_MAX_FRACTION: f32 = 0.92;
const CORNER_AREA_SPLIT_THRESHOLD_PX: f32 = 18.0;

pub struct RunenwerkEditorShellController;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellCursorIntent {
    Default,
    ResizeColumn,
    ResizeRow,
    ResizeNwse,
    ResizeNesw,
    Grab,
    Grabbing,
}

impl RunenwerkEditorShellController {
    pub fn rebuild_tree(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        theme: &ThemeTokens,
    ) -> UiTree {
        Self::rebuild_tree_with_viewport_products(app, shell_state, theme, None)
    }

    pub fn rebuild_tree_with_viewport_products(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        theme: &ThemeTokens,
        _viewport_products: Option<&ArtifactObservationFrame>,
    ) -> UiTree {
        let frame_model = Self::rebuild_frame_model_with_provider_context(
            app,
            shell_state,
            theme,
            app.surface_provider_registry(),
            None,
            None,
        );
        let docking_visual_state = shell_state.docking_visual_state();
        let build_result = build_editor_shell_frame_with_docking_visual_state(
            &frame_model,
            theme,
            shell_state.workspace_state(),
            Some(&docking_visual_state),
        );
        let tree = build_result.tree;
        shell_state.cache_projection_artifacts(build_result.projection_artifacts);
        shell_state.set_last_tree(tree.clone());
        tree
    }

    pub fn build_frame(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
    ) -> UiFrame {
        Self::build_expression_frame(app, shell_state, bounds, theme, atlas_source).into_ui_frame()
    }

    pub fn build_frame_with_viewport_products(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        viewport_products: Option<&ArtifactObservationFrame>,
    ) -> UiFrame {
        Self::build_expression_frame_with_viewport_products(
            app,
            shell_state,
            bounds,
            theme,
            atlas_source,
            viewport_products,
        )
        .into_ui_frame()
    }

    pub fn build_expression_frame(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
    ) -> ShellUiExpressionFrame {
        Self::build_expression_frame_with_viewport_products(
            app,
            shell_state,
            bounds,
            theme,
            atlas_source,
            None,
        )
    }

    pub fn build_expression_frame_with_viewport_products(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        viewport_products: Option<&ArtifactObservationFrame>,
    ) -> ShellUiExpressionFrame {
        let tree =
            Self::rebuild_tree_with_viewport_products(app, shell_state, theme, viewport_products);
        Self::finish_expression_frame(app, shell_state, bounds, atlas_source, tree)
    }

    pub fn build_expression_frame_with_surface_resources(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    ) -> ShellUiExpressionFrame {
        let frame_model = Self::rebuild_frame_model_with_provider_context(
            app,
            shell_state,
            theme,
            app.surface_provider_registry(),
            viewport_observations,
            tool_surface_bindings,
        );
        let docking_visual_state = shell_state.docking_visual_state();
        let build_result = build_editor_shell_frame_with_docking_visual_state(
            &frame_model,
            theme,
            shell_state.workspace_state(),
            Some(&docking_visual_state),
        );
        let tree = build_result.tree;
        shell_state.cache_projection_artifacts(build_result.projection_artifacts);
        shell_state.set_last_tree(tree.clone());
        Self::finish_expression_frame(app, shell_state, bounds, atlas_source, tree)
    }

    fn finish_expression_frame(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        atlas_source: &dyn FontAtlasSource,
        tree: UiTree,
    ) -> ShellUiExpressionFrame {
        shell_state.set_last_bounds(bounds);
        if console_follow_enabled_for_active_surface(app, shell_state)
            && let Some(max_offset) =
                shell_state
                    .runtime()
                    .max_scroll_offset(&tree, bounds, CONSOLE_SCROLL_WIDGET_ID)
        {
            shell_state
                .runtime_mut()
                .set_scroll_offset(CONSOLE_SCROLL_WIDGET_ID, max_offset);
        }
        shell_state.runtime_mut().state_mut().advance_frame();
        let frame = shell_state
            .runtime()
            .build_frame(&tree, bounds, atlas_source);

        ShellUiExpressionFrame::new(app.runtime().current_scene_reality_version(), frame)
    }

    #[allow(clippy::too_many_arguments)]
    fn rebuild_frame_model_with_provider_context(
        app: &RunenwerkEditorApp,
        shell_state: &RunenwerkEditorShellState,
        theme: &ThemeTokens,
        registry: &EditorSurfaceProviderRegistry,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    ) -> EditorShellFrameModel {
        build_editor_shell_frame_model(
            app,
            shell_state,
            registry,
            theme,
            viewport_observations,
            tool_surface_bindings,
        )
    }

    pub fn dispatch_input(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        event: &UiInputEvent,
    ) -> Result<UiInputOutcome, editor_core::EditorMutationError> {
        Self::dispatch_input_with_viewport_products(
            app,
            shell_state,
            bounds,
            theme,
            event,
            None,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_input_with_viewport_products(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        event: &UiInputEvent,
        _viewport_products: Option<&ArtifactObservationFrame>,
        viewport_presentations: Option<&mut ViewportPresentationStateResource>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    ) -> Result<UiInputOutcome, editor_core::EditorMutationError> {
        let registry = app.surface_provider_registry_handle();
        let frame_model = Self::rebuild_frame_model_with_provider_context(
            app,
            shell_state,
            theme,
            registry.as_ref(),
            viewport_observations,
            tool_surface_bindings,
        );
        let docking_visual_state = shell_state.docking_visual_state();
        let build_result = build_editor_shell_frame_with_docking_visual_state(
            &frame_model,
            theme,
            shell_state.workspace_state(),
            Some(&docking_visual_state),
        );
        let tree = build_result.tree.clone();
        let projection_artifacts =
            shell_state.cache_projection_artifacts(build_result.projection_artifacts);
        shell_state.set_last_tree(tree.clone());
        shell_state.set_last_bounds(bounds);

        let layouts = shell_state.runtime().compute_layout(&tree, bounds);
        let pre_offset = shell_state
            .runtime()
            .scroll_offset(CONSOLE_SCROLL_WIDGET_ID);
        let pre_max = shell_state
            .runtime()
            .max_scroll_offset_for_layout(&tree, &layouts, CONSOLE_SCROLL_WIDGET_ID)
            .unwrap_or(0.0);
        let pre_at_bottom = is_at_bottom(pre_offset, pre_max);

        if let Some(outcome) = Self::handle_corner_area_split_event(
            app,
            shell_state,
            event,
            &layouts,
            &projection_artifacts,
        )? {
            return Ok(outcome);
        }

        if let Some(outcome) =
            Self::handle_split_resize_event(shell_state, event, &layouts, &projection_artifacts)
        {
            return Ok(outcome);
        }
        if let Some(outcome) =
            Self::handle_tab_context_menu_event(shell_state, event, &layouts, &projection_artifacts)
        {
            return Ok(outcome);
        }
        if let Some(outcome) = Self::handle_tab_popup_dismiss_event(shell_state, event, &layouts) {
            return Ok(outcome);
        }
        if let Some(outcome) = Self::handle_toolbar_menu_dismiss_event(shell_state, event, &layouts)
        {
            return Ok(outcome);
        }

        let outcome = shell_state
            .runtime_mut()
            .dispatch_input(&tree, &layouts, event);

        if is_console_scroll_event(event, &outcome) {
            let post_layouts = shell_state.runtime().compute_layout(&tree, bounds);
            let post_offset = shell_state
                .runtime()
                .scroll_offset(CONSOLE_SCROLL_WIDGET_ID);
            let post_max = shell_state
                .runtime()
                .max_scroll_offset_for_layout(&tree, &post_layouts, CONSOLE_SCROLL_WIDGET_ID)
                .unwrap_or(0.0);
            let post_at_bottom = is_at_bottom(post_offset, post_max);
            if let Some(surface_id) = active_console_surface(shell_state) {
                if post_at_bottom {
                    app.surface_sessions_mut()
                        .session_mut(surface_id)
                        .console_follow_enabled = true;
                } else if pre_at_bottom {
                    app.surface_sessions_mut()
                        .session_mut(surface_id)
                        .console_follow_enabled = false;
                }
            }
        }

        let (mut docking_commands, suppress_tab_activation) = Self::collect_docking_commands(
            shell_state,
            event,
            &tree,
            &layouts,
            &projection_artifacts,
        );
        if !suppress_tab_activation
            && let UiInputEvent::Pointer(pointer) = event
            && pointer.kind == PointerEventKind::Up
            && pointer.button == Some(ui_input::PointerButton::Primary)
            && let Some(route) =
                tab_button_route_at_pointer(&projection_artifacts, &layouts, pointer.position)
        {
            docking_commands.push(ShellCommand::SetTabStackActivePanel {
                tab_stack_id: route.tab_stack_id,
                panel_instance_id: route.panel_instance_id,
                projection_epoch: projection_artifacts.projection_epoch,
            });
        }

        let mut commands =
            map_interactions_to_shell_commands(&outcome.interactions, &projection_artifacts);
        if suppress_tab_activation {
            commands
                .retain(|command| !matches!(command, ShellCommand::SetTabStackActivePanel { .. }));
        }
        commands.append(&mut docking_commands);

        Self::dispatch_commands(
            app,
            shell_state,
            commands,
            registry.as_ref(),
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
        )?;

        Ok(outcome)
    }

    fn handle_tab_context_menu_event(
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
        projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        if !matches!(pointer.kind, PointerEventKind::Down)
            || pointer.button != Some(ui_input::PointerButton::Secondary)
        {
            return None;
        }
        let (anchor_widget_id, route) =
            tab_button_hit_at_pointer(projection_artifacts, layouts, pointer.position)?;
        shell_state.open_tab_stack_popup_menu(
            TabStackPopupMenuKind::AreaActions,
            route.tab_stack_id,
            anchor_widget_id,
        );
        Some(consumed_pointer_outcome(Some(anchor_widget_id), true))
    }

    fn handle_tab_popup_dismiss_event(
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        if !matches!(pointer.kind, PointerEventKind::Down) {
            return None;
        }
        if !matches!(
            pointer.button,
            Some(ui_input::PointerButton::Primary | ui_input::PointerButton::Secondary)
        ) {
            return None;
        }
        let active_menu = shell_state.active_tab_stack_popup_menu()?;
        let area_popup = tab_stack_action_menu_popup_widget_id(active_menu.tab_stack_id);
        let surface_popup = tab_stack_surface_menu_popup_widget_id(active_menu.tab_stack_id);
        let inside_area = layouts
            .get(&area_popup)
            .is_some_and(|layout| layout.bounds.contains(pointer.position));
        let inside_surface = layouts
            .get(&surface_popup)
            .is_some_and(|layout| layout.bounds.contains(pointer.position));
        if inside_area || inside_surface {
            return None;
        }
        shell_state.close_tab_stack_action_menu();
        Some(consumed_pointer_outcome(None, true))
    }

    fn handle_toolbar_menu_dismiss_event(
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        if shell_state.active_toolbar_menu().is_none()
            || !matches!(pointer.kind, PointerEventKind::Down)
            || !matches!(
                pointer.button,
                Some(ui_input::PointerButton::Primary | ui_input::PointerButton::Secondary)
            )
        {
            return None;
        }
        let toolbar_widgets = [
            TOOLBAR_FILE_MENU_WIDGET_ID,
            TOOLBAR_EDIT_MENU_WIDGET_ID,
            TOOLBAR_WINDOW_MENU_WIDGET_ID,
            TOOLBAR_ADD_WORKSPACE_WIDGET_ID,
            TOOLBAR_MENU_POPUP_WIDGET_ID,
        ];
        let inside_toolbar_menu = toolbar_widgets.iter().any(|widget_id| {
            layouts
                .get(widget_id)
                .is_some_and(|layout| layout.bounds.contains(pointer.position))
        });
        if inside_toolbar_menu {
            return None;
        }
        shell_state.close_toolbar_menu();
        Some(consumed_pointer_outcome(None, true))
    }

    fn handle_corner_area_split_event(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
        projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    ) -> Result<Option<UiInputOutcome>, editor_core::EditorMutationError> {
        let UiInputEvent::Pointer(pointer) = event else {
            return Ok(None);
        };

        if let Some(session) = shell_state.active_corner_area_split_session() {
            return match pointer.kind {
                PointerEventKind::Move | PointerEventKind::Enter => {
                    let delta = pointer.position - session.pointer_down;
                    let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
                    if distance < CORNER_AREA_SPLIT_THRESHOLD_PX {
                        return Ok(Some(consumed_pointer_outcome(None, false)));
                    }
                    let axis = if delta.x.abs() >= delta.y.abs() {
                        WorkspaceSplitAxis::Horizontal
                    } else {
                        WorkspaceSplitAxis::Vertical
                    };
                    let tool_surface_kind =
                        active_tool_surface_kind_for_tab_stack(shell_state, session.tab_stack_id);
                    let command = ShellCommand::SplitTabStackArea {
                        tab_stack_id: session.tab_stack_id,
                        axis,
                        tool_surface_kind,
                        projection_epoch: session.projection_epoch,
                    };
                    shell_state.clear_corner_area_split();
                    dispatch_shell_command(
                        app,
                        Some(shell_state),
                        command,
                        None,
                        None,
                        None,
                        Some(session.projection_epoch),
                    )?;
                    Ok(Some(consumed_pointer_outcome(None, true)))
                }
                PointerEventKind::Up | PointerEventKind::Leave => {
                    shell_state.clear_corner_area_split();
                    Ok(Some(consumed_pointer_outcome(None, false)))
                }
                PointerEventKind::Down | PointerEventKind::Scroll => {
                    Ok(Some(consumed_pointer_outcome(None, false)))
                }
            };
        }

        if !matches!(pointer.kind, PointerEventKind::Down)
            || pointer.button != Some(ui_input::PointerButton::Primary)
            || !pointer.modifiers.shift
        {
            return Ok(None);
        }
        let Some(tab_stack) =
            tab_stack_corner_at_pointer(pointer.position, layouts, projection_artifacts)
        else {
            return Ok(None);
        };
        shell_state.begin_corner_area_split(CornerAreaSplitSession {
            tab_stack_id: tab_stack.tab_stack_id,
            pointer_down: pointer.position,
            projection_epoch: projection_artifacts.projection_epoch,
        });
        Ok(Some(consumed_pointer_outcome(
            Some(tab_stack_container_widget_id(tab_stack.tab_stack_id)),
            false,
        )))
    }

    fn collect_docking_commands(
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    ) -> (Vec<ShellCommand>, bool) {
        let mut commands = Vec::new();
        let mut suppress_tab_activation = false;

        let UiInputEvent::Pointer(pointer) = event else {
            return (commands, suppress_tab_activation);
        };

        match pointer.kind {
            PointerEventKind::Down => {
                if pointer.button != Some(ui_input::PointerButton::Primary) {
                    return (commands, suppress_tab_activation);
                }
                let pressed = shell_state.runtime().state().pressed_widget;
                let route =
                    tab_button_route_at_pointer(projection_artifacts, layouts, pointer.position)
                        .or_else(|| {
                            pressed.and_then(|pressed_widget| {
                                projection_artifacts
                                    .workspace
                                    .tab_button_route_by_widget_id
                                    .get(&pressed_widget)
                                    .copied()
                            })
                        });
                if let Some(route) = route {
                    shell_state.begin_tab_drag_candidate(
                        route.panel_instance_id,
                        route.tab_stack_id,
                        pointer.position,
                        projection_artifacts.projection_epoch,
                    );
                } else {
                    shell_state.clear_tab_drag();
                }
            }
            PointerEventKind::Move | PointerEventKind::Enter => {
                let became_active = shell_state.update_tab_drag_pointer(
                    pointer.position,
                    projection_artifacts.projection_epoch,
                );
                if became_active || shell_state.docking_visual_state().active_tab_drag.is_some() {
                    suppress_tab_activation = true;
                    let preview_target = resolve_tab_drop_preview_target(
                        projection_artifacts,
                        tree,
                        layouts,
                        pointer.position,
                    );
                    shell_state.set_tab_drag_preview_target(
                        preview_target,
                        projection_artifacts.projection_epoch,
                    );
                }
            }
            PointerEventKind::Leave => {
                shell_state
                    .set_tab_drag_preview_target(None, projection_artifacts.projection_epoch);
            }
            PointerEventKind::Up => {
                let click_candidate = shell_state.tab_drag_candidate();
                let preview_target = resolve_tab_drop_preview_target(
                    projection_artifacts,
                    tree,
                    layouts,
                    pointer.position,
                );
                shell_state.set_tab_drag_preview_target(
                    preview_target,
                    projection_artifacts.projection_epoch,
                );
                let finished = shell_state.finish_tab_drag(projection_artifacts.projection_epoch);
                if let Some((panel_instance_id, source_tab_stack_id, preview_target, drag_epoch)) =
                    finished
                {
                    suppress_tab_activation = true;
                    if let Some(destination) = preview_target.map(|target| match target {
                        DockingPreviewDropTarget::TabStack {
                            tab_stack_id,
                            insert_index,
                        } => TabDropDestination::TabStack {
                            tab_stack_id,
                            insert_index,
                        },
                        DockingPreviewDropTarget::NewFloatingHost => {
                            TabDropDestination::NewFloatingHost
                        }
                    }) {
                        commands.push(ShellCommand::CommitTabDrop {
                            panel_instance_id,
                            source_tab_stack_id,
                            destination,
                            projection_epoch: drag_epoch,
                        });
                    }
                } else if let Some((panel_instance_id, tab_stack_id, _drag_epoch, false)) =
                    click_candidate
                {
                    commands.push(ShellCommand::SetTabStackActivePanel {
                        tab_stack_id,
                        panel_instance_id,
                        projection_epoch: projection_artifacts.projection_epoch,
                    });
                    suppress_tab_activation = true;
                }
            }
            PointerEventKind::Scroll => {}
        }

        (commands, suppress_tab_activation)
    }

    pub(crate) fn dispatch_commands(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        commands: Vec<ShellCommand>,
        registry: &EditorSurfaceProviderRegistry,
        mut viewport_presentations: Option<&mut ViewportPresentationStateResource>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    ) -> Result<(), editor_core::EditorMutationError> {
        for command in commands {
            if let Some(projection_epoch) = command.projection_epoch()
                && !shell_state.is_projection_epoch_current(projection_epoch)
            {
                continue;
            }
            let current_epoch = shell_state.current_projection_epoch();

            if let ShellCommand::DispatchSurfaceLocalAction {
                provider_id,
                tool_surface_instance_id,
                target,
                action,
                projection_epoch,
            } = command
            {
                if !shell_state.is_projection_epoch_current(projection_epoch)
                    || target.active_tool_surface != Some(tool_surface_instance_id)
                {
                    continue;
                }
                let document_context = active_document_context(app);
                let Some(request) = mounted_surface_requests(shell_state, document_context)
                    .into_iter()
                    .find(|request| {
                        request.tool_surface_instance_id == tool_surface_instance_id
                            && request.panel_instance_id == target.panel_instance_id
                            && request.tab_stack_id == target.tab_stack_id
                    })
                else {
                    continue;
                };
                let dispatch_context = SurfaceProviderDispatchContext {
                    projection_epoch: current_epoch,
                    _marker: std::marker::PhantomData,
                };
                if let Some(proposal) =
                    registry.map_action(&dispatch_context, &request, provider_id, action)?
                    && let Some(command) = shell_command_from_surface_proposal(proposal)
                {
                    dispatch_shell_command(
                        app,
                        Some(shell_state),
                        command,
                        viewport_presentations.as_deref_mut(),
                        viewport_observations,
                        tool_surface_bindings,
                        Some(current_epoch),
                    )?;
                }
                continue;
            }

            dispatch_shell_command(
                app,
                Some(shell_state),
                command,
                viewport_presentations.as_deref_mut(),
                viewport_observations,
                tool_surface_bindings,
                Some(current_epoch),
            )?;
        }

        Ok(())
    }

    pub fn cursor_intent_for_pointer(
        shell_state: &RunenwerkEditorShellState,
        pointer: ui_math::UiPoint,
    ) -> ShellCursorIntent {
        if shell_state.active_corner_split_resize_session().is_some() {
            return ShellCursorIntent::ResizeNwse;
        }
        if let Some((_, _, axis)) = shell_state.active_split_resize_session() {
            return split_cursor_intent(axis);
        }
        if shell_state.docking_visual_state().active_tab_drag.is_some() {
            return ShellCursorIntent::Grabbing;
        }

        let (Some(tree), Some(bounds), Some(projection_artifacts)) = (
            shell_state.last_tree(),
            shell_state.last_bounds(),
            shell_state.last_projection_artifacts(),
        ) else {
            return ShellCursorIntent::Default;
        };
        let layouts = shell_state.runtime().compute_layout(tree, bounds);
        if let Some(corner) =
            resolve_split_corner_resize_target(pointer, &layouts, projection_artifacts)
        {
            return corner.cursor_intent;
        }
        resolve_split_resize_target(pointer, &layouts, projection_artifacts)
            .map(|target| split_cursor_intent(target.axis))
            .unwrap_or(ShellCursorIntent::Default)
    }
}

fn shell_command_from_surface_proposal(proposal: SurfaceCommandProposal) -> Option<ShellCommand> {
    match proposal {
        SurfaceCommandProposal::SurfaceSession(proposal) => match proposal.mutation {
            SurfaceSessionMutation::AppendEntityTableSearchText { text } => {
                Some(ShellCommand::AppendEntityTableSearchText {
                    text,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::BackspaceEntityTableSearch => {
                Some(ShellCommand::BackspaceEntityTableSearch {
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::ToggleEntityTableSort { sort_key } => {
                Some(ShellCommand::ToggleEntityTableSort {
                    sort_key,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::ToggleViewportDetails => {
                Some(ShellCommand::ToggleViewportDetails {
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::ToggleViewportStatistics => {
                Some(ShellCommand::ToggleViewportStatistics {
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::ToggleViewportOptionsMenu => {
                Some(ShellCommand::ToggleViewportOptionsMenu {
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::ActivateInspectorField { index } => {
                Some(ShellCommand::ActivateInspectorField {
                    index,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::FocusInspectorField { index } => {
                Some(ShellCommand::FocusInspectorField {
                    index,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::AppendInspectorFieldText { index, text } => {
                Some(ShellCommand::AppendInspectorFieldText {
                    index,
                    text,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::BackspaceInspectorFieldText { index } => {
                Some(ShellCommand::BackspaceInspectorFieldText {
                    index,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::CommitInspectorFieldText { index } => {
                Some(ShellCommand::CommitInspectorFieldText {
                    index,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            SurfaceSessionMutation::CancelInspectorFieldText { index } => {
                Some(ShellCommand::CancelInspectorFieldText {
                    index,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
        },
        SurfaceCommandProposal::EditorDomain(proposal) => match proposal.mutation {
            EditorDomainMutation::SelectOutlinerEntity { entity } => {
                Some(ShellCommand::SelectOutlinerEntity {
                    entity,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            EditorDomainMutation::SelectEntityTableRow { entities } => {
                let entity = entities.first().copied()?;
                Some(ShellCommand::SelectEntityTableEntity {
                    entity,
                    target: proposal.target,
                    projection_epoch: proposal.projection_epoch,
                })
            }
            EditorDomainMutation::SelectViewportProduct {
                viewport_id,
                product_id,
            } => Some(ShellCommand::SelectViewportProduct {
                viewport_id,
                product_id,
                target: proposal.target,
                projection_epoch: proposal.projection_epoch,
            }),
        },
        SurfaceCommandProposal::Shell(command) => Some(command),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SplitResizeTarget {
    split_host_id: PanelHostId,
    widget_id: WidgetId,
    axis: WorkspaceSplitAxis,
    fraction: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SplitCornerResizeTarget {
    horizontal: SplitResizeTarget,
    vertical: SplitResizeTarget,
    cursor_intent: ShellCursorIntent,
    aspect_ratio: f32,
}

impl RunenwerkEditorShellController {
    fn handle_split_resize_event(
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
        projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };

        if let Some(session) = shell_state.active_corner_split_resize_session() {
            match pointer.kind {
                PointerEventKind::Move | PointerEventKind::Enter => {
                    let Some(horizontal_target) = split_resize_target_by_host(
                        projection_artifacts,
                        session.horizontal_split_host_id,
                    ) else {
                        shell_state.clear_split_resize();
                        return Some(consumed_pointer_outcome(
                            Some(session.horizontal_split_widget_id),
                            false,
                        ));
                    };
                    let Some(vertical_target) = split_resize_target_by_host(
                        projection_artifacts,
                        session.vertical_split_host_id,
                    ) else {
                        shell_state.clear_split_resize();
                        return Some(consumed_pointer_outcome(
                            Some(session.horizontal_split_widget_id),
                            false,
                        ));
                    };
                    let horizontal_layout = layouts.get(&session.horizontal_split_widget_id)?;
                    let vertical_layout = layouts.get(&session.vertical_split_widget_id)?;
                    let mut horizontal_fraction = split_fraction_from_pointer(
                        WorkspaceSplitAxis::Horizontal,
                        horizontal_layout.bounds,
                        pointer,
                    )
                    .clamp(SPLIT_MIN_FRACTION, SPLIT_MAX_FRACTION);
                    let mut vertical_fraction = split_fraction_from_pointer(
                        WorkspaceSplitAxis::Vertical,
                        vertical_layout.bounds,
                        pointer,
                    )
                    .clamp(SPLIT_MIN_FRACTION, SPLIT_MAX_FRACTION);
                    if pointer.modifiers.shift && session.aspect_ratio > 0.0 {
                        let width = horizontal_layout.bounds.width * horizontal_fraction;
                        let height = width / session.aspect_ratio;
                        vertical_fraction = (height / vertical_layout.bounds.height.max(1.0))
                            .clamp(SPLIT_MIN_FRACTION, SPLIT_MAX_FRACTION);
                        horizontal_fraction = (width / horizontal_layout.bounds.width.max(1.0))
                            .clamp(SPLIT_MIN_FRACTION, SPLIT_MAX_FRACTION);
                    }
                    let horizontal_changed =
                        (horizontal_fraction - horizontal_target.fraction).abs() > 0.001;
                    let vertical_changed =
                        (vertical_fraction - vertical_target.fraction).abs() > 0.001;
                    if horizontal_changed {
                        let _ = shell_state.set_workspace_split_host_fraction(
                            session.horizontal_split_host_id,
                            horizontal_fraction,
                        );
                    }
                    if vertical_changed {
                        let _ = shell_state.set_workspace_split_host_fraction(
                            session.vertical_split_host_id,
                            vertical_fraction,
                        );
                    }
                    return Some(consumed_pointer_outcome(
                        Some(session.horizontal_split_widget_id),
                        horizontal_changed || vertical_changed,
                    ));
                }
                PointerEventKind::Up => {
                    shell_state.clear_split_resize();
                    return Some(consumed_pointer_outcome(
                        Some(session.horizontal_split_widget_id),
                        false,
                    ));
                }
                PointerEventKind::Down | PointerEventKind::Leave | PointerEventKind::Scroll => {
                    return Some(consumed_pointer_outcome(
                        Some(session.horizontal_split_widget_id),
                        false,
                    ));
                }
            }
        }

        if let Some((split_host_id, split_widget_id, split_axis)) =
            shell_state.active_split_resize_session()
        {
            match pointer.kind {
                PointerEventKind::Move | PointerEventKind::Enter => {
                    let Some(active_target) =
                        split_resize_target_by_host(projection_artifacts, split_host_id)
                    else {
                        shell_state.clear_split_resize();
                        return Some(consumed_pointer_outcome(Some(split_widget_id), false));
                    };
                    let split_layout = layouts.get(&split_widget_id)?;
                    let next_fraction =
                        split_fraction_from_pointer(split_axis, split_layout.bounds, pointer)
                            .clamp(SPLIT_MIN_FRACTION, SPLIT_MAX_FRACTION);
                    let current_fraction = active_target.fraction;
                    let changed = (next_fraction - current_fraction).abs() > 0.001;
                    if changed {
                        let _ = shell_state
                            .set_workspace_split_host_fraction(split_host_id, next_fraction);
                    }
                    return Some(consumed_pointer_outcome(Some(split_widget_id), changed));
                }
                PointerEventKind::Up => {
                    shell_state.clear_split_resize();
                    return Some(consumed_pointer_outcome(Some(split_widget_id), false));
                }
                PointerEventKind::Down | PointerEventKind::Leave | PointerEventKind::Scroll => {
                    return Some(consumed_pointer_outcome(Some(split_widget_id), false));
                }
            }
        }

        if !matches!(pointer.kind, PointerEventKind::Down)
            || pointer.button != Some(ui_input::PointerButton::Primary)
        {
            return None;
        }
        if pointer.modifiers.shift {
            return None;
        }

        if let Some(corner) =
            resolve_split_corner_resize_target(pointer.position, layouts, projection_artifacts)
        {
            shell_state.begin_workspace_corner_split_resize(CornerSplitResizeSession {
                horizontal_split_host_id: corner.horizontal.split_host_id,
                horizontal_split_widget_id: corner.horizontal.widget_id,
                vertical_split_host_id: corner.vertical.split_host_id,
                vertical_split_widget_id: corner.vertical.widget_id,
                aspect_ratio: corner.aspect_ratio,
            });
            return Some(consumed_pointer_outcome(
                Some(corner.horizontal.widget_id),
                false,
            ));
        }

        let candidate =
            resolve_split_resize_target(pointer.position, layouts, projection_artifacts)?;
        if tab_button_route_at_pointer(projection_artifacts, layouts, pointer.position).is_some() {
            return None;
        }
        shell_state.begin_workspace_split_resize(
            candidate.split_host_id,
            candidate.widget_id,
            candidate.axis,
        );
        Some(consumed_pointer_outcome(Some(candidate.widget_id), false))
    }
}

fn resolve_split_resize_target(
    cursor: ui_math::UiPoint,
    layouts: &ComputedLayoutMap,
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
) -> Option<SplitResizeTarget> {
    split_resize_targets(projection_artifacts)
        .into_iter()
        .filter_map(|target| {
            let layout = layouts.get(&target.widget_id)?;
            let bounds = layout.bounds;
            if !bounds.contains(cursor) {
                return None;
            }
            let boundary = split_boundary_position(target.axis, bounds, target.fraction);
            let distance = match workspace_axis_to_ui_axis(target.axis) {
                ui_math::Axis::Horizontal => (cursor.x - boundary).abs(),
                ui_math::Axis::Vertical => (cursor.y - boundary).abs(),
            };
            (distance <= SPLIT_HIT_SLOP_PX).then_some((distance, target))
        })
        .min_by(|(left, _), (right, _)| left.total_cmp(right))
        .map(|(_, target)| target)
}

fn resolve_split_corner_resize_target(
    cursor: ui_math::UiPoint,
    layouts: &ComputedLayoutMap,
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
) -> Option<SplitCornerResizeTarget> {
    let targets = split_resize_targets(projection_artifacts);
    let mut best: Option<(f32, SplitCornerResizeTarget)> = None;
    for horizontal in targets
        .iter()
        .copied()
        .filter(|target| target.axis == WorkspaceSplitAxis::Horizontal)
    {
        let Some(horizontal_layout) = layouts.get(&horizontal.widget_id) else {
            continue;
        };
        if !horizontal_layout.bounds.contains(cursor) {
            continue;
        }
        let horizontal_boundary = split_boundary_position(
            horizontal.axis,
            horizontal_layout.bounds,
            horizontal.fraction,
        );
        let horizontal_distance = (cursor.x - horizontal_boundary).abs();
        if horizontal_distance > SPLIT_HIT_SLOP_PX {
            continue;
        }

        for vertical in targets
            .iter()
            .copied()
            .filter(|target| target.axis == WorkspaceSplitAxis::Vertical)
        {
            let Some(vertical_layout) = layouts.get(&vertical.widget_id) else {
                continue;
            };
            if !vertical_layout.bounds.contains(cursor) {
                continue;
            }
            let vertical_boundary =
                split_boundary_position(vertical.axis, vertical_layout.bounds, vertical.fraction);
            let vertical_distance = (cursor.y - vertical_boundary).abs();
            if vertical_distance > SPLIT_HIT_SLOP_PX {
                continue;
            }
            let width = (horizontal_boundary - horizontal_layout.bounds.x).max(1.0);
            let height = (vertical_boundary - vertical_layout.bounds.y).max(1.0);
            let corner = SplitCornerResizeTarget {
                horizontal,
                vertical,
                cursor_intent: if cursor.x <= horizontal_boundary && cursor.y <= vertical_boundary {
                    ShellCursorIntent::ResizeNwse
                } else {
                    ShellCursorIntent::ResizeNesw
                },
                aspect_ratio: width / height,
            };
            let distance = horizontal_distance.max(vertical_distance);
            if best
                .as_ref()
                .is_none_or(|(best_distance, _)| distance < *best_distance)
            {
                best = Some((distance, corner));
            }
        }
    }
    best.map(|(_, target)| target)
}

fn tab_stack_corner_at_pointer<'a>(
    cursor: ui_math::UiPoint,
    layouts: &ComputedLayoutMap,
    projection_artifacts: &'a editor_shell::ShellProjectionArtifacts,
) -> Option<&'a ProjectedTabStackSlot> {
    projected_tab_stacks_for_controller(&projection_artifacts.workspace.root_host)
        .into_iter()
        .find(|stack| {
            let Some(layout) = layouts.get(&tab_stack_container_widget_id(stack.tab_stack_id))
            else {
                return false;
            };
            let bounds = layout.bounds;
            if !bounds.contains(cursor) {
                return false;
            }
            let near_left = (cursor.x - bounds.x).abs() <= SPLIT_HIT_SLOP_PX;
            let near_right = (cursor.x - (bounds.x + bounds.width)).abs() <= SPLIT_HIT_SLOP_PX;
            let near_top = (cursor.y - bounds.y).abs() <= SPLIT_HIT_SLOP_PX;
            let near_bottom = (cursor.y - (bounds.y + bounds.height)).abs() <= SPLIT_HIT_SLOP_PX;
            (near_left || near_right) && (near_top || near_bottom)
        })
}

fn projected_tab_stacks_for_controller(
    host: &ProjectedWorkspaceHostSlot,
) -> Vec<&ProjectedTabStackSlot> {
    let mut stacks = Vec::new();
    collect_projected_tab_stacks_for_controller(host, &mut stacks);
    stacks
}

fn collect_projected_tab_stacks_for_controller<'a>(
    host: &'a ProjectedWorkspaceHostSlot,
    stacks: &mut Vec<&'a ProjectedTabStackSlot>,
) {
    match host {
        ProjectedWorkspaceHostSlot::Split {
            first_child,
            second_child,
            ..
        } => {
            collect_projected_tab_stacks_for_controller(first_child, stacks);
            collect_projected_tab_stacks_for_controller(second_child, stacks);
        }
        ProjectedWorkspaceHostSlot::TabStack { tab_stack, .. } => stacks.push(tab_stack),
        ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder { .. } => {}
    }
}

fn active_tool_surface_kind_for_tab_stack(
    shell_state: &RunenwerkEditorShellState,
    tab_stack_id: editor_shell::TabStackId,
) -> ToolSurfaceKind {
    let Some(tab_stack) = shell_state.workspace_state().tab_stack(tab_stack_id) else {
        return ToolSurfaceKind::Viewport;
    };
    let Some(panel_id) = tab_stack
        .active_panel
        .or_else(|| tab_stack.ordered_panels.first().copied())
    else {
        return ToolSurfaceKind::Viewport;
    };
    shell_state
        .workspace_state()
        .panel(panel_id)
        .and_then(|panel| panel.active_tool_surface)
        .and_then(|surface_id| shell_state.workspace_state().tool_surface(surface_id))
        .map(|surface| surface.tool_surface_kind)
        .unwrap_or(ToolSurfaceKind::Viewport)
}

fn split_boundary_position(
    axis: WorkspaceSplitAxis,
    bounds: ui_math::UiRect,
    fraction: f32,
) -> f32 {
    match workspace_axis_to_ui_axis(axis) {
        ui_math::Axis::Horizontal => bounds.x + bounds.width * fraction,
        ui_math::Axis::Vertical => bounds.y + bounds.height * fraction,
    }
}

fn split_fraction_from_pointer(
    axis: WorkspaceSplitAxis,
    bounds: ui_math::UiRect,
    pointer: &PointerEvent,
) -> f32 {
    match workspace_axis_to_ui_axis(axis) {
        ui_math::Axis::Horizontal => (pointer.position.x - bounds.x) / bounds.width.max(1.0),
        ui_math::Axis::Vertical => (pointer.position.y - bounds.y) / bounds.height.max(1.0),
    }
}

fn split_resize_target_by_host(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    split_host_id: PanelHostId,
) -> Option<SplitResizeTarget> {
    split_resize_targets(projection_artifacts)
        .into_iter()
        .find(|target| target.split_host_id == split_host_id)
}

fn split_resize_targets(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
) -> Vec<SplitResizeTarget> {
    if projection_artifacts.workspace.fixed_layout.is_some()
        && let Some(targets) =
            fixed_layout_split_resize_targets(&projection_artifacts.workspace.root_host)
    {
        return targets;
    }
    let mut targets = Vec::new();
    collect_split_resize_targets(&projection_artifacts.workspace.root_host, &mut targets);
    targets
}

fn fixed_layout_split_resize_targets(
    root_host: &ProjectedWorkspaceHostSlot,
) -> Option<Vec<SplitResizeTarget>> {
    let ProjectedWorkspaceHostSlot::Split {
        host_id: body_console_host_id,
        axis: body_console_axis,
        fraction: body_console_fraction,
        first_child: left_right_host,
        ..
    } = root_host
    else {
        return None;
    };
    let ProjectedWorkspaceHostSlot::Split {
        host_id: left_right_host_id,
        axis: left_right_axis,
        fraction: left_right_fraction,
        second_child: center_right_host,
        ..
    } = left_right_host.as_ref()
    else {
        return None;
    };
    let ProjectedWorkspaceHostSlot::Split {
        host_id: center_right_host_id,
        axis: center_right_axis,
        fraction: center_right_fraction,
        ..
    } = center_right_host.as_ref()
    else {
        return None;
    };
    Some(vec![
        SplitResizeTarget {
            split_host_id: *body_console_host_id,
            widget_id: BODY_CONSOLE_SPLIT_WIDGET_ID,
            axis: *body_console_axis,
            fraction: *body_console_fraction,
        },
        SplitResizeTarget {
            split_host_id: *left_right_host_id,
            widget_id: LEFT_RIGHT_SPLIT_WIDGET_ID,
            axis: *left_right_axis,
            fraction: *left_right_fraction,
        },
        SplitResizeTarget {
            split_host_id: *center_right_host_id,
            widget_id: CENTER_RIGHT_SPLIT_WIDGET_ID,
            axis: *center_right_axis,
            fraction: *center_right_fraction,
        },
    ])
}

fn collect_split_resize_targets(
    host: &ProjectedWorkspaceHostSlot,
    targets: &mut Vec<SplitResizeTarget>,
) {
    match host {
        ProjectedWorkspaceHostSlot::Split {
            host_id,
            widget_id,
            axis,
            fraction,
            first_child,
            second_child,
            ..
        } => {
            targets.push(SplitResizeTarget {
                split_host_id: *host_id,
                widget_id: *widget_id,
                axis: *axis,
                fraction: *fraction,
            });
            collect_split_resize_targets(first_child, targets);
            collect_split_resize_targets(second_child, targets);
        }
        ProjectedWorkspaceHostSlot::TabStack { .. }
        | ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder { .. } => {}
    }
}

fn workspace_axis_to_ui_axis(axis: WorkspaceSplitAxis) -> ui_math::Axis {
    match axis {
        WorkspaceSplitAxis::Horizontal => ui_math::Axis::Horizontal,
        WorkspaceSplitAxis::Vertical => ui_math::Axis::Vertical,
    }
}

fn split_cursor_intent(axis: WorkspaceSplitAxis) -> ShellCursorIntent {
    match workspace_axis_to_ui_axis(axis) {
        ui_math::Axis::Horizontal => ShellCursorIntent::ResizeColumn,
        ui_math::Axis::Vertical => ShellCursorIntent::ResizeRow,
    }
}

fn consumed_pointer_outcome(
    target: Option<editor_shell::WidgetId>,
    changed_layout: bool,
) -> UiInputOutcome {
    UiInputOutcome {
        dispatch: editor_shell::UiInputDispatchResult {
            target,
            response: InputResponse {
                propagation: EventPropagation::Stop,
                capture: PointerCapture::None,
                focus_change: FocusChange::None,
                repaint: changed_layout,
                relayout: changed_layout,
            },
        },
        interactions: UiInteractionResults::new(),
        invalidation: editor_shell::UiInvalidation {
            repaint: changed_layout,
            relayout: changed_layout,
        },
    }
}

fn is_console_scroll_event(event: &UiInputEvent, outcome: &UiInputOutcome) -> bool {
    matches!(
        event,
        UiInputEvent::Pointer(pointer) if pointer.kind == PointerEventKind::Scroll
    ) && outcome.dispatch.target == Some(CONSOLE_SCROLL_WIDGET_ID)
}

fn console_follow_enabled_for_active_surface(
    app: &RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> bool {
    active_console_surface(shell_state)
        .map(|surface_id| {
            app.surface_sessions()
                .session_or_default(surface_id)
                .console_follow_enabled
        })
        .unwrap_or(true)
}

fn active_console_surface(
    shell_state: &RunenwerkEditorShellState,
) -> Option<editor_shell::ToolSurfaceInstanceId> {
    shell_state
        .workspace_state()
        .panels()
        .filter_map(|panel| {
            let surface_id = panel.active_tool_surface?;
            let surface = shell_state.workspace_state().tool_surface(surface_id)?;
            (surface.tool_surface_kind == editor_shell::ToolSurfaceKind::Console)
                .then_some(surface_id)
        })
        .next()
}

fn resolve_tab_drop_preview_target(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<DockingPreviewDropTarget> {
    editor_shell::hit_test_widget(tree, layouts, pointer_position)
        .and_then(|widget_id| {
            projection_artifacts
                .workspace
                .tab_drop_route_by_widget_id
                .get(&widget_id)
                .copied()
        })
        .or_else(|| tab_drop_route_at_pointer(projection_artifacts, layouts, pointer_position))
        .map(map_projected_tab_drop_target)
}

fn tab_drop_route_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabDropRoute> {
    let containing = projection_artifacts
        .workspace
        .tab_drop_route_by_widget_id
        .iter()
        .filter_map(|(widget_id, route)| {
            let layout = layouts.get(widget_id)?;
            layout.bounds.contains(pointer_position).then(|| {
                let center_x = layout.bounds.x + layout.bounds.width * 0.5;
                let center_y = layout.bounds.y + layout.bounds.height * 0.5;
                let distance =
                    (center_x - pointer_position.x).abs() + (center_y - pointer_position.y).abs();
                (distance, *route)
            })
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, route)| route);
    if containing.is_some() {
        return containing;
    }

    let edge_route =
        tab_stack_edge_drop_route_at_pointer(projection_artifacts, layouts, pointer_position);
    if edge_route.is_some() {
        return edge_route;
    }

    projection_artifacts
        .workspace
        .tab_drop_route_by_widget_id
        .iter()
        .filter_map(|(widget_id, route)| {
            let layout = layouts.get(widget_id)?;
            let y_min = layout.bounds.y - 6.0;
            let y_max = layout.bounds.y + layout.bounds.height + 6.0;
            if pointer_position.y < y_min || pointer_position.y > y_max {
                return None;
            }
            let center_x = layout.bounds.x + layout.bounds.width * 0.5;
            let distance = (center_x - pointer_position.x).abs();
            (distance <= 64.0).then_some((distance, *route))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, route)| route)
}

fn tab_stack_edge_drop_route_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabDropRoute> {
    projected_tab_stacks_for_docking(&projection_artifacts.workspace)
        .into_iter()
        .filter_map(|stack| {
            let container_id = editor_shell::tab_stack_container_widget_id(stack.tab_stack_id);
            let container = layouts.get(&container_id)?;
            container.bounds.contains(pointer_position).then(|| {
                let insert_index = tab_insert_index_for_pointer(stack, layouts, pointer_position);
                (
                    edge_drop_priority(container.bounds, pointer_position),
                    editor_shell::ProjectedTabDropRoute {
                        target: editor_shell::ProjectedTabDropTarget::TabStack {
                            tab_stack_id: stack.tab_stack_id,
                            insert_index,
                        },
                    },
                )
            })
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, route)| route)
}

fn projected_tab_stacks_for_docking(
    projection: &editor_shell::WorkspaceProjectionArtifact,
) -> Vec<&editor_shell::ProjectedTabStackSlot> {
    let mut stacks = editor_shell::projected_host_tab_stacks(&projection.root_host);
    stacks.extend(projection.floating_hosts.iter().map(|host| &host.tab_stack));
    stacks
}

fn tab_insert_index_for_pointer(
    stack: &editor_shell::ProjectedTabStackSlot,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> usize {
    stack
        .tabs
        .iter()
        .enumerate()
        .find_map(|(index, tab)| {
            let layout = layouts.get(&tab.widget_id)?;
            let midpoint = layout.bounds.x + layout.bounds.width * 0.5;
            (pointer_position.x < midpoint).then_some(index)
        })
        .unwrap_or(stack.tabs.len())
}

fn edge_drop_priority(bounds: ui_math::UiRect, pointer_position: ui_math::UiPoint) -> f32 {
    let left = (pointer_position.x - bounds.x).abs();
    let right = (pointer_position.x - (bounds.x + bounds.width)).abs();
    let top = (pointer_position.y - bounds.y).abs();
    let bottom = (pointer_position.y - (bounds.y + bounds.height)).abs();
    left.min(right).min(top).min(bottom)
}

fn tab_button_route_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabButtonRoute> {
    tab_button_hit_at_pointer(projection_artifacts, layouts, pointer_position)
        .map(|(_, route)| route)
}

fn tab_button_hit_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<(WidgetId, editor_shell::ProjectedTabButtonRoute)> {
    let containing = projection_artifacts
        .workspace
        .tab_button_route_by_widget_id
        .iter()
        .filter_map(|(widget_id, route)| {
            let layout = layouts.get(widget_id)?;
            layout.bounds.contains(pointer_position).then(|| {
                let center_x = layout.bounds.x + layout.bounds.width * 0.5;
                let center_y = layout.bounds.y + layout.bounds.height * 0.5;
                let distance =
                    (center_x - pointer_position.x).abs() + (center_y - pointer_position.y).abs();
                (distance, *widget_id, *route)
            })
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, widget_id, route)| (widget_id, route));
    if containing.is_some() {
        return containing;
    }

    projection_artifacts
        .workspace
        .tab_button_route_by_widget_id
        .iter()
        .filter_map(|(widget_id, route)| {
            let layout = layouts.get(widget_id)?;
            let y_min = layout.bounds.y - 4.0;
            let y_max = layout.bounds.y + layout.bounds.height + 4.0;
            if pointer_position.y < y_min || pointer_position.y > y_max {
                return None;
            }
            let center_x = layout.bounds.x + layout.bounds.width * 0.5;
            let distance = (center_x - pointer_position.x).abs();
            (distance <= 48.0).then_some((distance, *widget_id, *route))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, widget_id, route)| (widget_id, route))
}

fn map_projected_tab_drop_target(
    route: editor_shell::ProjectedTabDropRoute,
) -> DockingPreviewDropTarget {
    match route.target {
        editor_shell::ProjectedTabDropTarget::TabStack {
            tab_stack_id,
            insert_index,
        } => DockingPreviewDropTarget::TabStack {
            tab_stack_id,
            insert_index,
        },
        editor_shell::ProjectedTabDropTarget::NewFloatingHost => {
            DockingPreviewDropTarget::NewFloatingHost
        }
    }
}

fn is_at_bottom(offset: f32, max_offset: f32) -> bool {
    max_offset <= CONSOLE_FOLLOW_BOTTOM_EPSILON
        || offset >= (max_offset - CONSOLE_FOLLOW_BOTTOM_EPSILON)
}
