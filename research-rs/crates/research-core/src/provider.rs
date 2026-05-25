use crate::cache::digest_str;
use crate::config::EngineConfig;
use crate::io::{ensure_dir, read_json, write_if_changed, write_json};
use crate::types::{ProviderPayload, ProviderStatus, RunContext};
use anyhow::{anyhow, Result};
use chrono::Local;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROVIDER_COMMON: &str = "providers/provider_common.py";

pub fn discover_repo_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    discover_repo_root_from(&current_dir)
}

pub fn discover_repo_root_from(start: &Path) -> Result<PathBuf> {
    if start.join(PROVIDER_COMMON).exists() {
        return Ok(start.to_path_buf());
    }
    for parent in start.ancestors().skip(1) {
        if parent.join(PROVIDER_COMMON).exists() {
            return Ok(parent.to_path_buf());
        }
    }
    if let Ok(root) = std::env::var("RESEARCH_ENGINE_ROOT") {
        let root = PathBuf::from(root);
        if root.join(PROVIDER_COMMON).exists() {
            return Ok(root);
        }
    }
    Err(anyhow!(
        "providers/provider_common.py not found. Set RESEARCH_ENGINE_ROOT=/path/to/openbb-company-research-tool"
    ))
}

pub fn resolve_repo_path(relative: &str) -> Result<PathBuf> {
    let root = discover_repo_root()?;
    Ok(root.join(relative))
}

pub fn resolve_provider_script(configured: &str) -> Result<PathBuf> {
    let root = discover_repo_root()?;
    let configured_path = Path::new(configured);
    if configured_path.is_absolute() {
        return Ok(configured_path.to_path_buf());
    }
    let file_name = configured_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("provider_common.py");
    Ok(root.join("providers").join(file_name))
}

pub fn resolve_python() -> Result<PathBuf> {
    let root = discover_repo_root()?;
    let venv_python = root.join(".venv/bin/python");
    if venv_python.exists() {
        Ok(venv_python)
    } else {
        Ok(PathBuf::from("python3"))
    }
}

pub fn fetch_provider_payload(
    ctx: &RunContext,
    config: &EngineConfig,
    out_path: &Path,
) -> Result<ProviderPayload> {
    if out_path.exists() && !ctx.force {
        let payload: ProviderPayload = read_json(out_path)?;
        write_provider_status(ctx, out_path, true, 0, "", "", "PASS")?;
        return Ok(payload);
    }

    let python = resolve_python()?;
    let provider_script = resolve_provider_script(&config.provider_script)?;
    let output = Command::new(python)
        .arg(&provider_script)
        .arg("--ticker")
        .arg(&ctx.ticker)
        .arg("--market")
        .arg(&ctx.market)
        .arg("--provider")
        .arg(&ctx.provider)
        .arg("--out")
        .arg(out_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        write_provider_status(ctx, out_path, false, 1, &stdout, &stderr, "PROVIDER_ERROR")?;
        return Err(anyhow!(
            "provider script failed for {}: {}",
            ctx.ticker,
            stderr.trim()
        ));
    }
    let payload: ProviderPayload = read_json(out_path)?;
    write_json(out_path, &payload)?;
    let status = if payload.error.is_some() {
        "PROVIDER_ERROR"
    } else {
        "PASS"
    };
    write_provider_status(ctx, out_path, false, 1, &stdout, &stderr, status)?;
    Ok(payload)
}

fn excerpt(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.len() > 600 {
        format!("{}...", &trimmed[..600])
    } else {
        trimmed.to_string()
    }
}

fn write_provider_status(
    ctx: &RunContext,
    out_path: &Path,
    cache_hit: bool,
    attempts: usize,
    stdout: &str,
    stderr: &str,
    status: &str,
) -> Result<()> {
    let root = out_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow!("invalid provider output path"))?;
    let metadata = root.join("metadata");
    let audit = root.join("audit");
    ensure_dir(&metadata)?;
    ensure_dir(&audit)?;
    let cache_key = digest_str(&format!(
        "{}:{}:{}:{}:{}",
        ctx.ticker, ctx.market, ctx.provider, ctx.run_id, ctx.force
    ));
    let provider_status = ProviderStatus {
        ticker: ctx.ticker.clone(),
        provider: ctx.provider.clone(),
        status: status.to_string(),
        cache_hit,
        attempts,
        stdout_excerpt: excerpt(stdout),
        stderr_excerpt: excerpt(stderr),
        user_message: if status == "PASS" {
            "Provider payload is available and parsed into the locked-data contract.".to_string()
        } else {
            "Provider fetch failed; this ticker should be isolated and classified without crashing the batch.".to_string()
        },
        suggested_next_action: if status == "PASS" {
            "Continue to validation and report rendering.".to_string()
        } else {
            "Retry with --force, inspect provider stderr, or try a fallback provider.".to_string()
        },
    };
    write_json(&metadata.join("provider_status.json"), &provider_status)?;
    write_if_changed(
        &metadata.join("provider_cache_info.json"),
        &format!(
            "{{\n  \"cache_key\": \"{}\",\n  \"cache_hit\": {},\n  \"created_at\": \"{}\",\n  \"source\": \"{}\"\n}}\n",
            cache_key,
            cache_hit,
            Local::now().to_rfc3339(),
            ctx.provider
        ),
    )?;
    write_if_changed(
        &audit.join("provider_validation.md"),
        &format!(
            "# Provider Validation\n\nStatus: {}\n\nProvider: {}\nCache hit: {}\nAttempts: {}\n\n## User Message\n\n{}\n\n## Suggested Next Action\n\n{}\n\n## stderr excerpt\n\n```text\n{}\n```\n",
            provider_status.status,
            provider_status.provider,
            provider_status.cache_hit,
            provider_status.attempts,
            provider_status.user_message,
            provider_status.suggested_next_action,
            provider_status.stderr_excerpt
        ),
    )?;
    Ok(())
}
