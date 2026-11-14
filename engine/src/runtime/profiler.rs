//! Native first-party diagnostic session. Each semantic owner emits its own facts;
//! this is only an optional, bounded read-only capture and export consumer.
//! CPU timestamps share one process-local monotonic epoch. GPU timings do not.
use crate::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use crate::runtime::{FramePacingPolicyResource, WindowStateRegistryResource};
use anyhow::{Context, Result, ensure};
use runen_ecs::World;
use runenui_core::{PointerId, PointerPhase, WorkSequence};
use runenui_runtime::{
    InputArbitrationRecord, InputPumpBatch, PumpOutcome, Trace, TraceRecord, TraceRecordKind,
    TraceSurfaceSnapshotKind, UiInputFinality,
};
use serde::Serialize;
use std::collections::{BTreeMap, VecDeque};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const ENV_OUTPUT: &str = "RUNENWERK_PROFILE_OUTPUT";
const EVENT_LIMIT: usize = 16_384;
const RENDER_HISTORY_LIMIT: usize = 512;
/// Fixed logarithmic-ish time buckets. Counts cover [0, bound) and one unbounded tail.
const DURATION_HISTOGRAM_UPPER_NS: [u64; 10] = [
    50_000, 100_000, 250_000, 500_000, 1_000_000, 2_000_000, 4_000_000, 8_000_000, 16_000_000,
    33_333_334,
];
type StageGroupKey = (&'static str, &'static str, Option<u64>, Option<u64>);
static PROCESS_SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub(crate) struct NativeProfilerSink {
    inner: Arc<Mutex<Recorder>>,
    origin: Instant,
    output: Arc<PathBuf>,
    session_id: Arc<str>,
}

impl runen_ecs::Resource for NativeProfilerSink {}

#[derive(Default)]
struct Recorder {
    events: VecDeque<CpuObservation>,
    dropped: u64,
    next_event_ordinal: u64,
    // A Host frame belongs only to the native thread that admitted it.
    active_frame: Option<(std::thread::ThreadId, u64)>,
}

#[derive(Clone, Serialize)]
struct CpuObservation {
    /// Completion-order ordinal scoped to the parent capture session.
    event_ordinal: u64,
    domain: &'static str,
    stage: &'static str,
    start_ns: u64,
    duration_ns: u64,
    host_frame: Option<u64>,
    window: Option<u64>,
    ui_slot: Option<u64>,
    m13: Option<M13Observation>,
}

#[derive(Clone, Serialize)]
struct M13Observation {
    settlement_status: &'static str,
    pump_outcome: &'static str,
    processed_envelopes: usize,
    terminal_receipts_total: usize,
    sampled_receipts: Vec<M13Receipt>,
    omitted_receipts: usize,
    processed_through: Option<u64>,
    paused: bool,
}

#[derive(Clone, Serialize)]
struct M13Receipt {
    sequence: u64,
    finality: &'static str,
    ownership_revision: u64,
    pointer_trace: PointerTraceEvidence,
}

/// Bounded read-only projection of the source-owned canonical RunenUI Trace.
/// Only known status strings and scalar facts are exported; no authored ID,
/// opaque mounted identity, app content or trace Debug string escapes.
#[derive(Clone, Serialize)]
pub(crate) struct PointerTraceEvidence {
    sequence: u64,
    status: &'static str,
    phase: Option<&'static str>,
    displayed_hit_generation: Option<u64>,
    displayed_snapshot: Option<&'static str>,
    physical_target_present: Option<bool>,
    physical_path_depth: Option<usize>,
    matches_previous_down_physical_target: Option<bool>,
    pointer_default: Option<&'static str>,
    activation_trace: &'static str,
    trace_evicted_before: Option<u64>,
}

impl PointerTraceEvidence {
    const fn unavailable(sequence: u64) -> Self {
        Self {
            sequence,
            status: "unavailable_no_canonical_pointer_trace",
            phase: None,
            displayed_hit_generation: None,
            displayed_snapshot: None,
            physical_target_present: None,
            physical_path_depth: None,
            matches_previous_down_physical_target: None,
            pointer_default: None,
            activation_trace: "unavailable",
            trace_evicted_before: None,
        }
    }
}

/// Projects at most the same 16 exact terminal receipt sequences exported by
/// the M13 sink. This reads RunenUI's own bounded trace; it never replays work,
/// captures authored labels or creates a parallel mounted/route authority.
pub(crate) fn project_native_pointer_trace(
    trace: &Trace,
    batch: &InputPumpBatch,
) -> Vec<PointerTraceEvidence> {
    let mut evidence = Vec::new();
    for record in batch.ordered_records() {
        if let InputArbitrationRecord::InputSettled(receipt) = record {
            if evidence.len() >= 16 {
                break;
            }
            evidence.push(project_one_pointer_trace(trace, receipt.sequence()));
        }
    }
    evidence
}

const fn pointer_phase_name(phase: PointerPhase) -> &'static str {
    match phase {
        PointerPhase::Move => "move",
        PointerPhase::Down => "down",
        PointerPhase::Up => "up",
        PointerPhase::Cancel => "cancel",
        PointerPhase::Wheel => "wheel",
        _ => "unknown",
    }
}

fn project_one_pointer_trace(trace: &Trace, work: WorkSequence) -> PointerTraceEvidence {
    let mut evidence = PointerTraceEvidence::unavailable(work.get());
    evidence.trace_evicted_before = trace.dropped_before_sequence().map(|seq| seq.get());
    let mut submission: Option<(PointerId, PointerPhase)> = None;
    let mut physical: Option<&TraceRecord> = None;
    let mut committed = false;
    let mut activated = false;
    for record in trace
        .records()
        .filter(|record| record.work_sequence() == Some(work))
    {
        match record.kind() {
            TraceRecordKind::PointerSubmissionAccepted { pointer_id, phase } => {
                submission = Some((*pointer_id, *phase));
            }
            TraceRecordKind::PointerPhysicalTargetResolved => physical = Some(record),
            TraceRecordKind::PointerInteractionCommitted { .. } => committed = true,
            TraceRecordKind::PointerActivateCollected { .. } => activated = true,
            TraceRecordKind::PointerDefaultApplied { .. } => {
                evidence.pointer_default = Some("applied");
            }
            TraceRecordKind::PointerDefaultSuppressed { .. } => {
                evidence.pointer_default = Some("suppressed");
            }
            _ => {}
        }
    }
    evidence.phase = submission.map(|(_, phase)| pointer_phase_name(phase));
    let Some(observation) = physical else {
        evidence.status = if submission.is_some() {
            "unavailable_no_physical_observation"
        } else {
            "unavailable_no_canonical_pointer_trace"
        };
        return evidence;
    };
    evidence.status = if committed {
        "observed_committed_pointer_transaction"
    } else {
        // The original hit can be recorded without any routed transaction,
        // e.g. a non-activating Move; this is not fabricated trace loss.
        "physical_observed_without_routed_commit"
    };
    evidence.physical_target_present = Some(observation.target().is_some());
    evidence.physical_path_depth = observation
        .context()
        .physical_path()
        .map(|path| path.targets().len());
    if let Some(surface) = observation.context().surface() {
        evidence.displayed_hit_generation = Some(surface.hit_test_generation());
        evidence.displayed_snapshot = match surface.snapshot() {
            Some(TraceSurfaceSnapshotKind::Current) => Some("current"),
            Some(TraceSurfaceSnapshotKind::RetainedHistorical) => Some("retained_historical"),
            _ => None,
        };
    }
    // "not_observed" is a statement about retained source records, not a
    // claim about widget action success or completion of application effects.
    evidence.activation_trace = if activated {
        "collected"
    } else if committed {
        "not_observed_in_retained_trace"
    } else {
        "unavailable"
    };
    // Compare canonical physical hit identity without releasing the opaque
    // MountedNodeId. This is NOT the actual pressed-owner eligibility fact.
    if let Some((pointer_id, PointerPhase::Up)) = submission
        && let Some(current_target) = observation.target()
    {
        // Pointer and surface are source-local: never relate coincident
        // pointer numbers from different devices or source surfaces.
        let current_pointer = observation.context().pointer();
        let current_surface = observation.context().surface();
        evidence.matches_previous_down_physical_target = trace
            .records()
            .take_while(|record| record.sequence() < observation.sequence())
            .filter(|record| {
                if !matches!(
                    record.kind(),
                    TraceRecordKind::PointerPhysicalTargetResolved
                ) {
                    return false;
                }
                let Some((prior_pointer, current_pointer)) =
                    record.context().pointer().zip(current_pointer)
                else {
                    return false;
                };
                let Some((prior_surface, current_surface)) =
                    record.context().surface().zip(current_surface)
                else {
                    return false;
                };
                *prior_pointer.pointer_id() == pointer_id
                    && prior_pointer.phase() == Some(PointerPhase::Down)
                    && prior_pointer.device_id() == current_pointer.device_id()
                    && prior_surface.surface_id() == current_surface.surface_id()
            })
            .last()
            .and_then(|prior| prior.target())
            .map(|prior| prior.mounted_node_id() == current_target.mounted_node_id());
    }
    evidence
}

#[derive(Serialize)]
struct StageStatistics {
    domain: &'static str,
    stage: &'static str,
    window: Option<u64>,
    ui_slot: Option<u64>,
    sample_count: usize,
    median_ns: u64,
    p95_ns: u64,
    duration_histogram_ns: DurationHistogram,
}

/// Each count corresponds to [previous upper bound, this upper bound);
/// the final count is >= the last upper bound. One event belongs to one bin.
#[derive(Serialize)]
struct DurationHistogram {
    upper_bounds_exclusive_ns: [u64; 10],
    counts: [usize; 11],
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
    omitted_by_export_limit: usize,
    evicted: u64,
    sampled_out: u64,
    dropped_correlated: u64,
    uncorrelated: u64,
    host_frame_join_status: &'static str,
}

#[derive(Serialize)]
struct HostPacingSnapshot {
    observed_at: &'static str,
    mode: &'static str,
    target_fps: Option<u32>,
}

#[derive(Serialize)]
struct NativeWindowSnapshot {
    window_id: u64,
    size_px: (u32, u32),
    scale_factor: f64,
    focused: bool,
}

#[derive(Serialize)]
struct NativeWindowCapture {
    status: &'static str,
    observed_at: &'static str,
    windows: Vec<NativeWindowSnapshot>,
}

#[derive(Serialize)]
struct Capture {
    schema_version: u32,
    /// Non-secret diagnostic grouping identity; not an authentication token.
    session_id: String,
    kind: &'static str,
    status: &'static str,
    clock: &'static str,
    gpu_clock_relation: &'static str,
    process_id: u32,
    os: &'static str,
    profile: &'static str,
    source_revision: &'static str,
    host_pacing: Option<HostPacingSnapshot>,
    native_windows: NativeWindowCapture,
    event_capacity: usize,
    dropped_cpu_events: u64,
    cpu_events: Vec<CpuObservation>,
    stage_statistics: Vec<StageStatistics>,
    render: RenderCapture,
    unavailable: &'static [&'static str],
}

/// Why a canonical M13 input pump did not return a batch. No input payloads are captured.
#[derive(Clone, Copy)]
pub(crate) enum M13PumpFailureStage {
    SlotLookup,
    RuntimePump,
}

/// Removes incomplete temporary files on ordinary export errors. The final
/// output path is never touched before a complete same-directory publication.
struct TemporaryProfilePath(PathBuf);

impl Drop for TemporaryProfilePath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
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
            self.sink.end_frame();
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
        // Distinguish sessions even when a PID is reused. This is a diagnostic
        // correlation label, not a security identifier or a timing clock.
        let time_part = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(delta) => format!("a{:032x}", delta.as_nanos()),
            Err(error) => format!("b{:032x}", error.duration().as_nanos()),
        };
        let counter = PROCESS_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let session_id = format!(
            "native-{:08x}-{time_part}-{counter:016x}",
            std::process::id()
        );
        Self {
            inner: Arc::new(Mutex::new(Recorder::default())),
            origin: Instant::now(),
            output: Arc::new(output),
            session_id: Arc::from(session_id),
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
        if let Ok(mut recorder) = self.inner.lock() {
            recorder.active_frame = Some((std::thread::current().id(), frame));
        }
        let mut span = self.span("host", "ordinary_frame", None, None);
        span.is_frame = true;
        span
    }

    fn end_frame(&self) {
        if let Ok(mut recorder) = self.inner.lock() {
            recorder.active_frame = None;
        }
    }

    fn record(
        &self,
        domain: &'static str,
        stage: &'static str,
        started: Instant,
        window: Option<u64>,
        ui_slot: Option<u64>,
        m13: Option<M13Observation>,
    ) {
        let duration_ns = saturating_u64(started.elapsed().as_nanos());
        let start_ns = saturating_u64(started.duration_since(self.origin).as_nanos());
        if let Ok(mut recorder) = self.inner.lock() {
            recorder.next_event_ordinal = recorder.next_event_ordinal.saturating_add(1);
            let event_ordinal = recorder.next_event_ordinal;
            let host_frame = match recorder.active_frame {
                Some((thread, frame)) if thread == std::thread::current().id() => Some(frame),
                _ => None,
            };
            let event = CpuObservation {
                event_ordinal,
                domain,
                stage,
                start_ns,
                duration_ns,
                host_frame,
                window,
                ui_slot,
                m13,
            };
            if recorder.events.len() < EVENT_LIMIT {
                recorder.events.push_back(event);
            } else {
                recorder.events.pop_front();
                recorder.events.push_back(event);
                recorder.dropped = recorder.dropped.saturating_add(1);
            }
        }
    }

    pub(crate) fn record_schedule(&self, stage: &'static str, started: Instant) {
        self.record("ecs_schedule", stage, started, None, None, None);
    }

    /// A missing batch cannot justify zero processed inputs or a settled outcome.
    pub(crate) fn record_m13_pump_failure(
        &self,
        slot: u64,
        started: Instant,
        failure: M13PumpFailureStage,
    ) {
        let stage = match failure {
            M13PumpFailureStage::SlotLookup => "canonical_input_slot_lookup_failed",
            M13PumpFailureStage::RuntimePump => "canonical_input_runtime_pump_failed",
        };
        self.record("runenui_m13", stage, started, None, Some(slot), None);
    }

    pub(crate) fn record_m13_pump(
        &self,
        slot: u64,
        started: Instant,
        batch: &InputPumpBatch,
        settlement_status: &'static str,
        pointer_trace: &[PointerTraceEvidence],
    ) {
        let outcome = match batch.report().outcome() {
            PumpOutcome::Quiescent => "quiescent",
            PumpOutcome::BudgetExhausted => "budget_exhausted",
            PumpOutcome::Closed => "closed",
            PumpOutcome::Terminal(_) => "terminal",
            _ => "unknown",
        };
        let mut receipt_samples = Vec::new();
        let mut receipt_count = 0_usize;
        for record in batch.ordered_records() {
            if let InputArbitrationRecord::InputSettled(receipt) = record {
                receipt_count = receipt_count.saturating_add(1);
                if receipt_samples.len() < 16 {
                    let finality = match receipt.finality() {
                        UiInputFinality::Committed(_) => "committed",
                        UiInputFinality::ProcessingRejected(_) => "processing_rejected",
                        UiInputFinality::Aborted(_) => "aborted",
                        _ => "unknown",
                    };
                    receipt_samples.push(M13Receipt {
                        sequence: receipt.sequence().get(),
                        finality,
                        ownership_revision: receipt.ownership_revision().get(),
                        pointer_trace: pointer_trace
                            .iter()
                            .find(|trace| trace.sequence == receipt.sequence().get())
                            .cloned()
                            .unwrap_or_else(|| {
                                PointerTraceEvidence::unavailable(receipt.sequence().get())
                            }),
                    });
                }
            }
        }
        let m13 = M13Observation {
            settlement_status,
            pump_outcome: outcome,
            processed_envelopes: batch.report().processed_envelopes(),
            terminal_receipts_total: receipt_count,
            omitted_receipts: receipt_count.saturating_sub(receipt_samples.len()),
            sampled_receipts: receipt_samples,
            processed_through: batch.processed_through().map(|sequence| sequence.get()),
            paused: batch.pause_reason().is_some(),
        };
        self.record(
            "runenui_m13",
            "canonical_input_pump",
            started,
            None,
            Some(slot),
            Some(m13),
        );
    }

    pub(crate) fn finish(&self, world: &World, completed: bool) -> Result<()> {
        let (events, dropped) = {
            let state = self
                .inner
                .lock()
                .map_err(|_| anyhow::anyhow!("profiling observer poisoned"))?;
            (
                state.events.iter().cloned().collect::<Vec<_>>(),
                state.dropped,
            )
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
                        .skip(history.len().saturating_sub(RENDER_HISTORY_LIMIT))
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
                    omitted_by_export_limit: history.len().saturating_sub(RENDER_HISTORY_LIMIT),
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
                omitted_by_export_limit: 0,
                evicted: 0,
                sampled_out: 0,
                dropped_correlated: 0,
                uncorrelated: 0,
                host_frame_join_status: "unavailable_no_owner_correlation",
            },
        };
        let report = Capture {
            schema_version: 2,
            session_id: self.session_id.to_string(),
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
            host_pacing: world
                .resource::<FramePacingPolicyResource>()
                .ok()
                .map(|policy| HostPacingSnapshot {
                    observed_at: "capture_close",
                    mode: policy.mode.as_str(),
                    target_fps: policy.mode.target_fps(),
                }),
            native_windows: match world.resource::<WindowStateRegistryResource>() {
                Ok(registry) => NativeWindowCapture {
                    status: "available",
                    observed_at: "capture_close_not_display_timestamp",
                    windows: registry
                        .records()
                        .map(|record| NativeWindowSnapshot {
                            window_id: record.native_window_id.raw(),
                            size_px: record.size_px,
                            scale_factor: record.scale_factor,
                            focused: record.focused,
                        })
                        .collect(),
                },
                Err(_) => NativeWindowCapture {
                    status: "registry_unavailable",
                    observed_at: "capture_close_not_display_timestamp",
                    windows: Vec::new(),
                },
            },
            event_capacity: EVENT_LIMIT,
            dropped_cpu_events: dropped,
            stage_statistics: summarize_events(&events),
            cpu_events: events,
            render,
            unavailable: &[
                "per_system_ecs_cpu",
                "runenui_internal_layout_shaping_paint_cpu",
                "process_cpu_and_memory",
                "native_wakeup_deadline_lateness",
                "gpu_device_driver_and_display_provenance",
                "physical_display_scanout",
                "input_to_photon_latency",
            ],
        };
        write_profile_atomically(self.output.as_ref(), &report)?;
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

/// Build and sync the JSON completely before atomically publishing the final name.
/// A hard-link publication cannot replace an existing target. A filesystem that
/// cannot provide that guarantee must reject capture rather than expose partial JSON.
fn write_profile_atomically(path: &Path, report: &Capture) -> Result<()> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let temporary = TemporaryProfilePath(directory.join(format!(
        ".runenwerk-profile-{}-{nonce}.tmp",
        std::process::id()
    )));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(&temporary.0)
        .context("cannot create temporary profiler output")?;
    {
        let mut output = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut output, report)
            .context("cannot serialize native profiler capture")?;
        output
            .flush()
            .context("cannot flush native profiler capture")?;
        output
            .get_ref()
            .sync_all()
            .context("cannot sync native profiler capture")?;
    }
    std::fs::hard_link(&temporary.0, path)
        .context("cannot publish profiler report without overwrite (hard links required)")?;
    Ok(())
}

fn saturating_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Close-only reducer: these durations are CPU stage spans, not additive frame wall time.
fn summarize_events(events: &[CpuObservation]) -> Vec<StageStatistics> {
    let mut groups = BTreeMap::<StageGroupKey, Vec<u64>>::new();
    for event in events {
        groups
            .entry((event.domain, event.stage, event.window, event.ui_slot))
            .or_default()
            .push(event.duration_ns);
    }
    groups
        .into_iter()
        .map(|((domain, stage, window, ui_slot), mut durations)| {
            durations.sort_unstable();
            let len = durations.len();
            StageStatistics {
                domain,
                stage,
                window,
                ui_slot,
                sample_count: len,
                median_ns: durations[(len - 1) / 2],
                p95_ns: durations[(len * 95).div_ceil(100) - 1],
                duration_histogram_ns: duration_histogram(&durations),
            }
        })
        .collect()
}

fn duration_histogram(durations: &[u64]) -> DurationHistogram {
    let mut counts = [0; 11];
    for &duration in durations {
        let bin = DURATION_HISTOGRAM_UPPER_NS.partition_point(|&upper| duration >= upper);
        counts[bin] += 1;
    }
    DurationHistogram {
        upper_bounds_exclusive_ns: DURATION_HISTOGRAM_UPPER_NS,
        counts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_pointer_trace_is_original_and_excludes_ui_content() {
        use runenui_core::{
            Effects, InputDeviceId, LogicalPoint, NoHostProtocol, PointerButton, PointerButtons,
            PointerDeviceKind, PointerEvent, PointerId, PointerPhase, StyleEnvironment, UiApp,
            View, button,
        };
        use runenui_runtime::{AppRuntime, LogicalSize, PumpBudget, SurfaceBuildContext};

        struct TraceProbe;
        impl UiApp for TraceProbe {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn initial_producer_budget(_: &Self::State) -> runenui_core::ProducerBudget {
                runenui_core::ProducerBudget::new(24, 0).with_reconciliation(4, 0, 4)
            }

            fn action_producer_budget(
                _: &Self::State,
                _: &Self::Action,
            ) -> runenui_core::ProducerBudget {
                runenui_core::ProducerBudget::new(64, 32).with_reconciliation(4, 4, 4)
            }

            fn interaction_producer_budget(_: &Self::State) -> runenui_core::ProducerBudget {
                runenui_core::ProducerBudget::new(64, 32).with_reconciliation(4, 4, 4)
            }

            fn root(_: &Self::State) -> impl View<Self::Action> {
                button("SENSITIVE SECRET BUTTON CAPTION")
                    .id("SENSITIVE SECRET AUTHORED ID")
                    .on_activate(|| ())
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                Effects::none()
            }
        }

        let mut runtime = AppRuntime::<TraceProbe>::mount(()).unwrap();
        let style = StyleEnvironment::default();
        let publication = runtime
            .publish_surface(&SurfaceBuildContext::tight(
                &style,
                LogicalSize::try_new(800.0, 600.0).unwrap(),
            ))
            .unwrap();
        let rect = publication
            .frame()
            .nodes()
            .iter()
            .find(|node| node.authored_id().is_some())
            .expect("source-authored control is publicly published")
            .bounds();
        let point = LogicalPoint::new(
            rect.x() + rect.width() / 2.0,
            rect.y() + rect.height() / 2.0,
        )
        .unwrap();
        assert!(
            publication.hit_test_scene().target_at(point).is_some(),
            "published control center must have a real physical hit"
        );
        let pointer = PointerId::new(41).unwrap();
        let device = InputDeviceId::new(42).unwrap();
        for phase in [PointerPhase::Move, PointerPhase::Down, PointerPhase::Up] {
            let mut event = PointerEvent::new(
                pointer,
                PointerDeviceKind::Mouse,
                phase,
                point,
                publication.input_context().clone(),
            )
            .with_device_id(device);
            if phase == PointerPhase::Down {
                event = event
                    .with_buttons(PointerButtons::new([PointerButton::Primary]))
                    .with_changed_button(PointerButton::Primary);
            } else if phase == PointerPhase::Up {
                event = event.with_changed_button(PointerButton::Primary);
            }
            let submitted = runtime.submit_pointer(event).unwrap();
            let batch = runtime.pump(PumpBudget::new(128, 128, 128, 128)).unwrap();
            let evidence = project_native_pointer_trace(runtime.trace(), &batch);
            let observed = evidence
                .iter()
                .find(|item| item.sequence == submitted.sequence().get())
                .expect("exact original pointer receipt is projected");
            assert_eq!(observed.phase, Some(pointer_phase_name(phase)));
            assert!(
                matches!(
                    observed.status,
                    "observed_committed_pointer_transaction"
                        | "physical_observed_without_routed_commit"
                ),
                "phase={phase:?} source trace: {}",
                serde_json::to_string(observed).unwrap()
            );
            assert_eq!(observed.physical_target_present, Some(true));
            assert_eq!(observed.displayed_snapshot, Some("current"));
            if phase == PointerPhase::Up {
                assert_eq!(
                    observed.status, "observed_committed_pointer_transaction",
                    "original primary Up must complete the routed activation"
                );
                assert_eq!(
                    observed.matches_previous_down_physical_target,
                    Some(true),
                    "source-owned physical hit is stable across original Down and Up"
                );
                assert_eq!(observed.activation_trace, "collected");
            }
            let serialized = serde_json::to_string(observed).unwrap();
            assert!(!serialized.contains("SENSITIVE"));
            assert!(!serialized.contains("AUTHORED"));
            assert!(!serialized.contains("CAPTION"));
        }
    }

    #[test]
    fn canonical_pointer_trace_reports_unavailable_without_fabricating_an_action() {
        use runenui_core::{Effects, NoHostProtocol, UiApp, View, button};
        use runenui_runtime::{AppRuntime, PumpBudget, RuntimeConfig, TraceConfig};
        struct TraceOff;
        impl UiApp for TraceOff {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;
            fn initial_producer_budget(_: &Self::State) -> runenui_core::ProducerBudget {
                runenui_core::ProducerBudget::new(24, 0).with_reconciliation(4, 0, 4)
            }
            fn action_producer_budget(
                _: &Self::State,
                _: &Self::Action,
            ) -> runenui_core::ProducerBudget {
                runenui_core::ProducerBudget::new(64, 32).with_reconciliation(4, 4, 4)
            }
            fn interaction_producer_budget(_: &Self::State) -> runenui_core::ProducerBudget {
                runenui_core::ProducerBudget::new(64, 32).with_reconciliation(4, 4, 4)
            }
            fn root(_: &Self::State) -> impl View<Self::Action> {
                button("SECRET").on_activate(|| ())
            }
            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                Effects::none()
            }
        }
        let mut runtime = AppRuntime::<TraceOff>::mount_with_config(
            (),
            RuntimeConfig::default().with_trace_config(TraceConfig::new(0)),
        )
        .unwrap();
        let batch = runtime.pump(PumpBudget::new(128, 128, 128, 128)).unwrap();
        assert!(runtime.trace().records().next().is_none());
        assert!(project_native_pointer_trace(runtime.trace(), &batch).is_empty());
        let unavailable = project_one_pointer_trace(
            runtime.trace(),
            WorkSequence::__runtime_new(std::num::NonZeroU64::new(1).unwrap()),
        );
        assert_eq!(unavailable.status, "unavailable_no_canonical_pointer_trace");
        assert_eq!(unavailable.activation_trace, "unavailable");
        assert!(unavailable.physical_target_present.is_none());
    }
    #[test]
    fn capture_is_bounded_and_reports_losses() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        for index in 0..(EVENT_LIMIT + 3) {
            let _scope = sink.span("host", "test_event", Some(index as u64), None);
        }
        let state = sink.inner.lock().unwrap();
        assert_eq!(state.events.len(), EVENT_LIMIT);
        assert_eq!(state.dropped, 3);
        assert_eq!(state.events.front().unwrap().event_ordinal, 4);
        assert_eq!(
            state.events.back().unwrap().event_ordinal,
            (EVENT_LIMIT + 3) as u64
        );
        assert_eq!(state.events.front().unwrap().window, Some(3));
        assert_eq!(
            state.events.back().unwrap().window,
            Some((EVENT_LIMIT + 2) as u64)
        );
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
    fn capture_exports_schema_and_refuses_to_overwrite_an_existing_file() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runenwerk-native-profiler-{}-{nonce}.json",
            std::process::id()
        ));
        let sink = NativeProfilerSink::new(path.clone());
        let mut world = World::new();
        world.insert_resource(RenderFrameObservationPolicyResource::default());
        world.insert_resource(FramePacingPolicyResource::continuous_capped(60));
        let mut windows = WindowStateRegistryResource::default();
        windows.register_primary_window("SENSITIVE WINDOW TITLE", (800, 600), 1.25, true);
        world.insert_resource(windows);
        sink.enable_render_history(&mut world);
        assert!(
            world
                .resource::<RenderFrameObservationPolicyResource>()
                .unwrap()
                .enabled
        );
        {
            let _span = sink.span("host", "window_keyboard", Some(1), None);
        }
        sink.finish(&world, false).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(value["schema_version"], 2);
        assert_eq!(value["status"], "failed_or_interrupted");
        assert_eq!(value["session_id"], sink.session_id.as_ref());
        assert_eq!(value["cpu_events"][0]["event_ordinal"], 1);
        assert_eq!(value["cpu_events"][0]["stage"], "window_keyboard");
        assert_eq!(value["cpu_events"][0]["window"], 1);
        assert_eq!(value["host_pacing"]["mode"], "continuous_capped");
        assert_eq!(value["host_pacing"]["target_fps"], 60);
        assert_eq!(value["native_windows"]["status"], "available");
        assert_eq!(
            value["native_windows"]["windows"][0]["size_px"],
            serde_json::json!([800, 600])
        );
        assert_eq!(value["native_windows"]["windows"][0]["scale_factor"], 1.25);
        assert!(
            !serde_json::to_string(&value)
                .unwrap()
                .contains("SENSITIVE WINDOW TITLE")
        );
        assert_eq!(
            value["render"]["host_frame_join_status"],
            "unavailable_no_owner_correlation"
        );
        assert!(
            sink.finish(&world, true).is_err(),
            "output must not be replaced"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn output_publication_never_overwrites_and_cleans_failed_temporary_files() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "runenwerk-native-profiler-publication-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("capture.json");
        let sink = NativeProfilerSink::new(path.clone());
        let world = World::new();

        std::fs::write(&path, b"original").unwrap();
        assert!(sink.finish(&world, true).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);

        std::fs::remove_file(&path).unwrap();
        sink.finish(&world, true).unwrap();
        let capture: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(capture["status"], "completed");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
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
    fn independent_sessions_never_reuse_a_process_local_identity() {
        let first = NativeProfilerSink::new(PathBuf::from("first-profile.json"));
        let second = NativeProfilerSink::new(PathBuf::from("second-profile.json"));
        assert_ne!(first.session_id, second.session_id);
        let first_clone = first.clone();
        assert_eq!(first.session_id, first_clone.session_id);
        {
            let _event = first.span("host", "first", None, None);
        }
        {
            let _event = first.span("host", "second", None, None);
        }
        let state = first.inner.lock().unwrap();
        assert_eq!(state.events[0].event_ordinal, 1);
        assert_eq!(state.events[1].event_ordinal, 2);
    }

    #[test]
    fn stage_summary_preserves_sample_count_and_percentiles() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        for _ in 0..5 {
            let _scope = sink.span("host", "window_resize", None, None);
        }
        let mut values = sink
            .inner
            .lock()
            .unwrap()
            .events
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for (value, duration) in values.iter_mut().zip([100, 200, 300, 400, 500]) {
            value.duration_ns = duration;
        }
        let stats = summarize_events(&values);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].sample_count, 5);
        assert_eq!(stats[0].median_ns, 300);
        assert_eq!(stats[0].p95_ns, 500);
    }

    #[test]
    fn histogram_bins_use_actual_durations_and_preserve_boundary_semantics() {
        let values = [0, 49_999, 50_000, 99_999, 100_000, 33_333_334];
        let histogram = duration_histogram(&values);
        assert_eq!(
            histogram.upper_bounds_exclusive_ns,
            DURATION_HISTOGRAM_UPPER_NS
        );
        assert_eq!(histogram.counts, [2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(histogram.counts.iter().sum::<usize>(), values.len());
    }

    #[test]
    fn stage_stats_do_not_mix_window_or_ui_slot_identities() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        {
            let _scope = sink.span("host", "window_resize", Some(1), None);
        }
        {
            let _scope = sink.span("host", "window_resize", Some(2), None);
        }
        {
            let _scope = sink.span("runenui_m13", "input", None, Some(7));
        }
        let events = sink
            .inner
            .lock()
            .unwrap()
            .events
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let stats = summarize_events(&events);
        assert_eq!(stats.len(), 3);
        assert_eq!(
            stats
                .iter()
                .filter(|item| item.stage == "window_resize")
                .count(),
            2
        );
    }

    #[test]
    fn failed_m13_pump_does_not_claim_unobserved_settlement_or_receipts() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        sink.record_m13_pump_failure(9, Instant::now(), M13PumpFailureStage::SlotLookup);
        sink.record_m13_pump_failure(9, Instant::now(), M13PumpFailureStage::RuntimePump);
        let state = sink.inner.lock().unwrap();
        assert_eq!(state.events.len(), 2);
        assert_eq!(state.events[0].stage, "canonical_input_slot_lookup_failed");
        assert_eq!(state.events[1].stage, "canonical_input_runtime_pump_failed");
        assert!(
            state
                .events
                .iter()
                .all(|event| event.ui_slot == Some(9) && event.m13.is_none())
        );
    }

    #[test]
    fn other_threads_do_not_inherit_an_active_native_frame() {
        let sink = NativeProfilerSink::new(PathBuf::from("unused-profile.json"));
        let frame = sink.frame_span(17);
        let worker_sink = sink.clone();
        std::thread::spawn(move || {
            let _worker = worker_sink.span("ecs_schedule", "worker", None, None);
        })
        .join()
        .unwrap();
        {
            let _main = sink.span("ecs_schedule", "update", None, None);
        }
        drop(frame);
        let state = sink.inner.lock().unwrap();
        let events = &state.events;
        assert_eq!(events[0].stage, "worker");
        assert_eq!(events[0].host_frame, None);
        assert_eq!(events[1].host_frame, Some(17));
        assert_eq!(events[2].host_frame, Some(17));
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
