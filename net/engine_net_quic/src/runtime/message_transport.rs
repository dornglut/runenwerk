use anyhow::{Context, Result};
use engine_net::{MessageEnvelope, encode_message};
use quinn::Connection;

use crate::read_message;
use crate::write_message;

pub fn should_send_via_datagram(payload_len: usize, max_datagram_size: Option<usize>) -> bool {
    max_datagram_size.is_some_and(|budget| payload_len <= budget)
}

pub async fn send_envelope(connection: &Connection, envelope: &MessageEnvelope) -> Result<()> {
    let encoded = encode_message(envelope)?;
    if should_send_via_datagram(encoded.len(), connection.max_datagram_size()) {
        connection
            .send_datagram(encoded.into())
            .context("send datagram envelope")?;
        return Ok(());
    }

    let mut stream = connection
        .open_uni()
        .await
        .context("open envelope stream")?;
    write_message(&mut stream, envelope)
        .await
        .context("write envelope stream")?;
    stream.finish().context("finish envelope stream")?;
    Ok(())
}

pub async fn receive_stream_envelope(connection: &Connection) -> Result<Option<MessageEnvelope>> {
    let mut stream = connection
        .accept_uni()
        .await
        .context("accept envelope stream")?;
    read_message(&mut stream).await
}

#[cfg(test)]
mod tests {
    use super::should_send_via_datagram;

    #[test]
    fn datagram_budget_controls_transport_choice() {
        assert!(should_send_via_datagram(512, Some(1200)));
        assert!(!should_send_via_datagram(1300, Some(1200)));
        assert!(!should_send_via_datagram(128, None));
    }
}
