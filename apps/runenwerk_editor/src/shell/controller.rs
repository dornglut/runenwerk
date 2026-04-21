use editor_shell::{
    CONSOLE_SCROLL_WIDGET_ID, EditorShellViewModel, ShellCommand, ShellUiExpressionFrame,
    UiInputOutcome, UiTree, WorkspaceMutation, build_editor_shell,
    map_interactions_to_shell_commands,
};
use editor_viewport::ArtifactObservationFrame;
use ui_input::{PointerEventKind, UiInputEvent};
use ui_math::UiRect;
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::viewport::{
    ViewportArtifactObservationResource, ViewportPresentationStateResource,
};
use crate::shell::{
    RunenwerkEditorShellState, build_editor_shell_view_model_with_viewport_products,
    dispatch_shell_command,
};

const CONSOLE_FOLLOW_BOTTOM_EPSILON: f32 = 1.0;

pub struct RunenwerkEditorShellController;

impl RunenwerkEditorShellController {
    pub fn rebuild_view_model(app: &RunenwerkEditorApp) -> EditorShellViewModel {
        Self::rebuild_view_model_with_viewport_products(app, None)
    }

    pub fn rebuild_view_model_with_viewport_products(
        app: &RunenwerkEditorApp,
        viewport_products: Option<&ArtifactObservationFrame>,
    ) -> EditorShellViewModel {
        build_editor_shell_view_model_with_viewport_products(app, viewport_products)
    }

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
        viewport_products: Option<&ArtifactObservationFrame>,
    ) -> UiTree {
        let view_model = Self::rebuild_view_model_with_viewport_products(app, viewport_products);
        let build_result = build_editor_shell(&view_model, theme, shell_state.workspace_state());
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
        shell_state.set_last_bounds(bounds);
        if app.console_follow_enabled()
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
        )
    }

    pub fn dispatch_input_with_viewport_products(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        event: &UiInputEvent,
        viewport_products: Option<&ArtifactObservationFrame>,
        mut viewport_presentations: Option<&mut ViewportPresentationStateResource>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
    ) -> Result<UiInputOutcome, editor_core::EditorMutationError> {
        let view_model = Self::rebuild_view_model_with_viewport_products(app, viewport_products);
        let build_result = build_editor_shell(&view_model, theme, shell_state.workspace_state());
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
            if post_at_bottom {
                app.set_console_follow_enabled(true);
            } else if pre_at_bottom {
                app.set_console_follow_enabled(false);
            }
        }

        handle_tab_drag_event(shell_state, event, &outcome, &projection_artifacts)?;

        let commands =
            map_interactions_to_shell_commands(&outcome.interactions, &projection_artifacts);
        Self::dispatch_commands(
            app,
            shell_state,
            commands,
            viewport_presentations.as_deref_mut(),
            viewport_observations,
        )?;

        Ok(outcome)
    }

    fn dispatch_commands(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        commands: Vec<ShellCommand>,
        mut viewport_presentations: Option<&mut ViewportPresentationStateResource>,
        viewport_observations: Option<&ViewportArtifactObservationResource>,
    ) -> Result<(), editor_core::EditorMutationError> {
        for command in commands {
            if let Some(projection_epoch) = command.projection_epoch()
                && !shell_state.is_projection_epoch_current(projection_epoch)
            {
                continue;
            }
            let current_projection_epoch = shell_state.current_projection_epoch();

            dispatch_shell_command(
                app,
                Some(shell_state),
                command,
                viewport_presentations.as_deref_mut(),
                viewport_observations,
                Some(current_projection_epoch),
            )?;
        }

        Ok(())
    }
}

fn handle_tab_drag_event(
    shell_state: &mut RunenwerkEditorShellState,
    event: &UiInputEvent,
    outcome: &UiInputOutcome,
    projection_artifacts: &editor_shell::ShellProjectionArtifacts,
) -> Result<(), editor_core::EditorMutationError> {
    let UiInputEvent::Pointer(pointer) = event else {
        return Ok(());
    };

    match pointer.kind {
        PointerEventKind::Down => {
            let Some(widget_id) = outcome.dispatch.target else {
                return Ok(());
            };
            let Some(tab) = projection_artifacts
                .workspace
                .tab_button_by_widget_id
                .get(&widget_id)
                .copied()
            else {
                return Ok(());
            };
            let Some(tab_stack_id) = projection_artifacts
                .workspace
                .tab_stack_drop_target_by_widget_id
                .get(&widget_id)
                .copied()
            else {
                return Ok(());
            };
            shell_state.begin_tab_drag(tab.panel_instance_id, tab_stack_id);
            shell_state.set_tab_drag_hover_target(Some(tab_stack_id));
        }
        PointerEventKind::Move => {
            if shell_state.tab_drag_state().is_some() {
                let hovered = shell_state.runtime().state().hovered_widget;
                let hovered_stack = hovered.and_then(|widget_id| {
                    projection_artifacts
                        .workspace
                        .tab_stack_drop_target_by_widget_id
                        .get(&widget_id)
                        .copied()
                });
                shell_state.set_tab_drag_hover_target(hovered_stack);
            }
        }
        PointerEventKind::Up => {
            let Some(drag_state) = shell_state.end_tab_drag() else {
                return Ok(());
            };
            let hovered_stack = shell_state
                .runtime()
                .state()
                .hovered_widget
                .and_then(|widget_id| {
                    projection_artifacts
                        .workspace
                        .tab_stack_drop_target_by_widget_id
                        .get(&widget_id)
                        .copied()
                })
                .or(drag_state.hovered_tab_stack_id);
            let Some(destination_tab_stack_id) = hovered_stack else {
                return Ok(());
            };
            if destination_tab_stack_id == drag_state.source_tab_stack_id {
                return Ok(());
            }

            shell_state
                .apply_workspace_mutation(WorkspaceMutation::MovePanelToTabStack {
                    panel_id: drag_state.panel_instance_id,
                    source_tab_stack_id: drag_state.source_tab_stack_id,
                    destination_tab_stack_id,
                    destination_index: None,
                    activate_in_destination: true,
                })
                .map_err(|_| {
                    editor_core::EditorMutationError::runtime_rejected(
                        "failed to move tab between tab stacks",
                    )
                })?;
        }
        PointerEventKind::Leave => {
            if shell_state.tab_drag_state().is_some() {
                shell_state.set_tab_drag_hover_target(None);
            }
        }
        PointerEventKind::Enter | PointerEventKind::Scroll => {}
    }

    Ok(())
}

fn is_console_scroll_event(event: &UiInputEvent, outcome: &UiInputOutcome) -> bool {
    matches!(
        event,
        UiInputEvent::Pointer(pointer) if pointer.kind == PointerEventKind::Scroll
    ) && outcome.dispatch.target == Some(CONSOLE_SCROLL_WIDGET_ID)
}

fn is_at_bottom(offset: f32, max_offset: f32) -> bool {
    max_offset <= CONSOLE_FOLLOW_BOTTOM_EPSILON
        || offset >= (max_offset - CONSOLE_FOLLOW_BOTTOM_EPSILON)
}
