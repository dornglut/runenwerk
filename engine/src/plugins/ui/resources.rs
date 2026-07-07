use ui_surface::{
    MountedSurfaceInstance, MountedSurfaceRegistry, SessionScopeHandle, SurfaceDefinitionId,
    SurfaceHostInstanceId, SurfaceInstanceId,
};

use super::{
    UiMountRecord, UiMountReport, UiMountRequest, UiMountSource, UiMountedSessionRecord,
    UiUnmountReport,
};

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

#[derive(Debug, Clone, PartialEq, Eq, ecs::Resource)]
pub struct UiMountRequestsResource {
    records: Vec<UiMountRecord>,
    reports: Vec<UiMountReport>,
    unmount_reports: Vec<UiUnmountReport>,
    mounted_surfaces: MountedSurfaceRegistry,
    mounted_sessions: Vec<UiMountedSessionRecord>,
    next_surface_instance_id: u64,
    next_definition_id: u64,
    next_host_instance_id: u64,
    next_session_scope_id: u64,
}

impl Default for UiMountRequestsResource {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            reports: Vec::new(),
            unmount_reports: Vec::new(),
            mounted_surfaces: MountedSurfaceRegistry::default(),
            mounted_sessions: Vec::new(),
            next_surface_instance_id: 1,
            next_definition_id: 1,
            next_host_instance_id: 1,
            next_session_scope_id: 1,
        }
    }
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

    pub fn unmount_reports(&self) -> &[UiUnmountReport] {
        &self.unmount_reports
    }

    pub fn latest_unmount_report(&self) -> Option<&UiUnmountReport> {
        self.unmount_reports.last()
    }

    pub fn mounted_sessions(&self) -> &[UiMountedSessionRecord] {
        &self.mounted_sessions
    }

    pub fn mounted_surface(
        &self,
        surface_instance_id: SurfaceInstanceId,
    ) -> Option<MountedSurfaceInstance> {
        self.mounted_surfaces.mounted_surface(surface_instance_id)
    }

    pub fn mounted_surfaces(&self) -> impl Iterator<Item = MountedSurfaceInstance> + '_ {
        self.mounted_surfaces.mounted_surfaces()
    }

    pub fn mounted_generation(&self) -> u64 {
        self.mounted_surfaces.generation()
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
        let mounted_session = self.mount_session_for_record(&record);
        let report = UiMountReport::accepted(&record, &mounted_session);
        self.records.push(record);
        self.reports.push(report.clone());
        report
    }

    pub fn unmount_surface(&mut self, surface_instance_id: SurfaceInstanceId) -> UiUnmountReport {
        let mounted_count_before = self.mounted_sessions.len();
        self.mounted_sessions
            .retain(|session| session.surface_instance_id() != surface_instance_id);
        let removed = self.mounted_sessions.len() != mounted_count_before;

        if removed {
            self.rebuild_mounted_surfaces();
        }

        let report = UiUnmountReport::new(
            surface_instance_id,
            removed,
            self.mounted_surfaces.generation(),
            self.mounted_sessions.len(),
        );
        self.unmount_reports.push(report.clone());
        report
    }

    fn mount_session_for_record(&mut self, record: &UiMountRecord) -> UiMountedSessionRecord {
        let surface_instance_id = self.next_surface_instance_id();
        let mounted_surface = MountedSurfaceInstance::new(
            surface_instance_id,
            self.next_definition_id(),
            self.next_host_instance_id(),
        );
        let session = SessionScopeHandle::new(
            surface_instance_id,
            self.next_session_scope_id(),
            record.retention_class(),
        );
        self.mounted_sessions.push(UiMountedSessionRecord::new(
            record,
            mounted_surface,
            session,
        ));
        self.rebuild_mounted_surfaces();
        self.mounted_sessions
            .iter()
            .find(|session| session.surface_instance_id() == surface_instance_id)
            .expect("newly mounted UI session should be present after registry rebuild")
            .clone()
    }

    fn rebuild_mounted_surfaces(&mut self) {
        self.mounted_surfaces.rebuild(
            self.mounted_sessions
                .iter()
                .map(|session| session.mounted_surface()),
        );
        for session in &mut self.mounted_sessions {
            if let Some(mounted_surface) = self
                .mounted_surfaces
                .mounted_surface(session.surface_instance_id())
            {
                session.replace_mounted_surface(mounted_surface);
            }
        }
    }

    fn next_surface_instance_id(&mut self) -> SurfaceInstanceId {
        let id = SurfaceInstanceId::new(self.next_surface_instance_id);
        self.next_surface_instance_id = self.next_surface_instance_id.saturating_add(1);
        id
    }

    fn next_definition_id(&mut self) -> SurfaceDefinitionId {
        let id = SurfaceDefinitionId::new(self.next_definition_id);
        self.next_definition_id = self.next_definition_id.saturating_add(1);
        id
    }

    fn next_host_instance_id(&mut self) -> SurfaceHostInstanceId {
        let id = SurfaceHostInstanceId::new(self.next_host_instance_id);
        self.next_host_instance_id = self.next_host_instance_id.saturating_add(1);
        id
    }

    fn next_session_scope_id(&mut self) -> u64 {
        let id = self.next_session_scope_id;
        self.next_session_scope_id = self.next_session_scope_id.saturating_add(1);
        id
    }
}
