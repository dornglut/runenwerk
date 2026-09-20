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
    apply_render_lab_input(&mut camera, &input);
    if before != *camera && std::env::var("GROTTO_RENDER_CAMERA_LOG").is_ok() {
        eprintln!(
            "runenwerk_render_lab_camera yaw={:.4} pitch={:.4} distance={:.4} pan=({:.4},{:.4})",
            camera.yaw_radians, camera.pitch_radians, camera.distance, camera.pan[0], camera.pan[1]
        );
    }
}

fn apply_render_lab_input(camera: &mut RenderLabCamera, input: &InputState) {
    if input.mouse_delta != (0.0, 0.0) {
        // Raw relative motion and absolute cursor samples are separate observations. Prefer raw
        // motion when available, and never add the absolute fallback a second time.
        apply_camera_motion(
            camera,
            input.left_mouse_down(),
            input.middle_mouse_down(),
            input.mouse_delta,
        );
    } else {
        replay_cursor_motion(camera, input);
    }

    let scroll_delta = input.scroll_delta;
    if scroll_delta != 0.0 {
        camera.distance =
            (camera.distance * (-f64::from(scroll_delta) * 0.1).exp()).clamp(1.0, 8.0);
    }
}

fn replay_cursor_motion(camera: &mut RenderLabCamera, input: &InputState) {
    let mut left_mouse_down = input.left_mouse_down();
    let mut middle_mouse_down = input.middle_mouse_down();

    // InputState exposes the final held state plus ordered transitions. Rewind those transitions
    // to recover the state at the start of this frame, then replay transitions at their recorded
    // cursor-sample boundary.
    for transition in input.mouse_button_transitions().iter().rev() {
        let was_down = transition.state == winit::event::ElementState::Released;
        if transition.button == winit::event::MouseButton::Left {
            left_mouse_down = was_down;
        } else if transition.button == winit::event::MouseButton::Middle {
            middle_mouse_down = was_down;
        }
    }

    let motions = input.mouse_motion_samples();
    let transitions = input.mouse_button_transitions();
    let mut next_transition = 0;
    for motion_index in 0..=motions.len() {
        while transitions
            .get(next_transition)
            .is_some_and(|transition| transition.motion_sample_index == motion_index)
        {
            let transition = transitions[next_transition];
            let is_down = transition.state == winit::event::ElementState::Pressed;
            if transition.button == winit::event::MouseButton::Left {
                left_mouse_down = is_down;
            } else if transition.button == winit::event::MouseButton::Middle {
                middle_mouse_down = is_down;
            }
            next_transition += 1;
        }

        if let Some(motion) = motions.get(motion_index) {
            apply_camera_motion(camera, left_mouse_down, middle_mouse_down, motion.delta);
        }
    }
}

fn apply_camera_motion(
    camera: &mut RenderLabCamera,
    left_mouse_down: bool,
    middle_mouse_down: bool,
    (delta_x, delta_y): (f32, f32),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(camera: &mut RenderLabCamera, input: &InputState) {
        apply_render_lab_input(camera, input);
    }

    fn baseline_cursor(input: &mut InputState) {
        input.handle_cursor_moved(0.0, 0.0);
    }

    #[test]
    fn left_drag_orbits_without_pan_or_zoom() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(10.0, -5.0);
        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.1);
        assert_eq!(camera.pitch_radians, 0.05);
        assert_eq!(camera.pan, [0.0, 0.0]);
        assert_eq!(camera.distance, 3.0);
    }

    #[test]
    fn middle_drag_pans_without_orbit_or_zoom() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Middle,
        );
        input.handle_cursor_moved(10.0, -5.0);
        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.0);
        assert_eq!(camera.pitch_radians, 0.0);
        assert_eq!(camera.pan, [0.1, 0.05]);
        assert_eq!(camera.distance, 3.0);
    }

    #[test]
    fn button_combinations_apply_both_declared_camera_intents() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Middle,
        );
        input.handle_cursor_moved(10.0, -5.0);
        input.handle_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(20.0, -10.0);
        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.1);
        assert_eq!(camera.pitch_radians, 0.05);
        assert_eq!(camera.pan, [0.2, 0.1]);
        assert_eq!(camera.distance, 3.0);
    }

    #[test]
    fn no_drag_ignores_relative_motion_but_scroll_zoom_is_bounded() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_motion(10.0, -5.0);
        input.handle_cursor_moved(10.0, -5.0);
        input.handle_mouse_wheel_delta(100.0);
        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.0);
        assert_eq!(camera.pitch_radians, 0.0);
        assert_eq!(camera.pan, [0.0, 0.0]);
        assert_eq!(camera.distance, 1.0);

        input.clear_frame();
        input.handle_mouse_wheel_delta(-100.0);
        apply(&mut camera, &input);
        assert_eq!(camera.distance, 8.0);
    }

    #[test]
    fn frame_local_input_is_consumed_once_before_clear() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(10.0, -5.0);
        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        let consumed = camera;
        input.clear_frame();
        apply(&mut camera, &input);
        assert_eq!(camera, consumed);
    }

    #[test]
    fn cursor_motion_samples_are_all_applied_in_order() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(10.0, -5.0);
        input.handle_cursor_moved(20.0, -10.0);
        input.handle_cursor_moved(30.0, -15.0);

        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);

        assert!((camera.yaw_radians - 0.3).abs() < f64::EPSILON * 2.0);
        assert!((camera.pitch_radians - 0.15).abs() < f64::EPSILON * 2.0);
        assert_ne!(
            camera.observation_to_scene(),
            RenderAffineTransform3::identity()
        );
    }

    #[test]
    fn motion_before_press_is_not_retroactively_classified_as_drag() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_cursor_moved(10.0, 0.0);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(15.0, 0.0);

        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.05);
    }

    #[test]
    fn press_before_motion_classifies_the_following_cursor_sample_as_drag() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(10.0, 5.0);

        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.1);
        assert_eq!(camera.pitch_radians, -0.05);
    }

    #[test]
    fn motion_before_release_is_drag_but_motion_after_release_is_not() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(10.0, 0.0);
        input.handle_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
        input.handle_cursor_moved(20.0, 0.0);

        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.1);
    }

    #[test]
    fn raw_relative_motion_takes_precedence_over_absolute_cursor_samples() {
        let mut input = InputState::new();
        baseline_cursor(&mut input);
        input.handle_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        input.handle_mouse_motion(4.0, 0.0);
        input.handle_cursor_moved(10.0, 0.0);

        let mut camera = RenderLabCamera::default();
        apply(&mut camera, &input);
        assert_eq!(camera.yaw_radians, 0.04);
    }
}
