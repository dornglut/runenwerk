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
    OnDemandStatic,
    OnDemandChanging,
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub content: Content,
    pub frames: u64,
}

impl Options {
    pub fn on_demand(self) -> bool {
        matches!(
            self.content,
            Content::OnDemandStatic | Content::OnDemandChanging
        )
    }
}

/// Bounded OnDemand regression consumer, separate from frame-driven lifetime
/// bookkeeping. GPU execution and UI terminal truth remain with their owners.
pub struct OnDemandProbe {
    options: Options,
    pub report: Arc<Mutex<Report>>,
    ui: UiProgress,
    next_action: Option<u64>,
    terminal_ids: BTreeSet<GpuSubmissionId>,
    drained: bool,
    progress_turns: u64,
}

impl OnDemandProbe {
    pub fn new(options: Options, report: Arc<Mutex<Report>>) -> Self {
        Self {
            options,
            report,
            ui: UiProgress::default(),
            next_action: None,
            terminal_ids: BTreeSet::new(),
            drained: false,
            progress_turns: 0,
        }
    }

    pub fn take_action(&mut self) -> Option<u64> {
        self.next_action.take()
    }

    /// Consumes existing terminal delivery only; no poll, frame observer,
    /// action mutation, schedule execution, or visual submission occurs here.
    pub fn terminal_progress(
        &mut self,
        world: &mut World,
        slot: UiRuntimeSlotId,
        flow: RenderFlowId,
    ) -> Result<bool> {
        self.progress_turns += 1;
        let observation = UiObservation::read(world, slot)?;
        let gfx = world.resource_mut::<Gfx>()?;
        let batch = gfx.take_gpu_frame_terminal_outcomes();
        anyhow::ensure!(
            batch.delivery_enabled() && !batch.has_delivery_gap(),
            "OnDemand terminal delivery gap"
        );
        let mut exact_publications = Vec::new();
        for terminal in batch.outcomes() {
            anyhow::ensure!(
                terminal.affinity() == gfx.gpu_context_affinity()
                    && terminal.surface() == RenderSurfaceId::primary().raw()
                    && terminal
                        .invocations()
                        .iter()
                        .any(|invocation| invocation.flow_id() == flow),
                "OnDemand terminal belongs to another context, surface or flow"
            );
            anyhow::ensure!(
                matches!(
                    terminal.disposition(),
                    RenderGpuFrameTerminalDisposition::Completed
                ),
                "OnDemand submission failed: {:?}",
                terminal.disposition()
            );
            anyhow::ensure!(
                self.terminal_ids.insert(terminal.submission_id()),
                "duplicate terminal submission"
            );
            exact_publications.extend(
                terminal
                    .completed_ui_publications()
                    .iter()
                    .map(|id| id.raw()),
            );
            println!(
                "RW1392_TERMINAL submission={} frame={} publications={:?} disposition={:?} observation_age_us={}",
                terminal.submission_id(),
                terminal.frame(),
                terminal.completed_ui_publications(),
                terminal.disposition(),
                terminal.host_observed_terminal_age().as_micros()
            );
        }
        let execution = gfx.execution_stats();
        let pending = gfx.pending_gpu_frame_submissions().len();
        println!(
            "RW1392_PROGRESS turn={} submissions={} readbacks={} renderer_pending={} terminal_count={} intake={:?} displayed={:?}",
            self.progress_turns,
            execution.in_flight_submissions(),
            execution.pending_readbacks(),
            pending,
            self.terminal_ids.len(),
            observation.intake,
            observation.displayed
        );
        // A displayed snapshot alone cannot acknowledge this consumer's wait:
        // it must be in this exact once-delivered completed submission batch.
        if self.continuation_for_terminal(observation, &exact_publications) {
            return Ok(true);
        }
        if !self.drained
            && pending == 0
            && execution.in_flight_submissions() == 0
            && execution.pending_readbacks() == 0
            && observation.ready()
            && !self.terminal_ids.is_empty()
            && (self.options.content == Content::OnDemandStatic
                || self.ui.acknowledged >= self.options.frames)
        {
            self.drained = true;
            self.report
                .lock()
                .map_err(|_| anyhow::anyhow!("report poisoned"))?
                .complete = true;
            println!(
                "RW1392_DRAINED content={:?} actions={} terminal_submissions={} progress_turns={} scanout=unobserved",
                self.options.content,
                self.ui.acknowledged,
                self.terminal_ids.len(),
                self.progress_turns
            );
            // Optional external idle observation changes only the final close,
            // after the same actions and exact terminal/readback drain.
            if self.options.content == Content::OnDemandChanging
                && std::env::var_os("RUNENWERK_ONDEMAND_HOLD_AFTER_DRAIN").is_none()
            {
                let record = world
                    .resource_mut::<engine::runtime::WindowStateRegistryResource>()?
                    .record_mut(NativeWindowId::primary())
                    .context("primary missing at completed OnDemand close")?;
                record.request_close();
                record.approve_close();
            }
        }
        Ok(false)
    }

    fn continuation_for_terminal(
        &mut self,
        observation: UiObservation,
        publications: &[u64],
    ) -> bool {
        if !observation.ready()
            || !observation
                .displayed
                .is_some_and(|(id, _)| publications.contains(&id))
        {
            return false;
        }
        self.ui.observe(observation);
        if self.options.content == Content::OnDemandChanging
            && self.ui.acknowledged < self.options.frames
            && self.next_action.is_none()
            && let Some(generation) = self.ui.request()
        {
            self.next_action = Some(generation);
            println!(
                "RW1392_CONTINUATION generation={generation} completed_publication={:?}",
                observation.displayed
            );
            return true;
        }
        false
    }
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
        "--ondemand-static" => Content::OnDemandStatic,
        "--ondemand-changing" => Content::OnDemandChanging,
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
    peak_resources: usize,
    peak_bindings: usize,
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
            peak_resources: 0,
            peak_bindings: 0,
            ui: UiProgress::default(),
            qualified_completed: 0,
            recovery_qualified: 0,
            recovery_changes: 0,
            last_ui_ready: Instant::now(),
        }
    }

    /// Called before each canonical host frame, including while this example's flow is absent.
    pub fn observe(
        &mut self,
        world: &mut World,
        flow: &mut RenderFlowId,
        host_frame: u64,
        slot: UiRuntimeSlotId,
    ) -> Result<bool> {
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
        let (resources, bindings, execution, pending) = {
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
                if self
                    .ui
                    .credits(self.options.content, outcome.completed_ui_publications())
                {
                    self.qualified_completed += 1;
                    self.last_ui_ready = Instant::now();
                    if self.options.content != Content::Control {
                        println!(
                            "U9C_UI_FRAME submission={} gpu_frame={} publications={:?}",
                            outcome.submission_id(),
                            outcome.frame(),
                            outcome.completed_ui_publications()
                        );
                    }
                }
            }
            (
                gfx.resource_realization_stats(),
                gfx.program_binding_realization_stats(),
                gfx.execution_stats(),
                gfx.pending_gpu_frame_submissions().len(),
            )
        };
        self.peak_resources = self.peak_resources.max(resources.retained_records());
        self.peak_bindings = self.peak_bindings.max(bindings.retained_records());
        println!(
            "U9C_WORKLOAD frame={host_frame} ui_ready={} qualified_completed={} published_changes={}",
            self.ui.latest.ready(),
            self.qualified_completed,
            self.ui.acknowledged
        );
        println!(
            "U9C_RESOURCE frame={host_frame} phase={:?} accepted={} completed={} observed_pending={} renderer_pending={pending} g4c1={} g4c1_limit={} buffers={} textures={} views={} samplers={} g4c2={} g4c2_limit={} groups={} g4c2_pending={} g5_in_flight={} peak_g4c1={} peak_g4c2={}",
            self.phase,
            self.accepted,
            self.completed,
            self.observed.len(),
            resources.retained_records(),
            resources.max_records(),
            resources.buffers(),
            resources.textures(),
            resources.texture_views(),
            resources.samplers(),
            bindings.retained_records(),
            bindings.max_records(),
            bindings.bind_groups(),
            bindings.in_flight_records(),
            execution.in_flight_submissions(),
            self.peak_resources,
            self.peak_bindings
        );
        let drained = pending == 0
            && execution.in_flight_submissions() == 0
            && execution.pending_readbacks() == 0
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
                            "U9C_LIFETIME_COMPLETE content={:?} requested_completed={} accepted={} completed={} recovery_completed={} peak_g4c1={} peak_g4c2={} scanout=unobserved memory=external_operator",
                            self.options.content,
                            self.options.frames,
                            self.accepted,
                            self.completed,
                            self.completed - self.recovery_start,
                            self.peak_resources,
                            self.peak_bindings
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
    fn ondemand_consumer_requests_once_per_exact_completed_generation() {
        let mut probe = OnDemandProbe::new(
            Options {
                content: Content::OnDemandChanging,
                frames: 16,
            },
            Arc::new(Mutex::new(Report::default())),
        );
        let initial = UiObservation {
            intake: Some((1, 1)),
            displayed: Some((1, 1)),
            applied_generation: 0,
        };
        assert!(!probe.continuation_for_terminal(initial, &[]));
        assert!(!probe.continuation_for_terminal(initial, &[2]));
        assert!(probe.continuation_for_terminal(initial, &[1]));
        assert_eq!(probe.take_action(), Some(1));
        for _ in 0..3 {
            assert!(!probe.continuation_for_terminal(initial, &[1]));
        }
        assert!(probe.take_action().is_none());
        let accepted_only = UiObservation {
            intake: Some((2, 2)),
            displayed: Some((1, 1)),
            applied_generation: 1,
        };
        assert!(!probe.continuation_for_terminal(accepted_only, &[1]));
        let next = UiObservation {
            intake: Some((2, 2)),
            displayed: Some((2, 2)),
            applied_generation: 1,
        };
        assert!(!probe.continuation_for_terminal(next, &[1]));
        assert!(probe.continuation_for_terminal(next, &[2]));
        assert_eq!(probe.ui.acknowledged, 1);
        assert_eq!(probe.take_action(), Some(2));
    }

    #[test]
    fn ondemand_static_consumer_never_requests_an_application_frame() {
        let mut probe = OnDemandProbe::new(
            Options {
                content: Content::OnDemandStatic,
                frames: 1,
            },
            Arc::new(Mutex::new(Report::default())),
        );
        let shown = UiObservation {
            intake: Some((1, 1)),
            displayed: Some((1, 1)),
            applied_generation: 0,
        };
        assert!(!probe.continuation_for_terminal(shown, &[1]));
        assert!(probe.take_action().is_none());
    }

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
