//! Example-local observations; RunenGPU and the renderer retain execution authority.
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use engine::plugins::render::RunenUiPaintSubmissionRegistryResource;
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::render::renderer::RenderGpuFrameTerminalDisposition;
use engine::plugins::render::{Gfx, RenderFlowId, RenderFlowRegistryResource};
use engine::plugins::ui::{
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotId, UiRuntimeSlotsResource,
};
use engine::runtime::NativeWindowId;
use runen_ecs::World;
use runen_gpu::GpuSubmissionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Content {
    Static,
    Changing,
    Control,
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub content: Content,
    pub frames: u64,
}

pub fn parse(args: &[String]) -> Result<Option<Options>> {
    if args == ["--interactive"] {
        return Ok(None);
    }
    anyhow::ensure!(
        args.len() == 3 && args[1] == "--frames",
        "use --interactive, or --lifetime-static|--lifetime-changing|--lifetime-control --frames N"
    );
    let content = match args[0].as_str() {
        "--lifetime-static" => Content::Static,
        "--lifetime-changing" => Content::Changing,
        "--lifetime-control" => Content::Control,
        _ => anyhow::bail!("unknown qualification mode"),
    };
    let frames = args[2]
        .parse::<u64>()
        .context("frames must be a positive integer")?;
    anyhow::ensure!(frames > 0, "frames must be positive");
    Ok(Some(Options { content, frames }))
}

#[derive(Debug, Default)]
pub struct Report {
    pub complete: bool,
    pub failure: Option<String>,
}

impl Report {
    pub fn record_failure(&mut self, error: String) {
        self.complete = false;
        self.failure.get_or_insert(error);
    }

    pub fn ensure_complete(&self) -> Result<()> {
        anyhow::ensure!(
            self.failure.is_none(),
            "qualification observation failed: {:?}",
            self.failure
        );
        anyhow::ensure!(
            self.complete,
            "qualification stopped before workload, idle/recovery and final drain completed"
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct UiObservation {
    intake: Option<(u64, u64)>,
    displayed: Option<(u64, u64)>,
    applied_generation: u64,
}

impl UiObservation {
    fn read(world: &World, slot: UiRuntimeSlotId) -> Result<Self> {
        let applied_generation = world
            .resource::<UiRuntimeSlotsResource>()?
            .observe_app::<super::QualificationUi, _>(slot, |app| app.state().text_generation)
            .map_err(|error| anyhow::anyhow!("UI state unavailable: {error:?}"))?;
        let associations = world.resource::<UiRuntimePresentationAssociationsResource>()?;
        let binding = associations
            .binding(slot)
            .context("qualification UI binding missing")?;
        anyhow::ensure!(
            binding.render_surface_id == RenderSurfaceId::primary(),
            "qualification UI surface changed"
        );
        let intake = world
            .resource::<RunenUiPaintSubmissionRegistryResource>()?
            .get_for_surface(&binding.producer_id, binding.render_surface_id)
            .map(|paint| {
                (
                    paint.publication_id.raw(),
                    paint.publication.revision().get(),
                )
            });
        let displayed = associations
            .displayed_for_native_recipient(slot, NativeWindowId::primary())
            .filter(|shown| {
                shown.producer_id == binding.producer_id
                    && shown.render_surface_id == binding.render_surface_id
            })
            .map(|shown| (shown.publication_id.raw(), shown.paint_revision.get()));
        Ok(Self {
            intake,
            displayed,
            applied_generation,
        })
    }

    fn ready(self) -> bool {
        self.intake.is_some() && self.intake == self.displayed
    }
}

struct ChangeRequest {
    generation: u64,
    before: (u64, u64),
    started: Instant,
}

#[derive(Default)]
struct UiProgress {
    latest: UiObservation,
    requested: u64,
    acknowledged: u64,
    pending: Option<ChangeRequest>,
}

impl UiProgress {
    fn observe(&mut self, observation: UiObservation) {
        self.latest = observation;
        if let Some(request) = &self.pending
            && observation.ready()
            && observation.applied_generation == request.generation
            && let Some(now) = observation.intake
            && now.0 > request.before.0
            && now.1 > request.before.1
        {
            self.acknowledged += 1;
            println!(
                "U9C_UI_CHANGE_ACK generation={} publication={} paint_revision={}",
                request.generation, now.0, now.1
            );
            self.pending = None;
        }
    }

    fn request(&mut self) -> Option<u64> {
        if self.pending.is_some() || !self.latest.ready() {
            return None;
        }
        self.requested += 1;
        self.pending = Some(ChangeRequest {
            generation: self.requested,
            before: self.latest.intake?,
            started: Instant::now(),
        });
        Some(self.requested)
    }

    fn credits(
        &self,
        content: Content,
        publications: &[engine::plugins::render::RunenUiPublicationId],
    ) -> bool {
        if content == Content::Control {
            return publications.is_empty();
        }
        self.latest.ready()
            && self.latest.intake.is_some_and(|(id, _)| {
                publications
                    .iter()
                    .any(|publication| publication.raw() == id)
            })
    }

    fn qualifies(&self, content: Content, frames: u64, completed: u64, change_origin: u64) -> bool {
        completed >= frames
            && self.pending.is_none()
            && (content == Content::Control || self.latest.ready())
            && (content != Content::Changing || self.acknowledged - change_origin >= frames)
    }
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Running,
    Drain,
    Idle,
    Recovery,
    FinalDrain,
}

pub struct Probe {
    pub options: Options,
    pub report: Arc<Mutex<Report>>,
    phase: Phase,
    since: Instant,
    observed: BTreeSet<GpuSubmissionId>,
    accepted: u64,
    completed: u64,
    recovery_start: u64,
    idle_accepted: u64,
    enabled: bool,
    peak_gpu_in_flight: usize,
    peak_pending_readbacks: usize,
    ui: UiProgress,
    qualified_completed: u64,
    recovery_qualified: u64,
    recovery_changes: u64,
    last_ui_ready: Instant,
}

impl Probe {
    pub fn new(options: Options, report: Arc<Mutex<Report>>) -> Self {
        Self {
            options,
            report,
            phase: Phase::Running,
            since: Instant::now(),
            observed: BTreeSet::new(),
            accepted: 0,
            completed: 0,
            recovery_start: 0,
            idle_accepted: 0,
            enabled: false,
            peak_gpu_in_flight: 0,
            peak_pending_readbacks: 0,
            ui: UiProgress::default(),
            qualified_completed: 0,
            recovery_qualified: 0,
            recovery_changes: 0,
            last_ui_ready: Instant::now(),
        }
    }

    // The current public renderer outcome carries G5 submission terminality,
    // but no same-submission RunenUI terminal-Present publication IDs. Only
    // Control (GPU-only) can be qualified without that missing original proof.
    // The independently correct displayed-input ledger must NOT be retrofitted
    // to an unrelated GPU terminal outcome to produce a synthetic per-frame PASS.
    fn require_terminal_evidence(&self) -> Result<()> {
        anyhow::ensure!(
            self.options.content == Content::Control,
            "RunenUI static/changing GPU lifetime qualification HOLD: the current public GPU terminal outcome does not carry correlated completed-Present publication IDs; existing UI displayed state cannot replace exact per-submission proof"
        );
        Ok(())
    }

    /// Called before each canonical host frame, including while this example's flow is absent.
    pub fn observe(
        &mut self,
        world: &mut World,
        flow: &mut RenderFlowId,
        host_frame: u64,
        slot: UiRuntimeSlotId,
    ) -> Result<bool> {
        self.require_terminal_evidence()?;
        if self.options.content != Content::Control {
            self.ui.observe(UiObservation::read(world, slot)?);
            if matches!(self.phase, Phase::Running | Phase::Recovery) {
                anyhow::ensure!(
                    self.last_ui_ready.elapsed() < Duration::from_secs(15),
                    "UI contribution/completed-Present witness did not become available"
                );
                anyhow::ensure!(
                    self.ui
                        .pending
                        .as_ref()
                        .is_none_or(|request| request.started.elapsed() < Duration::from_secs(15)),
                    "changing text did not reach applied and completed publication"
                );
            }
        }
        let (execution, pending) = {
            let gfx = world.resource_mut::<Gfx>()?;
            if !self.enabled {
                gfx.set_gpu_frame_terminal_delivery(true);
                self.enabled = true;
                println!("U9C_ADAPTER facts={:?}", gfx.adapter_facts());
            }
            gfx.progress_gpu_before_producer_update()?;
            // Observe both handles that remain pending and submissions that accepted and
            // terminalized between polls. Keep only outstanding IDs, never GPU handles.
            for receipt in gfx.pending_gpu_frame_submissions() {
                anyhow::ensure!(
                    receipt.surface() == RenderSurfaceId::primary().raw()
                        && receipt
                            .invocations()
                            .iter()
                            .any(|invocation| invocation.flow_id() == *flow),
                    "unexpected flow/surface submission in qualification"
                );
                if self.observed.insert(receipt.submission().id()) {
                    self.accepted += 1;
                }
            }
            let batch = gfx.take_gpu_frame_terminal_outcomes();
            anyhow::ensure!(
                batch.delivery_enabled() && !batch.has_delivery_gap(),
                "terminal delivery disabled or incomplete"
            );
            for outcome in batch.outcomes() {
                anyhow::ensure!(
                    outcome.surface() == RenderSurfaceId::primary().raw()
                        && outcome
                            .invocations()
                            .iter()
                            .any(|invocation| invocation.flow_id() == *flow),
                    "unexpected terminal flow/surface in qualification"
                );
                if !self.observed.remove(&outcome.submission_id()) {
                    self.accepted += 1;
                }
                anyhow::ensure!(
                    matches!(
                        outcome.disposition(),
                        RenderGpuFrameTerminalDisposition::Completed
                    ),
                    "accepted frame failed: {:?}",
                    outcome.disposition()
                );
                self.completed += 1;
                // No per-submission UI terminal-publication proof is exported
                // by this Gfx outcome. An empty slice preserves the original
                // strict UI credit rule: control-only GPU work can advance;
                // UI-required work fails the explicit gate above.
                if self.ui.credits(self.options.content, &[]) {
                    self.qualified_completed += 1;
                    self.last_ui_ready = Instant::now();
                    println!(
                        "U9C_GPU_TERMINAL submission={} gpu_frame={} ui_per_submission=unavailable",
                        outcome.submission_id(),
                        outcome.frame()
                    );
                }
            }
            (gfx.execution_stats(), gfx.pending_gpu_frame_submissions().len())
        };
        self.peak_gpu_in_flight = self.peak_gpu_in_flight.max(execution.in_flight_submissions());
        self.peak_pending_readbacks = self.peak_pending_readbacks.max(execution.pending_readbacks());
        println!(
            "U9C_WORKLOAD frame={host_frame} ui_ready={} qualified_completed={} published_changes={}",
            self.ui.latest.ready(),
            self.qualified_completed,
            self.ui.acknowledged
        );
        println!(
            "U9C_EXECUTION frame={host_frame} phase={:?} accepted={} completed={} observed_pending={} renderer_pending={pending} g5_in_flight={} prepared={} pending_readbacks={} peak_g5_in_flight={} peak_pending_readbacks={} g4c1_realization=unavailable g4c2_realization=unavailable",
            self.phase,
            self.accepted,
            self.completed,
            self.observed.len(),
            execution.in_flight_submissions(),
            execution.prepared_submissions(),
            execution.pending_readbacks(),
            self.peak_gpu_in_flight,
            self.peak_pending_readbacks
        );
        let drained = pending == 0
            && execution.in_flight_submissions() == 0
            && self.observed.is_empty()
            && self.accepted == self.completed;
        match self.phase {
            Phase::Running
                if self.ui.qualifies(
                    self.options.content,
                    self.options.frames,
                    self.qualified_completed,
                    0,
                ) =>
            {
                anyhow::ensure!(
                    world
                        .resource_mut::<RenderFlowRegistryResource>()?
                        .remove_flow(*flow),
                    "qualification flow missing at drain"
                );
                self.transition(Phase::Drain);
            }
            Phase::Drain | Phase::FinalDrain => {
                anyhow::ensure!(
                    self.since.elapsed() < Duration::from_secs(15),
                    "accepted work did not drain within 15 seconds"
                );
                if drained {
                    if matches!(self.phase, Phase::FinalDrain) {
                        anyhow::ensure!(
                            self.ui.pending.is_none(),
                            "text change still pending at final drain"
                        );
                        self.report
                            .lock()
                            .map_err(|_| anyhow::anyhow!("report poisoned"))?
                            .complete = true;
                        println!(
                            "U9C_LIFETIME_COMPLETE content={:?} requested_completed={} accepted={} completed={} recovery_completed={} peak_g5_in_flight={} peak_pending_readbacks={} g4c1_realization=unobserved g4c2_realization=unobserved scanout=unobserved memory=external_operator",
                            self.options.content,
                            self.options.frames,
                            self.accepted,
                            self.completed,
                            self.completed - self.recovery_start,
                            self.peak_gpu_in_flight,
                            self.peak_pending_readbacks
                        );
                        return Ok(true);
                    }
                    self.idle_accepted = self.accepted;
                    self.transition(Phase::Idle);
                }
            }
            Phase::Idle => {
                anyhow::ensure!(
                    drained && self.accepted == self.idle_accepted,
                    "GPU work appeared while qualification flow was idle"
                );
                if self.since.elapsed() >= Duration::from_secs(3) {
                    let resumed =
                        super::qualification_flow(self.options.content != Content::Control)?;
                    *flow = resumed.id();
                    world
                        .resource_mut::<RenderFlowRegistryResource>()?
                        .upsert_flow(resumed);
                    self.recovery_start = self.completed;
                    self.recovery_qualified = self.qualified_completed;
                    self.recovery_changes = self.ui.acknowledged;
                    self.last_ui_ready = Instant::now();
                    self.transition(Phase::Recovery);
                }
            }
            Phase::Recovery
                if self.ui.qualifies(
                    self.options.content,
                    120,
                    self.qualified_completed - self.recovery_qualified,
                    self.recovery_changes,
                ) =>
            {
                anyhow::ensure!(
                    world
                        .resource_mut::<RenderFlowRegistryResource>()?
                        .remove_flow(*flow),
                    "recovery flow missing"
                );
                self.transition(Phase::FinalDrain);
            }
            _ => {}
        }
        Ok(false)
    }

    pub fn next_text_change(&mut self) -> Option<u64> {
        if self.options.content == Content::Changing
            && matches!(self.phase, Phase::Running | Phase::Recovery)
        {
            self.ui.request()
        } else {
            None
        }
    }

    fn transition(&mut self, phase: Phase) {
        self.phase = phase;
        self.since = Instant::now();
        println!(
            "U9C_PHASE phase={phase:?} accepted={} completed={}",
            self.accepted, self.completed
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_observation_failure_invalidates_completed_report_and_preserves_first_error() {
        let mut report = Report {
            complete: true,
            failure: None,
        };
        report.record_failure("missing presentation observation".to_owned());
        report.record_failure("later close failure".to_owned());
        assert!(!report.complete);
        assert!(report.ensure_complete().is_err());
        assert_eq!(
            report.failure.as_deref(),
            Some("missing presentation observation")
        );
    }

    #[test]
    fn unavailable_per_submission_ui_terminal_receipts_fail_closed() {
        for content in [Content::Static, Content::Changing] {
            let probe = Probe::new(content_options(content), Arc::new(Mutex::new(Report::default())));
            assert!(probe.require_terminal_evidence().is_err());
        }
        let control = Probe::new(
            content_options(Content::Control),
            Arc::new(Mutex::new(Report::default())),
        );
        assert!(control.require_terminal_evidence().is_ok());
    }

    fn content_options(content: Content) -> Options {
        Options { content, frames: 1 }
    }

    #[test]
    fn gpu_completion_cannot_replace_ui_or_advancing_published_text() {
        let mut ui = UiProgress::default();
        assert!(!ui.qualifies(Content::Static, 1, 10_000, 0));
        assert!(!ui.qualifies(Content::Changing, 1, 10_000, 0));
        assert!(ui.qualifies(Content::Control, 1, 1, 0));
        ui.observe(UiObservation {
            intake: Some((1, 1)),
            displayed: None,
            applied_generation: 0,
        });
        assert!(!ui.qualifies(Content::Static, 1, 10_000, 0));
        ui.observe(UiObservation {
            intake: Some((1, 1)),
            displayed: Some((1, 1)),
            applied_generation: 0,
        });
        assert!(ui.qualifies(Content::Static, 1, 1, 0));
        // Historical displayed state cannot credit clear-only later submissions.
        let publication = engine::plugins::render::RunenUiPublicationId::try_from_raw(1).unwrap();
        assert!(!ui.credits(Content::Static, &[]));
        assert!(ui.credits(Content::Static, &[publication]));
        let other = engine::plugins::render::RunenUiPublicationId::try_from_raw(2).unwrap();
        assert!(!ui.credits(Content::Static, &[other]));
        assert_eq!(ui.request(), Some(1));
        // Applied state alone, fresh paint alone, and a nonmatching completed
        // publication all fail to acknowledge the requested text generation.
        for observation in [
            UiObservation {
                intake: Some((1, 1)),
                displayed: Some((1, 1)),
                applied_generation: 1,
            },
            UiObservation {
                intake: Some((2, 2)),
                displayed: Some((2, 2)),
                applied_generation: 0,
            },
            UiObservation {
                intake: Some((2, 2)),
                displayed: Some((1, 1)),
                applied_generation: 1,
            },
        ] {
            ui.observe(observation);
            assert!(!ui.qualifies(Content::Changing, 1, 10_000, 0));
        }
        ui.observe(UiObservation {
            intake: Some((2, 2)),
            displayed: Some((2, 2)),
            applied_generation: 1,
        });
        assert!(ui.qualifies(Content::Changing, 1, 1, 0));
        assert_eq!(ui.acknowledged, 1);
        assert_eq!(ui.request(), Some(2));
        assert!(
            !ui.qualifies(Content::Changing, 1, 10_000, 0),
            "pending next change must prevent drain despite prior completed target"
        );
    }
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| (*s).to_owned()).collect()
    }
    #[test]
    fn qualification_modes_require_explicit_positive_workload_and_reject_ambiguity() {
        assert!(parse(&args(&["--interactive"])).unwrap().is_none());
        assert_eq!(
            parse(&args(&["--lifetime-changing", "--frames", "2048"]))
                .unwrap()
                .unwrap()
                .content,
            Content::Changing
        );
        for bad in [
            &["--lifetime-static"][..],
            &["--lifetime-static", "--frames", "0"],
            &["--lifetime-static", "--frames", "-1"],
            &["--interactive", "--frames", "2048"],
            &["--lifetime-control", "--frames", "oops"],
        ] {
            assert!(parse(&args(bad)).is_err());
        }
    }
}
