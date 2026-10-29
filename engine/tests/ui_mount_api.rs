use engine::plugins::ui::{
    UiPlugin, UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource, UiRuntimeFontConfiguration,
    UiRuntimeSlotId, UiRuntimeSlotMountFailure, UiRuntimeSlotsResource, UiScreen, UiTypedScreenId,
    UiTypedSource,
};
use engine::prelude::{App, AppUiExt};
use ui_definition::{AuthoredId, UiNodeDefinition, UiValueBinding};
use ui_program::UiProgramSourceId;

#[derive(Debug, Copy, Clone)]
struct CounterScreen;

impl UiScreen for CounterScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("counter.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("counter.screen.source"),
            UiNodeDefinition::Column {
                id: AuthoredId::new("counter.root"),
                children: vec![UiNodeDefinition::Label {
                    id: AuthoredId::new("counter.title"),
                    label: UiValueBinding::static_text("Counter"),
                    availability: None,
                }],
            },
        )
    }
}

#[test]
fn ui_mount_normal_path_records_typed_slot_rejection_diagnostic() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    app.mount_ui(UnsupportedScreen);

    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert!(slots.is_empty());

    let diagnostics = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap();
    let diagnostic = diagnostics
        .entries()
        .last()
        .expect("rejected ordinary mount should remain observable");
    assert_eq!(
        diagnostic.code,
        UiRuntimeDiagnosticCode::RuntimeSlotMountRejected
    );
    let mount = diagnostic
        .runtime_slot_mount
        .as_ref()
        .expect("slot rejection should include owner-accurate mount facts");
    assert_eq!(mount.screen_id, "unsupported.screen");
    assert_eq!(
        mount.failure_reason,
        UiRuntimeSlotMountFailure::UnsupportedProjection
    );
}

#[derive(Debug, Copy, Clone)]
struct UnsupportedScreen;

impl UiScreen for UnsupportedScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("unsupported.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("unsupported.screen.source"),
            UiNodeDefinition::Spacer {
                id: AuthoredId::new("unsupported.spacer"),
            },
        )
    }
}

#[test]
fn ui_mount_normal_path_creates_runenui_runtime_slot() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    app.mount_ui(CounterScreen);

    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 1);
    let slot_id = slots.slot_ids().next().expect("one slot should be mounted");
    assert_eq!(slot_id.value(), 1);
    assert!(slots.contains(slot_id));
    assert_eq!(
        slots.screen_id(slot_id).map(UiTypedScreenId::as_str),
        Some("counter.screen")
    );
}

#[test]
fn ui_mount_advanced_path_returns_engine_slot_report() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let report = app.ui().mount(CounterScreen);

    assert!(report.accepted_mount(), "{report:?}");
    assert_eq!(report.screen_id().as_str(), "counter.screen");
    assert_eq!(report.slot_id().map(UiRuntimeSlotId::value), Some(1));
    assert!(report.failure().is_none());
}

#[test]
fn ui_mount_repeated_screen_instances_have_distinct_engine_slots() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let first = app.ui().mount(CounterScreen);
    let second = app.ui().mount(CounterScreen);

    assert_eq!(first.slot_id().map(UiRuntimeSlotId::value), Some(1));
    assert_eq!(second.slot_id().map(UiRuntimeSlotId::value), Some(2));
    assert_ne!(first.slot_id(), second.slot_id());

    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(
        slots
            .screen_id(first.slot_id().unwrap())
            .map(UiTypedScreenId::as_str),
        Some("counter.screen")
    );
    assert_eq!(
        slots
            .screen_id(second.slot_id().unwrap())
            .map(UiTypedScreenId::as_str),
        Some("counter.screen")
    );
}

#[test]
fn ui_mount_unmount_shuts_down_and_removes_engine_slot() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let report = app.ui().mount(CounterScreen);
    let slot_id = report.slot_id().unwrap();

    let slots = app
        .world_mut()
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap();
    assert_eq!(slots.unmount(slot_id), Ok(true));
    assert!(slots.is_empty());
    assert!(!slots.contains(slot_id));
    assert_eq!(slots.unmount(slot_id), Ok(false));
}

struct DiagnosticTypedApp;

impl runenui_core::UiApp for DiagnosticTypedApp {
    type State = ();
    type Action = ();
    type HostProtocol = runenui_core::NoHostProtocol;

    fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
        runenui_core::text("typed")
    }

    fn update(
        _: &mut Self::State,
        (): Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        runenui_core::Effects::none()
    }
}

#[test]
fn typed_app_mount_rejection_records_the_same_engine_diagnostic_as_counter() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);

    let fonts = UiRuntimeFontConfiguration::new(vec![vec![0, 1, 2, 3]]);
    let report = app.ui().mount_app::<DiagnosticTypedApp>(
        UiTypedScreenId::new("typed.invalid-font"),
        (),
        &fonts,
    );
    assert_eq!(
        report.failure(),
        Some(UiRuntimeSlotMountFailure::FontProvisioning)
    );
    assert!(
        app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .is_empty()
    );

    let diagnostics = app
        .world()
        .resource::<UiRuntimeDiagnosticsResource>()
        .unwrap();
    let last = diagnostics
        .entries()
        .last()
        .expect("typed mount rejection diagnostic");
    assert_eq!(last.code, UiRuntimeDiagnosticCode::RuntimeSlotMountRejected);
    let facts = last
        .runtime_slot_mount
        .as_ref()
        .expect("typed mount rejection preserves exact screen and reason");
    assert_eq!(facts.screen_id, "typed.invalid-font");
    assert_eq!(
        facts.failure_reason,
        UiRuntimeSlotMountFailure::FontProvisioning
    );
}

struct SecondaryTypedApp;

impl runenui_core::UiApp for SecondaryTypedApp {
    type State = usize;
    type Action = bool;
    type HostProtocol = runenui_core::NoHostProtocol;

    fn root(state: &Self::State) -> impl runenui_core::View<Self::Action> {
        runenui_core::text(format!("secondary {state}"))
    }

    fn update(
        state: &mut Self::State,
        action: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        if action {
            *state += 1;
        }
        runenui_core::Effects::none()
    }
}

#[test]
fn native_primary_typed_convenience_mounts_two_apps_under_one_selected_flow() {
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::render::{Gfx, ManagedRenderProducerIds, RenderFlowRegistryResource};
    use engine::plugins::ui::UiRuntimePresentationAssociationsResource;
    use engine::prelude::AppRenderExt;

    let mut app = App::new();
    app.select_native_ui_only().expect("native composition selected");
    let first = app
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("native.primary.first"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .expect("first typed app placed");
    let second = app
        .ui()
        .mount_app_on_primary::<SecondaryTypedApp>(
            UiTypedScreenId::new("native.primary.second"),
            0,
            &UiRuntimeFontConfiguration::default(),
        )
        .expect("second typed app placed");

    assert_ne!(first, second);
    assert_eq!(
        app.world()
            .resource::<RenderFlowRegistryResource>()
            .unwrap()
            .flow_count(),
        1,
        "the convenience path must not create another Present or RenderFlow"
    );
    assert!(!app.world().has_resource::<Gfx>());
    let associations = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    let first_binding = associations.binding(first).unwrap();
    let second_binding = associations.binding(second).unwrap();
    assert_eq!(first_binding.render_surface_id, RenderSurfaceId::primary());
    assert_eq!(second_binding.render_surface_id, RenderSurfaceId::primary());
    assert_ne!(first_binding.producer_id, second_binding.producer_id);
    let leases = app.world().resource::<ManagedRenderProducerIds>().unwrap();
    assert!(leases.is_owned_by(first_binding.producer_id, first.value()));
    assert!(leases.is_owned_by(second_binding.producer_id, second.value()));

    assert!(app.ui().unpresent(first).unwrap());
    assert!(app.ui().unmount_presented(second).unwrap().is_some());
    assert!(app.world().resource::<UiRuntimeSlotsResource>().unwrap().contains(first));
    assert!(!app.world().resource::<UiRuntimeSlotsResource>().unwrap().contains(second));
}

#[test]
fn native_primary_typed_convenience_preflight_rejects_without_mounting_or_gpu() {
    use engine::plugins::render::{Gfx, RenderPlugin};
    use engine::plugins::ui::UiAppPrimaryMountError;

    let mut no_plugins = App::new();
    let error = no_plugins
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("primary.no-plugins"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap_err();
    assert!(matches!(error, UiAppPrimaryMountError::Preflight(_)));
    assert!(!no_plugins.world().has_resource::<UiRuntimeSlotsResource>());
    assert!(!no_plugins.world().has_resource::<Gfx>());

    let mut no_render = App::new();
    no_render.add_plugin(UiPlugin);
    let error = no_render
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("primary.no-render"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap_err();
    assert!(matches!(error, UiAppPrimaryMountError::Preflight(_)));
    assert!(no_render.world().resource::<UiRuntimeSlotsResource>().unwrap().is_empty());
    assert!(!no_render.world().has_resource::<Gfx>());

    let mut no_ui = App::new();
    no_ui.add_plugin(RenderPlugin);
    let error = no_ui
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("primary.no-ui"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap_err();
    assert!(matches!(error, UiAppPrimaryMountError::Preflight(_)));
    assert!(!no_ui.world().has_resource::<UiRuntimeSlotsResource>());
    assert!(!no_ui.world().has_resource::<Gfx>());

    let mut headless = App::headless();
    headless.add_plugin(UiPlugin);
    headless.add_plugin(RenderPlugin);
    let error = headless
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("primary.headless"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap_err();
    assert!(matches!(error, UiAppPrimaryMountError::Preflight(_)));
    assert!(headless.world().resource::<UiRuntimeSlotsResource>().unwrap().is_empty());
    assert!(!headless.world().has_resource::<Gfx>());
}

#[test]
fn native_primary_typed_convenience_preserves_font_mount_rejection_report() {
    use engine::plugins::render::Gfx;
    use engine::plugins::ui::{UiAppPrimaryMountError, UiRuntimePresentationAssociationsResource};
    use engine::prelude::AppRenderExt;

    let mut app = App::new();
    app.select_native_ui_only().unwrap();
    let error = app
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("primary.bad-font"),
            (),
            &UiRuntimeFontConfiguration::new(vec![vec![0, 1, 2, 3]]),
        )
        .unwrap_err();
    let UiAppPrimaryMountError::MountRejected { report } = error else {
        panic!("font failure must return the original typed mount report");
    };
    assert_eq!(report.failure(), Some(UiRuntimeSlotMountFailure::FontProvisioning));
    assert_eq!(report.screen_id().as_str(), "primary.bad-font");
    assert!(app.world().resource::<UiRuntimeSlotsResource>().unwrap().is_empty());
    assert_eq!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .bindings()
            .count(),
        0
    );
    assert!(!app.world().has_resource::<Gfx>());
}

#[test]
fn native_primary_typed_convenience_placement_failure_retains_slot_and_foreign_lease() {
    use engine::plugins::render::{Gfx, ManagedRenderProducerIds};
    use engine::plugins::ui::{UiAppPrimaryMountError, UiRuntimePresentationAssociationsResource};
    use engine::prelude::AppRenderExt;
    use std::collections::BTreeSet;

    let mut app = App::new();
    app.select_native_ui_only().unwrap();
    // A pre-existing lease for the next slot owner forces the existing Render
    // allocator to reject without creating a new producer or presentation.
    let existing = app
        .world_mut()
        .resource_mut::<ManagedRenderProducerIds>()
        .unwrap()
        .allocate(1, &BTreeSet::new())
        .unwrap();
    let error = app
        .ui()
        .mount_app_on_primary::<DiagnosticTypedApp>(
            UiTypedScreenId::new("primary.conflict"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap_err();
    let UiAppPrimaryMountError::PlacementRetained { slot_id, source } = error else {
        panic!("failed placement must return a recoverable mounted slot");
    };
    assert_eq!(slot_id.value(), 1);
    assert!(source.to_string().contains("already holds a managed Render producer"));
    assert!(app.world().resource::<UiRuntimeSlotsResource>().unwrap().contains(slot_id));
    assert!(app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .binding(slot_id)
        .is_none());
    assert!(app
        .world()
        .resource::<ManagedRenderProducerIds>()
        .unwrap()
        .is_owned_by(existing, slot_id.value()));
    // Explicit retirement exposes the full RunenUI terminal batch to its
    // caller. The convenience path must not discard it or release this lease.
    let shutdown = app
        .world_mut()
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap()
        .unmount_with_retirement(slot_id)
        .expect("caller can explicitly settle retained slot");
    assert!(shutdown.is_some());
    assert!(app
        .world()
        .resource::<ManagedRenderProducerIds>()
        .unwrap()
        .is_owned_by(existing, slot_id.value()));
    assert!(!app.world().has_resource::<Gfx>());
}

#[test]
fn managed_ui_placement_allocates_distinct_render_producers_and_rebinds() {
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::render::{RenderFrameProducerId, RenderPlugin};
    use engine::plugins::ui::UiRuntimePresentationAssociationsResource;

    let mut app = App::new();
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    let first = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("managed.first"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .expect("first mount");
    let second = app
        .ui()
        .mount_app::<SecondaryTypedApp>(
            UiTypedScreenId::new("managed.second"),
            0,
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .expect("second mount");
    let primary = RenderSurfaceId::primary();
    app.ui().present(first, primary).expect("first placement");
    app.ui().present(second, primary).expect("second placement");
    let bindings = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    let first_id = bindings.binding(first).unwrap().producer_id;
    let second_id = bindings.binding(second).unwrap().producer_id;
    assert_ne!(first_id, second_id);
    assert_eq!(first_id, RenderFrameProducerId::try_from_raw(1).unwrap());

    let secondary = RenderSurfaceId::try_from_raw(2).unwrap();
    app.ui().present(first, secondary).expect("rebind");
    let bindings = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap();
    assert_eq!(
        bindings.binding(first).unwrap().render_surface_id,
        secondary
    );
    assert_eq!(bindings.binding(first).unwrap().producer_id, first_id);
    assert_eq!(bindings.binding(second).unwrap().render_surface_id, primary);

    assert!(app.ui().unpresent(first).unwrap());
    assert!(!app.ui().unpresent(first).unwrap());
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(first)
            .is_none()
    );
    assert!(app.ui().unmount_presented(second).unwrap().is_some());
    assert!(
        !app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .contains(second)
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(second)
            .is_none()
    );
    // It is valid to continue headlessly after withdrawing presentation.
    assert!(
        app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .contains(first)
    );
}

#[test]
fn managed_placement_skips_legacy_producer_id_and_rejects_missing_capabilities() {
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::render::{
        RenderFrameProducerId, RenderPlugin, SurfaceFrameSubmission,
        SurfaceFrameSubmissionRegistryResource,
    };
    use engine::plugins::ui::UiRuntimePresentationAssociationsResource;

    let mut app = App::new();
    app.add_plugin(UiPlugin);
    let slot = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("managed.not-ready"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    assert!(app.ui().present(slot, RenderSurfaceId::primary()).is_err());
    app.add_plugin(RenderPlugin);
    app.world_mut()
        .resource_mut::<SurfaceFrameSubmissionRegistryResource>()
        .unwrap()
        .replace(
            SurfaceFrameSubmission::new(RenderFrameProducerId::try_from_raw(1).unwrap())
                .with_render_surface(RenderSurfaceId::primary()),
        );
    app.ui()
        .present(slot, RenderSurfaceId::primary())
        .expect("reserved producer");
    let assigned = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .binding(slot)
        .unwrap()
        .producer_id;
    assert_ne!(assigned, RenderFrameProducerId::try_from_raw(1).unwrap());
}

#[test]
fn managed_placement_rejects_headless_host_without_changing_runtime() {
    use engine::plugins::render::RenderPlugin;
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::ui::UiRuntimePresentationAssociationsResource;

    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    let slot = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("headless.mount"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    assert!(app.ui().present(slot, RenderSurfaceId::primary()).is_err());
    assert!(
        app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .contains(slot)
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(slot)
            .is_none()
    );
}

#[test]
fn managed_producer_ownership_does_not_confuse_expert_binding_on_other_surface() {
    use engine::plugins::render::RenderPlugin;
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::ui::{
        UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding,
    };
    use runenui_core::StyleEnvironment;

    let mut app = App::new();
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    let owned = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("lease.owner"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    let foreign = app
        .ui()
        .mount_app::<SecondaryTypedApp>(
            UiTypedScreenId::new("lease.foreign"),
            0,
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    let primary = RenderSurfaceId::primary();
    let secondary = RenderSurfaceId::try_from_raw(2).unwrap();
    app.ui().present(owned, primary).unwrap();
    let producer = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .binding(owned)
        .unwrap()
        .producer_id;

    // Expert API can bind the same raw ID on another surface, but ownership
    // must not be inferred from that scalar identity.
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .bind(UiRuntimePresentationBinding::new(
            foreign,
            producer,
            secondary,
            StyleEnvironment::default(),
        ))
        .unwrap();

    assert!(app.ui().unpresent(foreign).is_err());
    assert!(app.ui().present(foreign, primary).is_err());
    assert!(app.ui().unmount_presented(foreign).is_err());
    assert!(
        app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .contains(foreign)
    );
    assert_eq!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(owned)
            .unwrap()
            .render_surface_id,
        primary
    );
    assert!(app.ui().unpresent(owned).unwrap());
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(foreign)
            .is_some()
    );
}

#[test]
fn managed_producer_lease_can_be_reconciled_after_expert_unbind() {
    use engine::plugins::render::RenderPlugin;
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::ui::UiRuntimePresentationAssociationsResource;

    let mut app = App::new();
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    let slot = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("lease.orphan"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    app.ui().present(slot, RenderSurfaceId::primary()).unwrap();
    let former_producer = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .binding(slot)
        .unwrap()
        .producer_id;
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .unbind(slot);
    // An expert unbind leaves a managed lease, so a second presentation
    // must not silently allocate a second lease for the same UI slot.
    assert!(app.ui().present(slot, RenderSurfaceId::primary()).is_err());
    assert!(app.ui().unpresent(slot).unwrap());
    assert!(!app.ui().unpresent(slot).unwrap());
    app.ui().present(slot, RenderSurfaceId::primary()).unwrap();
    let replacement_producer = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .binding(slot)
        .unwrap()
        .producer_id;
    assert_ne!(former_producer, replacement_producer);
}

#[test]
fn late_explicit_legacy_claim_rejects_managed_producer_before_render_prepare() {
    use engine::plugins::render::{
        ManagedRenderProducerIds, SurfaceFrameSubmission, SurfaceFrameSubmissionRegistryResource,
    };
    use engine::plugins::{RenderPlugin, TimePlugin};
    use std::collections::BTreeSet;

    let mut app = App::headless();
    app.add_plugin(TimePlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);
    let producer = app
        .world_mut()
        .resource_mut::<ManagedRenderProducerIds>()
        .unwrap()
        .allocate(123, &BTreeSet::new())
        .unwrap();
    app.world_mut()
        .resource_mut::<SurfaceFrameSubmissionRegistryResource>()
        .unwrap()
        .replace(SurfaceFrameSubmission::new(producer));
    let error = app
        .run_for_frames(1)
        .err()
        .expect("a late legacy claim must fail before frame preparation");
    assert!(format!("{error:#}").contains("leased to UI slot 123"));
}

#[test]
fn expert_unbind_retires_managed_lease_at_next_render_publication() {
    use engine::plugins::TimePlugin;
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::render::{ManagedRenderProducerIds, RenderPlugin};
    use engine::plugins::ui::{
        UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding,
    };
    use runenui_core::StyleEnvironment;
    use std::collections::BTreeSet;

    let mut app = App::headless();
    app.add_plugin(TimePlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);
    let slot = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("managed.withdrawal"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    let producer = app
        .world_mut()
        .resource_mut::<ManagedRenderProducerIds>()
        .unwrap()
        .allocate(slot.value(), &BTreeSet::new())
        .unwrap();
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .bind(UiRuntimePresentationBinding::new(
            slot,
            producer,
            RenderSurfaceId::primary(),
            StyleEnvironment::default(),
        ))
        .unwrap();
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .unbind(slot);
    assert!(
        app.world()
            .resource::<ManagedRenderProducerIds>()
            .unwrap()
            .is_owned_by(producer, slot.value())
    );
    let app = app
        .run_for_frames(1)
        .expect("withdrawals settle under headless RenderPrepare");
    assert!(
        !app.world()
            .resource::<ManagedRenderProducerIds>()
            .unwrap()
            .is_owned_by(producer, slot.value())
    );
}

#[test]
fn expert_unmount_retires_managed_lease_and_binding() {
    use engine::plugins::TimePlugin;
    use engine::plugins::render::host::RenderSurfaceId;
    use engine::plugins::render::{ManagedRenderProducerIds, RenderPlugin};
    use engine::plugins::ui::{
        UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding,
    };
    use runenui_core::StyleEnvironment;
    use std::collections::BTreeSet;

    let mut app = App::headless();
    app.add_plugin(TimePlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);
    let slot = app
        .ui()
        .mount_app::<DiagnosticTypedApp>(
            UiTypedScreenId::new("managed.retired"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .unwrap();
    let producer = app
        .world_mut()
        .resource_mut::<ManagedRenderProducerIds>()
        .unwrap()
        .allocate(slot.value(), &BTreeSet::new())
        .unwrap();
    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .unwrap()
        .bind(UiRuntimePresentationBinding::new(
            slot,
            producer,
            RenderSurfaceId::primary(),
            StyleEnvironment::default(),
        ))
        .unwrap();
    let retirement = app
        .world_mut()
        .resource_mut::<UiRuntimeSlotsResource>()
        .unwrap()
        .unmount_with_retirement(slot);
    assert!(retirement.unwrap().is_some());

    let app = app
        .run_for_frames(1)
        .expect("missing slot withdraws at RenderPrepare");
    assert!(
        !app.world()
            .resource::<ManagedRenderProducerIds>()
            .unwrap()
            .is_owned_by(producer, slot.value())
    );
    assert!(
        app.world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap()
            .binding(slot)
            .is_none()
    );
}
