//! U5 typed-paint preparation and headless publication proof.
//!
//! Deliberately no renderer-presented receipt is forged by a prepared-frame test:
//! only exact terminal RunenGPU and RunenRender evidence may promote displayed input.

use std::collections::BTreeMap;

use engine::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
use engine::plugins::render::{
    FeatureContributionStatus, PreparedUiFrameResource, PreparedUiSubmissionKind,
    RenderFrameProducerId, RenderPlugin, RunenUiPaintSubmissionRegistryResource,
    SurfaceFrameRoute, SurfaceFrameSubmission, SurfaceFrameSubmissionOrder,
    SurfaceFrameSubmissionRegistryResource,
};
use engine::plugins::ui::{
    AppUiExt, UiPlugin, UiRuntimeFontConfiguration, UiRuntimeNativeMapping,
    UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding,
    UiRuntimeSlotsResource, UiScreen, UiTypedActionDescriptor, UiTypedScreenId,
    UiTypedSource,
};
use engine::plugins::TimePlugin;
use engine::prelude::App;
use engine::runtime::{NativeWindowId, WindowStateRegistryResource};

use runenui_core::{FontFamilyName, GenericFontFamily, StyleEnvironment};
use ui_controls::BUTTON_CONTROL_KIND_ID;
use ui_definition::{
    AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
    AuthoredControlValue, AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_program::{
    RouteCapability, RouteId, RouteSchemaVersion, UiProgramSourceId,
};
use ui_schema::UiSchemaRef;

const CONTROLLED_FONT: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/fonts/JetBrainsMono-Regular.ttf"));

#[derive(Debug, Clone, Copy)]
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
                children: vec![
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.title"),
                        label: UiValueBinding::static_text("Counter"),
                        availability: None,
                    },
                    counter_output(),
                ],
            },
        )
        .with_action_descriptor(UiTypedActionDescriptor::new(
            engine::plugins::ui::UiTypedActionId::new("counter.increment.action"),
            RouteId::new("counter.increment"),
            RouteSchemaVersion::new(1),
            UiSchemaRef::new("runenwerk.ui.controls.button.event", 1),
            RouteCapability::new("counter.action.increment"),
        ))
    }
}

fn counter_output() -> UiNodeDefinition {
    let mut properties = BTreeMap::new();
    properties.insert(
        "label".to_owned(),
        AuthoredControlValue::String("Counter output".to_owned()),
    );
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "selected".to_owned(),
        AuthoredBindingRef::new("counter.output.selected"),
    );
    UiNodeDefinition::Control {
        id: AuthoredId::new("counter.output"),
        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
        properties,
        bindings,
        route: Some(AuthoredRouteId::new("counter.increment")),
        accessibility: Some(AuthoredControlAccessibilityDefinition {
            role: "button".to_owned(),
            label: Some("Counter output".to_owned()),
        }),
        children: Vec::new(),
    }
}

fn controlled_fonts() -> UiRuntimeFontConfiguration {
    UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT.to_vec()]).with_generic_mapping(
        GenericFontFamily::SansSerif,
        vec![FontFamilyName::new("JetBrains Mono").expect("valid test font family")],
    )
}

fn producer(id: u64) -> RenderFrameProducerId {
    RenderFrameProducerId::try_from_raw(id).expect("valid producer")
}

#[test]
fn mounted_counter_requires_no_native_or_renderer_resource() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);
    let mount = app.ui().mount_with_fonts(CounterScreen, &controlled_fonts());
    let slot = mount.slot_id().expect("headless typed Counter must mount");
    assert!(app
        .world()
        .resource::<UiRuntimeSlotsResource>()
        .expect("RunenUI slots resource")
        .contains(slot));
    assert!(app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .expect("presentation ledger installed")
        .binding(slot)
        .is_none());
    assert!(app
        .world()
        .resource::<RenderSurfaceRegistryResource>()
        .is_err());
}

#[test]
fn attached_counter_prepares_runenui_paint_but_never_forges_presented_input() {
    let mut app = App::headless();
    app.add_plugin(TimePlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(RenderPlugin);
    let slot = app
        .ui()
        .mount_with_fonts(CounterScreen, &controlled_fonts())
        .slot_id()
        .expect("controlled typed Counter must mount");
    let surface = RenderSurfaceId::primary();
    let mut windows = WindowStateRegistryResource::default();
    let native = windows.register_primary_window("Counter", (320, 160), 1.0, true);
    assert_eq!(native, NativeWindowId::primary());
    app.insert_resource(windows);
    app.world_mut()
        .resource_mut::<RenderSurfaceRegistryResource>()
        .expect("render surface registry installed")
        .confirm_surface_attachment(surface, native, (320, 160))
        .expect("attached source facts");

    app.world_mut()
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .expect("presentation associations installed")
        .bind(
            UiRuntimePresentationBinding::new(
                slot,
                producer(71),
                surface,
                StyleEnvironment::default(),
            )
            .with_route(SurfaceFrameRoute::Screen)
            .with_order(SurfaceFrameSubmissionOrder::new(20, 0)),
        )
        .expect("one valid producer/surface binding");

    // Neighboring legacy producers must retain their positions around RunenUI.
    let registry = app
        .world_mut()
        .resource_mut::<SurfaceFrameSubmissionRegistryResource>()
        .expect("legacy registry remains installed");
    registry.replace(
        SurfaceFrameSubmission::new(producer(70))
            .with_order(SurfaceFrameSubmissionOrder::new(0, 0)),
    );
    registry.replace(
        SurfaceFrameSubmission::new(producer(72))
            .with_order(SurfaceFrameSubmissionOrder::new(40, 0)),
    );

    let app = app
        .run_for_frames(1)
        .expect("headless RenderPrepare must publish without a GPU");
    let paint = app
        .world()
        .resource::<RunenUiPaintSubmissionRegistryResource>()
        .expect("typed paint intake installed");
    let published = paint.ordered_submissions_for_surface(surface);
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].producer_id, producer(71));
    assert!(!published[0].publication.scene().is_empty());
    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("UI feature prepared on the same surface");
    assert_eq!(
        prepared.status_for_surface(surface),
        FeatureContributionStatus::Ready,
    );
    let ordered = prepared.payload_for_surface(surface);
    assert_eq!(
        ordered.ordered,
        vec![
            PreparedUiSubmissionKind::Legacy(0),
            PreparedUiSubmissionKind::RunenUi(0),
            PreparedUiSubmissionKind::Legacy(1),
        ]
    );
    assert_eq!(ordered.submissions[0].submission_order, 0);
    assert_eq!(ordered.runenui_submissions[0].submission_order, 1);
    assert_eq!(ordered.submissions[1].submission_order, 2);
    assert_eq!(ordered.submissions[0].producer_id, producer(70));
    assert_eq!(ordered.submissions[1].producer_id, producer(72));
    let ledger = app
        .world()
        .resource::<UiRuntimePresentationAssociationsResource>()
        .expect("presentation ledger");
    assert!(ledger.pending(slot).is_some());
    assert!(ledger
        .displayed_for_mapping(
            slot,
            UiRuntimeNativeMapping::new(native, surface, (320, 160), 1.0)
                .expect("valid native mapping"),
        )
        .is_none(), "RenderPrepare is not a terminal display receipt");
}
