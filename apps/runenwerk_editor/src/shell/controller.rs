use editor_shell::{
    EditorShellViewModel, ShellCommand, UiInputOutcome, UiTree, build_editor_shell,
    map_interactions_to_shell_commands,
};
use ui_input::UiInputEvent;
use ui_math::UiRect;
use ui_render_data::UiFrame;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{
    RunenwerkEditorShellState, build_editor_shell_view_model, dispatch_shell_command,
};

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
    ) -> UiFrame {
        let tree = Self::rebuild_tree(app, shell_state, theme);
        shell_state.set_last_bounds(bounds);
        shell_state.runtime().build_frame(&tree, bounds)
    }

    pub fn dispatch_input(
        app: &mut RunenwerkEditorApp,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        event: &UiInputEvent,
    ) -> Result<UiInputOutcome, &'static str> {
        let view_model = Self::rebuild_view_model(app);
        let tree = build_editor_shell(&view_model, theme);
        shell_state.set_last_tree(tree.clone());
        shell_state.set_last_bounds(bounds);

        let layouts = shell_state.runtime().compute_layout(&tree, bounds);
        let outcome = shell_state
            .runtime_mut()
            .dispatch_input(&tree, &layouts, event);

        let commands = map_interactions_to_shell_commands(&outcome.interactions, &view_model);
        Self::dispatch_commands(app, commands)?;

        Ok(outcome)
    }

    fn dispatch_commands(
        app: &mut RunenwerkEditorApp,
        commands: Vec<ShellCommand>,
    ) -> Result<(), &'static str> {
        for command in commands {
            dispatch_shell_command(app, command)?;
        }

        Ok(())
    }
}
