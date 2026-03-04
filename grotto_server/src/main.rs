use anyhow::Result;
use engine_net::{ProtocolVersion, SimulationRole};

fn main() -> Result<()> {
    let role = SimulationRole::Server;
    let protocol = ProtocolVersion::new(1, 1, 1);
    println!("grotto_server bootstrap role={role:?} protocol={protocol:?}");
    Ok(())
}
