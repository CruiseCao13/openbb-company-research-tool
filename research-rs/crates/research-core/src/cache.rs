use crate::io::{ensure_dir, write_json};
use anyhow::Result;
use serde::Serialize;
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
