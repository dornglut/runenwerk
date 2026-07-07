use super::UiRuntimeInstallState;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UiRuntimeReport {
    pub install_state: UiRuntimeInstallState,
    pub diagnostic_count: usize,
}

impl Default for UiRuntimeReport {
    fn default() -> Self {
        Self {
            install_state: UiRuntimeInstallState::Uninstalled,
            diagnostic_count: 0,
        }
    }
}

/// Latest lightweight UI runtime status report.
#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiRuntimeReportResource {
    latest: UiRuntimeReport,
}

impl UiRuntimeReportResource {
    pub fn latest(&self) -> UiRuntimeReport {
        self.latest
    }

    pub(crate) fn record_plugin_installed(&mut self, diagnostic_count: usize) {
        self.latest = UiRuntimeReport {
            install_state: UiRuntimeInstallState::Installed,
            diagnostic_count,
        };
    }
}
