use crate::io::{ensure_dir, write_json};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn digest_str(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn ai_cache_path(cache_root: &Path, key_material: &str) -> PathBuf {
    cache_root
        .join("ai")
        .join(format!("{}.json", digest_str(key_material)))
}

pub fn write_cache<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    write_json(path, value)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub cache_key: String,
    pub input_digest: String,
    pub output_digest: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub version: String,
    pub source: String,
    pub hit_count: usize,
    pub invalidation_reason: Option<String>,
}
