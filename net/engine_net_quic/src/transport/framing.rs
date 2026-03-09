use anyhow::Result;
use engine_net::{MessageEnvelope, decode_message, encode_message};
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn write_message(stream: &mut SendStream, envelope: &MessageEnvelope) -> Result<()> {
    let bytes = encode_message(envelope)?;
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

pub async fn read_message(stream: &mut RecvStream) -> Result<Option<MessageEnvelope>> {
    let length = match stream.read_u32().await {
        Ok(length) => length,
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = vec![0u8; length as usize];
    stream.read_exact(&mut bytes).await?;
    Ok(Some(decode_message(&bytes)?))
}
