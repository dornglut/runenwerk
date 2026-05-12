use serde::{Deserialize, Serialize};

use crate::{FieldProductDiagnostic, ProductIdentity, ProductResidency, ProductScaleBand};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderTargetDescriptor {
    pub target_id: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

impl RenderTargetDescriptor {
    pub fn new(
        target_id: impl Into<String>,
        width: u32,
        height: u32,
        format: impl Into<String>,
    ) -> Self {
        Self {
            target_id: target_id.into(),
            width,
            height,
            format: format.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderResidencyRequest {
    pub product_id: ProductIdentity,
    pub residency: ProductResidency,
    pub priority: i32,
    pub hard_pin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderSelectedProduct {
    pub product_id: ProductIdentity,
    pub scale_band: ProductScaleBand,
    pub generation: u64,
    pub freshness_marker: Option<String>,
    pub residency_marker: Option<ProductResidency>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderDiagnosticsSelection {
    pub overlays_enabled: bool,
    pub selected_overlay_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderProductSelection {
    pub view_id: String,
    pub selected_products: Vec<RenderSelectedProduct>,
    pub required_targets: Vec<RenderTargetDescriptor>,
    pub residency_requests: Vec<RenderResidencyRequest>,
    pub diagnostics: Vec<FieldProductDiagnostic>,
    pub diagnostics_selection: RenderDiagnosticsSelection,
}

impl RenderProductSelection {
    pub fn new(view_id: impl Into<String>) -> Self {
        Self {
            view_id: view_id.into(),
            selected_products: Vec::new(),
            required_targets: Vec::new(),
            residency_requests: Vec::new(),
            diagnostics: Vec::new(),
            diagnostics_selection: RenderDiagnosticsSelection::default(),
        }
    }

    pub fn with_selected_product(mut self, selected: RenderSelectedProduct) -> Self {
        self.selected_products.push(selected);
        self
    }
}
