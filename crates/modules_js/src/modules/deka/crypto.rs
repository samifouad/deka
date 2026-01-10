use std::collections::HashMap;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, Payload},
};
use base64::Engine;
use deno_core::{error::CoreError, op2};
use getrandom::getrandom;
use openssl::bn::BigNum;
use openssl::ec::{EcGroup, EcKey, EcPoint};
use openssl::hash::{Hasher, MessageDigest};
use openssl::nid::Nid;
use openssl::pkcs5;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{RsaPssSaltlen, Signer};

#[derive(Clone)]
enum KeyKind {
    Secret(Vec<u8>),
    Private(PKey<Private>),
    Public(PKey<Public>),
}

#[derive(Clone)]
struct KeyEntry {
    kind: KeyKind,
    asymmetric_key_type: Option<String>,
    asymmetric_key_details: Option<serde_json::Value>,
}

fn key_store() -> &'static Mutex<HashMap<u64, KeyEntry>> {
    static KEY_STORE: OnceLock<Mutex<HashMap<u64, KeyEntry>>> = OnceLock::new();
    KEY_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_key_id() -> u64 {
    static KEY_ID: AtomicU64 = AtomicU64::new(1);
    KEY_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
struct EcdhEntry {
    nid: Nid,
    key: Option<EcKey<Private>>,
}

fn ecdh_store() -> &'static Mutex<HashMap<u64, EcdhEntry>> {
    static ECDH_STORE: OnceLock<Mutex<HashMap<u64, EcdhEntry>>> = OnceLock::new();
    ECDH_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_ecdh_id() -> u64 {
    static ECDH_ID: AtomicU64 = AtomicU64::new(1);
    ECDH_ID.fetch_add(1, Ordering::Relaxed)
}

fn insert_key(entry: KeyEntry) -> u64 {
    let id = next_key_id();
    let store = key_store();
    if let Ok(mut guard) = store.lock() {
        guard.insert(id, entry);
    }
    id
}

fn get_key(id: u64) -> Result<KeyEntry, CoreError> {
    let store = key_store();
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Key store locked",
        ))
    })?;
    guard.get(&id).cloned().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Key not found",
        ))
    })
}

fn curve_name(nid: Nid) -> Option<String> {
    match nid {
        Nid::X9_62_PRIME256V1 => Some("prime256v1".to_string()),
        Nid::SECP256K1 => Some("secp256k1".to_string()),
        Nid::SECP384R1 => Some("secp384r1".to_string()),
        Nid::SECP521R1 => Some("secp521r1".to_string()),
        _ => nid.short_name().ok().map(|s| s.to_string()),
    }
}

fn key_details_from_pkey(pkey: &PKey<Private>) -> (Option<String>, Option<serde_json::Value>) {
    match pkey.id() {
        openssl::pkey::Id::RSA | openssl::pkey::Id::RSA_PSS => {
            if let Ok(rsa) = pkey.rsa() {
                let modulus_len = rsa.n().num_bits();
                let exponent = rsa.e().to_dec_str().ok().map(|s| s.to_string());
                let details = serde_json::json!({
                    "modulusLength": modulus_len,
                    "publicExponent": exponent,
                });
                let key_type = if pkey.id() == openssl::pkey::Id::RSA_PSS {
                    "rsa-pss"
                } else {
                    "rsa"
                };
                return (Some(key_type.to_string()), Some(details));
            }
            (Some("rsa".to_string()), None)
        }
        openssl::pkey::Id::EC => {
            if let Ok(ec) = pkey.ec_key() {
                if let Some(nid) = ec.group().curve_name() {
                    let details = serde_json::json!({
                        "namedCurve": curve_name(nid).unwrap_or_else(|| "unknown".to_string()),
                    });
                    return (Some("ec".to_string()), Some(details));
                }
            }
            (Some("ec".to_string()), None)
        }
        openssl::pkey::Id::ED25519 => (Some("ed25519".to_string()), Some(serde_json::json!({}))),
        openssl::pkey::Id::ED448 => (Some("ed448".to_string()), Some(serde_json::json!({}))),
        _ => (None, None),
    }
}

fn key_details_from_public(pkey: &PKey<Public>) -> (Option<String>, Option<serde_json::Value>) {
    match pkey.id() {
        openssl::pkey::Id::RSA | openssl::pkey::Id::RSA_PSS => {
            if let Ok(rsa) = pkey.rsa() {
                let modulus_len = rsa.n().num_bits();
                let exponent = rsa.e().to_dec_str().ok().map(|s| s.to_string());
                let details = serde_json::json!({
                    "modulusLength": modulus_len,
                    "publicExponent": exponent,
                });
                let key_type = if pkey.id() == openssl::pkey::Id::RSA_PSS {
                    "rsa-pss"
                } else {
                    "rsa"
                };
                return (Some(key_type.to_string()), Some(details));
            }
            (Some("rsa".to_string()), None)
        }
        openssl::pkey::Id::EC => {
            if let Ok(ec) = pkey.ec_key() {
                if let Some(nid) = ec.group().curve_name() {
                    let details = serde_json::json!({
                        "namedCurve": curve_name(nid).unwrap_or_else(|| "unknown".to_string()),
                    });
                    return (Some("ec".to_string()), Some(details));
                }
            }
            (Some("ec".to_string()), None)
        }
        openssl::pkey::Id::ED25519 => (Some("ed25519".to_string()), Some(serde_json::json!({}))),
        openssl::pkey::Id::ED448 => (Some("ed448".to_string()), Some(serde_json::json!({}))),
        _ => (None, None),
    }
}

fn jwk_curve_to_nid(curve: &str) -> Option<Nid> {
    match curve {
        "P-256" => Some(Nid::X9_62_PRIME256V1),
        "P-384" => Some(Nid::SECP384R1),
        "P-521" => Some(Nid::SECP521R1),
        "secp256k1" => Some(Nid::SECP256K1),
        _ => None,
    }
}

fn curve_name_to_nid(curve: &str) -> Option<Nid> {
    let name = curve.to_ascii_lowercase();
    match name.as_str() {
        "prime256v1" => Some(Nid::X9_62_PRIME256V1),
        "secp256k1" => Some(Nid::SECP256K1),
        "secp384r1" => Some(Nid::SECP384R1),
        "secp521r1" => Some(Nid::SECP521R1),
        "p-256" | "p-384" | "p-521" => jwk_curve_to_nid(curve),
        _ => None,
    }
}

fn nid_to_jwk_curve(nid: Nid) -> Option<String> {
    match nid {
        Nid::X9_62_PRIME256V1 => Some("P-256".to_string()),
        Nid::SECP384R1 => Some("P-384".to_string()),
        Nid::SECP521R1 => Some("P-521".to_string()),
        Nid::SECP256K1 => Some("secp256k1".to_string()),
        _ => None,
    }
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_random(#[bigint] len: u64) -> Result<Vec<u8>, CoreError> {
    let len = len.min(1024 * 1024) as usize;
    let mut buf = vec![0u8; len];
    getrandom(&mut buf).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("getrandom failed: {}", err),
        ))
    })?;
    Ok(buf)
}

fn normalize_digest_name(name: &str) -> (String, Option<usize>) {
    let lower = name.to_ascii_lowercase();
    let normalized = lower.replace('_', "-");
    let cleaned = normalized.replace("--", "-");
    match cleaned.as_str() {
        "sha1" | "sha-1" | "sha128" => ("sha1".to_string(), None),
        "sha224" | "sha-224" => ("sha224".to_string(), None),
        "sha256" | "sha-256" => ("sha256".to_string(), None),
        "sha384" | "sha-384" => ("sha384".to_string(), None),
        "sha512" | "sha-512" => ("sha512".to_string(), None),
        "sha512/224" | "sha512-224" | "sha512_224" | "sha512224" => {
            ("sha512-224".to_string(), None)
        }
        "sha512/256" | "sha512-256" | "sha512_256" | "sha512256" => {
            ("sha512-256".to_string(), None)
        }
        "sha3-224" => ("sha3-224".to_string(), None),
        "sha3-256" => ("sha3-256".to_string(), None),
        "sha3-384" => ("sha3-384".to_string(), None),
        "sha3-512" => ("sha3-512".to_string(), None),
        "shake128" => ("shake128".to_string(), Some(16)),
        "shake256" => ("shake256".to_string(), Some(32)),
        "blake2b256" => ("blake2b256".to_string(), None),
        "blake2b512" => ("blake2b512".to_string(), None),
        "blake2s256" => ("blake2s256".to_string(), None),
        "ripemd160" | "rmd160" => ("ripemd160".to_string(), None),
        "md4" => ("md4".to_string(), None),
        "md5" => ("md5".to_string(), None),
        other => (other.to_string(), None),
    }
}

fn b64url_decode(input: &str) -> Result<Vec<u8>, CoreError> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input.as_bytes())
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid base64url: {}", err),
            ))
        })
}

fn b64url_encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn bignum_from_b64(input: &str) -> Result<BigNum, CoreError> {
    let bytes = b64url_decode(input)?;
    BigNum::from_slice(&bytes).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid BigNum: {}", err),
        ))
    })
}

fn cipher_from_name(name: &str) -> Option<openssl::symm::Cipher> {
    match name {
        "aes-128-cbc" => Some(openssl::symm::Cipher::aes_128_cbc()),
        "aes-192-cbc" => Some(openssl::symm::Cipher::aes_192_cbc()),
        "aes-256-cbc" => Some(openssl::symm::Cipher::aes_256_cbc()),
        _ => None,
    }
}

fn digest_with_openssl(algorithm: &str, data: &[u8]) -> Result<Vec<u8>, CoreError> {
    let (name, xof_len) = normalize_digest_name(algorithm);
    let digest = MessageDigest::from_name(&name).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported digest algorithm: {}", algorithm),
        ))
    })?;
    let mut hasher = Hasher::new(digest).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Digest init failed: {}", err),
        ))
    })?;
    hasher.update(data).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Digest update failed: {}", err),
        ))
    })?;
    if let Some(out_len) = xof_len {
        let mut buf = vec![0u8; out_len];
        hasher.finish_xof(&mut buf).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Digest finalize failed: {}", err),
            ))
        })?;
        return Ok(buf);
    }
    hasher.finish().map(|bytes| bytes.to_vec()).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Digest finalize failed: {}", err),
        ))
    })
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_digest(
    #[string] algorithm: String,
    #[buffer] data: &[u8],
) -> Result<Vec<u8>, CoreError> {
    digest_with_openssl(&algorithm, data)
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_hmac(
    #[string] algorithm: String,
    #[buffer] key: &[u8],
    #[buffer] data: &[u8],
) -> Result<Vec<u8>, CoreError> {
    let (name, _) = normalize_digest_name(&algorithm);
    let digest = MessageDigest::from_name(&name).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported HMAC algorithm: {}", algorithm),
        ))
    })?;
    let pkey = PKey::hmac(key).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid HMAC key: {}", err),
        ))
    })?;
    let mut signer = Signer::new(digest, &pkey).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("HMAC init failed: {}", err),
        ))
    })?;
    signer.update(data).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("HMAC update failed: {}", err),
        ))
    })?;
    signer.sign_to_vec().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("HMAC finalize failed: {}", err),
        ))
    })
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_pbkdf2(
    #[buffer] password: &[u8],
    #[buffer] salt: &[u8],
    #[bigint] iterations: u64,
    #[bigint] key_len: u64,
    #[string] algorithm: String,
) -> Result<Vec<u8>, CoreError> {
    let iterations = iterations.min(u32::MAX as u64) as usize;
    let key_len = key_len.min(1024 * 1024) as usize;
    let mut out = vec![0u8; key_len];
    let (name, _) = normalize_digest_name(&algorithm);
    let digest = MessageDigest::from_name(&name).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported PBKDF2 digest algorithm: {}", algorithm),
        ))
    })?;
    pkcs5::pbkdf2_hmac(password, salt, iterations, digest, &mut out).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("PBKDF2 failed: {}", err),
        ))
    })?;
    Ok(out)
}
#[derive(serde::Serialize)]
struct CryptoAesGcmResult {
    ciphertext: Vec<u8>,
    tag: Vec<u8>,
}

#[op2]
#[serde]
pub(crate) fn op_crypto_aes_gcm_encrypt(
    #[buffer] key: &[u8],
    #[buffer] iv: &[u8],
    #[buffer] data: &[u8],
    #[buffer] aad: &[u8],
) -> Result<CryptoAesGcmResult, CoreError> {
    if key.len() != 32 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "AES-256-GCM requires a 32-byte key",
        )));
    }
    if iv.len() != 12 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "AES-256-GCM requires a 12-byte IV",
        )));
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid AES key: {}", err),
        ))
    })?;
    let nonce = aes_gcm::Nonce::from_slice(iv);
    let encrypted = cipher
        .encrypt(nonce, Payload { msg: data, aad })
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("AES-GCM encrypt failed: {}", err),
            ))
        })?;
    if encrypted.len() < 16 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "AES-GCM ciphertext too short",
        )));
    }
    let tag = encrypted[encrypted.len() - 16..].to_vec();
    let ciphertext = encrypted[..encrypted.len() - 16].to_vec();
    Ok(CryptoAesGcmResult { ciphertext, tag })
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_aes_gcm_decrypt(
    #[buffer] key: &[u8],
    #[buffer] iv: &[u8],
    #[buffer] data: &[u8],
    #[buffer] tag: &[u8],
    #[buffer] aad: &[u8],
) -> Result<Vec<u8>, CoreError> {
    if key.len() != 32 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "AES-256-GCM requires a 32-byte key",
        )));
    }
    if iv.len() != 12 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "AES-256-GCM requires a 12-byte IV",
        )));
    }
    if tag.len() != 16 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "AES-256-GCM requires a 16-byte auth tag",
        )));
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid AES key: {}", err),
        ))
    })?;
    let nonce = aes_gcm::Nonce::from_slice(iv);
    let mut combined = Vec::with_capacity(data.len() + tag.len());
    combined.extend_from_slice(data);
    combined.extend_from_slice(tag);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: &combined,
                aad,
            },
        )
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("AES-GCM decrypt failed: {}", err),
            ))
        })
}
#[derive(serde::Serialize)]
struct CryptoKeyInfo {
    key_type: String,
    asymmetric_key_type: Option<String>,
    asymmetric_key_details: Option<serde_json::Value>,
    symmetric_key_size: Option<u64>,
}

fn key_info_from_entry(entry: &KeyEntry) -> CryptoKeyInfo {
    match &entry.kind {
        KeyKind::Secret(bytes) => CryptoKeyInfo {
            key_type: "secret".to_string(),
            asymmetric_key_type: None,
            asymmetric_key_details: None,
            symmetric_key_size: Some(bytes.len() as u64),
        },
        KeyKind::Private(_) => CryptoKeyInfo {
            key_type: "private".to_string(),
            asymmetric_key_type: entry.asymmetric_key_type.clone(),
            asymmetric_key_details: entry.asymmetric_key_details.clone(),
            symmetric_key_size: None,
        },
        KeyKind::Public(_) => CryptoKeyInfo {
            key_type: "public".to_string(),
            asymmetric_key_type: entry.asymmetric_key_type.clone(),
            asymmetric_key_details: entry.asymmetric_key_details.clone(),
            symmetric_key_size: None,
        },
    }
}

#[op2]
#[serde]
pub(crate) fn op_crypto_key_info(#[bigint] id: u64) -> Result<CryptoKeyInfo, CoreError> {
    let entry = get_key(id)?;
    Ok(key_info_from_entry(&entry))
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_crypto_key_from_secret(#[buffer] key: &[u8]) -> Result<u64, CoreError> {
    Ok(insert_key(KeyEntry {
        kind: KeyKind::Secret(key.to_vec()),
        asymmetric_key_type: None,
        asymmetric_key_details: None,
    }))
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_crypto_key_from_pem(
    #[buffer] key: &[u8],
    #[string] format: String,
    #[string] passphrase: String,
) -> Result<u64, CoreError> {
    let passphrase_bytes = if passphrase.is_empty() {
        None
    } else {
        Some(passphrase.as_bytes())
    };
    let format_lower = format.to_ascii_lowercase();
    let entry = if format_lower == "public" {
        let pkey = PKey::public_key_from_pem(key).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid public key: {}", err),
            ))
        })?;
        let (asym, details) = key_details_from_public(&pkey);
        KeyEntry {
            kind: KeyKind::Public(pkey),
            asymmetric_key_type: asym,
            asymmetric_key_details: details,
        }
    } else {
        let pkey = if let Some(pass) = passphrase_bytes {
            PKey::private_key_from_pem_passphrase(key, pass)
        } else {
            PKey::private_key_from_pem(key)
        }
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid private key: {}", err),
            ))
        })?;
        let (asym, details) = key_details_from_pkey(&pkey);
        KeyEntry {
            kind: KeyKind::Private(pkey),
            asymmetric_key_type: asym,
            asymmetric_key_details: details,
        }
    };
    Ok(insert_key(entry))
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_crypto_key_from_der(
    #[buffer] key: &[u8],
    #[string] format: String,
    #[string] typ: String,
) -> Result<u64, CoreError> {
    let format_lower = format.to_ascii_lowercase();
    let type_lower = typ.to_ascii_lowercase();
    let entry = if format_lower == "public" {
        let pkey = match type_lower.as_str() {
            "spki" => PKey::public_key_from_der(key),
            "pkcs1" => {
                let rsa = Rsa::public_key_from_der_pkcs1(key).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid RSA public key: {}", err),
                    ))
                })?;
                PKey::from_rsa(rsa)
            }
            _ => {
                return Err(CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported public key type: {}", typ),
                )));
            }
        }
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid public key: {}", err),
            ))
        })?;
        let (asym, details) = key_details_from_public(&pkey);
        KeyEntry {
            kind: KeyKind::Public(pkey),
            asymmetric_key_type: asym,
            asymmetric_key_details: details,
        }
    } else {
        let pkey = match type_lower.as_str() {
            "pkcs8" => PKey::private_key_from_der(key),
            "pkcs1" => {
                let rsa = Rsa::private_key_from_der(key).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid RSA private key: {}", err),
                    ))
                })?;
                PKey::from_rsa(rsa)
            }
            "sec1" => {
                let ec = EcKey::private_key_from_der(key).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid EC private key: {}", err),
                    ))
                })?;
                PKey::from_ec_key(ec)
            }
            _ => {
                return Err(CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported private key type: {}", typ),
                )));
            }
        }
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid private key: {}", err),
            ))
        })?;
        let (asym, details) = key_details_from_pkey(&pkey);
        KeyEntry {
            kind: KeyKind::Private(pkey),
            asymmetric_key_type: asym,
            asymmetric_key_details: details,
        }
    };
    Ok(insert_key(entry))
}

#[op2]
#[bigint]
pub(crate) fn op_crypto_key_from_jwk(#[serde] jwk: serde_json::Value) -> Result<u64, CoreError> {
    let kty = jwk.get("kty").and_then(|v| v.as_str()).unwrap_or("");
    match kty {
        "oct" => {
            let key = jwk.get("k").and_then(|v| v.as_str()).unwrap_or("");
            let bytes = b64url_decode(key)?;
            Ok(insert_key(KeyEntry {
                kind: KeyKind::Secret(bytes.clone()),
                asymmetric_key_type: None,
                asymmetric_key_details: None,
            }))
        }
        "RSA" => {
            let n = bignum_from_b64(jwk.get("n").and_then(|v| v.as_str()).unwrap_or(""))?;
            let e = bignum_from_b64(jwk.get("e").and_then(|v| v.as_str()).unwrap_or(""))?;
            if let Some(d_val) = jwk.get("d").and_then(|v| v.as_str()) {
                let d = bignum_from_b64(d_val)?;
                let p = bignum_from_b64(jwk.get("p").and_then(|v| v.as_str()).unwrap_or(""))?;
                let q = bignum_from_b64(jwk.get("q").and_then(|v| v.as_str()).unwrap_or(""))?;
                let dp = bignum_from_b64(jwk.get("dp").and_then(|v| v.as_str()).unwrap_or(""))?;
                let dq = bignum_from_b64(jwk.get("dq").and_then(|v| v.as_str()).unwrap_or(""))?;
                let qi = bignum_from_b64(jwk.get("qi").and_then(|v| v.as_str()).unwrap_or(""))?;
                let rsa =
                    Rsa::from_private_components(n, e, d, p, q, dp, dq, qi).map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid RSA private key: {}", err),
                        ))
                    })?;
                let pkey = PKey::from_rsa(rsa).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid RSA private key: {}", err),
                    ))
                })?;
                let (asym, details) = key_details_from_pkey(&pkey);
                Ok(insert_key(KeyEntry {
                    kind: KeyKind::Private(pkey),
                    asymmetric_key_type: asym,
                    asymmetric_key_details: details,
                }))
            } else {
                let rsa = Rsa::from_public_components(n, e).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid RSA public key: {}", err),
                    ))
                })?;
                let pkey = PKey::from_rsa(rsa).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid RSA public key: {}", err),
                    ))
                })?;
                let (asym, details) = key_details_from_public(&pkey);
                Ok(insert_key(KeyEntry {
                    kind: KeyKind::Public(pkey),
                    asymmetric_key_type: asym,
                    asymmetric_key_details: details,
                }))
            }
        }
        "EC" => {
            let crv = jwk.get("crv").and_then(|v| v.as_str()).unwrap_or("");
            let nid = jwk_curve_to_nid(crv).ok_or_else(|| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported EC curve: {}", crv),
                ))
            })?;
            let group = EcGroup::from_curve_name(nid).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid EC curve: {}", err),
                ))
            })?;
            let x = bignum_from_b64(jwk.get("x").and_then(|v| v.as_str()).unwrap_or(""))?;
            let y = bignum_from_b64(jwk.get("y").and_then(|v| v.as_str()).unwrap_or(""))?;
            let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid BN context: {}", err),
                ))
            })?;
            if let Some(d_val) = jwk.get("d").and_then(|v| v.as_str()) {
                let d = bignum_from_b64(d_val)?;
                let mut point = EcPoint::new(&group).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid EC point: {}", err),
                    ))
                })?;
                point
                    .set_affine_coordinates_gfp(&group, &x, &y, &mut ctx)
                    .map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid EC coordinates: {}", err),
                        ))
                    })?;
                let ec = EcKey::from_private_components(&group, &d, &point).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid EC private key: {}", err),
                    ))
                })?;
                let pkey = PKey::from_ec_key(ec).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid EC private key: {}", err),
                    ))
                })?;
                let (asym, details) = key_details_from_pkey(&pkey);
                Ok(insert_key(KeyEntry {
                    kind: KeyKind::Private(pkey),
                    asymmetric_key_type: asym,
                    asymmetric_key_details: details,
                }))
            } else {
                let ec =
                    EcKey::from_public_key_affine_coordinates(&group, &x, &y).map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid EC public key: {}", err),
                        ))
                    })?;
                let pkey = PKey::from_ec_key(ec).map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid EC public key: {}", err),
                    ))
                })?;
                let (asym, details) = key_details_from_public(&pkey);
                Ok(insert_key(KeyEntry {
                    kind: KeyKind::Public(pkey),
                    asymmetric_key_type: asym,
                    asymmetric_key_details: details,
                }))
            }
        }
        "OKP" => {
            let crv = jwk.get("crv").and_then(|v| v.as_str()).unwrap_or("");
            let x = jwk.get("x").and_then(|v| v.as_str()).unwrap_or("");
            let x_bytes = b64url_decode(x)?;
            if let Some(d_val) = jwk.get("d").and_then(|v| v.as_str()) {
                let d_bytes = b64url_decode(d_val)?;
                let pkey = match crv {
                    "Ed25519" => {
                        PKey::private_key_from_raw_bytes(&d_bytes, openssl::pkey::Id::ED25519)
                    }
                    "Ed448" => PKey::private_key_from_raw_bytes(&d_bytes, openssl::pkey::Id::ED448),
                    _ => {
                        return Err(CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Unsupported OKP curve: {}", crv),
                        )));
                    }
                }
                .map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid OKP private key: {}", err),
                    ))
                })?;
                let (asym, details) = key_details_from_pkey(&pkey);
                Ok(insert_key(KeyEntry {
                    kind: KeyKind::Private(pkey),
                    asymmetric_key_type: asym,
                    asymmetric_key_details: details,
                }))
            } else {
                let pkey = match crv {
                    "Ed25519" => {
                        PKey::public_key_from_raw_bytes(&x_bytes, openssl::pkey::Id::ED25519)
                    }
                    "Ed448" => PKey::public_key_from_raw_bytes(&x_bytes, openssl::pkey::Id::ED448),
                    _ => {
                        return Err(CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Unsupported OKP curve: {}", crv),
                        )));
                    }
                }
                .map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid OKP public key: {}", err),
                    ))
                })?;
                let (asym, details) = key_details_from_public(&pkey);
                Ok(insert_key(KeyEntry {
                    kind: KeyKind::Public(pkey),
                    asymmetric_key_type: asym,
                    asymmetric_key_details: details,
                }))
            }
        }
        _ => Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unsupported JWK key type",
        ))),
    }
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_key_export_der(
    #[bigint] id: u64,
    #[string] typ: String,
) -> Result<Vec<u8>, CoreError> {
    let entry = get_key(id)?;
    match entry.kind {
        KeyKind::Secret(bytes) => Ok(bytes),
        KeyKind::Public(pkey) => {
            let type_lower = typ.to_ascii_lowercase();
            match type_lower.as_str() {
                "spki" => pkey.public_key_to_der().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                }),
                "pkcs1" => {
                    let rsa = pkey.rsa().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    rsa.public_key_to_der_pkcs1().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })
                }
                _ => Err(CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported export type: {}", typ),
                ))),
            }
        }
        KeyKind::Private(pkey) => {
            let type_lower = typ.to_ascii_lowercase();
            match type_lower.as_str() {
                "pkcs8" => pkey.private_key_to_der().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                }),
                "pkcs1" => {
                    let rsa = pkey.rsa().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    rsa.private_key_to_der().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })
                }
                "sec1" => {
                    let ec = pkey.ec_key().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    ec.private_key_to_der().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })
                }
                _ => Err(CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported export type: {}", typ),
                ))),
            }
        }
    }
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_key_export_pem(
    #[bigint] id: u64,
    #[string] typ: String,
    #[string] cipher: String,
    #[string] passphrase: String,
) -> Result<Vec<u8>, CoreError> {
    let entry = get_key(id)?;
    match entry.kind {
        KeyKind::Secret(bytes) => Ok(bytes),
        KeyKind::Public(pkey) => {
            let type_lower = typ.to_ascii_lowercase();
            match type_lower.as_str() {
                "spki" => {
                    let pem = pkey.public_key_to_pem().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    Ok(pem)
                }
                "pkcs1" => {
                    let rsa = pkey.rsa().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    let pem = rsa.public_key_to_pem_pkcs1().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    Ok(pem)
                }
                _ => Err(CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported export type: {}", typ),
                ))),
            }
        }
        KeyKind::Private(pkey) => {
            let type_lower = typ.to_ascii_lowercase();
            let use_cipher = !cipher.is_empty() && !passphrase.is_empty();
            let pem = match type_lower.as_str() {
                "pkcs8" => {
                    if use_cipher {
                        let cipher = cipher_from_name(&cipher).ok_or_else(|| {
                            CoreError::from(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Unsupported cipher: {}", cipher),
                            ))
                        })?;
                        pkey.private_key_to_pem_pkcs8_passphrase(cipher, passphrase.as_bytes())
                    } else {
                        pkey.private_key_to_pem_pkcs8()
                    }
                }
                "pkcs1" => {
                    let rsa = pkey.rsa().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    if use_cipher {
                        let cipher = cipher_from_name(&cipher).ok_or_else(|| {
                            CoreError::from(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Unsupported cipher: {}", cipher),
                            ))
                        })?;
                        rsa.private_key_to_pem_passphrase(cipher, passphrase.as_bytes())
                    } else {
                        rsa.private_key_to_pem()
                    }
                }
                "sec1" => {
                    let ec = pkey.ec_key().map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                    if use_cipher {
                        let cipher = cipher_from_name(&cipher).ok_or_else(|| {
                            CoreError::from(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Unsupported cipher: {}", cipher),
                            ))
                        })?;
                        ec.private_key_to_pem_passphrase(cipher, passphrase.as_bytes())
                    } else {
                        ec.private_key_to_pem()
                    }
                }
                _ => {
                    return Err(CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Unsupported export type: {}", typ),
                    )));
                }
            }
            .map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Export failed: {}", err),
                ))
            })?;
            Ok(pem)
        }
    }
}

#[op2]
#[serde]
pub(crate) fn op_crypto_key_export_jwk(#[bigint] id: u64) -> Result<serde_json::Value, CoreError> {
    let entry = get_key(id)?;
    match entry.kind {
        KeyKind::Secret(bytes) => Ok(serde_json::json!({
            "kty": "oct",
            "k": b64url_encode(&bytes),
        })),
        KeyKind::Public(pkey) => match pkey.id() {
            openssl::pkey::Id::RSA | openssl::pkey::Id::RSA_PSS => {
                let rsa = pkey.rsa().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                Ok(serde_json::json!({
                    "kty": "RSA",
                    "n": b64url_encode(&rsa.n().to_vec()),
                    "e": b64url_encode(&rsa.e().to_vec()),
                }))
            }
            openssl::pkey::Id::EC => {
                let ec = pkey.ec_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                let group = ec.group();
                let nid = group.curve_name().ok_or_else(|| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unknown EC curve",
                    ))
                })?;
                let crv = nid_to_jwk_curve(nid).ok_or_else(|| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unsupported EC curve",
                    ))
                })?;
                let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid BN context: {}", err),
                    ))
                })?;
                let mut x = BigNum::new().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid BN: {}", err),
                    ))
                })?;
                let mut y = BigNum::new().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid BN: {}", err),
                    ))
                })?;
                ec.public_key()
                    .affine_coordinates(group, &mut x, &mut y, &mut ctx)
                    .map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                Ok(serde_json::json!({
                    "kty": "EC",
                    "crv": crv,
                    "x": b64url_encode(&x.to_vec()),
                    "y": b64url_encode(&y.to_vec()),
                }))
            }
            openssl::pkey::Id::ED25519 => {
                let raw = pkey.raw_public_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                Ok(serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": b64url_encode(&raw),
                }))
            }
            openssl::pkey::Id::ED448 => {
                let raw = pkey.raw_public_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                Ok(serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed448",
                    "x": b64url_encode(&raw),
                }))
            }
            _ => Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported key type",
            ))),
        },
        KeyKind::Private(pkey) => match pkey.id() {
            openssl::pkey::Id::RSA | openssl::pkey::Id::RSA_PSS => {
                let rsa = pkey.rsa().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                Ok(serde_json::json!({
                    "kty": "RSA",
                    "n": b64url_encode(&rsa.n().to_vec()),
                    "e": b64url_encode(&rsa.e().to_vec()),
                    "d": b64url_encode(&rsa.d().to_vec()),
                    "p": b64url_encode(&rsa.p().map(|v| v.to_vec()).unwrap_or_default()),
                    "q": b64url_encode(&rsa.q().map(|v| v.to_vec()).unwrap_or_default()),
                    "dp": b64url_encode(&rsa.dmp1().map(|v| v.to_vec()).unwrap_or_default()),
                    "dq": b64url_encode(&rsa.dmq1().map(|v| v.to_vec()).unwrap_or_default()),
                    "qi": b64url_encode(&rsa.iqmp().map(|v| v.to_vec()).unwrap_or_default()),
                }))
            }
            openssl::pkey::Id::EC => {
                let ec = pkey.ec_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                let group = ec.group();
                let nid = group.curve_name().ok_or_else(|| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unknown EC curve",
                    ))
                })?;
                let crv = nid_to_jwk_curve(nid).ok_or_else(|| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unsupported EC curve",
                    ))
                })?;
                let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid BN context: {}", err),
                    ))
                })?;
                let mut x = BigNum::new().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid BN: {}", err),
                    ))
                })?;
                let mut y = BigNum::new().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid BN: {}", err),
                    ))
                })?;
                ec.public_key()
                    .affine_coordinates(group, &mut x, &mut y, &mut ctx)
                    .map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Export failed: {}", err),
                        ))
                    })?;
                let d = ec.private_key().to_vec();
                Ok(serde_json::json!({
                    "kty": "EC",
                    "crv": crv,
                    "x": b64url_encode(&x.to_vec()),
                    "y": b64url_encode(&y.to_vec()),
                    "d": b64url_encode(&d),
                }))
            }
            openssl::pkey::Id::ED25519 => {
                let raw = pkey.raw_private_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                let pub_raw = pkey.raw_public_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                Ok(serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": b64url_encode(&pub_raw),
                    "d": b64url_encode(&raw),
                }))
            }
            openssl::pkey::Id::ED448 => {
                let raw = pkey.raw_private_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                let pub_raw = pkey.raw_public_key().map_err(|err| {
                    CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Export failed: {}", err),
                    ))
                })?;
                Ok(serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed448",
                    "x": b64url_encode(&pub_raw),
                    "d": b64url_encode(&raw),
                }))
            }
            _ => Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported key type",
            ))),
        },
    }
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_crypto_key_public(#[bigint] id: u64) -> Result<u64, CoreError> {
    let entry = get_key(id)?;
    match entry.kind {
        KeyKind::Private(pkey) => {
            let pub_key = PKey::public_key_from_der(&pkey.public_key_to_der().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to derive public key: {}", err),
                ))
            })?)
            .map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to derive public key: {}", err),
                ))
            })?;
            let (asym, details) = key_details_from_public(&pub_key);
            Ok(insert_key(KeyEntry {
                kind: KeyKind::Public(pub_key),
                asymmetric_key_type: asym,
                asymmetric_key_details: details,
            }))
        }
        _ => Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Key is not private",
        ))),
    }
}

#[op2(fast)]
pub(crate) fn op_crypto_key_equals(#[bigint] a: u64, #[bigint] b: u64) -> Result<bool, CoreError> {
    let key_a = get_key(a)?;
    let key_b = get_key(b)?;
    match (key_a.kind, key_b.kind) {
        (KeyKind::Secret(a_bytes), KeyKind::Secret(b_bytes)) => Ok(a_bytes == b_bytes),
        (KeyKind::Public(a_key), KeyKind::Public(b_key)) => {
            let a_der = a_key.public_key_to_der().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Key compare failed: {}", err),
                ))
            })?;
            let b_der = b_key.public_key_to_der().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Key compare failed: {}", err),
                ))
            })?;
            Ok(a_der == b_der)
        }
        (KeyKind::Private(a_key), KeyKind::Private(b_key)) => {
            let a_der = a_key.private_key_to_der().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Key compare failed: {}", err),
                ))
            })?;
            let b_der = b_key.private_key_to_der().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Key compare failed: {}", err),
                ))
            })?;
            Ok(a_der == b_der)
        }
        _ => Ok(false),
    }
}

fn message_digest_from_alg(alg: &str) -> Result<MessageDigest, CoreError> {
    let (name, _) = normalize_digest_name(alg);
    MessageDigest::from_name(&name).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported digest algorithm: {}", alg),
        ))
    })
}

fn dsa_sig_to_p1363(sig: &[u8], key: &PKey<Private>) -> Result<Vec<u8>, CoreError> {
    let ec = key.ec_key().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid EC key: {}", err),
        ))
    })?;
    let sig = openssl::ecdsa::EcdsaSig::from_der(sig).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid ECDSA signature: {}", err),
        ))
    })?;
    let degree = ec.group().degree();
    let len = ((degree + 7) / 8) as usize;
    let r = sig.r().to_vec();
    let s = sig.s().to_vec();
    let mut out = vec![0u8; len * 2];
    let r_start = len.saturating_sub(r.len());
    let s_start = len.saturating_sub(s.len());
    out[r_start..r_start + r.len()].copy_from_slice(&r);
    out[len + s_start..len + s.len()].copy_from_slice(&s);
    Ok(out)
}

fn p1363_to_dsa_sig(data: &[u8], key: &PKey<Public>) -> Result<Vec<u8>, CoreError> {
    let ec = key.ec_key().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid EC key: {}", err),
        ))
    })?;
    let degree = ec.group().degree();
    let len = ((degree + 7) / 8) as usize;
    if data.len() != len * 2 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid P1363 signature length",
        )));
    }
    let r = BigNum::from_slice(&data[..len]).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid signature r: {}", err),
        ))
    })?;
    let s = BigNum::from_slice(&data[len..]).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid signature s: {}", err),
        ))
    })?;
    let sig = openssl::ecdsa::EcdsaSig::from_private_components(r, s).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid signature: {}", err),
        ))
    })?;
    sig.to_der().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid signature: {}", err),
        ))
    })
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_sign(
    #[string] algorithm: String,
    #[buffer] data: &[u8],
    #[bigint] key_id: u64,
    #[number] padding: u64,
    #[number] salt_len: u64,
    #[string] dsa_encoding: String,
) -> Result<Vec<u8>, CoreError> {
    let entry = get_key(key_id)?;
    let key = match entry.kind {
        KeyKind::Private(pkey) => pkey,
        _ => {
            return Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Private key required",
            )));
        }
    };
    let digest = if key.id() == openssl::pkey::Id::ED25519 || key.id() == openssl::pkey::Id::ED448 {
        MessageDigest::null()
    } else {
        message_digest_from_alg(&algorithm)?
    };
    let mut signer = Signer::new(digest, &key).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Sign init failed: {}", err),
        ))
    })?;
    if key.id() == openssl::pkey::Id::RSA || key.id() == openssl::pkey::Id::RSA_PSS {
        let padding = padding as i32;
        let salt_len = salt_len as i32;
        if padding != 0 {
            let pad = match padding {
                1 => openssl::rsa::Padding::PKCS1,
                3 => openssl::rsa::Padding::NONE,
                4 => openssl::rsa::Padding::PKCS1_OAEP,
                6 => openssl::rsa::Padding::PKCS1_PSS,
                _ => openssl::rsa::Padding::PKCS1,
            };
            signer.set_rsa_padding(pad).ok();
        }
        if salt_len > 0 {
            signer
                .set_rsa_pss_saltlen(RsaPssSaltlen::custom(salt_len))
                .ok();
        }
    }
    signer.update(data).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Sign update failed: {}", err),
        ))
    })?;
    let mut sig = signer.sign_to_vec().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Sign failed: {}", err),
        ))
    })?;
    if key.id() == openssl::pkey::Id::EC && dsa_encoding == "ieee-p1363" {
        sig = dsa_sig_to_p1363(&sig, &key)?;
    }
    Ok(sig)
}

#[op2(fast)]
pub(crate) fn op_crypto_verify(
    #[string] algorithm: String,
    #[buffer] data: &[u8],
    #[bigint] key_id: u64,
    #[buffer] signature: &[u8],
    #[number] padding: u64,
    #[number] salt_len: u64,
    #[string] dsa_encoding: String,
) -> Result<bool, CoreError> {
    let entry = get_key(key_id)?;
    let key = match entry.kind {
        KeyKind::Public(pkey) => pkey,
        KeyKind::Private(pkey) => {
            let pub_key = PKey::public_key_from_der(&pkey.public_key_to_der().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Verify failed: {}", err),
                ))
            })?)
            .map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Verify failed: {}", err),
                ))
            })?;
            pub_key
        }
        KeyKind::Secret(_) => {
            return Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Public key required",
            )));
        }
    };
    let digest = if key.id() == openssl::pkey::Id::ED25519 || key.id() == openssl::pkey::Id::ED448 {
        MessageDigest::null()
    } else {
        message_digest_from_alg(&algorithm)?
    };
    let mut verifier = openssl::sign::Verifier::new(digest, &key).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Verify init failed: {}", err),
        ))
    })?;
    if key.id() == openssl::pkey::Id::RSA || key.id() == openssl::pkey::Id::RSA_PSS {
        let padding = padding as i32;
        let salt_len = salt_len as i32;
        if padding != 0 {
            let pad = match padding {
                1 => openssl::rsa::Padding::PKCS1,
                3 => openssl::rsa::Padding::NONE,
                4 => openssl::rsa::Padding::PKCS1_OAEP,
                6 => openssl::rsa::Padding::PKCS1_PSS,
                _ => openssl::rsa::Padding::PKCS1,
            };
            verifier.set_rsa_padding(pad).ok();
        }
        if salt_len > 0 {
            verifier
                .set_rsa_pss_saltlen(RsaPssSaltlen::custom(salt_len))
                .ok();
        }
    }
    verifier.update(data).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Verify update failed: {}", err),
        ))
    })?;
    let mut sig = signature.to_vec();
    if key.id() == openssl::pkey::Id::EC && dsa_encoding == "ieee-p1363" {
        sig = p1363_to_dsa_sig(&sig, &key)?;
    }
    verifier.verify(&sig).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Verify failed: {}", err),
        ))
    })
}

#[derive(serde::Serialize)]
struct CryptoKeyPairResult {
    public_id: u64,
    private_id: u64,
}

#[op2]
#[serde]
pub(crate) fn op_crypto_generate_keypair(
    #[string] key_type: String,
    #[serde] options: serde_json::Value,
) -> Result<CryptoKeyPairResult, CoreError> {
    let key_type_lower = key_type.to_ascii_lowercase();
    let (private_key, asym_type, details) = match key_type_lower.as_str() {
        "rsa" | "rsa-pss" => {
            let modulus = options
                .get("modulusLength")
                .and_then(|v| v.as_u64())
                .unwrap_or(2048);
            let exponent_val = options
                .get("publicExponent")
                .and_then(|v| v.as_u64())
                .unwrap_or(65537);
            let exp = BigNum::from_u32(exponent_val as u32).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid exponent: {}", err),
                ))
            })?;
            let rsa = Rsa::generate_with_e(modulus as u32, &exp).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("RSA generation failed: {}", err),
                ))
            })?;
            let pkey = PKey::from_rsa(rsa).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("RSA generation failed: {}", err),
                ))
            })?;
            let details = serde_json::json!({
                "modulusLength": modulus,
                "publicExponent": exponent_val.to_string(),
            });
            (pkey, key_type_lower.clone(), Some(details))
        }
        "ec" => {
            let curve = options
                .get("namedCurve")
                .and_then(|v| v.as_str())
                .unwrap_or("prime256v1");
            let nid = curve_name_to_nid(curve).ok_or_else(|| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsupported curve: {}", curve),
                ))
            })?;
            let group = EcGroup::from_curve_name(nid).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid curve: {}", err),
                ))
            })?;
            let ec_key = EcKey::generate(&group).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("EC generation failed: {}", err),
                ))
            })?;
            let pkey = PKey::from_ec_key(ec_key).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("EC generation failed: {}", err),
                ))
            })?;
            let named_curve = curve_name(nid).unwrap_or_else(|| curve.to_string());
            let details = serde_json::json!({ "namedCurve": named_curve });
            (pkey, "ec".to_string(), Some(details))
        }
        "ed25519" => {
            let pkey = PKey::generate_ed25519().map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Ed25519 generation failed: {}", err),
                ))
            })?;
            (pkey, "ed25519".to_string(), Some(serde_json::json!({})))
        }
        _ => {
            return Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unsupported key type: {}", key_type),
            )));
        }
    };

    let public_key =
        PKey::public_key_from_der(&private_key.public_key_to_der().map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Key export failed: {}", err),
            ))
        })?)
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Key export failed: {}", err),
            ))
        })?;

    let private_id = insert_key(KeyEntry {
        kind: KeyKind::Private(private_key),
        asymmetric_key_type: Some(asym_type.clone()),
        asymmetric_key_details: details.clone(),
    });
    let public_id = insert_key(KeyEntry {
        kind: KeyKind::Public(public_key),
        asymmetric_key_type: Some(asym_type),
        asymmetric_key_details: details,
    });
    Ok(CryptoKeyPairResult {
        public_id,
        private_id,
    })
}

#[op2]
#[serde]
pub(crate) fn op_crypto_get_curves() -> Result<Vec<String>, CoreError> {
    Ok(vec![
        "prime256v1".to_string(),
        "secp256k1".to_string(),
        "secp384r1".to_string(),
        "secp521r1".to_string(),
    ])
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_crypto_ecdh_new(#[string] curve: String) -> Result<u64, CoreError> {
    let nid = curve_name_to_nid(&curve).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported curve: {}", curve),
        ))
    })?;
    let id = next_ecdh_id();
    let store = ecdh_store();
    if let Ok(mut guard) = store.lock() {
        guard.insert(id, EcdhEntry { nid, key: None });
    }
    Ok(id)
}

fn get_ecdh_entry(id: u64) -> Result<EcdhEntry, CoreError> {
    let store = ecdh_store();
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ECDH store locked",
        ))
    })?;
    guard.get(&id).cloned().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "ECDH not found",
        ))
    })
}

fn set_ecdh_entry(id: u64, entry: EcdhEntry) -> Result<(), CoreError> {
    let store = ecdh_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ECDH store locked",
        ))
    })?;
    guard.insert(id, entry);
    Ok(())
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_ecdh_generate(
    #[bigint] id: u64,
    #[string] _format: String,
    #[string] compress: String,
) -> Result<Vec<u8>, CoreError> {
    let mut entry = get_ecdh_entry(id)?;
    let group = EcGroup::from_curve_name(entry.nid).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid curve: {}", err),
        ))
    })?;
    let key = EcKey::generate(&group).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH generate failed: {}", err),
        ))
    })?;
    let public = key.public_key();
    let form = if compress == "compressed" {
        openssl::ec::PointConversionForm::COMPRESSED
    } else {
        openssl::ec::PointConversionForm::UNCOMPRESSED
    };
    let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH context failed: {}", err),
        ))
    })?;
    let bytes = public.to_bytes(&group, form, &mut ctx).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH public key failed: {}", err),
        ))
    })?;
    entry.key = Some(key);
    set_ecdh_entry(id, entry)?;
    Ok(bytes)
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_ecdh_get_public(
    #[bigint] id: u64,
    #[string] compress: String,
) -> Result<Vec<u8>, CoreError> {
    let entry = get_ecdh_entry(id)?;
    let key = entry.key.ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ECDH key not generated",
        ))
    })?;
    let group = EcGroup::from_curve_name(entry.nid).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid curve: {}", err),
        ))
    })?;
    let public = key.public_key();
    let form = if compress == "compressed" {
        openssl::ec::PointConversionForm::COMPRESSED
    } else {
        openssl::ec::PointConversionForm::UNCOMPRESSED
    };
    let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH context failed: {}", err),
        ))
    })?;
    public.to_bytes(&group, form, &mut ctx).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH public key failed: {}", err),
        ))
    })
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_ecdh_get_private(#[bigint] id: u64) -> Result<Vec<u8>, CoreError> {
    let entry = get_ecdh_entry(id)?;
    let key = entry.key.ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ECDH key not generated",
        ))
    })?;
    Ok(key.private_key().to_vec())
}

#[op2(fast)]
pub(crate) fn op_crypto_ecdh_set_private(
    #[bigint] id: u64,
    #[buffer] key_bytes: &[u8],
) -> Result<(), CoreError> {
    let mut entry = get_ecdh_entry(id)?;
    let group = EcGroup::from_curve_name(entry.nid).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid curve: {}", err),
        ))
    })?;
    let priv_bn = BigNum::from_slice(key_bytes).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid private key: {}", err),
        ))
    })?;
    let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid context: {}", err),
        ))
    })?;
    let mut point = EcPoint::new(&group).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid EC point: {}", err),
        ))
    })?;
    point
        .mul_generator(&group, &priv_bn, &mut ctx)
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid EC private key: {}", err),
            ))
        })?;
    let ec_key = EcKey::from_private_components(&group, &priv_bn, &point).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid EC private key: {}", err),
        ))
    })?;
    entry.key = Some(ec_key);
    set_ecdh_entry(id, entry)?;
    Ok(())
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_ecdh_compute_secret(
    #[bigint] id: u64,
    #[buffer] other_public: &[u8],
) -> Result<Vec<u8>, CoreError> {
    let entry = get_ecdh_entry(id)?;
    let group = EcGroup::from_curve_name(entry.nid).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid curve: {}", err),
        ))
    })?;
    let key = entry.key.ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ECDH key not generated",
        ))
    })?;
    let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH context failed: {}", err),
        ))
    })?;
    let point = EcPoint::from_bytes(&group, other_public, &mut ctx).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid public key: {}", err),
        ))
    })?;
    let other = EcKey::from_public_key(&group, &point).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid public key: {}", err),
        ))
    })?;
    let priv_pkey = PKey::from_ec_key(key).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid key: {}", err),
        ))
    })?;
    let pub_pkey = PKey::from_ec_key(other).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid key: {}", err),
        ))
    })?;
    let mut deriver = openssl::derive::Deriver::new(&priv_pkey).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH derive failed: {}", err),
        ))
    })?;
    deriver.set_peer(&pub_pkey).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH derive failed: {}", err),
        ))
    })?;
    deriver.derive_to_vec().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH derive failed: {}", err),
        ))
    })
}

#[op2]
#[buffer]
pub(crate) fn op_crypto_ecdh_convert(
    #[string] curve: String,
    #[buffer] input: &[u8],
    #[string] input_format: String,
    #[string] output_format: String,
    #[string] output_compress: String,
) -> Result<Vec<u8>, CoreError> {
    let nid = curve_name_to_nid(&curve).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported curve: {}", curve),
        ))
    })?;
    let group = EcGroup::from_curve_name(nid).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid curve: {}", err),
        ))
    })?;
    let input_bytes = match input_format.as_str() {
        "hex" => hex::decode(input).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid hex: {}", err),
            ))
        })?,
        "base64" => base64::engine::general_purpose::STANDARD
            .decode(input)
            .map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid base64: {}", err),
                ))
            })?,
        _ => input.to_vec(),
    };
    let mut ctx = openssl::bn::BigNumContext::new().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ECDH context failed: {}", err),
        ))
    })?;
    let point = EcPoint::from_bytes(&group, &input_bytes, &mut ctx).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid public key: {}", err),
        ))
    })?;
    let form = if output_compress == "compressed" {
        openssl::ec::PointConversionForm::COMPRESSED
    } else {
        openssl::ec::PointConversionForm::UNCOMPRESSED
    };
    let out = point.to_bytes(&group, form, &mut ctx).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid public key: {}", err),
        ))
    })?;
    let output = match output_format.as_str() {
        "hex" => hex::encode(out).into_bytes(),
        "base64" => base64::engine::general_purpose::STANDARD
            .encode(out)
            .into_bytes(),
        _ => out,
    };
    Ok(output)
}
