//! Native first-party diagnostic session. Each semantic owner emits its own facts;
//! this is only an optional, bounded read-only capture and export consumer.
//! CPU timestamps share one process-local monotonic epoch. GPU timings do not.
use crate::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use anyhow::{Context, Result, ensure};
use runen_ecs::World;
use runenui_runtime::{InputArbitrationRecord, InputPumpBatch, PumpOutcome};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

const ENV_OUTPUT: &str = "RUNENWERK_PROFILE_OUTPUT";
const EVENT_LIMIT: usize = 16_384;
const RENDER_HISTORY_LIMIT: usize = 512;

#[derive(Clone)]
pub(crate) struct NativeProfilerSink {
    inner: Arc<Mutex<Recorder>>,
    origin: Instant,
    frame: Arc<AtomicU64>,
    output: Arc<PathBuf>,
}

impl runen_ecs::Resource for NativeProfilerSink {}

#[derive(Default)]
struct Recorder {
    events: Vec<CpuObservation>,
    dropped: u64,
}

#[derive(Clone, Serialize)]
struct CpuObservation {
    domain: &'static str,
    stage: &'static str,
    start_ns: u64,
    duration_ns: u64,
    host_frame: Option<u64>,
    window: Option<u64>,
    ui_slot: Option<u64>,
    m13_status: Option<&'static str>,
    m13_outcome: Option<&'static str>,
    m13_processed_envelopes: Option<usize>,
    m13_terminal_receipts: Option<usize>,
    m13_processed_through: Option<u64>,
    m13_paused: Option<bool>,
}

#[derive(Serialize)]
struct RenderObservation {
    render_frame: u64,
    render_surface: u64,
    prepare_epoch: u64,
    target_size_px: (u32, u32),
    acquire_cpu_ms: f32,
    prepare_ui_cpu_ms: f32,
    preflight_cpu_ms: f32,
    flow_encode_cpu_ms: f32,
    encode_submit_cpu_ms: f32,
    composed_gpu_status: &'static str,
    composed_gpu_duration_ms: Option<f32>,
    present_operation_submitted: bool,
    display_timestamp_status: &'static str,
}

#[derive(Serialize)]
struct RenderCapture {
    status: &'static str,
    observations: Vec<RenderObservation>,
    evicted: u64,
    sampled_out: u64,
    dropped_correlated: u64,
    uncorrelated: u64,
    host_frame_join_status: &'static str,
}

#[derive(Serialize)]
struct Capture {
    schema_version: u32,
    kind: &'static str,
    status: &'static str,
    clock: &'static str,
    gpu_clock_relation: &'static str,
    process_id: u32,
    os: &'static str,
    profile: &'static str,
    source_revision: &'static str,
    event_capacity: usize,
    dropped_cpu_events: u64,
    cpu_events: Vec<CpuObservation>,
    render: RenderCapture,
    unavailable: &'static [&'static str],
}

pub(crate) struct ProfileSpan {
    sink: NativeProfilerSink,
    started: Instant,
    domain: &'static str,
    stage: &'static str,
    window: Option<u64>,
    ui_slot: Option<u64>,
    is_frame: bool,
}

impl Drop for ProfileSpan {
    fn drop(&mut self) {
        self.sink.record(
            self.domain,
            self.stage,
            self.started,
            self.window,
            self.ui_slot,
            None,
        );
        if self.is_frame {
            self.sink.frame.store(0, Ordering::Relaxed);
        }
    }
}

impl NativeProfilerSink {
    pub(crate) fn from_environment() -> Result<Option<Self>> {
        let Some(path) = std::env::var_os(ENV_OUTPUT) else {
            return Ok(None);
        };
        let path = path
            .into_string()
            .map_err(|_| anyhow::anyhow!("{ENV_OUTPUT} must be a Unicode file path"))?;
        let path = PathBuf::from(path);
        ensure!(
            path.file_name().is_some() && !path.as_os_str().is_empty(),
            "{ENV_OUTPUT} must name an output file"
        );
        ensure!(
            !path.exists(),
            "{ENV_OUTPUT} output already exists (refusing overwrite)"
        );
        Ok(Some(Self::new(path)))
    }

    fn new(output: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Recorder::default())),
            origin: Instant::now(),
            frame: Arc::new(AtomicU64::new(0)),
            output: Arc::new(output),
        }
    }

    pub(crate) fn enable_render_history(&self, world: &mut World) {
        if let Ok(policy) = world.resource_mut::<RenderFrameObservationPolicyResource>() {
            // The existing Render-owned observation policy remains the sole producer.
            // A profiler session explicitly enables its bounded measurements.
            if !policy.enabled {
                *policy = RenderFrameObservationPolicyResource::enabled(RENDER_HISTORY_LIMIT);
            }
        }
    }

    pub(crate) fn span(
        &self,
        domain: &'static str,
        stage: &'static str,
        window: Option<u64>,
        ui_slot: Option<u64>,
    ) -> ProfileSpan {
        ProfileSpan {
            sink: self.clone(),
            started: Instant::now(),
            domain,
            stage,
            window,
            ui_slot,
            is_frame: false,
        }
    }

    pub(crate) fn frame_span(&self, frame: u64) -> ProfileSpan {
        self.frame.store(frame, Ordering::Relaxed);
        let mut span = self.span("host", "ordinary_frame", None, None);
        span.is_frame = true;
        span
    }

    fn record(
        &self,
        domain: &'static str,
        stage: &'static str,
        started: Instant,
        window: Option<u64>,
        ui_slot: Option<u64>,
        m13: Option<(&'static str, &'static str, usize, usize, Option<u64>, bool)>,
    ) {
        let duration_ns = saturating_u64(started.elapsed().as_nanos());
        let start_ns = saturating_u64(started.duration_since(self.origin).as_nanos());
        let (status, outcome, processed, terminal, through, paused) = match m13 {
            Some((status, outcome, processed, terminal, through, paused)) => (
                Some(status),
                Some(outcome),
                Some(processed),
                Some(terminal),
                through,
                Some(paused),
            ),
            None => (None, None, None, None, None, None),
        };
        let event = CpuObservation {
            domain,
            stage,
            start_ns,
            duration_ns,
            host_frame: match self.frame.load(Ordering::Relaxed) {
                0 => None,
                frame => Some(frame),
            },
            window,
            ui_slot,
            m13_status: status,
            m13_outcome: outcome,
            m13_processed_envelopes: processed,
            m13_terminal_receipts: terminal,
            m13_processed_through: through,
            m13_paused: paused,
        };
        if let Ok(mut recorder) = self.inner.lock() {
            if recorder.events.len() < EVENT_LIMIT {
                recorder.events.push(event);
            } else {
                recorder.dropped = recorder.dropped.saturating_add(1);
            }
        }
    }

    pub(crate) fn record_schedule(&self, stage: &'static str, started: Instant) {
        self.record("ecs_schedule", stage, started, None, None, None);
    }

    pub(crate) fn record_m13_pump(
        &self,
        slot: u64,
        started: Instant,
        batch: &InputPumpBatch,
        settlement_status: &'static str,
    ) {
        let outcome = match batch.report().outcome() {
            PumpOutcome::Quiescent => "quiescent",
            PumpOutcome::BudgetExhausted => "budget_exhausted",
            PumpOutcome::Closed => "closed",
            PumpOutcome::Terminal(_) => "terminal",
            _ => "unknown",
        };
        let terminal_receipts = batch
            .ordered_records()
            .iter()
            .filter(|record| matches!(record, InputArbitrationRecord::InputSettled(_)))
            .count();
        self.record(
            "runenui_m13",
            "canonical_input_pump",
            started,
            None,
            Some(slot),
            Some((
                settlement_status,
                outcome,
                batch.report().processed_envelopes(),
                terminal_receipts,
                batch.processed_through().map(|sequence| sequence.get()),
                batch.pause_reason().is_some(),
            )),
        );
    }

    pub(crate) fn finish(&self, world: &World, completed: bool) -> Result<()> {
        let (events, dropped) = {
            let state = self
                .inner
                .lock()
                .map_err(|_| anyhow::anyhow!("profiling observer poisoned"))?;
            (state.events.clone(), state.dropped)
        };
        let render = match world.resource::<RenderFrameHistoryState>() {
            Ok(history) => {
                let stats = history.drop_stats();
                RenderCapture {
                    status: if history.is_empty() {
                        "no_observations"
                    } else {
                        "available"
                    },
                    observations: history
                        .observations()
                        .map(|value| RenderObservation {
                            render_frame: value.key.frame_index,
                            render_surface: value.key.render_surface_id,
                            prepare_epoch: value.prepare_epoch,
                            target_size_px: value.target_size_px,
                            acquire_cpu_ms: value.cpu.acquire_ms,
                            prepare_ui_cpu_ms: value.cpu.renderer.prepare_ui_ms,
                            preflight_cpu_ms: value.cpu.renderer.preflight_ms,
                            flow_encode_cpu_ms: value.cpu.renderer.flow_encode_ms,
                            encode_submit_cpu_ms: value.cpu.renderer.encode_submit_ms,
                            composed_gpu_status: value.gpu.composed_timing_capability.as_str(),
                            composed_gpu_duration_ms: value
                                .gpu
                                .composed_timing_evidence
                                .as_ref()
                                .and_then(|gpu| gpu.gpu_composed_frame_ms),
                            present_operation_submitted: value
                                .presentation
                                .present_operation_submitted,
                            display_timestamp_status: "unavailable",
                        })
                        .collect(),
                    evicted: stats.evicted_observations,
                    sampled_out: stats.sampled_out_observations,
                    dropped_correlated: stats.dropped_correlated_evidence,
                    uncorrelated: stats.uncorrelated_evidence,
                    host_frame_join_status: "unavailable_no_owner_correlation",
                }
            }
            Err(_) => RenderCapture {
                status: "render_plugin_not_installed",
                observations: Vec::new(),
                evicted: 0,
                sampled_out: 0,
                dropped_correlated: 0,
                uncorrelated: 0,
                host_frame_join_status: "unavailable_no_owner_correlation",
            },
        };
        let report = Capture {
            schema_version: 1,
            kind: "runenwerk_native_application_profile",
            status: if completed {
                "completed"
            } else {
                "failed_or_interrupted"
            },
            clock: "cpu_monotonic_process_local_nanoseconds",
            gpu_clock_relation: "not_synchronized_to_cpu",
            process_id: std::process::id(),
            os: std::env::consts::OS,
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "optimized"
            },
            source_revision: option_env!("RUNENWERK_SOURCE_REVISION").unwrap_or("unknown"),
            event_capacity: EVENT_LIMIT,
            dropped_cpu_events: dropped,
            cpu_events: events,
            render,
            unavailable: &[
                "per_system_ecs_cpu",
                "runenui_internal_layout_shaping_paint_cpu",
                "process_cpu_and_memory",
                "physical_display_scanout",
                "input_to_photon_latency",
            ],
        };
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.output.as_ref())
            .with_context(|| "cannot create new profiler output (no overwrite)")?;
        let mut output = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut output, &report)
            .context("cannot serialize native profiler capture")?;
        output
            .flush()
            .context("cannot flush native profiler capture")?;
        println!(
            "Runenwerk profiler: events={} dropped={} render_frames={} status={}",
            report.cpu_events.len(),
            report.dropped_cpu_events,
            report.render.observations.len(),
            report.status
        );
        Ok(())
    }
}

fn saturating_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_is_bounded_and_reports_losses() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        for _ in 0..(EVENT_LIMIT + 3) {
            let _scope = sink.span("host", "test_event", None, None);
        }
        let state = sink.inner.lock().unwrap();
        assert_eq!(state.events.len(), EVENT_LIMIT);
        assert_eq!(state.dropped, 3);
        assert!(
            state
                .events
                .iter()
                .all(|event| event.duration_ns < u64::MAX)
        );
    }

    #[test]
    fn default_render_history_is_not_implicitly_active() {
        assert!(!RenderFrameObservationPolicyResource::default().enabled);
    }

    #[test]
    fn profiler_records_no_arbitrary_text_or_original_input_payload() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        {
            let _scope = sink.span("host", "window_keyboard", Some(1), None);
        }
        let state = sink.inner.lock().unwrap();
        let json = serde_json::to_string(&state.events).unwrap();
        assert!(json.contains("window_keyboard"));
        assert!(!json.contains("text"));
        assert!(!json.contains("title"));
    }

    #[test]
    fn frame_identity_is_scoped_and_not_invented_for_idle_callbacks() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        {
            let _scope = sink.frame_span(7);
            let _nested = sink.span("ecs_schedule", "update", None, None);
        }
        {
            let _unrelated = sink.span("host", "about_to_wait", None, None);
        }
        let state = sink.inner.lock().unwrap();
        assert_eq!(state.events[0].host_frame, Some(7));
        assert_eq!(state.events[1].host_frame, Some(7));
        assert_eq!(state.events[2].host_frame, None);
    }
}
