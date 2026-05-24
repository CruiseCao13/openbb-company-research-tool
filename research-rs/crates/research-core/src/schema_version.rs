use crate::io::write_if_changed;
use crate::run_folder::RunFolder;
use crate::types::SCHEMA_VERSION;
use anyhow::Result;

pub fn validate_schema_version(label: &str, version: &str) -> (&'static str, String) {
    if version == SCHEMA_VERSION {
        (
            "PASS",
            format!("{label} schema version {version} is current."),
        )
    } else if version.trim().is_empty() {
        (
            "WARNING",
            format!("{label} has no schema_version; treating as legacy v5-compatible input."),
        )
    } else {
        (
            "WARNING",
            format!(
                "{label} schema version {version} is not recognized by current {SCHEMA_VERSION}."
            ),
        )
    }
}

pub fn write_schema_validation_report(folder: &RunFolder, checks: &[(&str, String)]) -> Result<()> {
    let mut rows = String::new();
    let mut overall = "PASS";
    for (label, version) in checks {
        let (status, message) = validate_schema_version(label, version);
        if status != "PASS" {
            overall = "WARNING";
        }
        rows.push_str(&format!("| {label} | {version} | {status} | {message} |\n"));
    }
    write_if_changed(
        &folder.audit.join("schema_validation.md"),
        &format!(
            "# Schema Validation\n\nOverall: {overall}\n\nCurrent schema: {SCHEMA_VERSION}\n\n| Artifact | Version | Status | Message |\n|---|---|---|---|\n{rows}\nUnknown future versions are warnings in this v5 alpha unless migration is impossible; incompatible migrations should become blocking validation failures in v5.1.\n"
        ),
    )?;
    Ok(())
}
