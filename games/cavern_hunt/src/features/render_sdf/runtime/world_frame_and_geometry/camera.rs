use super::*;

// Owner: Cavern Hunt SDF Renderer - Camera and HUD Projection
pub(crate) fn project_mouse_to_world(
    camera: &CavernCameraState,
    window: &WindowState,
    layout: &CavernLayout,
    cursor: (f32, f32),
) -> [f32; 2] {
    let size = (
        window.size_px.0.max(1) as f32,
        window.size_px.1.max(1) as f32,
    );
    let aspect = size.0 / size.1.max(1.0);
    let view_h = camera.distance * 0.55;
    let view_w = view_h * aspect;
    let ndc_x = (cursor.0 / size.0) * 2.0 - 1.0;
    let ndc_y = 1.0 - (cursor.1 / size.1) * 2.0;
    [
        (camera.target[0] + ndc_x * view_w).clamp(layout.world_bounds[0], layout.world_bounds[2]),
        (camera.target[2] - ndc_y * view_h).clamp(layout.world_bounds[1], layout.world_bounds[3]),
    ]
}

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
