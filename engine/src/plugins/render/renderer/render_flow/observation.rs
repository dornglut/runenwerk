#![allow(clippy::items_after_test_module)]

use super::*;
use crate::plugins::gpu::{
    GpuContext, GpuContextAffinity, GpuExecutionLifecycleState, GpuReadbackId, GpuReadbackStatus,
    GpuSubmission, GpuSubmissionFailure,
};

#[derive(Debug, Default)]
pub(in crate::plugins::render::renderer) struct RendererGpuObservationOutput {
    pub timing_evidence: Vec<RenderPassTimingEvidence>,
    pub captured_textures: Vec<RenderCapturedTexture>,
    pub capture_results: Vec<RenderCaptureSelectorResult>,
}

#[derive(Debug, Default)]
pub(in crate::plugins::render::renderer) struct RendererGpuObservationState {
    accepted: Vec<AcceptedRendererObservation>,
}

#[derive(Debug)]
struct AcceptedRendererObservation {
    submission: GpuSubmission,
    timings: Vec<GpuPassTimingFrame>,
    captures: Vec<CaptureObservation>,
}

#[derive(Debug)]
struct CaptureObservation {
    readback_id: GpuReadbackId,
    selector_index: usize,
    selector: RenderCaptureSelector,
    identity: RenderCaptureIdentity,
    width: u32,
    height: u32,
    source_format: TextureFormat,
    readback_format: TextureReadbackFormat,
}

impl CaptureObservation {
    fn from_prepared(prepared: &PreparedCaptureReadback) -> Self {
        Self {
            readback_id: prepared.canonical_operation().id(),
            selector_index: prepared.selector_index,
            selector: prepared.selector.clone(),
            identity: prepared.identity.clone(),
            width: prepared.width,
            height: prepared.height,
            source_format: prepared.source_format,
            readback_format: texture_readback_format(prepared.source_format)
                .expect("prepared capture already admitted its renderer readback format"),
        }
    }

    fn ready(&self, readback: &crate::plugins::gpu::GpuReadbackBytes) -> RenderCapturedTexture {
        let expected_format =
            crate::plugins::render::renderer::resource_descriptors::gpu_texture_format(
                self.source_format,
            );
        let Ok(expected_format) = expected_format else {
            return self.failed(
                "capture_format_unmapped",
                "capture source format has no admitted RunenGPU readback mapping",
            );
        };
        if readback.texture_format() != Some(expected_format) {
            return self.failed(
                "capture_format_mismatch",
                format!(
                    "capture readback format mismatch: expected {expected_format:?}, got {:?}",
                    readback.texture_format()
                ),
            );
        }
        let expected_len = u64::from(self.width)
            .checked_mul(u64::from(self.height))
            .and_then(|pixels| pixels.checked_mul(4))
            .and_then(|bytes| usize::try_from(bytes).ok());
        let Some(expected_len) = expected_len else {
            return self.failed(
                "capture_byte_length_overflow",
                "capture dimensions exceed the renderer byte-length domain",
            );
        };
        if readback.as_bytes().len() != expected_len {
            return self.failed(
                "capture_byte_length_mismatch",
                format!(
                    "capture readback byte length mismatch: expected {expected_len}, got {}",
                    readback.as_bytes().len()
                ),
            );
        }

        let mut bytes_rgba8 = readback.as_bytes().to_vec();
        if self.readback_format.mode == TextureReadbackMode::Bgra8 {
            for pixel in bytes_rgba8.as_chunks_mut::<4>().0 {
                pixel.swap(0, 2);
            }
        }
        RenderCapturedTexture {
            identity: self.identity.clone(),
            width: self.width,
            height: self.height,
            format: format!("{:?}", self.source_format),
            bytes_rgba8: Some(bytes_rgba8),
            terminal: RenderCaptureTerminal::completed(),
        }
    }

    fn failed(
        &self,
        reason_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> RenderCapturedTexture {
        RenderCapturedTexture {
            identity: self.identity.clone(),
            width: self.width,
            height: self.height,
            format: format!("{:?}", self.source_format),
            bytes_rgba8: None,
            terminal: RenderCaptureTerminal::with_reason(
                RenderCaptureTerminalCode::ReadbackFailed,
                reason_code,
                detail,
            ),
        }
    }

    fn result(&self, capture: &RenderCapturedTexture) -> RenderCaptureSelectorResult {
        RenderCaptureSelectorResult {
            selector_index: self.selector_index,
            selector: self.selector.clone(),
            capture_point: self.identity.capture_point.clone(),
            frame_identity: Some(self.identity.clone()),
            terminal: capture.terminal.clone(),
            artifact_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuContextAffinity, GpuContextId, GpuDataLayout, GpuDeviceGeneration, GpuReadback,
        GpuReadbackBytes, GpuResourceLabel, GpuResourceProvenance, GpuSubmissionFailureKind,
        GpuSubmissionId, GpuSubmissionStatus, GpuTextureFormat,
    };
    use std::num::NonZeroU64;
    use std::sync::{Arc, Mutex};

    type SubmissionFixture = (
        GpuSubmission,
        Arc<Mutex<GpuSubmissionStatus>>,
        Vec<Arc<Mutex<GpuReadbackStatus>>>,
    );

    fn affinity() -> GpuContextAffinity {
        GpuContextAffinity::test_value(
            GpuContextId::test_value(NonZeroU64::new(11).unwrap()),
            GpuDeviceGeneration::test_value(NonZeroU64::new(7).unwrap()),
        )
    }

    fn submission(
        id: u64,
        submission_status: GpuSubmissionStatus,
        readbacks: Vec<(GpuReadbackId, GpuReadbackStatus)>,
    ) -> SubmissionFixture {
        let submission_status = Arc::new(Mutex::new(submission_status));
        let mut status_handles = Vec::with_capacity(readbacks.len());
        let readbacks = readbacks
            .into_iter()
            .map(|(readback_id, status)| {
                let status = Arc::new(Mutex::new(status));
                status_handles.push(Arc::clone(&status));
                GpuReadback::new(readback_id, status)
            })
            .collect();
        (
            GpuSubmission::new(
                GpuSubmissionId::from_nonzero(NonZeroU64::new(id).unwrap()),
                affinity(),
                Arc::clone(&submission_status),
                readbacks,
            ),
            submission_status,
            status_handles,
        )
    }

    fn readback_bytes(
        bytes: Vec<u8>,
        texture_format: Option<GpuTextureFormat>,
    ) -> GpuReadbackBytes {
        let byte_len = u64::try_from(bytes.len()).unwrap();
        GpuReadbackBytes::from_normalized_bytes(
            "renderer observation test",
            bytes,
            GpuDataLayout::new("renderer observation test", byte_len, 1, 1, byte_len).unwrap(),
            texture_format,
            GpuResourceProvenance::new(
                GpuResourceLabel::new("renderer observation test").unwrap(),
                None,
                None,
            ),
        )
        .unwrap()
    }

    fn timing(readback_id: GpuReadbackId, frame_index: u64) -> GpuPassTimingFrame {
        GpuPassTimingFrame::for_test(
            readback_id,
            2.0,
            2,
            [(
                GpuPassTimestampIndices { begin: 0, end: 1 },
                frame_index,
                91,
                "original.flow",
                "original.pass",
                "compute",
            )],
        )
    }

    fn selector(frame_index: u64) -> (RenderCaptureSelector, RenderCaptureIdentity) {
        let selector = RenderCaptureSelector {
            flow_id: Some("original.flow".to_string()),
            pass_id: Some("original.pass".to_string()),
            stage: CaptureStage::After,
            resource_id: "surface.color".to_string(),
            texture_class: CaptureTextureClass::ColorTarget,
        };
        let identity = RenderCaptureIdentity {
            frame_index,
            pass_label: "Original Pass".to_string(),
            capture_point: RenderCapturePointIdentity {
                flow_id: "original.flow".to_string(),
                pass_id: "original.pass".to_string(),
                stage: CaptureStage::After,
                resource_id: "surface.color".to_string(),
                texture_class: CaptureTextureClass::ColorTarget,
            },
        };
        (selector, identity)
    }

    fn capture(
        readback_id: GpuReadbackId,
        frame_index: u64,
        source_format: TextureFormat,
    ) -> CaptureObservation {
        let (selector, identity) = selector(frame_index);
        CaptureObservation {
            readback_id,
            selector_index: 0,
            selector,
            identity,
            width: 1,
            height: 1,
            source_format,
            readback_format: texture_readback_format(source_format).unwrap(),
        }
    }

    fn accepted(
        submission: GpuSubmission,
        timings: Vec<GpuPassTimingFrame>,
        captures: Vec<CaptureObservation>,
    ) -> AcceptedRendererObservation {
        AcceptedRendererObservation {
            submission,
            timings,
            captures,
        }
    }

    #[test]
    fn completed_submission_can_keep_timing_pending_then_ready_with_original_identity_once() {
        let readback_id = GpuReadbackId::allocate().unwrap();
        let (submission, _submission_status, statuses) = submission(
            1,
            GpuSubmissionStatus::Completed,
            vec![(readback_id, GpuReadbackStatus::Pending)],
        );
        let mut state = RendererGpuObservationState {
            accepted: vec![accepted(submission, vec![timing(readback_id, 41)], vec![])],
        };

        let pending = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert!(pending.timing_evidence.is_empty());
        assert_eq!(state.retained_submission_count(), 1);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&100_u64.to_le_bytes());
        bytes.extend_from_slice(&600_u64.to_le_bytes());
        *statuses[0].lock().unwrap() = GpuReadbackStatus::Ready(readback_bytes(bytes, None));
        let ready = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert_eq!(ready.timing_evidence.len(), 1);
        let evidence = &ready.timing_evidence[0];
        assert_eq!(evidence.frame_index, Some(41));
        assert_eq!(evidence.render_surface_id, Some(91));
        assert_eq!(evidence.flow_id, "original.flow");
        assert_eq!(evidence.pass_id, "original.pass");
        assert_eq!(evidence.millis, Some(0.001));
        assert_eq!(state.retained_submission_count(), 0);

        let repeated = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert!(repeated.timing_evidence.is_empty());
    }

    #[test]
    fn capture_pending_then_ready_preserves_original_identity_and_rgba_once() {
        let readback_id = GpuReadbackId::allocate().unwrap();
        let (submission, _, statuses) = submission(
            2,
            GpuSubmissionStatus::Accepted,
            vec![(readback_id, GpuReadbackStatus::Pending)],
        );
        let mut state = RendererGpuObservationState {
            accepted: vec![accepted(
                submission,
                vec![],
                vec![capture(readback_id, 52, TextureFormat::Rgba8Unorm)],
            )],
        };
        let pending = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert!(pending.captured_textures.is_empty());

        *statuses[0].lock().unwrap() = GpuReadbackStatus::Ready(readback_bytes(
            vec![1, 2, 3, 4],
            Some(GpuTextureFormat::Rgba8Unorm),
        ));
        let ready = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert_eq!(ready.captured_textures.len(), 1);
        assert_eq!(ready.capture_results.len(), 1);
        assert_eq!(ready.captured_textures[0].identity.frame_index, 52);
        assert_eq!(
            ready.captured_textures[0].bytes_rgba8,
            Some(vec![1, 2, 3, 4])
        );
        assert_eq!(
            ready.capture_results[0]
                .frame_identity
                .as_ref()
                .unwrap()
                .frame_index,
            52
        );
        assert_eq!(state.retained_submission_count(), 0);
        assert!(
            state
                .progress_with_context(affinity(), GpuExecutionLifecycleState::Running)
                .captured_textures
                .is_empty()
        );
    }

    #[test]
    fn failed_timing_and_capture_readbacks_emit_truthful_terminal_evidence() {
        let timing_id = GpuReadbackId::allocate().unwrap();
        let capture_id = GpuReadbackId::allocate().unwrap();
        let failure = GpuSubmissionFailure::new(
            GpuSubmissionFailureKind::ReadbackMapping,
            "mapping failed in test",
        );
        let (submission, _, _) = submission(
            3,
            GpuSubmissionStatus::Failed(failure.clone()),
            vec![
                (timing_id, GpuReadbackStatus::Failed(failure.clone())),
                (capture_id, GpuReadbackStatus::Failed(failure)),
            ],
        );
        let mut state = RendererGpuObservationState {
            accepted: vec![accepted(
                submission,
                vec![timing(timing_id, 61)],
                vec![capture(capture_id, 61, TextureFormat::Rgba8Unorm)],
            )],
        };
        let output = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert_eq!(output.timing_evidence.len(), 1);
        assert!(output.timing_evidence[0].millis.is_none());
        assert!(
            output.timing_evidence[0].diagnostics[0]
                .message
                .contains("mapping failed in test")
        );
        assert_eq!(
            output.captured_textures[0].terminal.code,
            RenderCaptureTerminalCode::ReadbackFailed
        );
        assert!(output.captured_textures[0].bytes_rgba8.is_none());
        assert_eq!(state.retained_submission_count(), 0);
    }

    #[test]
    fn multiple_in_flight_submissions_progress_independently() {
        let first_id = GpuReadbackId::allocate().unwrap();
        let second_id = GpuReadbackId::allocate().unwrap();
        let (first, _, first_status) = submission(
            4,
            GpuSubmissionStatus::Completed,
            vec![(first_id, GpuReadbackStatus::Pending)],
        );
        let (second, _, second_status) = submission(
            5,
            GpuSubmissionStatus::Completed,
            vec![(second_id, GpuReadbackStatus::Pending)],
        );
        let mut state = RendererGpuObservationState {
            accepted: vec![
                accepted(
                    first,
                    vec![],
                    vec![capture(first_id, 70, TextureFormat::Rgba8Unorm)],
                ),
                accepted(
                    second,
                    vec![],
                    vec![capture(second_id, 71, TextureFormat::Rgba8Unorm)],
                ),
            ],
        };
        *first_status[0].lock().unwrap() = GpuReadbackStatus::Ready(readback_bytes(
            vec![8, 7, 6, 5],
            Some(GpuTextureFormat::Rgba8Unorm),
        ));
        let first_output =
            state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert_eq!(first_output.captured_textures.len(), 1);
        assert_eq!(first_output.captured_textures[0].identity.frame_index, 70);
        assert_eq!(state.retained_submission_count(), 1);

        let failure = GpuSubmissionFailure::new(
            GpuSubmissionFailureKind::ContextOrDeviceUnavailableOrLost,
            "device lost in test",
        );
        *second_status[0].lock().unwrap() = GpuReadbackStatus::Failed(failure);
        let second_output =
            state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert_eq!(second_output.captured_textures.len(), 1);
        assert_eq!(second_output.captured_textures[0].identity.frame_index, 71);
        assert_eq!(state.retained_submission_count(), 0);
    }

    #[test]
    fn timing_and_capture_from_one_submission_terminalize_independently() {
        let timing_id = GpuReadbackId::allocate().unwrap();
        let capture_id = GpuReadbackId::allocate().unwrap();
        let mut timing_bytes = Vec::new();
        timing_bytes.extend_from_slice(&10_u64.to_le_bytes());
        timing_bytes.extend_from_slice(&20_u64.to_le_bytes());
        let (submission, _, _) = submission(
            6,
            GpuSubmissionStatus::Completed,
            vec![
                (
                    timing_id,
                    GpuReadbackStatus::Ready(readback_bytes(timing_bytes, None)),
                ),
                (
                    capture_id,
                    GpuReadbackStatus::Ready(readback_bytes(
                        vec![9, 8, 7, 6],
                        Some(GpuTextureFormat::Rgba8Unorm),
                    )),
                ),
            ],
        );
        let mut state = RendererGpuObservationState {
            accepted: vec![accepted(
                submission,
                vec![timing(timing_id, 80)],
                vec![capture(capture_id, 80, TextureFormat::Rgba8Unorm)],
            )],
        };
        let output = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Running);
        assert_eq!(output.timing_evidence.len(), 1);
        assert_eq!(output.captured_textures.len(), 1);
        assert_eq!(state.retained_submission_count(), 0);
    }

    #[test]
    fn malformed_and_reversed_timestamps_never_fabricate_samples() {
        let timing_id = GpuReadbackId::allocate().unwrap();
        let timing = timing(timing_id, 90);
        let malformed = timing.ready_evidence(&readback_bytes(1_u64.to_le_bytes().to_vec(), None));
        assert!(malformed[0].millis.is_none());
        assert!(
            malformed[0].diagnostics[0]
                .message
                .contains("length mismatch")
        );

        let mut reversed_bytes = Vec::new();
        reversed_bytes.extend_from_slice(&50_u64.to_le_bytes());
        reversed_bytes.extend_from_slice(&40_u64.to_le_bytes());
        let reversed = timing.ready_evidence(&readback_bytes(reversed_bytes, None));
        assert!(reversed[0].millis.is_none());
        assert!(reversed[0].diagnostics[0].message.contains("precedes"));
    }

    #[test]
    fn capture_length_mismatch_fails_and_bgra_converts_to_rgba() {
        let capture_id = GpuReadbackId::allocate().unwrap();
        let rgba_capture = capture(capture_id, 100, TextureFormat::Rgba8Unorm);
        let mismatch = rgba_capture.ready(&readback_bytes(
            vec![1, 2, 3],
            Some(GpuTextureFormat::Rgba8Unorm),
        ));
        assert_eq!(
            mismatch.terminal.code,
            RenderCaptureTerminalCode::ReadbackFailed
        );
        assert!(mismatch.bytes_rgba8.is_none());

        let bgra_capture = capture(capture_id, 100, TextureFormat::Bgra8Unorm);
        let converted = bgra_capture.ready(&readback_bytes(
            vec![3, 2, 1, 4],
            Some(GpuTextureFormat::Bgra8Unorm),
        ));
        assert_eq!(
            converted.terminal.code,
            RenderCaptureTerminalCode::Completed
        );
        assert_eq!(converted.bytes_rgba8, Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn closed_context_retires_pending_observations() {
        let timing_id = GpuReadbackId::allocate().unwrap();
        let capture_id = GpuReadbackId::allocate().unwrap();
        let (submission, _, _) = submission(
            7,
            GpuSubmissionStatus::Completed,
            vec![
                (timing_id, GpuReadbackStatus::Pending),
                (capture_id, GpuReadbackStatus::Pending),
            ],
        );
        let mut state = RendererGpuObservationState {
            accepted: vec![accepted(
                submission,
                vec![timing(timing_id, 110)],
                vec![capture(capture_id, 110, TextureFormat::Rgba8Unorm)],
            )],
        };
        let output = state.progress_with_context(affinity(), GpuExecutionLifecycleState::Closed);
        assert_eq!(output.timing_evidence.len(), 1);
        assert_eq!(
            output.captured_textures[0].terminal.code,
            RenderCaptureTerminalCode::ReadbackFailed
        );
        assert_eq!(state.retained_submission_count(), 0);
    }

    #[test]
    fn submission_without_renderer_observed_readbacks_retains_no_state() {
        let (submission, _, _) = submission(8, GpuSubmissionStatus::Accepted, vec![]);
        let mut state = RendererGpuObservationState::default();
        let mut capture_runtime =
            FrameCaptureRuntime::new(120, &RenderDebugControlResource::default(), &[]);
        let output = state.accept_with_bound(submission, vec![], vec![], &mut capture_runtime, 1);
        assert!(output.timing_evidence.is_empty());
        assert!(output.captured_textures.is_empty());
        assert_eq!(state.retained_submission_count(), 0);
    }
}

impl RendererGpuObservationState {
    pub fn accept(
        &mut self,
        context: &GpuContext,
        submission: GpuSubmission,
        timings: Vec<GpuPassTimingFrame>,
        captures: Vec<PreparedCaptureReadback>,
        capture_runtime: &mut FrameCaptureRuntime,
    ) -> RendererGpuObservationOutput {
        self.accept_with_bound(
            submission,
            timings,
            captures,
            capture_runtime,
            context.execution_policy().max_in_flight_submissions().get(),
        )
    }

    fn accept_with_bound(
        &mut self,
        submission: GpuSubmission,
        timings: Vec<GpuPassTimingFrame>,
        captures: Vec<PreparedCaptureReadback>,
        capture_runtime: &mut FrameCaptureRuntime,
        bound: usize,
    ) -> RendererGpuObservationOutput {
        let mut output = RendererGpuObservationOutput::default();
        let mut accepted_timings = Vec::with_capacity(timings.len());
        for timing in timings {
            if submission.readback(timing.readback_id()).is_some() {
                accepted_timings.push(timing);
            } else {
                output.timing_evidence.extend(timing.diagnostic_evidence(
                    "accepted GPU submission omitted the renderer timing readback",
                ));
            }
        }

        let mut accepted_captures = Vec::with_capacity(captures.len());
        for prepared in captures {
            let capture = CaptureObservation::from_prepared(&prepared);
            if submission.readback(capture.readback_id).is_some() {
                capture_runtime.set_readback_pending(capture.selector_index);
                accepted_captures.push(capture);
            } else {
                let terminal = capture.failed(
                    "accepted_capture_readback_missing",
                    "accepted GPU submission omitted the renderer capture readback",
                );
                capture_runtime.set_terminal(capture.selector_index, terminal.terminal.clone());
                output.captured_textures.push(terminal);
            }
        }

        if accepted_timings.is_empty() && accepted_captures.is_empty() {
            return output;
        }

        if self.accepted.len() >= bound {
            let detail = format!(
                "renderer GPU observation bound {bound} was exhausted for accepted submission {}",
                submission.id()
            );
            for timing in accepted_timings {
                output
                    .timing_evidence
                    .extend(timing.diagnostic_evidence(detail.clone()));
            }
            for capture in accepted_captures {
                let terminal = capture.failed("renderer_observation_capacity_exceeded", &detail);
                capture_runtime.set_terminal(capture.selector_index, terminal.terminal.clone());
                output.captured_textures.push(terminal);
            }
            return output;
        }

        for timing in &accepted_timings {
            output.timing_evidence.extend(timing.pending_evidence());
        }
        self.accepted.push(AcceptedRendererObservation {
            submission,
            timings: accepted_timings,
            captures: accepted_captures,
        });
        output
    }

    pub fn progress(&mut self, context: &GpuContext) -> RendererGpuObservationOutput {
        self.progress_with_context(context.affinity(), context.execution_lifecycle_state())
    }

    fn progress_with_context(
        &mut self,
        affinity: GpuContextAffinity,
        lifecycle: GpuExecutionLifecycleState,
    ) -> RendererGpuObservationOutput {
        let mut output = RendererGpuObservationOutput::default();

        for accepted in &mut self.accepted {
            let invalid_context = (accepted.submission.affinity() != affinity)
                .then_some("renderer GPU observation belongs to another context/device generation");

            accepted.timings.retain(|timing| {
                let status = accepted
                    .submission
                    .readback(timing.readback_id())
                    .map(|readback| readback.status());
                match status {
                    Some(GpuReadbackStatus::Pending)
                        if invalid_context.is_none()
                            && lifecycle != GpuExecutionLifecycleState::Closed =>
                    {
                        true
                    }
                    Some(GpuReadbackStatus::Ready(bytes)) => {
                        output.timing_evidence.extend(timing.ready_evidence(&bytes));
                        false
                    }
                    Some(GpuReadbackStatus::Failed(failure)) => {
                        output
                            .timing_evidence
                            .extend(timing.failed_evidence(&failure));
                        false
                    }
                    _ => {
                        output.timing_evidence.extend(timing.diagnostic_evidence(
                            invalid_context.unwrap_or(
                                "GPU context closed before timing readback became terminal",
                            ),
                        ));
                        false
                    }
                }
            });

            accepted.captures.retain(|capture| {
                let status = accepted
                    .submission
                    .readback(capture.readback_id)
                    .map(|readback| readback.status());
                let terminal = match status {
                    Some(GpuReadbackStatus::Pending)
                        if invalid_context.is_none()
                            && lifecycle != GpuExecutionLifecycleState::Closed =>
                    {
                        return true;
                    }
                    Some(GpuReadbackStatus::Ready(bytes)) => capture.ready(&bytes),
                    Some(GpuReadbackStatus::Failed(failure)) => {
                        capture.failed_from_submission(&failure)
                    }
                    _ => capture.failed(
                        "capture_context_unavailable",
                        invalid_context.unwrap_or(
                            "GPU context closed before capture readback became terminal",
                        ),
                    ),
                };
                output.capture_results.push(capture.result(&terminal));
                output.captured_textures.push(terminal);
                false
            });
        }
        self.accepted
            .retain(|accepted| !accepted.timings.is_empty() || !accepted.captures.is_empty());
        output
    }

    #[cfg(test)]
    pub fn retained_submission_count(&self) -> usize {
        self.accepted.len()
    }
}

impl CaptureObservation {
    fn failed_from_submission(&self, failure: &GpuSubmissionFailure) -> RenderCapturedTexture {
        self.failed(
            "capture_readback_failed",
            format!(
                "GPU capture readback failed ({:?}): {}",
                failure.kind(),
                failure.detail()
            ),
        )
    }
}
