//! File: apps/runenwerk_editor/src/shell/providers/tool_suite_registry_inspector.rs
//! Purpose: Read-only Tool Suite Registry Inspector provider.

use std::collections::BTreeMap;

use editor_shell::{
    EditorToolSuite, PanelInstanceId, PanelKind, SurfaceDocumentContext,
    SurfaceProviderAvailability, SurfaceProviderId, SurfaceProviderSupportMode, TabStackId,
    ToolSuiteRegistry, ToolSurfaceDefinition, ToolSurfaceInstanceId, ToolSurfaceKind,
    ToolSurfaceRoute, WorkspaceProfileId,
};

use crate::shell::tool_suites::TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY;

use super::*;

pub(super) struct ToolSuiteRegistryInspectorProvider;

impl EditorSurfaceProvider for ToolSuiteRegistryInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID,
            "Tool Suite Registry Inspector",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        if request.matches_stable_key(TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY) {
            SurfaceProviderSupportMode::StableKey
        } else {
            SurfaceProviderSupportMode::Unsupported
        }
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let host = context.app.workbench_host();
        let view_model = build_tool_suite_registry_inspector_view_model(
            host.tool_suite_registry(),
            host.workspace_profile_registry(),
            host.provider_family_provider_map(),
            host.provider_registry(),
            context.shell_state,
            active_document_context(context.app),
        );
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            inspector_lines(&view_model),
            Vec::new(),
        );

        Ok(ProviderSurfaceFrame {
            title: "Tool Suite Registry Inspector".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorViewModel {
    pub suite_rows: Vec<ToolSuiteRegistryInspectorSuiteRow>,
    pub surface_rows: Vec<ToolSuiteRegistryInspectorSurfaceRow>,
    pub provider_family_rows: Vec<ToolSuiteRegistryInspectorProviderFamilyRow>,
    pub provider_assignment_rows: Vec<ToolSuiteRegistryInspectorProviderAssignmentRow>,
    pub provider_rows: Vec<ToolSuiteRegistryInspectorProviderRow>,
    pub mounted_surface_rows: Vec<ToolSuiteRegistryInspectorMountedSurfaceRow>,
    pub persistence_preview_summary: ToolSuiteRegistryInspectorPersistencePreviewSummary,
    pub persistence_preview_rows: Vec<ToolSuiteRegistryInspectorPersistedSurfacePreviewRow>,
    pub diagnostic_summary: ToolSuiteRegistryInspectorDiagnosticSummary,
    pub diagnostic_rows: Vec<ToolSuiteRegistryInspectorDiagnosticRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorSuiteRow {
    pub suite_id: String,
    pub label: String,
    pub surface_count: usize,
    pub provider_family_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorSurfaceRow {
    pub stable_key: String,
    pub label: String,
    pub role: String,
    pub route: String,
    pub persistence: String,
    pub suite_id: String,
    pub provider_family_id: String,
    pub metadata_status: ToolSuiteRegistryInspectorMetadataStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorProviderFamilyRow {
    pub family_id: String,
    pub suite_id: String,
    pub label: String,
    pub assigned_provider_ids: Vec<SurfaceProviderId>,
    pub missing_assignment: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorProviderAssignmentRow {
    pub provider_family_id: String,
    pub provider_id: SurfaceProviderId,
    pub provider_label: Option<String>,
    pub assignment_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorProviderRow {
    pub provider_id: SurfaceProviderId,
    pub label: String,
    pub priority: SurfaceProviderPriority,
    pub registry_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorMountedSurfaceRow {
    pub workspace_profile_id: WorkspaceProfileId,
    pub panel_instance_id: PanelInstanceId,
    pub panel_kind: PanelKind,
    pub tab_stack_id: TabStackId,
    pub tool_surface_instance_id: ToolSurfaceInstanceId,
    pub stable_surface_key: String,
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    pub provider_family_id: Option<String>,
    pub surface_route: Option<ToolSurfaceRoute>,
    pub candidate_provider_ids: Vec<SurfaceProviderId>,
    pub support_modes: Vec<ToolSuiteRegistryInspectorSupportModeRow>,
    pub resolved_provider_id: Option<SurfaceProviderId>,
    pub resolution_status: SurfaceProviderAvailability,
    pub diagnostics: Vec<ToolSuiteRegistryInspectorDiagnosticRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorSupportModeRow {
    pub provider_id: SurfaceProviderId,
    pub support_mode: SurfaceProviderSupportMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorPersistencePreviewSummary {
    pub version: u32,
    pub surface_count: usize,
    pub stable_key_count: usize,
    pub legacy_metadata_count: usize,
    pub invalid_surface_count: usize,
    pub preview_status: ToolSuiteRegistryInspectorPersistencePreviewStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorPersistedSurfacePreviewRow {
    pub workspace_profile_id: WorkspaceProfileId,
    pub panel_instance_id: Option<PanelInstanceId>,
    pub tab_stack_id: Option<TabStackId>,
    pub tool_surface_instance_id: Option<ToolSurfaceInstanceId>,
    pub stable_surface_key: String,
    pub legacy_tool_surface_kind: Option<String>,
    pub composition_identity: String,
    pub legacy_metadata_status: ToolSuiteRegistryInspectorLegacyMetadataStatus,
    pub validation_status: ToolSuiteRegistryInspectorPersistenceValidationStatus,
    pub diagnostics: Vec<ToolSuiteRegistryInspectorDiagnosticRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolSuiteRegistryInspectorPersistencePreviewStatus {
    Available,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolSuiteRegistryInspectorLegacyMetadataStatus {
    Absent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolSuiteRegistryInspectorPersistenceValidationStatus {
    Valid,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ToolSuiteRegistryInspectorDiagnosticRow {
    pub severity: ToolSuiteRegistryInspectorDiagnosticSeverity,
    pub code: &'static str,
    pub scope: ToolSuiteRegistryInspectorDiagnosticScope,
    pub message: String,
    pub related_suite_id: Option<String>,
    pub related_surface_key: Option<String>,
    pub related_provider_family_id: Option<String>,
    pub related_provider_id: Option<SurfaceProviderId>,
    pub related_mounted_surface_id: Option<ToolSurfaceInstanceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct ToolSuiteRegistryInspectorDiagnosticSummary {
    pub info_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolSuiteRegistryInspectorDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolSuiteRegistryInspectorDiagnosticScope {
    Suite,
    Surface,
    ProviderFamily,
    ProviderAssignment,
    ProviderResolution,
    MountedSurface,
    PersistencePreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToolSuiteRegistryInspectorMetadataStatus {
    Registered,
    MetadataOnly,
    Conservative,
}

pub(super) fn build_tool_suite_registry_inspector_view_model(
    tool_suite_registry: &ToolSuiteRegistry,
    workspace_profile_registry: &WorkspaceProfileRegistry,
    provider_family_provider_map: &ProviderFamilyProviderMap,
    provider_registry: &EditorSurfaceProviderRegistry,
    shell_state: &RunenwerkEditorShellState,
    document_context: SurfaceDocumentContext,
) -> ToolSuiteRegistryInspectorViewModel {
    let provider_descriptors = provider_registry.provider_descriptors().collect::<Vec<_>>();
    let provider_labels = provider_descriptors
        .iter()
        .map(|descriptor| (descriptor.id, descriptor.label.clone()))
        .collect::<BTreeMap<_, _>>();
    let assigned_provider_ids = provider_family_assignments_by_family(provider_family_provider_map);

    let suite_rows = tool_suite_registry.suites().iter().map(suite_row).collect();
    let surface_rows = tool_suite_registry
        .suites()
        .iter()
        .flat_map(|suite| surface_rows(suite, &assigned_provider_ids))
        .collect::<Vec<_>>();
    let provider_family_rows = tool_suite_registry
        .suites()
        .iter()
        .flat_map(|suite| provider_family_rows(suite, provider_family_provider_map))
        .collect();
    let provider_assignment_rows = provider_family_provider_map
        .assignments()
        .iter()
        .enumerate()
        .map(
            |(index, assignment)| ToolSuiteRegistryInspectorProviderAssignmentRow {
                provider_family_id: assignment.provider_family_id.as_str().to_string(),
                provider_id: assignment.provider_id,
                provider_label: provider_labels.get(&assignment.provider_id).cloned(),
                assignment_order: index,
            },
        )
        .collect::<Vec<_>>();
    let provider_rows = provider_descriptors
        .into_iter()
        .enumerate()
        .map(
            |(index, descriptor)| ToolSuiteRegistryInspectorProviderRow {
                provider_id: descriptor.id,
                label: descriptor.label,
                priority: descriptor.priority,
                registry_order: index,
            },
        )
        .collect();
    let mounted_surface_rows = mounted_surface_rows(
        shell_state,
        document_context,
        tool_suite_registry,
        workspace_profile_registry,
        provider_family_provider_map,
        provider_registry,
    );
    let (persistence_preview_summary, persistence_preview_rows) =
        build_composition_persistence_preview(shell_state);
    let diagnostic_rows = diagnostic_rows(
        tool_suite_registry,
        provider_family_provider_map,
        &provider_labels,
        shell_state.composition_runtime(),
        &surface_rows,
        &mounted_surface_rows,
        &persistence_preview_rows,
    );
    let diagnostic_summary = diagnostic_summary(&diagnostic_rows);

    ToolSuiteRegistryInspectorViewModel {
        suite_rows,
        surface_rows,
        provider_family_rows,
        provider_assignment_rows,
        provider_rows,
        mounted_surface_rows,
        persistence_preview_summary,
        persistence_preview_rows,
        diagnostic_summary,
        diagnostic_rows,
    }
}

fn provider_family_assignments_by_family(
    provider_family_provider_map: &ProviderFamilyProviderMap,
) -> BTreeMap<String, Vec<SurfaceProviderId>> {
    let mut assignments = BTreeMap::<String, Vec<SurfaceProviderId>>::new();
    for assignment in provider_family_provider_map.assignments() {
        assignments
            .entry(assignment.provider_family_id.as_str().to_string())
            .or_default()
            .push(assignment.provider_id);
    }
    assignments
}

fn suite_row(suite: &EditorToolSuite) -> ToolSuiteRegistryInspectorSuiteRow {
    ToolSuiteRegistryInspectorSuiteRow {
        suite_id: suite.suite_id.as_str().to_string(),
        label: suite.label.clone(),
        surface_count: suite.surfaces.len(),
        provider_family_count: suite.provider_families.len(),
    }
}

fn surface_rows(
    suite: &EditorToolSuite,
    assigned_provider_ids: &BTreeMap<String, Vec<SurfaceProviderId>>,
) -> Vec<ToolSuiteRegistryInspectorSurfaceRow> {
    suite
        .surfaces
        .iter()
        .map(|surface| surface_row(suite, surface, assigned_provider_ids))
        .collect()
}

fn surface_row(
    suite: &EditorToolSuite,
    surface: &ToolSurfaceDefinition,
    assigned_provider_ids: &BTreeMap<String, Vec<SurfaceProviderId>>,
) -> ToolSuiteRegistryInspectorSurfaceRow {
    ToolSuiteRegistryInspectorSurfaceRow {
        stable_key: surface.key.as_str().to_string(),
        label: surface.label.clone(),
        role: format!("{:?}", surface.role),
        route: format!("{:?}", surface.route),
        persistence: format!("{:?}", surface.persistence),
        suite_id: suite.suite_id.as_str().to_string(),
        provider_family_id: surface.provider_family.as_str().to_string(),
        metadata_status: metadata_status(surface, assigned_provider_ids),
    }
}

fn provider_family_rows(
    suite: &EditorToolSuite,
    provider_family_provider_map: &ProviderFamilyProviderMap,
) -> Vec<ToolSuiteRegistryInspectorProviderFamilyRow> {
    suite
        .provider_families
        .iter()
        .map(|provider_family| {
            let assigned_provider_ids = provider_family_provider_map
                .providers_for(&provider_family.id)
                .collect::<Vec<_>>();
            ToolSuiteRegistryInspectorProviderFamilyRow {
                family_id: provider_family.id.as_str().to_string(),
                suite_id: suite.suite_id.as_str().to_string(),
                label: provider_family.label.clone(),
                missing_assignment: assigned_provider_ids.is_empty(),
                assigned_provider_ids,
            }
        })
        .collect()
}

fn metadata_status(
    surface: &ToolSurfaceDefinition,
    assigned_provider_ids: &BTreeMap<String, Vec<SurfaceProviderId>>,
) -> ToolSuiteRegistryInspectorMetadataStatus {
    let assigned = assigned_provider_ids
        .get(surface.provider_family.as_str())
        .is_some_and(|ids| !ids.is_empty());
    if surface.key.as_str() == "runenwerk.diagnostics.placeholder" || !assigned {
        ToolSuiteRegistryInspectorMetadataStatus::MetadataOnly
    } else if surface.key.as_str() == TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY {
        ToolSuiteRegistryInspectorMetadataStatus::Registered
    } else {
        ToolSuiteRegistryInspectorMetadataStatus::Conservative
    }
}

fn diagnostic_rows(
    tool_suite_registry: &ToolSuiteRegistry,
    provider_family_provider_map: &ProviderFamilyProviderMap,
    provider_labels: &BTreeMap<SurfaceProviderId, String>,
    composition: &editor_shell::EditorCompositionRuntime,
    surface_rows: &[ToolSuiteRegistryInspectorSurfaceRow],
    mounted_surface_rows: &[ToolSuiteRegistryInspectorMountedSurfaceRow],
    persistence_preview_rows: &[ToolSuiteRegistryInspectorPersistedSurfacePreviewRow],
) -> Vec<ToolSuiteRegistryInspectorDiagnosticRow> {
    let mut diagnostics = Vec::new();
    for row in surface_rows {
        if row.metadata_status == ToolSuiteRegistryInspectorMetadataStatus::MetadataOnly {
            diagnostics.push(
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Info,
                    ToolSuiteRegistryInspectorDiagnosticScope::Surface,
                    "inspector.surface.metadata_only",
                    format!(
                        "surface `{}` is registered as metadata-only or has no active provider assignment",
                        row.stable_key
                    ),
                )
                .with_suite_id(row.suite_id.clone())
                .with_surface_key(row.stable_key.clone())
                .with_provider_family_id(row.provider_family_id.clone()),
            );
        }
    }

    for suite in tool_suite_registry.suites() {
        if suite.surfaces.is_empty() && suite.provider_families.is_empty() {
            diagnostics.push(
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Warning,
                    ToolSuiteRegistryInspectorDiagnosticScope::Suite,
                    "inspector.suite.empty",
                    format!(
                        "suite `{}` has no surfaces or provider families",
                        suite.suite_id
                    ),
                )
                .with_suite_id(suite.suite_id.as_str().to_string()),
            );
        }

        for provider_family in &suite.provider_families {
            if provider_family_provider_map
                .providers_for(&provider_family.id)
                .next()
                .is_none()
            {
                diagnostics.push(
                    diagnostic_row(
                        ToolSuiteRegistryInspectorDiagnosticSeverity::Warning,
                        ToolSuiteRegistryInspectorDiagnosticScope::ProviderFamily,
                        "inspector.provider_family.missing_assignment",
                        format!(
                            "provider family `{}` has no concrete provider assignment",
                            provider_family.id.as_str()
                        ),
                    )
                    .with_suite_id(suite.suite_id.as_str().to_string())
                    .with_provider_family_id(provider_family.id.as_str().to_string()),
                );
            }
        }
    }

    for assignment in provider_family_provider_map.assignments() {
        if !provider_labels.contains_key(&assignment.provider_id) {
            diagnostics.push(
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Error,
                    ToolSuiteRegistryInspectorDiagnosticScope::ProviderAssignment,
                    "inspector.provider_assignment.unknown_provider",
                    format!(
                        "provider family `{}` references unknown provider `{}`",
                        assignment.provider_family_id.as_str(),
                        assignment.provider_id
                    ),
                )
                .with_provider_family_id(assignment.provider_family_id.as_str().to_string())
                .with_provider_id(assignment.provider_id),
            );
        }
    }

    for mounted_unit in composition.extension().mounted_units() {
        let Ok(stable_key) = ToolSurfaceStableKey::new(mounted_unit.stable_content_key.clone())
        else {
            continue;
        };
        if tool_suite_registry.surfaces().get(&stable_key).is_none()
            && let Ok(tool_surface_id) =
                ToolSurfaceInstanceId::try_from_raw(mounted_unit.compatibility_surface_raw)
        {
            diagnostics.push(
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Error,
                    ToolSuiteRegistryInspectorDiagnosticScope::Surface,
                    "inspector.surface.unknown_stable_key",
                    format!(
                        "mounted unit `{}` references unknown stable key `{}`",
                        mounted_unit.mounted_unit_id.raw(),
                        stable_key.as_str()
                    ),
                )
                .with_surface_key(stable_key.as_str().to_string())
                .with_mounted_surface_id(tool_surface_id),
            );
        }
    }

    diagnostics.extend(mounted_surface_diagnostics(mounted_surface_rows));
    diagnostics.extend(persistence_preview_diagnostics(persistence_preview_rows));

    diagnostics
}

fn mounted_surface_diagnostics(
    mounted_surface_rows: &[ToolSuiteRegistryInspectorMountedSurfaceRow],
) -> Vec<ToolSuiteRegistryInspectorDiagnosticRow> {
    let mut diagnostics = Vec::new();
    for row in mounted_surface_rows {
        if row.provider_family_id.is_none() {
            diagnostics.push(
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Error,
                    ToolSuiteRegistryInspectorDiagnosticScope::MountedSurface,
                    "inspector.mounted_surface.missing_surface_definition",
                    format!(
                        "mounted surface `{}` has no registered surface definition for stable key `{}`",
                        row.tool_surface_instance_id, row.stable_surface_key
                    ),
                )
                .with_surface_key(row.stable_surface_key.clone())
                .with_mounted_surface_id(row.tool_surface_instance_id),
            );
        }

        if row.candidate_provider_ids.is_empty()
            && row.resolution_status != SurfaceProviderAvailability::Available
        {
            diagnostics.push(
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Warning,
                    ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution,
                    "inspector.provider_resolution.no_candidate_provider",
                    format!(
                        "mounted surface `{}` has no candidate providers after provider-family filtering",
                        row.tool_surface_instance_id
                    ),
                )
                .with_surface_key(row.stable_surface_key.clone())
                .with_optional_provider_family_id(row.provider_family_id.clone())
                .with_mounted_surface_id(row.tool_surface_instance_id),
            );
        }

        diagnostics.extend(row.diagnostics.iter().cloned().map(|diagnostic| {
            diagnostic
                .with_scope(ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution)
                .with_surface_key(row.stable_surface_key.clone())
                .with_optional_provider_family_id(row.provider_family_id.clone())
                .with_mounted_surface_id(row.tool_surface_instance_id)
                .with_severity(provider_resolution_severity(row.resolution_status.clone()))
        }));
    }
    diagnostics
}

fn persistence_preview_diagnostics(
    persistence_preview_rows: &[ToolSuiteRegistryInspectorPersistedSurfacePreviewRow],
) -> Vec<ToolSuiteRegistryInspectorDiagnosticRow> {
    persistence_preview_rows
        .iter()
        .flat_map(|row| {
            row.diagnostics.iter().cloned().map(|diagnostic| {
                diagnostic
                    .with_scope(ToolSuiteRegistryInspectorDiagnosticScope::PersistencePreview)
                    .with_surface_key(row.stable_surface_key.clone())
                    .with_optional_mounted_surface_id(row.tool_surface_instance_id)
            })
        })
        .collect()
}

fn provider_resolution_severity(
    availability: SurfaceProviderAvailability,
) -> ToolSuiteRegistryInspectorDiagnosticSeverity {
    match availability {
        SurfaceProviderAvailability::Available => {
            ToolSuiteRegistryInspectorDiagnosticSeverity::Info
        }
        SurfaceProviderAvailability::Unsupported => {
            ToolSuiteRegistryInspectorDiagnosticSeverity::Warning
        }
        SurfaceProviderAvailability::Ambiguous | SurfaceProviderAvailability::Error => {
            ToolSuiteRegistryInspectorDiagnosticSeverity::Error
        }
    }
}

fn diagnostic_summary(
    rows: &[ToolSuiteRegistryInspectorDiagnosticRow],
) -> ToolSuiteRegistryInspectorDiagnosticSummary {
    let mut summary = ToolSuiteRegistryInspectorDiagnosticSummary::default();
    for row in rows {
        match row.severity {
            ToolSuiteRegistryInspectorDiagnosticSeverity::Info => {
                summary.info_count += 1;
            }
            ToolSuiteRegistryInspectorDiagnosticSeverity::Warning => {
                summary.warning_count += 1;
            }
            ToolSuiteRegistryInspectorDiagnosticSeverity::Error => {
                summary.error_count += 1;
            }
        }
    }
    summary
}

fn diagnostic_row(
    severity: ToolSuiteRegistryInspectorDiagnosticSeverity,
    scope: ToolSuiteRegistryInspectorDiagnosticScope,
    code: &'static str,
    message: String,
) -> ToolSuiteRegistryInspectorDiagnosticRow {
    ToolSuiteRegistryInspectorDiagnosticRow {
        severity,
        code,
        scope,
        message,
        related_suite_id: None,
        related_surface_key: None,
        related_provider_family_id: None,
        related_provider_id: None,
        related_mounted_surface_id: None,
    }
}

impl ToolSuiteRegistryInspectorDiagnosticRow {
    fn with_severity(mut self, severity: ToolSuiteRegistryInspectorDiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }

    fn with_scope(mut self, scope: ToolSuiteRegistryInspectorDiagnosticScope) -> Self {
        self.scope = scope;
        self
    }

    fn with_suite_id(mut self, suite_id: String) -> Self {
        self.related_suite_id = Some(suite_id);
        self
    }

    fn with_surface_key(mut self, surface_key: String) -> Self {
        self.related_surface_key = Some(surface_key);
        self
    }

    fn with_provider_family_id(mut self, provider_family_id: String) -> Self {
        self.related_provider_family_id = Some(provider_family_id);
        self
    }

    fn with_optional_provider_family_id(mut self, provider_family_id: Option<String>) -> Self {
        self.related_provider_family_id = provider_family_id;
        self
    }

    fn with_provider_id(mut self, provider_id: SurfaceProviderId) -> Self {
        self.related_provider_id = Some(provider_id);
        self
    }

    fn with_mounted_surface_id(mut self, mounted_surface_id: ToolSurfaceInstanceId) -> Self {
        self.related_mounted_surface_id = Some(mounted_surface_id);
        self
    }

    fn with_optional_mounted_surface_id(
        mut self,
        mounted_surface_id: Option<ToolSurfaceInstanceId>,
    ) -> Self {
        self.related_mounted_surface_id = mounted_surface_id;
        self
    }
}

fn mounted_surface_rows(
    shell_state: &RunenwerkEditorShellState,
    document_context: SurfaceDocumentContext,
    tool_suite_registry: &ToolSuiteRegistry,
    workspace_profile_registry: &WorkspaceProfileRegistry,
    provider_family_provider_map: &ProviderFamilyProviderMap,
    provider_registry: &EditorSurfaceProviderRegistry,
) -> Vec<ToolSuiteRegistryInspectorMountedSurfaceRow> {
    mounted_surface_requests_with_registry(
        shell_state,
        document_context,
        Some(tool_suite_registry.surfaces()),
    )
    .into_iter()
    .filter_map(|request| {
        let panel_kind_key = &shell_state
            .composition_runtime()
            .extension()
            .mounted_unit(request.mounted_unit_id)?
            .panel_kind_key;
        let panel_kind = editor_shell::tool_surface_kind_from_definition_key(panel_kind_key)
            .map(editor_shell::panel_kind_for_tool_surface_kind)?;
        let observation = provider_registry.observe_resolution_for_request(
            &request,
            workspace_profile_registry,
            Some(provider_family_provider_map),
        );
        Some(ToolSuiteRegistryInspectorMountedSurfaceRow {
            workspace_profile_id: request.workspace_profile_id,
            panel_instance_id: request.panel_instance_id,
            panel_kind,
            tab_stack_id: request.tab_stack_id,
            tool_surface_instance_id: request.tool_surface_instance_id,
            stable_surface_key: request.stable_surface_key.as_str().to_string(),
            legacy_tool_surface_kind: tool_surface_kind_for_stable_key(&request.stable_surface_key),
            provider_family_id: request
                .provider_family_id
                .as_ref()
                .map(|id| id.as_str().to_string()),
            surface_route: request.surface_route,
            candidate_provider_ids: observation.candidate_provider_ids,
            support_modes: observation
                .support_modes
                .into_iter()
                .map(|row| ToolSuiteRegistryInspectorSupportModeRow {
                    provider_id: row.provider_id,
                    support_mode: row.support_mode,
                })
                .collect(),
            resolved_provider_id: observation.selected_provider_id,
            resolution_status: observation.availability,
            diagnostics: observation
                .diagnostic
                .map(|diagnostic| {
                    vec![
                        diagnostic_row(
                            ToolSuiteRegistryInspectorDiagnosticSeverity::Info,
                            ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution,
                            diagnostic.code,
                            diagnostic.message,
                        )
                        .with_surface_key(request.stable_surface_key.as_str().to_string())
                        .with_mounted_surface_id(request.tool_surface_instance_id),
                    ]
                })
                .unwrap_or_default(),
        })
    })
    .collect()
}

fn build_composition_persistence_preview(
    shell_state: &RunenwerkEditorShellState,
) -> (
    ToolSuiteRegistryInspectorPersistencePreviewSummary,
    Vec<ToolSuiteRegistryInspectorPersistedSurfacePreviewRow>,
) {
    let runtime = shell_state.composition_runtime();
    let rows = runtime
        .extension()
        .mounted_units()
        .iter()
        .map(|record| {
            let panel_instance_id = PanelInstanceId::try_from_raw(record.panel_instance_raw).ok();
            let tool_surface_instance_id =
                ToolSurfaceInstanceId::try_from_raw(record.compatibility_surface_raw).ok();
            let tab_stack_id = runtime
                .composition()
                .definition()
                .regions()
                .iter()
                .find_map(|region| {
                    let ui_composition::RegionKind::Stack { ordered_units, .. } = &region.kind
                    else {
                        return None;
                    };
                    ordered_units.contains(&record.mounted_unit_id).then(|| {
                        runtime
                            .extension()
                            .region(region.id)
                            .and_then(|extension| extension.tab_stack_raw)
                            .and_then(|raw| TabStackId::try_from_raw(raw).ok())
                    })?
                });
            let valid = panel_instance_id.is_some()
                && tool_surface_instance_id.is_some()
                && tab_stack_id.is_some();
            let diagnostics = (!valid)
                .then(|| {
                    vec![diagnostic_row(
                        ToolSuiteRegistryInspectorDiagnosticSeverity::Error,
                        ToolSuiteRegistryInspectorDiagnosticScope::PersistencePreview,
                        "inspector.composition_preview.invalid_editor_binding",
                        format!(
                            "mounted unit {} has invalid editor compatibility bindings",
                            record.mounted_unit_id.raw()
                        ),
                    )]
                })
                .unwrap_or_default();
            ToolSuiteRegistryInspectorPersistedSurfacePreviewRow {
                workspace_profile_id: shell_state.active_workspace_profile_id(),
                panel_instance_id,
                tab_stack_id,
                tool_surface_instance_id,
                stable_surface_key: record.stable_content_key.clone(),
                legacy_tool_surface_kind: None,
                composition_identity: format!("mounted-unit:{}", record.mounted_unit_id.raw()),
                legacy_metadata_status: ToolSuiteRegistryInspectorLegacyMetadataStatus::Absent,
                validation_status: if valid {
                    ToolSuiteRegistryInspectorPersistenceValidationStatus::Valid
                } else {
                    ToolSuiteRegistryInspectorPersistenceValidationStatus::Error
                },
                diagnostics,
            }
        })
        .collect::<Vec<_>>();
    let invalid_surface_count = rows
        .iter()
        .filter(|row| {
            row.validation_status == ToolSuiteRegistryInspectorPersistenceValidationStatus::Error
        })
        .count();
    let summary = ToolSuiteRegistryInspectorPersistencePreviewSummary {
        version: editor_shell::EditorCompositionExtensionV1::SCHEMA_VERSION.into(),
        surface_count: rows.len(),
        stable_key_count: rows.len(),
        legacy_metadata_count: 0,
        invalid_surface_count,
        preview_status: if invalid_surface_count == 0 {
            ToolSuiteRegistryInspectorPersistencePreviewStatus::Available
        } else {
            ToolSuiteRegistryInspectorPersistencePreviewStatus::Error
        },
    };
    (summary, rows)
}

fn inspector_lines(view_model: &ToolSuiteRegistryInspectorViewModel) -> Vec<String> {
    let mut lines = vec![
        "Tool Suite Registry Inspector".to_string(),
        format!(
            "summary: {} suites, {} surfaces, {} provider families, {} providers, {} assignments, {} mounted surfaces, {} V5 preview surfaces",
            view_model.suite_rows.len(),
            view_model.surface_rows.len(),
            view_model.provider_family_rows.len(),
            view_model.provider_rows.len(),
            view_model.provider_assignment_rows.len(),
            view_model.mounted_surface_rows.len(),
            view_model.persistence_preview_rows.len()
        ),
        String::new(),
        "Suites".to_string(),
    ];
    lines.extend(view_model.suite_rows.iter().map(|row| {
        format!(
            "- {} ({}) surfaces={} provider_families={}",
            row.label, row.suite_id, row.surface_count, row.provider_family_count
        )
    }));
    lines.push(String::new());
    lines.push("Surfaces".to_string());
    lines.extend(view_model.surface_rows.iter().map(|row| {
        format!(
            "- {} ({}) suite={} family={} route={} status={:?}",
            row.label,
            row.stable_key,
            row.suite_id,
            row.provider_family_id,
            row.route,
            row.metadata_status
        )
    }));
    lines.push(String::new());
    lines.push("Provider Families".to_string());
    lines.extend(view_model.provider_family_rows.iter().map(|row| {
        format!(
            "- {} ({}) assigned={} missing_assignment={}",
            row.label,
            row.family_id,
            row.assigned_provider_ids.len(),
            row.missing_assignment
        )
    }));
    lines.push(String::new());
    lines.push("Providers".to_string());
    lines.extend(view_model.provider_rows.iter().map(|row| {
        format!(
            "- {} id={} priority={} order={}",
            row.label, row.provider_id, row.priority.0, row.registry_order
        )
    }));
    lines.push(String::new());
    lines.push("Mounted Surfaces".to_string());
    if view_model.mounted_surface_rows.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(view_model.mounted_surface_rows.iter().map(|row| {
            let provider_family = row.provider_family_id.as_deref().unwrap_or("<missing>");
            let surface_route = row
                .surface_route
                .map(|route| format!("{route:?}"))
                .unwrap_or_else(|| "<missing>".to_string());
            let candidates = format_provider_ids(&row.candidate_provider_ids);
            let support_modes = row
                .support_modes
                .iter()
                .map(|mode| format!("{}:{:?}", mode.provider_id, mode.support_mode))
                .collect::<Vec<_>>()
                .join(",");
            let resolved = row
                .resolved_provider_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "<none>".to_string());
            format!(
                "- surface={} panel={} {:?} stack={} key={} family={} route={} candidates=[{}] support=[{}] resolved={} status={:?}",
                row.tool_surface_instance_id,
                row.panel_instance_id,
                row.panel_kind,
                row.tab_stack_id,
                row.stable_surface_key,
                provider_family,
                surface_route,
                candidates,
                support_modes,
                resolved,
                row.resolution_status
            )
        }));
    }
    lines.push(String::new());
    lines.push("Composition Persistence Preview".to_string());
    lines.push(format!(
        "- version={} surfaces={} stable_keys={} legacy_metadata={} invalid={} status={:?}",
        view_model.persistence_preview_summary.version,
        view_model.persistence_preview_summary.surface_count,
        view_model.persistence_preview_summary.stable_key_count,
        view_model.persistence_preview_summary.legacy_metadata_count,
        view_model.persistence_preview_summary.invalid_surface_count,
        view_model.persistence_preview_summary.preview_status
    ));
    if view_model.persistence_preview_rows.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(view_model.persistence_preview_rows.iter().map(|row| {
            let surface_id = row
                .tool_surface_instance_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "<invalid>".to_string());
            let panel_id = row
                .panel_instance_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "<unmounted>".to_string());
            let tab_stack_id = row
                .tab_stack_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "<none>".to_string());
            let legacy = row.legacy_tool_surface_kind.as_deref().unwrap_or("<none>");
            format!(
                "- surface={} panel={} stack={} stable_key={} primary={} legacy={} legacy_status={:?} validation={:?}",
                surface_id,
                panel_id,
                tab_stack_id,
                row.stable_surface_key,
                row.composition_identity,
                legacy,
                row.legacy_metadata_status,
                row.validation_status
            )
        }));
    }
    lines.push(String::new());
    lines.push("Diagnostics".to_string());
    lines.push(format!(
        "- summary errors={} warnings={} info={}",
        view_model.diagnostic_summary.error_count,
        view_model.diagnostic_summary.warning_count,
        view_model.diagnostic_summary.info_count
    ));
    if view_model.diagnostic_rows.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(view_model.diagnostic_rows.iter().map(|row| {
            format!(
                "- {:?} {:?} {}: {}{}",
                row.severity,
                row.scope,
                row.code,
                row.message,
                format_diagnostic_related(row)
            )
        }));
    }

    lines
}

fn format_diagnostic_related(row: &ToolSuiteRegistryInspectorDiagnosticRow) -> String {
    let mut related = Vec::new();
    if let Some(suite_id) = row.related_suite_id.as_deref() {
        related.push(format!("suite={suite_id}"));
    }
    if let Some(surface_key) = row.related_surface_key.as_deref() {
        related.push(format!("surface={surface_key}"));
    }
    if let Some(provider_family_id) = row.related_provider_family_id.as_deref() {
        related.push(format!("family={provider_family_id}"));
    }
    if let Some(provider_id) = row.related_provider_id {
        related.push(format!("provider={provider_id}"));
    }
    if let Some(surface_id) = row.related_mounted_surface_id {
        related.push(format!("mounted={surface_id}"));
    }
    if related.is_empty() {
        String::new()
    } else {
        format!(" ({})", related.join(" "))
    }
}

fn format_provider_ids(provider_ids: &[SurfaceProviderId]) -> String {
    provider_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use editor_shell::{
        WorkspaceIdentityAllocator, WorkspaceMutation, WorkspaceState, reduce_workspace,
    };

    use super::*;

    fn view_model() -> ToolSuiteRegistryInspectorViewModel {
        let mut app = RunenwerkEditorApp::new();
        app.runtime_mut()
            .session_mut()
            .upsert_document(editor_core::DocumentDescriptor::new(
                editor_core::DocumentId(1),
                editor_core::DocumentKind::Scene,
                "Scene",
            ));
        let shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("shell state should build from hosted registry");

        build_tool_suite_registry_inspector_view_model(
            app.workbench_host().tool_suite_registry(),
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().provider_family_provider_map(),
            app.workbench_host().provider_registry(),
            &shell_state,
            active_document_context(&app),
        )
    }

    fn view_model_with_unassigned_future_suite() -> ToolSuiteRegistryInspectorViewModel {
        let app = RunenwerkEditorApp::new();
        let tool_suite_registry = ToolSuiteRegistry::new(vec![
            crate::shell::tool_suites::diagnostics_tool_suite::diagnostics_tool_suite(),
            crate::shell::tool_suites::gameplay_tool_suite::gameplay_tool_suite(),
        ])
        .expect("inspector fixture suite registry should build");
        let provider_family_provider_map =
            ProviderFamilyProviderMap::new(&tool_suite_registry, Vec::new())
                .expect("empty assignments are valid for inspector diagnostics fixtures");
        let shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("shell state should build from hosted registry");

        build_tool_suite_registry_inspector_view_model(
            &tool_suite_registry,
            app.workbench_host().workspace_profile_registry(),
            &provider_family_provider_map,
            app.workbench_host().provider_registry(),
            &shell_state,
            active_document_context(&app),
        )
    }

    fn view_model_from_shell_state(
        app: &RunenwerkEditorApp,
        shell_state: &RunenwerkEditorShellState,
    ) -> ToolSuiteRegistryInspectorViewModel {
        build_tool_suite_registry_inspector_view_model(
            app.workbench_host().tool_suite_registry(),
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().provider_family_provider_map(),
            app.workbench_host().provider_registry(),
            shell_state,
            active_document_context(app),
        )
    }

    fn shell_state() -> RunenwerkEditorShellState {
        let app = RunenwerkEditorApp::new();
        RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("shell state should build from hosted registry")
    }

    fn workspace_with_unknown_stable_key() -> WorkspaceState {
        let shell_state = shell_state();
        let workspace = shell_state.workspace_state();
        let panel_id = workspace
            .panels()
            .find(|panel| panel.active_tool_surface.is_some())
            .expect("default workspace should have a mounted panel")
            .id;
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());

        reduce_workspace(
            workspace,
            WorkspaceMutation::ReplacePanelToolSurfaceStableKey {
                panel_id,
                tool_surface_id: allocator.allocate_tool_surface_instance_id(),
                stable_surface_key: editor_shell::ToolSurfaceStableKey::new(
                    "runenwerk.unknown.surface",
                )
                .expect("test stable key should be syntactically valid"),
            },
        )
        .expect("test workspace should accept stable-key-native unknown registry key")
    }

    fn mounted_surface_row_with_diagnostic(
        availability: SurfaceProviderAvailability,
        code: &'static str,
    ) -> ToolSuiteRegistryInspectorMountedSurfaceRow {
        let surface_id =
            ToolSurfaceInstanceId::try_from_raw(1).expect("test surface id should be non-zero");
        ToolSuiteRegistryInspectorMountedSurfaceRow {
            workspace_profile_id: editor_shell::SCENE_WORKSPACE_PROFILE_ID,
            panel_instance_id: PanelInstanceId::try_from_raw(1)
                .expect("test panel id should be non-zero"),
            panel_kind: PanelKind::Inspector,
            tab_stack_id: TabStackId::try_from_raw(1)
                .expect("test tab stack id should be non-zero"),
            tool_surface_instance_id: surface_id,
            stable_surface_key: TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY.to_string(),
            legacy_tool_surface_kind: None,
            provider_family_id: Some("runenwerk.diagnostics".to_string()),
            surface_route: Some(ToolSurfaceRoute::ProviderOwnedLocal),
            candidate_provider_ids: Vec::new(),
            support_modes: Vec::new(),
            resolved_provider_id: None,
            resolution_status: availability,
            diagnostics: vec![
                diagnostic_row(
                    ToolSuiteRegistryInspectorDiagnosticSeverity::Info,
                    ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution,
                    code,
                    "test diagnostic".to_string(),
                )
                .with_mounted_surface_id(surface_id),
            ],
        }
    }

    #[test]
    fn inspector_provider_builds_dto_without_mutating_registries() {
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new_with_tool_surface_registry(
            app.workbench_host().tool_surface_registry(),
        )
        .expect("shell state should build from hosted registry");
        let workspace_before = shell_state.workspace_state().clone();
        let suite_count_before = app.workbench_host().tool_suite_registry().suites().len();
        let surface_count_before = app.workbench_host().tool_surface_registry().iter().count();
        let provider_count_before = app
            .workbench_host()
            .provider_registry()
            .provider_ids()
            .count();
        let assignment_count_before = app
            .workbench_host()
            .provider_family_provider_map()
            .assignments()
            .len();

        let view_model = build_tool_suite_registry_inspector_view_model(
            app.workbench_host().tool_suite_registry(),
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().provider_family_provider_map(),
            app.workbench_host().provider_registry(),
            &shell_state,
            active_document_context(&app),
        );

        assert_eq!(view_model.suite_rows.len(), suite_count_before);
        assert_eq!(view_model.surface_rows.len(), surface_count_before);
        assert_eq!(view_model.provider_rows.len(), provider_count_before);
        assert_eq!(
            view_model.provider_assignment_rows.len(),
            assignment_count_before
        );
        assert_eq!(
            app.workbench_host().tool_suite_registry().suites().len(),
            suite_count_before
        );
        assert_eq!(
            app.workbench_host().tool_surface_registry().iter().count(),
            surface_count_before
        );
        assert_eq!(
            app.workbench_host()
                .provider_registry()
                .provider_ids()
                .count(),
            provider_count_before
        );
        assert_eq!(
            app.workbench_host()
                .provider_family_provider_map()
                .assignments()
                .len(),
            assignment_count_before
        );
        assert_eq!(shell_state.workspace_state(), &workspace_before);
    }

    #[test]
    fn surface_rows_include_tool_suite_registry_inspector() {
        let view_model = view_model();

        let row = view_model
            .surface_rows
            .iter()
            .find(|row| row.stable_key == TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY)
            .expect("inspector surface row should be present");

        assert_eq!(row.label, "Tool Suite Registry Inspector");
        assert_eq!(row.suite_id, "runenwerk.diagnostics");
        assert_eq!(row.provider_family_id, "runenwerk.diagnostics");
        assert_eq!(
            row.metadata_status,
            ToolSuiteRegistryInspectorMetadataStatus::Registered
        );
    }

    #[test]
    fn installed_suite_list_includes_diagnostics_and_material_lab() {
        let view_model = view_model();
        let suite_ids = view_model
            .suite_rows
            .iter()
            .map(|row| row.suite_id.as_str())
            .collect::<BTreeSet<_>>();

        assert!(suite_ids.contains("runenwerk.diagnostics"));
        assert!(suite_ids.contains("runenwerk.material_lab"));
    }

    #[test]
    fn provider_family_assignments_are_visible() {
        let view_model = view_model();
        let diagnostics_family = view_model
            .provider_family_rows
            .iter()
            .find(|row| row.family_id == "runenwerk.diagnostics")
            .expect("diagnostics provider family should be visible");

        assert!(
            diagnostics_family
                .assigned_provider_ids
                .contains(&M6_WORKSPACE_PROVIDER_ID)
        );
        assert!(
            diagnostics_family
                .assigned_provider_ids
                .contains(&TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID)
        );
        assert!(!diagnostics_family.missing_assignment);
    }

    #[test]
    fn concrete_provider_registry_entries_are_visible() {
        let view_model = view_model();
        let row = view_model
            .provider_rows
            .iter()
            .find(|row| row.provider_id == TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID)
            .expect("inspector provider row should be present");

        assert_eq!(row.label, "Tool Suite Registry Inspector");
    }

    #[test]
    fn placeholder_future_suites_are_marked_metadata_only() {
        let view_model = view_model_with_unassigned_future_suite();

        let placeholder = view_model
            .surface_rows
            .iter()
            .find(|row| row.stable_key == "runenwerk.diagnostics.placeholder")
            .expect("diagnostics placeholder should be visible");
        assert_eq!(
            placeholder.metadata_status,
            ToolSuiteRegistryInspectorMetadataStatus::MetadataOnly
        );

        let gameplay = view_model
            .surface_rows
            .iter()
            .find(|row| row.stable_key == "runenwerk.gameplay.graph_canvas")
            .expect("future gameplay metadata should be visible");
        assert_eq!(
            gameplay.metadata_status,
            ToolSuiteRegistryInspectorMetadataStatus::MetadataOnly
        );
    }

    #[test]
    fn inspector_view_model_includes_mounted_surface_rows() {
        let view_model = view_model();

        assert!(!view_model.mounted_surface_rows.is_empty());
    }

    #[test]
    fn mounted_surface_rows_use_stable_keys_as_authority() {
        let view_model = view_model();

        assert!(
            view_model
                .mounted_surface_rows
                .iter()
                .all(|row| row.stable_surface_key.starts_with("runenwerk."))
        );
    }

    #[test]
    fn mounted_surface_rows_treat_legacy_kind_as_display_metadata_only() {
        let view_model = view_model();

        assert!(view_model.mounted_surface_rows.iter().all(|row| {
            row.legacy_tool_surface_kind.is_some_and(|legacy_kind| {
                editor_shell::stable_key_for_tool_surface_kind(legacy_kind)
                    .as_ref()
                    .is_some_and(|stable_key| stable_key.as_str() == row.stable_surface_key)
            })
        }));
    }

    #[test]
    fn mounted_surface_rows_include_provider_family_and_route() {
        let view_model = view_model();

        assert!(
            view_model
                .mounted_surface_rows
                .iter()
                .all(|row| row.provider_family_id.is_some() && row.surface_route.is_some())
        );
    }

    #[test]
    fn mounted_surface_rows_include_candidate_providers() {
        let view_model = view_model();

        assert!(
            view_model
                .mounted_surface_rows
                .iter()
                .all(|row| !row.candidate_provider_ids.is_empty())
        );
    }

    #[test]
    fn mounted_surface_rows_include_support_modes() {
        let view_model = view_model();

        assert!(view_model.mounted_surface_rows.iter().all(|row| {
            row.support_modes.len() == row.candidate_provider_ids.len()
                && row
                    .support_modes
                    .iter()
                    .all(|support| row.candidate_provider_ids.contains(&support.provider_id))
        }));
    }

    #[test]
    fn mounted_surface_rows_include_resolved_provider() {
        let view_model = view_model();

        assert!(view_model.mounted_surface_rows.iter().any(|row| {
            row.resolution_status == SurfaceProviderAvailability::Available
                && row.resolved_provider_id.is_some()
        }));
    }

    #[test]
    fn composition_preview_is_read_only_and_uses_mounted_unit_identity() {
        let view_model = view_model();
        assert_eq!(
            view_model.persistence_preview_summary.version,
            editor_shell::EditorCompositionExtensionV1::SCHEMA_VERSION as u32,
        );
        assert!(!view_model.persistence_preview_rows.is_empty());
        assert!(view_model.persistence_preview_rows.iter().all(|row| {
            row.stable_surface_key.starts_with("runenwerk.")
                && row.composition_identity.starts_with("mounted-unit:")
                && row.validation_status
                    == ToolSuiteRegistryInspectorPersistenceValidationStatus::Valid
        }));

        let provider_source = include_str!("tool_suite_registry_inspector.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(!provider_source.contains("PersistedWorkspaceStateV5"));
        assert!(!provider_source.contains("to_persisted_v5"));
        assert!(!provider_source.contains("WorkspaceMutation"));
    }

    #[test]
    fn inspector_phase_c_preserves_phase_a_registry_rows() {
        let view_model = view_model();

        assert!(!view_model.suite_rows.is_empty());
        assert!(!view_model.surface_rows.is_empty());
        assert!(!view_model.provider_family_rows.is_empty());
        assert!(!view_model.provider_rows.is_empty());
    }

    #[test]
    fn inspector_phase_c_preserves_phase_b_mounted_surface_rows() {
        let view_model = view_model();

        assert!(!view_model.mounted_surface_rows.is_empty());
    }

    #[test]
    fn diagnostics_section_summarizes_severity_counts() {
        let view_model = view_model();

        assert_eq!(
            view_model.diagnostic_summary.info_count,
            view_model
                .diagnostic_rows
                .iter()
                .filter(|row| row.severity == ToolSuiteRegistryInspectorDiagnosticSeverity::Info)
                .count()
        );
        assert_eq!(
            view_model.diagnostic_summary.warning_count,
            view_model
                .diagnostic_rows
                .iter()
                .filter(|row| {
                    row.severity == ToolSuiteRegistryInspectorDiagnosticSeverity::Warning
                })
                .count()
        );
        assert_eq!(
            view_model.diagnostic_summary.error_count,
            view_model
                .diagnostic_rows
                .iter()
                .filter(|row| row.severity == ToolSuiteRegistryInspectorDiagnosticSeverity::Error)
                .count()
        );
    }

    #[test]
    fn metadata_only_placeholder_appears_as_info_diagnostic() {
        let view_model = view_model();

        let diagnostic = view_model
            .diagnostic_rows
            .iter()
            .find(|row| {
                row.code == "inspector.surface.metadata_only"
                    && row.related_surface_key.as_deref()
                        == Some("runenwerk.diagnostics.placeholder")
            })
            .expect("metadata-only placeholder diagnostic should be visible");

        assert_eq!(
            diagnostic.severity,
            ToolSuiteRegistryInspectorDiagnosticSeverity::Info
        );
        assert_eq!(
            diagnostic.scope,
            ToolSuiteRegistryInspectorDiagnosticScope::Surface
        );
    }

    #[test]
    fn missing_provider_family_assignment_appears_as_diagnostic() {
        let view_model = view_model_with_unassigned_future_suite();

        assert!(view_model.diagnostic_rows.iter().any(|row| {
            row.code == "inspector.provider_family.missing_assignment"
                && row.scope == ToolSuiteRegistryInspectorDiagnosticScope::ProviderFamily
                && row.related_provider_family_id.is_some()
        }));
    }

    #[test]
    fn unknown_stable_key_appears_as_diagnostic() {
        let app = RunenwerkEditorApp::new();
        let mut shell_state = shell_state();
        shell_state.replace_workspace_state(workspace_with_unknown_stable_key());

        let view_model = view_model_from_shell_state(&app, &shell_state);

        assert!(view_model.diagnostic_rows.iter().any(|row| {
            row.code == "inspector.surface.unknown_stable_key"
                && row.scope == ToolSuiteRegistryInspectorDiagnosticScope::Surface
                && row.related_surface_key.as_deref() == Some("runenwerk.unknown.surface")
        }));
    }

    #[test]
    fn unsupported_provider_resolution_appears_as_diagnostic() {
        let rows = mounted_surface_diagnostics(&[mounted_surface_row_with_diagnostic(
            SurfaceProviderAvailability::Unsupported,
            "editor.surface.unsupported_provider",
        )]);

        assert!(rows.iter().any(|row| {
            row.code == "editor.surface.unsupported_provider"
                && row.scope == ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution
                && row.severity == ToolSuiteRegistryInspectorDiagnosticSeverity::Warning
        }));
    }

    #[test]
    fn ambiguous_provider_resolution_appears_as_diagnostic() {
        let rows = mounted_surface_diagnostics(&[mounted_surface_row_with_diagnostic(
            SurfaceProviderAvailability::Ambiguous,
            "editor.surface.ambiguous_provider",
        )]);

        assert!(rows.iter().any(|row| {
            row.code == "editor.surface.ambiguous_provider"
                && row.scope == ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution
                && row.severity == ToolSuiteRegistryInspectorDiagnosticSeverity::Error
        }));
    }

    #[test]
    fn provider_error_resolution_appears_as_diagnostic() {
        let rows = mounted_surface_diagnostics(&[mounted_surface_row_with_diagnostic(
            SurfaceProviderAvailability::Error,
            "editor.surface.provider_error",
        )]);

        assert!(rows.iter().any(|row| {
            row.code == "editor.surface.provider_error"
                && row.scope == ToolSuiteRegistryInspectorDiagnosticScope::ProviderResolution
                && row.severity == ToolSuiteRegistryInspectorDiagnosticSeverity::Error
        }));
    }

    #[test]
    fn phase_d_preserves_phase_a_b_c_sections() {
        let view_model = view_model();

        assert!(!view_model.suite_rows.is_empty());
        assert!(!view_model.surface_rows.is_empty());
        assert!(!view_model.provider_rows.is_empty());
        assert!(!view_model.mounted_surface_rows.is_empty());
        assert!(!view_model.persistence_preview_rows.is_empty());
        assert!(!view_model.diagnostic_rows.is_empty());
    }
}
