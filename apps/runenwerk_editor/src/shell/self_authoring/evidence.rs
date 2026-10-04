//! UI Designer runtime-evidence capture DTOs owned by the self-authoring session.

use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub struct EditorLabProductPathEvidenceCapture {
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
    pub performance_baselines: Vec<EditorLabPerformanceBaseline>,
}

impl EditorLabProductPathEvidenceCapture {
    pub fn new(
        artifacts: impl IntoIterator<Item = EditorLabEvidenceArtifact>,
        performance_baselines: impl IntoIterator<Item = EditorLabPerformanceBaseline>,
    ) -> Self {
        Self {
            artifacts: artifacts.into_iter().collect(),
            performance_baselines: performance_baselines.into_iter().collect(),
        }
    }
}

impl SelfAuthoringWorkspaceState {
    pub fn selected_source_version_label(&self) -> Option<String> {
        self.selected_source_revision()
            .map(|revision| revision.display_label())
    }

    pub fn selected_source_revision(&self) -> Option<EditorLabSourceRevision> {
        self.selected_document()
            .map(|document| self.source_revision_for_document(document))
    }

    pub fn last_scenario_evidence_packets(&self) -> &[EditorLabScenarioEvidencePacket] {
        &self.last_scenario_evidence_packets
    }

    pub fn capture_pm005_scenario_evidence_packets(
        &mut self,
        theme: &ThemeTokens,
    ) -> Result<Vec<EditorLabScenarioEvidencePacket>, UiDefinitionDiagnostic> {
        self.capture_pm005_scenario_evidence_packets_with_product_capture(
            theme,
            EditorLabProductPathEvidenceCapture::default(),
        )
    }

    pub fn capture_pm005_scenario_evidence_packets_with_product_capture(
        &mut self,
        theme: &ThemeTokens,
        product_capture: EditorLabProductPathEvidenceCapture,
    ) -> Result<Vec<EditorLabScenarioEvidencePacket>, UiDefinitionDiagnostic> {
        let document = self.selected_document().cloned().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.lab.evidence.scenario.no_selected_document",
                "no selected UI Designer document is available for scenario evidence capture",
            )
        })?;
        let source_revision = self.source_revision_for_document(&document);
        let diagnostics = self.diagnostics_for_document(&document.id);
        let runtime_product_capture =
            self.runtime_product_evidence_capture_for_selected_document(theme, product_capture)?;
        let mut packets = Vec::new();
        for target_profile in UI_DESIGNER_SCENARIO_EVIDENCE_TARGETS {
            let packet = self.capture_pm005_scenario_evidence_packet_for_target(
                &document,
                source_revision.clone(),
                diagnostics.clone(),
                theme,
                target_profile,
                &runtime_product_capture,
            )?;
            packets.push(packet);
        }
        self.last_scenario_evidence_packets = packets.clone();
        Ok(packets)
    }

    fn source_revision_for_document(
        &self,
        document: &EditorDefinitionDocument,
    ) -> EditorLabSourceRevision {
        let serialized = ron::ser::to_string_pretty(document, PrettyConfig::new())
            .unwrap_or_else(|_| format!("{document:?}"));
        EditorLabSourceRevision::new(
            document.id.as_str(),
            document.schema_version,
            format!("blake3:{}", blake3::hash(serialized.as_bytes()).to_hex()),
            self.source_revision_epoch,
        )
    }

    fn capture_pm005_scenario_evidence_packet_for_target(
        &self,
        document: &EditorDefinitionDocument,
        source_revision: EditorLabSourceRevision,
        diagnostics: Vec<UiDefinitionDiagnostic>,
        _theme: &ThemeTokens,
        target_profile: &str,
        runtime_product_capture: &EditorLabProductPathEvidenceCapture,
    ) -> Result<EditorLabScenarioEvidencePacket, UiDefinitionDiagnostic> {
        if target_profile == game_runtime::GAME_RUNTIME_TARGET_PROFILE {
            let unsupported_report = format!(
                "target={target_profile}\ndocument={}\nsource={}\nstatus=descriptor-compatible\nruntime-proof-owner=PT-GAME-RUNTIME-UI\n",
                document.id.as_str(),
                source_revision.display_label()
            );
            let artifact = EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::UnsupportedCheckReport,
                format!(
                    "evidence://ui-designer/v1-closure/pm005/{}/{}/descriptor-compatibility",
                    target_profile.replace('.', "-"),
                    document.id.as_str()
                ),
                unsupported_report.as_bytes(),
                EditorLabEvidenceArtifactProvenance::UnsupportedCheck,
                "game.runtime descriptor compatibility report; runtime proof is owned by PT-GAME-RUNTIME-UI",
            );
            let packet = EditorLabDescriptorCompatibilityEvidencePacket::new(
                "runenwerk.editor.ui_designer_workbench.v1",
                document.id.as_str(),
                source_revision,
                target_profile,
                pm005_scenario_id(target_profile),
            )
            .with_diagnostics(diagnostics)
            .with_artifacts(vec![artifact])
            .with_unsupported_checks(vec![EditorLabUnsupportedCheckDiagnostic::new(
                "concrete game HUD runtime",
                "PT-GAME-RUNTIME-UI owns concrete game HUD behavior; UI Designer records descriptor compatibility only",
            )])
            .with_fixture_bindings(pm005_fixture_binding_descriptors(target_profile))
            .with_intent_descriptors(pm005_intent_descriptors(target_profile));

            let packet = EditorLabScenarioEvidencePacket::descriptor(packet);
            packet.validate_scenario_evidence()?;
            return Ok(packet);
        }

        let packet = EditorLabRuntimeProductEvidencePacket::new(
            "runenwerk.editor.ui_designer_workbench.v1",
            document.id.as_str(),
            source_revision,
            target_profile,
            pm005_scenario_id(target_profile),
        )
        .with_diagnostics(diagnostics)
        .with_artifacts(runtime_product_capture.artifacts.clone())
        .with_performance_baselines(runtime_product_capture.performance_baselines.clone());

        let packet = EditorLabScenarioEvidencePacket::runtime(packet);
        packet.validate_scenario_evidence()?;
        Ok(packet)
    }

    fn measured_product_path_baseline(
        &self,
        kind: EditorLabPerformanceBaselineKind,
        description: impl Into<String>,
        sample: impl FnOnce() -> usize,
    ) -> EditorLabPerformanceBaseline {
        let started = Instant::now();
        let sample_count = sample().max(1);
        let elapsed_micros = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        EditorLabPerformanceBaseline::product_path(kind, elapsed_micros, sample_count, description)
    }

    fn runtime_product_evidence_capture_for_selected_document(
        &self,
        theme: &ThemeTokens,
        mut product_capture: EditorLabProductPathEvidenceCapture,
    ) -> Result<EditorLabProductPathEvidenceCapture, UiDefinitionDiagnostic> {
        let document = self.selected_document().ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.lab.evidence.runtime.no_selected_document",
                "runtime product evidence capture requires a selected document",
            )
        })?;
        let source_revision = self.source_revision_for_document(document);
        let preview_started = Instant::now();
        let preview = self.formed_selected_preview(theme).ok_or_else(|| {
            UiDefinitionDiagnostic::error(
                "editor.lab.evidence.runtime.no_retained_preview",
                "runtime product evidence capture requires a retained UI preview product",
            )
        })?;
        let preview_debug = format!("{preview:#?}");
        let preview_elapsed = preview_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        product_capture
            .artifacts
            .push(EditorLabEvidenceArtifact::from_content(
                EditorLabEvidenceArtifactKind::RetainedUiDebug,
                format!(
                    "evidence://ui-designer/runtime/{}/{}/retained-ui-debug",
                    document.id.as_str(),
                    source_revision.content_hash
                ),
                preview_debug.as_bytes(),
                EditorLabEvidenceArtifactProvenance::ProductPath,
                "retained UI Designer workbench product formed through ui_definition",
            ));
        product_capture
            .performance_baselines
            .push(EditorLabPerformanceBaseline::product_path(
                EditorLabPerformanceBaselineKind::CanvasInteraction,
                preview_elapsed,
                1,
                "editor.workbench retained canvas preview formation through ui_definition",
            ));
        product_capture
            .performance_baselines
            .push(self.measured_product_path_baseline(
                EditorLabPerformanceBaselineKind::Resize,
                "editor.workbench retained canvas formation across responsive widget id scopes",
                || {
                    [10_u64, 20_u64, 30_u64]
                        .into_iter()
                        .filter(|scope| {
                            self.formed_selected_preview_with_scope(theme, Some(*scope))
                                .is_some()
                        })
                        .count()
                },
            ));
        product_capture
            .performance_baselines
            .push(self.measured_product_path_baseline(
                EditorLabPerformanceBaselineKind::CatalogProjection,
                "editor.workbench component catalog projection from editor design recipe library",
                || {
                    editor_shell::editor_design_system_recipe_library()
                        .declarations
                        .len()
                },
            ));
        product_capture
            .performance_baselines
            .push(self.measured_product_path_baseline(
            EditorLabPerformanceBaselineKind::DiagnosticsProjection,
            "editor.workbench selected diagnostics projection through editor_definition validation",
            || self.selected_diagnostics().len(),
        ));
        if !product_capture
            .performance_baselines
            .iter()
            .any(|baseline| baseline.kind == EditorLabPerformanceBaselineKind::FrameBuild)
        {
            product_capture
                .performance_baselines
                .push(self.measured_product_path_baseline(
                    EditorLabPerformanceBaselineKind::FrameBuild,
                    "editor.workbench apply preview frame inventory from selected product state",
                    || {
                        usize::from(self.build_apply_preview().is_some())
                            + self.draft_documents().count()
                            + self.applied_count()
                    },
                ));
        }

        dedupe_product_path_baselines(&mut product_capture.performance_baselines);
        Ok(product_capture)
    }
}

fn pm005_scenario_id(target_profile: &str) -> String {
    format!(
        "ui-designer.v1-closure.pm005.{}",
        target_profile.replace('.', "-")
    )
}

fn dedupe_product_path_baselines(baselines: &mut Vec<EditorLabPerformanceBaseline>) {
    let mut seen = BTreeSet::new();
    baselines.retain(|baseline| seen.insert(baseline.kind));
}

fn pm005_fixture_binding_descriptors(
    target_profile: &str,
) -> Vec<EditorLabReadOnlyFixtureBindingDescriptor> {
    if target_profile == game_runtime::GAME_RUNTIME_TARGET_PROFILE {
        return game_runtime::descriptor_fixture_bindings();
    }
    let profile_segment = target_profile.replace('.', "-");
    vec![
        EditorLabReadOnlyFixtureBindingDescriptor::new(
            format!("fixture.{profile_segment}.selected-template"),
            format!("binding.{profile_segment}.source-version"),
            target_profile,
            EditorLabDescriptorCompatibility::Compatible,
            "source-versioned selected UI template fixture",
        ),
        EditorLabReadOnlyFixtureBindingDescriptor::new(
            format!("fixture.{profile_segment}.safe-area"),
            format!("binding.{profile_segment}.target-profile"),
            target_profile,
            EditorLabDescriptorCompatibility::Compatible,
            "read-only target-profile compatibility fixture",
        ),
    ]
}

fn pm005_intent_descriptors(target_profile: &str) -> Vec<EditorLabValidatedIntentDescriptor> {
    if target_profile == game_runtime::GAME_RUNTIME_TARGET_PROFILE {
        return game_runtime::validated_intent_descriptors();
    }
    let profile_segment = target_profile.replace('.', "-");
    vec![
        EditorLabValidatedIntentDescriptor::new(
            format!("intent.{profile_segment}.select-node"),
            target_profile,
            "validated EditorDefinition route descriptor",
        ),
        EditorLabValidatedIntentDescriptor::new(
            format!("intent.{profile_segment}.preview-activation"),
            target_profile,
            "validated preview intent descriptor; no game-runtime command execution",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_revision_changes_and_evidence_invalidates_on_mutations() {
        let mut state =
            SelfAuthoringWorkspaceState::from_checked_in_fixtures().expect("fixtures should load");
        let theme = ThemeTokens::default();
        let library = editor_shell::editor_design_system_recipe_library();

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("initial evidence capture should validate");
        let before_revision = state
            .selected_source_revision()
            .expect("selected document should expose source revision");
        assert_eq!(state.last_scenario_evidence_packets().len(), 2);

        state
            .rename_selected("toolbar source revision test")
            .expect("rename should mutate selected document");
        let after_rename = state
            .selected_source_revision()
            .expect("renamed document should expose source revision");
        assert_ne!(before_revision, after_rename);
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture after rename should validate");
        state
            .insert_selected_ui_recipe(
                &library,
                ui_definition::UiRecipeId::new(editor_shell::EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID),
                ui_definition::UiRecipeTargetProfileId::new(
                    editor_shell::UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
                ),
            )
            .expect("recipe insertion should mutate selected document");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture after insert should validate");
        state
            .undo_editor_lab_operation()
            .expect("undo should mutate selected document");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture after undo should validate");
        state
            .redo_editor_lab_operation()
            .expect("redo should mutate selected document");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture before apply should validate");
        state
            .apply_selected()
            .expect("apply should mutate applied source state");
        assert!(state.last_scenario_evidence_packets().is_empty());

        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture before rollback should validate");
        state
            .rollback_selected()
            .expect("rollback should mutate applied source state");
        assert!(state.last_scenario_evidence_packets().is_empty());

        let saved = state
            .save_project_package_to_ron()
            .expect("project package should save");
        state
            .capture_pm005_scenario_evidence_packets(&theme)
            .expect("evidence capture before project load should validate");
        state
            .load_project_package_from_ron(&saved)
            .expect("project package load should mutate session source state");
        assert!(state.last_scenario_evidence_packets().is_empty());
    }
}
