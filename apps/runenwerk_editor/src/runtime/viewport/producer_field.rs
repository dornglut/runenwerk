//! File: apps/runenwerk_editor/src/runtime/viewport/producer_field.rs
//! Purpose: Field-product debug preview descriptor helpers for editor viewports.

use editor_viewport::{ExpressionChannelLayerSliceMetadata, ExpressionPresentationHints};

pub const SCALAR_FIELD_DEBUG_PRODUCER: &str = "editor.viewport.scalar_field_debug_producer";
pub const VECTOR_FIELD_DEBUG_PRODUCER: &str = "editor.viewport.vector_field_debug_producer";

pub fn field_debug_presentation_hints() -> ExpressionPresentationHints {
    ExpressionPresentationHints {
        srgb: true,
        premultiplied_alpha: false,
        y_flipped: false,
    }
}

pub fn scalar_field_debug_metadata() -> ExpressionChannelLayerSliceMetadata {
    ExpressionChannelLayerSliceMetadata {
        channel_label: Some("scalar_debug_rgba".to_string()),
        layer_label: Some("field".to_string()),
        slice_label: None,
    }
}

pub fn vector_field_debug_metadata() -> ExpressionChannelLayerSliceMetadata {
    ExpressionChannelLayerSliceMetadata {
        channel_label: Some("vector_debug_rgba".to_string()),
        layer_label: Some("field".to_string()),
        slice_label: None,
    }
}
