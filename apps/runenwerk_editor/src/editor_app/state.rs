use crate::editor_runtime::runtime::RunenwerkEditorRuntime;
use crate::editor_runtime::tool_state::EditorToolRuntimeState;
use std::sync::Arc;

use editor_definition::EditorDefinitionDocument;
use editor_preview::{PreviewMode, PreviewSessionId};
use editor_shell::WorkspaceState;

use super::console::{ConsoleMessage, ConsoleMessageKind};
use crate::asset_pipeline::{
    AssetCatalogRuntime, EditorAssetProjectSession, EditorFieldProductPublication,
    EditorFieldProductPublicationJournalEntry, ImportJobStatus, catalog_with_import_artifact,
    execute_import_for_asset, load_project_catalog, save_project_catalog,
};
use crate::material_lab::{
    EditorMaterialPreviewPublication, EditorMaterialPreviewPublicationJournalEntry,
    MaterialLabRuntime,
};
use crate::runtime::procgen::ProcgenRuntimeState;
use crate::runtime::viewport::{
    EditorViewportGpuResidencyJournalEntry, EditorViewportGpuResidencySummary,
    EditorViewportQuerySnapshotJournalEntry, EditorViewportQuerySnapshotSummary,
    EditorViewportRenderSelectionJournalEntry, EditorViewportRenderSelectionSummary,
};
use crate::shell::{
    EditorDefinitionActivationReport, EditorDefinitionActivationStatus,
    EditorSurfaceProviderRegistry, PendingEditorDefinitionActivation, RunenwerkWorkbenchHost,
    RunenwerkWorkbenchHostError, SurfaceSessionStore,
};
use crate::texture_preview::TexturePreviewRuntime;

use super::sdf_operations::SdfOperationWorkspaceState;

pub struct RunenwerkEditorApp {
    pub(crate) runtime: RunenwerkEditorRuntime,
    pub(crate) runtime_mode_sessions: RuntimeModeSessions,
    pub(crate) tool_runtime_state: EditorToolRuntimeState,
    pub(crate) console_lines: Vec<ConsoleMessage>,
    pub(crate) console_max_lines: usize,
    pub(crate) debug_logs_enabled: bool,
    pub(crate) surface_sessions: SurfaceSessionStore,
    pub(crate) workbench_host: Arc<RunenwerkWorkbenchHost>,
    pub(crate) pending_editor_definition_activations: Vec<PendingEditorDefinitionActivation>,
    pub(crate) editor_definition_activation_reports: Vec<EditorDefinitionActivationReport>,
    pub(crate) failed_editor_definition_activations: Vec<PendingEditorDefinitionActivation>,
    pub(crate) asset_catalog_runtime: AssetCatalogRuntime,
    pub(crate) asset_project_session: Option<EditorAssetProjectSession>,
    pub(crate) material_lab_runtime: MaterialLabRuntime,
    pub(crate) texture_preview_runtime: TexturePreviewRuntime,
    pub(crate) sdf_operation_workspace: SdfOperationWorkspaceState,
    pub(crate) pending_field_product_publications: Vec<EditorFieldProductPublication>,
    pub(crate) field_product_publication_journal: Vec<EditorFieldProductPublicationJournalEntry>,
    pub(crate) pending_material_preview_publications: Vec<EditorMaterialPreviewPublication>,
    pub(crate) material_preview_publication_journal:
        Vec<EditorMaterialPreviewPublicationJournalEntry>,
    pub(crate) viewport_query_snapshot_journal: Vec<EditorViewportQuerySnapshotJournalEntry>,
    pub(crate) last_viewport_query_snapshot_summary: Option<EditorViewportQuerySnapshotSummary>,
    pub(crate) viewport_render_selection_journal: Vec<EditorViewportRenderSelectionJournalEntry>,
    pub(crate) last_viewport_render_selection_summary: Option<EditorViewportRenderSelectionSummary>,
    pub(crate) viewport_gpu_residency_journal: Vec<EditorViewportGpuResidencyJournalEntry>,
    pub(crate) last_viewport_gpu_residency_summary: Option<EditorViewportGpuResidencySummary>,
    pub(crate) procgen_runtime: ProcgenRuntimeState,
}

impl Default for RunenwerkEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RunenwerkEditorApp {
    pub fn new() -> Self {
        let workbench_host =
            RunenwerkWorkbenchHost::new().expect("default workbench host composition must build");
        Self::with_workbench_host(workbench_host)
    }

    pub fn try_new() -> Result<Self, RunenwerkWorkbenchHostError> {
        RunenwerkWorkbenchHost::new().map(Self::with_workbench_host)
    }

    pub fn new_material_lab_workbench() -> Self {
        let workbench_host = RunenwerkWorkbenchHost::material_lab()
            .expect("Material Lab workbench host composition must build");
        Self::with_workbench_host(workbench_host)
    }

    pub fn try_new_material_lab_workbench() -> Result<Self, RunenwerkWorkbenchHostError> {
        RunenwerkWorkbenchHost::material_lab().map(Self::with_workbench_host)
    }

    fn with_workbench_host(workbench_host: RunenwerkWorkbenchHost) -> Self {
        Self {
            runtime: RunenwerkEditorRuntime::new(),
            runtime_mode_sessions: RuntimeModeSessions::default(),
            tool_runtime_state: EditorToolRuntimeState::new(),
            console_lines: Vec::new(),
            console_max_lines: 256,
            debug_logs_enabled: true,
            surface_sessions: SurfaceSessionStore::default(),
            workbench_host: Arc::new(workbench_host),
            pending_editor_definition_activations: Vec::new(),
            editor_definition_activation_reports: Vec::new(),
            failed_editor_definition_activations: Vec::new(),
            asset_catalog_runtime: AssetCatalogRuntime::new(),
            asset_project_session: None,
            material_lab_runtime: MaterialLabRuntime::default(),
            texture_preview_runtime: TexturePreviewRuntime::default(),
            sdf_operation_workspace: SdfOperationWorkspaceState::default(),
            pending_field_product_publications: Vec::new(),
            field_product_publication_journal: Vec::new(),
            pending_material_preview_publications: Vec::new(),
            material_preview_publication_journal: Vec::new(),
            viewport_query_snapshot_journal: Vec::new(),
            last_viewport_query_snapshot_summary: None,
            viewport_render_selection_journal: Vec::new(),
            last_viewport_render_selection_summary: None,
            viewport_gpu_residency_journal: Vec::new(),
            last_viewport_gpu_residency_summary: None,
            procgen_runtime: ProcgenRuntimeState::new(),
        }
    }

    pub fn with_surface_provider_registry(
        surface_provider_registry: EditorSurfaceProviderRegistry,
    ) -> Self {
        let workbench_host =
            RunenwerkWorkbenchHost::from_tool_suites_provider_registry_and_provider_family_assignments(
            vec![crate::material_lab::material_lab_tool_suite()],
            surface_provider_registry,
            Vec::new(),
        )
        .expect("custom workbench host composition must build");
        Self::with_workbench_host(workbench_host)
    }

    pub fn runtime(&self) -> &RunenwerkEditorRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut RunenwerkEditorRuntime {
        &mut self.runtime
    }

    pub fn runtime_mode_sessions(&self) -> &RuntimeModeSessions {
        &self.runtime_mode_sessions
    }

    pub fn runtime_mode_sessions_mut(&mut self) -> &mut RuntimeModeSessions {
        &mut self.runtime_mode_sessions
    }

    pub fn reset_transient_editor_ui_state(&mut self) {
        self.tool_runtime_state = EditorToolRuntimeState::new();
        self.surface_sessions.clear_transient();
    }

    pub fn tool_runtime_state(&self) -> &EditorToolRuntimeState {
        &self.tool_runtime_state
    }

    pub fn tool_runtime_state_mut(&mut self) -> &mut EditorToolRuntimeState {
        &mut self.tool_runtime_state
    }

    pub fn console_lines(&self) -> &[ConsoleMessage] {
        &self.console_lines
    }

    pub fn append_console_line(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Info, line);
    }

    pub fn append_console_input(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Input, line);
    }

    pub fn append_console_warning(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Warning, line);
    }

    pub fn append_console_error(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Error, line);
    }

    pub fn append_console_debug(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Debug, line);
    }

    pub fn append_console_message(&mut self, kind: ConsoleMessageKind, line: impl Into<String>) {
        self.console_lines.push(ConsoleMessage::new(kind, line));
        if self.console_lines.len() > self.console_max_lines {
            let drain = self.console_lines.len() - self.console_max_lines;
            self.console_lines.drain(0..drain);
        }
    }

    pub fn debug_logs_enabled(&self) -> bool {
        self.debug_logs_enabled
    }

    pub fn set_debug_logs_enabled(&mut self, enabled: bool) {
        self.debug_logs_enabled = enabled;
    }

    pub fn toggle_debug_logs_enabled(&mut self) {
        self.debug_logs_enabled = !self.debug_logs_enabled;
    }

    pub fn surface_sessions(&self) -> &SurfaceSessionStore {
        &self.surface_sessions
    }

    pub fn surface_sessions_mut(&mut self) -> &mut SurfaceSessionStore {
        &mut self.surface_sessions
    }

    pub fn prune_surface_sessions_for_workspace(&mut self, workspace: &WorkspaceState) {
        self.surface_sessions.prune_for_workspace(workspace);
    }

    pub fn surface_provider_registry(&self) -> &EditorSurfaceProviderRegistry {
        self.workbench_host.provider_registry()
    }

    pub fn surface_provider_registry_handle(&self) -> Arc<EditorSurfaceProviderRegistry> {
        self.workbench_host.provider_registry_handle()
    }

    pub fn workbench_host(&self) -> &RunenwerkWorkbenchHost {
        self.workbench_host.as_ref()
    }

    pub fn queue_editor_definition_activation(&mut self, document: EditorDefinitionDocument) {
        self.queue_editor_definition_activation_for_review(None, document);
    }

    pub fn queue_editor_definition_activation_for_review(
        &mut self,
        review_id: Option<String>,
        document: EditorDefinitionDocument,
    ) {
        let request = PendingEditorDefinitionActivation::new(review_id, document);
        self.record_editor_definition_activation_report(
            EditorDefinitionActivationReport::from_request(
                &request,
                EditorDefinitionActivationStatus::Queued,
                vec!["queued editor definition activation".to_string()],
                Vec::new(),
                true,
            ),
        );
        self.pending_editor_definition_activations.push(request);
    }

    pub fn take_pending_editor_definition_activations(
        &mut self,
    ) -> Vec<PendingEditorDefinitionActivation> {
        std::mem::take(&mut self.pending_editor_definition_activations)
    }

    pub fn pending_editor_definition_activation_count(&self) -> usize {
        self.pending_editor_definition_activations.len()
    }

    pub fn record_editor_definition_activation_report(
        &mut self,
        report: EditorDefinitionActivationReport,
    ) {
        self.editor_definition_activation_reports.push(report);
    }

    pub fn preserve_failed_editor_definition_activation(
        &mut self,
        request: PendingEditorDefinitionActivation,
    ) {
        self.failed_editor_definition_activations.push(request);
    }

    pub fn editor_definition_activation_reports(&self) -> &[EditorDefinitionActivationReport] {
        &self.editor_definition_activation_reports
    }

    pub fn last_editor_definition_activation_report(
        &self,
    ) -> Option<&EditorDefinitionActivationReport> {
        self.editor_definition_activation_reports.last()
    }

    pub fn failed_editor_definition_activations(&self) -> &[PendingEditorDefinitionActivation] {
        &self.failed_editor_definition_activations
    }

    pub fn asset_catalog_runtime(&self) -> &AssetCatalogRuntime {
        &self.asset_catalog_runtime
    }

    pub fn asset_catalog_runtime_mut(&mut self) -> &mut AssetCatalogRuntime {
        &mut self.asset_catalog_runtime
    }

    pub fn asset_project_session(&self) -> Option<&EditorAssetProjectSession> {
        self.asset_project_session.as_ref()
    }

    pub fn asset_project_session_mut(&mut self) -> Option<&mut EditorAssetProjectSession> {
        self.asset_project_session.as_mut()
    }

    pub fn set_asset_project_session(&mut self, session: EditorAssetProjectSession) {
        self.asset_project_session = Some(session);
    }

    pub fn material_lab_runtime(&self) -> &MaterialLabRuntime {
        &self.material_lab_runtime
    }

    pub fn material_lab_runtime_mut(&mut self) -> &mut MaterialLabRuntime {
        &mut self.material_lab_runtime
    }

    pub fn asset_catalog_status_lines(&self) -> Vec<String> {
        self.asset_project_session
            .as_ref()
            .map(EditorAssetProjectSession::status_lines)
            .unwrap_or_else(|| {
                vec![
                    "No asset project session".to_string(),
                    "Load a project file before catalog IO or import execution".to_string(),
                ]
            })
    }

    pub fn load_asset_project_catalog(&mut self) -> anyhow::Result<()> {
        let Some(session) = self.asset_project_session.as_mut() else {
            self.record_missing_asset_project_session("load catalog");
            return Ok(());
        };
        let outcome = load_project_catalog(session)?;
        if let Some(catalog) = outcome.accepted_catalog {
            self.asset_catalog_runtime.replace_catalog(catalog);
            if let Some(session) = self.asset_project_session.as_mut() {
                session.set_catalog_load_status("accepted");
            }
        } else {
            for diagnostic in outcome.diagnostics {
                self.asset_catalog_runtime.record_diagnostic(diagnostic);
            }
            if let Some(session) = self.asset_project_session.as_mut() {
                session.set_catalog_load_status("rejected; previous catalog preserved");
            }
        }
        Ok(())
    }

    pub fn save_asset_project_catalog(&mut self) -> anyhow::Result<()> {
        let Some(session) = self.asset_project_session.as_ref() else {
            self.record_missing_asset_project_session("save catalog");
            return Ok(());
        };
        let diagnostics = save_project_catalog(session, self.asset_catalog_runtime.catalog())?;
        if diagnostics.is_empty() {
            if let Some(session) = self.asset_project_session.as_mut() {
                session.set_catalog_save_status("written");
            }
        } else {
            for diagnostic in diagnostics {
                self.asset_catalog_runtime.record_diagnostic(diagnostic);
            }
            if let Some(session) = self.asset_project_session.as_mut() {
                session.set_catalog_save_status("rejected; invalid catalog not written");
            }
        }
        Ok(())
    }

    pub fn reimport_selected_asset(&mut self) -> anyhow::Result<()> {
        let Some(asset_id) = self.asset_catalog_runtime.selected_asset_id() else {
            self.asset_catalog_runtime
                .record_diagnostic(asset::AssetDiagnosticRecord::error(
                    asset::AssetDiagnosticCode::RatificationRejected,
                    "no selected asset to reimport",
                ));
            return Ok(());
        };
        self.reimport_asset(asset_id)
    }

    pub fn reimport_asset(&mut self, asset_id: asset::AssetId) -> anyhow::Result<()> {
        if self.asset_project_session.is_none() {
            self.record_missing_asset_project_session("reimport asset");
            return Ok(());
        }
        let catalog_snapshot = self.asset_catalog_runtime.catalog().clone();
        let outcome = {
            let session = self
                .asset_project_session
                .as_mut()
                .expect("asset project session checked above");
            execute_import_for_asset(&catalog_snapshot, session, asset_id)
        };
        for diagnostic in &outcome.diagnostics {
            self.asset_catalog_runtime
                .record_diagnostic(diagnostic.clone());
        }
        if let Some(artifact) = outcome.artifact.clone() {
            match catalog_with_import_artifact(&catalog_snapshot, artifact.clone()) {
                Ok(catalog) => {
                    self.asset_catalog_runtime
                        .publish_catalog_update(catalog, Some(asset_id));
                    let status = self
                        .asset_catalog_runtime
                        .classify_artifact_reload(&artifact);
                    self.asset_catalog_runtime.record_reload_status(status);
                }
                Err(diagnostics) => {
                    for diagnostic in diagnostics {
                        self.asset_catalog_runtime.record_diagnostic(diagnostic);
                    }
                }
            }
        }
        if let Some(session) = self.asset_project_session.as_mut() {
            session.set_import_status(match outcome.status {
                ImportJobStatus::Imported => "imported",
                ImportJobStatus::Failed => "failed",
                ImportJobStatus::FailedPreserved => "failed; prior valid artifact preserved",
            });
        }
        Ok(())
    }

    pub(crate) fn record_missing_asset_project_session(&mut self, action: &'static str) {
        self.asset_catalog_runtime
            .record_diagnostic(asset::AssetDiagnosticRecord::error(
                asset::AssetDiagnosticCode::RatificationRejected,
                format!("cannot {action}: no asset project session is active"),
            ));
    }

    pub fn sdf_operation_workspace(&self) -> &SdfOperationWorkspaceState {
        &self.sdf_operation_workspace
    }

    pub fn sdf_operation_workspace_mut(&mut self) -> &mut SdfOperationWorkspaceState {
        &mut self.sdf_operation_workspace
    }

    pub fn queue_field_product_publication(&mut self, publication: EditorFieldProductPublication) {
        self.pending_field_product_publications.push(publication);
    }

    pub fn take_pending_field_product_publications(
        &mut self,
    ) -> Vec<EditorFieldProductPublication> {
        std::mem::take(&mut self.pending_field_product_publications)
    }

    pub fn pending_field_product_publication_count(&self) -> usize {
        self.pending_field_product_publications.len()
    }

    pub fn queue_material_preview_publication(
        &mut self,
        publication: EditorMaterialPreviewPublication,
    ) {
        self.pending_material_preview_publications.push(publication);
    }

    pub fn take_pending_material_preview_publications(
        &mut self,
    ) -> Vec<EditorMaterialPreviewPublication> {
        std::mem::take(&mut self.pending_material_preview_publications)
    }

    pub fn pending_material_preview_publication_count(&self) -> usize {
        self.pending_material_preview_publications.len()
    }

    pub fn material_preview_publication_journal(
        &self,
    ) -> &[EditorMaterialPreviewPublicationJournalEntry] {
        &self.material_preview_publication_journal
    }

    pub fn record_material_preview_publication(
        &mut self,
        entry: EditorMaterialPreviewPublicationJournalEntry,
    ) {
        self.material_preview_publication_journal.push(entry);
    }

    pub fn field_product_publication_journal(
        &self,
    ) -> &[EditorFieldProductPublicationJournalEntry] {
        &self.field_product_publication_journal
    }

    pub fn record_field_product_publication(
        &mut self,
        entry: EditorFieldProductPublicationJournalEntry,
    ) {
        self.field_product_publication_journal.push(entry);
    }

    pub fn viewport_query_snapshot_journal(&self) -> &[EditorViewportQuerySnapshotJournalEntry] {
        &self.viewport_query_snapshot_journal
    }

    pub fn record_viewport_query_snapshot(
        &mut self,
        entry: EditorViewportQuerySnapshotJournalEntry,
    ) {
        self.viewport_query_snapshot_journal.push(entry);
        const VIEWPORT_QUERY_SNAPSHOT_JOURNAL_LIMIT: usize = 128;
        if self.viewport_query_snapshot_journal.len() > VIEWPORT_QUERY_SNAPSHOT_JOURNAL_LIMIT {
            let drain =
                self.viewport_query_snapshot_journal.len() - VIEWPORT_QUERY_SNAPSHOT_JOURNAL_LIMIT;
            self.viewport_query_snapshot_journal.drain(0..drain);
        }
    }

    pub fn update_viewport_query_snapshot_summary(
        &mut self,
        summary: EditorViewportQuerySnapshotSummary,
    ) -> bool {
        if self.last_viewport_query_snapshot_summary.as_ref() == Some(&summary) {
            return false;
        }
        self.last_viewport_query_snapshot_summary = Some(summary);
        true
    }

    pub fn viewport_render_selection_journal(
        &self,
    ) -> &[EditorViewportRenderSelectionJournalEntry] {
        &self.viewport_render_selection_journal
    }

    pub fn record_viewport_render_selection(
        &mut self,
        entry: EditorViewportRenderSelectionJournalEntry,
    ) {
        self.viewport_render_selection_journal.push(entry);
    }

    pub fn update_viewport_render_selection_summary(
        &mut self,
        summary: EditorViewportRenderSelectionSummary,
    ) -> bool {
        if self.last_viewport_render_selection_summary == Some(summary) {
            return false;
        }
        self.last_viewport_render_selection_summary = Some(summary);
        true
    }

    pub fn viewport_gpu_residency_journal(&self) -> &[EditorViewportGpuResidencyJournalEntry] {
        &self.viewport_gpu_residency_journal
    }

    pub fn record_viewport_gpu_residency(&mut self, entry: EditorViewportGpuResidencyJournalEntry) {
        self.viewport_gpu_residency_journal.push(entry);
    }

    pub fn update_viewport_gpu_residency_summary(
        &mut self,
        summary: EditorViewportGpuResidencySummary,
    ) -> bool {
        if self.last_viewport_gpu_residency_summary == Some(summary) {
            return false;
        }
        self.last_viewport_gpu_residency_summary = Some(summary);
        true
    }

    pub fn procgen_runtime(&self) -> &ProcgenRuntimeState {
        &self.procgen_runtime
    }

    pub fn procgen_runtime_mut(&mut self) -> &mut ProcgenRuntimeState {
        &mut self.procgen_runtime
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeModeSessions {
    pub edit: EditSessionState,
    pub preview: ExternalRuntimeSessionState,
    pub simulate: ExternalRuntimeSessionState,
    pub play: ExternalRuntimeSessionState,
}

impl Default for RuntimeModeSessions {
    fn default() -> Self {
        Self {
            edit: EditSessionState { active: true },
            preview: ExternalRuntimeSessionState::new(PreviewMode::Preview),
            simulate: ExternalRuntimeSessionState::new(PreviewMode::Simulate),
            play: ExternalRuntimeSessionState::new(PreviewMode::Play),
        }
    }
}

impl RuntimeModeSessions {
    pub fn begin_external_session(&mut self, mode: PreviewMode, session_id: PreviewSessionId) {
        self.external_session_mut(mode).session_id = Some(session_id);
    }

    pub fn end_external_session(&mut self, mode: PreviewMode) {
        self.external_session_mut(mode).session_id = None;
    }

    pub fn external_session(&self, mode: PreviewMode) -> &ExternalRuntimeSessionState {
        match mode {
            PreviewMode::Preview => &self.preview,
            PreviewMode::Simulate => &self.simulate,
            PreviewMode::Play => &self.play,
        }
    }

    pub fn external_session_mut(&mut self, mode: PreviewMode) -> &mut ExternalRuntimeSessionState {
        match mode {
            PreviewMode::Preview => &mut self.preview,
            PreviewMode::Simulate => &mut self.simulate,
            PreviewMode::Play => &mut self.play,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditSessionState {
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRuntimeSessionState {
    pub mode: PreviewMode,
    pub session_id: Option<PreviewSessionId>,
}

impl ExternalRuntimeSessionState {
    pub const fn new(mode: PreviewMode) -> Self {
        Self {
            mode,
            session_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_preview::preview_session_id;

    #[test]
    fn runtime_mode_sessions_keep_preview_simulate_and_play_separate() {
        let mut sessions = RuntimeModeSessions::default();
        sessions.begin_external_session(PreviewMode::Preview, preview_session_id(1));
        sessions.begin_external_session(PreviewMode::Play, preview_session_id(2));

        assert_eq!(
            sessions.external_session(PreviewMode::Preview).session_id,
            Some(preview_session_id(1))
        );
        assert_eq!(
            sessions.external_session(PreviewMode::Simulate).session_id,
            None
        );
        assert_eq!(
            sessions.external_session(PreviewMode::Play).session_id,
            Some(preview_session_id(2))
        );
    }
}
