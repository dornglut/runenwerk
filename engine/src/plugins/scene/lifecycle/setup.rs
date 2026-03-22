use super::super::runtime::{
    SceneTemplateFlowResource, publish_scene_state, sync_overlay_viewport,
    sync_world_scene_context_from_session,
};
use crate::plugins::ui::domain::UiPresentationMode;
use crate::plugins::{SceneManager, SceneResource};
use crate::runtime::{Res, ResMut, WindowState};
use crate::{
    GameplayRuntimeConfig, SceneCatalog, SceneRuntimeState, SessionRuntimeState, UiOverlayState,
};
use anyhow::Result;

pub(crate) fn scene_setup_system(
    window: Res<WindowState>,
    scene_catalog: Res<SceneCatalog>,
    mut scene_templates: ResMut<SceneTemplateFlowResource>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    session: Res<SessionRuntimeState>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(SceneManager::new(&window)?);
    }
    if let Some(manager) = scene_resource.manager.as_mut() {
        sync_overlay_viewport(manager, &window);
        scene_templates.ensure_loaded_from_catalog(&scene_catalog)?;
        if scene_templates.has_scenes() {
            manager.set_active_overlay_visible(true);
            manager.overlay_runtime.ui.presentation_mode = UiPresentationMode::CenteredDemo;
            manager.overlay_runtime.ui.layout_dirty = true;
            manager.world.paused = scene_templates.active_scene_id() != Some("game_scene");
        }
        sync_world_scene_context_from_session(manager, &session);
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    }
    Ok(())
}
