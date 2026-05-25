use crate::eval_set::{is_valid_ticker_symbol, load_eval_set};
use crate::lint::{expected_family_conflict, lint_status};
use crate::quality::{grade, score_report, QualityScores};
use crate::training::TrainingRunOptions;
use crate::training_case::{correction_case_path, TrainingCase};
use std::path::Path;

#[test]
fn broad_30_eval_set_loads() {
    let eval = load_eval_set(Path::new("../eval_sets/broad_30_probe.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/broad_30_probe.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/broad_30_probe.yaml")))
        .expect("broad_30 eval set should load");
    assert_eq!(eval.tickers.len(), 30);
    assert_eq!(
        eval.expected_family.get("GOOGL").unwrap(),
        "Platform Internet / Mega-cap Tech"
    );
}

#[test]
fn broad_500_eval_set_loads() {
    let eval = load_eval_set(Path::new("../eval_sets/broad_500_us_cn.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/broad_500_us_cn.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/broad_500_us_cn.yaml")))
        .expect("broad_500 eval set should load");
    assert_eq!(eval.tickers.len(), 500);
    assert!(eval.expected_family.contains_key("600519.SH"));
    assert!(eval.expected_family.contains_key("AAPL"));
}

#[test]
fn regression_hard_cases_eval_set_loads() {
    let eval = load_eval_set(Path::new("../eval_sets/regression_hard_cases.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/regression_hard_cases.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/regression_hard_cases.yaml")))
        .expect("regression hard cases should load");
    assert!(eval.tickers.contains(&"LUNR".to_string()));
    assert_eq!(
        eval.expected_family.get("LUNR").unwrap(),
        "Space / Lunar Infrastructure or Data-Limited Aerospace"
    );
    assert!(eval.tickers.contains(&"600519.SH".to_string()));
}

#[test]
fn training_case_rejects_non_ticker_strings() {
    for value in [
        "SPACE / LUNAR INFRASTRUCTURE",
        "AEROSPACE SERVICES",
        "SPECULATIVE AEROSPACE / SPACE SYSTEMS",
        "TELECOM / INFRASTRUCTURE CASH FLOW",
        "WIRELESS SERVICE REVENUE",
        "BROADBAND / NETWORK REVENUE",
        "SUBSCRIBER CHURN",
        "FINANCIALS",
        "SEMICONDUCTOR SELLER",
        "INSURANCE-LIKE SCREENING",
        "BIOTECH DRUG PIPELINE",
        "GENERIC UNKNOWN",
    ] {
        assert!(!is_valid_ticker_symbol(value), "{value} must be rejected");
    }
    for value in ["AAPL", "RKLB", "BRK-B", "600519.SH", "000001.SZ"] {
        assert!(is_valid_ticker_symbol(value), "{value} should be valid");
    }
}

#[test]
fn training_loop_outputs_required_artifact_names() {
    let source = include_str!("training.rs");
    for artifact in [
        "training_summary.md",
        "quality_matrix.csv",
        "quality_matrix.json",
        "issue_distribution.md",
        "wrong_framework_cases.md",
        "weak_money_flow_cases.md",
        "weak_chart_table_cases.md",
        "unsupported_claim_cases.md",
        "provider_failure_cases.md",
        "prompt_improvement_suggestions.md",
        "validator_improvement_suggestions.md",
        "training_cases_generated.jsonl",
        "regression_cases_generated.jsonl",
        "external_correction_cases_generated.jsonl",
        "negative_regression_cases_generated.jsonl",
        "iteration_log.md",
        "cost_report.md",
        "final_acceptance.md",
        "model_improvement_review.md",
        "dashboard.html",
    ] {
        assert!(
            source.contains(artifact),
            "training output missing {artifact}"
        );
    }
}

#[test]
fn isrg_biotech_frame_is_wrong_framework_conflict() {
    assert!(expected_family_conflict(
        "Medical Devices / Surgical Robotics",
        "Biotech / Pharma Research Frame"
    ));
    let status = research_core::types::ReportStatus {
        schema_version: "v5.0.0".into(),
        overall_status: "WARNING".into(),
        provider_payload_valid: "PASS".into(),
        company_understanding_present: "PASS".into(),
        financial_interpretation_present: "PASS".into(),
        research_blueprint_present: "PASS".into(),
        ai_self_review_present: "PASS".into(),
        money_flow_present: "PASS".into(),
        human_review_required: true,
        ai_mode: "compact".into(),
        ai_calls: 0,
        cache_hits: 0,
        provider_status: "PASS".into(),
        visual_lint_status: "PASS".into(),
        pdf_export_status: "PASS".into(),
    };
    let expected = "Medical Devices / Surgical Robotics".to_string();
    let lint = lint_status(&status, Some(&expected), "Biotech / Pharma Research Frame");
    assert!(lint
        .failed_checks
        .contains(&"wrong_framework_conflict".into()));
}

#[test]
fn local_mock_cases_not_positive() {
    let case = TrainingCase {
        ticker: "RKLB".into(),
        initial_profile: "Speculative Aerospace / Space Systems".into(),
        final_profile: "Speculative Aerospace / Space Systems".into(),
        expected_family: "Space Launch / Space Systems / Speculative Aerospace".into(),
        issue_type: "local_mock_case".into(),
        training_case_type: "local_mock_case".into(),
        ai_source: "local_mock".into(),
        wrong_output: "".into(),
        expected_output_features: vec![],
        must_contain: vec![],
        must_not_contain: vec![],
        data_refs_used: vec![],
        fixed_by: "test".into(),
        regression_status: "new".into(),
    };
    let path = correction_case_path(Path::new("training_cases"), &case).unwrap();
    assert!(path.ends_with("corrections/v5_local_mock_cases.jsonl"));
    assert!(!path.ends_with("corrections/v5_external_correction_cases.jsonl"));
}

#[test]
fn correction_cases_are_split_by_source_and_issue() {
    let source = include_str!("runner.rs");
    assert!(source.contains("correction_case_path"));
    let case_source = include_str!("training_case.rs");
    assert!(case_source.contains("v5_external_correction_cases.jsonl"));
    assert!(case_source.contains("v5_local_mock_cases.jsonl"));
    assert!(case_source.contains("v5_negative_regression_cases.jsonl"));
    assert!(!source.contains("v5_correction_cases.jsonl"));
}

#[test]
fn polluted_training_cases_are_filtered() {
    let eval = load_eval_set(Path::new("../eval_sets/regression_hard_cases.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/regression_hard_cases.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/regression_hard_cases.yaml")))
        .expect("regression hard cases should load");
    for polluted in [
        "SPACE / LUNAR INFRASTRUCTURE",
        "WIRELESS SERVICE REVENUE",
        "BROADBAND / NETWORK REVENUE",
        "SUBSCRIBER CHURN",
        "FINANCIALS",
        "BIOTECH DRUG PIPELINE",
    ] {
        assert!(!eval.tickers.contains(&polluted.to_string()));
    }
}

#[test]
fn isrg_not_biotech_pipeline() {
    let eval = load_eval_set(Path::new("../eval_sets/regression_hard_cases.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/regression_hard_cases.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/regression_hard_cases.yaml")))
        .expect("regression hard cases should load");
    assert_eq!(
        eval.expected_family.get("ISRG").unwrap(),
        "Medical Devices / Surgical Robotics"
    );
}

#[test]
fn aapl_not_platform_ads_cloud() {
    let eval = load_eval_set(Path::new("../eval_sets/broad_500_us_cn.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/broad_500_us_cn.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/broad_500_us_cn.yaml")))
        .expect("broad_500 should load");
    assert_ne!(
        eval.expected_family.get("AAPL").unwrap(),
        "Platform Internet / Digital Ads / Cloud"
    );
}

#[test]
fn lly_not_early_biotech_cash_runway() {
    let eval = load_eval_set(Path::new("../eval_sets/regression_hard_cases.yaml"))
        .or_else(|_| load_eval_set(Path::new("../../eval_sets/regression_hard_cases.yaml")))
        .or_else(|_| load_eval_set(Path::new("../../../eval_sets/regression_hard_cases.yaml")))
        .expect("regression hard cases should load");
    assert_eq!(
        eval.expected_family.get("LLY").unwrap(),
        "Large Pharma / Drug Portfolio / Regulatory and Patent Risk"
    );
}

#[test]
fn training_cli_options_support_staged_filters() {
    let options = TrainingRunOptions {
        eval_set: "eval_sets/regression_hard_cases.yaml".into(),
        stage: "regression".into(),
        workers: 2,
        ai_mode: "compact".into(),
        require_external_ai: true,
        no_ai_cache: true,
        budget_calls: 100,
        max_iterations: 5,
        quality_threshold: 75,
        run_id: "test".into(),
        limit: Some(50),
        offset: 0,
        resume: true,
        only_failed: true,
        only_weak: true,
        only_wrong_framework: true,
        only_provider_failed: true,
        only_low_quality: true,
        force: false,
        pack: false,
    };
    assert_eq!(options.stage, "regression");
    assert!(options.require_external_ai);
    assert!(options.only_wrong_framework);
}

#[test]
fn issue_taxonomy_file_lists_blockers_and_fix_targets() {
    let taxonomy = include_str!("../../../../training/issue_taxonomy/issue_types.yaml");
    assert!(taxonomy.contains("wrong_profile"));
    assert!(taxonomy.contains("hallucinated_revenue_engine"));
    assert!(taxonomy.contains("unsupported_numeric_claim"));
    assert!(taxonomy.contains("fix_target: validator"));
    assert!(taxonomy.contains("training_use: negative"));
}

#[test]
fn prompt_version_files_exist() {
    let base = Path::new("../crates/research-ai/prompts")
        .canonicalize()
        .or_else(|_| Path::new("../../crates/research-ai/prompts").canonicalize())
        .or_else(|_| Path::new("../../../research-rs/crates/research-ai/prompts").canonicalize())
        .expect("prompt directory should exist");
    for name in [
        "company_understanding_v1.md",
        "financial_interpretation_v1.md",
        "research_blueprint_v1.md",
        "self_review_v1.md",
        "content_quality_judge_v1.md",
        "chart_explanation_v1.md",
        "table_explanation_v1.md",
    ] {
        let raw = std::fs::read_to_string(base.join(name)).expect("prompt file should be readable");
        assert!(raw.contains("prompt_version:"));
        assert!(raw.contains("forbidden_behavior:"));
    }
}

#[test]
fn content_quality_score_has_all_dimensions() {
    let scores = QualityScores {
        company_understanding_score: 15,
        business_model_score: 10,
        financial_interpretation_score: 15,
        money_flow_score: 15,
        blueprint_fit_score: 10,
        valuation_fit_score: 8,
        risk_score: 7,
        data_gap_score: 7,
        chart_table_score: 5,
        language_score: 5,
        unsupported_claims_score: 3,
    };
    let total = scores.company_understanding_score
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
    assert_eq!(total, 100);
}

#[test]
fn low_quality_report_fails() {
    assert_eq!(grade(92, &["unsupported_numeric_claim".into()]), "FAIL");
    assert_eq!(grade(55, &[]), "FAIL");
}

#[test]
fn content_quality_detects_missing_report() {
    let review = score_report(
        Path::new("/tmp/definitely_missing_v5_report"),
        "MISSING",
        "Unknown",
    );
    assert!(review
        .hard_failures
        .contains(&"generated_report_missing".to_string()));
    assert_eq!(review.grade, "FAIL");
}

#[test]
fn batch_summary_counts_external_vs_local_ai() {
    let source = include_str!("runner.rs");
    assert!(source.contains("External AI calls"));
    assert!(source.contains("New external AI calls"));
    assert!(source.contains("Local fallback reports"));
    assert!(source.contains("Cache-hit AI reports"));
    assert!(source.contains("Reports with no AI"));
}

#[test]
fn codex_self_review_records_ai_provenance() {
    let review = include_str!("../../../../reports/release_checks/v5_0/codex_self_review.md");
    assert!(review.contains("AI Provenance Review"));
    assert!(review.contains("External OpenAI API used"));
    assert!(review.contains("Local fallback used"));
    assert!(review.contains("New external API calls"));
}

#[test]
fn training_cases_record_ai_source_to_prevent_local_mock_positive_cases() {
    let source = include_str!("runner.rs");
    assert!(source.contains("ai_source"));
    assert!(source.contains("local_mock_case"));
    assert!(!source.contains("positive_case"));
}

#[test]
fn quality_judge_provenance_marks_local_judge() {
    let source = include_str!("quality.rs");
    assert!(source.contains("quality_judge_provenance.json"));
    assert!(source.contains("Quality score is from local fallback judge"));
    assert!(source.contains("local-deterministic-quality-judge"));
}
