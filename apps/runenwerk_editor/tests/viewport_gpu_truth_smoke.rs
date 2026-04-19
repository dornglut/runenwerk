use engine::plugins::render::Gfx;
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderCaptureTerminalCode,
    RenderCapturedTextureState, RenderPassProvenanceState, RenderPixelCoordinate,
};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;

const FLOW_ID: &str = "runenwerk.editor.main";
const VIEWPORT_PASS_ID: &str = "runenwerk.editor.viewport.product.scene";
const UI_PASS_ID: &str = "runenwerk.editor.main.ui";
const VIEWPORT_RESOURCE_ID: &str = "editor.viewport.v1.scene_color";
const SURFACE_RESOURCE_ID: &str = "surface.color";

#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "requires macOS main-thread-safe windowed GPU harness; set RUNENWERK_ENABLE_GPU_SMOKE=1 and RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 for manual runs"
)]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "requires a real windowed GPU device"
)]
fn viewport_gpu_truth_smoke() {
    if !gpu_smoke_enabled() {
        eprintln!("RUNENWERK_ENABLE_GPU_SMOKE is not enabled; skipping windowed GPU smoke test");
        return;
    }
    if cfg!(target_os = "macos") && !macos_main_thread_smoke_enabled() {
        eprintln!(
            "RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE is not enabled; skipping macOS main-thread GPU smoke path"
        );
        return;
    }

    let event_loop = EventLoop::new().expect("event loop should initialize");
    let attrs = Window::default_attributes()
        .with_title("runenwerk viewport gpu smoke")
        .with_visible(false)
        .with_inner_size(PhysicalSize::new(1280, 720));
    let window = Arc::new(
        event_loop
            .create_window(attrs)
            .expect("hidden smoke-test window should be created"),
    );
    let gfx = Gfx::new(window).expect("gfx should initialize for smoke test");

    let mut app = runenwerk_editor::runtime::build_headless_app();
    app.world_mut().insert_resource(gfx);

    app.update_render_debug_control(|debug_control| {
        debug_control.provenance_enabled = true;
        debug_control.capture_enabled = true;
        debug_control.readback_enabled = true;
        debug_control.artifact_export_enabled = false;
    });

    app.update_render_debug_config(|debug_config| {
        debug_config.clear();
        debug_config.capture_selectors = vec![
            RenderCaptureSelector {
                flow_id: Some(FLOW_ID.to_string()),
                pass_id: Some(VIEWPORT_PASS_ID.to_string()),
                stage: CaptureStage::After,
                resource_id: VIEWPORT_RESOURCE_ID.to_string(),
                texture_class: CaptureTextureClass::ColorTarget,
            },
            RenderCaptureSelector {
                flow_id: Some(FLOW_ID.to_string()),
                pass_id: Some(UI_PASS_ID.to_string()),
                stage: CaptureStage::Before,
                resource_id: SURFACE_RESOURCE_ID.to_string(),
                texture_class: CaptureTextureClass::ImportedTexture,
            },
            RenderCaptureSelector {
                flow_id: Some(FLOW_ID.to_string()),
                pass_id: Some(UI_PASS_ID.to_string()),
                stage: CaptureStage::After,
                resource_id: SURFACE_RESOURCE_ID.to_string(),
                texture_class: CaptureTextureClass::ImportedTexture,
            },
        ];
    });

    let app = app
        .run_for_frames(3)
        .expect("windowed GPU smoke test app should render frames");

    let captures = app
        .world()
        .resource::<RenderCapturedTextureState>()
        .expect("captured texture state should exist");

    let viewport_after = captures
        .find(
            FLOW_ID,
            VIEWPORT_PASS_ID,
            CaptureStage::After,
            VIEWPORT_RESOURCE_ID,
        )
        .expect("viewport pass after-capture should exist");
    let ui_before = captures
        .find(FLOW_ID, UI_PASS_ID, CaptureStage::Before, SURFACE_RESOURCE_ID)
        .expect("ui pass before-capture should exist");
    let ui_after = captures
        .find(FLOW_ID, UI_PASS_ID, CaptureStage::After, SURFACE_RESOURCE_ID)
        .expect("ui pass after-capture should exist");

    assert!(
        viewport_after.terminal.code == RenderCaptureTerminalCode::Completed
            && ui_before.terminal.code == RenderCaptureTerminalCode::Completed
            && ui_after.terminal.code == RenderCaptureTerminalCode::Completed,
        "all required captures must complete without readback errors"
    );
    assert!(
        viewport_after.bytes_rgba8.is_some()
            && ui_before.bytes_rgba8.is_some()
            && ui_after.bytes_rgba8.is_some(),
        "all required captures must include rgba pixels"
    );

    let center_viewport = viewport_after
        .sample_center_rgba8()
        .expect("viewport after capture should provide center pixel");
    let center_ui_before = ui_before
        .sample_center_rgba8()
        .expect("ui before capture should provide center pixel");
    let center_ui_after = ui_after
        .sample_center_rgba8()
        .expect("ui after capture should provide center pixel");

    let inside_point = RenderPixelCoordinate {
        x: (viewport_after.width / 2).max(1),
        y: (viewport_after.height / 2).max(1),
    };
    let outside_point = RenderPixelCoordinate {
        x: 16_u32.min(viewport_after.width.saturating_sub(1)),
        y: 16_u32.min(viewport_after.height.saturating_sub(1)),
    };
    let inside_before = ui_before
        .sample_pixel_rgba8(inside_point)
        .expect("inside point should be sampleable before ui pass");
    let inside_after = ui_after
        .sample_pixel_rgba8(inside_point)
        .expect("inside point should be sampleable after ui pass");
    let outside_before = ui_before
        .sample_pixel_rgba8(outside_point)
        .expect("outside point should be sampleable before ui pass");
    let outside_after = ui_after
        .sample_pixel_rgba8(outside_point)
        .expect("outside point should be sampleable after ui pass");

    assert!(
        center_viewport != center_ui_before
            || center_ui_before != center_ui_after
            || inside_before != inside_after
            || outside_before != outside_after,
        "expected viewport product and ui composite passes to affect sampled points"
    );

    let provenance = app
        .world()
        .resource::<RenderPassProvenanceState>()
        .expect("pass provenance state should exist");
    let viewport_record = provenance
        .records
        .iter()
        .find(|record| record.flow_id == FLOW_ID && record.pass_id == VIEWPORT_PASS_ID)
        .expect("viewport pass provenance should exist");
    let ui_record = provenance
        .records
        .iter()
        .find(|record| record.flow_id == FLOW_ID && record.pass_id == UI_PASS_ID)
        .expect("ui pass provenance should exist");

    assert_eq!(viewport_record.pass_label, VIEWPORT_PASS_ID);
    assert_eq!(ui_record.pass_label, UI_PASS_ID);
    assert!(!viewport_record.shader_id.is_empty());
    assert!(!ui_record.shader_id.is_empty());
}

fn gpu_smoke_enabled() -> bool {
    std::env::var("RUNENWERK_ENABLE_GPU_SMOKE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn macos_main_thread_smoke_enabled() -> bool {
    std::env::var("RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
