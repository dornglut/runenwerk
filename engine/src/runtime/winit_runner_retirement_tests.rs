use super::tests::runner_with_frame_pacing;
use super::*;
use crate::plugins::render::host::{RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};

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
        .native_input_mut()
        .window_context(secondary, winit::event::DeviceId::dummy())
        .unwrap();
    let source = runner.native_input().window_source_context(secondary);
    assert_eq!(source.map(|context| context.source), Some(input.source));

    let error = runner
        .authorize_secondary_close(secondary)
        .expect_err("an attached surface without GPU must fail closed");
    assert!(format!("{error:#}").contains("no GPU attachment"));
    assert!(runner.owned_redraws.contains(&secondary));
    assert_eq!(
        runner.native_input().window_source_context(secondary),
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

#[test]
fn saturated_source_journal_cannot_retire_render_or_a_selected_ui_recipient() {
    use crate::plugins::ui::{
        NativeInputRecipient, UiRuntimeFontConfiguration, UiRuntimeSlotsResource, UiTypedScreenId,
    };

    struct Probe;
    impl runenui_core::UiApp for Probe {
        type State = ();
        type Action = ();
        type HostProtocol = runenui_core::NoHostProtocol;

        fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
            runenui_core::text("native secondary close admission")
        }

        fn update(
            _: &mut Self::State,
            _: Self::Action,
        ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        }
    }

    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(InputState::new());
    runner
        .state
        .world
        .insert_resource(RenderSurfaceRegistryResource::default());
    let secondary = NativeWindowId::try_from_raw(2).expect("secondary window");
    runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .unwrap()
        .register_created_window(secondary, "secondary", (640, 480), 1.0, false);
    let surface = runner
        .state
        .world
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(secondary, (640, 480));
    let source = runner
        .native_input_mut()
        .window_context(secondary, winit::event::DeviceId::dummy())
        .expect("one source-qualified physical input");

    let mut slots = UiRuntimeSlotsResource::default();
    let slot = slots
        .mount_app::<Probe>(
            UiTypedScreenId::new("native.secondary.close.preflight"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .expect("original typed RunenUI runtime");
    let mut recipients = UiNativeInputRecipientsResource::default();
    recipients
        .bind(
            secondary,
            NativeInputRecipient::RunenUiSlot(slot),
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .unwrap(),
            &mut slots,
        )
        .expect("selected native UI recipient");
    runner.state.world.insert_resource(slots);
    runner.state.world.insert_resource(recipients);
    runner
        .state
        .world
        .insert_resource(UiRuntimePresentationAssociationsResource::default());

    let journal = runner
        .state
        .world
        .resource_mut::<PlatformWindowEventQueueResource>()
        .expect("single bounded native input journal");
    for _ in 0..4096 {
        journal
            .publish(PlatformWindowEvent::new(
                secondary,
                PlatformEvent::RedrawRequested,
            ))
            .expect("fill exactly the finite original-event capacity");
    }
    assert!(
        runner.authorize_secondary_close(secondary).is_err(),
        "the original source loss must refuse BEFORE Render is logically retired"
    );
    assert_eq!(
        runner
            .state
            .world
            .resource::<RenderSurfaceRegistryResource>()
            .unwrap()
            .surface_for_native_window(secondary),
        Some(surface),
        "journal pressure must not release an original secondary Render reservation"
    );
    assert_eq!(
        runner.native_input().window_source_context(secondary).map(|ctx| ctx.source),
        Some(source.source),
        "no fabricated source retirement on journal refusal"
    );
    assert_eq!(
        runner.native_input_recipient(secondary).unwrap().0,
        NativeInputRecipient::RunenUiSlot(slot),
        "no RunenUI recipient or binding revocation before original admission"
    );
    assert_eq!(
        runner
            .state
            .world
            .resource::<PlatformWindowEventQueueResource>()
            .unwrap()
            .events()
            .len(),
        4096,
        "the journal must retain its exact original events without partial mutation"
    );
}
