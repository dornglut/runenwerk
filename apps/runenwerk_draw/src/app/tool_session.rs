//! App-owned drawing tool session intent boundary.

use drawing::CanvasCoordinate;
use ui_input::{
    Key, KeyState, KeyboardEvent, Modifiers, PointerDeviceId, PointerSourceKind, PointerToolKind,
};
use ui_math::UiPoint;

use crate::app::{DrawingToolInputEvent, DrawingToolRouteKind};

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingToolSession {
    next_session_id: u64,
    phase: DrawingToolSessionPhase,
}

impl Default for DrawingToolSession {
    fn default() -> Self {
        Self {
            next_session_id: 1,
            phase: DrawingToolSessionPhase::Idle,
        }
    }
}

impl DrawingToolSession {
    pub fn active_kind(&self) -> Option<DrawingToolSessionKind> {
        match self.phase {
            DrawingToolSessionPhase::Idle => None,
            DrawingToolSessionPhase::Active { kind, .. } => Some(kind),
        }
    }

    pub fn phase(&self) -> &DrawingToolSessionPhase {
        &self.phase
    }

    pub fn active_session_id(&self) -> Option<DrawingToolSessionId> {
        match self.phase {
            DrawingToolSessionPhase::Idle => None,
            DrawingToolSessionPhase::Active { id, .. } => Some(id),
        }
    }

    pub fn active_anchor(&self) -> Option<&DrawingToolSessionAnchor> {
        match &self.phase {
            DrawingToolSessionPhase::Idle => None,
            DrawingToolSessionPhase::Active { anchor, .. } => Some(anchor),
        }
    }

    pub fn handle_input(&mut self, input: DrawingToolInputEvent) -> DrawingToolSessionOutcome {
        match input.route_kind {
            DrawingToolRouteKind::BeginPreviewStroke => {
                self.begin_active_session(DrawingToolSessionKind::InkStroke, &input);
                DrawingToolSessionOutcome::handled(DrawingToolIntent::BeginPreviewStroke { input })
            }
            DrawingToolRouteKind::UpdatePreviewStroke => {
                if matches!(self.phase, DrawingToolSessionPhase::Idle) {
                    self.begin_active_session(DrawingToolSessionKind::InkStroke, &input);
                }
                DrawingToolSessionOutcome::handled(DrawingToolIntent::UpdatePreviewStroke { input })
            }
            DrawingToolRouteKind::EndPreviewStroke => {
                self.phase = DrawingToolSessionPhase::Idle;
                DrawingToolSessionOutcome::handled(DrawingToolIntent::FinishPreviewStroke { input })
            }
            DrawingToolRouteKind::Hover => {
                DrawingToolSessionOutcome::handled(DrawingToolIntent::Hover { input })
            }
            DrawingToolRouteKind::Scroll => {
                DrawingToolSessionOutcome::handled(DrawingToolIntent::Scroll { input })
            }
            DrawingToolRouteKind::Ignored => {
                DrawingToolSessionOutcome::ignored(DrawingToolIntent::Ignore { input })
            }
        }
    }

    pub fn handle_control_input(
        &mut self,
        input: DrawingToolControlInputEvent,
    ) -> DrawingToolSessionOutcome {
        match input.request {
            DrawingToolControlRequest::Observe => {
                DrawingToolSessionOutcome::ignored(DrawingToolIntent::ControlInputObserved {
                    input,
                })
            }
            DrawingToolControlRequest::Cancel => {
                let active_session_id = self.active_session_id();
                DrawingToolSessionOutcome::ignored(DrawingToolIntent::RequestCancel {
                    input,
                    active_session_id,
                })
            }
            DrawingToolControlRequest::RadialMenu => {
                let anchor = self.active_anchor().cloned();
                DrawingToolSessionOutcome::ignored(DrawingToolIntent::RequestRadialMenu {
                    input,
                    anchor,
                })
            }
        }
    }

    fn begin_active_session(
        &mut self,
        kind: DrawingToolSessionKind,
        input: &DrawingToolInputEvent,
    ) {
        let id = DrawingToolSessionId(self.next_session_id);
        self.next_session_id = self.next_session_id.saturating_add(1);
        self.phase = DrawingToolSessionPhase::Active {
            id,
            kind,
            anchor: DrawingToolSessionAnchor::from_input(input),
        };
    }
}

/// App-local interaction session id. This is not a domain drawing id and must
/// not participate in document, command, product, or cache identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DrawingToolSessionId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum DrawingToolSessionPhase {
    Idle,
    Active {
        id: DrawingToolSessionId,
        kind: DrawingToolSessionKind,
        anchor: DrawingToolSessionAnchor,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingToolSessionAnchor {
    pub device_id: Option<PointerDeviceId>,
    pub source_kind: PointerSourceKind,
    pub tool_kind: PointerToolKind,
    pub started_screen_position: UiPoint,
    pub started_canvas_position: Option<CanvasCoordinate>,
    pub timestamp_micros: Option<u64>,
}

impl DrawingToolSessionAnchor {
    fn from_input(input: &DrawingToolInputEvent) -> Self {
        Self {
            device_id: input.device_id,
            source_kind: input.source_kind,
            tool_kind: input.tool_kind,
            started_screen_position: input.screen_position,
            started_canvas_position: input.canvas_position,
            timestamp_micros: input.timestamp_micros,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingToolSessionKind {
    /// The current ink stroke gesture session. This is not a brush category or
    /// a domain stroke type.
    InkStroke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingToolControlInputEvent {
    pub source: DrawingToolControlInputSource,
    pub request: DrawingToolControlRequest,
}

impl DrawingToolControlInputEvent {
    pub fn from_keyboard(keyboard: &KeyboardEvent) -> Self {
        Self {
            source: DrawingToolControlInputSource::Keyboard {
                key: keyboard.key.clone(),
                state: keyboard.state,
                modifiers: keyboard.modifiers,
            },
            request: DrawingToolControlRequest::Observe,
        }
    }

    pub fn synthetic_cancel_request() -> Self {
        Self {
            source: DrawingToolControlInputSource::Synthetic,
            request: DrawingToolControlRequest::Cancel,
        }
    }

    pub fn synthetic_radial_menu_request() -> Self {
        Self {
            source: DrawingToolControlInputSource::Synthetic,
            request: DrawingToolControlRequest::RadialMenu,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawingToolControlInputSource {
    Keyboard {
        key: Key,
        state: KeyState,
        modifiers: Modifiers,
    },
    Synthetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingToolControlRequest {
    Observe,
    Cancel,
    RadialMenu,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawingToolIntent {
    BeginPreviewStroke {
        input: DrawingToolInputEvent,
    },
    UpdatePreviewStroke {
        input: DrawingToolInputEvent,
    },
    FinishPreviewStroke {
        input: DrawingToolInputEvent,
    },
    Hover {
        input: DrawingToolInputEvent,
    },
    Scroll {
        input: DrawingToolInputEvent,
    },
    Ignore {
        input: DrawingToolInputEvent,
    },
    ControlInputObserved {
        input: DrawingToolControlInputEvent,
    },
    RequestCancel {
        input: DrawingToolControlInputEvent,
        active_session_id: Option<DrawingToolSessionId>,
    },
    RequestRadialMenu {
        input: DrawingToolControlInputEvent,
        anchor: Option<DrawingToolSessionAnchor>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingToolSessionOutcome {
    pub intent: DrawingToolIntent,
    pub handled: bool,
}

impl DrawingToolSessionOutcome {
    fn handled(intent: DrawingToolIntent) -> Self {
        Self {
            intent,
            handled: true,
        }
    }

    fn ignored(intent: DrawingToolIntent) -> Self {
        Self {
            intent,
            handled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use drawing::CanvasCoordinate;
    use ui_input::{
        Key, KeyState, KeyboardEvent, Modifiers, PointerBarrelButtons, PointerDeviceId,
        PointerEventKind, PointerSourceKind, PointerToolKind,
    };
    use ui_math::UiPoint;

    use super::*;

    #[test]
    fn tool_session_records_active_session_id_phase_and_anchor() {
        let mut session = DrawingToolSession::default();
        assert_eq!(session.phase(), &DrawingToolSessionPhase::Idle);
        assert_eq!(session.active_kind(), None);
        assert_eq!(session.active_session_id(), None);
        assert_eq!(session.active_anchor(), None);

        let begin_input = input_for_route(
            DrawingToolRouteKind::BeginPreviewStroke,
            PointerEventKind::Down,
        );
        let outcome = session.handle_input(begin_input.clone());
        assert!(outcome.handled);
        assert!(matches!(
            outcome.intent,
            DrawingToolIntent::BeginPreviewStroke { .. }
        ));
        assert_eq!(
            session.active_kind(),
            Some(DrawingToolSessionKind::InkStroke)
        );
        assert_eq!(session.active_session_id(), Some(DrawingToolSessionId(1)));

        let anchor = session
            .active_anchor()
            .expect("begin should record an active session anchor");
        assert_eq!(anchor.device_id, begin_input.device_id);
        assert_eq!(anchor.source_kind, begin_input.source_kind);
        assert_eq!(anchor.tool_kind, begin_input.tool_kind);
        assert_eq!(anchor.started_screen_position, begin_input.screen_position);
        assert_eq!(anchor.started_canvas_position, begin_input.canvas_position);
        assert_eq!(anchor.timestamp_micros, begin_input.timestamp_micros);

        let anchor = anchor.clone();

        let update = session.handle_input(input_for_route(
            DrawingToolRouteKind::UpdatePreviewStroke,
            PointerEventKind::Move,
        ));
        assert!(update.handled);
        assert_eq!(session.active_session_id(), Some(DrawingToolSessionId(1)));
        assert_eq!(
            session
                .active_anchor()
                .expect("update should preserve the active session anchor"),
            &anchor
        );

        let finish = session.handle_input(input_for_route(
            DrawingToolRouteKind::EndPreviewStroke,
            PointerEventKind::Up,
        ));
        assert!(finish.handled);
        assert_eq!(session.phase(), &DrawingToolSessionPhase::Idle);
        assert_eq!(session.active_session_id(), None);
        assert_eq!(session.active_anchor(), None);

        session.handle_input(input_for_route(
            DrawingToolRouteKind::BeginPreviewStroke,
            PointerEventKind::Down,
        ));
        assert_eq!(session.active_session_id(), Some(DrawingToolSessionId(2)));
    }

    #[test]
    fn non_stroke_routes_do_not_create_active_sessions() {
        for (route_kind, pointer_kind) in [
            (DrawingToolRouteKind::Hover, PointerEventKind::Move),
            (DrawingToolRouteKind::Scroll, PointerEventKind::Scroll),
            (DrawingToolRouteKind::Ignored, PointerEventKind::Enter),
        ] {
            let mut session = DrawingToolSession::default();
            session.handle_input(input_for_route(route_kind, pointer_kind));
            assert_eq!(session.phase(), &DrawingToolSessionPhase::Idle);
            assert_eq!(session.active_session_id(), None);
            assert_eq!(session.active_anchor(), None);
        }
    }

    #[test]
    fn tool_session_maps_route_kinds_to_equivalent_intents() {
        let mut session = DrawingToolSession::default();

        let begin = session.handle_input(input_for_route(
            DrawingToolRouteKind::BeginPreviewStroke,
            PointerEventKind::Down,
        ));
        assert!(begin.handled);
        assert!(matches!(
            begin.intent,
            DrawingToolIntent::BeginPreviewStroke { .. }
        ));
        assert_eq!(
            session.active_kind(),
            Some(DrawingToolSessionKind::InkStroke)
        );

        let update = session.handle_input(input_for_route(
            DrawingToolRouteKind::UpdatePreviewStroke,
            PointerEventKind::Move,
        ));
        assert!(update.handled);
        assert!(matches!(
            update.intent,
            DrawingToolIntent::UpdatePreviewStroke { .. }
        ));
        assert_eq!(
            session.active_kind(),
            Some(DrawingToolSessionKind::InkStroke)
        );

        let finish = session.handle_input(input_for_route(
            DrawingToolRouteKind::EndPreviewStroke,
            PointerEventKind::Up,
        ));
        assert!(finish.handled);
        assert!(matches!(
            finish.intent,
            DrawingToolIntent::FinishPreviewStroke { .. }
        ));
        assert_eq!(session.active_kind(), None);

        let hover = session.handle_input(input_for_route(
            DrawingToolRouteKind::Hover,
            PointerEventKind::Move,
        ));
        assert!(hover.handled);
        assert!(matches!(hover.intent, DrawingToolIntent::Hover { .. }));

        let scroll = session.handle_input(input_for_route(
            DrawingToolRouteKind::Scroll,
            PointerEventKind::Scroll,
        ));
        assert!(scroll.handled);
        assert!(matches!(scroll.intent, DrawingToolIntent::Scroll { .. }));

        let ignored = session.handle_input(input_for_route(
            DrawingToolRouteKind::Ignored,
            PointerEventKind::Enter,
        ));
        assert!(!ignored.handled);
        assert!(matches!(ignored.intent, DrawingToolIntent::Ignore { .. }));
    }

    #[test]
    fn keyboard_control_input_observes_without_handling_or_session_changes() {
        let mut session = DrawingToolSession::default();
        let keyboard = KeyboardEvent {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
        };
        let input = DrawingToolControlInputEvent::from_keyboard(&keyboard);

        assert_eq!(
            input,
            DrawingToolControlInputEvent {
                source: DrawingToolControlInputSource::Keyboard {
                    key: Key::Escape,
                    state: KeyState::Pressed,
                    modifiers: Modifiers {
                        shift: true,
                        ..Modifiers::default()
                    },
                },
                request: DrawingToolControlRequest::Observe,
            }
        );

        let outcome = session.handle_control_input(input);

        assert!(!outcome.handled);
        assert!(matches!(
            outcome.intent,
            DrawingToolIntent::ControlInputObserved { .. }
        ));
        assert_eq!(session.phase(), &DrawingToolSessionPhase::Idle);
        assert_eq!(session.active_session_id(), None);
        assert_eq!(session.active_anchor(), None);
    }

    #[test]
    fn explicit_synthetic_cancel_request_reports_active_session_without_canceling() {
        let mut session = DrawingToolSession::default();
        session.handle_input(input_for_route(
            DrawingToolRouteKind::BeginPreviewStroke,
            PointerEventKind::Down,
        ));

        let outcome =
            session.handle_control_input(DrawingToolControlInputEvent::synthetic_cancel_request());

        assert!(!outcome.handled);
        assert!(matches!(
            outcome.intent,
            DrawingToolIntent::RequestCancel {
                input: DrawingToolControlInputEvent {
                    source: DrawingToolControlInputSource::Synthetic,
                    request: DrawingToolControlRequest::Cancel,
                },
                active_session_id: Some(DrawingToolSessionId(1)),
            }
        ));
        assert_eq!(session.active_session_id(), Some(DrawingToolSessionId(1)));
        assert_eq!(
            session.active_kind(),
            Some(DrawingToolSessionKind::InkStroke)
        );
    }

    #[test]
    fn explicit_synthetic_radial_request_reports_anchor_without_opening_ui() {
        let mut session = DrawingToolSession::default();
        let begin_input = input_for_route(
            DrawingToolRouteKind::BeginPreviewStroke,
            PointerEventKind::Down,
        );
        session.handle_input(begin_input.clone());

        let outcome = session
            .handle_control_input(DrawingToolControlInputEvent::synthetic_radial_menu_request());

        assert!(!outcome.handled);
        match outcome.intent {
            DrawingToolIntent::RequestRadialMenu {
                input:
                    DrawingToolControlInputEvent {
                        source: DrawingToolControlInputSource::Synthetic,
                        request: DrawingToolControlRequest::RadialMenu,
                    },
                anchor: Some(anchor),
            } => {
                assert_eq!(anchor.device_id, begin_input.device_id);
                assert_eq!(anchor.source_kind, begin_input.source_kind);
                assert_eq!(anchor.tool_kind, begin_input.tool_kind);
                assert_eq!(anchor.started_screen_position, begin_input.screen_position);
                assert_eq!(anchor.started_canvas_position, begin_input.canvas_position);
                assert_eq!(anchor.timestamp_micros, begin_input.timestamp_micros);
            }
            other => panic!("radial request should include the active anchor, got {other:?}"),
        }
        assert_eq!(session.active_session_id(), Some(DrawingToolSessionId(1)));
    }

    fn input_for_route(
        route_kind: DrawingToolRouteKind,
        pointer_kind: PointerEventKind,
    ) -> DrawingToolInputEvent {
        DrawingToolInputEvent {
            route_kind,
            pointer_kind,
            screen_position: UiPoint::new(12.0, 24.0),
            canvas_position: Some(CanvasCoordinate::new(12.0, 24.0)),
            source_kind: PointerSourceKind::Stylus,
            tool_kind: PointerToolKind::Pen,
            device_id: Some(PointerDeviceId(1)),
            timestamp_micros: Some(1_000),
            pressure: Some(0.5),
            tilt: None,
            twist_degrees: None,
            eraser: false,
            barrel_buttons: PointerBarrelButtons::none(),
            low_latency_preview: true,
            coalesced_sample_count: 0,
            predicted_sample_count: 0,
            coalesced_samples: Vec::new(),
        }
    }
}
