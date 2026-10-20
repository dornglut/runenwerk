use super::*;
use super::tests::runner_with_frame_pacing;
use crate::plugins::render::host::{
    RenderSurfaceLifecycleState, RenderSurfaceRegistryResource,
};

#[test]
fn refused_secondary_close_preserves_host_identity_and_redraw() {
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner
        .state
        .world
        .insert_resource(RenderSurfaceRegistryResource::default());
    let requested = runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .request_window("secondary", (640, 480));
    let secondary = requested.native_window_id;
    let id = runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(secondary, (640, 480));
    runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(id, secondary, (640, 480))
        .unwrap();
    runner.owned_redraws.insert(secondary);
    let input = runner
        .input_adapter
        .window_context(secondary, winit::event::DeviceId::dummy())
        .unwrap();
    let source = runner.input_adapter.window_source_context(secondary);
    assert_eq!(source.map(|context| context.source), Some(input.source));

    let error = runner
        .authorize_secondary_close(secondary)
        .expect_err("an attached surface without GPU must fail closed");
    assert!(format!("{error:#}").contains("no GPU attachment"));
    assert!(runner.owned_redraws.contains(&secondary));
    assert_eq!(
        runner.input_adapter.window_source_context(secondary),
        source
    );
    assert!(
        runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap()
            .record(secondary)
            .is_some()
    );
    let render = runner
        .state
        .world
        .resource::<RenderSurfaceRegistryResource>()
        .unwrap();
    assert_eq!(render.surface_for_native_window(secondary), Some(id));
    assert_eq!(
        render.record(id).map(|record| record.lifecycle_state),
        Some(RenderSurfaceLifecycleState::Attached)
    );
    assert!(
        !runner
            .window_realizer
            .realized_windows()
            .iter()
            .any(|(_, id, _)| *id == secondary)
    );
}

#[test]
fn creation_failure_cannot_discard_unresolved_render_attachment() {
    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner
        .state
        .world
        .insert_resource(RenderSurfaceRegistryResource::default());
    let request = runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .request_window("secondary", (640, 480));
    let surface = runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(request.native_window_id, (640, 480));
    runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(surface, request.native_window_id, (640, 480))
        .unwrap();
    assert!(
        runner
            .mark_window_creation_failed(request.native_window_id, "injected failure")
            .is_err()
    );
    assert_ne!(
        runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap()
            .record(request.native_window_id)
            .unwrap()
            .lifecycle_state,
        crate::runtime::NativeWindowLifecycleState::CreationFailed
    );
    assert_eq!(
        runner
            .state
            .world
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap()
            .surface_for_native_window(request.native_window_id),
        Some(surface)
    );
}

