use crate::provider::discover_repo_root;
use anyhow::Result;
use std::path::PathBuf;

pub fn reports_root() -> Result<PathBuf> {
    Ok(discover_repo_root()?.join("reports"))
}

pub fn reports_root_from_repo_root(repo_root: PathBuf) -> PathBuf {
    repo_root.join("reports")
}

pub fn ai_cache_dir() -> Result<PathBuf> {
    Ok(reports_root()?.join("_cache").join("ai"))
}

pub fn batch_runs_dir() -> Result<PathBuf> {
    Ok(reports_root()?.join("batch_runs"))
}

pub fn quality_runs_dir() -> Result<PathBuf> {
    Ok(reports_root()?.join("quality_runs"))
}

pub fn samples_dir() -> Result<PathBuf> {
    Ok(reports_root()?.join("samples"))
}

pub fn release_checks_dir() -> Result<PathBuf> {
    Ok(reports_root()?.join("release_checks"))
}

pub fn training_cases_dir() -> Result<PathBuf> {
    Ok(discover_repo_root()?.join("training_cases"))
}
