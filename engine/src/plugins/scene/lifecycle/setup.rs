use super::super::runtime::{
    SceneTemplateFlowResource, publish_scene_state, sync_overlay_viewport,
};
use crate::plugins::scene::ui::UiPresentationMode;
use crate::plugins::{SceneManager, SceneResource};
use crate::runtime::{PrimaryPresentationMetricsResource, Res, ResMut};
use crate::{SceneCatalog, SceneOverlayViewportState, SceneRuntimeState};
use anyhow::Result;

pub(crate) fn scene_setup_system(
    presentation: Res<PrimaryPresentationMetricsResource>,
    scene_catalog: Res<SceneCatalog>,
    mut scene_templates: ResMut<SceneTemplateFlowResource>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut viewport: ResMut<SceneOverlayViewportState>,
) -> Result<()> {
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(SceneManager::new(&presentation)?);
    }
    if let Some(manager) = scene_resource.manager.as_mut() {
        sync_overlay_viewport(manager, &presentation);
        scene_templates.ensure_loaded_from_catalog(&scene_catalog)?;
        if scene_templates.has_scenes() {
            manager.set_active_overlay_visible(true);
            manager.overlay_runtime.ui.presentation_mode = UiPresentationMode::CenteredDemo;
            manager.overlay_runtime.ui.layout_dirty = true;
            manager.world.paused = scene_templates.active_scene_id() != Some("game_scene");
        }
        publish_scene_state(manager, &mut scene_state, &mut viewport);
    }
    Ok(())
}
