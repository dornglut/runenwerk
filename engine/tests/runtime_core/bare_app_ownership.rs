use engine::plugins::{ActionState, InputState};
use engine::prelude::App;
use engine::runtime::platform::PlatformWindowEventQueueResource;
use engine::{
    PrimaryPresentationMetricsResource, ProductPublicationRuntimeResource,
    QuerySnapshotRuntimeResource, WindowStateRegistryResource,
};

#[test]
fn bare_apps_do_not_manufacture_optional_integration_state() {
    for app in [App::new(), App::headless()] {
        assert!(
            app.world()
                .resource::<PrimaryPresentationMetricsResource>()
                .is_err(),
            "bare App must not manufacture logical presentation state"
        );
        assert!(
            app.world().resource::<InputState>().is_err(),
            "bare App must not provision physical input capability state"
        );
        assert!(
            app.world().resource::<ActionState>().is_err(),
            "bare App must not provision product action capability state"
        );
        assert!(
            app.world()
                .resource::<WindowStateRegistryResource>()
                .is_err(),
            "bare App must not provision native window lifecycle state"
        );
        assert!(
            app.world()
                .resource::<PlatformWindowEventQueueResource>()
                .is_err(),
            "bare App must not provision native platform-window event state"
        );
        assert!(
            app.world()
                .resource::<ProductPublicationRuntimeResource>()
                .is_err(),
            "bare App must not provision product-publication integration state"
        );
        assert!(
            app.world()
                .resource::<QuerySnapshotRuntimeResource>()
                .is_err(),
            "bare App must not provision query-publication integration state"
        );
    }
}
