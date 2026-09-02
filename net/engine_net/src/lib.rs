extern crate self as engine_net;

pub mod prelude;
pub mod protocol;
pub mod replication;
pub mod simulation;

pub use engine_net_macros::{net_component, net_entity};
pub use engine_sim::{
    ActorId, AuthorityRole, CommandSource, DeterminismLevel, NetEntityId, SimulationCodec,
    SimulationCommandFrame, SimulationHash, SimulationProfile, SimulationProfileConfig,
    SimulationRng, SimulationSeed, SimulationSessionId, SimulationTick, WorldSimulationCodec,
};

// Re-exports retained only for the remaining live replication/prediction migration surface.
pub use protocol::*;
pub use replication::*;
pub use simulation::*;
