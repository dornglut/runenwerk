use engine::plugins::render::backend::{RenderSurfaceId, RenderSurfaceRegistryResource};
use engine::plugins::render::inspect::inspect_prepared_render_frame;
use engine::plugins::render::{
    PreparedFrameContext, PreparedFrameContributions, PreparedRenderFrame, PreparedShaderSnapshot,
    PreparedSurfaceInfo, PreparedViewFrame,
};
use engine::runtime::NativeWindowId;
use std::collections::BTreeMap;
use ui_render_data::ViewportSurfaceBindingRegistry;

fn native_window(raw: u64) -> NativeWindowId {
    NativeWindowId::try_from_raw(raw).expect("test native window id should be non-zero")
}

fn frame(surface: PreparedSurfaceInfo) -> PreparedRenderFrame {
    PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index: 7,
            flow_registry_revision: 1,
            shader_registry_revision: 1,
            prepare_epoch: 2,
        },
        surface,
        views: vec![PreparedViewFrame::main(surface.target_size_px())],
        flows: BTreeMap::new(),
        flow_invocations: Vec::new(),
        dynamic_texture_targets: Vec::new(),
        dynamic_texture_uploads: Vec::new(),
        product_selections: Vec::new(),
        viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
        contributions: PreparedFrameContributions::default(),
        shader: PreparedShaderSnapshot {
            registry_revision: 1,
        },
    }
}

#[test]
fn render_multi_surface_registry_scopes_surfaces_to_native_windows() {
    let mut registry = RenderSurfaceRegistryResource::default();
    let primary = registry.ensure_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
    let secondary_window = native_window(2);
    let secondary = registry.ensure_surface_for_native_window(secondary_window, (900, 600));

    assert_eq!(primary, RenderSurfaceId::primary());
    assert_ne!(primary, secondary);
    assert_eq!(
        registry.surface_for_native_window(secondary_window),
        Some(secondary)
    );
    assert_eq!(
        registry
            .record(secondary)
            .map(|record| record.native_window_id),
        Some(secondary_window)
    );
}

#[test]
fn render_multi_surface_prepared_frame_inspection_reports_surface_identity() {
    let mut registry = RenderSurfaceRegistryResource::default();
    let secondary_window = native_window(2);
    let secondary = registry.ensure_surface_for_native_window(secondary_window, (900, 600));
    assert_ne!(secondary, RenderSurfaceId::primary());
    assert_eq!(registry.primary_surface_id(), None);
    let prepared = frame(PreparedSurfaceInfo::for_surface(
        secondary,
        secondary_window,
        (900, 600),
    ));

    let inspection = inspect_prepared_render_frame(&prepared);

    assert_eq!(inspection.render_surface_id, secondary.raw());
    assert_eq!(inspection.native_window_id, Some(secondary_window.raw()));
    assert_eq!(inspection.surface_size, (900, 600));
}

#[test]
fn render_multi_surface_registry_reserves_primary_surface_for_primary_native_window() {
    let mut registry = RenderSurfaceRegistryResource::default();
    let secondary_window = native_window(3);

    let secondary = registry.ensure_surface_for_native_window(secondary_window, (900, 600));
    let primary = registry.ensure_surface_for_native_window(NativeWindowId::primary(), (1280, 720));

    assert_ne!(secondary, RenderSurfaceId::primary());
    assert_eq!(primary, RenderSurfaceId::primary());
    assert_eq!(
        registry.primary_surface_id(),
        Some(RenderSurfaceId::primary())
    );
    assert_eq!(
        registry.surface_for_native_window(secondary_window),
        Some(secondary)
    );
}
