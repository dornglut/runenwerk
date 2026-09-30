use super::{
    RenderCaptureIdentity, RenderCapturePointIdentity, RenderCaptureSelectorResult,
    RenderCaptureTerminalReason, RenderPixelCoordinate, RenderPixelProbeAssertionMode,
    RenderPixelSampleMode, RenderTextureDiffRequest, ResolvedRenderCapturePlan,
    validate_selector_terminal_invariant,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderPixelProbeStatus {
    Sampled,
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderPixelProbeResult {
    pub probe_id: String,
    pub capture_point_identity: RenderCapturePointIdentity,
    pub frame_identity: Option<RenderCaptureIdentity>,
    pub sample_mode: RenderPixelSampleMode,
    pub resolved_coordinate: Option<RenderPixelCoordinate>,
    pub comparison_mode: RenderPixelProbeAssertionMode,
    pub sampled_rgba8: Option<[u8; 4]>,
    pub compared_rgba8: Option<[u8; 4]>,
    pub status: RenderPixelProbeStatus,
    pub message: Option<RenderCaptureTerminalReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderTextureDiffStatus {
    Compared,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderTextureDiffMetrics {
    pub total_pixel_count: u64,
    pub changed_pixel_count: u64,
    pub changed_pixel_ratio: f32,
    pub max_delta: u8,
    pub mean_delta: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTextureDiffMismatchSample {
    pub coordinate: RenderPixelCoordinate,
    pub left_rgba8: [u8; 4],
    pub right_rgba8: [u8; 4],
    pub max_channel_delta: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTextureDiffResult {
    pub diff_id: String,
    pub request: RenderTextureDiffRequest,
    pub left_capture_point: RenderCapturePointIdentity,
    pub right_capture_point: RenderCapturePointIdentity,
    pub left_frame_identity: Option<RenderCaptureIdentity>,
    pub right_frame_identity: Option<RenderCaptureIdentity>,
    pub status: RenderTextureDiffStatus,
    pub metrics: Option<RenderTextureDiffMetrics>,
    pub mismatch_samples: Vec<RenderTextureDiffMismatchSample>,
    pub diff_image_path: Option<PathBuf>,
    pub message: Option<RenderCaptureTerminalReason>,
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct RenderDebugFrameReportState {
    pub latest: Option<RenderDebugFrameReport>,
    capture_results_by_frame: BTreeMap<u64, (Vec<RenderCaptureSelectorResult>, Option<PathBuf>)>,
    texture_diff_results_by_frame: BTreeMap<u64, Vec<RenderTextureDiffResult>>,
}

impl RenderDebugFrameReportState {
    pub fn observe_frame(&mut self, report: RenderDebugFrameReport) {
        if !report.texture_diff_results.is_empty() {
            self.texture_diff_results_by_frame
                .insert(report.frame_index, report.texture_diff_results.clone());
            while self.texture_diff_results_by_frame.len()
                > super::DEFAULT_RENDER_FRAME_HISTORY_CAPACITY
            {
                let Some(oldest) = self.texture_diff_results_by_frame.keys().next().copied() else {
                    break;
                };
                self.texture_diff_results_by_frame.remove(&oldest);
            }
        }
        if !report.capture_results.is_empty() {
            self.capture_results_by_frame.insert(
                report.frame_index,
                (
                    report.capture_results.clone(),
                    report.artifact_manifest_path.clone(),
                ),
            );
            while self.capture_results_by_frame.len() > super::DEFAULT_RENDER_FRAME_HISTORY_CAPACITY
            {
                let Some(oldest) = self.capture_results_by_frame.keys().next().copied() else {
                    break;
                };
                self.capture_results_by_frame.remove(&oldest);
            }
        }
        self.latest = Some(report);
    }

    pub fn capture_results_for_frame(
        &self,
        frame_index: u64,
    ) -> Option<&[RenderCaptureSelectorResult]> {
        self.capture_results_by_frame
            .get(&frame_index)
            .map(|(results, _)| results.as_slice())
    }

    pub fn capture_artifact_manifest_for_frame(&self, frame_index: u64) -> Option<&PathBuf> {
        self.capture_results_by_frame
            .get(&frame_index)
            .and_then(|(_, manifest)| manifest.as_ref())
    }

    pub fn texture_diff_results_for_frame(
        &self,
        frame_index: u64,
    ) -> Option<&[RenderTextureDiffResult]> {
        self.texture_diff_results_by_frame
            .get(&frame_index)
            .map(Vec::as_slice)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderDebugFrameReport {
    pub frame_index: u64,
    pub provenance: Vec<crate::plugins::render::inspect::RenderPassProvenanceRecord>,
    pub capture_plan: ResolvedRenderCapturePlan,
    pub capture_results: Vec<RenderCaptureSelectorResult>,
    pub artifact_manifest_path: Option<PathBuf>,
    pub pixel_probe_results: Vec<RenderPixelProbeResult>,
    pub texture_diff_results: Vec<RenderTextureDiffResult>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl RenderDebugFrameReport {
    pub fn validate_invariants(&self) -> Vec<String> {
        match validate_selector_terminal_invariant(&self.capture_plan, &self.capture_results) {
            Ok(()) => Vec::new(),
            Err(violations) => violations
                .into_iter()
                .map(|violation| {
                    format!(
                        "selector invariant violation at index {}: {}",
                        violation.selector_index, violation.message
                    )
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::inspect::{
        CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderCaptureTerminal,
    };

    fn selector_result(
        selector_index: usize,
        selector: RenderCaptureSelector,
    ) -> RenderCaptureSelectorResult {
        RenderCaptureSelectorResult {
            selector_index,
            capture_point: selector.stable_point_fallback(),
            selector,
            frame_identity: None,
            terminal: RenderCaptureTerminal::completed(),
            artifact_path: None,
        }
    }

    #[test]
    fn frame_report_state_keeps_capture_history_after_capture_free_latest_report() {
        let selector = RenderCaptureSelector {
            flow_id: Some("flow".to_string()),
            pass_id: Some("pass".to_string()),
            stage: CaptureStage::After,
            resource_id: "surface.color".to_string(),
            texture_class: CaptureTextureClass::ImportedTexture,
        };
        let manifest = PathBuf::from("captures/frame-1.json");
        let mut state = RenderDebugFrameReportState::default();

        state.observe_frame(RenderDebugFrameReport {
            frame_index: 1,
            capture_results: vec![selector_result(0, selector)],
            artifact_manifest_path: Some(manifest.clone()),
            ..RenderDebugFrameReport::default()
        });
        state.observe_frame(RenderDebugFrameReport {
            frame_index: 2,
            ..RenderDebugFrameReport::default()
        });

        let latest = state
            .latest
            .as_ref()
            .expect("latest report should be present");
        assert_eq!(latest.frame_index, 2);
        assert!(latest.capture_results.is_empty());
        assert_eq!(state.capture_results_for_frame(1).map(<[_]>::len), Some(1));
        assert_eq!(state.capture_results_for_frame(2), None);
        assert_eq!(
            state.capture_artifact_manifest_for_frame(1),
            Some(&manifest)
        );
    }

    #[test]
    fn frame_report_state_keeps_diff_results_after_a_later_report() {
        let selector = RenderCaptureSelector {
            flow_id: Some("flow".to_string()),
            pass_id: Some("pass".to_string()),
            stage: CaptureStage::After,
            resource_id: "display".to_string(),
            texture_class: CaptureTextureClass::ColorTarget,
        };
        let request =
            RenderTextureDiffRequest::new("comparison", selector.clone(), selector.clone());
        let point = selector.stable_point_fallback();
        let diff = RenderTextureDiffResult {
            diff_id: "comparison".to_string(),
            request,
            left_capture_point: point.clone(),
            right_capture_point: point,
            left_frame_identity: None,
            right_frame_identity: None,
            status: RenderTextureDiffStatus::Compared,
            metrics: Some(RenderTextureDiffMetrics {
                total_pixel_count: 4,
                changed_pixel_count: 1,
                changed_pixel_ratio: 0.25,
                max_delta: 1,
                mean_delta: 0.25,
            }),
            mismatch_samples: Vec::new(),
            diff_image_path: None,
            message: None,
        };
        let mut state = RenderDebugFrameReportState::default();
        state.observe_frame(RenderDebugFrameReport {
            frame_index: 5,
            texture_diff_results: vec![diff.clone()],
            ..RenderDebugFrameReport::default()
        });
        state.observe_frame(RenderDebugFrameReport {
            frame_index: 6,
            ..RenderDebugFrameReport::default()
        });
        assert_eq!(
            state.texture_diff_results_for_frame(5),
            Some([diff].as_slice())
        );
        assert_eq!(state.texture_diff_results_for_frame(6), None);
        assert_eq!(
            state.latest.as_ref().map(|report| report.frame_index),
            Some(6)
        );
    }
}
