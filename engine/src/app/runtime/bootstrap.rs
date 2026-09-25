use crate::app::App;
use crate::*;

impl App {
    /// Installs universal runtime resources required by App startup/frame execution.
    ///
    /// Optional capability and product-integration state is installed by its owning
    /// composition surface rather than by bare App construction.
    pub(crate) fn install_builtin_resources(&mut self) {
        if !self
            .world
            .has_resource::<PrimaryPresentationMetricsResource>()
        {
            self.world
                .insert_resource(PrimaryPresentationMetricsResource::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{ActionState, InputState};
    use crate::runtime::platform::PlatformWindowEventQueueResource;

    #[test]
    fn bare_apps_do_not_provision_optional_host_or_input_provider_state() {
        for app in [App::new(), App::headless()] {
            assert_eq!(
                app.world()
                    .resource::<PrimaryPresentationMetricsResource>()
                    .expect("bare App should install primary presentation metrics"),
                &PrimaryPresentationMetricsResource::default(),
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
            assert!(
                app.world()
                    .resource::<crate::runtime::publication::PublicationHandlers>()
                    .is_err(),
                "bare App must not provision publication handler registry state"
            );
        }
    }
}
