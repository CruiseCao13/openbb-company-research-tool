use crate::quality::{score_report, QualityReview};
use crate::runner::{run_batch, BatchRunOptions};
use crate::training_case::expected_features_for_ticker;
use anyhow::Result;
use chrono::Local;
use research_core::io::{ensure_dir, write_if_changed, write_json};
use research_core::paths::{training_root_dir, training_runs_dir};
use research_core::types::SCHEMA_VERSION;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TrainingRunOptions {
    pub eval_set: PathBuf,
    pub stage: String,
    pub workers: usize,
    pub ai_mode: String,
    pub require_external_ai: bool,
    pub no_ai_cache: bool,
    pub budget_calls: usize,
    pub max_iterations: usize,
    pub quality_threshold: u8,
    pub run_id: String,
    pub limit: Option<usize>,
    pub offset: usize,
    pub resume: bool,
    pub only_failed: bool,
    pub only_weak: bool,
    pub only_wrong_framework: bool,
    pub only_provider_failed: bool,
    pub only_low_quality: bool,
    pub force: bool,
    pub pack: bool,
}

#[derive(Debug, Serialize)]
struct TrainingCaseV5 {
    schema_version: String,
    ticker: String,
    market: String,
    provider: String,
    run_id: String,
    ai_source: String,
    model: String,
    prompt_versions: Vec<String>,
    issue_type: String,
    severity: String,
    bad_output_excerpt: String,
    why_bad: String,
    expected_behavior: String,
    must_contain: Vec<String>,
    must_not_contain: Vec<String>,
    data_refs: Vec<String>,
    evidence_refs: Vec<String>,
    fix_target: String,
    regression_status: String,
    human_review_required: bool,
}

#[derive(Debug, Serialize)]
struct ReportFeature {
    ticker: String,
    market: String,
    sector: String,
    industry: String,
    asset_profile: String,
    company_description_keywords: Vec<String>,
    ai_source: String,
    prompt_versions: Vec<String>,
    data_coverage: String,
    provider_status: String,
    money_flow_score: u8,
    chart_table_score: u8,
    language_score: u8,
    wrong_framework_flag: bool,
    hallucinated_terms: Vec<String>,
    unsupported_claim_count: usize,
    hard_failure_count: usize,
    quality_score: u8,
    issue_types: Vec<String>,
}

#[derive(Debug)]
struct MatrixRow {
    ticker: String,
    status: String,
    frame: String,
    run_folder: PathBuf,
    failed_checks: String,
}

fn read_json(path: &Path) -> serde_json::Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(serde_json::Value::Null)
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn issue_type(review: &QualityReview, row: &MatrixRow) -> String {
    let failed = format!(
        "{} {} {} {}",
        row.status,
        row.failed_checks,
        review.hard_failures.join(" "),
        review.specific_issues.join(" ")
    )
    .to_lowercase();
    if failed.contains("wrong_framework") {
        "wrong_profile"
    } else if failed.contains("generic_money_flow") || failed.contains("money_flow") {
        "weak_money_flow"
    } else if failed.contains("chart") {
        "generic_chart_explanation"
    } else if failed.contains("table") {
        "table_quality_bad"
    } else if failed.contains("unsupported_numeric") {
        "unsupported_numeric_claim"
    } else if failed.contains("unsupported") {
        "unsupported_claim"
    } else if failed.contains("provider") {
        "provider_data_gap"
    } else if failed.contains("language") || failed.contains("generic") {
        "generic_risk_language"
    } else if review.grade == "WEAK" || review.grade == "FAIL" {
        "weak_company_understanding"
    } else {
        "positive_case"
    }
    .to_string()
}

fn severity(review: &QualityReview, issue: &str) -> String {
    if review.grade == "FAIL" || issue == "wrong_profile" || issue.contains("unsupported") {
        "HIGH"
    } else if review.grade == "WEAK" || review.quality_score < 70 {
        "MEDIUM"
    } else {
        "LOW"
    }
    .to_string()
}

fn fix_target(issue: &str) -> String {
    match issue {
        "wrong_profile" | "unsupported_claim" | "unsupported_numeric_claim" => "validator",
        "weak_money_flow" | "weak_financial_interpretation" => "prompt",
        "generic_chart_explanation" => "chart",
        "table_quality_bad" => "table",
        "provider_data_gap" | "provider_schema_bad" => "provider",
        "generic_risk_language" | "translationese" => "language",
        _ => "prompt",
    }
    .to_string()
}

fn description_keywords(description: &str) -> Vec<String> {
    let lower = description.to_lowercase();
    [
        "space",
        "lunar",
        "nasa",
        "semiconductor",
        "bank",
        "insurance",
        "shipping",
        "biotech",
        "reit",
        "utility",
        "retail",
        "energy",
    ]
    .iter()
    .filter(|needle| lower.contains(**needle))
    .map(|needle| (*needle).to_string())
    .collect()
}

fn training_case(row: &MatrixRow, review: &QualityReview, issue: &str) -> TrainingCaseV5 {
    let payload = read_json(&row.run_folder.join("raw/provider_payload.json"));
    let usage = read_json(&row.run_folder.join("metadata/ai_usage.json"));
    let blueprint = read_json(&row.run_folder.join("metadata/research_blueprint.json"));
    let report = read_text(
        &row.run_folder
            .join("report")
            .join(format!("{}_research_report.md", row.ticker)),
    );
    let ai_source = usage
        .get("ai_provenance")
        .and_then(|p| p.get("source"))
        .or_else(|| {
            usage
                .get("tasks")
                .and_then(|tasks| tasks.get(0))
                .and_then(|t| t.get("source"))
        })
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let model = usage
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let provider = payload
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let market = payload
        .get("market")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let prompt_versions = read_json(&row.run_folder.join("metadata/prompt_versions.json"))
        .as_object()
        .map(|obj| {
            obj.values()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();
    let must_contain = blueprint
        .get("must_analyze")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_else(|| vec!["company identity".into(), "money flow".into()]);
    let must_contain = expected_features_for_ticker(&row.ticker, must_contain);
    let must_not_contain = blueprint
        .get("must_not_analyze_as_core")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_else(|| vec!["wrong framework".into(), "unsupported numeric claim".into()]);
    TrainingCaseV5 {
        schema_version: SCHEMA_VERSION.to_string(),
        ticker: row.ticker.clone(),
        market,
        provider,
        run_id: row
            .run_folder
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        ai_source,
        model,
        prompt_versions,
        issue_type: issue.to_string(),
        severity: severity(review, issue),
        bad_output_excerpt: report.chars().take(500).collect(),
        why_bad: if review.hard_failures.is_empty() {
            review.specific_issues.join("; ")
        } else {
            review.hard_failures.join("; ")
        },
        expected_behavior: "Use provider identity, locked data, evidence map, money-flow specifics, valuation boundaries, and explicit data gaps; never promote local/mock or wrong-framework output as positive training data.".into(),
        must_contain,
        must_not_contain,
        data_refs: vec![
            "raw/provider_payload.json".into(),
            "metadata/company_understanding.json".into(),
            "metadata/financial_interpretation.json".into(),
            "metadata/research_blueprint.json".into(),
        ],
        evidence_refs: vec![
            "metadata/evidence_map.json".into(),
            "audit/validator_report.md".into(),
            "audit/money_flow_quality_report.md".into(),
        ],
        fix_target: fix_target(issue),
        regression_status: "new".into(),
        human_review_required: review.human_review_required,
    }
}

fn report_feature(row: &MatrixRow, review: &QualityReview, issue: &str) -> ReportFeature {
    let payload = read_json(&row.run_folder.join("raw/provider_payload.json"));
    let usage = read_json(&row.run_folder.join("metadata/ai_usage.json"));
    let money = read_json(
        &row.run_folder
            .join("metadata/money_flow_specificity_score.json"),
    );
    let chart = read_json(&row.run_folder.join("metadata/chart_table_quality.json"));
    let language = read_json(&row.run_folder.join("metadata/language_quality_score.json"));
    let profile = payload
        .get("company_profile")
        .unwrap_or(&serde_json::Value::Null);
    let description = profile
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let report = read_text(
        &row.run_folder
            .join("report")
            .join(format!("{}_research_report.md", row.ticker)),
    )
    .to_lowercase();
    let hallucinated_terms = [
        "wireless service revenue",
        "broadband / network revenue",
        "subscriber churn",
        "regulated telecom",
    ]
    .iter()
    .filter(|term| report.contains(**term))
    .map(|term| (*term).to_string())
    .collect::<Vec<_>>();
    ReportFeature {
        ticker: row.ticker.clone(),
        market: payload
            .get("market")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        sector: profile
            .get("sector")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        industry: profile
            .get("industry")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        asset_profile: row.frame.clone(),
        company_description_keywords: description_keywords(description),
        ai_source: usage
            .get("ai_provenance")
            .and_then(|p| p.get("source"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        prompt_versions: read_json(&row.run_folder.join("metadata/prompt_versions.json"))
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        data_coverage: if payload.get("error").is_some() && !payload.get("error").unwrap().is_null()
        {
            "provider_error".into()
        } else {
            "compact_provider_payload".into()
        },
        provider_status: read_json(&row.run_folder.join("metadata/provider_status.json"))
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        money_flow_score: money
            .get("money_flow_specificity_score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8,
        chart_table_score: chart
            .get("chart_explanation_score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8,
        language_score: language
            .get("language_quality_score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8,
        wrong_framework_flag: row.failed_checks.contains("wrong_framework")
            || review
                .hard_failures
                .iter()
                .any(|failure| failure.contains("wrong_framework")),
        hallucinated_terms,
        unsupported_claim_count: review
            .hard_failures
            .iter()
            .filter(|failure| failure.contains("unsupported"))
            .count(),
        hard_failure_count: review.hard_failures.len(),
        quality_score: review.quality_score,
        issue_types: vec![issue.to_string()],
    }
}

pub fn run_training(options: &TrainingRunOptions) -> Result<PathBuf> {
    let training_root = training_runs_dir()?.join(&options.run_id);
    ensure_dir(&training_root)?;
    ensure_training_directories()?;
    let batch_root = run_batch(&BatchRunOptions {
        eval_set: options.eval_set.clone(),
        workers: options.workers,
        ai_mode: options.ai_mode.clone(),
        mode: "batch".into(),
        require_external_ai: options.require_external_ai,
        no_ai_cache: options.no_ai_cache,
        run_id: format!("{}_batch", options.run_id),
        limit: options.limit,
        offset: options.offset,
        pack: options.pack,
        force: options.force,
    })?;
    write_training_outputs(&training_root, &batch_root, options)?;
    Ok(training_root)
}

fn read_matrix(batch_root: &Path) -> Result<Vec<MatrixRow>> {
    let mut rdr = csv::Reader::from_path(batch_root.join("company_matrix.csv"))?;
    let headers = rdr.headers()?.clone();
    let idx = |name: &str| headers.iter().position(|h| h == name).unwrap_or(usize::MAX);
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record?;
        rows.push(MatrixRow {
            ticker: record.get(idx("ticker")).unwrap_or_default().to_string(),
            status: record.get(idx("status")).unwrap_or_default().to_string(),
            frame: record.get(idx("frame")).unwrap_or_default().to_string(),
            run_folder: PathBuf::from(record.get(idx("run_folder")).unwrap_or_default()),
            failed_checks: record
                .get(idx("failed_checks"))
                .unwrap_or_default()
                .to_string(),
        });
    }
    Ok(rows)
}

fn write_jsonl<T: Serialize>(path: &Path, items: &[T]) -> Result<()> {
    let mut file = fs::File::create(path)?;
    for item in items {
        writeln!(file, "{}", serde_json::to_string(item)?)?;
    }
    Ok(())
}

fn write_training_outputs(
    training_root: &Path,
    batch_root: &Path,
    options: &TrainingRunOptions,
) -> Result<()> {
    let rows = read_matrix(batch_root)?;
    let mut reviews = Vec::new();
    let mut cases = Vec::new();
    let mut features = Vec::new();
    let mut issue_counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        let review = score_report(&row.run_folder, &row.ticker, &row.frame);
        let mut issue = issue_type(&review, row);
        let ai_source = read_json(&row.run_folder.join("metadata/ai_usage.json"))
            .get("ai_provenance")
            .and_then(|p| p.get("source"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        if issue == "positive_case" && ai_source != "external_openai" {
            issue = "local_mock_case".into();
        }
        *issue_counts.entry(issue.clone()).or_default() += 1;
        let include_case = issue != "positive_case"
            || (review.quality_score >= options.quality_threshold && !review.human_review_required);
        if include_case {
            cases.push(training_case(row, &review, &issue));
        }
        features.push(report_feature(row, &review, &issue));
        reviews.push(review);
    }
    let total = reviews.len();
    let avg = if total == 0 {
        0.0
    } else {
        reviews.iter().map(|r| r.quality_score as f64).sum::<f64>() / total as f64
    };
    let fail = reviews.iter().filter(|r| r.grade == "FAIL").count();
    let weak = reviews.iter().filter(|r| r.grade == "WEAK").count();
    let external_calls = read_json(&batch_root.join("batch_summary.json"))
        .get("new_external_ai_calls")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let local_fallback = read_json(&batch_root.join("batch_summary.json"))
        .get("local_fallback_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    write_quality_matrix(training_root, &reviews)?;
    write_json(&training_root.join("quality_matrix.json"), &reviews)?;
    write_json(
        &training_root.join("quality_judge_provenance.json"),
        &serde_json::json!({
            "source": if options.require_external_ai { "external_openai_required" } else { "local_deterministic_quality_layer" },
            "external_ai_used": options.require_external_ai && external_calls > 0,
            "local_mock_used": !options.require_external_ai,
            "model": if options.require_external_ai { "configured-openai-model" } else { "local-deterministic-quality-layer" },
            "prompt_version": "content_quality_judge_v1",
            "cache_hit": false,
            "new_external_ai_call": options.require_external_ai && external_calls > 0,
            "note": if options.require_external_ai && external_calls == 0 {
                "External quality judge was required, but no external calls were recorded; treat this training run as not externally verified."
            } else if options.require_external_ai {
                "External AI was required for this training run; inspect ai_usage records for per-task proof."
            } else {
                "Quality score is from deterministic local training layer, not an external AI judge."
            }
        }),
    )?;
    write_jsonl(
        &training_root.join("training_cases_generated.jsonl"),
        &cases,
    )?;
    write_jsonl(
        &training_root_dir()?
            .join("cases")
            .join("generated")
            .join(format!("{}.jsonl", options.run_id)),
        &cases,
    )?;
    write_jsonl(
        &training_root.join("regression_cases_generated.jsonl"),
        &cases,
    )?;
    write_jsonl(
        &training_root_dir()?
            .join("ml_features")
            .join("report_features.jsonl"),
        &features,
    )?;
    write_training_markdown(
        training_root,
        batch_root,
        options,
        &reviews,
        &cases,
        &issue_counts,
        avg,
        fail,
        weak,
        external_calls,
        local_fallback,
    )?;
    Ok(())
}

fn write_quality_matrix(root: &Path, reviews: &[QualityReview]) -> Result<()> {
    let mut wtr = csv::Writer::from_path(root.join("quality_matrix.csv"))?;
    wtr.write_record([
        "ticker",
        "quality_score",
        "grade",
        "hard_failures",
        "specific_issues",
        "human_review_required",
        "run_folder",
    ])?;
    for review in reviews {
        wtr.write_record([
            &review.ticker,
            &review.quality_score.to_string(),
            &review.grade,
            &review.hard_failures.join(";"),
            &review.specific_issues.join(";"),
            &review.human_review_required.to_string(),
            &review.run_folder,
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_training_markdown(
    root: &Path,
    batch_root: &Path,
    options: &TrainingRunOptions,
    reviews: &[QualityReview],
    cases: &[TrainingCaseV5],
    issue_counts: &BTreeMap<String, usize>,
    avg: f64,
    fail: usize,
    weak: usize,
    external_calls: u64,
    local_fallback: u64,
) -> Result<()> {
    let issue_lines = issue_counts
        .iter()
        .map(|(issue, count)| format!("| {issue} | {count} | {} |", fix_target(issue)))
        .collect::<Vec<_>>()
        .join("\n");
    let summary = format!(
        "# v5 AI Quality Training Summary\n\n> Stage: {}  \n> Eval set: {}  \n> Generated: {}  \n> External AI required: {}  \n> AI mode: {}  \n> Budget calls: {}  \n> Batch source: `{}`\n\n## Results\n\n| Metric | Value |\n|---|---:|\n| Reports scored | {} |\n| Average quality score | {:.1} |\n| FAIL | {} |\n| WEAK | {} |\n| Training cases generated | {} |\n| New external AI calls | {} |\n| Local fallback reports | {} |\n\n## Stage Gate\n\nThis is system-level training, not fine-tuning. Local/mock outputs are allowed for fixture sanity, but they cannot become high-quality positive cases.\n",
        options.stage,
        options.eval_set.display(),
        Local::now().to_rfc3339(),
        options.require_external_ai,
        options.ai_mode,
        options.budget_calls,
        batch_root.display(),
        reviews.len(),
        avg,
        fail,
        weak,
        cases.len(),
        external_calls,
        local_fallback
    );
    write_if_changed(&root.join("training_summary.md"), &summary)?;
    write_if_changed(
        &root.join("issue_distribution.md"),
        &format!(
            "# Issue Distribution\n\n| Issue type | Count | Fix target |\n|---|---:|---|\n{}\n",
            if issue_lines.is_empty() {
                "| none | 0 | none |".to_string()
            } else {
                issue_lines.clone()
            }
        ),
    )?;
    write_case_file(root, "wrong_framework_cases.md", reviews, "wrong_framework")?;
    write_case_file(root, "weak_money_flow_cases.md", reviews, "money_flow")?;
    write_case_file(root, "weak_chart_table_cases.md", reviews, "chart")?;
    write_case_file(root, "unsupported_claim_cases.md", reviews, "unsupported")?;
    write_case_file(root, "provider_failure_cases.md", reviews, "provider")?;
    write_if_changed(
        &root.join("prompt_improvement_suggestions.md"),
        "# Prompt Improvement Suggestions\n\n- Add negative examples from `training_cases_generated.jsonl` before changing default prompt versions.\n- Keep provider identity and business description ahead of generic sector labels.\n- Require data-limited boundaries when revenue engines are inferred.\n",
    )?;
    write_if_changed(
        &root.join("validator_improvement_suggestions.md"),
        "# Validator Improvement Suggestions\n\n- Promote recurring wrong-framework cases into deterministic guards.\n- Treat generic money flow, unsupported numeric claims, and provenance mismatch as status blockers.\n- Add tests for every new validator rule before accepting a prompt change.\n",
    )?;
    write_if_changed(
        &root.join("iteration_log.md"),
        "# Training Iteration Log\n\n| Iteration | Action | Result |\n|---|---|---|\n| 1 | Run batch, score quality, classify issues, generate cases | Completed |\n\nNo prompt default was changed automatically in this pass.\n",
    )?;
    write_if_changed(
        &root.join("quality_score_trend.md"),
        &format!(
            "# Quality Score Trend\n\n| Metric | Current run |\n|---|---:|\n| Reports scored | {} |\n| Average quality score | {:.1} |\n| FAIL | {} |\n| WEAK | {} |\n| Training cases generated | {} |\n\nNo prior training run was selected for automatic comparison. Use this file as the baseline for the next staged run.\n",
            reviews.len(),
            avg,
            fail,
            weak,
            cases.len()
        ),
    )?;
    write_if_changed(
        &root.join("failure_delta.md"),
        &format!(
            "# Failure Delta\n\n| Failure metric | Current run | Previous run | Delta |\n|---|---:|---:|---:|\n| FAIL | {} | Not selected | Not available |\n| WEAK | {} | Not selected | Not available |\n| New external AI calls | {} | Not selected | Not available |\n| Local fallback reports | {} | Not selected | Not available |\n\nNo automatic source patch or default prompt switch is made from this delta. It is an audit artifact for comparing staged quality iterations.\n",
            fail, weak, external_calls, local_fallback
        ),
    )?;
    write_if_changed(
        &root.join("cost_report.md"),
        &format!(
            "# Training Cost Report\n\n- New external AI calls: {external_calls}\n- Local fallback reports: {local_fallback}\n- Budget calls: {}\n- Calls remaining: {}\n- Stopped due to budget: {}\n- Full reports sent to AI judge: No\n- Full CSV/charts sent to AI: No\n- Cache hits: see batch summary at `{}`\n",
            options.budget_calls,
            options.budget_calls.saturating_sub(external_calls as usize),
            external_calls as usize >= options.budget_calls && options.budget_calls > 0,
            batch_root.join("batch_summary.md").display()
        ),
    )?;
    write_if_changed(
        &root.join("similar_failure_retrieval.md"),
        &format!(
            "# Similar Failure Retrieval\n\nThis baseline retrieval groups reports by deterministic issue type. It is intentionally conservative and does not use investment conclusions.\n\n## Current issue clusters\n\n{}\n\nNext step: compare future cases against these issue clusters and the generated ML feature rows in `training/ml_features/report_features.jsonl`.\n",
            if issue_lines.is_empty() {
                "| none | 0 | none |".to_string()
            } else {
                issue_lines.clone()
            }
        ),
    )?;
    write_if_changed(
        &root.join("self_repair_plan.md"),
        "# Self-Repair Plan\n\nAllowed targets: prompt templates, validator rules, compact payload fields, chart/table explanation rules, renderer wording, and quality rubric. Locked data and numeric values are out of scope.\n",
    )?;
    write_if_changed(
        &root.join("self_repair_diff.md"),
        "# Self-Repair Diff\n\nNo automatic source patch was applied by the training loop. Suggested changes are listed in prompt and validator suggestion reports.\n",
    )?;
    write_if_changed(
        &root.join("model_improvement_review.md"),
        &format!(
            "# Model Improvement Review\n\n## What improved\n\nThe training loop produced scored quality rows, issue types, fix targets, regression candidates, and ML feature records.\n\n## What remains weak\n\nReview FAIL/WEAK cases and recurring issue types before advancing stages.\n\n## Stage recommendation\n\nAverage score: {:.1}. Threshold: {}. Proceed only if severe wrong-framework and unsupported numeric cases are reviewed.\n\n## Overfit risk\n\nDo not tune only one ticker. Run `eval_sets/regression_hard_cases.yaml` plus a broad_30 subset after any prompt or validator change.\n",
            avg, options.quality_threshold
        ),
    )?;
    write_if_changed(
        &root.join("final_acceptance.md"),
        &format!(
            "# Final Acceptance\n\n| Gate | Status |\n|---|---|\n| Reports scored | {} |\n| Average quality >= threshold | {} |\n| External AI calls recorded | {} |\n| Training cases generated | {} |\n| Local/mock positives blocked | PASS |\n\nTRAINING_STAGE_READY = {}\n",
            reviews.len(),
            if avg >= options.quality_threshold as f64 { "PASS" } else { "WARNING" },
            external_calls,
            cases.len(),
            avg >= options.quality_threshold as f64 && fail == 0
        ),
    )?;
    write_training_dashboard(
        root,
        reviews,
        issue_counts,
        avg,
        external_calls,
        local_fallback,
    )?;
    Ok(())
}

fn write_case_file(root: &Path, name: &str, reviews: &[QualityReview], needle: &str) -> Result<()> {
    let body = reviews
        .iter()
        .filter(|r| {
            r.hard_failures
                .iter()
                .chain(r.specific_issues.iter())
                .any(|i| i.contains(needle))
        })
        .map(|r| {
            format!(
                "## {} — {} ({})\n\nIssues: {}\n\nRun folder: `{}`\n\n",
                r.ticker,
                r.grade,
                r.quality_score,
                r.hard_failures
                    .iter()
                    .chain(r.specific_issues.iter())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; "),
                r.run_folder
            )
        })
        .collect::<String>();
    write_if_changed(
        &root.join(name),
        &format!(
            "# {}\n\n{}",
            name.trim_end_matches(".md").replace('_', " "),
            if body.is_empty() {
                "No cases detected.\n".to_string()
            } else {
                body
            }
        ),
    )?;
    Ok(())
}

fn write_training_dashboard(
    root: &Path,
    reviews: &[QualityReview],
    issues: &BTreeMap<String, usize>,
    avg: f64,
    external_calls: u64,
    local_fallback: u64,
) -> Result<()> {
    let rows = reviews
        .iter()
        .map(|r| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                r.ticker, r.quality_score, r.grade, r.human_review_required
            )
        })
        .collect::<String>();
    let issue_list = issues
        .iter()
        .map(|(issue, count)| format!("<li>{issue}: {count}</li>"))
        .collect::<String>();
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>v5 Training Dashboard</title><style>body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#101418;color:#e7edf2;padding:32px}}.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px}}.card{{border:1px solid #2b3540;background:#161d24;border-radius:8px;padding:16px}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #2b3540;padding:8px}}</style></head><body><h1>v5 Training Dashboard</h1><div class=\"grid\"><div class=\"card\"><b>Average quality</b><p>{avg:.1}</p></div><div class=\"card\"><b>External AI calls</b><p>{external_calls}</p></div><div class=\"card\"><b>Local fallback reports</b><p>{local_fallback}</p></div><div class=\"card\"><b>Issues</b><ul>{issue_list}</ul></div></div><h2>Quality Matrix</h2><table><thead><tr><th>Ticker</th><th>Score</th><th>Grade</th><th>Human review</th></tr></thead><tbody>{rows}</tbody></table></body></html>"
    );
    write_if_changed(&root.join("dashboard.html"), &html)?;
    Ok(())
}

fn ensure_training_directories() -> Result<()> {
    let root = training_root_dir()?;
    for dir in [
        "datasets",
        "cases",
        "cases/generated",
        "negative_cases",
        "positive_cases",
        "prompt_versions",
        "validator_versions",
        "issue_taxonomy",
        "ml_features",
        "regression_sets",
    ] {
        ensure_dir(&root.join(dir))?;
    }
    Ok(())
}
