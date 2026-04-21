use editor_shell::{
    CONSOLE_SCROLL_WIDGET_ID, EditorShellViewModel, ShellCommand, ShellUiExpressionFrame,
    UiInputOutcome, UiTree, build_editor_shell, map_interactions_to_shell_commands,
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

        let commands = map_interactions_to_shell_commands(
            &outcome.interactions,
            &projection_artifacts,
        );
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
        shell_state: &RunenwerkEditorShellState,
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

            dispatch_shell_command(
                app,
                command,
                viewport_presentations.as_deref_mut(),
                viewport_observations,
                Some(shell_state.current_projection_epoch()),
            )?;
        }

        Ok(())
    }
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
