use crate::config::EngineConfig;
use crate::io::write_json;
use crate::provider::fetch_provider_payload;
use crate::run_folder::RunFolder;
use crate::types::*;
use crate::validation::{
    apply_framework_challenge_guard, report_status, validate_ai_json, validate_provider_payload,
};
use anyhow::Result;
use research_ai::run_local_compact_analyst;
use research_report::pack::pack_run;
use research_report::renderer::{render_run, RenderRunInput};

#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub run_folder: RunFolder,
    pub status: ReportStatus,
    pub frame: String,
    pub pack_path: Option<String>,
}

pub fn run_pipeline(ctx: &RunContext, config: &EngineConfig) -> Result<PipelineResult> {
    let folder = RunFolder::new(ctx);
    folder.create()?;
    let raw_path = folder.raw.join("provider_payload.json");
    let payload = fetch_provider_payload(ctx, config, &raw_path)?;
    write_json(&raw_path, &payload)?;

    let provider_failures = validate_provider_payload(&payload);
    let provider_status = if payload.error.is_some() {
        "PROVIDER_ERROR".to_string()
    } else if provider_failures.is_empty() {
        "PASS".to_string()
    } else {
        "WARNING".to_string()
    };

    let (mut understanding, mut interpretation, mut blueprint, mut review, ai_calls, cache_hits) =
        run_local_compact_analyst(&payload);
    let mut ai_failures = apply_framework_challenge_guard(
        &payload,
        &mut understanding,
        &mut interpretation,
        &mut blueprint,
        &mut review,
    );
    ai_failures.extend(validate_ai_json(
        &understanding,
        &interpretation,
        &blueprint,
        &review,
    ));
    let status = report_status(
        &provider_failures,
        &ai_failures,
        &review,
        provider_status,
        ctx.ai_mode.clone(),
        ai_calls,
        cache_hits,
    );
    render_run(RenderRunInput {
        folder: &folder,
        payload: &payload,
        understanding: &understanding,
        interpretation: &interpretation,
        blueprint: &blueprint,
        review: &review,
        status: &status,
        lang: &ctx.lang,
    })?;
    let pack_path = if ctx.pack {
        Some(pack_run(&folder, &ctx.ticker)?.to_string_lossy().to_string())
    } else {
        None
    };
    Ok(PipelineResult {
        run_folder: folder,
        status,
        frame: understanding.correct_research_frame,
        pack_path,
    })
}
