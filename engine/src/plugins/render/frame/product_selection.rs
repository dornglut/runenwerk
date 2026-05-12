use product::RenderProductSelection;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct PreparedRenderProductSelectionResource {
    selections: Vec<RenderProductSelection>,
}

impl PreparedRenderProductSelectionResource {
    pub fn replace(
        &mut self,
        selections: impl IntoIterator<Item = RenderProductSelection>,
    ) -> Vec<RenderProductSelection> {
        std::mem::replace(&mut self.selections, selections.into_iter().collect())
    }

    pub fn push(&mut self, selection: RenderProductSelection) {
        self.selections.push(selection);
    }

    pub fn clear(&mut self) {
        self.selections.clear();
    }

    pub fn selections(&self) -> &[RenderProductSelection] {
        &self.selections
    }

    pub fn snapshot(&self) -> Vec<RenderProductSelection> {
        self.selections.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use product::{ProductIdentity, ProductScaleBand, RenderSelectedProduct};

    #[test]
    fn product_selection_resource_carries_generation_without_backend_handles() {
        let selection =
            RenderProductSelection::new("main").with_selected_product(RenderSelectedProduct {
                product_id: ProductIdentity::new(42),
                scale_band: ProductScaleBand::Preview,
                generation: 9,
                freshness_marker: None,
                residency_marker: None,
            });
        let mut resource = PreparedRenderProductSelectionResource::default();

        resource.replace([selection]);

        let selected = &resource.selections()[0].selected_products[0];
        assert_eq!(selected.product_id, ProductIdentity::new(42));
        assert_eq!(selected.generation, 9);
    }
}
