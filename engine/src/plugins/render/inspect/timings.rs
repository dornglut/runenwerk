#[derive(Debug, Clone, PartialEq)]
pub struct PassTimingSample {
    pub pass_id: String,
    pub millis: f32,
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
