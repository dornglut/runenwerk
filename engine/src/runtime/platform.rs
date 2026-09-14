use crate::plugins::{
    ContactInput, InputContext, InputState, KeyboardInput, Point2, PointerButtonInput, ScrollInput,
};
use crate::runtime::window::{NativeWindowId, WindowState, WindowStateRegistryResource};

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    Resumed,
    CloseRequested,
    Focused {
        focused: bool,
    },
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
        context: InputContext,
        input: KeyboardInput,
    },
    TextInput {
        text: String,
    },
    MouseWheel {
        context: InputContext,
        input: ScrollInput,
    },
    CursorMoved {
        context: InputContext,
        position: Point2,
    },
    MouseInput {
        context: InputContext,
        input: PointerButtonInput,
    },
    Touch {
        context: InputContext,
        input: ContactInput,
    },
    RedrawRequested,
}

#[derive(Debug, Clone)]
pub struct PlatformWindowEvent {
    pub native_window_id: NativeWindowId,
    pub event: PlatformEvent,
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct PlatformWindowEventQueueResource {
    events: Vec<PlatformWindowEvent>,
}

impl PlatformWindowEventQueueResource {
    pub fn publish(&mut self, event: PlatformWindowEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[PlatformWindowEvent] {
        &self.events
    }

    pub fn drain(&mut self) -> Vec<PlatformWindowEvent> {
        std::mem::take(&mut self.events)
    }
}

impl PlatformWindowEvent {
    pub fn new(native_window_id: NativeWindowId, event: PlatformEvent) -> Self {
        Self {
            native_window_id,
            event,
        }
    }
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
            window.receive_close_intent();
            window.request_redraw();
        }
        PlatformEvent::Focused { focused } => {
            window.focused = *focused;
            window.request_redraw();
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
        PlatformEvent::KeyboardInput {
            context,
            input: key,
        } => {
            input.handle_normalized_keyboard(*context, key);
        }
        PlatformEvent::TextInput { text } => {
            input.handle_text_input(text);
        }
        PlatformEvent::MouseWheel {
            context,
            input: scroll,
        } => {
            input.handle_scroll_input(*context, *scroll);
        }
        PlatformEvent::CursorMoved { context, position } => {
            input.handle_cursor_position(*context, *position);
        }
        PlatformEvent::MouseInput {
            context,
            input: button,
        } => {
            input.handle_pointer_button(*context, *button);
        }
        PlatformEvent::Touch {
            context,
            input: contact,
        } => {
            input.handle_contact_input(*context, contact);
        }
        PlatformEvent::RedrawRequested => {
            window.redraw_requested = false;
        }
    }
}

pub fn apply_platform_window_event(
    registry: &mut WindowStateRegistryResource,
    legacy_primary_window: &mut WindowState,
    input: &mut InputState,
    event: &PlatformWindowEvent,
) {
    if registry.primary_window_id().is_none() {
        registry.ensure_primary_from_legacy(legacy_primary_window);
    }

    let is_primary = registry.primary_window_id() == Some(event.native_window_id);
    if let Some(record) = registry.record_mut(event.native_window_id) {
        match &event.event {
            PlatformEvent::Resumed => {
                record.redraw_requested = true;
            }
            PlatformEvent::CloseRequested => {
                record.receive_close_intent();
                record.request_redraw();
            }
            PlatformEvent::Focused { focused } => {
                record.focused = *focused;
                record.request_redraw();
            }
            PlatformEvent::Resized { width, height } => {
                record.size_px = (*width, *height);
                record.request_redraw();
            }
            PlatformEvent::ScaleFactorChanged {
                scale_factor,
                width,
                height,
            } => {
                record.scale_factor = *scale_factor;
                record.size_px = (*width, *height);
                record.request_redraw();
            }
            PlatformEvent::RedrawRequested => {
                record.redraw_requested = false;
            }
            PlatformEvent::KeyboardInput { .. }
            | PlatformEvent::TextInput { .. }
            | PlatformEvent::MouseWheel { .. }
            | PlatformEvent::CursorMoved { .. }
            | PlatformEvent::MouseInput { .. }
            | PlatformEvent::Touch { .. } => {}
        }
        if is_primary {
            record.copy_to_legacy(legacy_primary_window);
        }
    }

    match &event.event {
        PlatformEvent::KeyboardInput { .. }
        | PlatformEvent::TextInput { .. }
        | PlatformEvent::MouseWheel { .. }
        | PlatformEvent::CursorMoved { .. }
        | PlatformEvent::MouseInput { .. }
        | PlatformEvent::Touch { .. } => {
            let mut shadow_window = WindowState::headless("");
            apply_platform_event(&mut shadow_window, input, &event.event);
        }
        PlatformEvent::Resumed
        | PlatformEvent::CloseRequested
        | PlatformEvent::Focused { .. }
        | PlatformEvent::Resized { .. }
        | PlatformEvent::ScaleFactorChanged { .. }
        | PlatformEvent::RedrawRequested => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlatformEvent, PlatformWindowEvent, apply_platform_event, apply_platform_window_event,
    };
    use crate::plugins::input::domain::action;
    use crate::plugins::{ActionState, InputState};
    use crate::plugins::{
        ContactInput, ContactPhase, CoordinateSpace, DigitalState, InputContext, InputSourceId,
        KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey, ObservationOrigin,
        PhysicalKeyIdentity, Point2, PointerButton, PointerButtonInput,
    };
    use crate::runtime::window::{NativeWindowId, WindowState, WindowStateRegistryResource};

    fn test_context() -> InputContext {
        InputContext::new(InputSourceId::new(90), None)
    }

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
        let context = test_context();

        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::KeyboardInput {
                context,
                input: KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyD"),
                    logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
            },
        );
        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::MouseInput {
                context,
                input: PointerButtonInput {
                    button: PointerButton::Left,
                    state: DigitalState::Pressed,
                },
            },
        );
        input.handle_relative_motion(context, 5.0, -2.0);
        apply_platform_event(
            &mut window,
            &mut input,
            &PlatformEvent::Touch {
                context,
                input: ContactInput {
                    id: 7,
                    phase: ContactPhase::Begin,
                    position: Point2::new(10.0, 12.0, CoordinateSpace::WindowPhysicalPixels),
                    pressure: None,
                    altitude_angle_radians: None,
                },
            },
        );

        let mut actions = ActionState::new();
        actions.project(&input);
        assert!(actions.action_down(action::WORLD_MOVE_RIGHT));
        assert!(input.left_mouse_pressed());
        assert_eq!(input.mouse_delta, (5.0, -2.0));
        assert_eq!(input.touch_samples().len(), 1);
    }

    #[test]
    fn window_scoped_resize_updates_selected_native_window_only() {
        let mut legacy = WindowState::windowed("Runtime");
        let mut registry = WindowStateRegistryResource::from_legacy(&legacy);
        let secondary = registry
            .request_window("Secondary", (640, 480))
            .native_window_id;
        let mut input = InputState::new();

        apply_platform_window_event(
            &mut registry,
            &mut legacy,
            &mut input,
            &PlatformWindowEvent::new(
                secondary,
                PlatformEvent::Resized {
                    width: 1920,
                    height: 1080,
                },
            ),
        );

        assert_eq!(legacy.size_px, (1280, 720));
        assert_eq!(
            registry.record(secondary).map(|record| record.size_px),
            Some((1920, 1080))
        );
        assert_eq!(
            registry
                .record(NativeWindowId::primary())
                .map(|record| record.size_px),
            Some((1280, 720))
        );
    }

    #[test]
    fn primary_window_scoped_resize_preserves_legacy_compatibility() {
        let mut legacy = WindowState::windowed("Runtime");
        let mut registry = WindowStateRegistryResource::from_legacy(&legacy);
        let mut input = InputState::new();

        apply_platform_window_event(
            &mut registry,
            &mut legacy,
            &mut input,
            &PlatformWindowEvent::new(
                NativeWindowId::primary(),
                PlatformEvent::Resized {
                    width: 1024,
                    height: 768,
                },
            ),
        );

        assert_eq!(legacy.size_px, (1024, 768));
        assert_eq!(
            registry
                .record(NativeWindowId::primary())
                .map(|record| record.size_px),
            Some((1024, 768))
        );
    }

    #[test]
    fn close_and_focus_events_remain_pending_for_app_policy() {
        let mut legacy = WindowState::windowed("Runtime");
        let mut registry = WindowStateRegistryResource::from_legacy(&legacy);
        let mut input = InputState::new();

        apply_platform_window_event(
            &mut registry,
            &mut legacy,
            &mut input,
            &PlatformWindowEvent::new(
                NativeWindowId::primary(),
                PlatformEvent::Focused { focused: false },
            ),
        );
        apply_platform_window_event(
            &mut registry,
            &mut legacy,
            &mut input,
            &PlatformWindowEvent::new(NativeWindowId::primary(), PlatformEvent::CloseRequested),
        );

        assert!(!legacy.focused);
        assert!(legacy.close_intent_pending);
        assert!(!legacy.close_requested);
        assert_eq!(
            registry
                .record(NativeWindowId::primary())
                .map(|record| record.lifecycle_state),
            Some(crate::runtime::window::NativeWindowLifecycleState::CloseIntentPending)
        );
    }

    #[test]
    fn platform_window_event_queue_preserves_window_identity_and_order() {
        let mut queue = super::PlatformWindowEventQueueResource::default();
        let primary = NativeWindowId::primary();
        let secondary = NativeWindowId::try_from_raw(2).expect("secondary id");

        queue.publish(PlatformWindowEvent::new(
            secondary,
            PlatformEvent::Focused { focused: true },
        ));
        queue.publish(PlatformWindowEvent::new(
            primary,
            PlatformEvent::CloseRequested,
        ));

        let events = queue.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].native_window_id, secondary);
        assert_eq!(events[1].native_window_id, primary);
        assert!(queue.events().is_empty());
    }
}
