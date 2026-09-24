use id_macros::id;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[id]
pub struct AssetId;

#[id]
pub struct AssetSourceId;

#[id]
pub struct AssetSourceRootId;

#[id]
pub struct AssetArtifactId;

#[id]
pub struct ImportJobId;

#[id]
pub struct AssetRevisionId;

#[id]
pub struct AssetSourceRevisionId;

#[id]
pub struct AssetArtifactRevisionId;

pub const fn asset_id(raw: u64) -> AssetId {
    match AssetId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset id constants must be non-zero"),
    }
}

pub const fn asset_source_id(raw: u64) -> AssetSourceId {
    match AssetSourceId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset source id constants must be non-zero"),
    }
}

pub const fn asset_source_root_id(raw: u64) -> AssetSourceRootId {
    match AssetSourceRootId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset source root id constants must be non-zero"),
    }
}

pub const fn asset_artifact_id(raw: u64) -> AssetArtifactId {
    match AssetArtifactId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset artifact id constants must be non-zero"),
    }
}

pub const fn import_job_id(raw: u64) -> ImportJobId {
    match ImportJobId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("import job id constants must be non-zero"),
    }
}

pub const fn asset_revision_id(raw: u64) -> AssetRevisionId {
    match AssetRevisionId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset revision id constants must be non-zero"),
    }
}

pub const fn asset_source_revision_id(raw: u64) -> AssetSourceRevisionId {
    match AssetSourceRevisionId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset source revision id constants must be non-zero"),
    }
}

pub const fn asset_artifact_revision_id(raw: u64) -> AssetArtifactRevisionId {
    match AssetArtifactRevisionId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("asset artifact revision id constants must be non-zero"),
    }
}

macro_rules! impl_id_serde {
    ($ty:ty, $label:literal) => {
        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_u64(self.raw())
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = u64::deserialize(deserializer)?;
                <$ty>::try_from_raw(raw).map_err(|_| {
                    serde::de::Error::custom(concat!($label, " must be a non-zero u64"))
                })
            }
        }
    };
}

impl_id_serde!(AssetId, "AssetId");
impl_id_serde!(AssetSourceId, "AssetSourceId");
impl_id_serde!(AssetSourceRootId, "AssetSourceRootId");
impl_id_serde!(AssetArtifactId, "AssetArtifactId");
impl_id_serde!(ImportJobId, "ImportJobId");
impl_id_serde!(AssetRevisionId, "AssetRevisionId");
impl_id_serde!(AssetSourceRevisionId, "AssetSourceRevisionId");
impl_id_serde!(AssetArtifactRevisionId, "AssetArtifactRevisionId");
