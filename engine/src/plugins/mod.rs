pub mod debug_metrics;
pub mod fixed_step;
pub mod grid;
pub mod input;
pub mod net;
pub mod render;
pub mod scene;
pub mod scheduler_diagnostics;
pub(crate) mod shared;
pub mod time;
pub mod ui;

pub use debug_metrics::DebugMetricsPlugin;
pub use fixed_step::FixedStepPlugin;
pub use grid::GridPlugin;
pub use input::InputFinalizePlugin;
pub use net::{
    NetworkClientInbox, NetworkClientOutbox, NetworkClientPlugin, NetworkDiagnostics,
    NetworkServerInbox, NetworkServerOutbox, NetworkServerPlugin, PredictionDiagnostics,
    PredictionPlugin, ReplicationDiagnostics, ReplicationPlugin,
};
pub use render::RenderPlugin;
pub use scene::ScenePlugin;
pub use scheduler_diagnostics::SchedulerDiagnosticsPlugin;
pub use time::TimePlugin;
pub use ui::UiInputPlugin;
pub use ui::UiRenderPlugin;

use crate::plugin::Plugin;

pub fn default_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(TimePlugin),
        Box::new(FixedStepPlugin),
        Box::new(InputFinalizePlugin),
    ]
}

pub fn default_plugins_with_diagnostics() -> Vec<Box<dyn Plugin>> {
    let mut plugins = default_plugins();
    plugins.push(Box::new(SchedulerDiagnosticsPlugin));
    plugins
}

#[cfg(test)]
mod tests {
    use super::{
        FixedStepPlugin, InputFinalizePlugin, NetworkClientPlugin, NetworkServerPlugin,
        PredictionPlugin, RenderPlugin, ReplicationPlugin, ScenePlugin, SchedulerDiagnosticsPlugin,
        TimePlugin, UiInputPlugin, UiRenderPlugin, default_plugins,
    };
    use crate::plugin::Plugin;

    #[test]
    fn default_plugins_have_stable_order() {
        let plugins = default_plugins();
        let names: Vec<_> = plugins.iter().map(|plugin| plugin.name()).collect();
        assert_eq!(
            names,
            vec![
                std::any::type_name::<TimePlugin>(),
                std::any::type_name::<FixedStepPlugin>(),
                std::any::type_name::<InputFinalizePlugin>(),
            ]
        );
    }

    #[test]
    fn foundational_plugins_implement_plugin_trait() {
        fn assert_plugin<T: Plugin>() {}

        assert_plugin::<TimePlugin>();
        assert_plugin::<FixedStepPlugin>();
        assert_plugin::<InputFinalizePlugin>();
        assert_plugin::<NetworkClientPlugin>();
        assert_plugin::<NetworkServerPlugin>();
        assert_plugin::<ReplicationPlugin>();
        assert_plugin::<PredictionPlugin>();
        assert_plugin::<SchedulerDiagnosticsPlugin>();
        assert_plugin::<ScenePlugin>();
        assert_plugin::<UiInputPlugin>();
        assert_plugin::<UiRenderPlugin>();
        assert_plugin::<RenderPlugin>();
    }
}
