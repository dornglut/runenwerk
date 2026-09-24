use core::fmt;
use core::marker::PhantomData;

use crate::{AllocationError, TypedId};

/// Monotonic allocator for tag-typed IDs.
///
/// `u64::MAX` is reserved as an exhaustion sentinel and is never issued.
pub struct MonotonicIdAllocator<Tag> {
    next: u64,
    _marker: PhantomData<fn() -> Tag>,
}

impl<Tag> MonotonicIdAllocator<Tag> {
    /// Creates an allocator whose next issued raw ID is `start_at`.
    ///
    /// Use this for reviewed, trusted start values. Raw, imported, persisted,
    /// or otherwise untrusted seeds should use [`Self::try_new`] so invalid
    /// zero starts are surfaced as typed errors.
    ///
    /// # Panics
    ///
    /// Panics when `start_at` is `0`. `TypedId` raw values are non-zero, so a
    /// monotonic allocator cannot issue zero as its first ID.
    pub const fn new(start_at: u64) -> Self {
        assert!(
            start_at != 0,
            "MonotonicIdAllocator start value must be non-zero"
        );

        Self {
            next: start_at,
            _marker: PhantomData,
        }
    }

    /// Creates an allocator from a raw start value.
    ///
    /// This is the correct constructor for raw, imported, persisted, or
    /// otherwise untrusted seeds. It rejects `0` because `TypedId` raw values
    /// are non-zero.
    pub const fn try_new(start_at: u64) -> Result<Self, AllocationError> {
        if start_at == 0 {
            return Err(AllocationError::InvalidStart { start_at });
        }

        Ok(Self {
            next: start_at,
            _marker: PhantomData,
        })
    }

    pub const fn next_raw(&self) -> u64 {
        self.next
    }

    pub fn try_allocate(&mut self) -> Option<TypedId<Tag>> {
        if self.next == u64::MAX {
            return None;
        }

        let id = self.next;
        self.next += 1;
        Some(TypedId::new(id))
    }

    pub fn allocate(&mut self) -> TypedId<Tag> {
        self.try_allocate()
            .expect("MonotonicIdAllocator exhausted at u64::MAX")
    }

    pub fn allocate_batch<const N: usize>(&mut self) -> [TypedId<Tag>; N] {
        core::array::from_fn(|_| self.allocate())
    }

    pub fn advance_to_at_least(&mut self, minimum_next: u64) {
        if self.next < minimum_next {
            self.next = minimum_next;
        }
    }
}

impl<Tag> Default for MonotonicIdAllocator<Tag> {
    fn default() -> Self {
        Self::new(1)
    }
}

impl<Tag> PartialEq for MonotonicIdAllocator<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}

impl<Tag> Eq for MonotonicIdAllocator<Tag> {}

impl<Tag> fmt::Debug for MonotonicIdAllocator<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonotonicIdAllocator")
            .field("next", &self.next)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_not_impl_any;

    enum ResourceTag {}
    enum OtherTag {}

    #[test]
    fn default_starts_at_one() {
        let allocator = MonotonicIdAllocator::<ResourceTag>::default();
        assert_eq!(allocator.next_raw(), 1);
    }

    #[test]
    fn try_new_rejects_zero_start() {
        assert_eq!(
            MonotonicIdAllocator::<ResourceTag>::try_new(0),
            Err(AllocationError::InvalidStart { start_at: 0 })
        );
    }

    #[test]
    #[should_panic(expected = "MonotonicIdAllocator start value must be non-zero")]
    fn new_panics_for_zero_start() {
        let _ = MonotonicIdAllocator::<ResourceTag>::new(0);
    }

    #[test]
    fn allocates_monotonic_ids() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(5);
        let a = allocator.allocate();
        let b = allocator.allocate();
        let c = allocator.allocate();

        assert_eq!(a.raw(), 5);
        assert_eq!(b.raw(), 6);
        assert_eq!(c.raw(), 7);
    }

    #[test]
    fn advance_to_at_least_no_op_when_lower_or_equal() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(10);
        allocator.advance_to_at_least(10);
        allocator.advance_to_at_least(3);
        assert_eq!(allocator.allocate().raw(), 10);
    }

    #[test]
    fn advance_to_at_least_moves_forward() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(1);
        allocator.advance_to_at_least(10);
        assert_eq!(allocator.allocate().raw(), 10);
    }

    #[test]
    fn advance_to_at_least_can_move_to_reserved_max_and_exhaust() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(1);
        allocator.advance_to_at_least(u64::MAX);
        assert!(allocator.try_allocate().is_none());
    }

    #[test]
    fn try_allocate_returns_none_at_exhaustion() {
        let mut allocator = MonotonicIdAllocator::<OtherTag>::new(u64::MAX - 1);
        let last = allocator.try_allocate();
        let exhausted = allocator.try_allocate();

        assert_eq!(last.map(TypedId::raw), Some(u64::MAX - 1));
        assert!(exhausted.is_none());
    }

    #[test]
    #[should_panic(expected = "MonotonicIdAllocator exhausted at u64::MAX")]
    fn allocate_panics_at_exhaustion() {
        let mut allocator = MonotonicIdAllocator::<OtherTag>::new(u64::MAX - 1);
        let _ = allocator.allocate();
        let _ = allocator.allocate();
    }

    #[test]
    fn batch_allocates_contiguous_ids() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(12);
        let ids = allocator.allocate_batch::<3>();
        let raws = ids.map(TypedId::raw);
        assert_eq!(raws, [12, 13, 14]);
    }

    #[test]
    #[should_panic(expected = "MonotonicIdAllocator exhausted at u64::MAX")]
    fn batch_panics_when_exhausted_midway() {
        let mut allocator = MonotonicIdAllocator::<ResourceTag>::new(u64::MAX - 1);
        let _ = allocator.allocate_batch::<2>();
    }

    #[test]
    fn allocator_is_not_copy_or_clone() {
        assert_not_impl_any!(MonotonicIdAllocator<ResourceTag>: Copy, Clone);
    }
}
