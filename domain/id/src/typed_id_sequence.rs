use core::fmt;
use core::marker::PhantomData;

use crate::TypedId;

#[repr(transparent)]
pub struct TypedIdSequence<Tag> {
	next: u64,
	_marker: PhantomData<fn() -> Tag>,
}

impl<Tag> TypedIdSequence<Tag> {
	pub const fn new(start_at: u64) -> Self {
		Self {
			next: start_at,
			_marker: PhantomData,
		}
	}

	pub const fn next_raw(&self) -> u64 {
		self.next
	}

	pub fn allocate(&mut self) -> TypedId<Tag> {
		let id = self.next;
		self.next = self.next.saturating_add(1);
		TypedId::new(id)
	}

	pub fn next_id(&mut self) -> TypedId<Tag> {
		self.allocate()
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

impl<Tag> Default for TypedIdSequence<Tag> {
	fn default() -> Self {
		Self::new(1)
	}
}

impl<Tag> Copy for TypedIdSequence<Tag> {}

impl<Tag> Clone for TypedIdSequence<Tag> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<Tag> PartialEq for TypedIdSequence<Tag> {
	fn eq(&self, other: &Self) -> bool {
		self.next == other.next
	}
}

impl<Tag> Eq for TypedIdSequence<Tag> {}

impl<Tag> fmt::Debug for TypedIdSequence<Tag> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TypedIdSequence")
			.field("next", &self.next)
			.finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	enum ResourceTag {}
	enum AnotherTag {}

	#[test]
	fn default_sequence_starts_at_one() {
		let sequence = TypedIdSequence::<ResourceTag>::default();
		assert_eq!(sequence.next_raw(), 1);
	}

	#[test]
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
	fn sequence_can_advance_forward() {
		let mut sequence = TypedIdSequence::<ResourceTag>::new(1);
		sequence.advance_to_at_least(10);

		let next = sequence.allocate();
		assert_eq!(next.raw(), 10);
	}

	#[test]
	fn advance_to_at_least_does_not_move_backward() {
		let mut sequence = TypedIdSequence::<ResourceTag>::new(10);
		sequence.advance_to_at_least(3);

		let next = sequence.allocate();
		assert_eq!(next.raw(), 10);
	}

	#[test]
	fn sequence_clone_and_copy_do_not_require_tag_traits() {
		let sequence = TypedIdSequence::<AnotherTag>::new(7);
		let copied = sequence;
		let cloned = sequence.clone();

		assert_eq!(copied.next_raw(), 7);
		assert_eq!(cloned.next_raw(), 7);
	}
}