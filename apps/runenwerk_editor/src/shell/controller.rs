use editor_shell::{
    BODY_CONSOLE_SPLIT_WIDGET_ID, CENTER_RIGHT_SPLIT_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID,
    ComputedLayoutMap, DockingPreviewDropTarget, EditorDomainMutation, EditorShellFrameModel,
    LEFT_RIGHT_SPLIT_WIDGET_ID, ShellCommand, ShellUiExpressionFrame, SurfaceCommandProposal,
    SurfaceSessionMutation, TabDropDestination, UiInputOutcome, UiInteractionResults, UiTree,
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
    WorkspaceSplitKind, active_document_context, build_editor_shell_frame_model,
    dispatch_shell_command, mounted_surface_requests,
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
                let Some(pressed_widget) = pressed else {
                    shell_state.clear_tab_drag();
                    return (commands, suppress_tab_activation);
                };
                if let Some(route) = projection_artifacts
                    .workspace
                    .tab_button_route_by_widget_id
                    .get(&pressed_widget)
                {
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
                    app,
                    shell_state,
                    projection_epoch: current_epoch,
                };
                if let Some(command) = shell_command_from_surface_proposal(registry.map_action(
                    &dispatch_context,
                    &request,
                    provider_id,
                    action,
                )?) {
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
            SurfaceSessionMutation::ConsoleSetFollow { .. } => None,
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
        SurfaceCommandProposal::NoOp => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitResizeTarget {
    BodyConsole,
    LeftRight,
    CenterRight,
}

impl SplitResizeTarget {
    fn widget_id(self) -> editor_shell::WidgetId {
        match self {
            Self::BodyConsole => BODY_CONSOLE_SPLIT_WIDGET_ID,
            Self::LeftRight => LEFT_RIGHT_SPLIT_WIDGET_ID,
            Self::CenterRight => CENTER_RIGHT_SPLIT_WIDGET_ID,
        }
    }

    fn axis(self) -> ui_math::Axis {
        match self {
            Self::BodyConsole => ui_math::Axis::Vertical,
            Self::LeftRight | Self::CenterRight => ui_math::Axis::Horizontal,
        }
    }

    fn projection_fraction(self, projection: &editor_shell::FixedLayoutProjection) -> f32 {
        match self {
            Self::BodyConsole => projection.body_console_fraction,
            Self::LeftRight => projection.left_right_fraction,
            Self::CenterRight => projection.center_right_fraction,
        }
    }

    fn workspace_kind(self) -> WorkspaceSplitKind {
        match self {
            Self::BodyConsole => WorkspaceSplitKind::BodyConsole,
            Self::LeftRight => WorkspaceSplitKind::LeftRight,
            Self::CenterRight => WorkspaceSplitKind::CenterRight,
        }
    }

    fn from_workspace_kind(value: WorkspaceSplitKind) -> Self {
        match value {
            WorkspaceSplitKind::BodyConsole => Self::BodyConsole,
            WorkspaceSplitKind::LeftRight => Self::LeftRight,
            WorkspaceSplitKind::CenterRight => Self::CenterRight,
        }
    }
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

        if let Some(active_kind) = shell_state.active_split_resize_kind() {
            let target = SplitResizeTarget::from_workspace_kind(active_kind);
            match pointer.kind {
                PointerEventKind::Move | PointerEventKind::Enter => {
                    let split_layout = layouts.get(&target.widget_id())?;
                    let next_fraction =
                        split_fraction_from_pointer(target, split_layout.bounds, pointer)
                            .clamp(SPLIT_MIN_FRACTION, SPLIT_MAX_FRACTION);
                    let current_fraction =
                        target.projection_fraction(&projection_artifacts.workspace.fixed_layout);
                    let changed = (next_fraction - current_fraction).abs() > 0.001;
                    if changed {
                        let _ = shell_state
                            .set_workspace_split_fraction(target.workspace_kind(), next_fraction);
                    }
                    return Some(consumed_pointer_outcome(Some(target.widget_id()), changed));
                }
                PointerEventKind::Up => {
                    shell_state.clear_split_resize();
                    return Some(consumed_pointer_outcome(Some(target.widget_id()), false));
                }
                PointerEventKind::Down | PointerEventKind::Leave | PointerEventKind::Scroll => {
                    return Some(consumed_pointer_outcome(Some(target.widget_id()), false));
                }
            }
        }

        if !matches!(pointer.kind, PointerEventKind::Down)
            || pointer.button != Some(ui_input::PointerButton::Primary)
        {
            return None;
        }

        let candidate =
            resolve_split_resize_target(pointer.position, layouts, projection_artifacts)?;
        shell_state.begin_split_resize(candidate.workspace_kind());
        Some(consumed_pointer_outcome(Some(candidate.widget_id()), false))
    }
}

fn resolve_split_resize_target(
    cursor: ui_math::UiPoint,
    layouts: &ComputedLayoutMap,
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
) -> Option<SplitResizeTarget> {
    [
        SplitResizeTarget::BodyConsole,
        SplitResizeTarget::LeftRight,
        SplitResizeTarget::CenterRight,
    ]
    .into_iter()
    .filter_map(|target| {
        let layout = layouts.get(&target.widget_id())?;
        let bounds = layout.bounds;
        if !bounds.contains(cursor) {
            return None;
        }
        let boundary = split_boundary_position(
            target,
            bounds,
            target.projection_fraction(&projection_artifacts.workspace.fixed_layout),
        );
        let distance = match target.axis() {
            ui_math::Axis::Horizontal => (cursor.x - boundary).abs(),
            ui_math::Axis::Vertical => (cursor.y - boundary).abs(),
        };
        (distance <= SPLIT_HIT_SLOP_PX).then_some((distance, target))
    })
    .min_by(|(left, _), (right, _)| left.total_cmp(right))
    .map(|(_, target)| target)
}

fn split_boundary_position(
    target: SplitResizeTarget,
    bounds: ui_math::UiRect,
    fraction: f32,
) -> f32 {
    match target.axis() {
        ui_math::Axis::Horizontal => bounds.x + bounds.width * fraction,
        ui_math::Axis::Vertical => bounds.y + bounds.height * fraction,
    }
}

fn split_fraction_from_pointer(
    target: SplitResizeTarget,
    bounds: ui_math::UiRect,
    pointer: &PointerEvent,
) -> f32 {
    match target.axis() {
        ui_math::Axis::Horizontal => (pointer.position.x - bounds.x) / bounds.width.max(1.0),
        ui_math::Axis::Vertical => (pointer.position.y - bounds.y) / bounds.height.max(1.0),
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
        .map(map_projected_tab_drop_target)
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
