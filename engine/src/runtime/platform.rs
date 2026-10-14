use crate::plugins::InputState;
use crate::plugins::ui::NativeInputRecipient;
use crate::runtime::window::{NativeWindowId, NativeWindowRecord};
use runen_input::{
    ContactInput, ContinuityLoss, InputContext, KeyboardInput, Point2, PointerButtonInput,
    ScrollInput,
};

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    Resumed,
    CloseRequested,
    Focused {
        focused: bool,
    },
    InputContinuityLost {
        context: InputContext,
        loss: ContinuityLoss,
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
    /// Frozen original host-selected semantic recipient, never a UI-consumed flag.
    input_recipient: Option<NativeInputRecipient>,
}

#[derive(Debug, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct PlatformWindowEventQueueResource {
    events: Vec<PlatformWindowEvent>,
    pending_text_bytes: usize,
    event_limit: usize,
    text_byte_limit: usize,
}

impl Default for PlatformWindowEventQueueResource {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            pending_text_bytes: 0,
            event_limit: 4096,
            text_byte_limit: 1024 * 1024,
        }
    }
}

/// Native journal admission is fail-stop: a rejected callback group changes
/// neither the journal nor physical state. Releases are never discarded to fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlatformEventAdmissionFailure {
    #[error("native frame journal event capacity exceeded (limit {limit}, requested {requested})")]
    EventCapacity { limit: usize, requested: usize },
    #[error("native frame journal text capacity exceeded (limit {limit}, requested {requested})")]
    TextCapacity { limit: usize, requested: usize },
    #[error("native frame journal allocation failed")]
    AllocationFailed,
}

impl PlatformWindowEventQueueResource {
    /// Admits one complete occurrence. Native compound occurrences use publish_group.
    pub fn publish(
        &mut self,
        event: PlatformWindowEvent,
    ) -> Result<(), PlatformEventAdmissionFailure> {
        self.publish_group([event])
    }

    /// Reserves the entire group before committing any member. A callback's
    /// keyboard+text or focus+continuity group is never partially recorded.
    pub fn publish_group<const N: usize>(
        &mut self,
        events: [PlatformWindowEvent; N],
    ) -> Result<(), PlatformEventAdmissionFailure> {
        let additional_text = events.iter().fold(0_usize, |total, event| {
            total.saturating_add(text_bytes(&event.event))
        });
        let text_bytes = self.reserve_group(N, additional_text)?;
        self.events.extend(events);
        self.pending_text_bytes = text_bytes;
        Ok(())
    }

    /// Check and reserve before the runner copies a compound callback payload.
    pub(crate) fn reserve_native_group(
        &mut self,
        events: &[PlatformEvent],
    ) -> Result<(), PlatformEventAdmissionFailure> {
        let additional_text = events.iter().fold(0_usize, |total, event| {
            total.saturating_add(text_bytes(event))
        });
        self.reserve_group(events.len(), additional_text)
            .map(|_| ())
    }

    fn reserve_group(
        &mut self,
        event_count: usize,
        additional_text: usize,
    ) -> Result<usize, PlatformEventAdmissionFailure> {
        let requested = self.events.len().saturating_add(event_count);
        if requested > self.event_limit {
            return Err(PlatformEventAdmissionFailure::EventCapacity {
                limit: self.event_limit,
                requested,
            });
        }
        let text_bytes = self.pending_text_bytes.saturating_add(additional_text);
        if text_bytes > self.text_byte_limit {
            return Err(PlatformEventAdmissionFailure::TextCapacity {
                limit: self.text_byte_limit,
                requested: text_bytes,
            });
        }
        self.events
            .try_reserve(event_count)
            .map_err(|_| PlatformEventAdmissionFailure::AllocationFailed)?;
        Ok(text_bytes)
    }

    pub fn events(&self) -> &[PlatformWindowEvent] {
        &self.events
    }

    pub fn drain(&mut self) -> Vec<PlatformWindowEvent> {
        self.pending_text_bytes = 0;
        std::mem::take(&mut self.events)
    }

    /// The canonical FrameEnd occurrence retires any events not drained by Editor.
    pub(crate) fn retire_frame(&mut self) {
        self.events.clear();
        self.pending_text_bytes = 0;
    }
}

fn text_bytes(event: &PlatformEvent) -> usize {
    match event {
        PlatformEvent::TextInput { text } => text.len(),
        PlatformEvent::KeyboardInput { input, .. } => {
            let physical = match &input.physical_key {
                runen_input::PhysicalKeyIdentity::Code(code) => code.len(),
                runen_input::PhysicalKeyIdentity::Native(_) => 0,
            };
            let logical = match &input.logical_key {
                runen_input::LogicalKey::Named(text)
                | runen_input::LogicalKey::Character(text)
                | runen_input::LogicalKey::Native(runen_input::NativeLogicalKey::Web(text)) => {
                    text.len()
                }
                _ => 0,
            };
            physical.saturating_add(logical)
        }
        _ => 0,
    }
}

impl PlatformWindowEvent {
    /// Existing explicit legacy producers retain their established Editor reader.
    pub fn new(native_window_id: NativeWindowId, event: PlatformEvent) -> Self {
        Self {
            native_window_id,
            event,
            input_recipient: Some(NativeInputRecipient::LegacyNativeConsumer),
        }
    }

    /// Only the native input authority may attach its original recipient.
    /// Later target retirement cannot reinterpret an earlier journal entry.
    pub(crate) fn with_native_input_recipient(
        mut self,
        recipient: Option<NativeInputRecipient>,
    ) -> Self {
        self.input_recipient = recipient;
        self
    }

    /// Window lifecycle is still visible to legacy readers. Semantic input
    /// is delivered to Editor only when the original native target was legacy.
    #[must_use]
    pub fn authorized_for_legacy_editor(&self) -> bool {
        if matches!(
            &self.event,
            PlatformEvent::KeyboardInput { .. }
                | PlatformEvent::TextInput { .. }
                | PlatformEvent::MouseWheel { .. }
                | PlatformEvent::CursorMoved { .. }
                | PlatformEvent::MouseInput { .. }
                | PlatformEvent::Touch { .. }
        ) {
            self.input_recipient == Some(NativeInputRecipient::LegacyNativeConsumer)
        } else {
            true
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
        PlatformEvent::InputContinuityLost { context, loss } => {
            input.handle_continuity_loss(*context, *loss)
        }
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
        | PlatformEvent::Touch { .. }
        | PlatformEvent::InputContinuityLost { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlatformEvent, PlatformWindowEvent, PlatformWindowEventQueueResource,
        apply_native_window_event, apply_platform_input_event,
    };
    use crate::plugins::input::domain::action;
    use crate::plugins::{ActionState, InputState};
    use crate::runtime::window::{NativeWindowId, WindowStateRegistryResource};
    use runen_input::{
        ContactId, ContactInput, ContactPhase, CoordinateSpace, DigitalState, InputContext,
        InputSourceId, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey, ObservationOrigin,
        PhysicalKeyIdentity, Point2, PointerButton, PointerButtonInput,
    };

    #[test]
    fn native_recipient_journal_cannot_dispatch_unassigned_input_to_editor() {
        let id = NativeWindowId::primary();
        let keyboard = PlatformEvent::KeyboardInput {
            context: test_context(),
            input: KeyboardInput {
                physical_key: PhysicalKeyIdentity::code("KeyA"),
                logical_key: LogicalKey::Character(String::from("a")),
                location: KeyLocation::Standard,
                state: DigitalState::Pressed,
                repeat: false,
                origin: ObservationOrigin::SourceReport,
            },
        };
        let legacy = PlatformWindowEvent::new(id, keyboard.clone());
        assert!(legacy.authorized_for_legacy_editor());
        let unassigned = PlatformWindowEvent::new(id, keyboard).with_native_input_recipient(None);
        assert!(!unassigned.authorized_for_legacy_editor());
        let lifecycle = PlatformWindowEvent::new(id, PlatformEvent::Focused { focused: false })
            .with_native_input_recipient(None);
        assert!(lifecycle.authorized_for_legacy_editor());
    }

    fn test_context() -> InputContext {
        InputContext::new(InputSourceId::new(90), None)
    }

    #[test]
    fn native_resize_and_scale_events_update_one_record() {
        let mut registry = WindowStateRegistryResource::default();
        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0, true);
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
                    contact: ContactId::new(7),
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
        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0, true);
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
        let primary = registry.register_primary_window("Runtime", (1280, 720), 1.0, true);
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

        queue
            .publish(PlatformWindowEvent::new(
                secondary,
                PlatformEvent::Focused { focused: true },
            ))
            .unwrap();
        queue
            .publish(PlatformWindowEvent::new(
                primary,
                PlatformEvent::CloseRequested,
            ))
            .unwrap();

        let events = queue.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].native_window_id, secondary);
        assert_eq!(events[1].native_window_id, primary);
        assert!(queue.events().is_empty());
    }

    #[test]
    fn journal_admits_whole_groups_or_nothing_and_never_drops_release_to_fit() {
        let mut queue = PlatformWindowEventQueueResource {
            event_limit: 1,
            ..Default::default()
        };
        let window = NativeWindowId::primary();
        assert!(
            queue
                .publish_group([
                    PlatformWindowEvent::new(window, PlatformEvent::Focused { focused: false }),
                    PlatformWindowEvent::new(
                        window,
                        PlatformEvent::InputContinuityLost {
                            context: test_context(),
                            loss: runen_input::ContinuityLoss::Source,
                        }
                    ),
                ])
                .is_err()
        );
        assert!(queue.events().is_empty());
        queue
            .publish(PlatformWindowEvent::new(
                window,
                PlatformEvent::CloseRequested,
            ))
            .unwrap();
        assert!(
            queue
                .publish(PlatformWindowEvent::new(
                    window,
                    PlatformEvent::Focused { focused: false }
                ))
                .is_err()
        );
        assert_eq!(queue.events().len(), 1);
        queue.retire_frame();
        assert!(queue.events().is_empty());
    }

    #[test]
    fn journal_text_bound_and_editor_drain_reinsertion_preserve_order() {
        let mut queue = PlatformWindowEventQueueResource {
            text_byte_limit: 3,
            ..Default::default()
        };
        let first = NativeWindowId::primary();
        let second = NativeWindowId::try_from_raw(2).unwrap();
        queue
            .publish(PlatformWindowEvent::new(
                first,
                PlatformEvent::TextInput { text: "abc".into() },
            ))
            .unwrap();
        assert!(
            queue
                .publish(PlatformWindowEvent::new(
                    second,
                    PlatformEvent::TextInput { text: "d".into() }
                ))
                .is_err()
        );
        let saved = queue.drain();
        for event in saved {
            queue.publish(event).unwrap();
        }
        assert_eq!(queue.events()[0].native_window_id, first);
        queue.retire_frame();
        queue
            .publish(PlatformWindowEvent::new(
                second,
                PlatformEvent::TextInput { text: "xyz".into() },
            ))
            .unwrap();
        queue.retire_frame();
        let oversized_key = PlatformWindowEvent::new(
            first,
            PlatformEvent::KeyboardInput {
                context: test_context(),
                input: KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyA"),
                    logical_key: LogicalKey::Character("a".into()),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
            },
        );
        assert_eq!(
            queue.publish(oversized_key),
            Err(super::PlatformEventAdmissionFailure::TextCapacity {
                limit: 3,
                requested: 5,
            })
        );
        assert!(
            queue.events().is_empty(),
            "keyboard text obeys the same payload bound"
        );
    }
}
