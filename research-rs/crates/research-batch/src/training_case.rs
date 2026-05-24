use anyhow::Result;
use research_core::io::write_if_changed;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct TrainingCase {
    pub ticker: String,
    pub initial_profile: String,
    pub final_profile: String,
    pub expected_family: String,
    pub issue_type: String,
    pub wrong_output: String,
    pub expected_output_features: Vec<String>,
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
    pub data_refs_used: Vec<String>,
    pub fixed_by: String,
    pub regression_status: String,
}

pub fn write_training_case(path: &Path, case: &TrainingCase) -> Result<()> {
    let raw = serde_json::to_string(case)?;
    write_if_changed(path, &(raw + "\n"))?;
    Ok(())
}
