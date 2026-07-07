#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum UiRuntimeInstallState {
    #[default]
    Uninstalled,
    Installed,
}

/// Foundation resource for UI runtime plugin installation state.
#[derive(Debug, Clone, PartialEq, Eq, ecs::Resource)]
pub struct UiRuntimeResource {
    install_state: UiRuntimeInstallState,
}

impl Default for UiRuntimeResource {
    fn default() -> Self {
        Self {
            install_state: UiRuntimeInstallState::Uninstalled,
        }
    }
}

impl UiRuntimeResource {
    pub fn install_state(&self) -> UiRuntimeInstallState {
        self.install_state
    }

    pub fn is_installed(&self) -> bool {
        self.install_state == UiRuntimeInstallState::Installed
    }

    pub(crate) fn mark_installed(&mut self) {
        self.install_state = UiRuntimeInstallState::Installed;
    }
}
