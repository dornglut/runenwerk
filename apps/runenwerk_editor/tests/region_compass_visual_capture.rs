use std::path::PathBuf;
use std::sync::Arc;

use editor_shell::{
    RegionCompassAccessibility, RegionCompassViewModel, projected_host_tab_stacks,
    tab_stack_container_widget_id,
};
use engine::plugins::render::Gfx;
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, RenderCaptureSelector, RenderCaptureTerminalCode,
    RenderCapturedTextureState, RenderPassProvenanceState, deterministic_capture_filename,
};
use runenwerk_editor::runtime::resources::EditorHostResource;
use ui_adaptive_composition::DockZone;
use ui_math::UiPoint;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

const ENABLE_ENV: &str = "RUNENWERK_CAPTURE_REGION_COMPASS";
const SURFACE_RESOURCE_ID: &str = "surface.color";
const OUTPUT_DIR: &str = "docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-007/artifacts";

fn main() {
    if std::env::var_os(ENABLE_ENV).is_none() {
        return;
    }
    capture().expect("Region Compass GPU visual capture should succeed");
}

fn capture() -> anyhow::Result<()> {
    eprintln!("region-compass-capture: create-window");
    let window = create_hidden_window()?;
    eprintln!("region-compass-capture: create-gfx");
    let gfx = Gfx::new(window)?;
    eprintln!("region-compass-capture: build-app");
    let mut app = runenwerk_editor::runtime::build_headless_app();
    activate_region_compass(&mut app)?;
    app.world_mut().insert_resource(gfx);
    app.update_render_debug_control(|control| {
        control.provenance_enabled = true;
        control.capture_enabled = false;
        control.readback_enabled = false;
        control.artifact_export_enabled = false;
    });
    eprintln!("region-compass-capture: warmup");
    let mut app = app.run_for_frames(3)?;
    eprintln!("region-compass-capture: configure-capture");
    let ui_record = app
        .world()
        .resource::<RenderPassProvenanceState>()?
        .records
        .iter()
        .find(|record| record.shader_id == "builtin:ui_composite")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("UI composite provenance is missing"))?;
    app.update_render_debug_config(|config| {
        config.clear();
        config.capture_selectors = vec![RenderCaptureSelector {
            flow_id: Some(ui_record.flow_id.clone()),
            pass_id: Some(ui_record.pass_id.clone()),
            stage: CaptureStage::After,
            resource_id: SURFACE_RESOURCE_ID.to_owned(),
            texture_class: CaptureTextureClass::ImportedTexture,
        }];
    });
    app.update_render_debug_control(|control| {
        control.provenance_enabled = true;
        control.capture_enabled = true;
        control.readback_enabled = true;
        control.artifact_export_enabled = true;
        control.artifact_output_dir = PathBuf::from(OUTPUT_DIR);
    });
    eprintln!("region-compass-capture: capture-frames");
    let app = app.run_for_frames(3)?;
    eprintln!("region-compass-capture: inspect");
    let captures = app.world().resource::<RenderCapturedTextureState>()?;
    let capture = captures
        .find(
            ui_record.flow_id.as_str(),
            ui_record.pass_id.as_str(),
            CaptureStage::After,
            SURFACE_RESOURCE_ID,
        )
        .ok_or_else(|| anyhow::anyhow!("Region Compass UI capture is missing"))?;
    anyhow::ensure!(
        capture.terminal.code == RenderCaptureTerminalCode::Completed,
        "capture did not complete: {:?}",
        capture.terminal
    );
    anyhow::ensure!(capture.bytes_rgba8.is_some(), "capture has no RGBA pixels");
    let path =
        PathBuf::from(OUTPUT_DIR).join(deterministic_capture_filename(&capture.identity, "png"));
    anyhow::ensure!(
        path.is_file(),
        "capture PNG was not exported at {}",
        path.display()
    );
    println!("region_compass_capture={}", path.display());
    Ok(())
}

fn activate_region_compass(app: &mut engine::App) -> anyhow::Result<()> {
    let host = app.world_mut().resource_mut::<EditorHostResource>()?;
    let (panel, source_stack, target_stack, target_tab_count) = {
        let stacks =
            projected_host_tab_stacks(&host.shell_state.composition_projection().shell.root_host)
                .into_iter()
                .filter(|stack| stack.active_panel.is_some())
                .collect::<Vec<_>>();
        let source = stacks
            .first()
            .ok_or_else(|| anyhow::anyhow!("default editor composition has no active stack"))?;
        let target = stacks.get(1).copied().unwrap_or(*source);
        (
            source.active_panel.as_ref().unwrap().panel_instance_id,
            source.tab_stack_id,
            target.tab_stack_id,
            target.tabs.len(),
        )
    };
    let unit = host
        .shell_state
        .mounted_unit_id_for_panel(panel)
        .ok_or_else(|| anyhow::anyhow!("active panel has no mounted composition unit"))?;
    let region = host
        .shell_state
        .region_id_for_tab_stack(target_stack)
        .ok_or_else(|| anyhow::anyhow!("target stack has no composition region"))?;
    let target_id = host.shell_state.primary_composition_target_id();
    let epoch = host.shell_state.current_projection_epoch();
    let anchor = tab_stack_container_widget_id(target_stack);
    host.shell_state.begin_tab_drag_candidate(
        panel,
        source_stack,
        UiPoint::new(320.0, 240.0),
        epoch,
    );
    host.shell_state
        .update_tab_drag_pointer(UiPoint::new(328.0, 240.0), epoch);
    let compass = RegionCompassViewModel::active(
        target_id,
        region,
        unit,
        DockZone::Right,
        "Scene Viewport",
        "Inspector region",
        RegionCompassAccessibility::default(),
    )
    .with_ordinal(target_tab_count);
    host.shell_state
        .set_region_compass_for_target(target_id, anchor, compass, epoch);
    Ok(())
}

fn create_hidden_window() -> anyhow::Result<Arc<Window>> {
    struct Bootstrap {
        window: Option<Arc<Window>>,
        error: Option<String>,
    }

    impl ApplicationHandler for Bootstrap {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let attributes = Window::default_attributes()
                .with_title("Runenwerk Region Compass visual capture")
                .with_visible(false)
                .with_inner_size(PhysicalSize::new(1280, 720));
            match event_loop.create_window(attributes) {
                Ok(window) => self.window = Some(Arc::new(window)),
                Err(error) => self.error = Some(error.to_string()),
            }
            event_loop.exit();
        }

        fn window_event(
            &mut self,
            _: &ActiveEventLoop,
            _: winit::window::WindowId,
            _: winit::event::WindowEvent,
        ) {
        }
    }

    let event_loop = EventLoop::new()?;
    let mut bootstrap = Bootstrap {
        window: None,
        error: None,
    };
    event_loop.run_app(&mut bootstrap)?;
    if let Some(error) = bootstrap.error {
        anyhow::bail!("hidden capture window failed: {error}");
    }
    bootstrap
        .window
        .ok_or_else(|| anyhow::anyhow!("hidden capture window was not created"))
}
