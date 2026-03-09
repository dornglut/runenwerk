use anyhow::{Result, anyhow};
use engine_net::ClientSessionTarget;
use rustls::pki_types::CertificateDer;

use crate::certificate_fingerprint_sha256;

#[derive(Debug, Clone)]
pub enum QuicTrustPolicy {
    DirectRoots(Vec<CertificateDer<'static>>),
    PinnedServer {
        expected_fingerprint_sha256: String,
        trusted_certificates: Vec<CertificateDer<'static>>,
    },
}

impl QuicTrustPolicy {
    pub(crate) fn retargeted_for(&self, target: &ClientSessionTarget) -> Self {
        match self {
            Self::DirectRoots(certificates) => Self::DirectRoots(certificates.clone()),
            Self::PinnedServer {
                trusted_certificates,
                ..
            } => Self::PinnedServer {
                expected_fingerprint_sha256: target.server_cert_fingerprint_sha256.clone(),
                trusted_certificates: trusted_certificates.clone(),
            },
        }
    }

    pub(crate) fn trusted_certificates(&self) -> Result<Vec<CertificateDer<'static>>> {
        let certificates = match self {
            Self::DirectRoots(certificates) => certificates.clone(),
            Self::PinnedServer {
                trusted_certificates,
                ..
            } => trusted_certificates.clone(),
        };
        if certificates.is_empty() {
            return Err(anyhow!(
                "QUIC trust policy requires at least one trusted certificate"
            ));
        }
        Ok(certificates)
    }

    pub(crate) fn validate_expected_fingerprint(&self) -> Result<()> {
        if let Self::PinnedServer {
            expected_fingerprint_sha256,
            trusted_certificates,
        } = self
        {
            let matches = trusted_certificates.iter().any(|certificate| {
                certificate_fingerprint_sha256(certificate) == *expected_fingerprint_sha256
            });
            if !matches {
                return Err(anyhow!(
                    "trusted server certificate does not match the expected fingerprint"
                ));
            }
        }
        Ok(())
    }
}
