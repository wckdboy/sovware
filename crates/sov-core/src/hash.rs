//! Canonical hashing. Compact JSON, sorted keys, sha256. The ledger depends on this being
//! boring and exact.

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::Result;

pub fn sha256_tag(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("sha256:{}", hex(&h.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Serialize to compact JSON with sorted keys. `serde_json::Value`'s map is ordered, so
/// this is canonical for the subset of JSON we accept.
pub fn canonical<T: serde::Serialize>(value: &T) -> Result<String> {
    let v = serde_json::to_value(value)?;
    Ok(serde_json::to_string(&v)?)
}

pub fn hash_json<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(sha256_tag(canonical(value)?.as_bytes()))
}

/// Hash a structure without one field, so the field can hold its own digest.
pub fn hash_without<T: serde::Serialize>(value: &T, field: &str) -> Result<String> {
    let mut v = serde_json::to_value(value)?;
    if let Some(map) = v.as_object_mut() {
        map.remove(field);
    }
    Ok(sha256_tag(serde_json::to_string(&v)?.as_bytes()))
}

pub fn hash_file(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_tag(&bytes))
}
