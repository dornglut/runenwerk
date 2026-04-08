use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::RunenwerkEditorShellState;

#[derive(ecs::Component, ecs::Resource)]
pub struct EditorHostResource {
    pub app: RunenwerkEditorApp,
    pub shell_state: RunenwerkEditorShellState,
    pub theme: ThemeTokens,
}

impl Default for EditorHostResource {
    fn default() -> Self {
        Self {
            app: RunenwerkEditorApp::new(),
            shell_state: RunenwerkEditorShellState::new(),
            theme: ThemeTokens::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, ecs::Component, ecs::Resource)]
pub struct EditorInputBridgeState {
    pub last_mouse_position: (f32, f32),
}
