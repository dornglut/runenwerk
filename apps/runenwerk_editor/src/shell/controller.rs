use editor_shell::{
    CONSOLE_SCROLL_WIDGET_ID, EditorShellViewModel, ShellCommand, ShellUiExpressionFrame,
    UiInputOutcome, UiTree, build_editor_shell, map_interactions_to_shell_commands,
};
use ui_input::{PointerEventKind, UiInputEvent};
use ui_math::UiRect;
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{
    RunenwerkEditorShellState, build_editor_shell_view_model, dispatch_shell_command,
};

const CONSOLE_FOLLOW_BOTTOM_EPSILON: f32 = 1.0;

pub struct RunenwerkEditorShellController;

impl RunenwerkEditorShellController {
    pub fn rebuild_view_model(app: &RunenwerkEditorApp) -> EditorShellViewModel {
        build_editor_shell_view_model(app)
    }

    pub fn rebuild_tree(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        theme: &ThemeTokens,
    ) -> UiTree {
        let view_model = Self::rebuild_view_model(app);
        let tree = build_editor_shell(&view_model, theme);
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

    pub fn build_expression_frame(
        app: &RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
    ) -> ShellUiExpressionFrame {
        let tree = Self::rebuild_tree(app, shell_state, theme);
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
        let view_model = Self::rebuild_view_model(app);
        let tree = build_editor_shell(&view_model, theme);
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

        let commands = map_interactions_to_shell_commands(&outcome.interactions, &view_model);
        Self::dispatch_commands(app, commands)?;

        Ok(outcome)
    }

    fn dispatch_commands(
        app: &mut RunenwerkEditorApp,
        commands: Vec<ShellCommand>,
    ) -> Result<(), editor_core::EditorMutationError> {
        for command in commands {
            dispatch_shell_command(app, command)?;
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
