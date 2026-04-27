//! File: domain/ui/ui_surface/src/observation.rs
//! Purpose: Observation-frame trait used to prepare surface presentation models.

pub trait ObservationFrame<ItemId>
where
    ItemId: Copy,
{
    fn selected_primary_item(&self) -> Option<ItemId>;

    fn is_item_available(&self, item_id: ItemId) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct DummyObservation;

    impl ObservationFrame<u64> for DummyObservation {
        fn selected_primary_item(&self) -> Option<u64> {
            Some(10)
        }

        fn is_item_available(&self, item_id: u64) -> bool {
            item_id == 10
        }
    }

    #[test]
    fn observation_frame_trait_provides_selected_and_availability_contract() {
        let frame = DummyObservation;
        assert_eq!(frame.selected_primary_item(), Some(10));
        assert!(frame.is_item_available(10));
        assert!(!frame.is_item_available(20));
    }
}
