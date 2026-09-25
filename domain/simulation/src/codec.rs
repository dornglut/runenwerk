use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::SimulationHash;

pub trait SimulationCodec {
    type Host;
    type Snapshot: Serialize + DeserializeOwned + Clone + Send + Sync + 'static;

    fn codec_id() -> &'static str;

    fn codec_version() -> u32 {
        1
    }

    fn capture(host: &Self::Host) -> Result<Self::Snapshot>;
    fn restore(host: &mut Self::Host, snapshot: &Self::Snapshot) -> Result<()>;

    fn hash(snapshot: &Self::Snapshot) -> Result<SimulationHash> {
        let bytes = postcard::to_allocvec(snapshot)?;
        Ok(SimulationHash(*blake3::hash(&bytes).as_bytes()))
    }
}
