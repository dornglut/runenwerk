use super::{
    RenderCaptureIdentity, RenderCapturePointIdentity, RenderCaptureSelector,
    RenderCaptureTerminal, RenderCaptureTerminalCode, RenderCaptureTerminalReason,
};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderSelectorResolution {
    Matched {
        capture_point: RenderCapturePointIdentity,
        frame_identity: RenderCaptureIdentity,
    },
    Unmatched {
        reason: RenderCaptureTerminalReason,
    },
    Disabled {
        reason: RenderCaptureTerminalReason,
    },
    Unsupported {
        reason: RenderCaptureTerminalReason,
    },
    Skipped {
        reason: RenderCaptureTerminalReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRenderCaptureSelector {
    pub selector_index: usize,
    pub selector: RenderCaptureSelector,
    pub resolution: RenderSelectorResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedRenderCapturePlan {
    pub frame_index: u64,
    pub selectors: Vec<ResolvedRenderCaptureSelector>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCaptureSelectorResult {
    pub selector_index: usize,
    pub selector: RenderCaptureSelector,
    pub capture_point: RenderCapturePointIdentity,
    pub frame_identity: Option<RenderCaptureIdentity>,
    pub terminal: RenderCaptureTerminal,
    pub artifact_path: Option<PathBuf>,
}

impl RenderCaptureSelectorResult {
    pub fn for_unmatched(selector_index: usize, selector: RenderCaptureSelector) -> Self {
        let capture_point = selector.stable_point_fallback();
        Self {
            selector_index,
            selector,
            capture_point,
            frame_identity: None,
            terminal: RenderCaptureTerminal::new(
                RenderCaptureTerminalCode::Unmatched,
                Some(RenderCaptureTerminalReason::new(
                    "selector_unmatched",
                    "selector matched no capture point in this frame",
                )),
            ),
            artifact_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCaptureInvariantViolation {
    pub selector_index: usize,
    pub message: String,
}

pub fn validate_selector_terminal_invariant(
    selectors: &[RenderCaptureSelector],
    results: &[RenderCaptureSelectorResult],
) -> Result<(), Vec<RenderCaptureInvariantViolation>> {
    let mut counts = vec![0usize; selectors.len()];
    let mut violations = Vec::<RenderCaptureInvariantViolation>::new();

    for result in results {
        if result.selector_index >= selectors.len() {
            violations.push(RenderCaptureInvariantViolation {
                selector_index: result.selector_index,
                message: format!(
                    "selector result index {} is out of bounds (selectors={})",
                    result.selector_index,
                    selectors.len()
                ),
            });
            continue;
        }
        counts[result.selector_index] = counts[result.selector_index].saturating_add(1);
    }

    for (selector_index, count) in counts.into_iter().enumerate() {
        if count == 0 {
            violations.push(RenderCaptureInvariantViolation {
                selector_index,
                message: format!(
                    "selector {} has no terminal outcome (silent drop)",
                    selectors[selector_index].describe()
                ),
            });
        } else if count > 1 {
            violations.push(RenderCaptureInvariantViolation {
                selector_index,
                message: format!(
                    "selector {} has {} terminal outcomes",
                    selectors[selector_index].describe(),
                    count
                ),
            });
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::inspect::{
        CaptureStage, CaptureTextureClass, RenderCaptureIdentity, RenderCaptureSelector,
        RenderCaptureTerminal,
    };

    fn selector(pass_id: &str) -> RenderCaptureSelector {
        RenderCaptureSelector {
            flow_id: Some("flow.main".to_string()),
            pass_id: Some(pass_id.to_string()),
            stage: CaptureStage::After,
            resource_id: "surface.color".to_string(),
            texture_class: CaptureTextureClass::ImportedTexture,
        }
    }

    fn completed_result(
        selector_index: usize,
        selector: RenderCaptureSelector,
    ) -> RenderCaptureSelectorResult {
        let capture_point = selector.stable_point_fallback();
        RenderCaptureSelectorResult {
            selector_index,
            selector,
            frame_identity: Some(RenderCaptureIdentity {
                frame_index: 7,
                pass_label: "pass.label".to_string(),
                capture_point: capture_point.clone(),
            }),
            capture_point,
            terminal: RenderCaptureTerminal::completed(),
            artifact_path: None,
        }
    }

    #[test]
    fn selector_terminal_invariant_accepts_exactly_one_terminal_per_selector() {
        let selectors = vec![selector("pass.a"), selector("pass.b")];
        let results = vec![
            completed_result(0, selectors[0].clone()),
            completed_result(1, selectors[1].clone()),
        ];

        assert!(validate_selector_terminal_invariant(&selectors, &results).is_ok());
    }

    #[test]
    fn selector_terminal_invariant_rejects_missing_and_duplicate_terminals() {
        let selectors = vec![selector("pass.a"), selector("pass.b")];
        let results = vec![
            completed_result(0, selectors[0].clone()),
            completed_result(0, selectors[0].clone()),
        ];

        let violations = validate_selector_terminal_invariant(&selectors, &results)
            .expect_err("duplicate + missing selector outcomes should violate invariant");
        assert!(
            violations
                .iter()
                .any(|value| value.message.contains("has 2 terminal outcomes"))
        );
        assert!(
            violations
                .iter()
                .any(|value| value.message.contains("no terminal outcome"))
        );
    }
}
