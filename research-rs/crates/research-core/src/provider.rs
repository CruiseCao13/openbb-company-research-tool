use crate::config::EngineConfig;
use crate::io::{read_json, write_json};
use crate::types::{ProviderPayload, RunContext};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub fn fetch_provider_payload(
    ctx: &RunContext,
    config: &EngineConfig,
    out_path: &Path,
) -> Result<ProviderPayload> {
    if out_path.exists() && !ctx.force {
        return read_json(out_path);
    }

    let python = if Path::new(".venv/bin/python").exists() {
        ".venv/bin/python"
    } else {
        "python3"
    };
    let status = Command::new(python)
        .arg(&config.provider_script)
        .arg("--ticker")
        .arg(&ctx.ticker)
        .arg("--market")
        .arg(&ctx.market)
        .arg("--provider")
        .arg(&ctx.provider)
        .arg("--out")
        .arg(out_path)
        .status()?;

    if !status.success() {
        return Err(anyhow!("provider script failed for {}", ctx.ticker));
    }
    let payload: ProviderPayload = read_json(out_path)?;
    write_json(out_path, &payload)?;
    Ok(payload)
}
