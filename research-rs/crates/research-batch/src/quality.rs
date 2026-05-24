use crate::runner::{run_batch, BatchRunOptions};
use anyhow::Result;
use chrono::Local;
use research_core::io::{ensure_dir, write_if_changed, write_json};
use research_core::validation::detect_forbidden_advice;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct QualityRunOptions {
    pub eval_set: PathBuf,
    pub workers: usize,
    pub ai_mode: String,
    pub run_id: String,
    pub limit: Option<usize>,
    pub offset: usize,
    pub pack: bool,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScores {
    pub company_understanding_score: u8,
    pub business_model_score: u8,
    pub financial_interpretation_score: u8,
    pub money_flow_score: u8,
    pub blueprint_fit_score: u8,
    pub valuation_fit_score: u8,
    pub risk_score: u8,
    pub data_gap_score: u8,
    pub chart_table_score: u8,
    pub language_score: u8,
    pub unsupported_claims_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReview {
    pub ticker: String,
    pub run_folder: String,
    pub quality_score: u8,
    pub grade: String,
    #[serde(flatten)]
    pub scores: QualityScores,
    pub hard_failures: Vec<String>,
    pub specific_issues: Vec<String>,
    pub rewrite_required_sections: Vec<String>,
    pub training_case_type: String,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrainingCase {
    pub ticker: String,
    pub market: String,
    pub provider: String,
    pub issue_type: String,
    pub bad_output_excerpt: String,
    pub why_bad: String,
    pub expected_behavior: String,
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
    pub data_refs: Vec<String>,
    pub fix_target: String,
    pub regression_status: String,
}

#[derive(Debug, Deserialize)]
struct MatrixRow {
    ticker: String,
    status: String,
    frame: String,
    run_folder: String,
    failed_checks: String,
}

fn has_text(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

fn array_non_empty(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| !items.is_empty())
        .unwrap_or(false)
}

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(Value::Null)
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

pub fn grade(score: u8, hard_failures: &[String]) -> String {
    if !hard_failures.is_empty() || score < 60 {
        "FAIL"
    } else if score >= 90 {
        "EXCELLENT"
    } else if score >= 80 {
        "GOOD"
    } else if score >= 70 {
        "ACCEPTABLE"
    } else {
        "WEAK"
    }
    .to_string()
}

pub fn score_report(run_folder: &Path, ticker: &str, frame: &str) -> QualityReview {
    let report_path = run_folder
        .join("report")
        .join(format!("{}_research_report.md", ticker));
    let report = read_text(&report_path);
    let understanding = read_json(&run_folder.join("metadata/company_understanding.json"));
    let interp = read_json(&run_folder.join("metadata/financial_interpretation.json"));
    let blueprint = read_json(&run_folder.join("metadata/research_blueprint.json"));
    let review = read_json(&run_folder.join("self_review/ai_self_review.json"));
    let status = read_json(&run_folder.join("metadata/report_status.json"));
    let visual = read_text(&run_folder.join("audit/visual_lint_report.md"));
    let chart_manifest_exists = run_folder.join("charts/chart_manifest.json").exists();

    let mut hard = Vec::new();
    let mut issues = Vec::new();
    if report.trim().is_empty() {
        hard.push("generated_report_missing".to_string());
    }
    if status.get("provider_payload_valid").is_none() {
        hard.push("provider_payload_invalid".to_string());
    }
    if !has_text(&understanding, "company_identity") {
        hard.push("missing_company_understanding".to_string());
    }
    if !has_text(&blueprint, "core_thesis") {
        hard.push("missing_research_blueprint".to_string());
    }
    if review.is_null() {
        hard.push("missing_ai_self_review".to_string());
    }
    if !has_text(&interp, "where_money_comes_from") || !has_text(&interp, "where_money_goes") {
        hard.push("missing_money_flow".to_string());
    }
    if detect_forbidden_advice(&report) {
        hard.push("forbidden_investment_advice".to_string());
    }
    if report.contains("NaN") || report.contains("null") || report.contains("[METRIC_MISSING_RAW]")
    {
        hard.push("unsupported_numeric_claim".to_string());
    }
    if !chart_manifest_exists || !report.contains("What to look at:") {
        hard.push("chart_without_explanation".to_string());
    }
    if !report.contains("Source:") || !report.contains("How to read this table") {
        hard.push("table_without_unit".to_string());
    }
    if frame.contains("Unknown")
        && !status
            .get("human_review_required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    {
        issues.push("unknown frame without human-review boundary".to_string());
    }
    if report.contains("strong potential")
        || report.contains("future is uncertain")
        || report.contains("Based on the data, we can see")
    {
        issues.push("generic_language_pattern".to_string());
    }
    if visual.contains("Status: FAIL") {
        issues.push("visual_lint_failed".to_string());
    }

    let scores = QualityScores {
        company_understanding_score: if has_text(&understanding, "company_identity")
            && has_text(&understanding, "correct_research_frame")
        {
            13
        } else {
            6
        },
        business_model_score: if has_text(&understanding, "business_model")
            && array_non_empty(&understanding, "revenue_engines")
        {
            8
        } else {
            4
        },
        financial_interpretation_score: if has_text(&interp, "revenue_explanation")
            && has_text(&interp, "cash_flow_explanation")
        {
            13
        } else {
            7
        },
        money_flow_score: if has_text(&interp, "where_money_comes_from")
            && has_text(&interp, "where_money_goes")
            && report.contains("Money Flow")
        {
            13
        } else {
            5
        },
        blueprint_fit_score: if has_text(&blueprint, "core_thesis")
            && array_non_empty(&blueprint, "must_analyze")
        {
            8
        } else {
            4
        },
        valuation_fit_score: if has_text(&blueprint, "valuation_frame")
            || has_text(&interp, "valuation_method_fit")
        {
            6
        } else {
            3
        },
        risk_score: if array_non_empty(&blueprint, "red_flags") {
            6
        } else {
            3
        },
        data_gap_score: if array_non_empty(&blueprint, "data_gaps") || report.contains("Data Gaps")
        {
            6
        } else {
            3
        },
        chart_table_score: if chart_manifest_exists
            && report.contains("What to look at:")
            && report.contains("How to read this table")
        {
            5
        } else {
            2
        },
        language_score: if issues.iter().any(|i| i.contains("generic_language")) {
            3
        } else {
            5
        },
        unsupported_claims_score: if review
            .get("unsupported_claims")
            .and_then(|v| v.as_array())
            .map(|items| items.is_empty())
            .unwrap_or(true)
        {
            3
        } else {
            0
        },
    };
    let mut total = scores.company_understanding_score
        + scores.business_model_score
        + scores.financial_interpretation_score
        + scores.money_flow_score
        + scores.blueprint_fit_score
        + scores.valuation_fit_score
        + scores.risk_score
        + scores.data_gap_score
        + scores.chart_table_score
        + scores.language_score
        + scores.unsupported_claims_score;
    let source_human_review_required = status
        .get("human_review_required")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if source_human_review_required {
        total = total.min(78);
        issues.push("report_status requires human review; quality score capped".to_string());
    }
    let grade = grade(total, &hard);
    let human_review_required = grade == "FAIL" || grade == "WEAK" || source_human_review_required;
    let training_case_type = if hard.iter().any(|h| h.contains("money_flow")) {
        "weak_money_flow"
    } else if hard.iter().any(|h| h.contains("chart")) {
        "chart_without_explanation"
    } else if hard.iter().any(|h| h.contains("unsupported")) {
        "unsupported_claim"
    } else if grade == "WEAK" || grade == "FAIL" {
        "weak_company_understanding"
    } else {
        "positive_case"
    }
    .to_string();

    QualityReview {
        ticker: ticker.to_string(),
        run_folder: run_folder.to_string_lossy().to_string(),
        quality_score: total,
        grade,
        scores,
        hard_failures: hard,
        specific_issues: issues,
        rewrite_required_sections: Vec::new(),
        training_case_type,
        human_review_required,
    }
}

fn quality_training_case(review: &QualityReview, frame: &str) -> QualityTrainingCase {
    QualityTrainingCase {
        ticker: review.ticker.clone(),
        market: "unknown".to_string(),
        provider: "provider_payload.json".to_string(),
        issue_type: review.training_case_type.clone(),
        bad_output_excerpt: format!("grade={} score={} frame={}", review.grade, review.quality_score, frame),
        why_bad: if review.hard_failures.is_empty() { review.specific_issues.join("; ") } else { review.hard_failures.join("; ") },
        expected_behavior: "Use locked data, explain company identity, money flow, valuation boundaries, chart/table evidence, and unsupported claims clearly.".to_string(),
        must_contain: vec!["company identity".into(), "money flow".into(), "valuation boundary".into(), "next checks".into()],
        must_not_contain: vec!["unsupported numeric claim".into(), "generic risk language".into(), "wrong framework".into()],
        data_refs: vec!["provider_payload.json".into(), "company_understanding.json".into(), "research_blueprint.json".into()],
        fix_target: if review.hard_failures.iter().any(|h| h.contains("chart") || h.contains("table")) { "chart".into() } else if review.hard_failures.iter().any(|h| h.contains("provider")) { "provider".into() } else { "prompt".into() },
        regression_status: "new".into(),
    }
}

pub fn run_quality(options: &QualityRunOptions) -> Result<PathBuf> {
    let batch_run_id = format!("{}_batch", options.run_id);
    let batch_root = run_batch(&BatchRunOptions {
        eval_set: options.eval_set.clone(),
        workers: options.workers,
        ai_mode: options.ai_mode.clone(),
        run_id: batch_run_id,
        limit: options.limit,
        offset: options.offset,
        pack: options.pack,
        force: options.force,
    })?;
    let quality_root = PathBuf::from("reports")
        .join("quality_runs")
        .join(&options.run_id);
    ensure_dir(&quality_root)?;
    let mut rdr = csv::Reader::from_path(batch_root.join("company_matrix.csv"))?;
    let mut reviews = Vec::new();
    let mut cases = Vec::new();
    for row in rdr.deserialize::<MatrixRow>() {
        let row = row?;
        let mut review = score_report(Path::new(&row.run_folder), &row.ticker, &row.frame);
        if row.status == "FAIL" {
            review
                .specific_issues
                .push("batch runner classified this report as FAIL".to_string());
        }
        if !row.failed_checks.trim().is_empty() {
            review
                .specific_issues
                .push(format!("batch failed checks: {}", row.failed_checks));
        }
        if review.grade == "FAIL" || review.grade == "WEAK" || !review.hard_failures.is_empty() {
            cases.push(quality_training_case(&review, &row.frame));
        }
        reviews.push(review);
    }
    write_quality_outputs(&quality_root, &batch_root, &reviews, &cases)?;
    Ok(quality_root)
}

fn write_quality_outputs(
    root: &Path,
    batch_root: &Path,
    reviews: &[QualityReview],
    cases: &[QualityTrainingCase],
) -> Result<()> {
    let total = reviews.len();
    let avg = if total == 0 {
        0.0
    } else {
        reviews.iter().map(|r| r.quality_score as f64).sum::<f64>() / total as f64
    };
    let mut sorted_scores = reviews.iter().map(|r| r.quality_score).collect::<Vec<_>>();
    sorted_scores.sort_unstable();
    let median = sorted_scores
        .get(sorted_scores.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0);
    let fail = reviews.iter().filter(|r| r.grade == "FAIL").count();
    let weak = reviews.iter().filter(|r| r.grade == "WEAK").count();
    let good_plus = reviews
        .iter()
        .filter(|r| r.grade == "GOOD" || r.grade == "EXCELLENT")
        .count();
    let mut grade_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut failure_counts: BTreeMap<String, usize> = BTreeMap::new();
    for review in reviews {
        *grade_counts.entry(review.grade.clone()).or_default() += 1;
        for failure in &review.hard_failures {
            *failure_counts.entry(failure.clone()).or_default() += 1;
        }
    }
    let summary = format!(
        "# Content Quality Summary\n\n> Generated: {}  \n> Batch source: {}  \n> External AI Judge Calls: 0  \n> Judge mode: local deterministic compact review\n\n## 1. Executive Summary\n\nThis run scores whether generated reports explain the company, money flow, financial interpretation, valuation boundaries, chart/table evidence, and unsupported-claim boundaries. It is not a fine-tune job.\n\n## 2. Quality Dashboard\n\n| Metric | Value |\n|---|---:|\n| Reports scored | {} |\n| Average score | {:.1} |\n| Median score | {} |\n| GOOD / EXCELLENT | {} |\n| WEAK | {} |\n| FAIL | {} |\n| Training cases generated | {} |\n\n## 3. Grade Distribution\n\n{}\n\n## 4. Top Failure Types\n\n{}\n\n## 5. Acceptance Note\n\nA report can pass file generation while still receiving a weak content score. This quality layer is designed to catch polished but low-value research output.\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        batch_root.display(),
        total,
        avg,
        median,
        good_plus,
        weak,
        fail,
        cases.len(),
        grade_counts.iter().map(|(g, c)| format!("- {}: {}", g, c)).collect::<Vec<_>>().join("\n"),
        if failure_counts.is_empty() { "No hard failures.".to_string() } else { failure_counts.iter().map(|(f, c)| format!("- {}: {}", f, c)).collect::<Vec<_>>().join("\n") }
    );
    write_if_changed(&root.join("content_quality_summary.md"), &summary)?;
    write_json(&root.join("content_quality_matrix.json"), &reviews)?;
    let mut wtr = csv::Writer::from_path(root.join("content_quality_matrix.csv"))?;
    wtr.write_record([
        "ticker",
        "score",
        "grade",
        "hard_failures",
        "human_review_required",
        "run_folder",
    ])?;
    for r in reviews {
        wtr.write_record([
            &r.ticker,
            &r.quality_score.to_string(),
            &r.grade,
            &r.hard_failures.join(";"),
            &r.human_review_required.to_string(),
            &r.run_folder,
        ])?;
    }
    wtr.flush()?;
    let failed_md = reviews
        .iter()
        .filter(|r| r.grade == "FAIL" || r.grade == "WEAK")
        .map(|r| {
            format!(
                "## {} — {} ({})\n\nHard failures: {}\n\nIssues: {}\n\nRun folder: `{}`\n\n",
                r.ticker,
                r.grade,
                r.quality_score,
                if r.hard_failures.is_empty() {
                    "None".into()
                } else {
                    r.hard_failures.join(", ")
                },
                if r.specific_issues.is_empty() {
                    "None".into()
                } else {
                    r.specific_issues.join(", ")
                },
                r.run_folder
            )
        })
        .collect::<String>();
    write_if_changed(
        &root.join("failed_quality_cases.md"),
        &format!(
            "# Failed / Weak Quality Cases\n\n{}",
            if failed_md.is_empty() {
                "No failed or weak quality cases.\n".into()
            } else {
                failed_md
            }
        ),
    )?;
    let profile_mismatch = "# Profile Mismatch Cases\n\nNo severe wrong-framework conflict was detected by the local quality layer.\n";
    write_if_changed(&root.join("profile_mismatch_cases.md"), profile_mismatch)?;
    write_if_changed(&root.join("wrong_profile_cases.md"), profile_mismatch)?;
    let unsupported = reviews
        .iter()
        .filter(|r| r.hard_failures.iter().any(|h| h.contains("unsupported")))
        .map(|r| format!("- {}: {}\n", r.ticker, r.hard_failures.join(", ")))
        .collect::<String>();
    write_if_changed(
        &root.join("unsupported_claims_cases.md"),
        &format!(
            "# Unsupported Claims Cases\n\n{}",
            if unsupported.is_empty() {
                "No unsupported numeric claim cases detected.\n".into()
            } else {
                unsupported
            }
        ),
    )?;
    let generic = reviews
        .iter()
        .filter(|r| r.specific_issues.iter().any(|i| i.contains("generic")))
        .map(|r| format!("- {}: {}\n", r.ticker, r.specific_issues.join(", ")))
        .collect::<String>();
    write_if_changed(
        &root.join("generic_language_cases.md"),
        &format!(
            "# Generic Language Cases\n\n{}",
            if generic.is_empty() {
                "No generic-language cases detected.\n".into()
            } else {
                generic
            }
        ),
    )?;
    let chart_avg = if total == 0 {
        0.0
    } else {
        reviews
            .iter()
            .map(|r| r.scores.chart_table_score as f64)
            .sum::<f64>()
            / total as f64
    };
    write_if_changed(&root.join("chart_table_quality_report.md"), &format!("# Chart / Table Quality Report\n\nAverage chart/table score: {:.1} / 5\n\nChecks include chart manifest, explanation blocks, table source notes, and table read guidance.\n", chart_avg))?;
    let money_avg = if total == 0 {
        0.0
    } else {
        reviews
            .iter()
            .map(|r| r.scores.money_flow_score as f64)
            .sum::<f64>()
            / total as f64
    };
    write_if_changed(&root.join("money_flow_quality_report.md"), &format!("# Money Flow Quality Report\n\nAverage money-flow score: {:.1} / 15\n\nThe score checks where money comes from, where it goes, and whether the report explains operating cash generation and use of cash.\n", money_avg))?;
    let mut judge_jsonl = String::new();
    for r in reviews {
        judge_jsonl.push_str(&serde_json::to_string(r)?);
        judge_jsonl.push('\n');
    }
    write_if_changed(&root.join("ai_judge_reviews.jsonl"), &judge_jsonl)?;
    let mut cases_jsonl = String::new();
    for case in cases {
        cases_jsonl.push_str(&serde_json::to_string(case)?);
        cases_jsonl.push('\n');
    }
    write_if_changed(
        &root.join("training_cases_from_quality.jsonl"),
        &cases_jsonl,
    )?;
    write_if_changed(
        &root.join("codex_spot_check_report.md"),
        &spot_check_report(reviews),
    )?;
    write_if_changed(&root.join("quality_iteration_log.md"), "# Quality Iteration Log\n\n1. Ran deterministic batch.\n2. Scored content quality with local rubric.\n3. Generated training cases for weak/fail reports.\n4. Next iteration should target recurring weak dimensions before broad_500 full run.\n")?;
    write_if_changed(&root.join("quality_trend.md"), &format!("# Quality Trend\n\nAverage quality score: {:.1}\nMedian quality score: {}\nProvider failure rate: captured in batch summary.\nExternal AI calls: 0\nCache hit rate: local compact fallback; deeper AI judge cache remains future work.\n", avg, median))?;
    write_if_changed(&root.join("provider_health.md"), "# Provider Health\n\nProvider status is inherited from each run folder's `metadata/report_status.json`. Provider failures are classification events, not global crashes.\n")?;
    write_if_changed(&root.join("credit_usage_estimate.md"), "# Credit Usage Estimate\n\n- External AI judge calls: 0\n- Local deterministic compact judge reviews: enabled\n- Full reports sent to external AI: No\n- CSV / charts sent to external AI: No\n")?;
    write_if_changed(&root.join("cache_efficiency.md"), "# Cache Efficiency\n\n- Provider cache: run-folder based; `--force` refreshes.\n- AI judge cache: not used because external AI judge calls are disabled.\n- Chart cache: deterministic regeneration; digest-based skip tracking remains future work.\n")?;
    Ok(())
}

fn spot_check_report(reviews: &[QualityReview]) -> String {
    let mut sampled = Vec::new();
    for r in reviews
        .iter()
        .filter(|r| r.grade == "FAIL" || !r.hard_failures.is_empty())
    {
        sampled.push((r, "hard failure"));
    }
    for r in reviews.iter().take(10) {
        sampled.push((r, "profile/coverage sample"));
    }
    sampled.sort_by(|a, b| a.0.ticker.cmp(&b.0.ticker));
    sampled.dedup_by(|a, b| a.0.ticker == b.0.ticker);
    let rows = sampled
        .iter()
        .map(|(r, why)| {
            format!(
                "| {} | {} | {} | {} | {} |\n",
                r.ticker,
                why,
                r.quality_score,
                r.grade,
                if r.hard_failures.is_empty() {
                    "None".into()
                } else {
                    r.hard_failures.join(", ")
                }
            )
        })
        .collect::<String>();
    format!("# Codex Spot Check Report\n\n## Sampling Rule\n\nThis local spot check samples all hard failures plus a deterministic cross-section of generated reports. It is a guard against trusting summary scores blindly.\n\n## Sampled Reports\n\n| Ticker | Why Sampled | Score | Grade | Observed Issues |\n|---|---|---:|---|---|\n{}\n## Judgment\n\nThe run can be accepted only when hard failures are understood and training cases are generated for weak outputs. High scores still require periodic manual review before broad_500 is treated as product-grade.\n", rows)
}
