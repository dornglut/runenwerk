use crate::plugins::{InputState, TouchInputPhase};
use crate::runtime::window::WindowState;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    Resumed,
    CloseRequested,
    Resized {
        width: u32,
        height: u32,
    },
    ScaleFactorChanged {
        scale_factor: f64,
        width: u32,
        height: u32,
    },
    KeyboardInput {
        key: KeyCode,
        state: ElementState,
        text: Option<String>,
    },
    MouseWheel {
        delta: f32,
    },
    CursorMoved {
        x: f32,
        y: f32,
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
    MouseMotion {
        delta_x: f32,
        delta_y: f32,
    },
    Touch {
        phase: TouchInputPhase,
        id: u64,
        x: f32,
        y: f32,
        pressure: Option<f32>,
    },
    RedrawRequested,
}

pub fn apply_platform_event(
    window: &mut WindowState,
    input: &mut InputState,
    event: &PlatformEvent,
) {
    match event {
        PlatformEvent::Resumed => {
            window.redraw_requested = true;
        }
        PlatformEvent::CloseRequested => {
            window.request_close();
        }
        PlatformEvent::Resized { width, height } => {
            window.size_px = (*width, *height);
            window.request_redraw();
        }
        PlatformEvent::ScaleFactorChanged {
            scale_factor,
            width,
            height,
        } => {
            window.scale_factor = *scale_factor;
            window.size_px = (*width, *height);
            window.request_redraw();
        }
        PlatformEvent::KeyboardInput { key, state, text } => {
            input.handle_keyboard_input(*key, *state, text.as_deref());
        }
        PlatformEvent::MouseWheel { delta } => {
            input.handle_mouse_wheel_delta(*delta);
        }
        PlatformEvent::CursorMoved { x, y } => {
            input.handle_cursor_moved(*x, *y);
        }
        PlatformEvent::MouseInput { state, button } => {
            input.handle_mouse_input(*state, *button);
        }
        PlatformEvent::MouseMotion { delta_x, delta_y } => {
            input.handle_mouse_motion(*delta_x, *delta_y);
        }
        PlatformEvent::Touch {
            phase,
            id,
            x,
            y,
            pressure,
        } => {
            input.handle_touch_input(*phase, *id, *x, *y, *pressure);
        }
        PlatformEvent::RedrawRequested => {
            window.redraw_requested = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformEvent, apply_platform_event};
    use crate::plugins::InputState;
    use crate::runtime::window::WindowState;
    use winit::event::{ElementState, MouseButton};
    use winit::keyboard::KeyCode;

    #[test]
    fn resize_and_scale_events_update_window_state() {
        let mut window = WindowState::windowed("Runtime");
        let mut input = InputState::new();

        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::Resized {
                width: 1600,
                height: 900,
            },
        );
        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::ScaleFactorChanged {
                scale_factor: 2.0,
                width: 1600,
                height: 900,
            },
        );

        assert_eq!(window.size_px, (1600, 900));
        assert_eq!(window.scale_factor, 2.0);
        assert!(window.redraw_requested);
    }

    #[test]
    fn normalized_input_events_update_input_state() {
        let mut window = WindowState::headless("Runtime");
        let mut input = InputState::new();

        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::KeyboardInput {
                key: KeyCode::KeyD,
                state: ElementState::Pressed,
                text: None,
            },
        );
        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
        );
        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::MouseMotion {
                delta_x: 5.0,
                delta_y: -2.0,
            },
        );
        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::Touch {
                phase: super::TouchInputPhase::Started,
                id: 7,
                x: 10.0,
                y: 12.0,
                pressure: Some(0.6),
            },
        );

        assert!(input.world_move_right);
        assert!(input.left_mouse_pressed());
        assert_eq!(input.mouse_delta, (5.0, -2.0));
        assert_eq!(input.touch_samples().len(), 1);
        assert_eq!(input.touch_samples()[0].pressure, Some(0.6));
    }
}
