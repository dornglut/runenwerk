use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFrame {
    pub tick: SimulationTick,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputCommandEnvelope {
    pub tick: SimulationTick,
    pub sequence: u32,
    pub payload: Vec<u8>,
}

pub fn encode_input_command(payload: &InputCommandEnvelope) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(payload)
}

pub fn decode_input_command(bytes: &[u8]) -> Result<InputCommandEnvelope, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::{InputCommandEnvelope, decode_input_command, encode_input_command};
    use engine_sim::SimulationTick;

    #[test]
    fn input_command_round_trip() {
        let command = InputCommandEnvelope {
            tick: SimulationTick(11),
            sequence: 7,
            payload: vec![1, 2, 3],
        };
        let encoded = encode_input_command(&command).expect("input command should encode");
        let decoded = decode_input_command(&encoded).expect("input command should decode");
        assert_eq!(decoded, command);
    }
}
