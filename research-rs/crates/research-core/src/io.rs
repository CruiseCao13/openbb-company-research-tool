use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::Path;

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    if path.exists() && fs::read_to_string(path)? == content {
        return Ok(false);
    }
    fs::write(path, content)?;
    Ok(true)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<bool> {
    let content = serde_json::to_string_pretty(value)?;
    write_if_changed(path, &(content + "\n"))
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
