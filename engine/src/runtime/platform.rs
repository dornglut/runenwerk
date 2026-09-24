use crate::plugins::{
    ContactInput, InputContext, InputState, KeyboardInput, Point2, PointerButtonInput, ScrollInput,
};
use crate::runtime::window::{NativeWindowId, NativeWindowRecord};

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

pub fn apply_platform_input_event(input: &mut InputState, event: &PlatformEvent) {
    match event {
        PlatformEvent::KeyboardInput {
            context,
            input: key,
        } => input.handle_normalized_keyboard(*context, key),
        PlatformEvent::TextInput { text } => input.handle_text_input(text),
        PlatformEvent::MouseWheel {
            context,
            input: scroll,
        } => input.handle_scroll_input(*context, *scroll),
        PlatformEvent::CursorMoved { context, position } => {
            input.handle_cursor_position(*context, *position)
        }
        PlatformEvent::MouseInput {
            context,
            input: button,
        } => input.handle_pointer_button(*context, *button),
        PlatformEvent::Touch {
            context,
            input: contact,
        } => input.handle_contact_input(*context, contact),
        PlatformEvent::Resumed
        | PlatformEvent::CloseRequested
        | PlatformEvent::Focused { .. }
        | PlatformEvent::Resized { .. }
        | PlatformEvent::ScaleFactorChanged { .. }
        | PlatformEvent::RedrawRequested => {}
    }
}

pub fn apply_native_window_event(record: &mut NativeWindowRecord, event: &PlatformEvent) {
    match event {
        PlatformEvent::Resumed => record.request_redraw(),
        PlatformEvent::CloseRequested => {
            record.receive_close_intent();
            record.request_redraw();
        }
        PlatformEvent::Focused { focused } => {
            record.focused = *focused;
            record.request_redraw();
        }
        PlatformEvent::Resized { width, height } => {
            record.size_px = ((*width).max(1), (*height).max(1));
            record.request_redraw();
        }
        PlatformEvent::ScaleFactorChanged {
            scale_factor,
            width,
            height,
        } => {
            record.scale_factor = *scale_factor;
            record.size_px = ((*width).max(1), (*height).max(1));
            record.request_redraw();
        }
        PlatformEvent::RedrawRequested => record.redraw_requested = false,
        PlatformEvent::KeyboardInput { .. }
        | PlatformEvent::TextInput { .. }
        | PlatformEvent::MouseWheel { .. }
        | PlatformEvent::CursorMoved { .. }
        | PlatformEvent::MouseInput { .. }
        | PlatformEvent::Touch { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlatformEvent, PlatformWindowEvent, apply_native_window_event, apply_platform_input_event,
    };
    use crate::plugins::input::domain::action;
    use crate::plugins::{ActionState, InputState};
    use crate::plugins::{
        ContactInput, ContactPhase, CoordinateSpace, DigitalState, InputContext, InputSourceId,
        KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey, ObservationOrigin,
        PhysicalKeyIdentity, Point2, PointerButton, PointerButtonInput,
    };
    use crate::runtime::window::{NativeWindowId, WindowStateRegistryResource};

    fn test_context() -> InputContext {
        InputContext::new(InputSourceId::new(90), None)
    }

    #[test]
    fn native_resize_and_scale_events_update_one_record() {
        let mut registry = WindowStateRegistryResource::default();
        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0);
        let record = registry.record_mut(primary).unwrap();

        apply_native_window_event(
            record,
            &PlatformEvent::Resized {
                width: 1600,
                height: 900,
            },
        );
        apply_native_window_event(
            record,
            &PlatformEvent::ScaleFactorChanged {
                scale_factor: 2.0,
                width: 1600,
                height: 900,
            },
        );

        assert_eq!(record.size_px, (1600, 900));
        assert_eq!(record.scale_factor, 2.0);
        assert!(record.redraw_requested);
    }

    #[test]
    fn normalized_input_events_update_input_state_without_window_state() {
        let mut input = InputState::new();
        let context = test_context();

        apply_platform_input_event(
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
        apply_platform_input_event(
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
        apply_platform_input_event(
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
    fn native_window_events_update_selected_record_only() {
        let mut registry = WindowStateRegistryResource::default();
        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0);
        let secondary = registry
            .request_window("Secondary", (640, 480))
            .native_window_id;

        apply_native_window_event(
            registry.record_mut(secondary).unwrap(),
            &PlatformEvent::Resized {
                width: 1920,
                height: 1080,
            },
        );

        assert_eq!(registry.record(primary).unwrap().size_px, (1280, 720));
        assert_eq!(registry.record(secondary).unwrap().size_px, (1920, 1080));
    }

    #[test]
    fn close_and_focus_events_remain_pending_for_product_policy() {
        let mut registry = WindowStateRegistryResource::default();
        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0);
        let record = registry.record_mut(primary).unwrap();

        apply_native_window_event(record, &PlatformEvent::Focused { focused: false });
        apply_native_window_event(record, &PlatformEvent::CloseRequested);

        assert!(!record.focused);
        assert!(record.close_intent_pending);
        assert!(!record.close_requested);
        assert_eq!(
            record.lifecycle_state,
            crate::runtime::window::NativeWindowLifecycleState::CloseIntentPending
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
