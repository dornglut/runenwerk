use editor_shell::{
    CONSOLE_SCROLL_WIDGET_ID, ComputedLayoutMap, DockingPreviewDropTarget, EditorDomainMutation,
    EditorShellFrameModel, PanelHostId, ProjectedWorkspaceHostSlot, ShellCommand,
    ShellUiExpressionFrame, SurfaceCommandProposal, SurfaceSessionMutation, TabDropDestination,
    UiInputOutcome, UiInteractionResults, UiTree, WidgetId, WorkspaceSplitAxis,
    build_editor_shell_frame_with_docking_visual_state, map_interactions_to_shell_commands,
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
    EditorSurfaceProviderRegistry, RunenwerkEditorShellState, SurfaceProviderDispatchContext,
    active_document_context, build_editor_shell_frame_model, dispatch_shell_command,
    mounted_surface_requests,
};

const CONSOLE_FOLLOW_BOTTOM_EPSILON: f32 = 1.0;
const SPLIT_HIT_SLOP_PX: f32 = 8.0;
const SPLIT_MIN_FRACTION: f32 = 0.08;
const SPLIT_MAX_FRACTION: f32 = 0.92;

pub struct RunenwerkEditorShellController;

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

        if let Some(outcome) =
            Self::handle_split_resize_event(shell_state, event, &layouts, &projection_artifacts)
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

        if tab_button_route_at_pointer(projection_artifacts, layouts, pointer.position).is_some() {
            return None;
        }

        let candidate =
            resolve_split_resize_target(pointer.position, layouts, projection_artifacts)?;
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

fn tab_button_route_at_pointer(
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
    layouts: &ComputedLayoutMap,
    pointer_position: ui_math::UiPoint,
) -> Option<editor_shell::ProjectedTabButtonRoute> {
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
                (distance, *route)
            })
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, route)| route);
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
            (distance <= 48.0).then_some((distance, *route))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, route)| route)
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
