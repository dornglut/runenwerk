//! Native OnDemand completion qualification. No synthetic redraws or GPU polls.
//! Run under an external watchdog; an idle Host cannot enforce its own timeout.
use anyhow::{Result, ensure};
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::render::{
    Gfx, RenderFlow, RenderFrameProducerId, RunenUiPaintSubmissionRegistryResource,
    RunenUiPublicationId,
};
use engine::plugins::ui::{
    UiRuntimeFontConfiguration, UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
    UiRuntimePresentationBinding, UiRuntimeSlotId, UiRuntimeSlotsResource, UiTypedScreenId,
};
use engine::plugins::{RenderPlugin, UiPlugin};
use engine::prelude::*;
use engine::runtime::{
    NativeWindowHook, NativeWindowHookRegistryResource, NativeWindowId, WindowStateRegistryResource,
};
use runenui_core::{
    Effects, FontFamilyName, GenericFontFamily, NoHostProtocol, StyleEnvironment, UiApp, View, text,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::event::{DeviceEvent, WindowEvent};
use winit::window::Window;

struct TextApp;
impl UiApp for TextApp {
    type State = u64;
    type Action = u64;
    type HostProtocol = NoHostProtocol;
    fn root(value: &u64) -> impl View<u64> {
        text(format!("OnDemand GPU progress — action {value}"))
    }
    fn update(
        state: &mut u64,
        value: u64,
    ) -> impl runenui_core::IntoUpdateOutput<u64, NoHostProtocol> {
        *state = value;
        Effects::none()
    }
}

#[derive(Default)]
struct Evidence {
    frames: u64,
    actions: u64,
    accepted: BTreeSet<u64>,
    completed: BTreeSet<u64>,
    input_events: u64,
    terminal_callbacks: u64,
    requests: u64,
    held: BTreeMap<u64, (runen_gpu::GpuSubmission, Instant)>,
    complete: bool,
    last_accepted_frame: Option<u64>,
}

#[derive(runen_ecs::Resource)]
struct Probe {
    slot: UiRuntimeSlotId,
    target: u64,
    evidence: Arc<Mutex<Evidence>>,
    last_publication: Option<u64>,
    waiting: Option<WaitingAction>,
    ready: Option<(WaitingAction, Instant)>,
    on_demand: bool,
    capture: bool,
}

struct WaitingAction {
    publication: RunenUiPublicationId,
    mapping: UiRuntimeNativeMapping,
    submission: runen_gpu::GpuSubmission,
}

fn observe_terminal(world: &mut World) -> Result<()> {
    let shared = world.resource::<Probe>()?.evidence.clone();
    let mut evidence = shared.lock().unwrap();
    ensure!(
        evidence.input_events == 0,
        "external input contaminated qualification"
    );
    let gfx = world.resource_mut::<Gfx>()?;
    let batch = gfx.take_gpu_frame_terminal_outcomes();
    ensure!(!batch.has_delivery_gap(), "terminal delivery gap");
    for terminal in batch.outcomes() {
        ensure!(
            matches!(
                terminal.disposition(),
                engine::plugins::render::renderer::RenderGpuFrameTerminalDisposition::Completed
            ),
            "GPU failure: {terminal:?}"
        );
        let id = terminal.submission_id().get();
        evidence.accepted.insert(id);
        ensure!(evidence.completed.insert(id), "duplicate terminal {id}");
        println!("terminal {terminal:?}");
    }
    evidence
        .held
        .retain(|id, (submission, accepted_observed_at)| {
            if !matches!(
                submission.status(),
                runen_gpu::GpuSubmissionStatus::Accepted
            ) {
                // Measures FrameEnd receipt observation to first terminal observation,
                // including Host scheduling. This is not a GPU execution duration.
                println!(
                    "terminal_observation_latency_ms id={id} elapsed={:.3}",
                    accepted_observed_at.elapsed().as_secs_f64() * 1000.0
                );
            }
            let readbacks = submission
                .readbacks()
                .iter()
                .map(|r| {
                    let status = match r.status() {
                        runen_gpu::GpuReadbackStatus::Pending => "Pending".to_owned(),
                        runen_gpu::GpuReadbackStatus::Ready(bytes) => {
                            format!("Ready({} bytes)", bytes.as_bytes().len())
                        }
                        runen_gpu::GpuReadbackStatus::Failed(failure) => {
                            format!("Failed({failure:?})")
                        }
                    };
                    (r.id(), status)
                })
                .collect::<Vec<_>>();
            println!(
                "submission_status id={id} status={:?} readbacks={:?}",
                submission.status(),
                readbacks
            );
            matches!(
                submission.status(),
                runen_gpu::GpuSubmissionStatus::Accepted
            ) || submission
                .readbacks()
                .iter()
                .any(|r| matches!(r.status(), runen_gpu::GpuReadbackStatus::Pending))
        });
    println!(
        "gpu_state in_flight={} readbacks={} prepared={}",
        gfx.execution_stats().in_flight_submissions(),
        gfx.execution_stats().pending_readbacks(),
        gfx.execution_stats().prepared_submissions()
    );
    Ok(())
}

fn observe_frame_end(mut world: WorldMut) -> Result<()> {
    let probe = world.resource::<Probe>()?;
    let shared = probe.evidence.clone();
    let mut evidence = shared.lock().unwrap();
    evidence.frames += 1;
    println!(
        "frame_interval_ms={:.3}",
        world
            .resource::<engine::runtime::FramePacingRuntimeStateResource>()?
            .last_frame_interval_ms
    );
    let receipts = world.resource::<Gfx>()?.pending_gpu_frame_submissions();
    let new_receipt = receipts
        .iter()
        .filter(|r| !evidence.accepted.contains(&r.submission().id().get()))
        .max_by_key(|r| r.frame())
        .cloned();
    for receipt in &receipts {
        let id = receipt.submission().id().get();
        evidence.accepted.insert(id);
        evidence
            .held
            .entry(id)
            .or_insert_with(|| (receipt.submission().clone(), Instant::now()));
        evidence.last_accepted_frame = Some(receipt.frame());
        println!(
            "accepted id={id} frame={} surface={} affinity={:?} status={:?}",
            receipt.frame(),
            receipt.surface(),
            receipt.submission().affinity(),
            receipt.submission().status()
        );
    }
    println!(
        "frame_end frames={} actions={} accepted={:?} completed={:?} input_events={}",
        evidence.frames,
        evidence.actions,
        evidence.accepted,
        evidence.completed,
        evidence.input_events
    );
    drop(evidence);
    observe_terminal(&mut world)?;
    let (slot, target) = {
        let probe = world.resource::<Probe>()?;
        (probe.slot, probe.target)
    };
    if target != 0 {
        let paint = world
            .resource::<RunenUiPaintSubmissionRegistryResource>()?
            .get_for_surface(
                &RenderFrameProducerId::try_from_raw(1).unwrap(),
                RenderSurfaceId::primary(),
            );
        let record = world
            .resource::<WindowStateRegistryResource>()?
            .record(NativeWindowId::primary())
            .unwrap();
        let mapping = UiRuntimeNativeMapping::new(
            NativeWindowId::primary(),
            RenderSurfaceId::primary(),
            record.size_px,
            record.scale_factor,
        )?;
        if world.resource::<UiRuntimeSlotsResource>()?.contains(slot)
            && let (Some(receipt), Some(paint)) = (new_receipt, paint)
        {
            let wait = WaitingAction {
                publication: paint.publication_id,
                mapping,
                submission: receipt.submission().clone(),
            };
            let probe = world.resource_mut::<Probe>()?;
            if probe.last_publication != Some(wait.publication.raw()) && probe.ready.is_none() {
                probe.waiting = Some(wait);
            }
        }
    }
    Ok(())
}

// The historical qualification consumer acts in the ordinary native frame
// hook, before the Host's full-slot settlement. Submitting in Update instead
// legitimately needs a second frame on the accepted native settlement contract.
fn application_action(world: &mut World) -> Result<()> {
    if let Some(record) = world
        .resource_mut::<WindowStateRegistryResource>()?
        .record_mut(NativeWindowId::primary())
        && record.close_intent_pending
    {
        record.approve_close();
        return Ok(());
    }
    let (slot, target, shared, previous) = {
        let probe = world.resource::<Probe>()?;
        (
            probe.slot,
            probe.target,
            probe.evidence.clone(),
            probe.last_publication,
        )
    };
    let record = world
        .resource::<WindowStateRegistryResource>()?
        .record(NativeWindowId::primary())
        .unwrap();
    let mapping = UiRuntimeNativeMapping::new(
        NativeWindowId::primary(),
        RenderSurfaceId::primary(),
        record.size_px,
        record.scale_factor,
    )?;
    let publication = world
        .resource::<UiRuntimePresentationAssociationsResource>()?
        .displayed_for_mapping(slot, mapping)
        .map(|p| p.publication_id.raw());
    let ready = world.resource_mut::<Probe>()?.ready.take();
    let live_wait = ready.as_ref().is_some_and(|(wait, _)| {
        world.resource::<Gfx>().is_ok_and(|gfx| {
            gfx.gpu_context_affinity() == wait.submission.affinity()
                && gfx.has_surface(RenderSurfaceId::primary())
        }) && matches!(
            wait.submission.status(),
            runen_gpu::GpuSubmissionStatus::Completed
        ) && wait.mapping == mapping
            && world
                .resource::<UiRuntimeSlotsResource>()
                .is_ok_and(|slots| slots.contains(slot))
            && world
                .resource::<WindowStateRegistryResource>()
                .ok()
                .and_then(|windows| windows.record(NativeWindowId::primary()))
                .is_some_and(|record| {
                    record.lifecycle_state == engine::runtime::NativeWindowLifecycleState::Created
                })
    });
    if let Some(publication) = publication.filter(|p| {
        live_wait
            && Some(*p) != previous
            && ready
                .as_ref()
                .is_some_and(|(wait, _)| wait.publication.raw() == *p)
    }) {
        world.resource_mut::<Probe>()?.last_publication = Some(publication);
        let mut evidence = shared.lock().unwrap();
        println!(
            "displayed publication={publication} action={}",
            evidence.actions
        );
        if evidence.actions < target {
            evidence.actions += 1;
            world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .submit_app_action::<TextApp>(slot, evidence.actions)
                .map_err(|error| anyhow::anyhow!("action rejected: {error:?}"))?;
            println!(
                "application_action={} preceding_publication={publication}",
                evidence.actions
            );
            println!(
                "continuation_latency_ms={:.3}",
                ready.unwrap().1.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
    Ok(())
}

struct Hook(Arc<Mutex<Evidence>>);
impl NativeWindowHook for Hook {
    fn name(&self) -> &'static str {
        "native_gpu_progress"
    }
    fn attach(&mut self, _: &Window, world: &mut World) -> Result<()> {
        let gfx = world.resource_mut::<Gfx>()?;
        gfx.set_gpu_frame_terminal_delivery(true);
        println!(
            "adapter={:?} affinity={:?}",
            gfx.adapter_facts().diagnostic_name(),
            gfx.gpu_context_affinity()
        );
        Ok(())
    }
    fn frame(&mut self, _: &Window, world: &mut World) -> Result<()> {
        application_action(world)
    }
    fn gpu_progress(&mut self, world: &mut World) -> Result<bool> {
        observe_terminal(world)?;
        let shared = world.resource::<Probe>()?.evidence.clone();
        shared.lock().unwrap().terminal_callbacks += 1;
        let (slot, target) = {
            let probe = world.resource::<Probe>()?;
            (probe.slot, probe.target)
        };
        let record = world
            .resource::<WindowStateRegistryResource>()?
            .record(NativeWindowId::primary())
            .cloned();
        let live = record.as_ref().is_some_and(|r| {
            r.lifecycle_state == engine::runtime::NativeWindowLifecycleState::Created
        }) && world.resource::<UiRuntimeSlotsResource>()?.contains(slot)
            && world
                .resource::<Gfx>()?
                .has_surface(RenderSurfaceId::primary());
        if !live {
            world.resource_mut::<Probe>()?.waiting = None;
            return Ok(false);
        }
        let wait = world.resource_mut::<Probe>()?.waiting.take();
        let mut requested = false;
        if let Some(wait) = wait {
            let affinity = world.resource::<Gfx>()?.gpu_context_affinity();
            let current = record.as_ref().unwrap();
            let mapping_matches = wait.mapping.target_size_px() == current.size_px
                && wait.mapping.native_scale_factor() == current.scale_factor;
            if wait.submission.affinity() == affinity && mapping_matches {
                match wait.submission.status() {
                    runen_gpu::GpuSubmissionStatus::Accepted => {
                        world.resource_mut::<Probe>()?.waiting = Some(wait)
                    }
                    runen_gpu::GpuSubmissionStatus::Completed => {
                        let displayed = world
                            .resource::<UiRuntimePresentationAssociationsResource>()?
                            .displayed_for_mapping(slot, wait.mapping)
                            .is_some_and(|p| p.publication_id == wait.publication);
                        let mut evidence = shared.lock().unwrap();
                        if displayed && evidence.actions < target {
                            evidence.requests += 1;
                            println!(
                                "continuation_request={} submission={} publication={}",
                                evidence.requests,
                                wait.submission.id(),
                                wait.publication.raw()
                            );
                            world.resource_mut::<Probe>()?.ready = Some((wait, Instant::now()));
                            requested = true;
                        }
                    }
                    runen_gpu::GpuSubmissionStatus::Failed(_) => {}
                }
            }
        }
        let mut evidence = shared.lock().unwrap();
        let stats = world.resource::<Gfx>()?.execution_stats();
        let record = record.unwrap();
        let mapping = UiRuntimeNativeMapping::new(
            NativeWindowId::primary(),
            RenderSurfaceId::primary(),
            record.size_px,
            record.scale_factor,
        )?;
        let displayed = world
            .resource::<UiRuntimePresentationAssociationsResource>()?
            .displayed_for_mapping(slot, mapping);
        let paint = world
            .resource::<RunenUiPaintSubmissionRegistryResource>()?
            .get_for_surface(
                &RenderFrameProducerId::try_from_raw(1).unwrap(),
                RenderSurfaceId::primary(),
            );
        let final_publication_matches = displayed.zip(paint).is_some_and(|(displayed, paint)| {
            displayed.publication_id == paint.publication_id
                && displayed.paint_revision == paint.publication.revision()
        });
        let applied_action = world
            .resource::<UiRuntimeSlotsResource>()?
            .observe_app::<TextApp, _>(slot, |app| *app.state())
            .map_err(|error| anyhow::anyhow!("typed observation failed: {error:?}"))?;
        let capture_ready = if world.resource::<Probe>()?.capture {
            world.resource::<engine::plugins::render::inspect::RenderCapturedTextureState>()?.captures.iter()
                .any(|capture| Some(capture.identity.frame_index) == evidence.last_accepted_frame
                    && capture.identity.surface_id == RenderSurfaceId::primary().raw()
                    && capture.terminal.code == engine::plugins::render::inspect::RenderCaptureTerminalCode::Completed
                    && capture.bytes_rgba8.as_ref().is_some_and(|bytes| !bytes.is_empty()))
        } else {
            true
        };
        if let Some(displayed) = displayed {
            println!(
                "ui_terminal publication={} paint_revision={:?} applied_action={applied_action}",
                displayed.publication_id.raw(),
                displayed.paint_revision
            );
        }
        if evidence.actions == target
            && applied_action == target
            && final_publication_matches
            && capture_ready
            && evidence.accepted == evidence.completed
            && stats.in_flight_submissions() == 0
            && stats.pending_readbacks() == 0
            && !evidence.complete
        {
            evidence.complete = true;
            if world.resource::<Probe>()?.capture {
                println!(
                    "capture_terminal source_frame={:?} actual_bytes=true",
                    evidence.last_accepted_frame
                );
            }
            println!(
                "qualification_complete frames={} actions={} requests={} terminal_callbacks={} accepted={:?} completed={:?} input_events={}",
                evidence.frames,
                evidence.actions,
                evidence.requests,
                evidence.terminal_callbacks,
                evidence.accepted,
                evidence.completed,
                evidence.input_events
            );
            if world.resource::<Probe>()?.on_demand {
                println!("idle_start=true");
            }
        }
        Ok(requested)
    }
    fn window_event(&mut self, _: &Window, event: &WindowEvent, _: &mut World) -> Result<()> {
        println!("window_event={event:?}");
        if matches!(
            event,
            WindowEvent::KeyboardInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::Touch(_)
        ) {
            self.0.lock().unwrap().input_events += 1;
        }
        Ok(())
    }
    fn device_event(&mut self, event: &DeviceEvent, _: &mut World) -> Result<()> {
        println!("device_event={event:?}");
        if matches!(event, DeviceEvent::MouseMotion { .. }) {
            self.0.lock().unwrap().input_events += 1;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let _guard = engine::utils::setup_tracing();
    let target = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(16);
    let pacing = std::env::args().nth(2).unwrap_or("on-demand".into());
    let capture = std::env::args().nth(3).is_some_and(|arg| arg == "capture");
    ensure!(
        ["on-demand", "continuous-30"].contains(&pacing.as_str()),
        "unknown pacing mode"
    );
    println!(
        "qualification source={} pacing={pacing} target={target} pid={}",
        option_env!("RUNENWERK_SOURCE_REVISION").unwrap_or("UNRECORDED"),
        std::process::id()
    );
    let mut app = App::new();
    app.set_title("Runenwerk #1392 OnDemand GPU progress");
    app.add_plugins(default_plugins());
    app.add_plugins((RenderPlugin, UiPlugin));
    app.with_frame_pacing(if pacing == "on-demand" {
        FramePacingPolicyResource::on_demand()
    } else {
        FramePacingPolicyResource::continuous_capped(30)
    });
    let flow = RenderFlow::new("progress.main")
        .with_surface_color()?
        .fullscreen_pass("clear")
        .main_surface_only()
        .clear_color([0.06, 0.07, 0.09, 1.0])
        .write_surface_color()?
        .finish()
        .builtin_ui_composite_pass("ui")?
        .main_surface_only()
        .finish()
        .present_pass("present")?
        .main_surface_only()
        .surface_color()?
        .finish()
        .validate()?;
    if capture {
        use engine::plugins::render::inspect::{
            RenderCaptureSelector, RenderDebugConfigResource, RenderDebugControlResource,
        };
        let ui_pass = flow
            .pass_id("ui")
            .ok_or_else(|| anyhow::anyhow!("UI composite pass missing"))?;
        let control = app
            .world_mut()
            .resource_mut::<RenderDebugControlResource>()?;
        control.capture_enabled = true;
        control.readback_enabled = true;
        // Capture selectors identify the compiled flow and pass, not their authoring labels.
        app.world_mut()
            .resource_mut::<RenderDebugConfigResource>()?
            .capture_selectors = vec![RenderCaptureSelector::named_pass_surface_color(
            flow.id().to_string(),
            ui_pass.to_string(),
        )];
    }
    app.add_render_flow(flow);
    let fonts = UiRuntimeFontConfiguration::new(vec![
        include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf").to_vec(),
    ])
    .with_generic_mapping(
        GenericFontFamily::SansSerif,
        vec![FontFamilyName::new("JetBrains Mono")?],
    );
    let report = app
        .ui()
        .mount_app::<TextApp>(UiTypedScreenId::new("progress.text"), 0, &fonts);
    let slot = report
        .slot_id()
        .ok_or_else(|| anyhow::anyhow!("mount rejected: {report:?}"))?;
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()?
        .bind(UiRuntimePresentationBinding::new(
            slot,
            RenderFrameProducerId::try_from_raw(1).unwrap(),
            RenderSurfaceId::primary(),
            StyleEnvironment::default(),
        ))?;
    let evidence = Arc::new(Mutex::new(Evidence::default()));
    app.insert_resource(Probe {
        slot,
        target,
        evidence: evidence.clone(),
        last_publication: None,
        waiting: None,
        ready: None,
        on_demand: pacing == "on-demand",
        capture,
    });
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(Hook(evidence.clone()));
    app.add_systems(FrameEnd, observe_frame_end.on_invoker_thread());
    app.run()?;
    let evidence = evidence.lock().unwrap();
    ensure!(
        evidence.complete,
        "final text/Present publication was not qualified"
    );
    ensure!(
        evidence.actions == target,
        "incomplete application continuation"
    );
    ensure!(
        evidence.accepted == evidence.completed,
        "incomplete GPU drain"
    );
    Ok(())
}
