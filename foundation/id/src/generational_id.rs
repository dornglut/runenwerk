use core::fmt;
use core::marker::PhantomData;

/// Packed generational handle value.
///
/// Bit layout (stable):
/// - low 32 bits: slot
/// - high 32 bits: generation
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
        serializer.serialize_u64(self.raw())
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
    fn has_no_default() {
        assert_not_impl_any!(GenerationalId<EntityTag>: Default);
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
