use rustls::pki_types::CertificateDer;
use sha2::{Digest, Sha256};

use crate::runtime::helpers::hex_digit;

pub fn certificate_fingerprint_sha256(cert: &CertificateDer<'_>) -> String {
    let digest = Sha256::digest(cert.as_ref());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}
