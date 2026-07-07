use super::{UiMountRecord, UiMountReport, UiMountRequest, UiMountSource};

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

#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiMountRequestsResource {
    records: Vec<UiMountRecord>,
    reports: Vec<UiMountReport>,
}

impl UiMountRequestsResource {
    pub fn records(&self) -> &[UiMountRecord] {
        &self.records
    }

    pub fn reports(&self) -> &[UiMountReport] {
        &self.reports
    }

    pub fn latest_report(&self) -> Option<&UiMountReport> {
        self.reports.last()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub(crate) fn record_mount_request(
        &mut self,
        request: UiMountRequest,
        mount_source: UiMountSource,
    ) -> UiMountReport {
        if let Some(reason) = request.failure_reason() {
            let report = UiMountReport::rejected(&request, mount_source, reason);
            self.reports.push(report.clone());
            return report;
        }

        let record = UiMountRecord::new(request, mount_source);
        let report = UiMountReport::accepted(&record);
        self.records.push(record);
        self.reports.push(report.clone());
        report
    }
}
