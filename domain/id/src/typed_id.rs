use core::fmt;
use core::marker::PhantomData;

use crate::IdTag;

#[repr(transparent)]
pub struct TypedId<Tag> {
    raw: u64,
    _marker: PhantomData<fn() -> Tag>,
}

impl<Tag> TypedId<Tag> {
    pub const fn new(raw: u64) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub const fn raw(self) -> u64 {
        self.raw
    }
}

impl<Tag> Default for TypedId<Tag> {
    fn default() -> Self {
        Self::new(0)
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

impl<Tag: IdTag> fmt::Debug for TypedId<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", Tag::DEBUG_NAME, self.raw)
    }
}

impl<Tag> fmt::Display for TypedId<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<Tag> From<u64> for TypedId<Tag> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<Tag> From<TypedId<Tag>> for u64 {
    fn from(value: TypedId<Tag>) -> Self {
        value.raw
    }
}

#[cfg(feature = "serde")]
impl<Tag> serde::Serialize for TypedId<Tag> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.raw)
    }
}

#[cfg(feature = "serde")]
impl<'de, Tag> serde::Deserialize<'de> for TypedId<Tag> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum UserTag {}
    enum OrderTag {}

    impl IdTag for UserTag {
        const DEBUG_NAME: &'static str = "UserId";
    }

    impl IdTag for OrderTag {
        const DEBUG_NAME: &'static str = "OrderId";
    }

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

        let restored = TypedId::<UserTag>::from(raw);
        assert_eq!(restored.raw(), 42);
    }

    #[test]
    fn debug_uses_tag_name() {
        let id = TypedId::<OrderTag>::new(7);
        assert_eq!(format!("{id:?}"), "OrderId(7)");
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
}
