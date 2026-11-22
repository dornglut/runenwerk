use std::collections::BTreeSet;

use runenui_core::{Effects, NoHostProtocol, ProducerBudget, StyleEnvironment, UiApp, View, text};

use super::*;
use crate::app::App;
use crate::plugins::TimePlugin;
use crate::plugins::render::{RenderFrameProducerId, RenderPlugin};
use crate::plugins::ui::{
    AppUiExt, UiPlugin, UiRuntimeFontConfiguration, UiRuntimePresentationBinding, UiRuntimeSlotId,
    UiTypedScreenId,
};

struct SimpleUi;

impl UiApp for SimpleUi {
    type State = ();
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn root(_: &Self::State) -> impl View<Self::Action> {
        text("retirement")
    }

    fn update(
        _: &mut Self::State,
        (): Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        Effects::none()
    }

    fn initial_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
    }

    fn action_producer_budget(_: &Self::State, _: &Self::Action) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
    }

    fn interaction_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0)
    }
}

fn app(with_render: bool) -> App {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);
    if with_render {
        app.add_plugin(RenderPlugin);
    }
    app
}

fn mount(app: &mut App) -> UiRuntimeSlotId {
    app.ui()
        .mount_app::<SimpleUi>(
            UiTypedScreenId::new("retirement.ui"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .expect("typed UI must mount")
}

fn producer(raw: u64) -> RenderFrameProducerId {
    RenderFrameProducerId::try_from_raw(raw).expect("valid producer")
}

fn binding(
    slot: UiRuntimeSlotId,
    producer: RenderFrameProducerId,
    surface: RenderSurfaceId,
) -> UiRuntimePresentationBinding {
    UiRuntimePresentationBinding::new(slot, producer, surface, StyleEnvironment::default())
}

#[test]
fn no_ui_is_distinct_from_incomplete_ui_owner() {
    let bare = App::headless();
    let primary = NativeWindowId::primary();
    let surface = RenderSurfaceId::primary();
    assert_eq!(
        inspect_native_surface_ui_retirement(bare.world(), primary, surface),
        Ok(UiSurfaceRetirementStatus::NoUiCapability)
    );

    let mut app = app(false);
    assert_eq!(
        inspect_native_surface_ui_retirement(app.world(), primary, surface),
        Ok(UiSurfaceRetirementStatus::Cleared)
    );
    let _ = app
        .world_mut()
        .remove_resource::<UiRuntimePresentationAssociationsResource>();
    assert_eq!(
        inspect_native_surface_ui_retirement(app.world(), primary, surface),
        Err(UiSurfaceRetirementError::IncompleteUiOwner(
            "presentation associations"
        ))
    );
}

#[test]
fn bound_primary_refuses_without_mutation_and_missing_paint_preserves_withdrawal() {
    let primary = NativeWindowId::primary();
    let surface = RenderSurfaceId::primary();
    let mut app = app(false);
    let slot = mount(&mut app);
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .bind(binding(slot, producer(21), surface))
        .unwrap();
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, surface),
        Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::LiveBinding
        ))
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(slot)
            .is_some()
    );

    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .unbind(slot);
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, surface),
        Err(UiSurfaceRetirementError::IncompleteRenderOwner(
            "paint registry"
        ))
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .has_withdrawal_for_surface(surface)
    );
}

#[test]
fn original_native_ui_request_and_world_only_retirement_do_not_certify_m13() {
    let primary = NativeWindowId::primary();
    let surface = RenderSurfaceId::primary();
    let mut app = app(false);
    let slot = mount(&mut app);
    let mut windows = crate::runtime::WindowStateRegistryResource::default();
    windows.register_primary_window("UI", (640, 480), 1.0, true);
    app.insert_resource(windows);
    app.ui().bind_native_primary_input(slot).unwrap();
    assert_eq!(
        inspect_native_surface_ui_retirement(app.world(), primary, surface),
        Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::LiveNativeInput
        ))
    );
    crate::plugins::ui::realize_requested_native_primary(app.world_mut()).unwrap();
    crate::plugins::ui::retire_native_input_recipient(app.world_mut(), primary).unwrap();
    assert_eq!(
        inspect_native_surface_ui_retirement(app.world(), primary, surface),
        Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::UnsettledNativeInput
        ))
    );
}

#[test]
fn rebind_keeps_same_producer_lease_on_surviving_target() {
    let primary = NativeWindowId::primary();
    let first = RenderSurfaceId::primary();
    let second = RenderSurfaceId::try_from_raw(2).unwrap();
    let mut app = app(true);
    let slot = mount(&mut app);
    let managed = app
        .world_mut()
        .resource_mut::<ManagedRenderProducerIds>()
        .unwrap()
        .allocate(slot.value(), &BTreeSet::new())
        .unwrap();
    let associations = app
        .world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    associations.bind(binding(slot, managed, first)).unwrap();
    associations.bind(binding(slot, managed, second)).unwrap();
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, first),
        Ok(UiSurfaceRetirementStatus::Cleared)
    );
    let leases = app.world().resource::<ManagedRenderProducerIds>().unwrap();
    assert!(leases.is_owned_by(managed, slot.value()));
    let associations = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    assert!(associations.has_live_surface_binding(second));
    assert!(!associations.has_withdrawal_for_surface(first));
}

#[test]
fn target_mismatch_rejects_without_consuming_withdrawal() {
    let primary = NativeWindowId::primary();
    let second_native = NativeWindowId::try_from_raw(2).unwrap();
    let first = RenderSurfaceId::primary();
    let mut app = app(true);
    let slot = mount(&mut app);
    let associations = app
        .world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    associations
        .bind(binding(slot, producer(33), first))
        .unwrap();
    associations.unbind(slot);
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(first, primary, (320, 160))
        .unwrap();
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), second_native, first),
        Err(UiSurfaceRetirementError::MismatchedTarget)
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .has_withdrawal_for_surface(first)
    );
}

#[test]
fn exactly_scoped_no_frame_withdrawal_preserves_secondary_and_is_idempotent() {
    // This exercises the real UiPlugin/RenderPlugin registries, never Gfx
    // or synthetic completion/Present evidence.
    let primary = NativeWindowId::primary();
    let secondary = NativeWindowId::try_from_raw(2).unwrap();
    let first = RenderSurfaceId::primary();
    let mut app = app(true);
    let a = mount(&mut app);
    let b = mount(&mut app);
    let second = app
        .world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .reserve_surface_for_native_window(secondary, (320, 160));
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(first, primary, (320, 160))
        .unwrap();
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap()
        .confirm_surface_attachment(second, secondary, (320, 160))
        .unwrap();
    let a_producer = producer(41);
    let b_producer = producer(42);
    let ledger = app
        .world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    ledger.bind(binding(a, a_producer, first)).unwrap();
    ledger.bind(binding(b, b_producer, second)).unwrap();
    ledger.unbind(a);
    ledger.unbind(b);

    assert_eq!(
        inspect_native_surface_ui_retirement(app.world(), primary, first),
        Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::PendingWithdrawal
        ))
    );
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, first),
        Ok(UiSurfaceRetirementStatus::Cleared)
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .has_withdrawal_for_surface(second)
    );
    assert_eq!(
        inspect_native_surface_ui_retirement(app.world(), secondary, second),
        Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::PendingWithdrawal
        ))
    );
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, first),
        Ok(UiSurfaceRetirementStatus::Cleared),
        "double close cannot drain another target"
    );
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), secondary, second),
        Ok(UiSurfaceRetirementStatus::Cleared)
    );
}

/// A real RenderPrepare publishes both paints. The following cleanup does
/// not run another Update, RenderPrepare, RenderSubmit or Present.
#[test]
fn real_prepared_paint_is_withdrawn_only_for_primary_without_another_frame() {
    use crate::plugins::render::RunenUiPaintSubmissionRegistryResource;
    use crate::plugins::ui::{UiScreen, UiTypedSource};
    use crate::runtime::WindowStateRegistryResource;

    #[derive(Clone, Copy, Debug)]
    struct PaintScreen;

    impl UiScreen for PaintScreen {
        fn screen_id(&self) -> UiTypedScreenId {
            UiTypedScreenId::new("retirement.paint")
        }

        fn build_source(&self) -> UiTypedSource {
            UiTypedSource::new(
                self.screen_id(),
                ui_program::UiProgramSourceId::new("retirement.paint.source"),
                ui_definition::UiNodeDefinition::Column {
                    id: ui_definition::AuthoredId::new("retirement.root"),
                    children: vec![ui_definition::UiNodeDefinition::Label {
                        id: ui_definition::AuthoredId::new("retirement.title"),
                        label: ui_definition::UiValueBinding::static_text("Retirement"),
                        availability: None,
                    }],
                },
            )
        }
    }

    let font = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../assets/fonts/JetBrainsMono-Regular.ttf"
    ));
    let fonts = UiRuntimeFontConfiguration::new(vec![font.to_vec()]).with_generic_mapping(
        runenui_core::GenericFontFamily::SansSerif,
        vec![runenui_core::FontFamilyName::new("JetBrains Mono").unwrap()],
    );
    let primary = NativeWindowId::primary();
    let secondary = NativeWindowId::try_from_raw(2).unwrap();
    let first = RenderSurfaceId::primary();
    let mut app = App::headless();
    app.add_plugin(TimePlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);
    let a = app
        .ui()
        .mount_with_fonts(PaintScreen, &fonts)
        .slot_id()
        .unwrap();
    let b = app
        .ui()
        .mount_with_fonts(PaintScreen, &fonts)
        .slot_id()
        .unwrap();

    let mut windows = WindowStateRegistryResource::default();
    windows.register_primary_window("Primary", (320, 160), 1.0, true);
    windows.register_created_window(secondary, "Secondary", (320, 160), 1.0, false);
    app.insert_resource(windows);
    let surfaces = app
        .world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .unwrap();
    let second = surfaces.reserve_surface_for_native_window(secondary, (320, 160));
    surfaces
        .confirm_surface_attachment(first, primary, (320, 160))
        .unwrap();
    surfaces
        .confirm_surface_attachment(second, secondary, (320, 160))
        .unwrap();
    let a_producer = producer(71);
    let b_producer = producer(72);
    let associations = app
        .world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    associations.bind(binding(a, a_producer, first)).unwrap();
    associations.bind(binding(b, b_producer, second)).unwrap();

    let mut app = app
        .run_for_frames(1)
        .expect("real RenderPrepare must publish paint");
    let paints = app
        .world()
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .unwrap();
    assert!(paints.get_for_surface(&a_producer, first).is_some());
    assert!(paints.get_for_surface(&b_producer, second).is_some());

    let associations = app
        .world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    associations.unbind(a);
    associations.unbind(b);
    assert_eq!(
        settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, first),
        Ok(UiSurfaceRetirementStatus::Cleared)
    );
    let paints = app
        .world()
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .unwrap();
    assert!(paints.get_for_surface(&a_producer, first).is_none());
    assert!(paints.get_for_surface(&b_producer, second).is_some());
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .has_withdrawal_for_surface(second),
        "secondary's existing withdrawal must remain pending"
    );
}
