// Owner: Grotto Quest Engine - UI Plugin

use crate::plugins::scene::SceneResource;
use crate::runtime::ResMut;

include!("legacy/layout_core.rs");

pub(crate) fn ui_layout_system(scene_resource: ResMut<SceneResource>) -> anyhow::Result<()> {
    super::batch::ui_layout_system(scene_resource)
}
