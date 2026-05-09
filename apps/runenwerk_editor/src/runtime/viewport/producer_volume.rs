//! File: apps/runenwerk_editor/src/runtime/viewport/producer_volume.rs
//! Purpose: Volume and history debug preview descriptor helpers for editor viewports.

use editor_viewport::{ExpressionChannelLayerSliceMetadata, ExpressionPresentationHints};

pub const ATLAS_DEBUG_PRODUCER: &str = "editor.viewport.atlas_debug_producer";
pub const VOLUME_SLICE_DEBUG_PRODUCER: &str = "editor.viewport.volume_slice_debug_producer";
pub const BRICKMAP_DEBUG_PRODUCER: &str = "editor.viewport.brickmap_debug_producer";
pub const HISTORY_COLOR_DEBUG_PRODUCER: &str = "editor.viewport.history_color_debug_producer";

pub fn volume_debug_presentation_hints() -> ExpressionPresentationHints {
    ExpressionPresentationHints {
        srgb: true,
        premultiplied_alpha: false,
        y_flipped: false,
    }
}

pub fn atlas_debug_metadata() -> ExpressionChannelLayerSliceMetadata {
    ExpressionChannelLayerSliceMetadata {
        channel_label: None,
        layer_label: Some("atlas_debug".to_string()),
        slice_label: None,
    }
}

pub fn volume_slice_debug_metadata() -> ExpressionChannelLayerSliceMetadata {
    ExpressionChannelLayerSliceMetadata {
        channel_label: None,
        layer_label: Some("volume_debug".to_string()),
        slice_label: Some("z=0".to_string()),
    }
}

pub fn brickmap_debug_metadata() -> ExpressionChannelLayerSliceMetadata {
    ExpressionChannelLayerSliceMetadata {
        channel_label: Some("occupancy_debug".to_string()),
        layer_label: Some("brickmap".to_string()),
        slice_label: None,
    }
}

pub fn history_color_debug_metadata() -> ExpressionChannelLayerSliceMetadata {
    ExpressionChannelLayerSliceMetadata {
        channel_label: Some("color_debug".to_string()),
        layer_label: Some("history".to_string()),
        slice_label: Some("previous".to_string()),
    }
}
