use editor_preview::{PreviewBootstrap, PreviewHexError, decode_lower_hex};
use rustls::pki_types::CertificateDer;
use std::fmt::{Display, Formatter};

pub fn trusted_certificate_from_bootstrap(
    bootstrap: &PreviewBootstrap,
) -> Result<CertificateDer<'static>, PreviewBootstrapCertificateError> {
    let bytes = decode_lower_hex(&bootstrap.trusted_certificate_der_hex)?;
    Ok(CertificateDer::from(bytes))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewBootstrapCertificateError {
    Hex(PreviewHexError),
}

impl Display for PreviewBootstrapCertificateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex(error) => write!(formatter, "certificate hex is malformed: {error}"),
        }
    }
}

impl std::error::Error for PreviewBootstrapCertificateError {}

impl From<PreviewHexError> for PreviewBootstrapCertificateError {
    fn from(value: PreviewHexError) -> Self {
        Self::Hex(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_trusted_certificate_from_bootstrap_hex() {
        let bootstrap = PreviewBootstrap {
            endpoint: "127.0.0.1:7777".to_string(),
            server_id: "srv".to_string(),
            server_name: "preview.local".to_string(),
            certificate_fingerprint_sha256: "abc".to_string(),
            trusted_certificate_der_hex: "00010aFf".to_string(),
            join_ticket: "ticket".to_string(),
        };

        let certificate =
            trusted_certificate_from_bootstrap(&bootstrap).expect("certificate hex should decode");

        assert_eq!(certificate.as_ref(), &[0, 1, 10, 255]);
    }
}
