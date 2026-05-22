use crate::plugins::render::graph::RenderPreparedFramePreflightCacheState;
use crate::plugins::render::renderer::GfxFrameTimings;
use crate::plugins::render::shader::ShaderReloadPollReport;
use crate::runtime::FramePacingRuntimeStateResource;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderDebugTimingsState {
    pub workload_ms: f32,
    pub total_ms: f32,
    pub preflight_ms: f32,
    pub flow_encode_ms: f32,
    pub encode_submit_ms: f32,
    pub pass_sample_count: usize,
    pub total_pass_millis: f32,
    pub slowest_pass_id: Option<String>,
    pub slowest_pass_millis: f32,
    pub compute_dispatches: Vec<ComputeDispatchSample>,
    pub world_rebuild_latency_ms: f32,
    pub world_rebuild_samples: u64,
    pub world_prepare_samples: u64,
    pub shader_reload_poll_ms: f32,
    pub shader_reload_poll_status: Option<String>,
    pub shader_reload_poll_interval_ms: f32,
    pub diagnostics_report_ms: f32,
    pub diagnostics_report_mode: Option<String>,
    pub preflight_cache_mode: Option<String>,
    pub preflight_cache_status: Option<String>,
    pub preflight_report_source: Option<String>,
    pub frame_pacing_mode: Option<String>,
    pub frame_pacing_target_fps: Option<u32>,
    pub frame_pacing_last_interval_ms: f32,
    pub frame_pacing_next_delay_ms: Option<f32>,
    pub frame_pacing_redraw_requested: bool,
}

impl RenderDebugTimingsState {
    pub fn observe_frame_timings(&mut self, timings: GfxFrameTimings) {
        self.workload_ms = timings.renderer.prepare_ui_ms
            + timings.renderer.prepare_mesh_ms
            + timings.renderer.world_prepare_ms
            + timings.renderer.preflight_ms
            + timings.renderer.flow_encode_ms
            + timings.renderer.encode_submit_ms;
        self.total_ms = timings.acquire_ms + self.workload_ms + timings.present_ms;
        self.preflight_ms = timings.renderer.preflight_ms;
        self.flow_encode_ms = timings.renderer.flow_encode_ms;
        self.encode_submit_ms = timings.renderer.encode_submit_ms;
    }

    pub fn observe_pass_timings(&mut self, samples: &[PassTimingSample]) {
        self.pass_sample_count = samples.len();
        let snapshot = summarize_pass_timings(samples);
        self.total_pass_millis = snapshot.total_millis;
        self.slowest_pass_id = snapshot.slowest_pass_id;
        self.slowest_pass_millis = snapshot.slowest_pass_millis;
        self.compute_dispatches = samples
            .iter()
            .filter_map(|sample| {
                sample
                    .dispatch_workgroups
                    .map(|workgroups| ComputeDispatchSample {
                        flow_id: sample.flow_id.clone(),
                        pass_id: sample.pass_id.clone(),
                        workgroups,
                    })
            })
            .collect();
    }

    pub fn observe_world_rebuild_latency(&mut self, latency_ms: f32) {
        self.world_rebuild_latency_ms = latency_ms.max(0.0);
        self.world_rebuild_samples = self.world_rebuild_samples.saturating_add(1);
    }

    pub fn observe_world_prepare_sample(&mut self) {
        self.world_prepare_samples = self.world_prepare_samples.saturating_add(1);
    }

    pub fn observe_shader_reload_poll(&mut self, report: ShaderReloadPollReport, measured_ms: f32) {
        self.shader_reload_poll_ms = measured_ms.max(0.0);
        self.shader_reload_poll_status = Some(report.status.as_str().to_string());
        self.shader_reload_poll_interval_ms = report.interval_ms.max(0.0);
    }

    pub fn observe_diagnostics_report(&mut self, mode: impl Into<String>, measured_ms: f32) {
        self.diagnostics_report_mode = Some(mode.into());
        self.diagnostics_report_ms = measured_ms.max(0.0);
    }

    pub fn observe_preflight_cache_state(
        &mut self,
        state: &RenderPreparedFramePreflightCacheState,
    ) {
        self.preflight_cache_mode = Some(state.mode.as_str().to_string());
        self.preflight_cache_status = Some(state.status.as_str().to_string());
        self.preflight_report_source = Some(state.report_source.as_str().to_string());
    }

    pub fn observe_frame_pacing(&mut self, pacing: &FramePacingRuntimeStateResource) {
        self.frame_pacing_mode = Some(pacing.mode.as_str().to_string());
        self.frame_pacing_target_fps = pacing.mode.target_fps();
        self.frame_pacing_last_interval_ms = pacing.last_frame_interval_ms;
        self.frame_pacing_next_delay_ms = pacing.next_frame_delay_ms;
        self.frame_pacing_redraw_requested = pacing.redraw_requested;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PassTimingSample {
    pub flow_id: String,
    pub pass_id: String,
    pub pass_kind: String,
    pub millis: f32,
    pub dispatch_workgroups: Option<[u32; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputeDispatchSample {
    pub flow_id: String,
    pub pass_id: String,
    pub workgroups: [u32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFlowTimingSnapshot {
    pub total_millis: f32,
    pub slowest_pass_id: Option<String>,
    pub slowest_pass_millis: f32,
    pub per_pass: Vec<PassTimingSample>,
}

pub fn summarize_pass_timings(samples: &[PassTimingSample]) -> RenderFlowTimingSnapshot {
    let mut total_millis = 0.0f32;
    let mut slowest_pass_id = None::<String>;
    let mut slowest_pass_millis = 0.0f32;

    for sample in samples {
        total_millis += sample.millis.max(0.0);
        if sample.millis > slowest_pass_millis {
            slowest_pass_millis = sample.millis;
            slowest_pass_id = Some(sample.pass_id.clone());
        }
    }

    RenderFlowTimingSnapshot {
        total_millis,
        slowest_pass_id,
        slowest_pass_millis,
        per_pass: samples.to_vec(),
    }
}
