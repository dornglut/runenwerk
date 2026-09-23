use crate::app::App;
use crate::app::domain::mode::AppMode;
use crate::plugins::{ActionState, InputState};
use crate::*;

impl App {
    /// Installs universal runtime resources required by App startup/frame execution.
    ///
    /// Capability state such as fixed cadence and simulation identity is installed by
    /// its owning integration plugin rather than by bare App construction.
    pub(crate) fn install_builtin_resources(&mut self) {
        if self.world.resource::<InputState>().is_err() {
            self.world.insert_resource(InputState::new());
        }
        if self.world.resource::<ActionState>().is_err() {
            self.world.insert_resource(ActionState::new());
        }
        if self.world.resource::<WindowState>().is_err() {
            let state = match self.mode {
                AppMode::Windowed => WindowState::windowed(self.title.clone()),
                AppMode::Headless => WindowState::headless(self.title.clone()),
            };
            self.world.insert_resource(state);
        }
        if !self
            .world
            .has_resource::<ProductPublicationRuntimeResource>()
        {
            self.world
                .insert_resource(ProductPublicationRuntimeResource::default());
        }
        if !self.world.has_resource::<QuerySnapshotRuntimeResource>() {
            self.world
                .insert_resource(QuerySnapshotRuntimeResource::default());
        }
        self.add_product_publication_handler(publish_staged_product_outcomes);
        self.add_query_snapshot_publication_handler(publish_staged_query_snapshots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::platform::PlatformWindowEventQueueResource;

    #[test]
    fn bare_apps_do_not_provision_native_window_provider_state() {
        for app in [App::new(), App::headless()] {
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
        }
    }
}
