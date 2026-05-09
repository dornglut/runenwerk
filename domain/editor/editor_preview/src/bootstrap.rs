use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const PREVIEW_BOOTSTRAP_PREFIX: &str = "RUNENWERK_RUNTIME_PREVIEW_BOOTSTRAP";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewBootstrap {
    pub endpoint: String,
    pub server_id: String,
    pub server_name: String,
    pub certificate_fingerprint_sha256: String,
    pub trusted_certificate_der_hex: String,
    pub join_ticket: String,
}

impl PreviewBootstrap {
    pub fn to_stdout_line(&self) -> Result<String, PreviewBootstrapParseError> {
        self.validate()?;
        let payload = postcard::to_allocvec(self)
            .map_err(|error| PreviewBootstrapParseError::Encode(error.to_string()))?;
        Ok(format!(
            "{PREVIEW_BOOTSTRAP_PREFIX} {}",
            encode_lower_hex(&payload)
        ))
    }

    pub fn parse_stdout_line(line: &str) -> Result<Self, PreviewBootstrapParseError> {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(PREVIEW_BOOTSTRAP_PREFIX) else {
            return Err(PreviewBootstrapParseError::MissingPrefix);
        };
        let payload = rest.trim();
        if payload.is_empty() {
            return Err(PreviewBootstrapParseError::MissingPayload);
        }
        let bytes = decode_lower_hex(payload).map_err(PreviewBootstrapParseError::MalformedHex)?;
        let bootstrap: Self = postcard::from_bytes(&bytes)
            .map_err(|error| PreviewBootstrapParseError::Decode(error.to_string()))?;
        bootstrap.validate()?;
        Ok(bootstrap)
    }

    fn validate(&self) -> Result<(), PreviewBootstrapParseError> {
        if self.endpoint.trim().is_empty() {
            return Err(PreviewBootstrapParseError::EmptyField("endpoint"));
        }
        if self.server_id.trim().is_empty() {
            return Err(PreviewBootstrapParseError::EmptyField("server_id"));
        }
        if self.server_name.trim().is_empty() {
            return Err(PreviewBootstrapParseError::EmptyField("server_name"));
        }
        if self.certificate_fingerprint_sha256.trim().is_empty() {
            return Err(PreviewBootstrapParseError::EmptyField(
                "certificate_fingerprint_sha256",
            ));
        }
        if self.trusted_certificate_der_hex.trim().is_empty() {
            return Err(PreviewBootstrapParseError::EmptyField(
                "trusted_certificate_der_hex",
            ));
        }
        if self.join_ticket.trim().is_empty() {
            return Err(PreviewBootstrapParseError::EmptyField("join_ticket"));
        }
        Ok(())
    }
}

pub fn encode_lower_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn decode_lower_hex(value: &str) -> Result<Vec<u8>, PreviewHexError> {
    if !value.len().is_multiple_of(2) {
        return Err(PreviewHexError::OddLength);
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = hex_value(pair[0])?;
        let low = hex_value(pair[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_value(byte: u8) -> Result<u8, PreviewHexError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(PreviewHexError::InvalidDigit),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewHexError {
    OddLength,
    InvalidDigit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewBootstrapParseError {
    MissingPrefix,
    MissingPayload,
    EmptyField(&'static str),
    MalformedHex(PreviewHexError),
    Encode(String),
    Decode(String),
}

impl Display for PreviewHexError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OddLength => write!(formatter, "hex value has odd length"),
            Self::InvalidDigit => write!(formatter, "hex value contains a non-hex digit"),
        }
    }
}

impl Display for PreviewBootstrapParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPrefix => write!(formatter, "preview bootstrap line is missing prefix"),
            Self::MissingPayload => write!(formatter, "preview bootstrap line is missing payload"),
            Self::EmptyField(field) => {
                write!(formatter, "preview bootstrap field is empty: {field}")
            }
            Self::MalformedHex(error) => {
                write!(
                    formatter,
                    "preview bootstrap payload hex is malformed: {error}"
                )
            }
            Self::Encode(error) => write!(formatter, "preview bootstrap encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "preview bootstrap decode failed: {error}"),
        }
    }
}

impl std::error::Error for PreviewHexError {}
impl std::error::Error for PreviewBootstrapParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_line_round_trips() {
        let bootstrap = PreviewBootstrap {
            endpoint: "127.0.0.1:7777".to_string(),
            server_id: "srv".to_string(),
            server_name: "preview.local".to_string(),
            certificate_fingerprint_sha256: "abc".to_string(),
            trusted_certificate_der_hex: "010203".to_string(),
            join_ticket: "ticket".to_string(),
        };

        let line = bootstrap
            .to_stdout_line()
            .expect("bootstrap line should encode");
        let decoded =
            PreviewBootstrap::parse_stdout_line(&line).expect("bootstrap line should decode");

        assert_eq!(decoded, bootstrap);
    }

    #[test]
    fn malformed_bootstrap_lines_are_rejected() {
        assert!(matches!(
            PreviewBootstrap::parse_stdout_line("wrong"),
            Err(PreviewBootstrapParseError::MissingPrefix)
        ));
        assert!(matches!(
            PreviewBootstrap::parse_stdout_line(PREVIEW_BOOTSTRAP_PREFIX),
            Err(PreviewBootstrapParseError::MissingPayload)
        ));
        assert!(matches!(
            PreviewBootstrap::parse_stdout_line("RUNENWERK_RUNTIME_PREVIEW_BOOTSTRAP not-hex"),
            Err(PreviewBootstrapParseError::MalformedHex(_))
        ));
    }
}
