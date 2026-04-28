//! File: domain/ui/ui_surface/src/presentation.rs
//! Purpose: Prepared presentation model contracts for surface rendering and interaction.

use std::collections::BTreeSet;

use crate::ObservationFrame;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePresentationModel<ItemId>
where
    ItemId: Copy + Ord,
{
    pub selected_primary_item: Option<ItemId>,
    selectable_primary_items: BTreeSet<ItemId>,
}

impl<ItemId> SurfacePresentationModel<ItemId>
where
    ItemId: Copy + Ord,
{
    pub fn new(
        selected_primary_item: Option<ItemId>,
        selectable_primary_items: impl IntoIterator<Item = ItemId>,
    ) -> Self {
        Self {
            selected_primary_item,
            selectable_primary_items: selectable_primary_items.into_iter().collect(),
        }
    }

    pub fn from_observation_frame(
        frame: &impl ObservationFrame<ItemId>,
        candidate_items: impl IntoIterator<Item = ItemId>,
    ) -> Self {
        let selectable = candidate_items
            .into_iter()
            .filter(|item| frame.is_item_available(*item));
        Self::new(frame.selected_primary_item(), selectable)
    }

    pub fn is_primary_selectable(&self, item_id: ItemId) -> bool {
        self.selectable_primary_items.contains(&item_id)
    }

    pub fn selectable_primary_items(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.selectable_primary_items.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyObservation;

    impl ObservationFrame<u64> for DummyObservation {
        fn selected_primary_item(&self) -> Option<u64> {
            Some(2)
        }

        fn is_item_available(&self, item_id: u64) -> bool {
            item_id.is_multiple_of(2)
        }
    }

    #[test]
    fn presentation_model_filters_selectable_items_from_observation() {
        let frame = DummyObservation;
        let model = SurfacePresentationModel::from_observation_frame(&frame, [1, 2, 3, 4]);

        assert_eq!(model.selected_primary_item, Some(2));
        assert_eq!(
            model.selectable_primary_items().collect::<Vec<_>>(),
            vec![2, 4]
        );
        assert!(model.is_primary_selectable(2));
        assert!(!model.is_primary_selectable(3));
    }
}
