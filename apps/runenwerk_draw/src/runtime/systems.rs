//! Runtime systems for the drawing app shell.

use engine::WindowState;
use engine::plugins::InputState;
use engine::plugins::render::{
    UiFrameProducerId, UiFrameRoute, UiFrameSubmission, UiFrameSubmissionOrder,
    UiFrameSubmissionRegistryResource,
};
use engine::runtime::{Res, ResMut};
use ui_input::{
    Modifiers, PointerButton, PointerEvent, PointerEventKind, PointerPacket, UiInputEvent,
};
use ui_math::{UiPoint, UiSize, UiVector};

use crate::runtime::resources::DrawingHostResource;

pub const DRAWING_UI_FRAME_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(4_001);

const fn ui_frame_producer_id(raw: u64) -> UiFrameProducerId {
    match UiFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

pub fn route_draw_input_system(input: Res<InputState>, mut host: ResMut<DrawingHostResource>) {
    let position = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    let delta = UiVector::new(input.mouse_delta.0, input.mouse_delta.1);
    let modifiers = Modifiers {
        shift: input.shift_down(),
        ctrl: false,
        alt: false,
        meta: false,
    };

    if input.left_mouse_pressed() {
        host.app
            .dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Down,
                position,
                UiVector::ZERO,
                Some(PointerButton::Primary),
                modifiers,
                1,
            )));
    }

    if input.left_mouse_down() && (delta.x.abs() > f32::EPSILON || delta.y.abs() > f32::EPSILON) {
        host.app
            .dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Move,
                position,
                delta,
                Some(PointerButton::Primary),
                modifiers,
                0,
            )));
    }

    if input.left_mouse_released() {
        host.app
            .dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Up,
                position,
                UiVector::ZERO,
                Some(PointerButton::Primary),
                modifiers,
                0,
            )));
    }

    if input.scroll_delta.abs() > f32::EPSILON {
        host.app
            .dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Scroll,
                position,
                UiVector::new(0.0, input.scroll_delta),
                None,
                modifiers,
                0,
            )));
    }
}

pub fn submit_draw_frame_system(
    window: Res<WindowState>,
    mut host: ResMut<DrawingHostResource>,
    mut submissions: ResMut<UiFrameSubmissionRegistryResource>,
) {
    let size = UiSize::new(window.size_px.0 as f32, window.size_px.1 as f32);
    let frame = host.app.rebuild_frame(size).clone();
    submissions.replace(
        UiFrameSubmission::new(DRAWING_UI_FRAME_PRODUCER_ID)
            .with_route(UiFrameRoute::Screen)
            .with_order(UiFrameSubmissionOrder::new(10, 0))
            .with_frame(frame),
    );
}

fn pointer_event(
    kind: PointerEventKind,
    position: UiPoint,
    delta: UiVector,
    button: Option<PointerButton>,
    modifiers: Modifiers,
    click_count: u8,
) -> PointerEvent {
    PointerEvent::new(kind, position, delta, button, modifiers, click_count)
        .with_packet(PointerPacket::mouse())
}
