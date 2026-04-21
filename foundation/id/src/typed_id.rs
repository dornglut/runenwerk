use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroU64;

use crate::InvalidRawId;

#[repr(transparent)]
pub struct TypedId<Tag> {
    raw: NonZeroU64,
    _marker: PhantomData<fn() -> Tag>,
}

impl<Tag> TypedId<Tag> {
    pub const fn new(raw: u64) -> Self {
        match NonZeroU64::new(raw) {
            Some(raw) => Self {
                raw,
                _marker: PhantomData,
            },
            None => panic!("TypedId raw value must be non-zero"),
        }
    }

    pub const fn from_non_zero(raw: NonZeroU64) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub const fn try_from_raw(raw: u64) -> Result<Self, InvalidRawId> {
        match NonZeroU64::new(raw) {
            Some(raw) => Ok(Self::from_non_zero(raw)),
            None => Err(InvalidRawId::new(raw)),
        }
    }

    pub const fn raw(self) -> u64 {
        self.raw.get()
    }

    pub const fn raw_non_zero(self) -> NonZeroU64 {
        self.raw
    }
}

impl<Tag> Copy for TypedId<Tag> {}

impl<Tag> Clone for TypedId<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Tag> PartialEq for TypedId<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<Tag> Eq for TypedId<Tag> {}

impl<Tag> PartialOrd for TypedId<Tag> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Tag> Ord for TypedId<Tag> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<Tag> core::hash::Hash for TypedId<Tag> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<Tag> fmt::Debug for TypedId<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypedId({})", self.raw)
    }
}

impl<Tag> fmt::Display for TypedId<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.get().fmt(f)
    }
}

impl<Tag> TryFrom<u64> for TypedId<Tag> {
    type Error = InvalidRawId;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_from_raw(value)
    }
}

impl<Tag> From<NonZeroU64> for TypedId<Tag> {
    fn from(value: NonZeroU64) -> Self {
        Self::from_non_zero(value)
    }
}

impl<Tag> From<TypedId<Tag>> for u64 {
    fn from(value: TypedId<Tag>) -> Self {
        value.raw()
    }
}

#[cfg(feature = "serde")]
impl<Tag> serde::Serialize for TypedId<Tag> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.raw())
    }
}

#[cfg(feature = "serde")]
impl<'de, Tag> serde::Deserialize<'de> for TypedId<Tag> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        Self::try_from_raw(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_not_impl_any;

    enum UserTag {}

    #[test]
    fn typed_ids_compare_by_raw_value() {
        let a = TypedId::<UserTag>::new(1);
        let b = TypedId::<UserTag>::new(2);

        assert!(a < b);
        assert_eq!(a.raw(), 1);
        assert_eq!(b.raw(), 2);
    }

    #[test]
    fn typed_ids_roundtrip_through_u64() {
        let id = TypedId::<UserTag>::new(42);
        let raw: u64 = id.into();

        assert_eq!(raw, 42);

        let restored = TypedId::<UserTag>::try_from(raw).expect("valid typed id");
        assert_eq!(restored.raw(), 42);
    }

    #[test]
    fn try_from_zero_is_rejected() {
        let result = TypedId::<UserTag>::try_from(0);
        assert_eq!(result, Err(InvalidRawId::new(0)));
    }

    #[test]
    #[should_panic(expected = "TypedId raw value must be non-zero")]
    fn new_panics_for_zero() {
        let _ = TypedId::<UserTag>::new(0);
    }

    #[test]
    fn typed_id_has_no_default() {
        assert_not_impl_any!(TypedId<UserTag>: Default);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_is_stable_u64() {
        let id = TypedId::<UserTag>::new(42);
        let encoded = serde_json::to_string(&id).expect("serialize typed id");
        assert_eq!(encoded, "42");

        let decoded: TypedId<UserTag> =
            serde_json::from_str(&encoded).expect("deserialize typed id");
        assert_eq!(decoded.raw(), 42);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_zero() {
        let decoded = serde_json::from_str::<TypedId<UserTag>>("0");
        assert!(decoded.is_err());
    }
}
