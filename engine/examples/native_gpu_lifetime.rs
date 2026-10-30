//! #1400 physical qualification using the ordinary native App/Render/UI path.
//! Registry diagnostics are opt-in observations; no extra GPU polling or redraw.
use anyhow::{Result, ensure};
use engine::plugins::render::host::RenderSurfaceId;
use engine::plugins::render::inspect::{
    RenderFrameHistoryState, RenderFrameObservationPolicyResource,
};
use engine::plugins::render::renderer::RenderGpuFrameTerminalDisposition;
use engine::plugins::render::{Gfx, RenderFlow, RenderFlowRegistryResource, RenderFrameProducerId};
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
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
    time::Instant,
};
use winit::window::Window;

struct TextApp;
impl UiApp for TextApp {
    type State = u64;
    type Action = u64;
    type HostProtocol = NoHostProtocol;
    fn root(value: &u64) -> impl View<u64> {
        text(format!("Native GPU lifetime — generation {value}"))
            .foreground(runenui_core::Color::rgb(238, 240, 246))
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
    accepted: BTreeSet<u64>,
    completed: BTreeSet<u64>,
    held: BTreeMap<u64, runen_gpu::GpuSubmission>,
    closed: bool,
    error: Option<String>,
}
struct Qualifier {
    shared: Arc<Mutex<Evidence>>,
    start: Instant,
    last: Instant,
    frames: u64,
    mode: String,
    target: usize,
    slot: Option<UiRuntimeSlotId>,
    draining: bool,
}
impl NativeWindowHook for Qualifier {
    fn name(&self) -> &'static str {
        "native_gpu_lifetime_1400"
    }
    fn attach(&mut self, _: &Window, world: &mut World) -> Result<()> {
        if let Ok(gfx) = world.resource_mut::<Gfx>() {
            gfx.set_gpu_frame_terminal_delivery(true);
            println!("adapter={:?}", gfx.adapter_facts());
        }
        println!(
            "qualification_start mode={} target={} pid={}",
            self.mode,
            self.target,
            std::process::id()
        );
        Ok(())
    }
    fn frame(&mut self, window: &Window, world: &mut World) -> Result<()> {
        if let Err(error) = self.observe(window, world) {
            self.shared.lock().unwrap().error = Some(format!("{error:#}"));
            if let Some(primary) = world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
            {
                primary.request_close();
            }
        }
        Ok(())
    }
    fn detach(&mut self, world: &mut World) -> Result<()> {
        let mut evidence = self.shared.lock().unwrap();
        println!(
            "detach slots={} accepted={} completed={} held={}",
            world
                .resource::<UiRuntimeSlotsResource>()
                .map_or(0, |s| s.len()),
            evidence.accepted.len(),
            evidence.completed.len(),
            evidence.held.len()
        );
        for (id, submission) in &evidence.held {
            println!("detach_submission id={id} status={:?}", submission.status());
        }
        evidence.closed = true;
        Ok(())
    }
}
impl Qualifier {
    fn observe(&mut self, window: &Window, world: &mut World) -> Result<()> {
        self.frames += 1;
        let interval = self.last.elapsed().as_secs_f64() * 1000.0;
        self.last = Instant::now();
        let (pending, terminal) = {
            let gfx = world.resource_mut::<Gfx>()?;
            gfx.set_gpu_frame_terminal_delivery(true);
            (
                gfx.pending_gpu_frame_submissions(),
                gfx.take_gpu_frame_terminal_outcomes(),
            )
        };
        ensure!(!terminal.has_delivery_gap(), "terminal delivery gap");
        let mut evidence = self.shared.lock().unwrap();
        for item in pending {
            let id = item.submission().id().get();
            evidence.accepted.insert(id);
            evidence.held.insert(id, item.submission().clone());
        }
        for item in terminal.outcomes() {
            let id = item.submission_id().get();
            evidence.accepted.insert(id);
            ensure!(
                matches!(
                    item.disposition(),
                    RenderGpuFrameTerminalDisposition::Completed
                ),
                "terminal failure: {:?}",
                item.disposition()
            );
            ensure!(evidence.completed.insert(id), "duplicate terminal {id}");
            evidence.held.remove(&id);
            println!(
                "terminal id={id} frame={} surface={} status=Completed age_ms={:.3}",
                item.frame(),
                item.surface(),
                item.host_observed_terminal_age().as_secs_f64() * 1000.0
            );
        }
        let display = self
            .slot
            .and_then(|slot| {
                let size = window.inner_size();
                let mapping = UiRuntimeNativeMapping::new(
                    NativeWindowId::primary(),
                    RenderSurfaceId::primary(),
                    (size.width, size.height),
                    window.scale_factor(),
                )
                .ok()?;
                world
                    .resource::<UiRuntimePresentationAssociationsResource>()
                    .ok()?
                    .displayed_for_mapping(slot, mapping)
                    .map(|_| true)
            })
            .unwrap_or(false);
        let delta = world
            .resource::<engine::plugins::time::domain::Time>()
            .map_or(0.0, |t| t.delta_seconds);
        let fixed = world
            .resource::<FixedTimeState>()
            .copied()
            .unwrap_or_default();
        let cpu = world
            .resource::<RenderFrameHistoryState>()
            .ok()
            .and_then(|h| {
                h.observations().last().map(|f| {
                    (
                        f.cpu.renderer.flow_encode_ms,
                        f.cpu.renderer.encode_submit_ms,
                    )
                })
            });
        if self.frames.is_multiple_of(32) || self.frames < 5 {
            println!(
                "sample frame={} elapsed={:.3} interval_ms={interval:.3} delta={delta:.6} fixed_steps={} saturated={} accepted={} completed={} pending={} displayed={} cpu={cpu:?}",
                self.frames,
                self.start.elapsed().as_secs_f64(),
                fixed.steps_ran_last_frame,
                fixed.saturated_frames,
                evidence.accepted.len(),
                evidence.completed.len(),
                evidence.held.len(),
                display
            );
        }
        ensure!(
            self.start.elapsed().as_secs() < 1800,
            "qualification timeout"
        );
        if !self.draining && evidence.completed.len() >= self.target {
            self.draining = true;
            println!(
                "drain_start accepted={} completed={} pending={}",
                evidence.accepted.len(),
                evidence.completed.len(),
                evidence.held.len()
            );
            world.resource_mut::<RenderFlowRegistryResource>()?.clear();
            if let Some(slot) = self.slot.take() {
                world
                    .resource_mut::<UiRuntimePresentationAssociationsResource>()?
                    .unbind(slot);
                let retirement = world
                    .resource_mut::<UiRuntimeSlotsResource>()?
                    .unmount_with_retirement(slot)
                    .map_err(|e| anyhow::anyhow!("retirement: {e:?}"))?
                    .expect("mounted slot retires");
                println!("ui_retirement={:?}", retirement.final_ownership().status());
            }
        }
        if self.draining && evidence.held.is_empty() {
            ensure!(
                evidence.accepted == evidence.completed,
                "acceptance/terminal mismatch"
            );
            println!(
                "qualification_complete mode={} accepted={} completed={} pending=0 elapsed={:.3}",
                self.mode,
                evidence.accepted.len(),
                evidence.completed.len(),
                self.start.elapsed().as_secs_f64()
            );
            if let Some(primary) = world
                .resource_mut::<WindowStateRegistryResource>()?
                .record_mut(NativeWindowId::primary())
            {
                primary.request_close();
            }
        } else if self.mode == "changing"
            && !self.draining
            && let Some(slot) = self.slot
        {
            world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .submit_app_action::<TextApp>(slot, self.frames)
                .map_err(|e| anyhow::anyhow!("action: {e:?}"))?;
        }
        Ok(())
    }
}
fn main() -> Result<()> {
    let _guard = engine::utils::setup_tracing();
    let mode = std::env::args().nth(1).unwrap_or("static".into());
    ensure!(
        ["static", "changing", "control"].contains(&mode.as_str()),
        "unknown mode"
    );
    let target = std::env::args()
        .nth(2)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(4096);
    let mut app = App::new();
    app.set_title("Runenwerk #1400 Native GPU qualification");
    app.add_plugins(default_plugins());
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    app.insert_resource(RenderFrameObservationPolicyResource::enabled(128));
    let flow = RenderFlow::new("lifetime.main").with_surface_color()?;
    let flow = flow
        .fullscreen_pass("clear")
        .main_surface_only()
        .clear_color([0.06, 0.07, 0.09, 1.0])
        .write_surface_color()?
        .finish();
    let flow = if mode != "control" {
        flow.builtin_ui_composite_pass("ui")?
            .main_surface_only()
            .finish()
    } else {
        flow
    };
    app.add_render_flow(
        flow.present_pass("present")?
            .main_surface_only()
            .surface_color()?
            .finish()
            .validate()?,
    );
    let slot = if mode != "control" {
        // Explicit licensed repository bytes are a controlled input, never a production default.
        let fonts = UiRuntimeFontConfiguration::new(vec![
            include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf").to_vec(),
        ])
        .with_generic_mapping(
            GenericFontFamily::SansSerif,
            vec![FontFamilyName::new("JetBrains Mono")?],
        );
        let report =
            app.ui()
                .mount_app::<TextApp>(UiTypedScreenId::new("lifetime.text"), 0, &fonts);
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
        Some(slot)
    } else {
        None
    };
    let shared = Arc::new(Mutex::new(Evidence::default()));
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(Qualifier {
            shared: shared.clone(),
            start: Instant::now(),
            last: Instant::now(),
            frames: 0,
            mode,
            target,
            slot,
            draining: false,
        });
    let result = app.run();
    let evidence = shared.lock().unwrap();
    println!(
        "host_return result={result:?} detach={} accepted={} completed={} pending={} error={:?}",
        evidence.closed,
        evidence.accepted.len(),
        evidence.completed.len(),
        evidence.held.len(),
        evidence.error
    );
    result?;
    ensure!(
        evidence.error.is_none(),
        "qualification failed: {:?}",
        evidence.error
    );
    ensure!(
        evidence.accepted.len() >= target
            && evidence.accepted == evidence.completed
            && evidence.held.is_empty(),
        "incomplete GPU qualification"
    );
    println!("qualification_exit=success");
    Ok(())
}
