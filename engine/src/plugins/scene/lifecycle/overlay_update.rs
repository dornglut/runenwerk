use super::super::runtime::{
    SceneTemplateFlowResource, apply_overlay_messages, publish_scene_state,
    rebuild_overlay_draw_list,
};
use crate::plugins::SceneResource;
use crate::runtime::ResMut;
use crate::{GameplayRuntimeConfig, SceneRuntimeState, UiOverlayState};
use anyhow::Result;

pub(crate) fn scene_overlay_update_system(
    scene_templates: ResMut<SceneTemplateFlowResource>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };

    apply_overlay_messages(manager);
    rebuild_overlay_draw_list(manager, &scene_templates)?;
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}
