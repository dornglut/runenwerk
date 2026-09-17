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
    let before = *camera;
    let mouse_delta = if input.mouse_delta != (0.0, 0.0) {
        input.mouse_delta
    } else {
        input
            .mouse_motion_samples()
            .last()
            .map(|sample| sample.delta)
            .unwrap_or((0.0, 0.0))
    };
    apply_render_lab_input(
        &mut camera,
        input.left_mouse_down(),
        input.middle_mouse_down(),
        mouse_delta,
        input.scroll_delta,
    );
    if before != *camera && std::env::var("GROTTO_RENDER_CAMERA_LOG").is_ok() {
        eprintln!(
            "runenwerk_render_lab_camera yaw={:.4} pitch={:.4} distance={:.4} pan=({:.4},{:.4})",
            camera.yaw_radians, camera.pitch_radians, camera.distance, camera.pan[0], camera.pan[1]
        );
    }
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
    fn left_drag_orbits_without_pan_or_zoom() {
        let mut camera = RenderLabCamera::default();
        apply_render_lab_input(&mut camera, true, false, (10.0, -5.0), 0.0);
        assert_eq!(camera.yaw_radians, 0.1);
        assert_eq!(camera.pitch_radians, 0.05);
        assert_eq!(camera.pan, [0.0, 0.0]);
        assert_eq!(camera.distance, 3.0);
    }

    #[test]
    fn middle_drag_pans_without_orbit_or_zoom() {
        let mut camera = RenderLabCamera::default();
        apply_render_lab_input(&mut camera, false, true, (10.0, -5.0), 0.0);
        assert_eq!(camera.yaw_radians, 0.0);
        assert_eq!(camera.pitch_radians, 0.0);
        assert_eq!(camera.pan, [0.1, 0.05]);
        assert_eq!(camera.distance, 3.0);
    }

    #[test]
    fn button_combinations_apply_both_declared_camera_intents() {
        let mut camera = RenderLabCamera::default();
        apply_render_lab_input(&mut camera, true, true, (10.0, -5.0), 0.0);
        assert_eq!(camera.yaw_radians, 0.1);
        assert_eq!(camera.pitch_radians, 0.05);
        assert_eq!(camera.pan, [0.1, 0.05]);
        assert_eq!(camera.distance, 3.0);
    }

    #[test]
    fn no_drag_ignores_relative_motion_but_scroll_zoom_is_bounded() {
        let mut camera = RenderLabCamera::default();
        apply_render_lab_input(&mut camera, false, false, (10.0, -5.0), 100.0);
        assert_eq!(camera.yaw_radians, 0.0);
        assert_eq!(camera.pitch_radians, 0.0);
        assert_eq!(camera.pan, [0.0, 0.0]);
        assert_eq!(camera.distance, 1.0);

        apply_render_lab_input(&mut camera, false, false, (0.0, 0.0), -100.0);
        assert_eq!(camera.distance, 8.0);
    }

    #[test]
    fn frame_local_input_is_consumed_once_before_clear() {
        let mut input = InputState::new();
        input.handle_mouse_motion(10.0, -5.0);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        let mut camera = RenderLabCamera::default();
        apply_render_lab_input(
            &mut camera,
            input.left_mouse_down(),
            input.middle_mouse_down(),
            input.mouse_delta,
            input.scroll_delta,
        );
        let consumed = camera;
        input.clear_frame();
        apply_render_lab_input(
            &mut camera,
            input.left_mouse_down(),
            input.middle_mouse_down(),
            input.mouse_delta,
            input.scroll_delta,
        );
        assert_eq!(camera, consumed);
    }

    #[test]
    fn cursor_motion_fallback_changes_semantic_request_when_raw_delta_is_absent() {
        let mut input = InputState::new();
        input.handle_cursor_moved(100.0, 100.0);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(120.0, 90.0);

        let mut camera = RenderLabCamera::default();
        let mouse_delta = input
            .mouse_motion_samples()
            .last()
            .map(|sample| sample.delta)
            .unwrap_or((0.0, 0.0));
        apply_render_lab_input(
            &mut camera,
            input.left_mouse_down(),
            input.middle_mouse_down(),
            mouse_delta,
            input.scroll_delta,
        );

        assert_eq!(camera.yaw_radians, 0.2);
        assert_eq!(camera.pitch_radians, 0.1);
        assert_ne!(
            camera.observation_to_scene(),
            RenderAffineTransform3::identity()
        );
    }
}
