use super::super::runtime::{
    publish_scene_state, sync_overlay_viewport, sync_world_scene_context_from_session,
};
use crate::plugins::{SceneManager, SceneResource};
use crate::runtime::{Res, ResMut, WindowState};
use crate::{GameplayRuntimeConfig, SceneRuntimeState, SessionRuntimeState, UiOverlayState};
use anyhow::Result;

pub(crate) fn scene_setup_system(
    window: Res<WindowState>,
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
        sync_world_scene_context_from_session(manager, &session);
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    }
    Ok(())
}
