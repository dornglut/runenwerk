use anyhow::Result;
use ecs::World;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SimulationTick(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimulationProfile {
    LocalSinglePlayer,
    DeterministicLockstep,
    RollbackSession,
    DedicatedAuthority,
    HighThroughputAuthority,
}

impl Default for SimulationProfile {
    fn default() -> Self {
        Self::LocalSinglePlayer
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthorityRole {
    Local,
    Client,
    Server,
    Peer,
}

impl Default for AuthorityRole {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeterminismLevel {
    Strict,
    Validated,
    BestEffort,
}

impl Default for DeterminismLevel {
    fn default() -> Self {
        Self::Validated
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimulationSessionId(pub u64);

impl Default for SimulationSessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationSessionId {
    pub fn new() -> Self {
        Self(NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimulationSeed(pub u64);

impl Default for SimulationSeed {
    fn default() -> Self {
        Self(0xC0DE_5EED_D15C_A11E)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationRng {
    state: u64,
    generated: u64,
}

impl Default for SimulationRng {
    fn default() -> Self {
        Self::from_seed(SimulationSeed::default())
    }
}

impl SimulationRng {
    pub fn from_seed(seed: SimulationSeed) -> Self {
        let state = if seed.0 == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed.0
        };
        Self {
            state,
            generated: 0,
        }
    }

    pub fn reseed(&mut self, seed: SimulationSeed) {
        *self = Self::from_seed(seed);
    }

    pub fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        self.generated = self.generated.saturating_add(1);
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        bits as f32 / (1u32 << 24) as f32
    }

    pub fn generated(&self) -> u64 {
        self.generated
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandSource {
    LocalPlayer,
    RemotePlayer,
    AI,
    Server,
    System,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationCommandFrame<C> {
    pub tick: SimulationTick,
    pub commands: Vec<C>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimulationHash(pub [u8; 32]);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationProfileConfig {
    pub profile: SimulationProfile,
    pub authority: AuthorityRole,
    pub determinism: DeterminismLevel,
}

impl Default for SimulationProfileConfig {
    fn default() -> Self {
        Self {
            profile: SimulationProfile::LocalSinglePlayer,
            authority: AuthorityRole::Local,
            determinism: DeterminismLevel::Validated,
        }
    }
}

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

pub trait WorldSimulationCodec {
    type Context;
    type Snapshot: Serialize + DeserializeOwned + Clone + Send + Sync + 'static;

    fn codec_id() -> &'static str;

    fn codec_version() -> u32 {
        1
    }

    fn capture_snapshot(world: &World, ctx: &Self::Context) -> Result<Self::Snapshot>;
    fn restore_snapshot(
        world: &mut World,
        ctx: &mut Self::Context,
        snapshot: &Self::Snapshot,
    ) -> Result<()>;

    fn hash_snapshot(snapshot: &Self::Snapshot) -> Result<SimulationHash> {
        let bytes = postcard::to_allocvec(snapshot)?;
        Ok(SimulationHash(*blake3::hash(&bytes).as_bytes()))
    }
}
