//! HTTP signature creation and validation.
//!
//! See https://datatracker.ietf.org/doc/html/rfc7230#section-3.2.4.

use itertools::Itertools;
use openssl::{
    ec::EcKey,
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    sign::{Signer, Verifier},
};
use std::{collections::HashMap, fmt::Display, fs::File, io::Read, str::FromStr};

/// Signature keyId="foo", algorithm="rsa-sha256", headers="x-request-id tpp-redirect-uri digest foo psu-id", signature=
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignatureHeader {
    key_id: String,
    algorithm: String,
    headers: Vec<String>,
    signature: String,
}

impl SignatureHeader {
    /// Creates a new signature header value.
    pub fn new(key_id: String, algorithm: String, headers: Vec<String>, signature: String) -> Self {
        Self {
            key_id,
            algorithm,
            headers,
            signature,
        }
    }

    /// Get a reference to the signature's key id.
    #[must_use]
    pub fn key_id(&self) -> &str {
        self.key_id.as_ref()
    }

    /// Get a reference to the signature's algorithm.
    #[must_use]
    pub fn algorithm(&self) -> &str {
        self.algorithm.as_ref()
    }

    /// Get a reference to the signature's headers.
    #[must_use]
    pub fn headers(&self) -> &[String] {
        self.headers.as_ref()
    }

    /// Get a reference to the signature's signature.
    #[must_use]
    pub fn signature(&self) -> &str {
        self.signature.as_ref()
    }
}

impl Display for SignatureHeader {
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_id = &self.key_id;
        let algorithm = &self.algorithm;
        let headers: String = self
            .headers
            .iter()
            .map(|s| s.to_lowercase())
            .intersperse(" ".to_string())
            .collect();
        let signature = &self.signature;
        write!(
            f,
            r#"Signature keyId="{key_id}", algorithm="{algorithm}", headers="{headers}", signature="{signature}""#
        )
    }
}

/// An error that occurs during parsing of a signature header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignatureHeaderParseError;

impl FromStr for SignatureHeader {
    type Err = SignatureHeaderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let without_signature = s.trim_start_matches("Signature ");
        let mut map = HashMap::new();
        for kv in without_signature.split(", ") {
            let mut kv = kv.split('=');
            let key = kv.next().ok_or(SignatureHeaderParseError)?;
            let value = kv
                .next()
                .ok_or(SignatureHeaderParseError)?
                .trim_matches('"');
            map.insert(key, value);
        }

        let signature = SignatureHeader {
            key_id: map
                .get("keyId")
                .ok_or(SignatureHeaderParseError)?
                .to_string(),
            algorithm: map
                .get("algorithm")
                .ok_or(SignatureHeaderParseError)?
                .to_string(),
            headers: map
                .get("headers")
                .ok_or(SignatureHeaderParseError)?
                .split(' ')
                .map(|s| s.to_string())
                .collect(),
            signature: map
                .get("signature")
                .ok_or(SignatureHeaderParseError)?
                .to_string(),
        };

        Ok(signature)
    }
}

/// An error that happened while loading a key.
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum KeyLoadError {
    /// Key file does not exist.
    #[error("key file does not exist")]
    NoSuchFile,
    /// Failed to read key file.
    #[error("failed to read key file")]
    ReadError,
    /// The key file format is wrong.
    #[error("key file format is wrong")]
    KeyFormatError,
}

fn load_key(path: &str) -> Result<Vec<u8>, KeyLoadError> {
    let mut file = File::open(path).map_err(|_| KeyLoadError::NoSuchFile)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)
        .map_err(|_| KeyLoadError::ReadError)?;
    Ok(buf)
}

/// Loads the specified private key.
pub fn load_private_key(path: &str) -> Result<PKey<Private>, KeyLoadError> {
    let buf = load_key(path)?;
    let ec_key = EcKey::private_key_from_pem(&buf).map_err(|_| KeyLoadError::KeyFormatError)?;
    let key = PKey::from_ec_key(ec_key).map_err(|_| KeyLoadError::KeyFormatError)?;
    Ok(key)
}

/// Loads the specified public key.
pub fn load_public_key(path: &str) -> Result<PKey<Public>, KeyLoadError> {
    let buf = load_key(path)?;
    let ec_key = EcKey::public_key_from_pem(&buf).map_err(|_| KeyLoadError::KeyFormatError)?;
    let key = PKey::from_ec_key(ec_key).map_err(|_| KeyLoadError::KeyFormatError)?;
    Ok(key)
}

/// Failed to sign the message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("failed to sign message")]
pub struct SignError;

/// Creates the signature of the provided data.
pub fn sign(message: &[u8], private_key: PKey<Private>) -> Result<Vec<u8>, SignError> {
    let mut signer = Signer::new(MessageDigest::sha256(), &private_key).map_err(|_| SignError)?;
    signer.update(message).map_err(|_| SignError)?;
    let signature = signer.sign_to_vec().map_err(|_| SignError)?;
    Ok(signature)
}

/// Failed to sign the message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("failed to verify message")]
pub struct VerifyError;

/// Uses a signature to verify the provided data.
pub fn verify(
    message: &[u8],
    signature: &[u8],
    public_key: PKey<Public>,
) -> Result<bool, VerifyError> {
    let mut verifier =
        Verifier::new(MessageDigest::sha256(), &public_key).map_err(|_| VerifyError)?;
    verifier.update(message).map_err(|_| VerifyError)?;
    let verified = verifier.verify(signature).map_err(|_| VerifyError)?;
    Ok(verified)
}

/// Compute the signature string used to create a signature.
#[allow(unstable_name_collisions)]
pub fn signature_string(header_order: &[&str], headers: &HashMap<&str, Vec<&str>>) -> String {
    header_order
        .iter()
        .filter_map(|k| headers.get_key_value(k))
        .map(|(k, vs)| {
            let vs_str: String = vs
                .iter()
                .map(|s| s.trim().to_string())
                .intersperse(", ".to_string())
                .collect();
            format!("{}: {}", k.to_lowercase(), vs_str)
        })
        .intersperse("\n".to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        load_private_key, load_public_key, sign, signature_string, verify, SignatureHeader,
    };
    use std::collections::HashMap;

    #[test]
    fn signature_string_works() {
        let mut headers = HashMap::new();
        headers.insert("(request-target)", vec!["get /foo"]);
        headers.insert("host", vec!["example.org"]);
        headers.insert("date", vec!["Tue, 07 Jun 2014 20:51:35 GMT"]);
        headers.insert("x-example", vec!["Example header with some whitespace."]);
        headers.insert("cache-control", vec!["max-age=60", "must-revalidate"]);
        let signature_string = signature_string(
            &[
                "(request-target)",
                "host",
                "date",
                "cache-control",
                "x-example",
            ],
            &headers,
        );
        assert_eq!(
            r#"(request-target): get /foo
host: example.org
date: Tue, 07 Jun 2014 20:51:35 GMT
cache-control: max-age=60, must-revalidate
x-example: Example header with some whitespace."#,
            signature_string
        );
    }

    #[test]
    fn verify_signature_works() {
        let data = b"hello there";
        let private_key = load_private_key("./tests/test-signing-key.pem").unwrap();
        let signature = sign(data, private_key).unwrap();
        let public_key = load_public_key("./key_repository/test.pem").unwrap();
        assert_eq!(Ok(true), verify(data, &signature, public_key))
    }

    #[test]
    fn verify_signature_fails_with_modified_data() {
        let data = b"hello foo";
        let private_key = load_private_key("./tests/test-signing-key.pem").unwrap();
        let signature = sign(data, private_key).unwrap();
        let modified_data = b"hello bar";
        let public_key = load_public_key("./key_repository/test.pem").unwrap();
        assert_eq!(Ok(false), verify(modified_data, &signature, public_key))
    }

    #[test]
    fn signature_display_impl() {
        let signature = SignatureHeader::new(
            "abc123".to_string(),
            "ecdsa-sha256".to_string(),
            vec![
                "(request-target)".to_string(),
                "date".to_string(),
                "digest".to_string(),
            ],
            "KJdh1i2&YD7yo8172i".to_string(),
        );
        assert_eq!(
            r#"Signature keyId="abc123", algorithm="ecdsa-sha256", headers="(request-target) date digest", signature="KJdh1i2&YD7yo8172i""#,
            signature.to_string()
        )
    }

    #[test]
    fn signature_from_str() {
        let signature = SignatureHeader::new(
            "abc123".to_string(),
            "ecdsa-sha256".to_string(),
            vec![
                "(request-target)".to_string(),
                "date".to_string(),
                "digest".to_string(),
            ],
            "KJdh1i2&YD7yo8172i".to_string(),
        );
        assert_eq!(
            Ok(signature),
            r#"Signature keyId="abc123", algorithm="ecdsa-sha256", headers="(request-target) date digest", signature="KJdh1i2&YD7yo8172i""#.parse(),
        )
    }
}
