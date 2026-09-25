//! Runtime systems for the drawing app shell.

use std::collections::BTreeSet;

use engine::PrimaryPresentationMetricsResource;
use engine::plugins::render::inspect::RenderDebugConfigResource;
use engine::plugins::render::{
    PreparedRenderFrameRequestResource, PreparedRenderProductSelectionResource,
    RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderDynamicTextureTargetRequestRegistryResource,
    RenderDynamicTextureUploadDescriptor, RenderDynamicTextureUploadRegistryResource,
    RenderFrameProducerId, RenderProductSurfaceManifest, RenderTextureSampleMode,
    RenderTextureTargetFormat, RenderTextureTargetUsage, RenderTextureUploadAlphaMode,
    SurfaceFrameRoute, SurfaceFrameSubmission, SurfaceFrameSubmissionOrder,
    SurfaceFrameSubmissionRegistryResource,
};
use engine::plugins::{InputState, MouseButtonTransitionSample, MouseMotionSample};
use engine::runtime::RuntimeJobExecutorResource;
use engine::runtime::{Res, ResMut};
use native_tablet_input::{
    NativeTabletBackendStatus, NativeTabletDeviceControlResource, NativeTabletFrameResource,
};
use product::{
    ProductAuthorityClass, ProductFreshness, ProductIdentity, ProductQueryPolicy, ProductResidency,
    RenderProductSelection, RenderSelectedProduct, RenderTargetDescriptor,
};
use runen_input::{
    AnalogMeasurement, ContactPhase, ContactPresence, DeliveryRole, EvidenceStatus,
    InputObservation, InputObservationGroup, InputToolKind, MeasurementDomain, SourceTime,
    SourceTimeUnit, StylusTilt, TabletObservation,
};
use ui_input::{
    Modifiers, PointerButton, PointerContactId, PointerContactPhase, PointerContactState,
    PointerDeviceCapabilities, PointerDeviceId, PointerEvent, PointerEventKind,
    PointerLatencyClass, PointerPacket, PointerSample, PointerSampleRole, PointerSourceKind,
    PointerTilt, PointerToolKind, UiInputEvent,
};
use ui_math::{UiPoint, UiSize, UiVector};
use ui_render_data::ProductSurfaceTextureBindingSource;

use crate::app::{
    DRAWING_INK_TEXTURE_NAMESPACE, DrawingInkSurfaceKind, DrawingTabletPanelProjection,
    RunenwerkDrawApp, drawing_ink_texture_target_id,
};
use crate::runtime::gpu_ink::{DrawingInkGpuFlowResource, prepare_drawing_ink_gpu_frame};
use crate::runtime::ink::process_drawing_preview_ink_jobs;
use crate::runtime::resources::{
    DrawingHostResource, DrawingInkUploadTrackerResource, NativeClaimStateResource,
    NativeInputStreamKey,
};

pub const DRAWING_UI_FRAME_PRODUCER_ID: RenderFrameProducerId = ui_frame_producer_id(4_001);
pub const DRAWING_RENDER_FRAME_PRODUCER_ID: RenderFrameProducerId = render_frame_producer_id(4_001);
pub const NATIVE_CONTACT_FALLBACK_SUPPRESSION_IDLE_FRAME_LIMIT: u32 = 12;

#[derive(runen_ecs::SystemParam)]
pub struct DrawingFrameSubmissionResources<'w> {
    submissions: ResMut<'w, SurfaceFrameSubmissionRegistryResource>,
    dynamic_targets: ResMut<'w, RenderDynamicTextureTargetRequestRegistryResource>,
    texture_uploads: ResMut<'w, RenderDynamicTextureUploadRegistryResource>,
    product_selections: ResMut<'w, PreparedRenderProductSelectionResource>,
    frame_requests: ResMut<'w, PreparedRenderFrameRequestResource>,
    debug_config: ResMut<'w, RenderDebugConfigResource>,
}

const fn ui_frame_producer_id(raw: u64) -> RenderFrameProducerId {
    match RenderFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

const fn render_frame_producer_id(raw: u64) -> RenderFrameProducerId {
    match RenderFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("render frame producer id constants must be non-zero"),
    }
}

pub fn route_draw_input_system(
    mut input: ResMut<InputState>,
    mut native_frame: ResMut<NativeTabletFrameResource>,
    native_control: Res<NativeTabletDeviceControlResource>,
    mut native_claims: ResMut<NativeClaimStateResource>,
    mut host: ResMut<DrawingHostResource>,
) {
    let position = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    let delta = UiVector::new(input.mouse_delta.0, input.mouse_delta.1);
    let motion_samples = input.mouse_motion_samples().to_vec();
    let touch_samples = input.touch_samples().to_vec();
    let modifiers = Modifiers {
        shift: input.shift_down(),
        ctrl: false,
        alt: false,
        meta: false,
    };

    host.app
        .update_tablet_panel(tablet_panel_projection(&native_frame, &native_control));
    native_frame.publish_to_neutral(&mut input);
    native_claims.begin_frame();
    let native_events =
        project_neutral_tablet_groups(input.drain_device_observation_groups(), &mut native_claims);
    for event in coalesce_pointer_move_events(native_events) {
        host.app.dispatch_input(&event);
    }
    if native_claims.has_active_stream()
        || native_claims.suppresses_fallback_this_frame()
        || native_claims.has_current_frame_activity()
    {
        return;
    }

    if !touch_samples.is_empty() {
        route_touch_input(&mut host.app, &touch_samples, modifiers);
        return;
    }

    route_winit_mouse_fallback(
        &mut host.app,
        &input,
        position,
        delta,
        &motion_samples,
        modifiers,
    );

    if input.scroll_delta.abs() > f32::EPSILON {
        host.app
            .dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Scroll,
                position,
                UiVector::new(0.0, input.scroll_delta),
                None,
                modifiers,
                0,
                PointerPacket::mouse(),
            )));
    }
}

fn project_neutral_tablet_groups(
    groups: Vec<InputObservationGroup>,
    claims: &mut NativeClaimStateResource,
) -> Vec<UiInputEvent> {
    let mut events = Vec::new();
    for group in groups {
        let Some(current) = group
            .observations
            .iter()
            .find_map(|observation| match observation {
                InputObservation::Tablet(tablet)
                    if tablet.delivery == DeliveryRole::OrdinaryCurrent
                        && tablet.evidence == EvidenceStatus::ObservedConfirmed =>
                {
                    Some(tablet)
                }
                _ => None,
            })
        else {
            continue;
        };

        let key = NativeInputStreamKey {
            context: group.context,
            tool: current.tool,
            contact: current.contact,
        };
        if current.phase == ContactPhase::Begin {
            claims.observe_current_frame_activity();
        }
        match current.phase {
            ContactPhase::Begin | ContactPhase::Update
                if current.presence == ContactPresence::Contact =>
            {
                claims.observe_contact(key)
            }
            ContactPhase::End | ContactPhase::Cancel => claims.observe_terminal(key),
            ContactPhase::Begin | ContactPhase::Update => {}
        }

        let packet = pointer_packet_from_neutral(&group, current);
        let (kind, button) = pointer_semantics(current);
        events.push(UiInputEvent::Pointer(
            PointerEvent::new(
                kind,
                ui_math::UiPoint::new(current.position.x, current.position.y),
                ui_math::UiVector::new(current.delta.x, current.delta.y),
                button,
                Modifiers::default(),
                0,
            )
            .with_packet(packet),
        ));
    }
    events
}

fn pointer_semantics(observation: &TabletObservation) -> (PointerEventKind, Option<PointerButton>) {
    let kind = match observation.phase {
        ContactPhase::Begin => PointerEventKind::Down,
        ContactPhase::End => PointerEventKind::Up,
        ContactPhase::Cancel => PointerEventKind::Leave,
        ContactPhase::Update if observation.presence == ContactPresence::Hover => {
            PointerEventKind::Move
        }
        ContactPhase::Update => PointerEventKind::Move,
    };
    let button =
        (observation.presence == ContactPresence::Contact).then_some(PointerButton::Primary);
    (kind, button)
}

fn pointer_packet_from_neutral(
    group: &InputObservationGroup,
    current: &TabletObservation,
) -> PointerPacket {
    let capabilities = ui_capabilities_for_neutral_group(group, current);
    let mut packet = PointerPacket {
        source_kind: pointer_source_kind(current.tool_kind),
        tool_kind: pointer_tool_kind(current.tool_kind),
        device_id: group
            .context
            .device
            .map(|device| PointerDeviceId(device.raw())),
        contact_id: Some(PointerContactId(current.contact.raw())),
        contact_phase: Some(pointer_contact_phase(current.phase)),
        timestamp_micros: source_time_micros(current.source_time),
        contact: pointer_contact_state(current.presence),
        pressure: project_pressure(current.pressure),
        tilt: project_tilt(current.tilt),
        twist_degrees: project_twist(current.twist),
        tangential_pressure: project_tangential_pressure(current.tangential_pressure),
        eraser: current.controls.eraser,
        barrel_buttons: ui_input::PointerBarrelButtons {
            primary: current.controls.barrel_primary,
            secondary: current.controls.barrel_secondary,
        },
        capabilities,
        calibration: None,
        latency_class: PointerLatencyClass::LowLatencyPreview,
        coalesced_samples: Vec::new(),
        predicted_samples: Vec::new(),
    };

    for observation in &group.observations {
        let InputObservation::Tablet(observation) = observation else {
            continue;
        };
        if observation.evidence == EvidenceStatus::PredictedProvisional {
            packet.predicted_samples.push(pointer_sample_from_neutral(
                observation,
                PointerSampleRole::Predicted,
            ));
        } else if observation.evidence == EvidenceStatus::ObservedConfirmed
            && observation.delivery == DeliveryRole::HistoricalCoalesced
        {
            packet.coalesced_samples.push(pointer_sample_from_neutral(
                observation,
                PointerSampleRole::Coalesced,
            ));
        }
    }
    packet.capabilities.coalesced_samples = !packet.coalesced_samples.is_empty();
    packet.capabilities.predicted_samples = !packet.predicted_samples.is_empty();
    packet
}

fn pointer_sample_from_neutral(
    observation: &TabletObservation,
    role: PointerSampleRole,
) -> PointerSample {
    PointerSample {
        role,
        position: ui_math::UiPoint::new(observation.position.x, observation.position.y),
        delta: ui_math::UiVector::new(observation.delta.x, observation.delta.y),
        timestamp_micros: source_time_micros(observation.source_time),
        pressure: project_pressure(observation.pressure),
        tilt: project_tilt(observation.tilt),
        twist_degrees: project_twist(observation.twist),
        tangential_pressure: project_tangential_pressure(observation.tangential_pressure),
        contact: pointer_contact_state(observation.presence),
    }
}

fn ui_capabilities_for_neutral_group(
    group: &InputObservationGroup,
    current: &TabletObservation,
) -> PointerDeviceCapabilities {
    PointerDeviceCapabilities {
        pressure: current.capabilities.pressure
            && group_measurements_are_projectable(
                group,
                |observation| observation.pressure,
                project_pressure,
            ),
        tilt: current.capabilities.tilt,
        twist: current.capabilities.twist
            && group_measurements_are_projectable(
                group,
                |observation| observation.twist,
                project_twist,
            ),
        tangential_pressure: current.capabilities.tangential_pressure
            && group_measurements_are_projectable(
                group,
                |observation| observation.tangential_pressure,
                project_tangential_pressure,
            ),
        hover: current.capabilities.hover,
        eraser: current.capabilities.eraser,
        barrel_buttons: current.capabilities.barrel_controls,
        coalesced_samples: current.capabilities.historical_samples,
        predicted_samples: current.capabilities.predicted_samples,
        calibration: false,
    }
}

fn group_measurements_are_projectable(
    group: &InputObservationGroup,
    select: impl Fn(&TabletObservation) -> Option<AnalogMeasurement>,
    project: impl Fn(Option<AnalogMeasurement>) -> Option<f32>,
) -> bool {
    group
        .observations
        .iter()
        .filter_map(|observation| {
            let InputObservation::Tablet(tablet) = observation else {
                return None;
            };
            select(tablet)
        })
        .all(|measurement| project(Some(measurement)).is_some())
}

fn project_pressure(measurement: Option<AnalogMeasurement>) -> Option<f32> {
    measurement.and_then(|measurement| match measurement.domain {
        MeasurementDomain::NormalizedUnitInterval
        | MeasurementDomain::Bounded { min: 0.0, max: 1.0 } => (0.0..=1.0)
            .contains(&measurement.value)
            .then_some(measurement.value),
        _ => None,
    })
}

fn project_tilt(tilt: Option<StylusTilt>) -> Option<PointerTilt> {
    tilt.map(|tilt| PointerTilt::new(tilt.x_degrees, tilt.y_degrees))
}

fn project_twist(measurement: Option<AnalogMeasurement>) -> Option<f32> {
    measurement.and_then(|measurement| match measurement.domain {
        MeasurementDomain::Degrees {
            min: 0.0,
            max: 360.0,
        } if (0.0..=360.0).contains(&measurement.value) => Some(measurement.value),
        _ => None,
    })
}

fn project_tangential_pressure(measurement: Option<AnalogMeasurement>) -> Option<f32> {
    measurement.and_then(|measurement| match measurement.domain {
        MeasurementDomain::NormalizedUnitInterval
        | MeasurementDomain::Bounded { min: 0.0, max: 1.0 }
            if (0.0..=1.0).contains(&measurement.value) =>
        {
            Some(measurement.value)
        }
        _ => None,
    })
}

fn source_time_micros(source_time: Option<SourceTime>) -> Option<u64> {
    source_time.and_then(|source_time| match source_time.unit {
        SourceTimeUnit::Microseconds => Some(source_time.value),
        SourceTimeUnit::Milliseconds => source_time.value.checked_mul(1_000),
        SourceTimeUnit::Nanoseconds => Some(source_time.value / 1_000),
        SourceTimeUnit::NativeTicks { ticks_per_second } if ticks_per_second > 0 => source_time
            .value
            .checked_mul(1_000_000)
            .map(|value| value / ticks_per_second),
        SourceTimeUnit::NativeTicks { .. } => None,
    })
}

fn pointer_source_kind(tool_kind: InputToolKind) -> PointerSourceKind {
    match tool_kind {
        InputToolKind::Mouse => PointerSourceKind::Mouse,
        InputToolKind::Finger => PointerSourceKind::Touch,
        _ => PointerSourceKind::Stylus,
    }
}

fn pointer_tool_kind(tool_kind: InputToolKind) -> PointerToolKind {
    match tool_kind {
        InputToolKind::Mouse => PointerToolKind::Mouse,
        InputToolKind::Pen => PointerToolKind::Pen,
        InputToolKind::Brush => PointerToolKind::Brush,
        InputToolKind::Marker => PointerToolKind::Marker,
        InputToolKind::Airbrush => PointerToolKind::Airbrush,
        InputToolKind::Eraser => PointerToolKind::Eraser,
        InputToolKind::Finger => PointerToolKind::Finger,
        InputToolKind::Unknown => PointerToolKind::Unknown,
    }
}

fn pointer_contact_state(presence: ContactPresence) -> PointerContactState {
    match presence {
        ContactPresence::Hover => PointerContactState::Hover,
        ContactPresence::Contact => PointerContactState::Contact,
        ContactPresence::OutOfRange => PointerContactState::OutOfRange,
    }
}

fn pointer_contact_phase(phase: ContactPhase) -> PointerContactPhase {
    match phase {
        ContactPhase::Begin => PointerContactPhase::Begin,
        ContactPhase::Update => PointerContactPhase::Update,
        ContactPhase::End => PointerContactPhase::End,
        ContactPhase::Cancel => PointerContactPhase::Cancel,
    }
}

fn route_winit_mouse_fallback(
    app: &mut crate::app::RunenwerkDrawApp,
    input: &InputState,
    frame_position: UiPoint,
    frame_delta: UiVector,
    motion_samples: &[MouseMotionSample],
    modifiers: Modifiers,
) {
    let left_press = input.left_mouse_pressed_transition();
    let left_release = input.left_mouse_released_transition();

    if input.left_mouse_pressed() {
        let down_position = left_press
            .map(mouse_transition_position)
            .unwrap_or(frame_position);
        app.dispatch_input(&UiInputEvent::Pointer(pointer_event(
            PointerEventKind::Down,
            down_position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            modifiers,
            1,
            PointerPacket::mouse(),
        )));
    }

    let contact_start_index = if input.left_mouse_pressed() {
        left_press
            .map(|transition| transition.motion_sample_index)
            .unwrap_or(motion_samples.len())
    } else {
        0
    }
    .min(motion_samples.len());
    let contact_end_index = if input.left_mouse_released() {
        left_release
            .map(|transition| transition.motion_sample_index)
            .unwrap_or(contact_start_index)
    } else {
        motion_samples.len()
    }
    .min(motion_samples.len());
    let contact_motion_samples = if contact_start_index <= contact_end_index {
        &motion_samples[contact_start_index..contact_end_index]
    } else {
        &[]
    };

    if input.left_mouse_released() {
        let release_position = left_release
            .map(mouse_transition_position)
            .unwrap_or(frame_position);
        let (sample_position, sample_delta, packet) =
            pointer_release_packet(release_position, contact_motion_samples);
        app.dispatch_input(&UiInputEvent::Pointer(pointer_event(
            PointerEventKind::Up,
            sample_position,
            sample_delta,
            Some(PointerButton::Primary),
            modifiers,
            0,
            packet,
        )));
        return;
    }

    if input.left_mouse_down() {
        if !contact_motion_samples.is_empty() {
            let (sample_position, sample_delta, packet) =
                pointer_motion_packet(frame_position, frame_delta, contact_motion_samples);
            app.dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Move,
                sample_position,
                sample_delta,
                Some(PointerButton::Primary),
                modifiers,
                0,
                packet,
            )));
        } else if !input.left_mouse_pressed()
            && (frame_delta.x.abs() > f32::EPSILON || frame_delta.y.abs() > f32::EPSILON)
        {
            app.dispatch_input(&UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Move,
                frame_position,
                frame_delta,
                Some(PointerButton::Primary),
                modifiers,
                0,
                PointerPacket::mouse(),
            )));
        }
    }
}

fn mouse_transition_position(transition: MouseButtonTransitionSample) -> UiPoint {
    UiPoint::new(transition.position.0, transition.position.1)
}

pub fn process_draw_preview_ink_jobs_system(
    mut host: ResMut<DrawingHostResource>,
    mut executor: ResMut<RuntimeJobExecutorResource>,
) {
    process_drawing_preview_ink_jobs(&mut host.app, &mut executor);
}

fn tablet_panel_projection(
    frame: &NativeTabletFrameResource,
    control: &NativeTabletDeviceControlResource,
) -> DrawingTabletPanelProjection {
    let active_backend = frame
        .backend_health
        .iter()
        .find(|health| health.status == NativeTabletBackendStatus::Active)
        .or_else(|| frame.backend_health.first())
        .map(|health| health.backend.label().to_string())
        .unwrap_or_else(|| "winit fallback".to_string());
    let active_device = frame
        .devices
        .iter()
        .find(|device| device.active)
        .or_else(|| frame.devices.first())
        .map(|device| device.name.clone())
        .unwrap_or_else(|| "mouse or trackpad".to_string());
    let backend_warnings = frame
        .backend_health
        .iter()
        .filter(|health| {
            matches!(
                health.status,
                NativeTabletBackendStatus::Unavailable | NativeTabletBackendStatus::Error
            )
        })
        .count();

    DrawingTabletPanelProjection {
        active_backend,
        active_device,
        sample_rate_hz: frame.telemetry.sample_rate_hz,
        max_segment_gap_px: frame.telemetry.max_segment_gap_px,
        pressure_available: frame.telemetry.pressure_available,
        tilt_available: frame.telemetry.tilt_available,
        dropped_samples: frame.telemetry.dropped_samples_this_frame,
        duplicate_samples: frame.telemetry.duplicate_samples_this_frame,
        warning_count: frame.diagnostics.len().saturating_add(backend_warnings),
        backend_mode: format!("{:?}", control.backend_preference),
        pressure_scale: control.calibration.pressure_scale,
        pressure_bias: control.calibration.pressure_bias,
        cursor_offset: UiVector::new(
            control.calibration.cursor_offset.x,
            control.calibration.cursor_offset.y,
        ),
    }
}

fn coalesce_pointer_move_events(events: Vec<UiInputEvent>) -> Vec<UiInputEvent> {
    let mut coalesced = Vec::with_capacity(events.len());
    let mut pending_move = None;

    for event in events {
        let pointer = match event {
            UiInputEvent::Pointer(pointer) => pointer,
            event => {
                flush_pending_move(&mut coalesced, &mut pending_move);
                coalesced.push(event);
                continue;
            }
        };

        if pointer.kind == PointerEventKind::Move
            && pointer.packet.contact == PointerContactState::Contact
        {
            pending_move = match pending_move.take() {
                Some(previous) if can_coalesce_pointer_moves(&previous, &pointer) => {
                    Some(coalesce_pointer_move_pair(previous, pointer))
                }
                Some(previous) => {
                    coalesced.push(UiInputEvent::Pointer(previous));
                    Some(pointer)
                }
                None => Some(pointer),
            };
        } else {
            flush_pending_move(&mut coalesced, &mut pending_move);
            coalesced.push(UiInputEvent::Pointer(pointer));
        }
    }

    flush_pending_move(&mut coalesced, &mut pending_move);
    coalesced
}

fn flush_pending_move(events: &mut Vec<UiInputEvent>, pending_move: &mut Option<PointerEvent>) {
    if let Some(pointer) = pending_move.take() {
        events.push(UiInputEvent::Pointer(pointer));
    }
}

fn can_coalesce_pointer_moves(previous: &PointerEvent, current: &PointerEvent) -> bool {
    previous.packet.source_kind == current.packet.source_kind
        && previous.packet.tool_kind == current.packet.tool_kind
        && previous.packet.device_id == current.packet.device_id
        && previous.packet.contact_id == current.packet.contact_id
        && previous.packet.eraser == current.packet.eraser
        && previous.button == current.button
        && previous.modifiers == current.modifiers
}

fn coalesce_pointer_move_pair(previous: PointerEvent, mut current: PointerEvent) -> PointerEvent {
    let previous_sample = pointer_sample_from_event(&previous);
    let mut samples = previous.packet.coalesced_samples;
    samples.push(previous_sample);
    samples.extend(current.packet.coalesced_samples);
    current.packet.coalesced_samples = samples;
    current.packet.capabilities.coalesced_samples = !current.packet.coalesced_samples.is_empty();
    current
}

fn pointer_sample_from_event(event: &PointerEvent) -> PointerSample {
    let packet = &event.packet;
    PointerSample {
        role: PointerSampleRole::Coalesced,
        position: event.position,
        delta: event.delta,
        timestamp_micros: packet.timestamp_micros,
        pressure: packet.pressure,
        tilt: packet.tilt,
        twist_degrees: packet.twist_degrees,
        tangential_pressure: packet.tangential_pressure,
        contact: packet.contact,
    }
}

pub fn submit_draw_frame_system(
    presentation: Res<PrimaryPresentationMetricsResource>,
    mut host: ResMut<DrawingHostResource>,
    mut upload_tracker: ResMut<DrawingInkUploadTrackerResource>,
    gpu_flow: Res<DrawingInkGpuFlowResource>,
    render_submission: DrawingFrameSubmissionResources<'_>,
) {
    let DrawingFrameSubmissionResources {
        mut submissions,
        mut dynamic_targets,
        mut texture_uploads,
        mut product_selections,
        mut frame_requests,
        mut debug_config,
    } = render_submission;
    let size_px = presentation.size_px();
    let size = UiSize::new(size_px.0 as f32, size_px.1 as f32);
    let frame = host.app.rebuild_frame(size).clone();
    let committed_products = host
        .app
        .ink_runtime()
        .visible_products()
        .cloned()
        .collect::<Vec<_>>();
    let preview_products = host.app.ink_runtime().preview_products().to_vec();
    upload_tracker.retain_products(DrawingInkSurfaceKind::Committed, &committed_products);
    upload_tracker.retain_products(DrawingInkSurfaceKind::Preview, &preview_products);
    let committed_upload_products = upload_tracker
        .products_requiring_upload(DrawingInkSurfaceKind::Committed, &committed_products);
    let preview_upload_products =
        upload_tracker.products_requiring_upload(DrawingInkSurfaceKind::Preview, &preview_products);

    let gpu_target_descriptors = prepare_drawing_ink_gpu_frame(
        &mut host.app,
        DRAWING_RENDER_FRAME_PRODUCER_ID,
        &gpu_flow,
        &mut frame_requests,
        &mut debug_config,
        &committed_products,
        &preview_products,
    );
    let target_descriptors =
        ink_target_descriptors(&committed_products, DrawingInkSurfaceKind::Committed)
            .into_iter()
            .chain(ink_target_descriptors(
                &preview_products,
                DrawingInkSurfaceKind::Preview,
            ))
            .chain(gpu_target_descriptors)
            .collect::<Vec<_>>();
    let uploads = ink_uploads(&committed_upload_products, DrawingInkSurfaceKind::Committed)
        .into_iter()
        .chain(ink_uploads(
            &preview_upload_products,
            DrawingInkSurfaceKind::Preview,
        ))
        .collect::<Vec<_>>();
    let manifest = drawing_ink_product_surface_manifest(
        &host.app,
        target_descriptors,
        uploads,
        &committed_products,
        &preview_products,
    );
    debug_assert!(
        !manifest.has_error_diagnostics(),
        "drawing ink product-surface manifest should be structurally valid"
    );
    let (target_descriptors, uploads, _, _) = manifest.into_render_parts();
    let target_requests_accepted =
        dynamic_targets.replace_contribution(DRAWING_RENDER_FRAME_PRODUCER_ID, target_descriptors)
            .map(|_| true)
            .unwrap_or_else(|err| {
                tracing::warn!(target = "runenwerk_draw.ink", error = %err, "drawing ink target request rejected");
                false
            });

    let uploads_accepted =
        texture_uploads.replace_contribution(DRAWING_RENDER_FRAME_PRODUCER_ID, uploads)
            .map(|_| true)
            .unwrap_or_else(|err| {
                tracing::warn!(target = "runenwerk_draw.ink", error = %err, "drawing ink upload rejected");
                false
            });
    if target_requests_accepted && uploads_accepted {
        upload_tracker
            .record_submitted_uploads(DrawingInkSurfaceKind::Committed, committed_upload_products);
        upload_tracker
            .record_submitted_uploads(DrawingInkSurfaceKind::Preview, preview_upload_products);
    }

    if let Err(err) = product_selections.replace_contribution(
        DRAWING_RENDER_FRAME_PRODUCER_ID,
        [ink_product_selection(&committed_products)],
    ) {
        tracing::warn!(target = "runenwerk_draw.ink", error = %err, "drawing ink product selection rejected");
    }

    submissions.replace(
        SurfaceFrameSubmission::new(DRAWING_UI_FRAME_PRODUCER_ID)
            .with_route(SurfaceFrameRoute::Screen)
            .with_order(SurfaceFrameSubmissionOrder::new(10, 0))
            .with_frame(frame),
    );
}

fn drawing_ink_product_surface_manifest(
    app: &RunenwerkDrawApp,
    target_descriptors: Vec<RenderDynamicTextureTargetDescriptor>,
    uploads: Vec<RenderDynamicTextureUploadDescriptor>,
    committed_products: &[drawing::DrawingInkTileProduct],
    preview_products: &[drawing::DrawingInkTileProduct],
) -> RenderProductSurfaceManifest {
    let upload_target_keys = uploads
        .iter()
        .map(|upload| upload.target_key.clone())
        .collect::<BTreeSet<_>>();
    let manifest =
        RenderProductSurfaceManifest::new(DRAWING_RENDER_FRAME_PRODUCER_ID, "runenwerk_draw.ink")
            .with_dynamic_targets(target_descriptors)
            .with_dynamic_uploads(uploads);
    committed_products.iter().fold(
        preview_products.iter().fold(manifest, |manifest, product| {
            with_ink_product_surface_binding(
                app,
                manifest,
                DrawingInkSurfaceKind::Preview,
                product,
                &upload_target_keys,
            )
        }),
        |manifest, product| {
            with_ink_product_surface_binding(
                app,
                manifest,
                DrawingInkSurfaceKind::Committed,
                product,
                &upload_target_keys,
            )
        },
    )
}

fn with_ink_product_surface_binding(
    app: &RunenwerkDrawApp,
    manifest: RenderProductSurfaceManifest,
    surface_kind: DrawingInkSurfaceKind,
    product: &drawing::DrawingInkTileProduct,
    upload_target_keys: &BTreeSet<RenderDynamicTextureTargetKey>,
) -> RenderProductSurfaceManifest {
    let binding = ink_surface_binding(app, surface_kind, product);
    match binding.backing {
        DrawingInkSurfaceBacking::Upload if upload_target_keys.contains(&binding.target_key) => {
            manifest.with_upload_backed_product_surface_binding(binding.surface_key, binding.source)
        }
        DrawingInkSurfaceBacking::Upload | DrawingInkSurfaceBacking::DynamicTarget => {
            manifest.with_product_surface_binding(binding.surface_key, binding.source)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DrawingInkSurfaceBinding {
    target_key: RenderDynamicTextureTargetKey,
    surface_key: String,
    source: ProductSurfaceTextureBindingSource,
    backing: DrawingInkSurfaceBacking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawingInkSurfaceBacking {
    Upload,
    DynamicTarget,
}

fn ink_target_descriptors(
    products: &[drawing::DrawingInkTileProduct],
    surface_kind: DrawingInkSurfaceKind,
) -> Vec<RenderDynamicTextureTargetDescriptor> {
    products
        .iter()
        .map(|product| {
            RenderDynamicTextureTargetDescriptor::new(
                ink_target_key(surface_kind, product),
                product.payload.width.max(1),
                product.payload.height.max(1),
                RenderTextureTargetFormat::Rgba8Unorm,
                ink_texture_target_usage(),
                RenderTextureSampleMode::FilterableFloat,
                RenderDynamicTextureRetention::RetainWhileRequested,
            )
        })
        .collect()
}

fn ink_uploads(
    products: &[&drawing::DrawingInkTileProduct],
    surface_kind: DrawingInkSurfaceKind,
) -> Vec<RenderDynamicTextureUploadDescriptor> {
    products
        .iter()
        .map(|product| {
            RenderDynamicTextureUploadDescriptor::rgba8(
                ink_target_key(surface_kind, product),
                0,
                0,
                product.payload.width.max(1),
                product.payload.height.max(1),
                RenderTextureUploadAlphaMode::Premultiplied,
                product.descriptor_generation,
                product.payload.rgba8_premultiplied.clone(),
            )
        })
        .collect()
}

fn ink_product_selection(products: &[drawing::DrawingInkTileProduct]) -> RenderProductSelection {
    let mut selection = RenderProductSelection::new("runenwerk.draw.canvas");
    for product in products {
        let target_id = drawing_ink_texture_target_id(
            DrawingInkSurfaceKind::Committed,
            product.metadata.quality_class,
            product.metadata.tile_id,
        );
        selection = selection
            .with_selected_product(RenderSelectedProduct {
                product_id: ProductIdentity::new(product.metadata.product_id.raw()),
                scale_band: drawing::drawing_quality_scale_band(product.metadata.quality_class),
                generation: product.descriptor_generation,
                freshness: ProductFreshness::Current,
                residency: ProductResidency::NotApplicable,
                authority_class: ProductAuthorityClass::DeterministicDerived,
                query_policy: ProductQueryPolicy::StrictCurrentOnly,
            })
            .with_required_target(RenderTargetDescriptor::new(
                target_id,
                product.payload.width.max(1),
                product.payload.height.max(1),
                "rgba8_unorm",
            ));
    }
    selection
}

fn ink_target_key(
    surface_kind: DrawingInkSurfaceKind,
    product: &drawing::DrawingInkTileProduct,
) -> RenderDynamicTextureTargetKey {
    RenderDynamicTextureTargetKey::new(
        DRAWING_INK_TEXTURE_NAMESPACE,
        drawing_ink_texture_target_id(
            surface_kind,
            product.metadata.quality_class,
            product.metadata.tile_id,
        ),
    )
}

fn ink_visible_surface_kind(
    app: &RunenwerkDrawApp,
    surface_kind: DrawingInkSurfaceKind,
    product: &drawing::DrawingInkTileProduct,
) -> DrawingInkSurfaceKind {
    app.ink_runtime()
        .visible_surface_kind_for(surface_kind, product)
}

fn ink_surface_binding(
    app: &RunenwerkDrawApp,
    surface_kind: DrawingInkSurfaceKind,
    product: &drawing::DrawingInkTileProduct,
) -> DrawingInkSurfaceBinding {
    let visible_surface_kind = ink_visible_surface_kind(app, surface_kind, product);
    let key = ink_target_key(visible_surface_kind, product);
    DrawingInkSurfaceBinding {
        target_key: key.clone(),
        surface_key: key.to_string(),
        source: ProductSurfaceTextureBindingSource::dynamic_texture(key.namespace, key.target_id),
        backing: ink_surface_backing(visible_surface_kind),
    }
}

fn ink_surface_backing(surface_kind: DrawingInkSurfaceKind) -> DrawingInkSurfaceBacking {
    match surface_kind {
        DrawingInkSurfaceKind::Committed | DrawingInkSurfaceKind::Preview => {
            DrawingInkSurfaceBacking::Upload
        }
        DrawingInkSurfaceKind::GpuCommitted | DrawingInkSurfaceKind::GpuPreview => {
            DrawingInkSurfaceBacking::DynamicTarget
        }
    }
}

fn ink_texture_target_usage() -> RenderTextureTargetUsage {
    RenderTextureTargetUsage {
        color_attachment: false,
        depth_attachment: false,
        sampled: true,
        storage: false,
        copy_src: true,
        copy_dst: true,
    }
}

fn pointer_event(
    kind: PointerEventKind,
    position: UiPoint,
    delta: UiVector,
    button: Option<PointerButton>,
    modifiers: Modifiers,
    click_count: u8,
    packet: PointerPacket,
) -> PointerEvent {
    PointerEvent::new(kind, position, delta, button, modifiers, click_count).with_packet(packet)
}

fn route_touch_input(
    app: &mut crate::app::RunenwerkDrawApp,
    samples: &[engine::plugins::TouchInputSample],
    modifiers: Modifiers,
) {
    let mut pending_motion_samples = Vec::new();
    for sample in samples {
        match sample.phase {
            engine::plugins::TouchInputPhase::Started => {
                dispatch_touch_motion(app, &mut pending_motion_samples, modifiers);
                app.dispatch_input(&UiInputEvent::Pointer(touch_pointer_event(
                    PointerEventKind::Down,
                    sample,
                    Vec::new(),
                    modifiers,
                    1,
                )));
            }
            engine::plugins::TouchInputPhase::Moved => {
                pending_motion_samples.push(*sample);
            }
            engine::plugins::TouchInputPhase::Ended
            | engine::plugins::TouchInputPhase::Cancelled => {
                dispatch_touch_motion(app, &mut pending_motion_samples, modifiers);
                app.dispatch_input(&UiInputEvent::Pointer(touch_pointer_event(
                    PointerEventKind::Up,
                    sample,
                    Vec::new(),
                    modifiers,
                    0,
                )));
            }
        }
    }
    dispatch_touch_motion(app, &mut pending_motion_samples, modifiers);
}

fn dispatch_touch_motion(
    app: &mut crate::app::RunenwerkDrawApp,
    pending_motion_samples: &mut Vec<engine::plugins::TouchInputSample>,
    modifiers: Modifiers,
) {
    let Some(current) = pending_motion_samples.pop() else {
        return;
    };
    let coalesced = std::mem::take(pending_motion_samples);
    app.dispatch_input(&UiInputEvent::Pointer(touch_pointer_event(
        PointerEventKind::Move,
        &current,
        coalesced,
        modifiers,
        0,
    )));
}

fn touch_pointer_event(
    kind: PointerEventKind,
    sample: &engine::plugins::TouchInputSample,
    coalesced_samples: Vec<engine::plugins::TouchInputSample>,
    modifiers: Modifiers,
    click_count: u8,
) -> PointerEvent {
    PointerEvent::new(
        kind,
        UiPoint::new(sample.position.0, sample.position.1),
        UiVector::new(sample.delta.0, sample.delta.1),
        Some(PointerButton::Primary),
        modifiers,
        click_count,
    )
    .with_packet(touch_pointer_packet(sample, coalesced_samples))
}

fn touch_pointer_packet(
    sample: &engine::plugins::TouchInputSample,
    coalesced_samples: Vec<engine::plugins::TouchInputSample>,
) -> PointerPacket {
    let pressure = sample.pressure;
    let coalesced_samples = coalesced_samples
        .into_iter()
        .map(|sample| {
            let mut pointer_sample = PointerSample::new(
                PointerSampleRole::Coalesced,
                UiPoint::new(sample.position.0, sample.position.1),
                UiVector::new(sample.delta.0, sample.delta.1),
            );
            pointer_sample.pressure = sample.pressure;
            pointer_sample
        })
        .collect::<Vec<_>>();
    PointerPacket {
        source_kind: PointerSourceKind::Touch,
        tool_kind: PointerToolKind::Finger,
        device_id: Some(PointerDeviceId(sample.id)),
        pressure,
        contact: PointerContactState::Contact,
        latency_class: ui_input::PointerLatencyClass::LowLatencyPreview,
        capabilities: PointerDeviceCapabilities {
            pressure: pressure.is_some(),
            coalesced_samples: !coalesced_samples.is_empty(),
            ..PointerDeviceCapabilities::default()
        },
        coalesced_samples,
        ..PointerPacket::default()
    }
}

fn pointer_motion_packet(
    fallback_position: UiPoint,
    fallback_delta: UiVector,
    samples: &[MouseMotionSample],
) -> (UiPoint, UiVector, PointerPacket) {
    let Some((last, coalesced)) = samples.split_last() else {
        return (fallback_position, fallback_delta, PointerPacket::mouse());
    };

    let packet = pointer_packet_with_coalesced_mouse_samples(coalesced);
    (
        UiPoint::new(last.position.0, last.position.1),
        UiVector::new(last.delta.0, last.delta.1),
        packet,
    )
}

fn pointer_release_packet(
    release_position: UiPoint,
    samples: &[MouseMotionSample],
) -> (UiPoint, UiVector, PointerPacket) {
    let Some((last, coalesced)) = samples.split_last() else {
        return (release_position, UiVector::ZERO, PointerPacket::mouse());
    };

    let last_position = UiPoint::new(last.position.0, last.position.1);
    if last_position == release_position {
        return (
            last_position,
            UiVector::new(last.delta.0, last.delta.1),
            pointer_packet_with_coalesced_mouse_samples(coalesced),
        );
    }

    (
        release_position,
        UiVector::new(
            release_position.x - last_position.x,
            release_position.y - last_position.y,
        ),
        pointer_packet_with_coalesced_mouse_samples(samples),
    )
}

fn pointer_packet_with_coalesced_mouse_samples(samples: &[MouseMotionSample]) -> PointerPacket {
    PointerPacket::mouse().with_coalesced_samples(samples.iter().map(|sample| {
        PointerSample::new(
            PointerSampleRole::Coalesced,
            UiPoint::new(sample.position.0, sample.position.1),
            UiVector::new(sample.delta.0, sample.delta.1),
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use drawing::{
        CanvasCoordinate, CanvasRect, CanvasTileId, CompositeOutputId, DrawingDocumentRevision,
        DrawingInkTilePayload, DrawingInkTileProduct, DrawingProductLineage, DrawingTileProduct,
        DrawingTileProductId, DrawingTileProductSource, FormationVersion, ProductQualityClass,
        TilePyramidLevel,
    };
    use runen_input::{
        ContactId, CoordinateSpace, InputContext, InputDeviceId, InputSourceId, ObservationOrigin,
        PhysicalTabletControls, Point2, TabletCapabilities, ToolId, Vector2,
    };

    fn drawing_product(
        product_id: u64,
        quality_class: ProductQualityClass,
        tile_x: i64,
    ) -> DrawingInkTileProduct {
        let tile_id = CanvasTileId::new(TilePyramidLevel::new(0), tile_x, 0);
        let revision = DrawingDocumentRevision::new(1);
        let source = DrawingTileProductSource::new(
            quality_class,
            revision,
            CompositeOutputId::new(1),
            DrawingProductLineage::new(revision),
            FormationVersion::new(1),
            CanvasRect::new(
                CanvasCoordinate::new(0.0, 0.0),
                CanvasCoordinate::new(2.0, 2.0),
            ),
        );

        DrawingInkTileProduct {
            metadata: DrawingTileProduct::new(
                DrawingTileProductId::new(product_id),
                tile_id,
                source,
            ),
            payload: DrawingInkTilePayload::new(
                2,
                2,
                vec![
                    255, 255, 255, 255, 0, 0, 0, 0, 255, 0, 0, 255, 0, 255, 0, 255,
                ],
            ),
            cache_key: format!("test-product-{product_id}"),
            descriptor_generation: product_id,
            diagnostics: Vec::new(),
        }
    }

    fn passing_gpu_metrics() -> crate::app::DrawingInkGpuValidationMetrics {
        crate::app::DrawingInkGpuValidationMetrics {
            max_channel_delta: 0,
            changed_pixel_count: 0,
            total_pixel_count: 4,
            changed_pixel_ratio: 0.0,
        }
    }

    #[test]
    fn drawing_product_surface_manifest_traces_upload_backed_committed_tiles() {
        let app = RunenwerkDrawApp::new();
        let product = drawing_product(7, ProductQualityClass::Final, 3);
        let targets = ink_target_descriptors(
            std::slice::from_ref(&product),
            DrawingInkSurfaceKind::Committed,
        );
        let uploads = ink_uploads(&[&product], DrawingInkSurfaceKind::Committed);

        let manifest = drawing_ink_product_surface_manifest(
            &app,
            targets,
            uploads,
            std::slice::from_ref(&product),
            &[],
        );

        assert_eq!(manifest.dynamic_targets().len(), 1);
        assert_eq!(manifest.dynamic_uploads().len(), 1);
        assert_eq!(manifest.product_bindings().len(), 1);
        assert_eq!(
            manifest.product_bindings()[0].surface_key,
            manifest.dynamic_targets()[0].key.to_string()
        );
        assert!(manifest.product_bindings()[0].upload_required);
        assert!(
            manifest.diagnostics().is_empty(),
            "drawing upload-backed product-surface manifest should be valid"
        );
    }

    #[test]
    fn drawing_product_surface_manifest_traces_preview_tiles() {
        let app = RunenwerkDrawApp::new();
        let product = drawing_product(8, ProductQualityClass::Preview, 4);
        let targets = ink_target_descriptors(
            std::slice::from_ref(&product),
            DrawingInkSurfaceKind::Preview,
        );
        let uploads = ink_uploads(&[&product], DrawingInkSurfaceKind::Preview);

        let manifest =
            drawing_ink_product_surface_manifest(&app, targets, uploads, &[], &[product]);

        assert_eq!(manifest.product_family(), "runenwerk_draw.ink");
        assert_eq!(manifest.product_bindings().len(), 1);
        assert_eq!(
            manifest.product_bindings()[0].source,
            ProductSurfaceTextureBindingSource::dynamic_texture(
                DRAWING_INK_TEXTURE_NAMESPACE,
                "preview.preview.L0.4.0",
            )
        );
        assert!(manifest.product_bindings()[0].upload_required);
        assert!(
            manifest.diagnostics().is_empty(),
            "drawing preview product-surface manifest should be valid"
        );
    }

    #[test]
    fn drawing_product_surface_manifest_binds_gpu_committed_tiles_without_upload_requirement() {
        let mut app = RunenwerkDrawApp::new();
        let product = drawing_product(9, ProductQualityClass::Final, 5);
        app.ink_runtime_mut().record_gpu_validation_pass(
            DrawingInkSurfaceKind::Committed,
            &product,
            passing_gpu_metrics(),
        );
        let targets = ink_target_descriptors(
            std::slice::from_ref(&product),
            DrawingInkSurfaceKind::Committed,
        )
        .into_iter()
        .chain(ink_target_descriptors(
            std::slice::from_ref(&product),
            DrawingInkSurfaceKind::GpuCommitted,
        ))
        .collect::<Vec<_>>();
        let gpu_key = ink_target_key(DrawingInkSurfaceKind::GpuCommitted, &product);

        let manifest = drawing_ink_product_surface_manifest(
            &app,
            targets,
            Vec::new(),
            std::slice::from_ref(&product),
            &[],
        );

        assert_eq!(manifest.dynamic_targets().len(), 2);
        assert!(manifest.dynamic_uploads().is_empty());
        assert_eq!(manifest.product_bindings().len(), 1);
        assert_eq!(
            manifest.product_bindings()[0].surface_key,
            gpu_key.to_string()
        );
        assert_eq!(
            manifest.product_bindings()[0].source,
            ProductSurfaceTextureBindingSource::dynamic_texture(
                gpu_key.namespace.clone(),
                gpu_key.target_id.clone(),
            )
        );
        assert!(!manifest.product_bindings()[0].upload_required);
        assert!(
            manifest.diagnostics().is_empty(),
            "GPU-promoted committed drawing surface should not require a CPU upload"
        );
    }

    #[test]
    fn drawing_product_surface_manifest_binds_gpu_preview_tiles_without_upload_requirement() {
        let mut app = RunenwerkDrawApp::new();
        let product = drawing_product(10, ProductQualityClass::Preview, 6);
        app.ink_runtime_mut().record_gpu_validation_pass(
            DrawingInkSurfaceKind::Preview,
            &product,
            passing_gpu_metrics(),
        );
        let targets = ink_target_descriptors(
            std::slice::from_ref(&product),
            DrawingInkSurfaceKind::Preview,
        )
        .into_iter()
        .chain(ink_target_descriptors(
            std::slice::from_ref(&product),
            DrawingInkSurfaceKind::GpuPreview,
        ))
        .collect::<Vec<_>>();
        let gpu_key = ink_target_key(DrawingInkSurfaceKind::GpuPreview, &product);

        let manifest =
            drawing_ink_product_surface_manifest(&app, targets, Vec::new(), &[], &[product]);

        assert_eq!(manifest.product_bindings().len(), 1);
        assert_eq!(
            manifest.product_bindings()[0].surface_key,
            gpu_key.to_string()
        );
        assert!(!manifest.product_bindings()[0].upload_required);
        assert!(
            manifest.diagnostics().is_empty(),
            "GPU-promoted preview drawing surface should not require a CPU upload"
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn tablet_observation(
        phase: ContactPhase,
        evidence: EvidenceStatus,
        delivery: DeliveryRole,
        position_x: f32,
        pressure: Option<AnalogMeasurement>,
        tangential_pressure: Option<AnalogMeasurement>,
        twist: Option<AnalogMeasurement>,
        capabilities: TabletCapabilities,
    ) -> TabletObservation {
        TabletObservation {
            contact: ContactId::new(44),
            tool: Some(ToolId::new(8)),
            tool_kind: InputToolKind::Pen,
            phase,
            presence: ContactPresence::Contact,
            position: Point2::new(position_x, 20.0, CoordinateSpace::WindowPhysicalPixels),
            delta: Vector2::new(1.0, 0.0),
            pressure,
            tangential_pressure,
            tilt: None,
            twist,
            controls: PhysicalTabletControls::default(),
            capabilities,
            source_time: None,
            evidence,
            delivery,
            origin: ObservationOrigin::SourceReport,
        }
    }

    fn tablet_group(observations: Vec<TabletObservation>) -> InputObservationGroup {
        InputObservationGroup::new(
            InputContext::new(InputSourceId::new(31), Some(InputDeviceId::new(9))),
            observations
                .into_iter()
                .map(InputObservation::Tablet)
                .collect(),
        )
    }

    #[test]
    fn draw_projects_capabilities_and_sample_evidence_without_relabeling() {
        let capabilities = TabletCapabilities {
            pressure: true,
            tangential_pressure: true,
            twist: true,
            historical_samples: true,
            predicted_samples: true,
            ..Default::default()
        };
        let historical = tablet_observation(
            ContactPhase::Update,
            EvidenceStatus::ObservedConfirmed,
            DeliveryRole::HistoricalCoalesced,
            10.0,
            Some(AnalogMeasurement::new(
                0.7,
                MeasurementDomain::NormalizedUnitInterval,
            )),
            None,
            None,
            capabilities,
        );
        let current = tablet_observation(
            ContactPhase::Update,
            EvidenceStatus::ObservedConfirmed,
            DeliveryRole::OrdinaryCurrent,
            20.0,
            None,
            Some(AnalogMeasurement::new(
                -0.4,
                MeasurementDomain::SignedNormalizedUnitInterval,
            )),
            Some(AnalogMeasurement::new(
                45.0,
                MeasurementDomain::Degrees {
                    min: 0.0,
                    max: 360.0,
                },
            )),
            capabilities,
        );
        let predicted_historical = tablet_observation(
            ContactPhase::Update,
            EvidenceStatus::PredictedProvisional,
            DeliveryRole::HistoricalCoalesced,
            30.0,
            Some(AnalogMeasurement::new(
                0.9,
                MeasurementDomain::NormalizedUnitInterval,
            )),
            None,
            None,
            capabilities,
        );
        let estimated = tablet_observation(
            ContactPhase::Update,
            EvidenceStatus::EstimatedRevisable,
            DeliveryRole::OrdinaryCurrent,
            40.0,
            Some(AnalogMeasurement::new(
                0.2,
                MeasurementDomain::NormalizedUnitInterval,
            )),
            None,
            None,
            capabilities,
        );
        let group = tablet_group(vec![
            historical,
            current.clone(),
            predicted_historical,
            estimated,
        ]);

        let packet = pointer_packet_from_neutral(&group, &current);

        assert_eq!(packet.pressure, None);
        assert_eq!(packet.coalesced_samples.len(), 1);
        assert_eq!(packet.coalesced_samples[0].pressure, Some(0.7));
        assert_eq!(packet.predicted_samples.len(), 1);
        assert_eq!(packet.predicted_samples[0].pressure, Some(0.9));
        assert_eq!(packet.tangential_pressure, None);
        assert!(packet.capabilities.pressure);
        assert!(!packet.capabilities.tangential_pressure);
        assert!(packet.capabilities.twist);
        assert_eq!(packet.twist_degrees, Some(45.0));
        assert!(packet.is_valid());

        let mut claims = NativeClaimStateResource::default();
        let events = project_neutral_tablet_groups(
            vec![tablet_group(vec![tablet_observation(
                ContactPhase::Update,
                EvidenceStatus::EstimatedRevisable,
                DeliveryRole::OrdinaryCurrent,
                50.0,
                None,
                None,
                None,
                capabilities,
            )])],
            &mut claims,
        );
        assert!(events.is_empty());
        assert!(!claims.has_current_frame_activity());
    }

    #[test]
    fn draw_marks_native_down_as_current_frame_activity_without_claiming_stale_hover() {
        let observation = tablet_observation(
            ContactPhase::Begin,
            EvidenceStatus::ObservedConfirmed,
            DeliveryRole::OrdinaryCurrent,
            10.0,
            None,
            None,
            None,
            TabletCapabilities {
                hover: true,
                ..Default::default()
            },
        );
        let mut group = tablet_group(vec![observation]);
        if let InputObservation::Tablet(tablet) = &mut group.observations[0] {
            tablet.presence = ContactPresence::Hover;
        }
        let mut claims = NativeClaimStateResource::default();

        let events = project_neutral_tablet_groups(vec![group], &mut claims);

        assert_eq!(events.len(), 1);
        assert!(claims.has_current_frame_activity());
        assert!(!claims.has_active_stream());
    }
}
