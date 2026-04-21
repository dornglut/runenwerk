use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;

use crate::{AllocationError, FreeError, GenerationalId};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SlotState {
    Live,
    Free,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GenerationalAllocatorStats {
    pub live_slots: usize,
    pub free_slots: usize,
    pub retired_slots: usize,
    pub capacity_slots: usize,
}

/// Allocator-only lifecycle manager for generation-tracked IDs.
///
/// This type only manages allocation/free/liveness and generation invalidation.
/// It does not provide payload storage or registry/index behavior.
pub struct GenerationalIdAllocator<Tag> {
    next_slot: u64,
    free_slots: Vec<u32>,
    generations: Vec<u32>,
    states: Vec<SlotState>,
    live_count: usize,
    retired_count: usize,
    _marker: PhantomData<fn() -> Tag>,
}

impl<Tag> GenerationalIdAllocator<Tag> {
    pub fn new() -> Self {
        Self {
            next_slot: 0,
            free_slots: Vec::new(),
            generations: Vec::new(),
            states: Vec::new(),
            live_count: 0,
            retired_count: 0,
            _marker: PhantomData,
        }
    }

    pub fn try_allocate(&mut self) -> Result<GenerationalId<Tag>, AllocationError> {
        if let Some(slot) = self.free_slots.pop() {
            let index = slot as usize;
            if self.states.get(index) != Some(&SlotState::Free) {
                return Err(AllocationError::Exhausted);
            }
            self.states[index] = SlotState::Live;
            self.live_count += 1;
            return Ok(GenerationalId::from_parts(slot, self.generations[index]));
        }

        if self.next_slot > u32::MAX as u64 {
            return Err(AllocationError::Exhausted);
        }

        let slot = self.next_slot as u32;
        self.next_slot += 1;
        self.generations.push(0);
        self.states.push(SlotState::Live);
        self.live_count += 1;
        Ok(GenerationalId::from_parts(slot, 0))
    }

    pub fn allocate(&mut self) -> GenerationalId<Tag> {
        self.try_allocate()
            .expect("GenerationalIdAllocator exhausted")
    }

    pub fn try_free(&mut self, id: GenerationalId<Tag>) -> Result<(), FreeError> {
        let slot = id.slot() as usize;
        let Some(generation) = self.generations.get_mut(slot) else {
            return Err(FreeError::UnknownSlot { slot: id.slot() });
        };
        let Some(state) = self.states.get_mut(slot) else {
            return Err(FreeError::UnknownSlot { slot: id.slot() });
        };

        match *state {
            SlotState::Live => {}
            SlotState::Free | SlotState::Retired => {
                return Err(FreeError::NotLive { slot: id.slot() });
            }
        }

        if *generation != id.generation() {
            return Err(FreeError::StaleGeneration {
                slot: id.slot(),
                expected_generation: *generation,
                provided_generation: id.generation(),
            });
        }

        self.live_count = self.live_count.saturating_sub(1);

        match generation.checked_add(1) {
            Some(next_generation) => {
                *generation = next_generation;
                *state = SlotState::Free;
                self.free_slots.push(slot as u32);
            }
            None => {
                *state = SlotState::Retired;
                self.retired_count += 1;
            }
        }

        Ok(())
    }

    pub fn free(&mut self, id: GenerationalId<Tag>) {
        self.try_free(id)
            .expect("GenerationalIdAllocator::free called with invalid handle")
    }

    pub fn is_live(&self, id: GenerationalId<Tag>) -> bool {
        let slot = id.slot() as usize;
        let Some(state) = self.states.get(slot) else {
            return false;
        };
        if *state != SlotState::Live {
            return false;
        }
        self.generations.get(slot).copied() == Some(id.generation())
    }

    pub fn live_count(&self) -> usize {
        self.live_count
    }

    pub fn capacity_slots(&self) -> usize {
        self.generations.len()
    }

    pub fn stats(&self) -> GenerationalAllocatorStats {
        GenerationalAllocatorStats {
            live_slots: self.live_count,
            free_slots: self.free_slots.len(),
            retired_slots: self.retired_count,
            capacity_slots: self.capacity_slots(),
        }
    }
}

impl<Tag> Default for GenerationalIdAllocator<Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Tag> fmt::Debug for GenerationalIdAllocator<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationalIdAllocator")
            .field("next_slot", &self.next_slot)
            .field("live_count", &self.live_count)
            .field("stats", &self.stats())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use super::*;
    use proptest::prelude::*;
    use static_assertions::assert_not_impl_any;

    enum EntityTag {}

    #[test]
    fn allocates_and_reports_liveness() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let id = allocator.allocate();

        assert!(allocator.is_live(id));
        assert_eq!(allocator.live_count(), 1);
        assert_eq!(allocator.capacity_slots(), 1);
        assert_eq!(
            allocator.stats(),
            GenerationalAllocatorStats {
                live_slots: 1,
                free_slots: 0,
                retired_slots: 0,
                capacity_slots: 1,
            }
        );
    }

    #[test]
    fn reuse_increments_generation_and_invalidates_stale() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let first = allocator.allocate();
        allocator
            .try_free(first)
            .expect("initial live handle should free");

        let reused = allocator.allocate();
        assert_eq!(reused.slot(), first.slot());
        assert_eq!(reused.generation(), first.generation() + 1);
        assert!(!allocator.is_live(first));
        assert!(allocator.is_live(reused));
    }

    #[test]
    fn try_free_returns_error_for_stale_or_unknown_ids() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let first = allocator.allocate();
        allocator
            .try_free(first)
            .expect("initial live handle should free");

        let stale = allocator.try_free(first);
        assert_eq!(stale, Err(FreeError::NotLive { slot: first.slot() }));

        let unknown = allocator.try_free(GenerationalId::from_parts(99, 0));
        assert_eq!(unknown, Err(FreeError::UnknownSlot { slot: 99 }));
    }

    #[test]
    fn stale_generation_error_is_reported() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let first = allocator.allocate();
        allocator
            .try_free(first)
            .expect("initial live handle should free");

        let reused = allocator.allocate();
        let stale = allocator.try_free(first);
        assert_eq!(
            stale,
            Err(FreeError::StaleGeneration {
                slot: reused.slot(),
                expected_generation: reused.generation(),
                provided_generation: first.generation(),
            })
        );
    }

    #[test]
    fn generation_overflow_retires_slot() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let id = allocator.allocate();

        allocator.generations[id.slot() as usize] = u32::MAX;
        allocator
            .try_free(GenerationalId::from_parts(id.slot(), u32::MAX))
            .expect("slot at generation max should retire");

        assert!(!allocator.is_live(GenerationalId::from_parts(id.slot(), u32::MAX)));
        assert!(allocator.free_slots.is_empty());
        assert_eq!(allocator.states[id.slot() as usize], SlotState::Retired);
        assert_eq!(allocator.stats().retired_slots, 1);
    }

    #[test]
    fn exhausted_when_no_free_and_no_new_slots() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let id = allocator.allocate();

        allocator.generations[id.slot() as usize] = u32::MAX;
        allocator
            .try_free(GenerationalId::from_parts(id.slot(), u32::MAX))
            .expect("slot at generation max should retire");
        allocator.next_slot = (u32::MAX as u64) + 1;

        assert_eq!(allocator.try_allocate(), Err(AllocationError::Exhausted));
    }

    #[test]
    #[should_panic(expected = "GenerationalIdAllocator exhausted")]
    fn allocate_panics_when_exhausted() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        allocator.next_slot = (u32::MAX as u64) + 1;
        let _ = allocator.allocate();
    }

    #[test]
    fn allocator_is_not_copy_or_clone() {
        assert_not_impl_any!(GenerationalIdAllocator<EntityTag>: Copy, Clone);
    }

    proptest! {
        #[test]
        fn property_live_handles_are_unique(count in 1usize..256usize) {
            let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
            let handles = (0..count)
                .map(|_| allocator.allocate())
                .collect::<Vec<_>>();

            let unique = handles
                .iter()
                .map(|id| id.raw())
                .collect::<BTreeSet<_>>();
            prop_assert_eq!(unique.len(), handles.len());
            prop_assert!(handles.into_iter().all(|id| allocator.is_live(id)));
        }
    }
}
