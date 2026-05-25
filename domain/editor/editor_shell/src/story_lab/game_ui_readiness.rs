//! File: domain/editor/editor_shell/src/story_lab/game_ui_readiness.rs
//! Purpose: Editor Story Lab evidence for future game-runtime UI compatibility seams.

use std::collections::BTreeSet;

use ui_definition::{
    UiReadinessCompatibilityAxis, UiReadinessEvidenceKind, game_runtime_required_compatibility_axes,
};

pub const GAME_RUNTIME_TARGET_PROFILE: &str = "game.runtime";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxGameUiReadinessEvidence {
    pub target_profile: &'static str,
    pub compatibility_axes: BTreeSet<UiReadinessCompatibilityAxis>,
    pub required_evidence: BTreeSet<UiReadinessEvidenceKind>,
    pub generic_validator_checks: BTreeSet<&'static str>,
    pub runtime_contract_terms: BTreeSet<&'static str>,
    pub forbidden_runtime_terms: BTreeSet<&'static str>,
    pub implements_game_hud_behavior: bool,
}

impl EditorUxGameUiReadinessEvidence {
    pub fn covers_required_axes(&self) -> bool {
        game_runtime_required_compatibility_axes().is_subset(&self.compatibility_axes)
    }

    pub fn runtime_contract_is_editor_free(&self) -> bool {
        self.runtime_contract_terms
            .is_disjoint(&self.forbidden_runtime_terms)
    }
}

pub fn game_ui_readiness_evidence() -> EditorUxGameUiReadinessEvidence {
    EditorUxGameUiReadinessEvidence {
        target_profile: GAME_RUNTIME_TARGET_PROFILE,
        compatibility_axes: game_runtime_required_compatibility_axes(),
        required_evidence: BTreeSet::from([
            UiReadinessEvidenceKind::CompatibilityReport,
            UiReadinessEvidenceKind::DiagnosticInspection,
            UiReadinessEvidenceKind::AccessibilityReport,
            UiReadinessEvidenceKind::PerformanceBudgetReport,
            UiReadinessEvidenceKind::ExampleScenario,
        ]),
        generic_validator_checks: BTreeSet::from([
            "ui.preview.matrix.game_runtime_axis_coverage_missing",
            "ui.readiness.compatibility_axis.missing",
            "ui.readiness.inspection.compatibility_axis.missing",
            "ui.binding.intent.descriptor.editor_command_for_runtime",
            "ui.binding.view_model_package.missing",
            "ui.binding.view_model_package.stale",
            "ui.persistence.activation.migration_report_missing",
            "ui.persistence.activation.diff_missing",
            "ui.visual_layout.target_profile.unsupported",
        ]),
        runtime_contract_terms: BTreeSet::from([
            "game.runtime",
            "safe_area",
            "input_modality",
            "platform_prompt",
            "localization_text_expansion",
            "accessibility",
            "split_screen_size",
            "performance_readability",
            "view_model_freshness",
            "game_intent",
            "external_evidence_reference",
        ]),
        forbidden_runtime_terms: BTreeSet::from([
            "editor_shell",
            "workbench_host",
            "editor_command",
            "editor_provider",
            "editor_surface",
            "provider_route",
        ]),
        implements_game_hud_behavior: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_ui_readiness_evidence_covers_required_axes() {
        let evidence = game_ui_readiness_evidence();

        assert_eq!(evidence.target_profile, GAME_RUNTIME_TARGET_PROFILE);
        assert!(evidence.covers_required_axes());
        assert!(
            evidence
                .required_evidence
                .contains(&UiReadinessEvidenceKind::CompatibilityReport)
        );
        assert!(
            evidence
                .generic_validator_checks
                .contains("ui.binding.intent.descriptor.editor_command_for_runtime")
        );
    }

    #[test]
    fn game_ui_readiness_evidence_contains_no_editor_runtime_vocabulary() {
        let evidence = game_ui_readiness_evidence();

        assert!(evidence.runtime_contract_is_editor_free());
        assert!(!evidence.implements_game_hud_behavior);
    }
}
