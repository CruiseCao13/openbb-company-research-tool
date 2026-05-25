use anyhow::{bail, Result};
use research_core::io::write_if_changed;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::eval_set::is_valid_ticker_symbol;

#[derive(Debug, Clone, Serialize)]
pub struct TrainingCase {
    pub ticker: String,
    pub initial_profile: String,
    pub final_profile: String,
    pub expected_family: String,
    pub issue_type: String,
    pub training_case_type: String,
    pub ai_source: String,
    pub wrong_output: String,
    pub expected_output_features: Vec<String>,
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
    pub data_refs_used: Vec<String>,
    pub fixed_by: String,
    pub regression_status: String,
}

pub fn expected_features_for_ticker(ticker: &str, fallback: Vec<String>) -> Vec<String> {
    let features = match ticker {
        "AAPL" => &[
            "hardware product revenue",
            "services revenue",
            "gross margin mix",
            "free cash flow",
            "buybacks only if supported by data",
        ][..],
        "ISRG" => &[
            "installed surgical robotics base",
            "procedure volume",
            "instruments and accessories revenue",
            "system placements",
            "hospital capital spending sensitivity",
        ],
        "LLY" => &[
            "drug portfolio revenue",
            "pipeline and indication expansion",
            "patent and regulatory risk",
            "manufacturing capacity",
            "R&D and commercialization spend",
        ],
        "RKLB" => &[
            "launch services and space systems revenue split",
            "spacecraft components and mission services evidence",
            "launch cadence and mission execution",
            "cash burn and financing runway",
            "customer or contract concentration",
        ],
        "LUNR" => &[
            "NASA or government-linked contract evidence",
            "mission milestone execution",
            "project margin and cost overrun risk",
            "cash burn and financing runway",
            "customer or contract concentration",
        ],
        "JPM" => &[
            "ROE",
            "ROA",
            "NIM",
            "credit loss",
            "deposit cost",
            "capital ratio",
        ],
        "CAT" => &[
            "machinery end-market cycle",
            "dealer inventory",
            "equipment demand",
            "operating margin",
            "working capital",
        ],
        "ZIM" => &[
            "freight rate / yield",
            "fleet utilization",
            "fuel cost",
            "orderbook",
            "leverage",
        ],
        "600519.SH" => &[
            "A-share",
            "RMB",
            "baijiu or premium liquor demand",
            "operating cash flow",
            "inventory and receivables",
        ],
        "000001.SZ" => &[
            "A-share bank",
            "RMB",
            "ROE",
            "NIM",
            "credit loss",
            "capital ratio",
        ],
        _ => return fallback,
    };
    features.iter().map(|item| (*item).to_string()).collect()
}

pub fn write_training_case(path: &Path, case: &TrainingCase) -> Result<()> {
    if !is_valid_ticker_symbol(&case.ticker) {
        bail!("invalid training-case ticker: {}", case.ticker);
    }
    let raw = serde_json::to_string(case)?;
    write_if_changed(path, &(raw + "\n"))?;
    Ok(())
}

pub fn correction_case_path(root: &Path, case: &TrainingCase) -> Result<PathBuf> {
    if !is_valid_ticker_symbol(&case.ticker) {
        bail!("invalid training-case ticker: {}", case.ticker);
    }

    let file =
        if case.training_case_type == "local_mock_case" || case.ai_source != "external_openai" {
            "v5_local_mock_cases.jsonl"
        } else if matches!(
            case.issue_type.as_str(),
            "wrong_framework_conflict"
                | "wrong_framework"
                | "wrong_profile"
                | "hallucinated_revenue_engine"
                | "unsupported_claim"
                | "unsupported_numeric_claim"
                | "report_status_false_pass"
                | "self_review_failed"
                | "self_review_failed_to_catch"
        ) {
            "v5_negative_regression_cases.jsonl"
        } else {
            "v5_external_correction_cases.jsonl"
        };

    Ok(root.join("corrections").join(file))
}
