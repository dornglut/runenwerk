//! Example-local observations; RunenGPU and the renderer retain execution authority.
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use engine::plugins::render::renderer::RenderGpuFrameTerminalDisposition;
use engine::plugins::render::{Gfx, RenderFlowId, RenderFlowRegistryResource};
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
        }
    }

    /// Called before each canonical host frame, including while this example's flow is absent.
    pub fn observe(
        &mut self,
        world: &mut World,
        flow: &mut RenderFlowId,
        host_frame: u64,
    ) -> Result<bool> {
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
            && self.observed.is_empty()
            && self.accepted == self.completed;
        match self.phase {
            Phase::Running if self.completed >= self.options.frames => {
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
                    self.transition(Phase::Recovery);
                }
            }
            Phase::Recovery if self.completed - self.recovery_start >= 120 => {
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

    pub fn changing(&self) -> bool {
        self.options.content == Content::Changing
            && matches!(self.phase, Phase::Running | Phase::Recovery)
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
