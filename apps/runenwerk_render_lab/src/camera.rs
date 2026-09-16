use super::*;

#[derive(Debug, Clone, Copy, PartialEq, runen_ecs::Resource)]
pub(super) struct RenderLabCamera {
    pub(super) yaw_radians: f64,
    pub(super) pitch_radians: f64,
    pub(super) distance: f64,
    pub(super) pan: [f64; 2],
}

impl Default for RenderLabCamera {
    fn default() -> Self {
        Self {
            yaw_radians: 0.0,
            pitch_radians: 0.0,
            distance: 3.0,
            pan: [0.0, 0.0],
        }
    }
}

impl RenderLabCamera {
    pub(super) fn observation_to_scene(self) -> RenderAffineTransform3 {
        let (sin_yaw, cos_yaw) = self.yaw_radians.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch_radians.sin_cos();
        let right = [cos_yaw, 0.0, sin_yaw];
        let up = [-sin_yaw * sin_pitch, cos_pitch, cos_yaw * sin_pitch];
        let backward = [-sin_yaw * cos_pitch, -sin_pitch, cos_yaw * cos_pitch];
        let view = scale(backward, -1.0);
        let target = [0.0, 0.0, -3.0];
        let origin = add(
            sub(target, scale(view, self.distance)),
            add(scale(right, self.pan[0]), scale(up, self.pan[1])),
        );
        RenderAffineTransform3::from_row_major_3x4([
            right[0],
            up[0],
            backward[0],
            origin[0],
            right[1],
            up[1],
            backward[1],
            origin[1],
            right[2],
            up[2],
            backward[2],
            origin[2],
        ])
        .expect("Render Lab camera basis is finite and invertible")
    }
}

pub(super) fn update_render_lab_camera_system(
    input: Res<InputState>,
    mut camera: ResMut<RenderLabCamera>,
) {
    apply_render_lab_input(
        &mut camera,
        input.left_mouse_down(),
        input.middle_mouse_down(),
        input.mouse_delta,
        input.scroll_delta,
    );
}

fn apply_render_lab_input(
    camera: &mut RenderLabCamera,
    left_mouse_down: bool,
    middle_mouse_down: bool,
    (delta_x, delta_y): (f32, f32),
    scroll_delta: f32,
) {
    if left_mouse_down {
        camera.yaw_radians += f64::from(delta_x) * 0.01;
        camera.pitch_radians =
            (camera.pitch_radians - f64::from(delta_y) * 0.01).clamp(-1.45, 1.45);
    }
    if middle_mouse_down {
        camera.pan[0] += f64::from(delta_x) * 0.01;
        camera.pan[1] -= f64::from(delta_y) * 0.01;
    }
    if scroll_delta != 0.0 {
        camera.distance =
            (camera.distance * (-f64::from(scroll_delta) * 0.1).exp()).clamp(1.0, 8.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_pointer_projection_drives_orbit_pan_and_bounded_zoom() {
        let mut camera = RenderLabCamera::default();
        apply_render_lab_input(&mut camera, true, true, (10.0, -5.0), 100.0);
        assert_eq!(camera.yaw_radians, 0.1);
        assert_eq!(camera.pitch_radians, 0.05);
        assert_eq!(camera.pan, [0.1, 0.05]);
        assert_eq!(camera.distance, 1.0);
    }
}
