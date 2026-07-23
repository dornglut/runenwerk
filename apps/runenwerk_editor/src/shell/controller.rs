use editor_shell::{
    CONSOLE_SCROLL_WIDGET_ID, CommandCapabilityKey, ComputedLayoutMap, DockDropCandidate,
    DockDropCandidateState, DockDropInvalidTargetReason, DockDropScope, DockSplitSide,
    DockingPreviewDropTarget, EditorShellFrameModel, HostCapabilityPolicy,
    HostCapabilityRequirements, RegionCompassAccessibility, RegionCompassViewModel, ShellCommand,
    ShellUiExpressionFrame, SurfaceCommandProposal, TOOLBAR_ADD_WORKSPACE_WIDGET_ID,
    TOOLBAR_EDIT_MENU_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID, TOOLBAR_MENU_POPUP_WIDGET_ID,
    TOOLBAR_WINDOW_MENU_WIDGET_ID, UiInputOutcome, UiInteractionResults, UiTree,
    VIEWPORT_OPTIONS_BUTTON_WIDGET_ID, VIEWPORT_OPTIONS_POPUP_WIDGET_ID,
    VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID, VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID,
    VIEWPORT_TOOLS_MENU_WIDGET_ID, WidgetId,
    build_editor_shell_frame_for_target_from_composition_projection_with_docking_visual_state,
    build_editor_shell_frame_from_composition_projection_with_docking_visual_state,
    map_interactions_to_shell_commands, surface_widget_id, tab_stack_container_widget_id,
    tab_stack_popup_menu_widget_id, viewport_tool_radial_item_widget_id,
};
use editor_viewport::ArtifactObservationFrame;
use ui_adaptive_composition::DockZone;
use ui_composition::PresentationTargetId;
use ui_input::{
    EventPropagation, FocusChange, InputResponse, PointerCapture, PointerEventKind,
    SemanticActionEvent, SemanticDirection, UiInputEvent, UiSemanticAction,
};
use ui_math::{Axis, UiRect};
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPresentationStateResource,
    ViewportRenderStateCommandQueueResource,
};
use crate::shell::tool_suites::EDITOR_CONSOLE_SURFACE_KEY;
use crate::shell::{
    EditorShellFrameMetrics, EditorSurfaceProviderRegistry, RunenwerkEditorShellState,
    SurfaceProviderDispatchContext, active_document_context,
    build_editor_shell_frame_model_with_frame_metrics,
    dispatch_shell_command_with_viewport_commands, mounted_surface_requests,
};

const CONSOLE_FOLLOW_BOTTOM_EPSILON: f32 = 1.0;
const SURFACE_SESSION_MUTATION_COMMAND_CAPABILITY: &str = "runenwerk.surface.session_mutation";
const EDITOR_DOMAIN_MUTATION_COMMAND_CAPABILITY: &str = "runenwerk.editor.domain_mutation";
const SHELL_COMMAND_PROPOSAL_COMMAND_CAPABILITY: &str = "runenwerk.shell.command";
const HOST_POLICY_DENIED_PROVIDER_PROPOSAL: &str =
    "host capability policy denied provider proposal";

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
        let build_result =
            build_editor_shell_frame_from_composition_projection_with_docking_visual_state(
                &frame_model,
                theme,
                shell_state.composition_projection(),
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
        let build_result =
            build_editor_shell_frame_from_composition_projection_with_docking_visual_state(
                &frame_model,
                theme,
                shell_state.composition_projection(),
                Some(&docking_visual_state),
            );
        let tree = build_result.tree;
        shell_state.cache_projection_artifacts(build_result.projection_artifacts);
        shell_state.set_last_tree(tree.clone());
        Self::finish_expression_frame(app, shell_state, bounds, atlas_source, tree)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_expression_frame_for_target_with_surface_resources(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        target_id: PresentationTargetId,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
        tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
        viewport_instances: Option<&ViewportInstanceRegistryResource>,
        frame_metrics: Option<EditorShellFrameMetrics>,
    ) -> Option<ShellUiExpressionFrame> {
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
        let docking_visual_state = shell_state.docking_visual_state_for_target(target_id);
        let build_result = build_editor_shell_frame_for_target_from_composition_projection_with_docking_visual_state(
            &frame_model,
            theme,
            shell_state.composition_projection(),
            target_id,
            Some(&docking_visual_state),
        )?;
        let tree = build_result.tree;
        shell_state
            .cache_projection_artifacts_for_target(target_id, build_result.projection_artifacts);
        shell_state.set_last_tree_for_target(target_id, tree.clone());
        shell_state.set_last_bounds_for_target(target_id, bounds);
        let frame = {
            let runtime = shell_state.runtime_for_target_mut(target_id);
            runtime.state_mut().advance_frame();
            runtime.build_frame(&tree, bounds, atlas_source)
        };
        Some(ShellUiExpressionFrame::new(
            app.runtime().current_scene_reality_version(),
            frame,
        ))
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
        let build_result =
            build_editor_shell_frame_from_composition_projection_with_docking_visual_state(
                &frame_model,
                theme,
                shell_state.composition_projection(),
                Some(&docking_visual_state),
            );
        let tree = build_result.tree.clone();
        let projection_artifacts =
            shell_state.cache_projection_artifacts(build_result.projection_artifacts);
        shell_state.set_last_tree(tree.clone());
        shell_state.set_last_bounds(bounds);

        let layouts = shell_state.runtime().compute_layout(&tree, bounds);
        let focused_widget = shell_state
            .runtime()
            .state()
            .focused_target
            .map(|target| WidgetId(target.0));
        let primary_target_id = shell_state.primary_composition_target_id();
        let (split_resize_handled, split_resize_command) = update_composition_split_resize(
            shell_state,
            primary_target_id,
            event,
            &layouts,
            &projection_artifacts,
        );
        let docking_command = (!split_resize_handled)
            .then(|| {
                update_region_compass_docking(
                    shell_state,
                    primary_target_id,
                    event,
                    &layouts,
                    &projection_artifacts,
                    focused_widget,
                )
            })
            .flatten();
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

        if !split_resize_handled
            && let Some(outcome) =
                Self::handle_tab_popup_dismiss_event(shell_state, event, &layouts)
        {
            return Ok(outcome);
        }
        if !split_resize_handled
            && let Some(outcome) =
                Self::handle_toolbar_menu_dismiss_event(shell_state, event, &layouts)
        {
            return Ok(outcome);
        }
        if !split_resize_handled
            && let Some(outcome) =
                Self::handle_viewport_options_menu_dismiss_event(app, shell_state, event, &layouts)
        {
            return Ok(outcome);
        }
        if !split_resize_handled
            && let Some(outcome) = Self::handle_viewport_tool_radial_menu_dismiss_event(
                app,
                shell_state,
                event,
                &layouts,
            )
        {
            return Ok(outcome);
        }
        let outcome = if split_resize_handled {
            consumed_pointer_outcome(None, true)
        } else {
            shell_state
                .runtime_mut()
                .dispatch_input(&tree, &layouts, event)
        };

        if is_console_scroll_event(event, &outcome, console_scroll_widget_id) {
            let post_layouts = shell_state.runtime().compute_layout(&tree, bounds);
            if let (Some((mounted_unit_id, _surface_id)), Some(console_scroll_widget_id)) = (
                active_console_binding(shell_state),
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
                        .session_mut(mounted_unit_id)
                        .console_follow_enabled = true;
                } else if pre_at_bottom {
                    app.surface_sessions_mut()
                        .session_mut(mounted_unit_id)
                        .console_follow_enabled = false;
                }
            }
        }

        let mut commands =
            map_interactions_to_shell_commands(&outcome.interactions, &projection_artifacts);
        if let Some(command) = docking_command {
            commands.push(command);
        }
        if let Some(command) = split_resize_command {
            commands.push(command);
        }

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

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_input_for_target_with_viewport_products(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        target_id: PresentationTargetId,
        bounds: UiRect,
        theme: &ThemeTokens,
        event: &UiInputEvent,
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
        let docking_visual_state = shell_state.docking_visual_state_for_target(target_id);
        let build_result = build_editor_shell_frame_for_target_from_composition_projection_with_docking_visual_state(
            &frame_model,
            theme,
            shell_state.composition_projection(),
            target_id,
            Some(&docking_visual_state),
        )
        .ok_or_else(|| {
            editor_core::EditorMutationError::runtime_rejected(
                "composition target has no editor shell projection",
            )
        })?;
        let tree = build_result.tree;
        let projection_artifacts = shell_state
            .cache_projection_artifacts_for_target(target_id, build_result.projection_artifacts);
        shell_state.set_last_tree_for_target(target_id, tree.clone());
        shell_state.set_last_bounds_for_target(target_id, bounds);
        let layouts = shell_state
            .runtime_for_target_mut(target_id)
            .compute_layout(&tree, bounds);
        let focused_widget = shell_state
            .runtime_for_target(target_id)
            .and_then(|runtime| runtime.state().focused_target)
            .map(|target| WidgetId(target.0));
        let (split_resize_handled, split_resize_command) = update_composition_split_resize(
            shell_state,
            target_id,
            event,
            &layouts,
            &projection_artifacts,
        );
        let docking_command = (!split_resize_handled)
            .then(|| {
                update_region_compass_docking(
                    shell_state,
                    target_id,
                    event,
                    &layouts,
                    &projection_artifacts,
                    focused_widget,
                )
            })
            .flatten();
        let outcome = if split_resize_handled {
            consumed_pointer_outcome(None, true)
        } else {
            shell_state
                .runtime_for_target_mut(target_id)
                .dispatch_input(&tree, &layouts, event)
        };
        let mut commands =
            map_interactions_to_shell_commands(&outcome.interactions, &projection_artifacts);
        if let Some(command) = docking_command {
            commands.push(command);
        }
        if let Some(command) = split_resize_command {
            commands.push(command);
        }
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

        let open_viewport_surfaces = mounted_content_bindings(shell_state)
            .into_iter()
            .filter(|(_, _, stable_key)| is_viewport_content_key(stable_key))
            .filter(|(mounted_unit_id, _, _)| {
                app.surface_sessions()
                    .session(*mounted_unit_id)
                    .is_some_and(|session| session.viewport_options_menu_open)
            })
            .map(|(_, surface_id, _)| surface_id)
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

        let open_viewport_surfaces = mounted_content_bindings(shell_state)
            .into_iter()
            .filter(|(_, _, stable_key)| is_viewport_content_key(stable_key))
            .filter(|(mounted_unit_id, _, _)| {
                app.surface_sessions()
                    .session(*mounted_unit_id)
                    .is_some_and(|session| {
                        session.viewport_tools_menu_open
                            || session.viewport_tool_radial_session.is_some()
                    })
            })
            .map(|(_, surface_id, _)| surface_id)
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

    pub fn dispatch_commands(
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
                {
                    enforce_surface_proposal_host_policy(
                        app.workbench_host().host_capability_policy(),
                        &proposal,
                    )?;
                    if let Some(command) = shell_command_from_surface_proposal(proposal) {
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
                )? {
                    enforce_surface_proposal_host_policy(
                        app.workbench_host().host_capability_policy(),
                        &proposal,
                    )?;
                    if let Some(command) = shell_command_from_surface_proposal(proposal) {
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
        _shell_state: &RunenwerkEditorShellState,
        _pointer: ui_math::UiPoint,
    ) -> ShellCursorIntent {
        ShellCursorIntent::Default
    }
}

fn enforce_surface_proposal_host_policy(
    policy: &HostCapabilityPolicy,
    proposal: &SurfaceCommandProposal,
) -> Result<(), editor_core::EditorMutationError> {
    let requirements = surface_proposal_capability_requirements(proposal);
    if requirements.denied_by(policy).is_some() {
        return Err(editor_core::EditorMutationError::session_rejected(
            HOST_POLICY_DENIED_PROVIDER_PROPOSAL,
        ));
    }

    Ok(())
}

fn surface_proposal_capability_requirements(
    proposal: &SurfaceCommandProposal,
) -> HostCapabilityRequirements {
    let command = match proposal {
        SurfaceCommandProposal::SurfaceSession(_) => SURFACE_SESSION_MUTATION_COMMAND_CAPABILITY,
        SurfaceCommandProposal::EditorDomain(_) => EDITOR_DOMAIN_MUTATION_COMMAND_CAPABILITY,
        SurfaceCommandProposal::Shell(_) => SHELL_COMMAND_PROPOSAL_COMMAND_CAPABILITY,
    };

    HostCapabilityRequirements::new().require_command(command_capability_key(command))
}

fn command_capability_key(value: &'static str) -> CommandCapabilityKey {
    CommandCapabilityKey::new(value).expect("compiled-in command capability key should be valid")
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

#[derive(Clone, Copy)]
struct ProjectedSplitResizeTarget {
    host: editor_shell::PanelHostId,
    widget: WidgetId,
    axis: editor_shell::WorkspaceSplitAxis,
    fraction: f32,
}

fn update_composition_split_resize(
    shell_state: &mut RunenwerkEditorShellState,
    target_id: PresentationTargetId,
    event: &UiInputEvent,
    layouts: &ComputedLayoutMap,
    projection: &editor_shell::ShellProjectionArtifacts,
) -> (bool, Option<ShellCommand>) {
    let UiInputEvent::Pointer(pointer) = event else {
        if matches!(
            event,
            UiInputEvent::Semantic(SemanticActionEvent {
                action: UiSemanticAction::Cancel | UiSemanticAction::Rollback,
                ..
            })
        ) && shell_state
            .active_split_resize_session_for_target(target_id)
            .is_some()
        {
            shell_state.clear_split_resize_for_target(target_id);
            return (true, None);
        }
        return (false, None);
    };

    match pointer.kind {
        PointerEventKind::Down if pointer.button == Some(ui_input::PointerButton::Primary) => {
            let Some(split) =
                nearest_split_boundary(&projection.workspace.root_host, layouts, pointer.position)
            else {
                return (false, None);
            };
            shell_state.begin_workspace_split_resize_for_target(
                target_id,
                split.host,
                split.widget,
                split.axis,
            );
            (true, None)
        }
        PointerEventKind::Move => {
            let Some((_, widget, axis)) =
                shell_state.active_split_resize_session_for_target(target_id)
            else {
                return (false, None);
            };
            let Some(layout) = layouts.get(&widget) else {
                shell_state.clear_split_resize_for_target(target_id);
                return (true, None);
            };
            let fraction = split_fraction_from_pointer(layout.bounds, axis, pointer.position);
            shell_state.update_split_resize_preview_for_target(target_id, fraction);
            (true, None)
        }
        PointerEventKind::Up if pointer.button == Some(ui_input::PointerButton::Primary) => {
            if let Some((_, widget, axis)) =
                shell_state.active_split_resize_session_for_target(target_id)
                && let Some(layout) = layouts.get(&widget)
            {
                let fraction = split_fraction_from_pointer(layout.bounds, axis, pointer.position);
                shell_state.update_split_resize_preview_for_target(target_id, fraction);
            }
            let Some((split, fraction, expected_revision)) =
                shell_state.finish_split_resize_for_target(target_id)
            else {
                return (false, None);
            };
            (
                true,
                Some(ShellCommand::ResizeCompositionSplit {
                    split,
                    fraction,
                    expected_revision,
                    projection_epoch: projection.projection_epoch,
                }),
            )
        }
        _ => (false, None),
    }
}

fn nearest_split_boundary(
    host: &editor_shell::ProjectedWorkspaceHostSlot,
    layouts: &ComputedLayoutMap,
    pointer: ui_math::UiPoint,
) -> Option<ProjectedSplitResizeTarget> {
    let mut targets = Vec::new();
    collect_projected_split_targets(host, &mut targets);
    targets
        .into_iter()
        .filter_map(|target| {
            let bounds = layouts.get(&target.widget)?.bounds;
            let (distance, inside_cross_axis) = match target.axis {
                editor_shell::WorkspaceSplitAxis::Horizontal => (
                    (pointer.x - (bounds.x + bounds.width * target.fraction)).abs(),
                    pointer.y >= bounds.y - 6.0 && pointer.y <= bounds.y + bounds.height + 6.0,
                ),
                editor_shell::WorkspaceSplitAxis::Vertical => (
                    (pointer.y - (bounds.y + bounds.height * target.fraction)).abs(),
                    pointer.x >= bounds.x - 6.0 && pointer.x <= bounds.x + bounds.width + 6.0,
                ),
            };
            (inside_cross_axis && distance <= 6.0).then_some((distance, target))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, target)| target)
}

fn collect_projected_split_targets(
    host: &editor_shell::ProjectedWorkspaceHostSlot,
    targets: &mut Vec<ProjectedSplitResizeTarget>,
) {
    let editor_shell::ProjectedWorkspaceHostSlot::Split {
        host_id,
        widget_id,
        axis,
        fraction,
        first_child,
        second_child,
        ..
    } = host
    else {
        return;
    };
    targets.push(ProjectedSplitResizeTarget {
        host: *host_id,
        widget: *widget_id,
        axis: *axis,
        fraction: *fraction,
    });
    collect_projected_split_targets(first_child, targets);
    collect_projected_split_targets(second_child, targets);
}

fn split_fraction_from_pointer(
    bounds: UiRect,
    axis: editor_shell::WorkspaceSplitAxis,
    pointer: ui_math::UiPoint,
) -> ui_composition::SplitFraction {
    let raw = match axis {
        editor_shell::WorkspaceSplitAxis::Horizontal => {
            (pointer.x - bounds.x) / bounds.width.max(1.0)
        }
        editor_shell::WorkspaceSplitAxis::Vertical => {
            (pointer.y - bounds.y) / bounds.height.max(1.0)
        }
    };
    let basis_points = (raw.clamp(0.1, 0.9) * 10_000.0).round() as u16;
    ui_composition::SplitFraction::try_new(basis_points)
        .expect("clamped composition split fraction is valid")
}

fn update_region_compass_docking(
    shell_state: &mut RunenwerkEditorShellState,
    target_id: PresentationTargetId,
    event: &UiInputEvent,
    layouts: &ComputedLayoutMap,
    projection: &editor_shell::ShellProjectionArtifacts,
    focused_widget: Option<WidgetId>,
) -> Option<ShellCommand> {
    let epoch = projection.projection_epoch;
    match event {
        UiInputEvent::Pointer(pointer)
            if pointer.kind == PointerEventKind::Down
                && pointer.button == Some(ui_input::PointerButton::Secondary) =>
        {
            let (widget_id, route) = projection
                .workspace
                .tab_button_route_by_widget_id
                .iter()
                .filter_map(|(widget_id, route)| {
                    layouts
                        .get(widget_id)
                        .filter(|layout| layout.bounds.contains(pointer.position))
                        .map(|layout| {
                            (
                                layout.bounds.width * layout.bounds.height,
                                *widget_id,
                                route,
                            )
                        })
                })
                .min_by(|left, right| left.0.total_cmp(&right.0))
                .map(|(_, widget_id, route)| (widget_id, route))?;
            Some(ShellCommand::ToggleTabStackActionMenu {
                tab_stack_id: route.tab_stack_id,
                anchor_widget_id: widget_id,
            })
        }
        UiInputEvent::Pointer(pointer)
            if pointer.kind == PointerEventKind::Down
                && pointer.button == Some(ui_input::PointerButton::Primary) =>
        {
            let route = projection
                .workspace
                .tab_button_route_by_widget_id
                .iter()
                .filter_map(|(widget_id, route)| {
                    layouts
                        .get(widget_id)
                        .filter(|layout| layout.bounds.contains(pointer.position))
                        .map(|layout| (layout.bounds.width * layout.bounds.height, route))
                })
                .min_by(|left, right| left.0.total_cmp(&right.0))
                .map(|(_, route)| route)?;
            shell_state.begin_tab_drag_candidate_for_target(
                target_id,
                route.panel_instance_id,
                route.tab_stack_id,
                pointer.position,
                epoch,
            );
            None
        }
        UiInputEvent::Pointer(pointer) if pointer.kind == PointerEventKind::Move => {
            if !shell_state.update_tab_drag_pointer_for_target(target_id, pointer.position, epoch) {
                return None;
            }
            update_region_compass_preview(
                shell_state,
                target_id,
                pointer.position,
                layouts,
                projection,
            );
            None
        }
        UiInputEvent::Pointer(pointer)
            if pointer.kind == PointerEventKind::Up
                && pointer.button == Some(ui_input::PointerButton::Primary) =>
        {
            finish_region_compass_docking(shell_state, target_id, epoch)
        }
        UiInputEvent::Keyboard(keyboard)
            if matches!(
                keyboard.state,
                ui_input::KeyState::Pressed | ui_input::KeyState::Repeated
            ) =>
        {
            let action = if keyboard.modifiers.alt
                && keyboard.modifiers.shift
                && keyboard.key == ui_input::Key::Character("d".to_owned())
            {
                Some(UiSemanticAction::EnterMoveMode)
            } else {
                match keyboard.key {
                    ui_input::Key::Escape => Some(UiSemanticAction::Cancel),
                    ui_input::Key::Enter => Some(UiSemanticAction::Commit),
                    ui_input::Key::Tab if keyboard.modifiers.shift => {
                        Some(UiSemanticAction::Focus(SemanticDirection::Previous))
                    }
                    ui_input::Key::Tab => Some(UiSemanticAction::Focus(SemanticDirection::Next)),
                    ui_input::Key::Left => Some(UiSemanticAction::Focus(SemanticDirection::Left)),
                    ui_input::Key::Right => Some(UiSemanticAction::Focus(SemanticDirection::Right)),
                    ui_input::Key::Up => Some(UiSemanticAction::Focus(SemanticDirection::Up)),
                    ui_input::Key::Down => Some(UiSemanticAction::Focus(SemanticDirection::Down)),
                    _ => None,
                }
            }?;
            handle_region_compass_semantic_action(
                shell_state,
                target_id,
                SemanticActionEvent::new(ui_input::SemanticInputSource::Keyboard, action)
                    .repeated(keyboard.state == ui_input::KeyState::Repeated),
                layouts,
                projection,
                focused_widget,
            )
        }
        UiInputEvent::Semantic(action) => handle_region_compass_semantic_action(
            shell_state,
            target_id,
            *action,
            layouts,
            projection,
            focused_widget,
        ),
        _ => None,
    }
}

fn handle_region_compass_semantic_action(
    shell_state: &mut RunenwerkEditorShellState,
    target_id: PresentationTargetId,
    event: SemanticActionEvent,
    layouts: &ComputedLayoutMap,
    projection: &editor_shell::ShellProjectionArtifacts,
    focused_widget: Option<WidgetId>,
) -> Option<ShellCommand> {
    let epoch = projection.projection_epoch;
    match event.action {
        UiSemanticAction::EnterMoveMode => {
            let focused_widget = focused_widget?;
            let route = projection
                .workspace
                .tab_button_route_by_widget_id
                .get(&focused_widget)?;
            let anchor = tab_stack_container_widget_id(route.tab_stack_id);
            let bounds = layouts.get(&anchor)?.bounds;
            let center = ui_math::UiPoint::new(
                bounds.x + bounds.width * 0.5,
                bounds.y + bounds.height * 0.5,
            );
            shell_state.begin_tab_drag_candidate_for_target(
                target_id,
                route.panel_instance_id,
                route.tab_stack_id,
                center,
                epoch,
            );
            shell_state.update_tab_drag_pointer_for_target(
                target_id,
                ui_math::UiPoint::new(center.x + 7.0, center.y),
                epoch,
            );
            set_region_compass_zone(
                shell_state,
                target_id,
                route.tab_stack_id,
                DockZone::Center,
                projection,
                epoch,
            );
            None
        }
        UiSemanticAction::Cancel | UiSemanticAction::Rollback => {
            shell_state.clear_tab_drag_for_target(target_id);
            None
        }
        UiSemanticAction::Commit | UiSemanticAction::Activate => {
            finish_region_compass_docking(shell_state, target_id, epoch)
        }
        UiSemanticAction::Focus(SemanticDirection::Next) => {
            shell_state.focus_region_compass_detach_for_target(target_id);
            shell_state.set_tab_drag_preview_for_target(
                target_id,
                Some(DockingPreviewDropTarget::NewFloatingHost),
                Vec::new(),
                None,
                epoch,
            );
            None
        }
        UiSemanticAction::Focus(direction) => {
            let compass = shell_state.region_compass_for_target(target_id)?;
            let target_stack = shell_state.tab_stack_id_for_region(compass.region)?;
            let zone = semantic_direction_to_dock_zone(direction)?;
            set_region_compass_zone(
                shell_state,
                target_id,
                target_stack,
                zone,
                projection,
                epoch,
            );
            None
        }
        UiSemanticAction::CycleTab(_) | UiSemanticAction::EnterResizeMode(_) => None,
    }
}

fn semantic_direction_to_dock_zone(direction: SemanticDirection) -> Option<DockZone> {
    match direction {
        SemanticDirection::Previous => Some(DockZone::Center),
        SemanticDirection::Left => Some(DockZone::Left),
        SemanticDirection::Right => Some(DockZone::Right),
        SemanticDirection::Up => Some(DockZone::Top),
        SemanticDirection::Down => Some(DockZone::Bottom),
        SemanticDirection::Next => None,
    }
}

fn update_region_compass_preview(
    shell_state: &mut RunenwerkEditorShellState,
    target_id: PresentationTargetId,
    pointer: ui_math::UiPoint,
    layouts: &ComputedLayoutMap,
    projection: &editor_shell::ShellProjectionArtifacts,
) {
    let epoch = projection.projection_epoch;
    let visual = shell_state.docking_visual_state_for_target(target_id);
    if let Some(anchor) = visual
        .active_tab_drag
        .as_ref()
        .and_then(|drag| drag.region_compass_anchor)
        && layouts
            .get(&editor_shell::region_compass_detach_button_widget_id(
                anchor,
            ))
            .is_some_and(|layout| layout.bounds.contains(pointer))
    {
        shell_state.focus_region_compass_detach_for_target(target_id);
        shell_state.set_tab_drag_preview_for_target(
            target_id,
            Some(DockingPreviewDropTarget::NewFloatingHost),
            Vec::new(),
            None,
            epoch,
        );
        return;
    }

    let target = projected_tab_stacks(&projection.workspace)
        .into_iter()
        .filter_map(|stack| {
            let anchor = tab_stack_container_widget_id(stack.tab_stack_id);
            layouts
                .get(&anchor)
                .filter(|layout| layout.bounds.contains(pointer))
                .map(|layout| {
                    (
                        layout.bounds.width * layout.bounds.height,
                        stack.tab_stack_id,
                        layout.bounds,
                    )
                })
        })
        .min_by(|left, right| left.0.total_cmp(&right.0));
    let Some((_, tab_stack_id, bounds)) = target else {
        shell_state.clear_region_compass_for_target(target_id);
        return;
    };
    let zone = compass_zone(bounds, pointer);
    set_region_compass_zone(
        shell_state,
        target_id,
        tab_stack_id,
        zone,
        projection,
        epoch,
    );
}

fn set_region_compass_zone(
    shell_state: &mut RunenwerkEditorShellState,
    target_id: PresentationTargetId,
    target_tab_stack_id: editor_shell::TabStackId,
    zone: DockZone,
    projection: &editor_shell::ShellProjectionArtifacts,
    epoch: u64,
) {
    let Some(target_stack) = projected_tab_stacks(&projection.workspace)
        .into_iter()
        .find(|stack| stack.tab_stack_id == target_tab_stack_id)
    else {
        return;
    };
    let source_stack = shell_state
        .docking_visual_state_for_target(target_id)
        .active_tab_drag
        .map(|drag| drag.source_tab_stack_id);
    let source_unit = shell_state
        .docking_visual_state_for_target(target_id)
        .active_tab_drag
        .and_then(|drag| shell_state.mounted_unit_id_for_panel(drag.panel_instance_id));
    let Some(source_unit) = source_unit else {
        shell_state.clear_region_compass_for_target(target_id);
        return;
    };
    let Some(target_region) = shell_state.region_id_for_tab_stack(target_tab_stack_id) else {
        shell_state.clear_region_compass_for_target(target_id);
        return;
    };
    let anchor_widget_id = tab_stack_container_widget_id(target_tab_stack_id);
    let sides = [
        (DockZone::Left, DockSplitSide::Left),
        (DockZone::Right, DockSplitSide::Right),
        (DockZone::Top, DockSplitSide::Top),
        (DockZone::Bottom, DockSplitSide::Bottom),
    ];
    let mut candidates = sides
        .into_iter()
        .map(|(candidate_zone, side)| {
            let invalid = source_stack == Some(target_tab_stack_id) && target_stack.tabs.len() == 1;
            DockDropCandidate {
                target: DockingPreviewDropTarget::SplitIntoArea {
                    target_tab_stack_id,
                    side,
                },
                scope: DockDropScope::Area,
                side,
                anchor_widget_id,
                state: if invalid {
                    DockDropCandidateState::Invalid {
                        reason: DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea,
                    }
                } else {
                    DockDropCandidateState::selectable(candidate_zone == zone)
                },
            }
        })
        .collect::<Vec<_>>();
    let preview = if zone == DockZone::Center {
        Some(DockingPreviewDropTarget::TabStack {
            tab_stack_id: target_tab_stack_id,
            insert_index: target_stack.tabs.len(),
        })
    } else {
        candidates
            .iter()
            .find(|candidate| candidate.state.is_active())
            .map(|candidate| candidate.target)
    };
    let active_side = candidates
        .iter()
        .find(|candidate| candidate.state.is_active())
        .map(|candidate| candidate.side);
    if zone == DockZone::Center {
        for candidate in &mut candidates {
            if candidate.state.is_selectable() {
                candidate.state = DockDropCandidateState::Candidate;
            }
        }
    }
    shell_state.set_tab_drag_preview_for_target(target_id, preview, candidates, active_side, epoch);
    let invalid_zones = if source_stack == Some(target_tab_stack_id) && target_stack.tabs.len() == 1
    {
        vec![
            DockZone::Left,
            DockZone::Right,
            DockZone::Top,
            DockZone::Bottom,
        ]
    } else {
        Vec::new()
    };
    let compass = RegionCompassViewModel::active_with_invalid_zones(
        target_id,
        target_region,
        source_unit,
        zone,
        "content",
        "region",
        RegionCompassAccessibility::default(),
        &invalid_zones,
    )
    .with_ordinal(target_stack.tabs.len());
    shell_state.set_region_compass_for_target(target_id, anchor_widget_id, compass, epoch);
}

fn projected_tab_stacks(
    projection: &editor_shell::WorkspaceProjectionArtifact,
) -> Vec<&editor_shell::ProjectedTabStackSlot> {
    let mut stacks = editor_shell::projected_host_tab_stacks(&projection.root_host);
    stacks.extend(projection.floating_hosts.iter().map(|host| &host.tab_stack));
    stacks
}

fn compass_zone(bounds: UiRect, pointer: ui_math::UiPoint) -> DockZone {
    let x = ((pointer.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0);
    let y = ((pointer.y - bounds.y) / bounds.height.max(1.0)).clamp(0.0, 1.0);
    if (0.35..=0.65).contains(&x) && (0.35..=0.65).contains(&y) {
        return DockZone::Center;
    }
    let distances = [
        (x, DockZone::Left),
        (1.0 - x, DockZone::Right),
        (y, DockZone::Top),
        (1.0 - y, DockZone::Bottom),
    ];
    distances
        .into_iter()
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, zone)| zone)
        .unwrap_or(DockZone::Center)
}

fn finish_region_compass_docking(
    shell_state: &mut RunenwerkEditorShellState,
    target_id: PresentationTargetId,
    epoch: u64,
) -> Option<ShellCommand> {
    let intent = shell_state.finish_region_compass_for_target(target_id, epoch)?;
    Some(ShellCommand::CommitCompositionDock {
        intent,
        projection_epoch: epoch,
    })
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
    active_console_binding(shell_state)
        .map(|(mounted_unit_id, _)| {
            app.surface_sessions()
                .session_or_default(mounted_unit_id)
                .console_follow_enabled
        })
        .unwrap_or(true)
}

fn active_console_surface(
    shell_state: &RunenwerkEditorShellState,
) -> Option<editor_shell::ToolSurfaceInstanceId> {
    active_console_binding(shell_state).map(|(_, surface_id)| surface_id)
}

fn active_console_binding(
    shell_state: &RunenwerkEditorShellState,
) -> Option<(
    ui_composition::MountedUnitId,
    editor_shell::ToolSurfaceInstanceId,
)> {
    mounted_content_bindings(shell_state).into_iter().find_map(
        |(mounted_unit_id, surface_id, stable_key)| {
            (stable_key == EDITOR_CONSOLE_SURFACE_KEY).then_some((mounted_unit_id, surface_id))
        },
    )
}

fn mounted_content_bindings(
    shell_state: &RunenwerkEditorShellState,
) -> Vec<(
    ui_composition::MountedUnitId,
    editor_shell::ToolSurfaceInstanceId,
    String,
)> {
    shell_state
        .composition_runtime()
        .extension()
        .mounted_units()
        .iter()
        .filter_map(|record| {
            editor_shell::ToolSurfaceInstanceId::try_from_raw(record.compatibility_surface_raw)
                .ok()
                .map(|surface_id| {
                    (
                        record.mounted_unit_id,
                        surface_id,
                        record.stable_content_key.clone(),
                    )
                })
        })
        .collect()
}

fn is_viewport_content_key(stable_key: &str) -> bool {
    editor_shell::ToolSurfaceStableKey::new(stable_key)
        .ok()
        .and_then(|key| editor_shell::tool_surface_kind_for_stable_key(&key))
        == Some(editor_shell::ToolSurfaceKind::Viewport)
}

fn active_console_scroll_widget(shell_state: &RunenwerkEditorShellState) -> Option<WidgetId> {
    active_console_surface(shell_state)
        .map(|surface_id| surface_widget_id(surface_id, CONSOLE_SCROLL_WIDGET_ID))
}

fn is_at_bottom(offset: f32, max_offset: f32) -> bool {
    max_offset <= CONSOLE_FOLLOW_BOTTOM_EPSILON
        || offset >= (max_offset - CONSOLE_FOLLOW_BOTTOM_EPSILON)
}

#[cfg(test)]
mod region_compass_semantic_tests {
    use super::*;
    use engine::plugins::render::UiFontAtlasResource;

    #[test]
    fn keyboard_touch_and_controller_share_region_compass_direction_semantics() {
        for source in [
            ui_input::SemanticInputSource::Keyboard,
            ui_input::SemanticInputSource::Touch,
            ui_input::SemanticInputSource::Controller,
        ] {
            let event =
                SemanticActionEvent::new(source, UiSemanticAction::Focus(SemanticDirection::Right));
            let UiSemanticAction::Focus(direction) = event.action else {
                unreachable!()
            };
            assert_eq!(
                semantic_direction_to_dock_zone(direction),
                Some(DockZone::Right)
            );
        }
    }

    #[test]
    fn split_resize_previews_transiently_and_commits_one_composition_transaction() {
        let mut app = RunenwerkEditorApp::new();
        let mut shell_state = RunenwerkEditorShellState::new();
        let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);
        let theme = ThemeTokens::default();
        let atlas = UiFontAtlasResource::default();
        let _ = RunenwerkEditorShellController::build_frame(
            &app,
            &mut shell_state,
            bounds,
            &theme,
            &atlas,
        );
        let projection = shell_state.last_projection_artifacts().unwrap().clone();
        let tree = shell_state.last_tree().unwrap().clone();
        let layouts = shell_state.runtime().compute_layout(&tree, bounds);
        let mut splits = Vec::new();
        collect_projected_split_targets(&projection.workspace.root_host, &mut splits);
        let split = splits.first().copied().expect("default composition split");
        let split_bounds = layouts.get(&split.widget).unwrap().bounds;
        let boundary = match split.axis {
            editor_shell::WorkspaceSplitAxis::Horizontal => ui_math::UiPoint::new(
                split_bounds.x + split_bounds.width * split.fraction,
                split_bounds.y + split_bounds.height * 0.5,
            ),
            editor_shell::WorkspaceSplitAxis::Vertical => ui_math::UiPoint::new(
                split_bounds.x + split_bounds.width * 0.5,
                split_bounds.y + split_bounds.height * split.fraction,
            ),
        };
        let moved = match split.axis {
            editor_shell::WorkspaceSplitAxis::Horizontal => {
                ui_math::UiPoint::new(boundary.x + 24.0, boundary.y)
            }
            editor_shell::WorkspaceSplitAxis::Vertical => {
                ui_math::UiPoint::new(boundary.x, boundary.y + 24.0)
            }
        };
        let revision_before = shell_state.composition_runtime().composition().revision();

        for (kind, position) in [
            (PointerEventKind::Down, boundary),
            (PointerEventKind::Move, moved),
        ] {
            RunenwerkEditorShellController::dispatch_input(
                &mut app,
                &mut shell_state,
                bounds,
                &theme,
                &UiInputEvent::Pointer(ui_input::PointerEvent {
                    kind,
                    position,
                    button: (kind == PointerEventKind::Down)
                        .then_some(ui_input::PointerButton::Primary),
                    ..ui_input::PointerEvent::default()
                }),
            )
            .unwrap();
        }
        assert_eq!(
            shell_state.composition_runtime().composition().revision(),
            revision_before,
            "drag-frame preview must remain transient"
        );
        assert!(
            shell_state
                .docking_visual_state()
                .active_split_preview_fraction
                .is_some()
        );

        RunenwerkEditorShellController::dispatch_input(
            &mut app,
            &mut shell_state,
            bounds,
            &theme,
            &UiInputEvent::Pointer(ui_input::PointerEvent {
                kind: PointerEventKind::Up,
                position: moved,
                button: Some(ui_input::PointerButton::Primary),
                ..ui_input::PointerEvent::default()
            }),
        )
        .unwrap();
        assert_eq!(
            shell_state
                .composition_runtime()
                .composition()
                .revision()
                .raw(),
            revision_before.raw() + 1
        );
        assert!(
            shell_state
                .docking_visual_state()
                .active_split_preview_fraction
                .is_none()
        );
    }
}
