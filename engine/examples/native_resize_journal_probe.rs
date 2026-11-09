//! Opt-in Windows resize/journal discriminator for Engine #1397.
//! No engine-owned event semantics are altered. Run as a Cargo example.

use anyhow::{Context, Result, bail, ensure};
use engine::plugins::render::RenderFlow;
use engine::plugins::{RenderPlugin, default_plugins};
use engine::prelude::{App, AppRenderExt};
use engine::runtime::platform::PlatformWindowEventQueueResource;
use engine::runtime::{
    FrameEnd, FramePacingPolicyResource, FramePacingRuntimeStateResource, NativeWindowHook,
    NativeWindowHookRegistryResource, NativeWindowId, RenderSubmit, WindowStateRegistryResource,
};
use runen_ecs::{Res, ResMut, Resource, World};
use std::collections::{BTreeMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::event::{DeviceEvent, WindowEvent};
use winit::window::{Window, WindowId};

const SAMPLE_LIMIT: usize = 128;

struct Arguments {
    render: bool,
    pacing: &'static str,
    slow_ms: u64,
    log_path: PathBuf,
}

fn parse_arguments() -> Result<Arguments> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 4,
        "usage: native_resize_journal_probe <no-render|render> <on-demand|capped30|capped60|uncapped> <slow-ms> <log-file>"
    );
    let render = match args[0].as_str() {
        "render" => true,
        "no-render" => false,
        other => bail!("invalid render choice: {other}"),
    };
    let pacing = match args[1].as_str() {
        "on-demand" => "on-demand",
        "capped30" => "capped30",
        "capped60" => "capped60",
        "uncapped" => "uncapped",
        other => bail!("invalid pacing choice: {other}"),
    };
    let slow_ms: u64 = args[2].parse().context("slow-ms must be an integer")?;
    ensure!(slow_ms <= 250, "slow-ms must be between 0 and 250");
    Ok(Arguments {
        render,
        pacing,
        slow_ms,
        log_path: PathBuf::from(&args[3]),
    })
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    ms: u128,
    callback: u64,
    source: &'static str,
    kind: &'static str,
    journal_before: usize,
    is_primary: bool,
    redraw_interest: bool,
}

struct Counters {
    started: Instant,
    callback_count: u64,
    counts: BTreeMap<&'static str, u64>,
    primary_callbacks: u64,
    secondary_callbacks: u64,
    device_callbacks: u64,
    frame_entries: u64,
    frame_end_visits: u64,
    observed_successful_retirements: u64,
    observed_retirement_failures: u64,
    last_checked_frame_end: u64,
    high_water: usize,
    first_full_ms: Option<u128>,
    last_callback_ms: Option<u128>,
    last_frame_start_ms: Option<u128>,
    last_frame_end_ms: Option<u128>,
    last_journal_before_frame_end: usize,
    samples: VecDeque<Sample>,
}

impl Counters {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            callback_count: 0,
            counts: BTreeMap::new(),
            primary_callbacks: 0,
            secondary_callbacks: 0,
            device_callbacks: 0,
            frame_entries: 0,
            frame_end_visits: 0,
            observed_successful_retirements: 0,
            observed_retirement_failures: 0,
            last_checked_frame_end: 0,
            high_water: 0,
            first_full_ms: None,
            last_callback_ms: None,
            last_frame_start_ms: None,
            last_frame_end_ms: None,
            last_journal_before_frame_end: 0,
            samples: VecDeque::with_capacity(SAMPLE_LIMIT),
        }
    }

    fn sample(
        &mut self,
        source: &'static str,
        kind: &'static str,
        journal: usize,
        is_primary: bool,
        redraw_interest: bool,
    ) {
        let ms = self.started.elapsed().as_millis();
        self.high_water = self.high_water.max(journal);
        if journal >= 4096 && self.first_full_ms.is_none() {
            self.first_full_ms = Some(ms);
        }
        if self.samples.len() == SAMPLE_LIMIT {
            self.samples.pop_front();
        }
        self.samples.push_back(Sample {
            ms,
            callback: self.callback_count,
            source,
            kind,
            journal_before: journal,
            is_primary,
            redraw_interest,
        });
    }

    fn window_callback(
        &mut self,
        kind: &'static str,
        journal: usize,
        primary: bool,
        redraw_interest: bool,
    ) {
        self.callback_count += 1;
        *self.counts.entry(kind).or_default() += 1;
        if primary {
            self.primary_callbacks += 1;
        } else {
            self.secondary_callbacks += 1;
        }
        if self.frame_end_visits > self.last_checked_frame_end {
            if journal == 0 {
                self.observed_successful_retirements += 1;
            } else {
                self.observed_retirement_failures += 1;
            }
            self.last_checked_frame_end = self.frame_end_visits;
        }
        self.last_callback_ms = Some(self.started.elapsed().as_millis());
        self.sample("window", kind, journal, primary, redraw_interest);
    }
}

#[derive(Clone)]
struct Shared(Arc<Mutex<Counters>>);
impl Resource for Shared {}

struct SlowFrameDelay(u64);
impl Resource for SlowFrameDelay {}

struct ProbeHook {
    counters: Arc<Mutex<Counters>>,
    primary_window: Option<WindowId>,
}

impl NativeWindowHook for ProbeHook {
    fn name(&self) -> &'static str {
        "resize_journal_probe"
    }

    fn attach(&mut self, window: &Window, _world: &mut World) -> Result<()> {
        if self.primary_window.is_none() {
            self.primary_window = Some(window.id());
        }
        Ok(())
    }

    fn window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
        world: &mut World,
    ) -> Result<()> {
        let kind = match event {
            WindowEvent::Resized(_) => "Resized",
            WindowEvent::ScaleFactorChanged { .. } => "ScaleFactorChanged",
            WindowEvent::RedrawRequested => "RedrawRequested",
            WindowEvent::CursorMoved { .. } => "CursorMoved",
            WindowEvent::MouseInput { .. } => "MouseInput",
            WindowEvent::MouseWheel { .. } => "MouseWheel",
            WindowEvent::KeyboardInput { .. } => "KeyboardInput",
            WindowEvent::Focused(_) => "Focused",
            WindowEvent::CloseRequested => "CloseRequested",
            WindowEvent::Touch(_) => "Touch",
            _ => "OtherWindowEvent",
        };
        let journal = journal_len(world);
        let interest = world
            .resource::<FramePacingRuntimeStateResource>()
            .ok()
            .is_some_and(|state| state.redraw_requested);
        let mut data = self.counters.lock().expect("probe counter mutex poisoned");
        data.window_callback(
            kind,
            journal,
            self.primary_window == Some(window.id()),
            interest,
        );
        Ok(())
    }

    fn device_event(&mut self, event: &DeviceEvent, world: &mut World) -> Result<()> {
        let kind = match event {
            DeviceEvent::MouseMotion { .. } => "DeviceMouseMotion",
            DeviceEvent::Removed => "DeviceRemoved",
            _ => "OtherDeviceEvent",
        };
        let mut data = self.counters.lock().expect("probe counter mutex poisoned");
        data.device_callbacks += 1;
        *data.counts.entry(kind).or_default() += 1;
        data.sample("device", kind, journal_len(world), false, false);
        Ok(())
    }

    fn frame(&mut self, _window: &Window, world: &mut World) -> Result<()> {
        let mut data = self.counters.lock().expect("probe counter mutex poisoned");
        data.frame_entries += 1;
        data.last_frame_start_ms = Some(data.started.elapsed().as_millis());
        data.sample("frame", "start", journal_len(world), true, false);
        Ok(())
    }
}

fn journal_len(world: &World) -> usize {
    world
        .resource::<PlatformWindowEventQueueResource>()
        .map(|journal| journal.events().len())
        .unwrap_or(0)
}

fn frame_end_observer(shared: Res<Shared>, journal: Res<PlatformWindowEventQueueResource>) {
    let mut data = shared.0.lock().expect("probe counter mutex poisoned");
    data.frame_end_visits += 1;
    data.last_frame_end_ms = Some(data.started.elapsed().as_millis());
    data.last_journal_before_frame_end = journal.events().len();
    data.sample(
        "schedule",
        "FrameEnd-visited",
        journal.events().len(),
        true,
        false,
    );
}

fn approve_close(mut windows: ResMut<WindowStateRegistryResource>) {
    if let Some(record) = windows.record_mut(NativeWindowId::primary())
        && record.close_intent_pending
    {
        record.approve_close();
    }
}

fn synthetic_render_schedule_delay(delay: Res<SlowFrameDelay>) {
    if delay.0 != 0 {
        std::thread::sleep(Duration::from_millis(delay.0));
    }
}

fn write_report(file: &mut File, counters: &Counters) -> Result<()> {
    writeln!(
        file,
        "elapsed_ms={}",
        counters.started.elapsed().as_millis()
    )?;
    writeln!(file, "callback_total={}", counters.callback_count)?;
    writeln!(file, "primary_callbacks={}", counters.primary_callbacks)?;
    writeln!(file, "secondary_callbacks={}", counters.secondary_callbacks)?;
    writeln!(file, "device_callbacks={}", counters.device_callbacks)?;
    writeln!(file, "frame_entries={}", counters.frame_entries)?;
    writeln!(
        file,
        "frame_end_visits_not_successes={}",
        counters.frame_end_visits
    )?;
    writeln!(
        file,
        "post_frame_end_empty_journal_checks={}",
        counters.observed_successful_retirements
    )?;
    writeln!(
        file,
        "post_frame_end_nonempty_journal_checks={}",
        counters.observed_retirement_failures
    )?;
    writeln!(file, "journal_high_water_observed={}", counters.high_water)?;
    writeln!(file, "journal_first_full_ms={:?}", counters.first_full_ms)?;
    writeln!(file, "last_callback_ms={:?}", counters.last_callback_ms)?;
    writeln!(
        file,
        "last_frame_start_ms={:?}",
        counters.last_frame_start_ms
    )?;
    writeln!(file, "last_frame_end_ms={:?}", counters.last_frame_end_ms)?;
    writeln!(
        file,
        "last_frame_end_journal_len={}",
        counters.last_journal_before_frame_end
    )?;
    writeln!(file, "about_to_wait_entries=not_exposed_by_public_hook")?;
    writeln!(
        file,
        "private_frame_requested_bit=not_exposed_by_public_hook"
    )?;
    writeln!(file, "event_types:")?;
    for (kind, count) in &counters.counts {
        writeln!(file, "  {kind}={count}")?;
    }
    writeln!(file, "last_{}_samples:", SAMPLE_LIMIT)?;
    for sample in &counters.samples {
        writeln!(
            file,
            "  ms={} callback={} source={} kind={} journal_before={} primary={} pacing_redraw_interest={}",
            sample.ms,
            sample.callback,
            sample.source,
            sample.kind,
            sample.journal_before,
            sample.is_primary,
            sample.redraw_interest,
        )?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = parse_arguments()?;
    let mut file = File::create(&args.log_path)
        .with_context(|| format!("cannot create log {}", args.log_path.display()))?;
    writeln!(file, "engine_package_version={}", env!("CARGO_PKG_VERSION"))?;
    writeln!(file, "render={}", args.render)?;
    writeln!(file, "pacing={}", args.pacing)?;
    writeln!(file, "synthetic_render_schedule_delay_ms={}", args.slow_ms)?;
    writeln!(
        file,
        "journal_callback_measurement=before_native_host_admission"
    )?;
    file.flush()?;

    let counters = Arc::new(Mutex::new(Counters::new()));
    let mut app = App::new();
    app.set_title("Runenwerk #1397 resize/journal probe");
    app.add_plugins(default_plugins());
    app.insert_resource(match args.pacing {
        "on-demand" => FramePacingPolicyResource::on_demand(),
        "capped30" => FramePacingPolicyResource::continuous_capped(30),
        "capped60" => FramePacingPolicyResource::continuous_capped(60),
        "uncapped" => FramePacingPolicyResource::continuous_uncapped(),
        _ => unreachable!("validated pacing"),
    });
    if args.render {
        app.add_plugin(RenderPlugin);
        let flow = RenderFlow::new("native.resize.probe")
            .with_surface_color()?
            .fullscreen_pass("native.resize.probe.clear")
            .main_surface_only()
            .clear_color([0.11, 0.12, 0.13, 1.0])
            .write_surface_color()?
            .finish()
            .present_pass("native.resize.probe.present")?
            .main_surface_only()
            .surface_color()?
            .finish()
            .validate()?;
        app.add_render_flow(flow);
    }
    app.insert_resource(Shared(Arc::clone(&counters)));
    app.insert_resource(SlowFrameDelay(args.slow_ms));
    app.add_systems(FrameEnd, frame_end_observer);
    app.add_systems(RenderSubmit, synthetic_render_schedule_delay);
    app.add_systems(engine::runtime::Update, approve_close);
    app.init_resource::<NativeWindowHookRegistryResource>();
    app.world_mut()
        .resource_mut::<NativeWindowHookRegistryResource>()?
        .register_hook(ProbeHook {
            counters: Arc::clone(&counters),
            primary_window: None,
        });

    let outcome = app.run();
    let mut file = OpenOptions::new().append(true).open(&args.log_path)?;
    writeln!(
        file,
        "runtime_outcome={}",
        if outcome.is_ok() { "ok" } else { "error" }
    )?;
    if let Err(err) = &outcome {
        writeln!(file, "runtime_error={err:#}")?;
    }
    write_report(
        &mut file,
        &counters.lock().expect("probe counter mutex poisoned"),
    )?;
    file.flush()?;
    outcome
}
