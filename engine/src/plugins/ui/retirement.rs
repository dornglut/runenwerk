//! Exact-target, no-frame RunenUI withdrawal for future Native primary retirement.
//!
//! This does NOT retire a native window, settle M13, complete a GPU submission,
//! or certify Render G5/G7 physical detachment. Those semantic owners remain
//! separate. The caller first explicitly retires the original input recipient
//! and unpresents/unmounts the UI, then asks this owner to withdraw only the
//! exact target's remaining renderer paint and producer lease.

use runen_ecs::World;

use crate::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
use crate::plugins::render::{
    ManagedRenderProducerIds, RunenUiPaintSubmissionRegistryResource, render_integration_is_active,
};
use crate::runtime::NativeWindowId;

use super::native_input::UiNativeInputRetirementState;
use super::publish::retire_withdrawn_managed_producer;
use super::{
    UiNativeInputRecipientsResource, UiPluginStateResource,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotsResource,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSurfaceRetirementBlocker {
    LiveBinding,
    UnsettledPublication,
    LiveNativeInput,
    UnsettledNativeInput,
    PendingWithdrawal,
    RendererPaint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSurfaceRetirementStatus {
    NoUiCapability,
    Cleared,
    Blocked(UiSurfaceRetirementBlocker),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum UiSurfaceRetirementError {
    #[error("UI retirement owner is incomplete: {0}")]
    IncompleteUiOwner(&'static str),
    #[error("Render paint withdrawal owner is unavailable: {0}")]
    IncompleteRenderOwner(&'static str),
    #[error("native window and Render surface have different actual owners")]
    MismatchedTarget,
}

/// Inspects exactly one native/render target without changing owner state.
/// A cleared UI witness is never a Render in-flight or physical GPU certificate.
pub(crate) fn inspect_native_surface_ui_retirement(
    world: &World,
    native: NativeWindowId,
    surface: RenderSurfaceId,
) -> Result<UiSurfaceRetirementStatus, UiSurfaceRetirementError> {
    let ui = world.resource::<UiPluginStateResource>().ok();
    if !ui.is_some_and(UiPluginStateResource::is_installed) {
        if ui.is_some()
            || world.has_resource::<UiRuntimePresentationAssociationsResource>()
            || world.has_resource::<UiNativeInputRecipientsResource>()
            || world.has_resource::<UiRuntimeSlotsResource>()
        {
            return Err(UiSurfaceRetirementError::IncompleteUiOwner(
                "UiPlugin is absent but UI integration resources exist",
            ));
        }
        return Ok(UiSurfaceRetirementStatus::NoUiCapability);
    }
    let associations = world
        .resource::<UiRuntimePresentationAssociationsResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteUiOwner("presentation associations"))?;
    let recipients = world
        .resource::<UiNativeInputRecipientsResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteUiOwner("native input recipients"))?;
    world
        .resource::<UiRuntimeSlotsResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteUiOwner("typed UI runtime slots"))?;

    // Correlate only existing Render-owned facts; do not manufacture a target
    // or claim attachment when no Render capability is selected.
    if let Ok(surfaces) = world.resource::<RenderSurfaceRegistryResource>() {
        if surfaces
            .record(surface)
            .is_some_and(|record| record.native_window_id != native)
            || surfaces
                .surface_for_native_window(native)
                .is_some_and(|actual| actual != surface)
        {
            return Err(UiSurfaceRetirementError::MismatchedTarget);
        }
    }
    if render_integration_is_active(world) {
        world
            .resource::<RunenUiPaintSubmissionRegistryResource>()
            .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("paint registry"))?;
        world
            .resource::<ManagedRenderProducerIds>()
            .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("managed producer leases"))?;
    }

    if associations.has_live_surface_binding(surface) {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::LiveBinding,
        ));
    }
    if associations.has_unsettled_surface_publication(surface) {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::UnsettledPublication,
        ));
    }
    match recipients.native_ui_retirement_state(native) {
        UiNativeInputRetirementState::Live => {
            return Ok(UiSurfaceRetirementStatus::Blocked(
                UiSurfaceRetirementBlocker::LiveNativeInput,
            ));
        }
        UiNativeInputRetirementState::Unsettled => {
            return Ok(UiSurfaceRetirementStatus::Blocked(
                UiSurfaceRetirementBlocker::UnsettledNativeInput,
            ));
        }
        UiNativeInputRetirementState::NoUiRecipient | UiNativeInputRetirementState::Settled => {}
    }
    if associations.has_withdrawal_for_surface(surface) {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::PendingWithdrawal,
        ));
    }
    // Includes global UI paint applying to this surface. An untracked
    // residual submission is not permission for physical retirement.
    if world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .ok()
        .is_some_and(|paints| !paints.ordered_submissions_for_surface(surface).is_empty())
    {
        return Ok(UiSurfaceRetirementStatus::Blocked(
            UiSurfaceRetirementBlocker::RendererPaint,
        ));
    }
    Ok(UiSurfaceRetirementStatus::Cleared)
}

/// Removes only exact, already-unbound target paint without a synthetic frame.
///
/// This operation deliberately does not unbind a live UI, drop an accepted
/// generation, fabricate RunenUI input finality, or force GPU detachment.
/// Refusal leaves the complete outstanding withdrawal queue unchanged.
pub(crate) fn settle_native_surface_ui_withdrawals_without_frame(
    world: &mut World,
    native: NativeWindowId,
    surface: RenderSurfaceId,
) -> Result<UiSurfaceRetirementStatus, UiSurfaceRetirementError> {
    let status = inspect_native_surface_ui_retirement(world, native, surface)?;
    if status
        != UiSurfaceRetirementStatus::Blocked(UiSurfaceRetirementBlocker::PendingWithdrawal)
    {
        return Ok(status);
    }
    // All fallible resource preflight is before consuming any obligation.
    world
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("paint registry"))?;
    world
        .resource::<ManagedRenderProducerIds>()
        .map_err(|_| UiSurfaceRetirementError::IncompleteRenderOwner("managed producer leases"))?;

    let withdrawals = world
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .expect("preflight retained the UI presentation owner")
        .take_withdrawals_for_surface(surface);
    for (producer, target) in withdrawals {
        world
            .resource_mut::<RunenUiPaintSubmissionRegistryResource>()
            .expect("preflight retained Render paint owner")
            .remove_for_surface(&producer, target);
        retire_withdrawn_managed_producer(world, producer);
    }
    inspect_native_surface_ui_retirement(world, native, surface)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use runenui_core::{Effects, NoHostProtocol, ProducerBudget, StyleEnvironment, UiApp, View, text};

    use super::*;
    use crate::app::App;
    use crate::plugins::TimePlugin;
    use crate::plugins::render::{RenderFrameProducerId, RenderPlugin};
    use crate::plugins::ui::{
        AppUiExt, UiPlugin, UiRuntimeFontConfiguration, UiRuntimePresentationBinding,
        UiRuntimeSlotId, UiTypedScreenId,
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
        app.world_mut()
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
        assert!(app.world().resource::<UiRuntimePresentationAssociationsResource>().unwrap().binding(slot).is_some());

        app.world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .unbind(slot);
        assert_eq!(
            settle_native_surface_ui_withdrawals_without_frame(app.world_mut(), primary, surface),
            Err(UiSurfaceRetirementError::IncompleteRenderOwner("paint registry"))
        );
        assert!(app.world().resource::<UiRuntimePresentationAssociationsResource>().unwrap().has_withdrawal_for_surface(surface));
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
        associations.bind(binding(slot, producer(33), first)).unwrap();
        associations.unbind(slot);
        app.world_mut()
            .resource_mut::<RenderSurfaceRegistryResource>()
            .unwrap()
            .confirm_surface_attachment(first, primary, (320, 160))
            .unwrap();
        assert_eq!(
            settle_native_surface_ui_withdrawals_without_frame(
                app.world_mut(),
                second_native,
                first,
            ),
            Err(UiSurfaceRetirementError::MismatchedTarget)
        );
        assert!(app.world().resource::<UiRuntimePresentationAssociationsResource>().unwrap().has_withdrawal_for_surface(first));
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
        assert!(app.world().resource::<UiRuntimePresentationAssociationsResource>().unwrap().has_withdrawal_for_surface(second));
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
        let a = app.ui().mount_with_fonts(PaintScreen, &fonts).slot_id().unwrap();
        let b = app.ui().mount_with_fonts(PaintScreen, &fonts).slot_id().unwrap();

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

        let mut app = app.run_for_frames(1).expect("real RenderPrepare must publish paint");
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

}
