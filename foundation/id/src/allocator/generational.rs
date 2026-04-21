use core::fmt;
use core::marker::PhantomData;

#[repr(transparent)]
pub struct GenerationalId<Tag> {
    raw: u64,
    _marker: PhantomData<fn() -> Tag>,
}

impl<Tag> GenerationalId<Tag> {
    pub const fn from_parts(slot: u32, generation: u32) -> Self {
        let raw = ((generation as u64) << 32) | (slot as u64);
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub const fn slot(self) -> u32 {
        self.raw as u32
    }

    pub const fn generation(self) -> u32 {
        (self.raw >> 32) as u32
    }

    pub const fn raw(self) -> u64 {
        self.raw
    }
}

impl<Tag> Default for GenerationalId<Tag> {
    fn default() -> Self {
        Self::from_parts(0, 0)
    }
}

impl<Tag> Copy for GenerationalId<Tag> {}

impl<Tag> Clone for GenerationalId<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Tag> PartialEq for GenerationalId<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<Tag> Eq for GenerationalId<Tag> {}

impl<Tag> PartialOrd for GenerationalId<Tag> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Tag> Ord for GenerationalId<Tag> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<Tag> core::hash::Hash for GenerationalId<Tag> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<Tag> fmt::Debug for GenerationalId<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationalId")
            .field("slot", &self.slot())
            .field("generation", &self.generation())
            .finish()
    }
}

impl<Tag> From<u64> for GenerationalId<Tag> {
    fn from(value: u64) -> Self {
        Self::from_raw(value)
    }
}

impl<Tag> From<GenerationalId<Tag>> for u64 {
    fn from(value: GenerationalId<Tag>) -> Self {
        value.raw()
    }
}

#[cfg(feature = "serde")]
impl<Tag> serde::Serialize for GenerationalId<Tag> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.raw)
    }
}

#[cfg(feature = "serde")]
impl<'de, Tag> serde::Deserialize<'de> for GenerationalId<Tag> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        Ok(Self::from_raw(raw))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SlotState {
    Live,
    Free,
    Retired,
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
            _marker: PhantomData,
        }
    }

    pub fn try_allocate(&mut self) -> Option<GenerationalId<Tag>> {
        if let Some(slot) = self.free_slots.pop() {
            let index = slot as usize;
            if self.states.get(index) != Some(&SlotState::Free) {
                return None;
            }
            self.states[index] = SlotState::Live;
            self.live_count += 1;
            return Some(GenerationalId::from_parts(slot, self.generations[index]));
        }

        if self.next_slot > u32::MAX as u64 {
            return None;
        }

        let slot = self.next_slot as u32;
        self.next_slot += 1;
        self.generations.push(0);
        self.states.push(SlotState::Live);
        self.live_count += 1;
        Some(GenerationalId::from_parts(slot, 0))
    }

    pub fn allocate(&mut self) -> GenerationalId<Tag> {
        self.try_allocate()
            .expect("GenerationalIdAllocator exhausted available slots")
    }

    pub fn free(&mut self, id: GenerationalId<Tag>) -> bool {
        let slot = id.slot() as usize;
        let Some(generation) = self.generations.get_mut(slot) else {
            return false;
        };
        let Some(state) = self.states.get_mut(slot) else {
            return false;
        };
        if *state != SlotState::Live || *generation != id.generation() {
            return false;
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
            }
        }

        true
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
            .field("capacity_slots", &self.capacity_slots())
            .field("free_slots", &self.free_slots.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_not_impl_any;

    enum EntityTag {}

    #[test]
    fn packed_layout_roundtrip_is_stable() {
        let id = GenerationalId::<EntityTag>::from_parts(7, 9);
        assert_eq!(id.raw(), ((9u64) << 32) | 7u64);
        assert_eq!(id.slot(), 7);
        assert_eq!(id.generation(), 9);
    }

    #[test]
    fn allocates_and_reports_liveness() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let id = allocator.allocate();

        assert!(allocator.is_live(id));
        assert_eq!(allocator.live_count(), 1);
        assert_eq!(allocator.capacity_slots(), 1);
    }

    #[test]
    fn reuse_increments_generation_and_invalidates_stale() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let first = allocator.allocate();
        assert!(allocator.free(first));

        let reused = allocator.allocate();
        assert_eq!(reused.slot(), first.slot());
        assert_eq!(reused.generation(), first.generation() + 1);
        assert!(!allocator.is_live(first));
        assert!(allocator.is_live(reused));
    }

    #[test]
    fn free_returns_false_for_stale_or_unknown_ids() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let first = allocator.allocate();
        assert!(allocator.free(first));
        assert!(!allocator.free(first));
        assert!(!allocator.free(GenerationalId::from_parts(99, 0)));
    }

    #[test]
    fn generation_overflow_retires_slot() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let id = allocator.allocate();

        allocator.generations[id.slot() as usize] = u32::MAX;
        assert!(allocator.free(GenerationalId::from_parts(id.slot(), u32::MAX)));

        assert!(!allocator.is_live(GenerationalId::from_parts(id.slot(), u32::MAX)));
        assert!(allocator.free_slots.is_empty());
        assert_eq!(allocator.states[id.slot() as usize], SlotState::Retired);
    }

    #[test]
    fn exhausted_when_no_free_and_no_new_slots() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        let id = allocator.allocate();

        allocator.generations[id.slot() as usize] = u32::MAX;
        assert!(allocator.free(GenerationalId::from_parts(id.slot(), u32::MAX)));
        allocator.next_slot = (u32::MAX as u64) + 1;

        assert!(allocator.try_allocate().is_none());
    }

    #[test]
    #[should_panic(expected = "GenerationalIdAllocator exhausted available slots")]
    fn allocate_panics_when_exhausted() {
        let mut allocator = GenerationalIdAllocator::<EntityTag>::new();
        allocator.next_slot = (u32::MAX as u64) + 1;
        let _ = allocator.allocate();
    }

    #[test]
    fn allocator_is_not_copy_or_clone() {
        assert_not_impl_any!(GenerationalIdAllocator<EntityTag>: Copy, Clone);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_uses_packed_u64() {
        let id = GenerationalId::<EntityTag>::from_parts(123, 456);
        let encoded = serde_json::to_string(&id).expect("serialize generational id");
        assert_eq!(encoded, id.raw().to_string());

        let decoded: GenerationalId<EntityTag> =
            serde_json::from_str(&encoded).expect("deserialize generational id");
        assert_eq!(decoded, id);
    }
}
