// Owner: Grotto Quest Engine - UI Plugin

use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::SceneResource;
use crate::plugins::ui::domain::UiWorldHudStats;
use crate::runtime::{PreUpdate, RenderPrepare, Update};
use crate::state::{SceneRuntimeState, UiOverlayState};

pub struct UiInputPlugin;
pub struct UiRenderPlugin;

impl Plugin for UiInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.add_systems(PreUpdate, super::input::ui_hot_reload_system);
        app.add_systems(Update, super::input::ui_input_system);
        app.add_systems(Update, super::editor::ui_editor_system);
    }
}

impl Plugin for UiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<UiOverlayState>();
        app.init_resource::<UiWorldHudStats>();
        app.init_resource::<SceneRuntimeState>();
        app.add_systems(RenderPrepare, super::layout::ui_layout_system);
        app.add_systems(RenderPrepare, super::batch::ui_build_batches_system);
        app.add_systems(RenderPrepare, super::extract::ui_render_extract_system);
    }
}
