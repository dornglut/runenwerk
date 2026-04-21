use core::fmt;

use crate::{MonotonicIdAllocator, TypedId};

/// Legacy façade retained for migration compatibility.
#[deprecated(
    note = "Use MonotonicIdAllocator<Tag> directly; TypedIdSequence<Tag> is a legacy compatibility shim."
)]
pub struct TypedIdSequence<Tag> {
    allocator: MonotonicIdAllocator<Tag>,
}

#[allow(deprecated)]
impl<Tag> TypedIdSequence<Tag> {
    #[deprecated(note = "Use MonotonicIdAllocator::new")]
    pub fn new(start_at: u64) -> Self {
        Self {
            allocator: MonotonicIdAllocator::new(start_at)
                .expect("TypedIdSequence start_at must be non-zero"),
        }
    }

    #[deprecated(note = "Use MonotonicIdAllocator::next_raw")]
    pub fn next_raw(&self) -> u64 {
        self.allocator.next_raw()
    }

    #[deprecated(note = "Use MonotonicIdAllocator::allocate")]
    pub fn allocate(&mut self) -> TypedId<Tag> {
        self.allocator.allocate()
    }

    #[deprecated(note = "Use MonotonicIdAllocator::allocate")]
    pub fn next_id(&mut self) -> TypedId<Tag> {
        self.allocator.allocate()
    }

    #[deprecated(note = "Use MonotonicIdAllocator::try_allocate")]
    pub fn try_allocate(&mut self) -> Option<TypedId<Tag>> {
        self.allocator.try_allocate().ok()
    }

    #[deprecated(note = "Use MonotonicIdAllocator::allocate_batch")]
    pub fn allocate_batch<const N: usize>(&mut self) -> [TypedId<Tag>; N] {
        self.allocator.allocate_batch()
    }

    #[deprecated(note = "Use MonotonicIdAllocator::advance_to_at_least")]
    pub fn advance_to_at_least(&mut self, minimum_next: u64) {
        self.allocator.advance_to_at_least(minimum_next);
    }
}

#[allow(deprecated)]
impl<Tag> Default for TypedIdSequence<Tag> {
    fn default() -> Self {
        Self {
            allocator: MonotonicIdAllocator::default(),
        }
    }
}

#[allow(deprecated)]
impl<Tag> PartialEq for TypedIdSequence<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.allocator == other.allocator
    }
}

#[allow(deprecated)]
impl<Tag> Eq for TypedIdSequence<Tag> {}

#[allow(deprecated)]
impl<Tag> fmt::Debug for TypedIdSequence<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedIdSequence")
            .field("next", &self.allocator.next_raw())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum ResourceTag {}
    enum AnotherTag {}

    #[test]
    #[allow(deprecated)]
    fn default_sequence_starts_at_one() {
        let sequence = TypedIdSequence::<ResourceTag>::default();
        assert_eq!(sequence.next_raw(), 1);
    }

    #[test]
    #[allow(deprecated)]
    fn sequence_allocates_monotonic_ids() {
        let mut sequence = TypedIdSequence::<ResourceTag>::new(5);

        let a = sequence.allocate();
        let b = sequence.allocate();
        let c = sequence.allocate();

        assert_eq!(a.raw(), 5);
        assert_eq!(b.raw(), 6);
        assert_eq!(c.raw(), 7);
    }

    #[test]
    #[allow(deprecated)]
    fn sequence_can_advance_forward() {
        let mut sequence = TypedIdSequence::<ResourceTag>::new(1);
        sequence.advance_to_at_least(10);

        let next = sequence.allocate();
        assert_eq!(next.raw(), 10);
    }

    #[test]
    #[allow(deprecated)]
    fn advance_to_at_least_does_not_move_backward() {
        let mut sequence = TypedIdSequence::<ResourceTag>::new(10);
        sequence.advance_to_at_least(3);

        let next = sequence.allocate();
        assert_eq!(next.raw(), 10);
    }

    #[test]
    #[allow(deprecated)]
    fn try_allocate_returns_none_after_u64_max() {
        let mut sequence = TypedIdSequence::<AnotherTag>::new(u64::MAX - 1);
        let last = sequence.try_allocate();
        let exhausted = sequence.try_allocate();

        assert_eq!(last.map(TypedId::raw), Some(u64::MAX - 1));
        assert!(exhausted.is_none());
    }

    #[test]
    #[allow(deprecated)]
    #[should_panic(expected = "MonotonicIdAllocator exhausted")]
    fn allocate_panics_after_u64_max() {
        let mut sequence = TypedIdSequence::<AnotherTag>::new(u64::MAX - 1);
        let _ = sequence.allocate();
        let _ = sequence.allocate();
    }

    #[test]
    #[allow(deprecated)]
    fn try_allocate_maps_exhausted_to_none() {
        let mut sequence = TypedIdSequence::<AnotherTag>::new(u64::MAX - 1);
        let _ = sequence.allocate();
        assert_eq!(
            sequence.allocator.try_allocate(),
            Err(crate::AllocationError::Exhausted)
        );
        assert!(sequence.try_allocate().is_none());
    }
}
