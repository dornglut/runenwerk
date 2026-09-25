use crate::{ReplayCheckpoint, ReplayHeader, ReplayJournalFrame};
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayArchive<S, C> {
    pub header: ReplayHeader,
    pub checkpoints: Vec<ReplayCheckpoint<S>>,
    pub journal: Vec<ReplayJournalFrame<C>>,
}

impl<S, C> ReplayArchive<S, C>
where
    S: Serialize + DeserializeOwned + Clone,
    C: Serialize + DeserializeOwned + Clone,
{
    pub fn encode_compressed(&self) -> Result<Vec<u8>> {
        let bytes = postcard::to_allocvec(self)?;
        Ok(zstd::stream::encode_all(bytes.as_slice(), 3)?)
    }

    pub fn decode_compressed(bytes: &[u8]) -> Result<Self> {
        let decompressed = zstd::stream::decode_all(bytes)?;
        Ok(postcard::from_bytes(&decompressed)?)
    }
}
