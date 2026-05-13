//! Runtime systems for the drawing app shell.

use engine::WindowState;
use engine::plugins::InputState;
use engine::plugins::render::{
    PreparedRenderProductSelectionResource, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey,
    RenderDynamicTextureTargetRequestRegistryResource, RenderDynamicTextureUploadDescriptor,
    RenderDynamicTextureUploadRegistryResource, RenderFrameProducerId, RenderTextureSampleMode,
    RenderTextureTargetFormat, RenderTextureTargetUsage, RenderTextureUploadAlphaMode,
    UiFrameProducerId, UiFrameRoute, UiFrameSubmission, UiFrameSubmissionOrder,
    UiFrameSubmissionRegistryResource,
};
use engine::runtime::{Res, ResMut};
use product::{
    ProductAuthorityClass, ProductFreshness, ProductIdentity, ProductQueryPolicy, ProductResidency,
    ProductScaleBand, RenderProductSelection, RenderSelectedProduct, RenderTargetDescriptor,
};
use ui_input::{
    Modifiers, PointerButton, PointerEvent, PointerEventKind, PointerPacket, UiInputEvent,
};
use ui_math::{UiPoint, UiSize, UiVector};

use crate::app::{
    DRAWING_INK_TEXTURE_NAMESPACE, DrawingInkSurfaceKind, drawing_ink_texture_target_id,
};
use crate::runtime::resources::DrawingHostResource;

pub const DRAWING_UI_FRAME_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(4_001);
pub const DRAWING_RENDER_FRAME_PRODUCER_ID: RenderFrameProducerId = render_frame_producer_id(4_001);

const fn ui_frame_producer_id(raw: u64) -> UiFrameProducerId {
    match UiFrameProducerId::try_from_raw(raw) {
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
    mut dynamic_targets: ResMut<RenderDynamicTextureTargetRequestRegistryResource>,
    mut texture_uploads: ResMut<RenderDynamicTextureUploadRegistryResource>,
    mut product_selections: ResMut<PreparedRenderProductSelectionResource>,
) {
    let size = UiSize::new(window.size_px.0 as f32, window.size_px.1 as f32);
    let frame = host.app.rebuild_frame(size).clone();
    let committed_products = host
        .app
        .ink_runtime()
        .visible_products()
        .cloned()
        .collect::<Vec<_>>();
    let preview_products = host.app.ink_runtime().preview_products().to_vec();

    let target_descriptors =
        ink_target_descriptors(&committed_products, DrawingInkSurfaceKind::Committed)
            .into_iter()
            .chain(ink_target_descriptors(
                &preview_products,
                DrawingInkSurfaceKind::Preview,
            ))
            .collect::<Vec<_>>();
    if let Err(err) =
        dynamic_targets.replace_contribution(DRAWING_RENDER_FRAME_PRODUCER_ID, target_descriptors)
    {
        tracing::warn!(target = "runenwerk_draw.ink", error = %err, "drawing ink target request rejected");
    }

    let uploads = ink_uploads(&committed_products, DrawingInkSurfaceKind::Committed)
        .into_iter()
        .chain(ink_uploads(
            &preview_products,
            DrawingInkSurfaceKind::Preview,
        ))
        .collect::<Vec<_>>();
    if let Err(err) =
        texture_uploads.replace_contribution(DRAWING_RENDER_FRAME_PRODUCER_ID, uploads)
    {
        tracing::warn!(target = "runenwerk_draw.ink", error = %err, "drawing ink upload rejected");
    }

    if let Err(err) = product_selections.replace_contribution(
        DRAWING_RENDER_FRAME_PRODUCER_ID,
        [ink_product_selection(&committed_products)],
    ) {
        tracing::warn!(target = "runenwerk_draw.ink", error = %err, "drawing ink product selection rejected");
    }

    submissions.replace(
        UiFrameSubmission::new(DRAWING_UI_FRAME_PRODUCER_ID)
            .with_route(UiFrameRoute::Screen)
            .with_order(UiFrameSubmissionOrder::new(10, 0))
            .with_frame(frame),
    );
}

fn ink_target_descriptors(
    products: &[drawing::DrawingInkTileProduct],
    surface_kind: DrawingInkSurfaceKind,
) -> Vec<RenderDynamicTextureTargetDescriptor> {
    products
        .iter()
        .map(|product| {
            RenderDynamicTextureTargetDescriptor::new(
                ink_target_key(surface_kind, product.metadata.tile_id),
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
    products: &[drawing::DrawingInkTileProduct],
    surface_kind: DrawingInkSurfaceKind,
) -> Vec<RenderDynamicTextureUploadDescriptor> {
    products
        .iter()
        .map(|product| {
            RenderDynamicTextureUploadDescriptor::rgba8(
                ink_target_key(surface_kind, product.metadata.tile_id),
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
            product.metadata.tile_id,
        );
        selection = selection
            .with_selected_product(RenderSelectedProduct {
                product_id: ProductIdentity::new(product.metadata.product_id.raw()),
                scale_band: ProductScaleBand::Preview,
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
    tile_id: drawing::CanvasTileId,
) -> RenderDynamicTextureTargetKey {
    RenderDynamicTextureTargetKey::new(
        DRAWING_INK_TEXTURE_NAMESPACE,
        drawing_ink_texture_target_id(surface_kind, tile_id),
    )
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
) -> PointerEvent {
    PointerEvent::new(kind, position, delta, button, modifiers, click_count)
        .with_packet(PointerPacket::mouse())
}
