use std::path::PathBuf;
use std::sync::Arc;

use engine::plugins::AppFixedStepExt;
use engine::plugins::render::backend::{RenderSurfaceId, RenderSurfaceRegistryResource};
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderCapturedTextureState,
    RenderPassProvenanceState,
};
use engine::plugins::render::{AppRenderExt, Gfx, RenderPlugin};
use engine::runtime::{
    NativeWindowId, PrimaryPresentationMetricsResource, WindowStateRegistryResource,
};
use runenwerk_arena::{ArenaPresentationPlugin, build_game_app};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::Window;

const ARENA_DISPLAY_SHADER_ID: &str = "assets/shaders/runenwerk_arena_radiance.wgsl";
const ARENA_PRESENT_SHADER_ID: &str = "builtin:present";
const SURFACE_COLOR_RESOURCE_ID: &str = "surface.color";

#[test]
#[ignore = "requires a windowed GPU adapter; retained Linux/Vulkan evidence runs this explicitly under Xvfb"]
fn arena_presentation_gpu_smoke() {
    if !gpu_smoke_enabled() {
        eprintln!("RUNENWERK_ARENA_GPU_SMOKE is not enabled; skipping retained arena GPU smoke");
        return;
    }
    if cfg!(target_os = "macos") && !running_on_macos_main_thread() {
        eprintln!("macOS arena GPU smoke requires the process main thread");
        return;
    }

    let _workspace_directory = WorkspaceDirectoryGuard::enter();

    let window = create_hidden_window();
    let gfx = Gfx::new(Arc::clone(&window)).expect("arena GPU smoke should create Gfx");

    let size = window.inner_size();
    let size_px = (size.width.max(1), size.height.max(1));
    let scale_factor = window.scale_factor();

    // Keep the test harness headless so deterministic fixed-step/frame advancement remains
    // available, while exercising the exact Render + ArenaPresentation composition against a
    // real window-backed primary surface. Separate composition tests prove build_game_app(false)
    // selects these same plugins only for the maintained native product path.
    let mut app = build_game_app(true);
    app.add_plugins((RenderPlugin, ArenaPresentationPlugin));
    app.world_mut().insert_resource(gfx);
    app.world_mut()
        .insert_resource(PrimaryPresentationMetricsResource::new(
            size_px,
            scale_factor,
        ));
    app.world_mut()
        .insert_resource(WindowStateRegistryResource::default());
    app.world_mut()
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window registry")
        .register_primary_window(window.title(), size_px, scale_factor, window.has_focus());
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .expect("render surface registry")
        .confirm_surface_attachment(
            RenderSurfaceId::primary(),
            NativeWindowId::primary(),
            size_px,
        )
        .expect("arena GPU smoke should correlate the primary surface");

    app.update_render_debug_control(|control| {
        control.provenance_enabled = true;
        control.capture_enabled = false;
        control.readback_enabled = false;
        control.artifact_export_enabled = false;
    });

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("arena truth should advance before presentation")
        .run_for_frames(4)
        .expect("arena rendered composition should submit warm-up frames");

    let provenance = app
        .world()
        .resource::<RenderPassProvenanceState>()
        .expect("arena GPU smoke should retain pass provenance");
    let display = provenance
        .records
        .iter()
        .find(|record| {
            record.pass_kind == engine::plugins::render::pipelines::FlowPassKind::Fullscreen
                && record.shader_id == ARENA_DISPLAY_SHADER_ID
        })
        .unwrap_or_else(|| {
            panic!(
                "arena display pass did not execute; records={:?}",
                provenance
                    .records
                    .iter()
                    .map(|record| (
                        &record.flow_id,
                        &record.pass_id,
                        record.pass_kind,
                        &record.shader_id,
                    ))
                    .collect::<Vec<_>>()
            )
        })
        .clone();
    assert!(
        provenance.records.iter().any(|record| {
            record.pass_kind == engine::plugins::render::pipelines::FlowPassKind::Present
                && record.shader_id == ARENA_PRESENT_SHADER_ID
        }),
        "arena presentation must reach its terminal Present pass"
    );

    app.update_render_debug_config(|config| {
        config.clear();
        config.capture_selectors = vec![RenderCaptureSelector {
            flow_id: Some(display.flow_id.clone()),
            pass_id: Some(display.pass_id.clone()),
            stage: CaptureStage::After,
            resource_id: SURFACE_COLOR_RESOURCE_ID.to_string(),
            texture_class: CaptureTextureClass::ImportedTexture,
        }];
    });
    app.update_render_debug_control(|control| {
        control.provenance_enabled = true;
        control.capture_enabled = true;
        control.readback_enabled = true;
        control.artifact_export_enabled = false;
    });

    let app = app
        .run_for_frames(4)
        .expect("arena GPU smoke should capture the ordinary presented frame");

    let captures = app
        .world()
        .resource::<RenderCapturedTextureState>()
        .expect("arena GPU smoke should retain capture state");
    let capture = captures
        .find(
            display.flow_id.as_str(),
            display.pass_id.as_str(),
            CaptureStage::After,
            SURFACE_COLOR_RESOURCE_ID,
        )
        .unwrap_or_else(|| {
            panic!(
                "arena display capture is missing; captures={:?}",
                captures
                    .captures
                    .iter()
                    .map(|capture| (
                        capture.identity.flow_id(),
                        capture.identity.pass_id(),
                        capture.identity.stage(),
                        capture.identity.resource_id(),
                        capture.terminal.code,
                    ))
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        capture.is_completed(),
        "arena display capture must complete: {:?}",
        capture.terminal
    );
    let pixels = capture
        .bytes_rgba8
        .as_ref()
        .expect("arena display capture must contain RGBA readback");
    assert_eq!(capture.width, size_px.0);
    assert_eq!(capture.height, size_px.1);
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0),
        "maintained arena + player presentation must produce a non-black primary-surface frame"
    );
}

struct WorkspaceDirectoryGuard {
    previous: PathBuf,
}

impl WorkspaceDirectoryGuard {
    fn enter() -> Self {
        let previous =
            std::env::current_dir().expect("arena GPU smoke should resolve its current directory");
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|apps| apps.parent())
            .expect("arena package should live under <workspace>/apps/runenwerk_arena");
        assert!(
            workspace_root.join("assets/shaders").is_dir(),
            "arena GPU smoke workspace root must expose the maintained shader asset directory"
        );
        std::env::set_current_dir(workspace_root)
            .expect("arena GPU smoke should enter the workspace asset root");
        Self { previous }
    }
}

impl Drop for WorkspaceDirectoryGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.previous)
            .expect("arena GPU smoke should restore its original working directory");
    }
}

fn gpu_smoke_enabled() -> bool {
    std::env::var("RUNENWERK_ARENA_GPU_SMOKE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn running_on_macos_main_thread() -> bool {
    unsafe extern "C" {
        fn pthread_main_np() -> std::os::raw::c_int;
    }

    unsafe { pthread_main_np() != 0 }
}

#[cfg(not(target_os = "macos"))]
fn running_on_macos_main_thread() -> bool {
    true
}

fn create_hidden_window() -> Arc<Window> {
    struct HiddenWindowBootstrap {
        attrs: winit::window::WindowAttributes,
        window: Option<Arc<Window>>,
        error: Option<String>,
    }

    impl ApplicationHandler for HiddenWindowBootstrap {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            match event_loop.create_window(self.attrs.clone()) {
                Ok(window) => self.window = Some(Arc::new(window)),
                Err(error) => self.error = Some(error.to_string()),
            }
            event_loop.exit();
        }

        fn window_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _window_id: winit::window::WindowId,
            _event: winit::event::WindowEvent,
        ) {
        }
    }

    let event_loop = create_smoke_event_loop();
    let attrs = Window::default_attributes()
        .with_title("runenwerk arena presentation gpu smoke")
        .with_visible(false)
        .with_inner_size(PhysicalSize::new(1280, 720));
    let mut bootstrap = HiddenWindowBootstrap {
        attrs,
        window: None,
        error: None,
    };
    event_loop
        .run_app(&mut bootstrap)
        .expect("arena hidden-window bootstrap should run");
    if let Some(error) = bootstrap.error {
        panic!("arena hidden-window creation failed: {error}");
    }
    bootstrap
        .window
        .expect("arena hidden-window bootstrap should capture the window")
}

fn create_smoke_event_loop() -> EventLoop<()> {
    #[cfg(target_os = "linux")]
    {
        let mut builder = EventLoop::builder();
        builder.with_x11().with_any_thread(true);
        builder
            .build()
            .expect("arena X11 event loop should initialize")
    }

    #[cfg(target_os = "windows")]
    {
        let mut builder = EventLoop::builder();
        builder.with_any_thread(true);
        builder.build().expect("arena event loop should initialize")
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        EventLoop::new().expect("arena event loop should initialize")
    }
}
