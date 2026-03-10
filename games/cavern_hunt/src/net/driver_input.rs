use ecs::World;
use engine_net::SimulationTick;
use engine_net::replication::InputDriver;

use super::driver::CavernReplicationDriver;

impl InputDriver for CavernReplicationDriver {
    fn receive_remote_input(
        _world: &mut World,
        _tick: SimulationTick,
        _input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn take_local_input(_world: &mut World) -> Result<Vec<Self::Input>, Self::Error> {
        Ok(Vec::new())
    }

    fn apply_input(_world: &mut World, _input: &[Self::Input]) -> Result<(), Self::Error> {
        Ok(())
    }
}
