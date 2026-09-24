use serde::{Deserialize, Serialize};

use crate::{
    FieldProductDiagnostic, ProductAuthorityClass, ProductFreshness, ProductIdentity,
    ProductQueryPolicy, ProductResidency, ProductScaleBand, QuerySnapshotProductDescriptor,
};

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

impl RenderResidencyRequest {
    pub fn new(
        product_id: ProductIdentity,
        residency: ProductResidency,
        priority: i32,
        hard_pin: bool,
    ) -> Self {
        Self {
            product_id,
            residency,
            priority,
            hard_pin,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderSelectedProduct {
    pub product_id: ProductIdentity,
    pub scale_band: ProductScaleBand,
    pub generation: u64,
    pub freshness: ProductFreshness,
    pub residency: ProductResidency,
    pub authority_class: ProductAuthorityClass,
    pub query_policy: ProductQueryPolicy,
}

impl RenderSelectedProduct {
    pub fn from_query_snapshot(snapshot: &QuerySnapshotProductDescriptor) -> Self {
        Self {
            product_id: snapshot.product_id(),
            scale_band: snapshot.descriptor.scale_band,
            generation: snapshot.response_generation,
            freshness: snapshot.freshness,
            residency: snapshot.descriptor.residency,
            authority_class: snapshot.descriptor.authority_class,
            query_policy: snapshot.requested_policy,
        }
    }
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

    pub fn with_required_target(mut self, target: RenderTargetDescriptor) -> Self {
        self.required_targets.push(target);
        self
    }

    pub fn with_residency_request(mut self, request: RenderResidencyRequest) -> Self {
        self.residency_requests.push(request);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: FieldProductDiagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn extend_diagnostics<I>(&mut self, diagnostics: I)
    where
        I: IntoIterator<Item = FieldProductDiagnostic>,
    {
        self.diagnostics.extend(diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProductConsumerClass, ProductDescriptorCore, ProductFamily, ProductKind, ProductLineage,
        ProductScope,
    };

    #[test]
    fn render_selection_selected_product_copies_typed_snapshot_state() {
        let mut descriptor = ProductDescriptorCore::new(
            ProductIdentity::new(17),
            ProductFamily::Expression,
            ProductKind::new("scene_color"),
            ProductScope::View {
                view_id: "viewport".to_string(),
            },
            ProductScaleBand::Preview,
            ProductLineage::new("test.producer", 5),
        );
        descriptor.consumer_class = ProductConsumerClass::Renderer;
        descriptor.freshness = ProductFreshness::Current;
        descriptor.residency = ProductResidency::Resident;
        descriptor.authority_class = ProductAuthorityClass::DeterministicDerived;
        let snapshot = QuerySnapshotProductDescriptor::new(
            descriptor,
            5,
            6,
            ProductQueryPolicy::StrictCurrentOnly,
        );

        let selected = RenderSelectedProduct::from_query_snapshot(&snapshot);

        assert_eq!(selected.product_id, ProductIdentity::new(17));
        assert_eq!(selected.generation, 6);
        assert_eq!(selected.freshness, ProductFreshness::Current);
        assert_eq!(selected.residency, ProductResidency::Resident);
        assert_eq!(
            selected.authority_class,
            ProductAuthorityClass::DeterministicDerived
        );
        assert_eq!(selected.query_policy, ProductQueryPolicy::StrictCurrentOnly);
    }
}
