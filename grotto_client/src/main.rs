use anyhow::Result;
use engine_net::{ProtocolVersion, SimulationRole};

fn main() -> Result<()> {
    let role = SimulationRole::Client;
    let protocol = ProtocolVersion::new(1, 1, 1);
    println!("grotto_client bootstrap role={role:?} protocol={protocol:?}");
    Ok(())
}
