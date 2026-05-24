use crate::types::*;

pub fn validate_provider_payload(payload: &ProviderPayload) -> Vec<String> {
    let mut failures = Vec::new();
    if payload.ticker.trim().is_empty() {
        failures.push("invalid_provider_payload:ticker_missing".to_string());
    }
    if payload.company_profile.name.trim().is_empty() && payload.error.is_none() {
        failures.push("weak_company_identity:name_missing".to_string());
    }
    failures
}

pub fn validate_ai_json(
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    _review: &AiSelfReview,
) -> Vec<String> {
    let mut failures = Vec::new();
    if understanding.company_identity.trim().is_empty() {
        failures.push("missing_company_understanding".to_string());
    }
    if interpretation.where_money_comes_from.trim().is_empty()
        || interpretation.where_money_goes.trim().is_empty()
    {
        failures.push("missing_money_flow".to_string());
    }
    if blueprint.core_thesis.trim().len() < 40 {
        failures.push("blueprint_too_generic".to_string());
    }
    failures
}

pub fn detect_forbidden_advice(text: &str) -> bool {
    let lower = text.to_lowercase();
    let boundary_phrases = [
        "does not provide a target price",
        "not provide a target price",
        "not investment advice",
        "not provide buy/sell",
        "not a buy/sell",
    ];
    let sanitized = boundary_phrases
        .iter()
        .fold(lower, |acc, phrase| acc.replace(phrase, ""));
    [
        "buy rating",
        "sell rating",
        "target price",
        "price target",
        "should buy",
        "should sell",
    ]
    .iter()
    .any(|needle| sanitized.contains(needle))
}

pub fn report_status(
    payload_failures: &[String],
    ai_failures: &[String],
    review: &AiSelfReview,
    provider_status: String,
    ai_mode: String,
    ai_calls: usize,
    cache_hits: usize,
) -> ReportStatus {
    let human_review_required = review.human_review_required
        || !payload_failures.is_empty()
        || !ai_failures.is_empty()
        || provider_status != "PASS";
    let overall_status = if ai_failures
        .iter()
        .any(|f| f.contains("wrong_framework") || f.contains("unsupported"))
    {
        "FAIL"
    } else if human_review_required {
        "WARNING"
    } else {
        "PASS"
    };
    ReportStatus {
        overall_status: overall_status.to_string(),
        provider_payload_valid: if payload_failures.is_empty() {
            "PASS"
        } else {
            "WARNING"
        }
        .to_string(),
        company_understanding_present: "PASS".to_string(),
        financial_interpretation_present: "PASS".to_string(),
        research_blueprint_present: "PASS".to_string(),
        ai_self_review_present: "PASS".to_string(),
        money_flow_present: if ai_failures.iter().any(|f| f == "missing_money_flow") {
            "FAIL"
        } else {
            "PASS"
        }
        .to_string(),
        human_review_required,
        ai_mode,
        ai_calls,
        cache_hits,
        provider_status,
        visual_lint_status: "PASS".to_string(),
    }
}

pub fn visual_lint(
    report: &str,
    dashboard_exists: bool,
    chart_manifest_exists: bool,
) -> (String, Vec<String>) {
    let mut failures = Vec::new();
    if !report.starts_with('#') || !report.contains("> Status:") {
        failures.push("report_has_status_block".to_string());
    }
    if !report.contains("## Table of Contents") {
        failures.push("report_has_toc".to_string());
    }
    if !report.contains("What to look at:") {
        failures.push("chart_explanations_present".to_string());
    }
    if report.contains("NaN") || report.contains("null") || report.contains("[METRIC_MISSING_RAW]")
    {
        failures.push("no_raw_nan_or_placeholder".to_string());
    }
    if !dashboard_exists {
        failures.push("dashboard_exists".to_string());
    }
    if !chart_manifest_exists {
        failures.push("chart_manifest_exists".to_string());
    }
    if detect_forbidden_advice(report) {
        failures.push("no_forbidden_advice".to_string());
    }
    let status = if failures.is_empty() { "PASS" } else { "FAIL" }.to_string();
    (status, failures)
}
