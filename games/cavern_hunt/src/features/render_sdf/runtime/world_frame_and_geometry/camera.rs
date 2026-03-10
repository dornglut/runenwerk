use super::*;

// Owner: Cavern Hunt SDF Renderer - Camera and HUD Projection
pub(crate) fn update_camera_and_hud_system(
    world: WorldRef,
    input: Res<InputState>,
    _time: Res<Time>,
    mut camera: ResMut<CavernCameraState>,
) -> Result<()> {
    let local_player_ref = world.resource::<LocalPlayerRef>()?;
    let local_player = local_player_ref.entity.and_then(|entity| {
        world.get::<Transform2>(entity).copied().map(|transform| {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            (transform, health)
        })
    });
    let Some((transform, _health)) = local_player else {
        return Ok(());
    };

    let zoom_delta = input.scroll_delta * 1.5;
    if zoom_delta.abs() > f32::EPSILON {
        camera.distance =
            (camera.distance - zoom_delta).clamp(camera.distance_min, camera.distance_max);
    }

    camera.target = [transform.x, 1.55, transform.y];

    Ok(())
}
