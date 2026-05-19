use editor_shell::{
    BODY_ROOT_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID, ComputedLayoutMap, DockDropCandidate,
    DockDropCandidateState, DockDropInvalidTargetReason, DockDropScope, DockSplitSide,
    DockingPreviewDropTarget, EditorShellFrameModel, PanelHostId, ProjectedTabStackSlot,
    PanelKind, ProjectedWorkspaceHostSlot, ShellCommand, ShellUiExpressionFrame,
    SurfaceCommandProposal,
    TOOLBAR_ADD_WORKSPACE_WIDGET_ID, TOOLBAR_EDIT_MENU_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID,
    TOOLBAR_MENU_POPUP_WIDGET_ID, TOOLBAR_WINDOW_MENU_WIDGET_ID, TabDropDestination,
    ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceStableKey, UiInputOutcome, UiInteractionResults,
    UiTree, VIEWPORT_OPTIONS_BUTTON_WIDGET_ID, VIEWPORT_OPTIONS_POPUP_WIDGET_ID,
    VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID, VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID,
    VIEWPORT_TOOLS_MENU_WIDGET_ID, WidgetId, WorkspaceSplitAxis,
    build_editor_shell_frame_with_docking_visual_state, map_interactions_to_shell_commands,
    surface_widget_id, tab_stack_container_widget_id, tab_stack_popup_menu_widget_id,
    viewport_tool_radial_item_widget_id,
};
use editor_viewport::ArtifactObservationFrame;
use ui_input::{
    EventPropagation, FocusChange, InputResponse, Key, KeyState, PointerCapture, PointerEvent,
    PointerEventKind, UiInputEvent,
};
use ui_math::{Axis, UiRect};
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPresentationStateResource,
    ViewportRenderStateCommandQueueResource, is_viewport_tool_surface,
};
use crate::shell::tool_suites::EDITOR_CONSOLE_SURFACE_KEY;
use crate::shell::{
    CornerAreaSplitSession, CornerSplitResizeSession, EditorShellFrameMetrics,
    EditorSurfaceProviderRegistry, RunenwerkEditorShellState, SurfaceProviderDispatchContext,
    active_document_context, build_editor_shell_frame_model_with_frame_metrics,
    dispatch_shell_command, dispatch_shell_command_with_viewport_commands,
    mounted_surface_requests,
};
use editor_shell::TabStackPopupMenuKind;

const CONSOLE_FOLLOW_BOTTOM_EPSILON: f32 = 1.0;
const SPLIT_HIT_SLOP_PX: f32 = 12.0;
const SPLIT_MIN_FRACTION: f32 = 0.08;
const SPLIT_MAX_FRACTION: f32 = 0.92;
const CORNER_AREA_SPLIT_THRESHOLD_PX: f32 = 18.0;
const SIDE_DOCK_DROP_TARGET_EDGE_PX: f32 = 48.0;

#[derive(Debug, Clone, PartialEq)]
struct ResolvedTabDropPreview {
    target: Option<DockingPreviewDropTarget>,
    candidates: Vec<DockDropCandidate>,
    active_side: Option<DockSplitSide>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TabDropPreviewRequest {
    pointer_position: ui_math::UiPoint,
    source_tab_stack_id: Option<editor_shell::TabStackId>,
    explicit_float: bool,
    cycle_index: usize,
    cycle_side: Option<DockSplitSide>,
}

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

    #[allow(clippy::too_many_arguments)]
    pub fn build_expression_frame_with_surface_resources(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
        viewport_instances: Option<&ViewportInstanceRegistryResource>,
        frame_metrics: Option<EditorShellFrameMetrics>,
    ) -> ShellUiExpressionFrame {
        let frame_model = Self::rebuild_frame_model_with_provider_context(
            app,
            shell_state,
            theme,
            app.surface_provider_registry(),
            frame_metrics,
            viewport_observations,
            tool_surface_bindings,
            viewport_instances,
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
            && let Some(console_scroll_widget_id) = active_console_scroll_widget(shell_state)
            && let Some(max_offset) = shell_state.runtime().max_scroll_offset_for_axis(
                &tree,
                bounds,
                console_scroll_widget_id,
                Axis::Vertical,
            )
        {
            shell_state.runtime_mut().set_scroll_offset_for_axis(
                console_scroll_widget_id,
                Axis::Vertical,
                max_offset,
            );
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
        frame_metrics: Option<EditorShellFrameMetrics>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
        viewport_instances: Option<&ViewportInstanceRegistryResource>,
    ) -> EditorShellFrameModel {
        build_editor_shell_frame_model_with_frame_metrics(
            app,
            shell_state,
            registry,
            theme,
            frame_metrics,
            viewport_observations,
            tool_surface_bindings,
            viewport_instances,
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
        viewport_instances: Option<&ViewportInstanceRegistryResource>,
        viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
    ) -> Result<UiInputOutcome, editor_core::EditorMutationError> {
        let registry = app.surface_provider_registry_handle();
        let frame_model = Self::rebuild_frame_model_with_provider_context(
            app,
            shell_state,
            theme,
            registry.as_ref(),
            None,
            viewport_observations,
            tool_surface_bindings,
            viewport_instances,
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
        let console_scroll_widget_id = active_console_scroll_widget(shell_state);
        let pre_at_bottom = console_scroll_widget_id
            .map(|widget_id| {
                let pre_offset = shell_state
                    .runtime()
                    .scroll_offset_for_axis(widget_id, Axis::Vertical);
                let pre_max = shell_state
                    .runtime()
                    .max_scroll_offset_for_layout_axis(&tree, &layouts, widget_id, Axis::Vertical)
                    .unwrap_or(0.0);
                is_at_bottom(pre_offset, pre_max)
            })
            .unwrap_or(true);

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
        if let Some(outcome) =
            Self::handle_viewport_options_menu_dismiss_event(app, shell_state, event, &layouts)
        {
            return Ok(outcome);
        }
        if let Some(outcome) =
            Self::handle_viewport_tool_radial_menu_dismiss_event(app, shell_state, event, &layouts)
        {
            return Ok(outcome);
        }
        if let Some(outcome) = Self::handle_tab_drag_scope_cycle_event(
            shell_state,
            event,
            &tree,
            &layouts,
            &projection_artifacts,
        ) {
            return Ok(outcome);
        }

        let outcome = shell_state
            .runtime_mut()
            .dispatch_input(&tree, &layouts, event);

        if is_console_scroll_event(event, &outcome, console_scroll_widget_id) {
            let post_layouts = shell_state.runtime().compute_layout(&tree, bounds);
            if let (Some(surface_id), Some(console_scroll_widget_id)) = (
                active_console_surface(shell_state),
                console_scroll_widget_id,
            ) {
                let post_offset = shell_state
                    .runtime()
                    .scroll_offset_for_axis(console_scroll_widget_id, Axis::Vertical);
                let post_max = shell_state
                    .runtime()
                    .max_scroll_offset_for_layout_axis(
                        &tree,
                        &post_layouts,
                        console_scroll_widget_id,
                        Axis::Vertical,
                    )
                    .unwrap_or(0.0);
                let post_at_bottom = is_at_bottom(post_offset, post_max);
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

        Self::dispatch_commands_with_viewport_commands(
            app,
            shell_state,
            commands,
            registry.as_ref(),
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
            viewport_render_commands,
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
        let active_popup =
            tab_stack_popup_menu_widget_id(active_menu.kind, active_menu.tab_stack_id);
        let inside_active_popup = layouts
            .get(&active_popup)
            .is_some_and(|layout| layout.bounds.contains(pointer.position));
        if inside_active_popup {
            return None;
        }
        shell_state.close_tab_stack_popup_menu();
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

    fn handle_viewport_options_menu_dismiss_event(
        app: &mut RunenwerkEditorApp,
        shell_state: &RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        if !matches!(pointer.kind, PointerEventKind::Down)
            || !matches!(
                pointer.button,
                Some(ui_input::PointerButton::Primary | ui_input::PointerButton::Secondary)
            )
        {
            return None;
        }

        let open_viewport_surfaces = shell_state
            .workspace_state()
            .tool_surfaces()
            .filter(|surface| is_viewport_tool_surface(surface))
            .filter(|surface| matches!(surface.mount, ToolSurfaceMount::Mounted { .. }))
            .filter(|surface| {
                app.surface_sessions()
                    .session(surface.id)
                    .is_some_and(|session| session.viewport_options_menu_open)
            })
            .map(|surface| surface.id)
            .collect::<Vec<_>>();
        if open_viewport_surfaces.is_empty() {
            return None;
        }

        let inside_open_menu = open_viewport_surfaces.iter().any(|surface_id| {
            [
                surface_widget_id(*surface_id, VIEWPORT_OPTIONS_BUTTON_WIDGET_ID),
                surface_widget_id(*surface_id, VIEWPORT_OPTIONS_POPUP_WIDGET_ID),
            ]
            .iter()
            .any(|widget_id| {
                layouts
                    .get(widget_id)
                    .is_some_and(|layout| layout.bounds.contains(pointer.position))
            })
        });
        if inside_open_menu {
            return None;
        }

        let changed = app
            .surface_sessions_mut()
            .close_all_viewport_options_menus();
        Some(consumed_pointer_outcome(None, changed))
    }

    fn handle_viewport_tool_radial_menu_dismiss_event(
        app: &mut RunenwerkEditorApp,
        shell_state: &RunenwerkEditorShellState,
        event: &UiInputEvent,
        layouts: &ComputedLayoutMap,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        if !matches!(pointer.kind, PointerEventKind::Down)
            || !matches!(
                pointer.button,
                Some(ui_input::PointerButton::Primary | ui_input::PointerButton::Secondary)
            )
        {
            return None;
        }

        let open_viewport_surfaces = shell_state
            .workspace_state()
            .tool_surfaces()
            .filter(|surface| is_viewport_tool_surface(surface))
            .filter(|surface| matches!(surface.mount, ToolSurfaceMount::Mounted { .. }))
            .filter(|surface| {
                app.surface_sessions()
                    .session(surface.id)
                    .is_some_and(|session| {
                        session.viewport_tools_menu_open
                            || session.viewport_tool_radial_session.is_some()
                    })
            })
            .map(|surface| surface.id)
            .collect::<Vec<_>>();
        if open_viewport_surfaces.is_empty() {
            return None;
        }

        let inside_open_menu = open_viewport_surfaces.iter().any(|surface_id| {
            [
                surface_widget_id(*surface_id, VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID),
                surface_widget_id(*surface_id, VIEWPORT_TOOLS_MENU_WIDGET_ID),
                surface_widget_id(*surface_id, VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID),
                surface_widget_id(*surface_id, viewport_tool_radial_item_widget_id(0)),
                surface_widget_id(*surface_id, viewport_tool_radial_item_widget_id(1)),
                surface_widget_id(*surface_id, viewport_tool_radial_item_widget_id(2)),
                surface_widget_id(*surface_id, viewport_tool_radial_item_widget_id(3)),
            ]
            .iter()
            .any(|widget_id| {
                layouts
                    .get(widget_id)
                    .is_some_and(|layout| layout.bounds.contains(pointer.position))
            })
        });
        if inside_open_menu {
            return None;
        }

        let changed = app.surface_sessions_mut().close_all_viewport_tools_menus()
            | app
                .surface_sessions_mut()
                .close_all_viewport_tool_radial_menus();
        Some(consumed_pointer_outcome(None, changed))
    }

    fn handle_tab_drag_scope_cycle_event(
        shell_state: &mut RunenwerkEditorShellState,
        event: &UiInputEvent,
        _tree: &UiTree,
        _layouts: &ComputedLayoutMap,
        _projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    ) -> Option<UiInputOutcome> {
        let UiInputEvent::Keyboard(keyboard) = event else {
            return None;
        };
        if keyboard.key != Key::Tab
            || !matches!(keyboard.state, KeyState::Pressed | KeyState::Repeated)
            || shell_state.docking_visual_state().active_tab_drag.is_none()
        {
            return None;
        }
        if !shell_state.cycle_active_tab_drag_preview_candidate() {
            return None;
        }
        Some(consumed_keyboard_outcome(true))
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
                    let Some(command) = split_tab_stack_area_command_for_active_surface_pending_c6(
                        shell_state,
                        session.tab_stack_id,
                        axis,
                        session.projection_epoch,
                    ) else {
                        shell_state.clear_corner_area_split();
                        return Ok(Some(consumed_pointer_outcome(None, false)));
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
                    let (cycle_index, cycle_side) = shell_state.tab_drag_drop_candidate_cycle();
                    let source_tab_stack_id = shell_state
                        .docking_visual_state()
                        .active_tab_drag
                        .map(|drag| drag.source_tab_stack_id);
                    let preview = resolve_tab_drop_preview_target(
                        projection_artifacts,
                        tree,
                        layouts,
                        TabDropPreviewRequest {
                            pointer_position: pointer.position,
                            source_tab_stack_id,
                            explicit_float: pointer.modifiers.alt,
                            cycle_index,
                            cycle_side,
                        },
                    );
                    shell_state.set_tab_drag_preview(
                        preview.target,
                        preview.candidates,
                        preview.active_side,
                        projection_artifacts.projection_epoch,
                    );
                }
            }
            PointerEventKind::Leave => {
                shell_state.set_tab_drag_preview(
                    None,
                    Vec::new(),
                    None,
                    projection_artifacts.projection_epoch,
                );
            }
            PointerEventKind::Up => {
                let click_candidate = shell_state.tab_drag_candidate();
                let (cycle_index, cycle_side) = shell_state.tab_drag_drop_candidate_cycle();
                let preview = resolve_tab_drop_preview_target(
                    projection_artifacts,
                    tree,
                    layouts,
                    TabDropPreviewRequest {
                        pointer_position: pointer.position,
                        source_tab_stack_id: shell_state
                            .docking_visual_state()
                            .active_tab_drag
                            .map(|drag| drag.source_tab_stack_id),
                        explicit_float: pointer.modifiers.alt,
                        cycle_index,
                        cycle_side,
                    },
                );
                shell_state.set_tab_drag_preview(
                    preview.target,
                    preview.candidates,
                    preview.active_side,
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
                        DockingPreviewDropTarget::SplitIntoArea {
                            target_tab_stack_id,
                            side,
                        } => TabDropDestination::SplitIntoArea {
                            target_tab_stack_id,
                            side,
                        },
                        DockingPreviewDropTarget::SplitIntoHost {
                            target_host_id,
                            side,
                        } => TabDropDestination::SplitIntoHost {
                            target_host_id,
                            side,
                        },
                        DockingPreviewDropTarget::SplitIntoRoot { side } => {
                            TabDropDestination::SplitIntoRoot { side }
                        }
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

    #[allow(dead_code)]
    pub(crate) fn dispatch_commands(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        commands: Vec<ShellCommand>,
        registry: &EditorSurfaceProviderRegistry,
        viewport_presentations: Option<&mut ViewportPresentationStateResource>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    ) -> Result<(), editor_core::EditorMutationError> {
        Self::dispatch_commands_with_viewport_commands(
            app,
            shell_state,
            commands,
            registry,
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dispatch_commands_with_viewport_commands(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        commands: Vec<ShellCommand>,
        registry: &EditorSurfaceProviderRegistry,
        mut viewport_presentations: Option<&mut ViewportPresentationStateResource>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
        mut viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
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
                    dispatch_shell_command_with_viewport_commands(
                        app,
                        Some(shell_state),
                        command,
                        viewport_presentations.as_deref_mut(),
                        viewport_observations,
                        tool_surface_bindings,
                        viewport_render_commands.as_deref_mut(),
                        Some(current_epoch),
                    )?;
                }
                continue;
            }

            if let ShellCommand::DispatchSurfaceInteraction {
                provider_id,
                tool_surface_instance_id,
                target,
                interaction,
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
                if let Some(proposal) = registry.map_interaction(
                    &dispatch_context,
                    &request,
                    provider_id,
                    interaction,
                )? && let Some(command) = shell_command_from_surface_proposal(proposal)
                {
                    dispatch_shell_command_with_viewport_commands(
                        app,
                        Some(shell_state),
                        command,
                        viewport_presentations.as_deref_mut(),
                        viewport_observations,
                        tool_surface_bindings,
                        viewport_render_commands.as_deref_mut(),
                        Some(current_epoch),
                    )?;
                }
                continue;
            }

            dispatch_shell_command_with_viewport_commands(
                app,
                Some(shell_state),
                command,
                viewport_presentations.as_deref_mut(),
                viewport_observations,
                tool_surface_bindings,
                viewport_render_commands.as_deref_mut(),
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
        SurfaceCommandProposal::SurfaceSession(proposal) => {
            Some(ShellCommand::ApplySurfaceSessionMutation {
                target: proposal.target,
                mutation: proposal.mutation,
                projection_epoch: proposal.projection_epoch,
            })
        }
        SurfaceCommandProposal::EditorDomain(proposal) => {
            Some(ShellCommand::ApplyEditorDomainMutation {
                target: proposal.target,
                mutation: proposal.mutation,
                projection_epoch: proposal.projection_epoch,
            })
        }
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
        let split_distance = split_resize_distance(pointer.position, layouts, candidate);
        if tab_button_route_at_pointer(projection_artifacts, layouts, pointer.position).is_some() {
            if split_distance > 3.0 {
                return None;
            }
        } else if routed_action_widget_at_pointer(projection_artifacts, layouts, pointer.position)
            .is_some()
        {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TabStackChromeSurfaceTarget {
    panel_kind: PanelKind,
    stable_surface_key: ToolSurfaceStableKey,
    legacy_tool_surface_kind: Option<ToolSurfaceKind>,
}

fn tab_stack_chrome_surface_target_pending_c6(
    shell_state: &RunenwerkEditorShellState,
    tab_stack_id: editor_shell::TabStackId,
) -> Option<TabStackChromeSurfaceTarget> {
    // C6D repair: corner split is still app-shell chrome, but it must preserve
    // stable-key identity instead of inventing viewport legacy fallback.
    let Some(tab_stack) = shell_state.workspace_state().tab_stack(tab_stack_id) else {
        return None;
    };
    let Some(panel_id) = tab_stack
        .active_panel
        .or_else(|| tab_stack.ordered_panels.first().copied())
    else {
        return None;
    };
    let panel = shell_state.workspace_state().panel(panel_id)?;
    let surface = panel
        .active_tool_surface
        .and_then(|surface_id| shell_state.workspace_state().tool_surface(surface_id))?;
    Some(TabStackChromeSurfaceTarget {
        panel_kind: panel.panel_kind,
        stable_surface_key: surface.stable_surface_key().clone(),
        legacy_tool_surface_kind: surface.legacy_tool_surface_kind(),
    })
}

fn split_tab_stack_area_command_for_active_surface_pending_c6(
    shell_state: &RunenwerkEditorShellState,
    tab_stack_id: editor_shell::TabStackId,
    axis: WorkspaceSplitAxis,
    projection_epoch: u64,
) -> Option<ShellCommand> {
    let target = tab_stack_chrome_surface_target_pending_c6(shell_state, tab_stack_id)?;
    Some(ShellCommand::SplitTabStackAreaStableKey {
        tab_stack_id,
        axis,
        panel_kind: target.panel_kind,
        stable_surface_key: target.stable_surface_key,
        legacy_tool_surface_kind: target.legacy_tool_surface_kind,
        projection_epoch,
    })
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

fn split_resize_distance(
    cursor: ui_math::UiPoint,
    layouts: &ComputedLayoutMap,
    target: SplitResizeTarget,
) -> f32 {
    let Some(layout) = layouts.get(&target.widget_id) else {
        return f32::MAX;
    };
    let boundary = split_boundary_position(target.axis, layout.bounds, target.fraction);
    match workspace_axis_to_ui_axis(target.axis) {
        ui_math::Axis::Horizontal => (cursor.x - boundary).abs(),
        ui_math::Axis::Vertical => (cursor.y - boundary).abs(),
    }
}

fn routed_action_widget_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    cursor: ui_math::UiPoint,
) -> Option<WidgetId> {
    projection_artifacts
        .widget_actions_by_id
        .keys()
        .copied()
        .filter(|widget_id| {
            layouts
                .get(widget_id)
                .is_some_and(|layout| layout.bounds.contains(cursor))
        })
        .min_by(|left, right| {
            let left_bounds = layouts
                .get(left)
                .expect("filtered routed widget should have layout")
                .bounds;
            let right_bounds = layouts
                .get(right)
                .expect("filtered routed widget should have layout")
                .bounds;
            let left_area = left_bounds.width * left_bounds.height;
            let right_area = right_bounds.width * right_bounds.height;
            left_area.total_cmp(&right_area)
        })
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
    let mut targets = Vec::new();
    collect_split_resize_targets(&projection_artifacts.workspace.root_host, &mut targets);
    targets
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_dock_drop_candidate_does_not_resolve_commit_target() {
        let source_tab_stack_id = editor_shell::TabStackId::try_from_raw(7).unwrap();
        let invalid_target = DockingPreviewDropTarget::SplitIntoArea {
            target_tab_stack_id: source_tab_stack_id,
            side: DockSplitSide::Left,
        };

        let resolved = activate_dock_drop_candidate(
            vec![DockDropCandidate {
                target: invalid_target,
                scope: DockDropScope::Area,
                side: DockSplitSide::Left,
                anchor_widget_id: WidgetId(99),
                state: DockDropCandidateState::Invalid {
                    reason: DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea,
                },
            }],
            0,
            None,
        );

        assert_eq!(resolved.target, None);
        assert_eq!(resolved.active_side, None);
        assert_eq!(resolved.candidates[0].target, invalid_target);
        assert!(matches!(
            resolved.candidates[0].state,
            DockDropCandidateState::Invalid {
                reason: DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea
            }
        ));
    }
}

fn consumed_keyboard_outcome(changed_layout: bool) -> UiInputOutcome {
    UiInputOutcome {
        dispatch: editor_shell::UiInputDispatchResult {
            target: None,
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

fn is_console_scroll_event(
    event: &UiInputEvent,
    outcome: &UiInputOutcome,
    console_scroll_widget_id: Option<WidgetId>,
) -> bool {
    matches!(
        event,
        UiInputEvent::Pointer(pointer) if pointer.kind == PointerEventKind::Scroll
    ) && console_scroll_widget_id
        .is_some_and(|widget_id| outcome.dispatch.target == Some(widget_id))
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
            (surface.stable_surface_key().as_str() == EDITOR_CONSOLE_SURFACE_KEY)
                .then_some(surface_id)
        })
        .next()
}

fn active_console_scroll_widget(shell_state: &RunenwerkEditorShellState) -> Option<WidgetId> {
    active_console_surface(shell_state)
        .map(|surface_id| surface_widget_id(surface_id, CONSOLE_SCROLL_WIDGET_ID))
}

fn resolve_tab_drop_preview_target(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    request: TabDropPreviewRequest,
) -> ResolvedTabDropPreview {
    if let Some(route) = tab_insertion_route_at_pointer(
        projection_artifacts,
        tree,
        layouts,
        request.pointer_position,
    ) {
        return ResolvedTabDropPreview {
            target: Some(map_projected_tab_drop_target(route)),
            candidates: Vec::new(),
            active_side: None,
        };
    }

    if request.explicit_float
        && let Some(route) = new_floating_host_route_at_pointer(layouts, request.pointer_position)
    {
        return ResolvedTabDropPreview {
            target: Some(map_projected_tab_drop_target(route)),
            candidates: Vec::new(),
            active_side: None,
        };
    }

    let candidates = collect_dock_drop_candidates(
        projection_artifacts,
        layouts,
        request.pointer_position,
        request.source_tab_stack_id,
    );
    if !candidates.is_empty() {
        return activate_dock_drop_candidate(candidates, request.cycle_index, request.cycle_side);
    }

    ResolvedTabDropPreview {
        target: None,
        candidates: Vec::new(),
        active_side: None,
    }
}

fn tab_insertion_route_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabDropRoute> {
    editor_shell::hit_test_widget(tree, layouts, pointer_position)
        .and_then(|widget_id| {
            projection_artifacts
                .workspace
                .tab_drop_route_by_widget_id
                .get(&widget_id)
                .copied()
        })
        .or_else(|| {
            containing_tab_drop_route_at_pointer(projection_artifacts, layouts, pointer_position)
        })
}

fn containing_tab_drop_route_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabDropRoute> {
    projection_artifacts
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
        .map(|(_, route)| route)
}

fn new_floating_host_route_at_pointer(
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabDropRoute> {
    let body = layouts.get(&BODY_ROOT_WIDGET_ID)?;
    let bounds = body.bounds;
    if !bounds.contains(pointer_position) {
        return None;
    }
    let edge_start = bounds.x + bounds.width - SIDE_DOCK_DROP_TARGET_EDGE_PX;
    (pointer_position.x >= edge_start).then_some(editor_shell::ProjectedTabDropRoute {
        target: editor_shell::ProjectedTabDropTarget::NewFloatingHost,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScoredDockDropCandidate {
    candidate: DockDropCandidate,
    edge_distance: f32,
    anchor_area: f32,
    pointer_inside_anchor: bool,
}

fn collect_dock_drop_candidates(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
    source_tab_stack_id: Option<editor_shell::TabStackId>,
) -> Vec<DockDropCandidate> {
    let mut candidates = Vec::new();
    candidates.extend(area_dock_drop_candidates(
        projection_artifacts,
        layouts,
        pointer_position,
    ));
    candidates.extend(group_dock_drop_candidates(
        projection_artifacts,
        layouts,
        pointer_position,
    ));
    candidates.extend(workspace_dock_drop_candidate(layouts, pointer_position));
    let source_context = source_tab_stack_id.and_then(|tab_stack_id| {
        source_dock_drag_context(&projection_artifacts.workspace, tab_stack_id)
    });
    rank_dock_drop_candidates(candidates)
        .into_iter()
        .map(|candidate| {
            classify_dock_drop_candidate(candidate, source_tab_stack_id, source_context.as_ref())
        })
        .collect()
}

fn area_dock_drop_candidates(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Vec<ScoredDockDropCandidate> {
    projected_tab_stacks_for_docking(&projection_artifacts.workspace)
        .into_iter()
        .filter_map(|stack| {
            let anchor_widget_id = editor_shell::tab_stack_container_widget_id(stack.tab_stack_id);
            let container = layouts.get(&anchor_widget_id)?;
            let side = tab_stack_split_side_for_pointer(container.bounds, pointer_position)?;
            Some(ScoredDockDropCandidate {
                candidate: DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoArea {
                        target_tab_stack_id: stack.tab_stack_id,
                        side,
                    },
                    scope: DockDropScope::Area,
                    side,
                    anchor_widget_id,
                    state: DockDropCandidateState::Candidate,
                },
                edge_distance: dock_split_side_priority(container.bounds, pointer_position, side),
                anchor_area: container.bounds.width * container.bounds.height,
                pointer_inside_anchor: container.bounds.contains(pointer_position),
            })
        })
        .collect()
}

fn group_dock_drop_candidates(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Vec<ScoredDockDropCandidate> {
    let root_host_id = projected_host_id(&projection_artifacts.workspace.root_host);
    split_resize_targets(projection_artifacts)
        .into_iter()
        .filter(|target| Some(target.split_host_id) != root_host_id)
        .filter_map(|target| {
            let layout = layouts.get(&target.widget_id)?;
            let side = dock_split_side_for_pointer(layout.bounds, pointer_position)?;
            Some(ScoredDockDropCandidate {
                candidate: DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoHost {
                        target_host_id: target.split_host_id,
                        side,
                    },
                    scope: DockDropScope::Group,
                    side,
                    anchor_widget_id: target.widget_id,
                    state: DockDropCandidateState::Candidate,
                },
                edge_distance: dock_split_side_priority(layout.bounds, pointer_position, side),
                anchor_area: layout.bounds.width * layout.bounds.height,
                pointer_inside_anchor: layout.bounds.contains(pointer_position),
            })
        })
        .collect()
}

fn projected_host_id(host: &ProjectedWorkspaceHostSlot) -> Option<PanelHostId> {
    match host {
        ProjectedWorkspaceHostSlot::Split { host_id, .. }
        | ProjectedWorkspaceHostSlot::TabStack { host_id, .. }
        | ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder { host_id, .. } => Some(*host_id),
    }
}

fn workspace_dock_drop_candidate(
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<ScoredDockDropCandidate> {
    let layout = layouts.get(&BODY_ROOT_WIDGET_ID)?;
    let side = dock_split_side_for_pointer(layout.bounds, pointer_position)?;
    Some(ScoredDockDropCandidate {
        candidate: DockDropCandidate {
            target: DockingPreviewDropTarget::SplitIntoRoot { side },
            scope: DockDropScope::Workspace,
            side,
            anchor_widget_id: BODY_ROOT_WIDGET_ID,
            state: DockDropCandidateState::Candidate,
        },
        edge_distance: dock_split_side_priority(layout.bounds, pointer_position, side),
        anchor_area: layout.bounds.width * layout.bounds.height,
        pointer_inside_anchor: layout.bounds.contains(pointer_position),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceDockDragContext {
    tab_count: usize,
    ancestor_split_hosts: Vec<PanelHostId>,
}

fn source_dock_drag_context(
    projection: &editor_shell::WorkspaceProjectionArtifact,
    source_tab_stack_id: editor_shell::TabStackId,
) -> Option<SourceDockDragContext> {
    source_dock_drag_context_in_host(&projection.root_host, source_tab_stack_id, &mut Vec::new())
        .or_else(|| {
            projection
                .floating_hosts
                .iter()
                .find(|host| host.tab_stack.tab_stack_id == source_tab_stack_id)
                .map(|host| SourceDockDragContext {
                    tab_count: host.tab_stack.tabs.len(),
                    ancestor_split_hosts: Vec::new(),
                })
        })
}

fn source_dock_drag_context_in_host(
    host: &ProjectedWorkspaceHostSlot,
    source_tab_stack_id: editor_shell::TabStackId,
    ancestors: &mut Vec<PanelHostId>,
) -> Option<SourceDockDragContext> {
    match host {
        ProjectedWorkspaceHostSlot::Split {
            host_id,
            first_child,
            second_child,
            ..
        } => {
            ancestors.push(*host_id);
            let found =
                source_dock_drag_context_in_host(first_child, source_tab_stack_id, ancestors)
                    .or_else(|| {
                        source_dock_drag_context_in_host(
                            second_child,
                            source_tab_stack_id,
                            ancestors,
                        )
                    });
            ancestors.pop();
            found
        }
        ProjectedWorkspaceHostSlot::TabStack { tab_stack, .. }
            if tab_stack.tab_stack_id == source_tab_stack_id =>
        {
            Some(SourceDockDragContext {
                tab_count: tab_stack.tabs.len(),
                ancestor_split_hosts: ancestors.clone(),
            })
        }
        ProjectedWorkspaceHostSlot::TabStack { .. }
        | ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder { .. } => None,
    }
}

fn classify_dock_drop_candidate(
    mut candidate: DockDropCandidate,
    source_tab_stack_id: Option<editor_shell::TabStackId>,
    source_context: Option<&SourceDockDragContext>,
) -> DockDropCandidate {
    let Some(source_tab_stack_id) = source_tab_stack_id else {
        return candidate;
    };
    let Some(source_context) = source_context else {
        return candidate;
    };
    if source_context.tab_count > 1 {
        return candidate;
    }

    let reason = match candidate.target {
        DockingPreviewDropTarget::SplitIntoArea {
            target_tab_stack_id,
            ..
        } if target_tab_stack_id == source_tab_stack_id => {
            Some(DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea)
        }
        DockingPreviewDropTarget::SplitIntoHost { target_host_id, .. }
            if source_context
                .ancestor_split_hosts
                .contains(&target_host_id) =>
        {
            Some(DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnHost)
        }
        _ => None,
    };

    if let Some(reason) = reason {
        candidate.state = DockDropCandidateState::Invalid { reason };
    }
    candidate
}

fn rank_dock_drop_candidates(
    mut candidates: Vec<ScoredDockDropCandidate>,
) -> Vec<DockDropCandidate> {
    let area_interior = candidates.iter().any(|candidate| {
        candidate.candidate.scope == DockDropScope::Area && candidate.pointer_inside_anchor
    });
    let group_available = candidates
        .iter()
        .any(|candidate| candidate.candidate.scope == DockDropScope::Group);
    candidates.sort_by(|left, right| {
        dock_drop_scope_priority(left, area_interior, group_available)
            .cmp(&dock_drop_scope_priority(
                right,
                area_interior,
                group_available,
            ))
            .then_with(|| left.edge_distance.total_cmp(&right.edge_distance))
            .then_with(|| left.anchor_area.total_cmp(&right.anchor_area))
    });
    candidates
        .into_iter()
        .map(|candidate| candidate.candidate)
        .collect()
}

fn dock_drop_scope_priority(
    candidate: &ScoredDockDropCandidate,
    area_interior: bool,
    group_available: bool,
) -> u8 {
    match candidate.candidate.scope {
        DockDropScope::Workspace
            if candidate.edge_distance <= SPLIT_HIT_SLOP_PX
                && (!group_available
                    || matches!(
                        candidate.candidate.side,
                        DockSplitSide::Top | DockSplitSide::Bottom
                    )) =>
        {
            0
        }
        DockDropScope::Area if candidate.pointer_inside_anchor => 1,
        DockDropScope::Group => 2,
        DockDropScope::Workspace if !area_interior && !group_available => 1,
        DockDropScope::Workspace => 2,
        DockDropScope::Area => 3,
    }
}

fn activate_dock_drop_candidate(
    mut candidates: Vec<DockDropCandidate>,
    cycle_index: usize,
    cycle_side: Option<DockSplitSide>,
) -> ResolvedTabDropPreview {
    let Some(default_side) = candidates
        .iter()
        .find(|candidate| candidate.state.is_selectable())
        .map(|candidate| candidate.side)
    else {
        return ResolvedTabDropPreview {
            target: None,
            candidates,
            active_side: None,
        };
    };
    let active_side = cycle_side.filter(|side| {
        candidates
            .iter()
            .any(|candidate| candidate.state.is_selectable() && candidate.side == *side)
    });
    let active_side = active_side.unwrap_or(default_side);
    let same_side_indices = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            (candidate.state.is_selectable() && candidate.side == active_side).then_some(index)
        })
        .collect::<Vec<_>>();
    let active_index = same_side_indices[cycle_index % same_side_indices.len()];
    for (index, candidate) in candidates.iter_mut().enumerate() {
        if candidate.state.is_selectable() {
            candidate.state = DockDropCandidateState::selectable(index == active_index);
        }
    }
    ResolvedTabDropPreview {
        target: Some(candidates[active_index].target),
        candidates,
        active_side: Some(active_side),
    }
}

fn dock_split_side_for_pointer(
    bounds: ui_math::UiRect,
    pointer_position: ui_math::UiPoint,
) -> Option<DockSplitSide> {
    let left = bounds.x;
    let right = bounds.x + bounds.width;
    let top = bounds.y;
    let bottom = bounds.y + bounds.height;
    let side_slop = SPLIT_HIT_SLOP_PX;
    let within_vertical_band =
        pointer_position.y >= top - side_slop && pointer_position.y <= bottom + side_slop;
    let within_horizontal_band =
        pointer_position.x >= left - side_slop && pointer_position.x <= right + side_slop;

    let mut candidates = Vec::with_capacity(4);
    let left_delta = pointer_position.x - left;
    if within_vertical_band
        && left_delta >= -side_slop
        && left_delta <= SIDE_DOCK_DROP_TARGET_EDGE_PX
    {
        candidates.push((left_delta.abs(), DockSplitSide::Left));
    }
    let right_delta = pointer_position.x - right;
    if within_vertical_band
        && right_delta >= -SIDE_DOCK_DROP_TARGET_EDGE_PX
        && right_delta <= side_slop
    {
        candidates.push((right_delta.abs(), DockSplitSide::Right));
    }
    let top_delta = pointer_position.y - top;
    if within_horizontal_band
        && top_delta >= -side_slop
        && top_delta <= SIDE_DOCK_DROP_TARGET_EDGE_PX
    {
        candidates.push((top_delta.abs(), DockSplitSide::Top));
    }
    let bottom_delta = pointer_position.y - bottom;
    if within_horizontal_band
        && bottom_delta >= -SIDE_DOCK_DROP_TARGET_EDGE_PX
        && bottom_delta <= side_slop
    {
        candidates.push((bottom_delta.abs(), DockSplitSide::Bottom));
    }

    candidates
        .into_iter()
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, side)| side)
}

fn tab_stack_split_side_for_pointer(
    bounds: ui_math::UiRect,
    pointer_position: ui_math::UiPoint,
) -> Option<DockSplitSide> {
    let left = bounds.x;
    let right = bounds.x + bounds.width;
    let top = bounds.y;
    let bottom = bounds.y + bounds.height;
    let side_slop = SPLIT_HIT_SLOP_PX;
    let within_vertical_band = pointer_position.y >= top && pointer_position.y <= bottom;
    let within_horizontal_band = pointer_position.x >= left && pointer_position.x <= right;

    let mut candidates = Vec::with_capacity(4);
    let left_delta = pointer_position.x - left;
    if within_vertical_band
        && left_delta >= -side_slop
        && left_delta <= SIDE_DOCK_DROP_TARGET_EDGE_PX
    {
        candidates.push((left_delta.abs(), DockSplitSide::Left));
    }
    let right_delta = pointer_position.x - right;
    if within_vertical_band
        && right_delta >= -SIDE_DOCK_DROP_TARGET_EDGE_PX
        && right_delta <= side_slop
    {
        candidates.push((right_delta.abs(), DockSplitSide::Right));
    }
    let top_delta = pointer_position.y - top;
    if within_horizontal_band
        && top_delta >= -side_slop
        && top_delta <= SIDE_DOCK_DROP_TARGET_EDGE_PX
    {
        candidates.push((top_delta.abs(), DockSplitSide::Top));
    }
    let bottom_delta = pointer_position.y - bottom;
    if within_horizontal_band
        && bottom_delta >= -SIDE_DOCK_DROP_TARGET_EDGE_PX
        && bottom_delta <= side_slop
    {
        candidates.push((bottom_delta.abs(), DockSplitSide::Bottom));
    }

    candidates
        .into_iter()
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, side)| side)
}

fn dock_split_side_priority(
    bounds: ui_math::UiRect,
    pointer_position: ui_math::UiPoint,
    side: DockSplitSide,
) -> f32 {
    match side {
        DockSplitSide::Left => (pointer_position.x - bounds.x).abs(),
        DockSplitSide::Right => (pointer_position.x - (bounds.x + bounds.width)).abs(),
        DockSplitSide::Top => (pointer_position.y - bounds.y).abs(),
        DockSplitSide::Bottom => (pointer_position.y - (bounds.y + bounds.height)).abs(),
    }
}

fn projected_tab_stacks_for_docking(
    projection: &editor_shell::WorkspaceProjectionArtifact,
) -> Vec<&editor_shell::ProjectedTabStackSlot> {
    let mut stacks = editor_shell::projected_host_tab_stacks(&projection.root_host);
    stacks.extend(projection.floating_hosts.iter().map(|host| &host.tab_stack));
    stacks
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
        editor_shell::ProjectedTabDropTarget::SplitIntoArea {
            target_tab_stack_id,
            side,
        } => DockingPreviewDropTarget::SplitIntoArea {
            target_tab_stack_id,
            side,
        },
        editor_shell::ProjectedTabDropTarget::SplitIntoHost {
            target_host_id,
            side,
        } => DockingPreviewDropTarget::SplitIntoHost {
            target_host_id,
            side,
        },
        editor_shell::ProjectedTabDropTarget::SplitIntoRoot { side } => {
            DockingPreviewDropTarget::SplitIntoRoot { side }
        }
        editor_shell::ProjectedTabDropTarget::NewFloatingHost => {
            DockingPreviewDropTarget::NewFloatingHost
        }
    }
}

fn is_at_bottom(offset: f32, max_offset: f32) -> bool {
    max_offset <= CONSOLE_FOLLOW_BOTTOM_EPSILON
        || offset >= (max_offset - CONSOLE_FOLLOW_BOTTOM_EPSILON)
}
