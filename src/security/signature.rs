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
pub struct Signature {
    key_id: String,
    algorithm: String,
    headers: Vec<String>,
    signature: String,
}

impl Signature {
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

impl Display for Signature {
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

impl FromStr for Signature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let without_signature = s.trim_start_matches("Signature ");
        let mut map = HashMap::new();
        for kv in without_signature.split(", ") {
            let mut kv = kv.split('=');
            let key = kv.next().unwrap();
            let value = kv.next().unwrap().trim_matches('"');
            map.insert(key, value);
        }

        let signature = Signature {
            key_id: map.get("keyId").unwrap().to_string(),
            algorithm: map.get("algorithm").unwrap().to_string(),
            headers: map
                .get("headers")
                .unwrap()
                .split(' ')
                .map(|s| s.to_string())
                .collect(),
            signature: map.get("signature").unwrap().to_string(),
        };

        Ok(signature)
    }
}

/// Loads the specified private key.
pub fn load_private_key(path: &str) -> PKey<Private> {
    let mut file = File::open(path).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    let ec_key = EcKey::private_key_from_pem(&buf).unwrap();
    PKey::from_ec_key(ec_key).unwrap()
}

/// Loads the specified public key.
pub fn load_public_key(path: &str) -> PKey<Public> {
    let mut file = File::open(path).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    let ec_key = EcKey::public_key_from_pem(&buf).unwrap();
    PKey::from_ec_key(ec_key).unwrap()
}

/// Creates the signature of the provided data.
pub fn sign(data: &[u8]) -> Vec<u8> {
    let privkey = load_private_key("./keys/private.pem");
    let mut signer = Signer::new(MessageDigest::sha256(), &privkey).unwrap();
    signer.update(data).unwrap();
    signer.sign_to_vec().unwrap()
}

/// Uses a signature to verify the provided data.
pub fn verify(data: &[u8], signature: &[u8]) -> bool {
    let pubkey = load_public_key("./keys/public.pem");
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pubkey).unwrap();
    verifier.update(data).unwrap();
    verifier.verify(signature).unwrap()
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
    use super::{sign, signature_string, verify, Signature};
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
        let signature = sign(data);
        assert!(verify(data, &signature))
    }

    #[test]
    fn verify_signature_fails_with_modified_data() {
        let data = b"hello foo";
        let signature = sign(data);
        let modified_data = b"hello bar";
        assert!(!verify(modified_data, &signature))
    }

    #[test]
    fn signature_display_impl() {
        let signature = Signature::new(
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
        let signature = Signature::new(
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
