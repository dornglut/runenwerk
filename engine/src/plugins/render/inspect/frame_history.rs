use super::timings::{
    PassTimingSample, RenderComposedFrameGpuTimingEvidence, RenderGpuTimingCapability,
    RenderPassTimingEvidence, summarize_gpu_pass_timing_evidence,
};
use crate::plugins::render::renderer::RendererFrameTimings;
use std::collections::{BTreeMap, VecDeque};

pub const DEFAULT_RENDER_FRAME_HISTORY_CAPACITY: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, runen_ecs::Resource)]
pub struct RenderFrameObservationPolicyResource {
    pub enabled: bool,
    pub capacity: usize,
    pub sample_every_nth_frame: u64,
}

impl Default for RenderFrameObservationPolicyResource {
    fn default() -> Self {
        Self {
            enabled: false,
            capacity: DEFAULT_RENDER_FRAME_HISTORY_CAPACITY,
            sample_every_nth_frame: 1,
        }
    }
}

impl RenderFrameObservationPolicyResource {
    pub fn enabled(capacity: usize) -> Self {
        Self {
            enabled: true,
            capacity: capacity.max(1),
            sample_every_nth_frame: 1,
        }
    }

    pub fn with_sampling(mut self, sample_every_nth_frame: u64) -> Self {
        self.sample_every_nth_frame = sample_every_nth_frame.max(1);
        self
    }

    pub fn retains_frame(self, frame_index: u64) -> bool {
        self.enabled && frame_index.is_multiple_of(self.sample_every_nth_frame.max(1))
    }

    fn bounded_capacity(self) -> usize {
        self.capacity.max(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderFrameObservationKey {
    pub frame_index: u64,
    pub render_surface_id: u64,
}

impl RenderFrameObservationKey {
    pub const fn new(frame_index: u64, render_surface_id: u64) -> Self {
        Self {
            frame_index,
            render_surface_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderFrameCpuObservation {
    pub acquire_ms: f32,
    pub renderer: RendererFrameTimings,
    pub pass_timings: Vec<PassTimingSample>,
}

impl RenderFrameCpuObservation {
    pub fn observed_renderer_stage_sum_ms(&self) -> f32 {
        self.renderer.prepare_ui_ms
            + self.renderer.prepare_mesh_ms
            + self.renderer.world_prepare_ms
            + self.renderer.preflight_ms
            + self.renderer.flow_encode_ms
            + self.renderer.encode_submit_ms
    }

    pub fn observed_stage_sum_with_acquire_ms(&self) -> f32 {
        self.acquire_ms + self.observed_renderer_stage_sum_ms()
    }
}

#[derive(Debug, Clone)]
pub struct RenderFrameGpuObservation {
    pub pass_timing_capability: RenderGpuTimingCapability,
    pub pass_evidence: Vec<RenderPassTimingEvidence>,
    pub composed_timing_capability: RenderGpuTimingCapability,
    pub composed_timing_evidence: Option<RenderComposedFrameGpuTimingEvidence>,
}

impl RenderFrameGpuObservation {
    fn new(capability: RenderGpuTimingCapability) -> Self {
        Self {
            pass_timing_capability: capability,
            pass_evidence: Vec::new(),
            composed_timing_capability: RenderGpuTimingCapability::UnavailableThisFrame,
            composed_timing_evidence: None,
        }
    }

    fn merge_pass_evidence(&mut self, incoming: Vec<RenderPassTimingEvidence>) {
        let mut occurrence_by_signature = BTreeMap::<(String, String, String), usize>::new();
        for evidence in incoming {
            let signature = (
                evidence.flow_id.clone(),
                evidence.pass_id.clone(),
                evidence.pass_kind.clone(),
            );
            let ordinal = occurrence_by_signature
                .entry(signature.clone())
                .or_default();
            let wanted_ordinal = *ordinal;
            *ordinal = ordinal.saturating_add(1);

            let mut matched = 0_usize;
            let position = self.pass_evidence.iter().position(|existing| {
                let same_signature = existing.flow_id == signature.0
                    && existing.pass_id == signature.1
                    && existing.pass_kind == signature.2;
                if !same_signature {
                    return false;
                }
                if matched == wanted_ordinal {
                    return true;
                }
                matched = matched.saturating_add(1);
                false
            });
            if let Some(position) = position {
                self.pass_evidence[position] = evidence;
            } else {
                self.pass_evidence.push(evidence);
            }
        }

        if !self.pass_evidence.is_empty() {
            self.pass_timing_capability =
                summarize_gpu_pass_timing_evidence(&self.pass_evidence).capability;
        }
    }

    fn merge_composed_timing_evidence(&mut self, evidence: RenderComposedFrameGpuTimingEvidence) {
        self.composed_timing_capability = evidence.gpu_capability;
        self.composed_timing_evidence = Some(evidence);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDisplayPresentationTiming {
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderFramePresentationObservation {
    pub present_operation_submitted: bool,
    pub display_timing: RenderDisplayPresentationTiming,
}

#[derive(Debug, Clone)]
pub struct RenderFrameObservation {
    pub key: RenderFrameObservationKey,
    pub prepare_epoch: u64,
    pub cpu: RenderFrameCpuObservation,
    pub gpu: RenderFrameGpuObservation,
    pub presentation: RenderFramePresentationObservation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderFrameHistoryDropStats {
    pub evicted_observations: u64,
    pub sampled_out_observations: u64,
    pub dropped_correlated_evidence: u64,
    pub uncorrelated_evidence: u64,
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct RenderFrameHistoryState {
    observations: VecDeque<RenderFrameObservation>,
    drop_stats: RenderFrameHistoryDropStats,
}

impl RenderFrameHistoryState {
    pub fn apply_policy(&mut self, policy: RenderFrameObservationPolicyResource) {
        if !policy.enabled {
            self.observations.clear();
            return;
        }
        while self.observations.len() > policy.bounded_capacity() {
            self.observations.pop_front();
            self.drop_stats.evicted_observations =
                self.drop_stats.evicted_observations.saturating_add(1);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn observe_submitted_frame(
        &mut self,
        policy: RenderFrameObservationPolicyResource,
        frame_index: u64,
        render_surface_id: u64,
        prepare_epoch: u64,
        acquire_ms: f32,
        renderer: RendererFrameTimings,
        pass_timings: &[PassTimingSample],
        gpu_capability: RenderGpuTimingCapability,
    ) {
        if !policy.enabled {
            return;
        }
        if !policy.retains_frame(frame_index) {
            self.drop_stats.sampled_out_observations =
                self.drop_stats.sampled_out_observations.saturating_add(1);
            return;
        }

        let key = RenderFrameObservationKey::new(frame_index, render_surface_id);
        let observation = RenderFrameObservation {
            key,
            prepare_epoch,
            cpu: RenderFrameCpuObservation {
                acquire_ms,
                renderer,
                pass_timings: pass_timings.to_vec(),
            },
            gpu: RenderFrameGpuObservation::new(gpu_capability),
            presentation: RenderFramePresentationObservation {
                present_operation_submitted: true,
                display_timing: RenderDisplayPresentationTiming::Unavailable,
            },
        };

        if let Some(existing) = self
            .observations
            .iter_mut()
            .find(|existing| existing.key == key)
        {
            *existing = observation;
            return;
        }

        self.observations.push_back(observation);
        while self.observations.len() > policy.bounded_capacity() {
            self.observations.pop_front();
            self.drop_stats.evicted_observations =
                self.drop_stats.evicted_observations.saturating_add(1);
        }
    }

    pub fn observe_gpu_pass_timing_evidence(
        &mut self,
        policy: RenderFrameObservationPolicyResource,
        evidence: &[RenderPassTimingEvidence],
    ) {
        let mut correlated =
            BTreeMap::<RenderFrameObservationKey, Vec<RenderPassTimingEvidence>>::new();
        for sample in evidence {
            let Some((frame_index, render_surface_id)) =
                sample.frame_index.zip(sample.render_surface_id)
            else {
                self.drop_stats.uncorrelated_evidence =
                    self.drop_stats.uncorrelated_evidence.saturating_add(1);
                continue;
            };
            correlated
                .entry(RenderFrameObservationKey::new(
                    frame_index,
                    render_surface_id,
                ))
                .or_default()
                .push(sample.clone());
        }

        for (key, samples) in correlated {
            let sample_count = u64::try_from(samples.len()).unwrap_or(u64::MAX);
            if let Some(observation) = self
                .observations
                .iter_mut()
                .find(|observation| observation.key == key)
            {
                observation.gpu.merge_pass_evidence(samples);
                continue;
            }
            if !policy.enabled {
                continue;
            }
            self.drop_stats.dropped_correlated_evidence = self
                .drop_stats
                .dropped_correlated_evidence
                .saturating_add(sample_count);
        }
    }

    pub fn observe_composed_gpu_timing_evidence(
        &mut self,
        policy: RenderFrameObservationPolicyResource,
        evidence: &[RenderComposedFrameGpuTimingEvidence],
    ) {
        for sample in evidence {
            let key = RenderFrameObservationKey::new(sample.frame_index, sample.render_surface_id);
            if let Some(observation) = self
                .observations
                .iter_mut()
                .find(|observation| observation.key == key)
            {
                observation.gpu.merge_composed_timing_evidence(sample.clone());
                continue;
            }
            if policy.enabled {
                self.drop_stats.dropped_correlated_evidence =
                    self.drop_stats.dropped_correlated_evidence.saturating_add(1);
            }
        }
    }

    pub fn observation(&self, key: RenderFrameObservationKey) -> Option<&RenderFrameObservation> {
        self.observations
            .iter()
            .find(|observation| observation.key == key)
    }

    pub fn observations(&self) -> impl ExactSizeIterator<Item = &RenderFrameObservation> {
        self.observations.iter()
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    pub const fn drop_stats(&self) -> RenderFrameHistoryDropStats {
        self.drop_stats
    }
}

#[cfg(test)]
mod tests {
    use super::super::timings::RenderGpuTimingDiagnostic;
    use super::*;

    fn policy(capacity: usize) -> RenderFrameObservationPolicyResource {
        RenderFrameObservationPolicyResource::enabled(capacity)
    }

    fn record(
        history: &mut RenderFrameHistoryState,
        policy: RenderFrameObservationPolicyResource,
        frame_index: u64,
        surface: u64,
        acquire_ms: f32,
        capability: RenderGpuTimingCapability,
    ) {
        history.observe_submitted_frame(
            policy,
            frame_index,
            surface,
            frame_index + 100,
            acquire_ms,
            RendererFrameTimings::default(),
            &[],
            capability,
        );
    }

    fn pending(frame: u64, surface: u64) -> RenderPassTimingEvidence {
        RenderPassTimingEvidence::gpu_diagnostic(
            Some(frame),
            Some(surface),
            "flow",
            "pass",
            "compute",
            RenderGpuTimingDiagnostic::readback_pending("pending"),
        )
    }

    fn measured(frame: u64, surface: u64, millis: f32) -> RenderPassTimingEvidence {
        RenderPassTimingEvidence::gpu_sample(
            Some(frame),
            Some(surface),
            "flow",
            "pass",
            "compute",
            millis,
        )
    }

    #[test]
    fn reverse_gpu_arrival_never_cross_joins_cpu_frames() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            1,
            1,
            1.0,
            RenderGpuTimingCapability::Supported,
        );
        record(
            &mut history,
            policy,
            2,
            1,
            2.0,
            RenderGpuTimingCapability::Supported,
        );

        history.observe_gpu_pass_timing_evidence(policy, &[measured(2, 1, 2.5)]);
        history.observe_gpu_pass_timing_evidence(policy, &[measured(1, 1, 1.5)]);

        let first = history
            .observation(RenderFrameObservationKey::new(1, 1))
            .expect("frame one retained");
        let second = history
            .observation(RenderFrameObservationKey::new(2, 1))
            .expect("frame two retained");
        assert_eq!(first.cpu.acquire_ms, 1.0);
        assert_eq!(first.gpu.pass_evidence[0].millis, Some(1.5));
        assert_eq!(second.cpu.acquire_ms, 2.0);
        assert_eq!(second.gpu.pass_evidence[0].millis, Some(2.5));
    }

    #[test]
    fn neighboring_surfaces_never_share_gpu_evidence() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            7,
            1,
            1.0,
            RenderGpuTimingCapability::Supported,
        );
        record(
            &mut history,
            policy,
            7,
            2,
            2.0,
            RenderGpuTimingCapability::Supported,
        );

        history.observe_gpu_pass_timing_evidence(policy, &[measured(7, 2, 3.0)]);

        assert!(
            history
                .observation(RenderFrameObservationKey::new(7, 1))
                .unwrap()
                .gpu
                .pass_evidence
                .is_empty()
        );
        assert_eq!(
            history
                .observation(RenderFrameObservationKey::new(7, 2))
                .unwrap()
                .gpu
                .pass_evidence[0]
                .millis,
            Some(3.0)
        );
    }

    #[test]
    fn pending_gpu_evidence_updates_exact_occurrence_to_measured() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            3,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        history.observe_gpu_pass_timing_evidence(policy, &[pending(3, 1)]);
        assert_eq!(
            history
                .observation(RenderFrameObservationKey::new(3, 1))
                .unwrap()
                .gpu
                .pass_timing_capability,
            RenderGpuTimingCapability::ReadbackPending
        );

        history.observe_gpu_pass_timing_evidence(policy, &[measured(3, 1, 4.0)]);
        let observation = history
            .observation(RenderFrameObservationKey::new(3, 1))
            .unwrap();
        assert_eq!(
            observation.gpu.pass_timing_capability,
            RenderGpuTimingCapability::Supported
        );
        assert_eq!(observation.gpu.pass_evidence.len(), 1);
        assert_eq!(observation.gpu.pass_evidence[0].millis, Some(4.0));
    }

    #[test]
    fn terminal_unavailable_evidence_replaces_pending_without_zero_sample() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            4,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        history.observe_gpu_pass_timing_evidence(policy, &[pending(4, 1)]);
        let unavailable = RenderPassTimingEvidence::gpu_diagnostic(
            Some(4),
            Some(1),
            "flow",
            "pass",
            "compute",
            RenderGpuTimingDiagnostic::unavailable_this_frame("readback failed"),
        );
        history.observe_gpu_pass_timing_evidence(policy, &[unavailable]);

        let observation = history
            .observation(RenderFrameObservationKey::new(4, 1))
            .unwrap();
        assert_eq!(
            observation.gpu.pass_timing_capability,
            RenderGpuTimingCapability::UnavailableThisFrame
        );
        assert_eq!(observation.gpu.pass_evidence[0].millis, None);
    }

    #[test]
    fn unsupported_frame_remains_unsupported_without_synthetic_zero() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            5,
            1,
            0.0,
            RenderGpuTimingCapability::Unsupported,
        );

        let observation = history
            .observation(RenderFrameObservationKey::new(5, 1))
            .unwrap();
        assert_eq!(
            observation.gpu.pass_timing_capability,
            RenderGpuTimingCapability::Unsupported
        );
        assert!(observation.gpu.pass_evidence.is_empty());
    }

    #[test]
    fn retained_pending_evidence_completes_after_sampling_policy_changes() {
        let original = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            original,
            3,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        history.observe_gpu_pass_timing_evidence(original, &[pending(3, 1)]);

        let changed = original.with_sampling(2);
        history.observe_gpu_pass_timing_evidence(changed, &[measured(3, 1, 2.0)]);

        let observation = history
            .observation(RenderFrameObservationKey::new(3, 1))
            .unwrap();
        assert_eq!(
            observation.gpu.pass_timing_capability,
            RenderGpuTimingCapability::Supported
        );
        assert_eq!(observation.gpu.pass_evidence[0].millis, Some(2.0));
        assert_eq!(history.drop_stats().sampled_out_observations, 0);
    }

    #[test]
    fn disabled_policy_clears_retained_observations() {
        let enabled = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            enabled,
            1,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        assert_eq!(history.len(), 1);

        history.apply_policy(RenderFrameObservationPolicyResource::default());

        assert!(history.is_empty());
    }

    #[test]
    fn finite_capacity_evicts_and_late_evidence_is_counted() {
        let policy = policy(1);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            1,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        record(
            &mut history,
            policy,
            2,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        assert!(
            history
                .observation(RenderFrameObservationKey::new(1, 1))
                .is_none()
        );
        history.observe_gpu_pass_timing_evidence(policy, &[measured(1, 1, 1.0)]);

        assert_eq!(history.drop_stats().evicted_observations, 1);
        assert_eq!(history.drop_stats().dropped_correlated_evidence, 1);
    }

    #[test]
    fn disabled_and_sampled_out_history_never_misattach_evidence() {
        let disabled = RenderFrameObservationPolicyResource::default();
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            disabled,
            1,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        history.observe_gpu_pass_timing_evidence(disabled, &[measured(1, 1, 1.0)]);
        assert!(history.is_empty());
        assert_eq!(history.drop_stats(), RenderFrameHistoryDropStats::default());

        let sampled = policy(8).with_sampling(2);
        record(
            &mut history,
            sampled,
            3,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        record(
            &mut history,
            sampled,
            4,
            1,
            0.0,
            RenderGpuTimingCapability::Supported,
        );
        history.observe_gpu_pass_timing_evidence(sampled, &[measured(3, 1, 3.0)]);
        assert!(
            history
                .observation(RenderFrameObservationKey::new(3, 1))
                .is_none()
        );
        assert!(
            history
                .observation(RenderFrameObservationKey::new(4, 1))
                .is_some()
        );
        assert_eq!(history.drop_stats().sampled_out_observations, 1);
        assert_eq!(history.drop_stats().dropped_correlated_evidence, 1);
    }

    #[test]
    fn presentation_state_never_promotes_legacy_present_placeholder() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(
            &mut history,
            policy,
            9,
            1,
            0.5,
            RenderGpuTimingCapability::UnavailableThisFrame,
        );
        let observation = history
            .observation(RenderFrameObservationKey::new(9, 1))
            .unwrap();
        assert!(observation.presentation.present_operation_submitted);
        assert_eq!(
            observation.presentation.display_timing,
            RenderDisplayPresentationTiming::Unavailable
        );
        assert_eq!(observation.cpu.observed_stage_sum_with_acquire_ms(), 0.5);
    }
    fn composed_pending(frame: u64, surface: u64) -> RenderComposedFrameGpuTimingEvidence {
        RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
            frame,
            surface,
            RenderGpuTimingDiagnostic::readback_pending("composed timing pending"),
        )
    }

    fn composed_measured(frame: u64, surface: u64, millis: f32) -> RenderComposedFrameGpuTimingEvidence {
        RenderComposedFrameGpuTimingEvidence::gpu_sample(frame, surface, millis)
    }

    #[test]
    fn composed_gpu_timing_updates_exact_frame_after_reverse_arrival() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(&mut history, policy, 1, 1, 1.0, RenderGpuTimingCapability::UnavailableThisFrame);
        record(&mut history, policy, 2, 1, 2.0, RenderGpuTimingCapability::UnavailableThisFrame);
        history.observe_composed_gpu_timing_evidence(policy, &[composed_measured(2, 1, 5.0)]);
        history.observe_composed_gpu_timing_evidence(policy, &[composed_measured(1, 1, 3.0)]);
        assert_eq!(history.observation(RenderFrameObservationKey::new(1, 1)).unwrap()
            .gpu.composed_timing_evidence.as_ref().unwrap().millis, Some(3.0));
        assert_eq!(history.observation(RenderFrameObservationKey::new(2, 1)).unwrap()
            .gpu.composed_timing_evidence.as_ref().unwrap().millis, Some(5.0));
    }

    #[test]
    fn composed_gpu_timing_never_cross_joins_surfaces() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(&mut history, policy, 7, 1, 0.0, RenderGpuTimingCapability::UnavailableThisFrame);
        record(&mut history, policy, 7, 2, 0.0, RenderGpuTimingCapability::UnavailableThisFrame);
        history.observe_composed_gpu_timing_evidence(policy, &[composed_measured(7, 2, 4.0)]);
        assert!(history.observation(RenderFrameObservationKey::new(7, 1)).unwrap()
            .gpu.composed_timing_evidence.is_none());
        assert_eq!(history.observation(RenderFrameObservationKey::new(7, 2)).unwrap()
            .gpu.composed_timing_capability, RenderGpuTimingCapability::Supported);
    }

    #[test]
    fn composed_gpu_timing_progresses_pending_to_measured_without_zero_fallback() {
        let policy = policy(8);
        let mut history = RenderFrameHistoryState::default();
        record(&mut history, policy, 9, 1, 0.0, RenderGpuTimingCapability::UnavailableThisFrame);
        history.observe_composed_gpu_timing_evidence(policy, &[composed_pending(9, 1)]);
        assert_eq!(history.observation(RenderFrameObservationKey::new(9, 1)).unwrap()
            .gpu.composed_timing_capability, RenderGpuTimingCapability::ReadbackPending);
        history.observe_composed_gpu_timing_evidence(policy, &[composed_measured(9, 1, 6.5)]);
        let observation=history.observation(RenderFrameObservationKey::new(9, 1)).unwrap();
        assert_eq!(observation.gpu.composed_timing_capability, RenderGpuTimingCapability::Supported);
        assert_eq!(observation.gpu.composed_timing_evidence.as_ref().unwrap().millis, Some(6.5));
    }

    #[test]
    fn evicted_frame_counts_late_composed_gpu_evidence() {
        let policy = policy(1);
        let mut history = RenderFrameHistoryState::default();
        record(&mut history, policy, 1, 1, 0.0, RenderGpuTimingCapability::UnavailableThisFrame);
        record(&mut history, policy, 2, 1, 0.0, RenderGpuTimingCapability::UnavailableThisFrame);
        history.observe_composed_gpu_timing_evidence(policy, &[composed_measured(1, 1, 2.0)]);
        assert_eq!(history.drop_stats().dropped_correlated_evidence, 1);
    }

}
