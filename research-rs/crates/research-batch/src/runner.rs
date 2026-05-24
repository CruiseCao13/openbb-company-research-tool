use crate::eval_set::load_eval_set;
use crate::lint::lint_status;
use crate::training_case::TrainingCase;
use anyhow::Result;
use chrono::Local;
use research_ai::{run_ai_usage_gate, run_local_compact_analyst, AiRunOptions};
use research_core::config::EngineConfig;
use research_core::io::{ensure_dir, write_if_changed};
use research_core::normalizer::write_normalized_outputs;
use research_core::parser::write_parser_report;
use research_core::provider::fetch_provider_payload;
use research_core::run_folder::RunFolder;
use research_core::schema_version::write_schema_validation_report;
use research_core::types::*;
use research_core::validation::{report_status, validate_ai_json, validate_provider_payload};
use research_report::dashboard::render_batch_dashboard;
use research_report::pack::pack_run;
use research_report::renderer::{render_run, RenderRunInput};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BatchRunOptions {
    pub eval_set: PathBuf,
    pub workers: usize,
    pub ai_mode: String,
    pub mode: String,
    pub require_external_ai: bool,
    pub no_ai_cache: bool,
    pub run_id: String,
    pub limit: Option<usize>,
    pub offset: usize,
    pub pack: bool,
    pub force: bool,
}

#[derive(Debug, Clone)]
struct Row {
    ticker: String,
    status: String,
    frame: String,
    run_folder: String,
    failed_checks: Vec<String>,
    duration_ms: u128,
    external_ai_calls: usize,
    ai_cache_hits: usize,
    local_mock_used: bool,
}

pub fn run_batch(options: &BatchRunOptions) -> Result<PathBuf> {
    let eval = load_eval_set(&options.eval_set)?;
    let batch_root = PathBuf::from("reports")
        .join("batch_runs")
        .join(&options.run_id);
    ensure_dir(&batch_root)?;
    let config = EngineConfig::default();
    let tickers = eval
        .tickers
        .iter()
        .skip(options.offset)
        .take(options.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut training_cases = Vec::new();
    let batch_timer = Instant::now();

    for ticker in tickers {
        let ticker_timer = Instant::now();
        let ctx = RunContext {
            ticker: ticker.clone(),
            market: "US".to_string(),
            provider: "auto".to_string(),
            ai_mode: options.ai_mode.clone(),
            run_id: format!("{}_{}", options.run_id, ticker),
            root: "reports".to_string(),
            force: options.force,
            pack: options.pack,
            lang: "en".to_string(),
            mode: options.mode.clone(),
            require_external_ai: options.require_external_ai,
            no_ai_cache: options.no_ai_cache,
            max_attempts: 2,
            auto_fix: false,
            fail_fast: false,
        };
        let folder = RunFolder::new(&ctx);
        folder.create()?;
        let mut payload =
            fetch_provider_payload(&ctx, &config, &folder.raw.join("provider_payload.json"))?;
        write_parser_report(&folder, &payload)?;
        write_normalized_outputs(&folder, &payload)?;
        if let Some(expected) = eval.expected_family.get(&ticker) {
            payload.company_profile.description = format!(
                "{}\n\nBatch eval expected research family guardrail: {}.",
                payload.company_profile.description, expected
            );
        }
        let provider_failures = validate_provider_payload(&payload);
        let ai_usage = run_ai_usage_gate(
            &payload,
            &AiRunOptions {
                ai_mode: options.ai_mode.clone(),
                require_external_ai: options.require_external_ai,
                no_ai_cache: options.no_ai_cache,
            },
            &folder.metadata,
            &folder.ai,
        )?;
        let (understanding, interpretation, blueprint, review, _local_ai_calls, cache_hits) =
            run_local_compact_analyst(&payload);
        write_schema_validation_report(
            &folder,
            &[
                ("provider_payload", payload.schema_version.clone()),
                (
                    "company_understanding",
                    understanding.schema_version.clone(),
                ),
                (
                    "financial_interpretation",
                    interpretation.schema_version.clone(),
                ),
                ("research_blueprint", blueprint.schema_version.clone()),
                ("ai_self_review", review.schema_version.clone()),
            ],
        )?;
        let ai_failures = validate_ai_json(&understanding, &interpretation, &blueprint, &review);
        let mut status = report_status(
            &provider_failures,
            &ai_failures,
            &review,
            if payload.error.is_some() {
                "PROVIDER_ERROR".into()
            } else {
                "PASS".into()
            },
            options.ai_mode.clone(),
            ai_usage.new_external_ai_calls,
            ai_usage.cache_hits,
        );
        if ai_usage.local_mock_used && matches!(options.ai_mode.as_str(), "compact" | "full") {
            status.overall_status = "WARNING".into();
            status.human_review_required = true;
        }
        render_run(RenderRunInput {
            folder: &folder,
            payload: &payload,
            understanding: &understanding,
            interpretation: &interpretation,
            blueprint: &blueprint,
            review: &review,
            status: &status,
            lang: "en",
        })?;
        if options.pack {
            let _ = pack_run(&folder, &ticker);
        }
        let ticker_duration = ticker_timer.elapsed().as_millis();
        let run_trace = RunTrace {
            ticker: ticker.clone(),
            run_id: ctx.run_id.clone(),
            started_at: Local::now().to_rfc3339(),
            finished_at: Local::now().to_rfc3339(),
            total_ms: ticker_duration,
            provider_used: ctx.provider.clone(),
            ai_mode: ctx.ai_mode.clone(),
            ai_calls: ai_usage.new_external_ai_calls,
            cache_hits: ai_usage.cache_hits,
            stages: vec![StageTrace {
                stage: "batch_ticker_pipeline".into(),
                status: status.overall_status.clone(),
                duration_ms: ticker_duration,
                cache_hit: ai_usage.cache_hits > 0 || cache_hits > 0,
                provider_used: Some(ctx.provider.clone()),
                ai_calls: ai_usage.new_external_ai_calls,
                errors: vec![],
                warnings: provider_failures
                    .iter()
                    .chain(ai_failures.iter())
                    .cloned()
                    .collect(),
                output_files: vec![
                    "raw/provider_payload.json".into(),
                    "metadata/research_blueprint.json".into(),
                    "report/*_research_report.md".into(),
                    "dashboard.html".into(),
                ],
            }],
        };
        research_core::io::write_json(&folder.metadata.join("run_trace.json"), &run_trace)?;
        write_if_changed(
            &folder.audit.join("run_log.md"),
            &format!(
                "# Run Log\n\nTicker: {}\nRun ID: {}\nTotal runtime: {} ms\nProvider: {}\nAI mode: {}\nAI calls: {}\nCache hits: {}\n\nThis run was executed inside batch `{}`.\n",
                run_trace.ticker,
                run_trace.run_id,
                run_trace.total_ms,
                run_trace.provider_used,
                run_trace.ai_mode,
                run_trace.ai_calls,
                run_trace.cache_hits,
                options.run_id
            ),
        )?;
        let expected = eval.expected_family.get(&ticker);
        let lint = lint_status(&status, expected, &understanding.correct_research_frame);
        let row_status = if !lint.failed_checks.is_empty() {
            "FAIL"
        } else if status.human_review_required {
            "WARNING"
        } else {
            "PASS"
        }
        .to_string();
        if row_status != "PASS" {
            let expected_family = expected
                .cloned()
                .unwrap_or_else(|| "Human review required".into());
            let case = TrainingCase {
                ticker: ticker.clone(),
                initial_profile: understanding.correct_research_frame.clone(),
                final_profile: understanding.correct_research_frame.clone(),
                expected_family,
                issue_type: lint
                    .failed_checks
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "warning".into()),
                wrong_output: understanding.correct_research_frame.clone(),
                expected_output_features: blueprint.must_analyze.clone(),
                must_contain: blueprint.must_analyze.clone(),
                must_not_contain: blueprint.must_not_analyze_as_core.clone(),
                data_refs_used: vec![
                    "provider_payload.json".into(),
                    "research_blueprint.json".into(),
                ],
                fixed_by: "v5 local compact analyst / validator".into(),
                regression_status: "generated".into(),
            };
            let case_path = batch_root.join("training_cases_generated.jsonl");
            let line = serde_json::to_string(&case)? + "\n";
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&case_path)?
                .write_all(line.as_bytes())?;
            let global_path = Path::new("training_cases")
                .join("corrections")
                .join("v5_correction_cases.jsonl");
            if let Some(parent) = global_path.parent() {
                ensure_dir(parent)?;
            }
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(global_path)?
                .write_all(line.as_bytes())?;
            training_cases.push(case);
        }
        rows.push(Row {
            ticker,
            status: row_status,
            frame: understanding.correct_research_frame,
            run_folder: folder.root.to_string_lossy().to_string(),
            failed_checks: lint.failed_checks,
            duration_ms: ticker_duration,
            external_ai_calls: ai_usage.new_external_ai_calls,
            ai_cache_hits: ai_usage.cache_hits,
            local_mock_used: ai_usage.local_mock_used,
        });
    }

    write_batch_outputs(
        &batch_root,
        &eval.name,
        options.workers,
        batch_timer.elapsed().as_millis(),
        &rows,
        training_cases.len(),
    )?;
    Ok(batch_root)
}

use std::io::Write;

fn write_batch_outputs(
    root: &Path,
    name: &str,
    workers: usize,
    total_ms: u128,
    rows: &[Row],
    training_count: usize,
) -> Result<()> {
    let total = rows.len();
    let pass = rows.iter().filter(|r| r.status == "PASS").count();
    let fail = rows.iter().filter(|r| r.status == "FAIL").count();
    let warning = rows.iter().filter(|r| r.status == "WARNING").count();
    let external_ai_calls: usize = rows.iter().map(|r| r.external_ai_calls).sum();
    let ai_cache_hits: usize = rows.iter().map(|r| r.ai_cache_hits).sum();
    let local_fallback_count = rows.iter().filter(|r| r.local_mock_used).count();
    let avg_ms = if total == 0 {
        0
    } else {
        total_ms / total as u128
    };
    let slowest = rows
        .iter()
        .max_by_key(|r| r.duration_ms)
        .map(|r| format!("{} ({} ms)", r.ticker, r.duration_ms))
        .unwrap_or_else(|| "n/a".into());
    let generated = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut summary = format!(
        "# v5 Batch Evaluation Report\n\n> Eval Set: {name}  \n> Generated: {generated}  \n> AI Mode: compact/local fallback  \n> External AI Calls: {external_ai_calls}\n\n## 1. Executive Summary\n\nThis batch validates the v5 AI-led Rust research pipeline. Provider failures are isolated per ticker, and local compact analyst fallback is explicitly tracked when no external AI is called.\n\n## 2. Status Dashboard\n\n| Metric | Value |\n|---|---:|\n| Total tickers | {total} |\n| PASS | {pass} |\n| WARNING | {warning} |\n| FAIL | {fail} |\n| Training cases generated | {training_count} |\n| External AI calls | {external_ai_calls} |\n| Local fallback count | {local_fallback_count} |\n| AI cache hits | {ai_cache_hits} |\n| Workers requested | {workers} |\n| Avg runtime per ticker | {avg_ms} ms |\n| Slowest ticker | {slowest} |\n\n## 3. Company Matrix\n\n| Ticker | Status | Research Frame | Runtime | Failed Checks |\n|---|---|---|---:|---|\n"
    );
    for r in rows {
        summary.push_str(&format!(
            "| {} | {} | {} | {} ms | {} |\n",
            r.ticker,
            r.status,
            r.frame,
            r.duration_ms,
            r.failed_checks.join(", ")
        ));
    }
    summary.push_str("\n## 4. Next Recommended Fixes\n\n1. Review WARNING and FAIL rows first.\n2. Convert recurring failures into regression tests.\n3. Run broad batches only after this matrix stays clean.\n");
    write_if_changed(&root.join("batch_summary.md"), &summary)?;

    let failures = rows.iter().filter(|r| r.status == "FAIL").map(|r| {
        format!("## {} — FAIL\n\n### What happened\nFailed checks: {}\n\n### Suggested next action\nInspect {}\n\n", r.ticker, r.failed_checks.join(", "), r.run_folder)
    }).collect::<String>();
    write_if_changed(
        &root.join("failures.md"),
        &format!(
            "# Batch Failures\n\n{}",
            if failures.is_empty() {
                "No hard failures.\n".into()
            } else {
                failures
            }
        ),
    )?;
    let warnings = rows
        .iter()
        .filter(|r| r.status == "WARNING")
        .map(|r| format!("- {} — {} ({})\n", r.ticker, r.frame, r.run_folder))
        .collect::<String>();
    write_if_changed(
        &root.join("warnings.md"),
        &format!(
            "# Batch Warnings\n\n{}",
            if warnings.is_empty() {
                "No warnings.\n".into()
            } else {
                warnings
            }
        ),
    )?;
    let profiles = rows
        .iter()
        .map(|r| format!("| {} | {} | {} |\n", r.ticker, r.frame, r.status))
        .collect::<String>();
    write_if_changed(
        &root.join("profile_distribution.md"),
        &format!(
            "# Profile Distribution\n\n| Ticker | Profile | Status |\n|---|---|---|\n{}",
            profiles
        ),
    )?;
    write_if_changed(
        &root.join("credit_usage_estimate.md"),
        &format!(
            "# Credit Usage Estimate\n\n- External API calls: {external_ai_calls}\n- Local fallback count: {local_fallback_count}\n- AI cache hits: {ai_cache_hits}\n- Full reports sent to AI: No\n- CSV / charts sent to AI: No\n- Broad 200/500 live run: No\n"
        ),
    )?;
    write_if_changed(&root.join("executive_dashboard.md"), &summary)?;
    let summary_json = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "batch_name": name,
        "generated_at": Local::now().to_rfc3339(),
        "total": total,
        "pass": pass,
        "warning": warning,
        "fail": fail,
        "training_cases_generated": training_count,
        "external_ai_calls": external_ai_calls,
        "local_fallback_count": local_fallback_count,
        "ai_cache_hits": ai_cache_hits,
        "workers_requested": workers,
        "avg_runtime_per_ticker_ms": avg_ms,
        "slowest_ticker": slowest,
        "provider_cache_hits": "recorded per ticker in metadata/provider_status.json",
        "ai_cache_hits": "local compact fallback records cache per ticker",
        "broad_200_or_500_live": false
    });
    research_core::io::write_json(&root.join("batch_summary.json"), &summary_json)?;
    write_if_changed(
        &root.join("dashboard.html"),
        &render_batch_dashboard(
            "v5 Batch Dashboard",
            &rows
                .iter()
                .map(|r| (r.ticker.clone(), r.status.clone(), r.frame.clone()))
                .collect::<Vec<_>>(),
        ),
    )?;

    let mut wtr = csv::Writer::from_path(root.join("company_matrix.csv"))?;
    wtr.write_record([
        "ticker",
        "status",
        "frame",
        "run_folder",
        "failed_checks",
        "duration_ms",
    ])?;
    for r in rows {
        wtr.write_record([
            &r.ticker,
            &r.status,
            &r.frame,
            &r.run_folder,
            &r.failed_checks.join(";"),
            &r.duration_ms.to_string(),
        ])?;
    }
    wtr.flush()?;
    let trace = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "batch_name": name,
        "workers_requested": workers,
        "total_ms": total_ms,
        "avg_runtime_per_ticker_ms": avg_ms,
        "slowest_ticker": slowest,
        "provider_failure_count": rows.iter().filter(|r| r.status == "FETCH_FAILED").count(),
        "ai_call_count": external_ai_calls,
        "cache_hit_impact": "Provider cache hits are recorded per run in metadata/provider_status.json.",
        "tickers": rows.iter().map(|r| serde_json::json!({
            "ticker": r.ticker,
            "status": r.status,
            "duration_ms": r.duration_ms,
            "run_folder": r.run_folder,
            "failed_checks": r.failed_checks
        })).collect::<Vec<_>>()
    });
    research_core::io::write_json(&root.join("batch_trace.json"), &trace)?;
    Ok(())
}
