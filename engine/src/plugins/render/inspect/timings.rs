use crate::plugins::render::renderer::GfxFrameTimings;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderDebugTimingsState {
    pub workload_ms: f32,
    pub total_ms: f32,
    pub pass_sample_count: usize,
    pub total_pass_millis: f32,
    pub slowest_pass_id: Option<String>,
    pub slowest_pass_millis: f32,
    pub compute_dispatches: Vec<ComputeDispatchSample>,
    pub world_rebuild_latency_ms: f32,
    pub world_rebuild_samples: u64,
    pub world_prepare_samples: u64,
}

impl RenderDebugTimingsState {
    pub fn observe_frame_timings(&mut self, timings: GfxFrameTimings) {
        self.workload_ms = timings.renderer.prepare_ui_ms
            + timings.renderer.prepare_mesh_ms
            + timings.renderer.world_prepare_ms
            + timings.renderer.encode_submit_ms;
        self.total_ms = timings.acquire_ms + self.workload_ms + timings.present_ms;
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
