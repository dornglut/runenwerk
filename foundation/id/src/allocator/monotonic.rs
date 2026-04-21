use core::fmt;
use core::marker::PhantomData;

use crate::{AllocationError, InvalidRawId, TypedId};

/// Monotonic allocator for typed scalar IDs.
///
/// Exhaustion is explicit allocator state. This allocator reserves `u64::MAX`
/// as a terminal next value and never issues it.
pub struct MonotonicIdAllocator<Tag> {
    next: u64,
    exhausted: bool,
    _marker: PhantomData<fn() -> Tag>,
}

impl<Tag> MonotonicIdAllocator<Tag> {
    pub fn new(start_at: u64) -> Result<Self, AllocationError> {
        if start_at == 0 {
            return Err(AllocationError::InvalidStart { start_at });
        }

        Ok(Self {
            next: start_at,
            exhausted: start_at == u64::MAX,
            _marker: PhantomData,
        })
    }

    pub fn next_raw(&self) -> u64 {
        self.next
    }

    pub fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    pub fn try_allocate(&mut self) -> Result<TypedId<Tag>, AllocationError> {
        if self.exhausted {
            return Err(AllocationError::Exhausted);
        }

        let id = TypedId::new(self.next);
        if self.next == u64::MAX - 1 {
            self.next = u64::MAX;
            self.exhausted = true;
        } else {
            self.next += 1;
        }
        Ok(id)
    }

    pub fn allocate(&mut self) -> TypedId<Tag> {
        self.try_allocate().expect("MonotonicIdAllocator exhausted")
    }

    pub fn try_allocate_batch<const N: usize>(
        &mut self,
    ) -> Result<[TypedId<Tag>; N], AllocationError> {
        if N == 0 {
            return Ok(core::array::from_fn(|_| self.allocate()));
        }

        let available = self.available_before_exhaustion();
        if (N as u64) > available {
            return Err(AllocationError::Exhausted);
        }

        let ids = core::array::from_fn(|_| self.allocate());
        Ok(ids)
    }

    pub fn allocate_batch<const N: usize>(&mut self) -> [TypedId<Tag>; N] {
        self.try_allocate_batch()
            .expect("MonotonicIdAllocator exhausted during batch allocation")
    }

    pub fn try_advance_to_at_least(&mut self, minimum_next: u64) -> Result<(), InvalidRawId> {
        if minimum_next == 0 {
            return Err(InvalidRawId::new(0));
        }
        if self.exhausted {
            return Ok(());
        }
        if self.next < minimum_next {
            self.next = minimum_next;
            if minimum_next == u64::MAX {
                self.exhausted = true;
            }
        }
        Ok(())
    }

    pub fn advance_to_at_least(&mut self, minimum_next: u64) {
        self.try_advance_to_at_least(minimum_next)
            .expect("minimum_next must be non-zero");
    }

    fn available_before_exhaustion(&self) -> u64 {
        if self.exhausted {
            0
        } else {
            u64::MAX - self.next
        }
    }
}

impl<Tag> Default for MonotonicIdAllocator<Tag> {
    fn default() -> Self {
        Self::new(1).expect("default monotonic start is valid")
    }
}

impl<Tag> PartialEq for MonotonicIdAllocator<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.exhausted == other.exhausted
    }
}

impl<Tag> Eq for MonotonicIdAllocator<Tag> {}

impl<Tag> fmt::Debug for MonotonicIdAllocator<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonotonicIdAllocator")
            .field("next", &self.next)
            .field("exhausted", &self.exhausted)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use static_assertions::assert_not_impl_any;

    enum ResourceTag {}
    enum OtherTag {}

    #[test]
    fn default_starts_at_one() {
        let allocator = MonotonicIdAllocator::<ResourceTag>::default();
        assert_eq!(allocator.next_raw(), 1);
        assert!(!allocator.is_exhausted());
    }

    #[test]
    fn new_rejects_zero_start() {
        let allocator = MonotonicIdAllocator::<ResourceTag>::new(0);
        assert_eq!(
            allocator,
            Err(AllocationError::InvalidStart { start_at: 0 })
        );
    }

    #[test]
    fn allocates_monotonic_ids() {
        let mut allocator =
            MonotonicIdAllocator::<ResourceTag>::new(5).expect("valid monotonic allocator");
        let a = allocator.allocate();
        let b = allocator.allocate();
        let c = allocator.allocate();

        assert_eq!(a.raw(), 5);
        assert_eq!(b.raw(), 6);
        assert_eq!(c.raw(), 7);
    }

    #[test]
    fn advance_to_at_least_no_op_when_lower_or_equal() {
        let mut allocator =
            MonotonicIdAllocator::<ResourceTag>::new(10).expect("valid monotonic allocator");
        allocator.advance_to_at_least(10);
        allocator.advance_to_at_least(3);
        assert_eq!(allocator.allocate().raw(), 10);
    }

    #[test]
    fn advance_to_at_least_moves_forward() {
        let mut allocator =
            MonotonicIdAllocator::<ResourceTag>::new(1).expect("valid monotonic allocator");
        allocator.advance_to_at_least(10);
        assert_eq!(allocator.allocate().raw(), 10);
    }

    #[test]
    fn advance_to_at_least_zero_is_rejected() {
        let mut allocator =
            MonotonicIdAllocator::<ResourceTag>::new(1).expect("valid monotonic allocator");
        assert_eq!(
            allocator.try_advance_to_at_least(0),
            Err(InvalidRawId::new(0))
        );
    }

    #[test]
    fn advance_to_at_least_can_move_to_reserved_max_and_exhaust() {
        let mut allocator =
            MonotonicIdAllocator::<ResourceTag>::new(1).expect("valid monotonic allocator");
        allocator.advance_to_at_least(u64::MAX);
        assert!(allocator.is_exhausted());
        assert_eq!(allocator.try_allocate(), Err(AllocationError::Exhausted));
    }

    #[test]
    fn try_allocate_returns_exhausted() {
        let mut allocator =
            MonotonicIdAllocator::<OtherTag>::new(u64::MAX - 1).expect("valid monotonic allocator");
        let last = allocator
            .try_allocate()
            .expect("last valid id before exhaustion");
        let exhausted = allocator.try_allocate();

        assert_eq!(last.raw(), u64::MAX - 1);
        assert_eq!(exhausted, Err(AllocationError::Exhausted));
    }

    #[test]
    #[should_panic(expected = "MonotonicIdAllocator exhausted")]
    fn allocate_panics_at_exhaustion() {
        let mut allocator =
            MonotonicIdAllocator::<OtherTag>::new(u64::MAX - 1).expect("valid monotonic allocator");
        let _ = allocator.allocate();
        let _ = allocator.allocate();
    }

    #[test]
    fn batch_allocates_contiguous_ids() {
        let mut allocator =
            MonotonicIdAllocator::<ResourceTag>::new(12).expect("valid monotonic allocator");
        let ids = allocator.allocate_batch::<3>();
        let raws = ids.map(TypedId::raw);
        assert_eq!(raws, [12, 13, 14]);
    }

    #[test]
    fn batch_returns_error_when_exhausted_midway() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(u64::MAX - 1)
            .expect("valid monotonic allocator");
        let result = allocator.try_allocate_batch::<2>();
        assert_eq!(result, Err(AllocationError::Exhausted));
        assert_eq!(allocator.next_raw(), u64::MAX - 1);
    }

    #[test]
    #[should_panic(expected = "MonotonicIdAllocator exhausted during batch allocation")]
    fn batch_panics_when_exhausted_midway() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(u64::MAX - 1)
            .expect("valid monotonic allocator");
        let _ = allocator.allocate_batch::<2>();
    }

    #[test]
    fn allocator_is_not_copy_or_clone() {
        assert_not_impl_any!(MonotonicIdAllocator<ResourceTag>: Copy, Clone);
    }

    proptest! {
        #[test]
        fn property_allocation_is_strictly_increasing(
            start in 1u64..1_000_000u64,
            count in 1usize..128usize
        ) {
            let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(start)
                .expect("generated start must be valid");
            let ids = (0..count)
                .map(|_| allocator.allocate().raw())
                .collect::<Vec<_>>();

            for pair in ids.windows(2) {
                prop_assert!(pair[0] < pair[1]);
            }
        }
    }
}
