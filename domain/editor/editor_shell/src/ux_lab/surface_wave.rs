//! File: domain/editor/editor_shell/src/ux_lab/surface_wave.rs
//! Purpose: Registered editor surface readiness evidence for the Editor UX Lab.

use crate::ToolSurfaceReadiness;
use ui_surface::{SurfaceDefinition, SurfaceDefinitionId};

pub const REGISTERED_SURFACE_TARGET_PROFILE: &str = "editor.surface.registry";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxRegisteredSurfaceEvidence {
    pub target_profile: &'static str,
    pub surface_definition_id: SurfaceDefinitionId,
    pub semantic_key: &'static str,
    pub display_name: &'static str,
    pub readiness: ToolSurfaceReadiness,
    pub visible_in_product: bool,
    pub required_artifact_kinds: Vec<&'static str>,
    pub required_state_kinds: Vec<&'static str>,
    pub required_route_kinds: Vec<&'static str>,
    pub provider_native_checks: Vec<&'static str>,
    pub readiness_reason: &'static str,
}

pub fn registered_surface_evidence(
    definition: SurfaceDefinition,
    readiness: ToolSurfaceReadiness,
) -> EditorUxRegisteredSurfaceEvidence {
    let contract = evidence_contract_for_readiness(readiness);
    EditorUxRegisteredSurfaceEvidence {
        target_profile: REGISTERED_SURFACE_TARGET_PROFILE,
        surface_definition_id: definition.id,
        semantic_key: definition.semantic_key,
        display_name: definition.display_name,
        readiness,
        visible_in_product: readiness.visible_in_product(),
        required_artifact_kinds: contract.required_artifact_kinds,
        required_state_kinds: contract.required_state_kinds,
        required_route_kinds: contract.required_route_kinds,
        provider_native_checks: contract.provider_native_checks,
        readiness_reason: contract.readiness_reason,
    }
}

struct SurfaceReadinessEvidenceContract {
    required_artifact_kinds: Vec<&'static str>,
    required_state_kinds: Vec<&'static str>,
    required_route_kinds: Vec<&'static str>,
    provider_native_checks: Vec<&'static str>,
    readiness_reason: &'static str,
}

fn evidence_contract_for_readiness(
    readiness: ToolSurfaceReadiness,
) -> SurfaceReadinessEvidenceContract {
    match readiness {
        ToolSurfaceReadiness::Product => SurfaceReadinessEvidenceContract {
            required_artifact_kinds: vec![
                "RetainedUiDebug",
                "SurfaceReadinessReport",
                "PlatformImpossibleReport",
                "FocusTraversalReport",
                "AccessibilityReport",
                "DiagnosticsSnapshot",
                "TimingReport",
            ],
            required_state_kinds: vec!["default", "focused", "selected", "overflow", "native"],
            required_route_kinds: vec!["capture_surface", "focus_surface", "provider_frame"],
            provider_native_checks: vec![
                "visible_widget_scan",
                "native_or_platform_capture",
                "focus_traversal",
                "accessibility_report",
                "diagnostics_snapshot",
                "timing_report",
                "provider_runtime_frame",
            ],
            readiness_reason: "certified_product_surface_requires_native_manifest_evidence",
        },
        ToolSurfaceReadiness::FallbackOnly => SurfaceReadinessEvidenceContract {
            required_artifact_kinds: vec![
                "RetainedUiDebug",
                "SurfaceReadinessReport",
                "DiagnosticsSnapshot",
            ],
            required_state_kinds: vec!["default", "fallback"],
            required_route_kinds: vec!["capture_fallback", "fallback_reason"],
            provider_native_checks: vec![
                "visible_widget_scan",
                "fallback_reason",
                "not_product_certified",
            ],
            readiness_reason: "fallback_only_surface_must_be_explicitly_non_product",
        },
        ToolSurfaceReadiness::Diagnostic => SurfaceReadinessEvidenceContract {
            required_artifact_kinds: vec![
                "RetainedUiDebug",
                "SurfaceReadinessReport",
                "DiagnosticsSnapshot",
            ],
            required_state_kinds: vec!["default", "diagnostic", "warning", "error"],
            required_route_kinds: vec!["capture_diagnostic", "diagnostic_snapshot"],
            provider_native_checks: vec![
                "visible_widget_scan",
                "diagnostic_reason",
                "diagnostics_snapshot",
                "not_product_certified",
            ],
            readiness_reason: "diagnostic_surface_must_prove_diagnostic_honesty",
        },
        ToolSurfaceReadiness::HiddenUntilProductized => SurfaceReadinessEvidenceContract {
            required_artifact_kinds: vec![
                "RetainedUiDebug",
                "SurfaceReadinessReport",
                "UnsupportedCheckReport",
            ],
            required_state_kinds: vec!["hidden", "registered"],
            required_route_kinds: vec!["hidden_registration_check"],
            provider_native_checks: vec![
                "registered_surface_hidden",
                "not_normal_product_workflow",
                "not_product_certified",
            ],
            readiness_reason: "hidden_surface_remains_registered_until_productized",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{editor_surface_definitions, tool_surface_readiness_for_definition_id};
    use std::collections::BTreeSet;

    #[test]
    fn registered_surface_evidence_covers_all_readiness_categories() {
        let evidence = editor_surface_definitions()
            .into_iter()
            .map(|definition| {
                registered_surface_evidence(
                    definition,
                    tool_surface_readiness_for_definition_id(definition.id),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(evidence.len(), editor_surface_definitions().len());
        let readiness = evidence
            .iter()
            .map(|item| item.readiness)
            .collect::<BTreeSet<_>>();
        assert!(readiness.contains(&ToolSurfaceReadiness::Product));
        assert!(readiness.contains(&ToolSurfaceReadiness::FallbackOnly));
        assert!(readiness.contains(&ToolSurfaceReadiness::Diagnostic));
        assert!(readiness.contains(&ToolSurfaceReadiness::HiddenUntilProductized));
        assert!(
            evidence
                .iter()
                .all(|item| item.target_profile == REGISTERED_SURFACE_TARGET_PROFILE)
        );
        assert!(evidence.iter().all(|item| {
            item.required_artifact_kinds
                .contains(&"SurfaceReadinessReport")
        }));
    }
}
