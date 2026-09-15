use crate::app::App;
use crate::app::domain::mode::AppMode;
use crate::plugins::{ActionState, InputState};
use crate::runtime::platform::PlatformWindowEventQueueResource;
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
        if !self.world.has_resource::<FramePacingPolicyResource>() {
            self.world
                .insert_resource(FramePacingPolicyResource::default());
        }
        if !self.world.has_resource::<FramePacingRuntimeStateResource>() {
            self.world
                .insert_resource(FramePacingRuntimeStateResource::default());
        }
        if !self.world.has_resource::<WindowStateRegistryResource>() {
            let registry = self
                .world
                .resource::<WindowState>()
                .ok()
                .map(WindowStateRegistryResource::from_legacy);
            if let Some(registry) = registry {
                self.world.insert_resource(registry);
            }
        }
        if !self
            .world
            .has_resource::<PlatformWindowEventQueueResource>()
        {
            self.world
                .insert_resource(PlatformWindowEventQueueResource::default());
        }
        if !self
            .world
            .has_resource::<NativeWindowHookRegistryResource>()
        {
            self.world
                .insert_resource(NativeWindowHookRegistryResource::default());
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
