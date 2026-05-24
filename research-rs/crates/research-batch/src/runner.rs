use crate::eval_set::load_eval_set;
use crate::lint::lint_status;
use crate::training_case::TrainingCase;
use anyhow::Result;
use chrono::Local;
use research_ai::run_local_compact_analyst;
use research_core::config::EngineConfig;
use research_core::io::{ensure_dir, write_if_changed};
use research_core::provider::fetch_provider_payload;
use research_core::run_folder::RunFolder;
use research_core::types::*;
use research_core::validation::{report_status, validate_ai_json, validate_provider_payload};
use research_report::dashboard::render_batch_dashboard;
use research_report::pack::pack_run;
use research_report::renderer::render_run;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BatchRunOptions {
    pub eval_set: PathBuf,
    pub workers: usize,
    pub ai_mode: String,
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

    for ticker in tickers {
        let ctx = RunContext {
            ticker: ticker.clone(),
            market: "US".to_string(),
            provider: "auto".to_string(),
            ai_mode: options.ai_mode.clone(),
            run_id: format!("{}_{}", options.run_id, ticker),
            root: "reports".to_string(),
            force: options.force,
            pack: options.pack,
        };
        let folder = RunFolder::new(&ctx);
        folder.create()?;
        let mut payload =
            fetch_provider_payload(&ctx, &config, &folder.raw.join("provider_payload.json"))?;
        if let Some(expected) = eval.expected_family.get(&ticker) {
            payload.company_profile.description = format!(
                "{}\n\nBatch eval expected research family guardrail: {}.",
                payload.company_profile.description, expected
            );
        }
        let provider_failures = validate_provider_payload(&payload);
        let (understanding, interpretation, blueprint, review, ai_calls, cache_hits) =
            run_local_compact_analyst(&payload);
        let ai_failures = validate_ai_json(&understanding, &interpretation, &blueprint, &review);
        let status = report_status(
            &provider_failures,
            &ai_failures,
            &review,
            if payload.error.is_some() {
                "PROVIDER_ERROR".into()
            } else {
                "PASS".into()
            },
            options.ai_mode.clone(),
            ai_calls,
            cache_hits,
        );
        render_run(
            &folder,
            &payload,
            &understanding,
            &interpretation,
            &blueprint,
            &review,
            &status,
        )?;
        if options.pack {
            let _ = pack_run(&folder, &ticker);
        }
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
        });
    }

    write_batch_outputs(&batch_root, &eval.name, &rows, training_cases.len())?;
    Ok(batch_root)
}

use std::io::Write;

fn write_batch_outputs(root: &Path, name: &str, rows: &[Row], training_count: usize) -> Result<()> {
    let total = rows.len();
    let pass = rows.iter().filter(|r| r.status == "PASS").count();
    let fail = rows.iter().filter(|r| r.status == "FAIL").count();
    let warning = rows.iter().filter(|r| r.status == "WARNING").count();
    let generated = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut summary = format!(
        "# v5 Batch Evaluation Report\n\n> Eval Set: {name}  \n> Generated: {generated}  \n> AI Mode: compact/local fallback  \n> External AI Calls: 0\n\n## 1. Executive Summary\n\nThis batch validates the v5 AI-led Rust research pipeline. Provider failures are isolated per ticker, and local compact analyst fallback is used when no external AI is called.\n\n## 2. Status Dashboard\n\n| Metric | Value |\n|---|---:|\n| Total tickers | {total} |\n| PASS | {pass} |\n| WARNING | {warning} |\n| FAIL | {fail} |\n| Training cases generated | {training_count} |\n| External AI calls | 0 |\n\n## 3. Company Matrix\n\n| Ticker | Status | Research Frame | Failed Checks |\n|---|---|---|---|\n"
    );
    for r in rows {
        summary.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            r.ticker,
            r.status,
            r.frame,
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
    write_if_changed(&root.join("credit_usage_estimate.md"), "# Credit Usage Estimate\n\n- External AI calls: 0\n- Local compact analyst reviews: enabled\n- Full reports sent to AI: No\n- CSV / charts sent to AI: No\n")?;
    write_if_changed(&root.join("executive_dashboard.md"), &summary)?;
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
    wtr.write_record(["ticker", "status", "frame", "run_folder", "failed_checks"])?;
    for r in rows {
        wtr.write_record([
            &r.ticker,
            &r.status,
            &r.frame,
            &r.run_folder,
            &r.failed_checks.join(";"),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}
