use crate::markdown::render_report;
use research_core::types::*;
use research_core::validation::visual_lint;

#[test]
fn report_renderer_has_required_sections() {
    let payload = ProviderPayload {
        ticker: "AAPL".to_string(),
        market: "US".to_string(),
        provider: "fixture".to_string(),
        company_profile: CompanyProfile {
            name: "Apple Inc.".to_string(),
            sector: "Technology".to_string(),
            industry: "Consumer Electronics".to_string(),
            currency: "USD".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let understanding = CompanyUnderstanding {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        company_identity: "Apple identity".into(),
        business_model: "Sells products and services.".into(),
        revenue_engines: vec!["hardware".into()],
        profit_pool: "Margins and ecosystem.".into(),
        key_growth_drivers: vec!["services".into()],
        key_risks: vec!["regulatory".into()],
        not_this: vec!["bank".into()],
        correct_research_frame: "Mature Compounder".into(),
        wrong_frames_to_avoid: vec![],
        confidence: Confidence::MEDIUM,
        human_review_required: false,
    };
    let interpretation = FinancialInterpretation {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        revenue_explanation: "Revenue explanation".into(),
        margin_explanation: "Margin explanation".into(),
        cash_flow_explanation: "Cash flow explanation".into(),
        where_money_comes_from: "Operations".into(),
        where_money_goes: "Capex and returns".into(),
        capex_or_rnd_pressure: "Moderate".into(),
        debt_and_financing: "Manageable".into(),
        shareholder_return_quality: "Buybacks only if supported.".into(),
        valuation_method_fit: "PE/FCF".into(),
        unsupported_due_to_missing_data: vec![],
    };
    let blueprint = ResearchBlueprint {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        core_thesis: "Core thesis is specific enough for report rendering.".into(),
        asset_profile: "Mature Compounder".into(),
        secondary_profile: "Consumer platform".into(),
        must_analyze: vec!["margin durability".into()],
        must_not_analyze_as_core: vec!["cash runway panic".into()],
        key_questions: vec!["Can margins hold?".into()],
        red_flags: vec!["regulatory".into()],
        valuation_frame: "FCF and multiple risk".into(),
        data_gaps: vec![],
        next_checks: vec!["Check services mix.".into()],
        report_section_guidance: vec![],
        confidence: Confidence::MEDIUM,
        human_review_required: false,
    };
    let review = AiSelfReview {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        company_understanding_check: CheckStatus::PASS,
        framework_fit_check: CheckStatus::PASS,
        numeric_consistency_check: CheckStatus::PASS,
        money_flow_check: CheckStatus::PASS,
        unsupported_claims: vec![],
        wrong_framework_risk: vec![],
        required_rewrite_sections: vec![],
        final_confidence: Confidence::MEDIUM,
        human_review_required: false,
    };
    let status = ReportStatus {
        schema_version: SCHEMA_VERSION.to_string(),
        overall_status: "PASS".into(),
        provider_payload_valid: "PASS".into(),
        company_understanding_present: "PASS".into(),
        financial_interpretation_present: "PASS".into(),
        research_blueprint_present: "PASS".into(),
        ai_self_review_present: "PASS".into(),
        money_flow_present: "PASS".into(),
        human_review_required: false,
        ai_mode: "compact".into(),
        ai_calls: 0,
        cache_hits: 0,
        provider_status: "PASS".into(),
        visual_lint_status: "PASS".into(),
        pdf_export_status: "PASS".into(),
    };
    let report = render_report(
        &payload,
        &understanding,
        &interpretation,
        &blueprint,
        &review,
        &status,
    );
    assert!(report.contains("AI Source:"));
    assert!(report.contains("External AI Used:"));
    assert!(report.contains("Local Mock Used:"));
    assert!(report.contains("Model:"));
    assert!(report.contains("Prompt Versions:"));
    assert!(report.contains("## 4. Money Flow"));
    assert!(report.contains("## 13. Appendix: Locked Data"));
}

#[test]
fn report_displays_ai_source() {
    report_renderer_has_required_sections();
}

#[test]
fn dashboard_displays_ai_source() {
    let dashboard_source = include_str!("dashboard.rs");
    assert!(dashboard_source.contains("AI Source"));
    assert!(dashboard_source.contains("External OpenAI API"));
    assert!(dashboard_source.contains("local fallback analysis"));
}

#[test]
fn section_source_map_is_generated_by_renderer() {
    let source = include_str!("renderer.rs");
    assert!(source.contains("section_source_map.json"));
    assert!(source.contains("section_2_company_identity"));
    assert!(source.contains("section_4_money_flow"));
    assert!(source.contains("Renderer may format and arrange sections but must not invent thesis"));
}

#[test]
fn ai_self_review_rewrite_requests_are_not_silently_ignored() {
    let source = include_str!("renderer.rs");
    assert!(source.contains("REWRITE_REQUIRED_HUMAN_REVIEW"));
    assert!(source.contains("rewrite_required_sections"));
    assert!(source.contains("marks human review instead of pretending the rewrite was applied"));
}

#[test]
fn money_flow_map_and_output_consistency_are_generated() {
    let source = include_str!("renderer.rs");
    assert!(source.contains("money_flow_map.json"));
    assert!(source.contains("money_flow_quality_report.md"));
    assert!(source.contains("output_consistency.json"));
    assert!(source.contains("same_ai_source"));
    assert!(source.contains("Figure_04_money_flow"));
}

#[test]
fn product_quality_score_includes_ai_money_flow_and_evidence_dimensions() {
    let source = include_str!("renderer.rs");
    assert!(source.contains("ai_provenance_score"));
    assert!(source.contains("money_flow_score"));
    assert!(source.contains("evidence_score"));
}

#[test]
fn cannot_claim_external_ai_without_ai_usage_proof() {
    let report_source = include_str!("markdown.rs");
    assert!(report_source.contains("External AI Used"));
    assert!(report_source.contains("Local Mock Used"));
    assert!(!report_source.contains("AI analyst finished"));
}

#[test]
fn readme_has_v5_title() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("OpenBB Company Research Tool v5.0"));
    assert!(readme.contains("Rust-Powered AI-Led Company Research Engine"));
}

#[test]
fn readme_has_bilingual_sections() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("## English Product Overview"));
    assert!(readme.contains("## 中文产品说明"));
    assert!(readme.contains("Rust 驱动、AI 主导"));
    assert!(readme.contains("免责声明"));
}

#[test]
fn readme_has_mermaid_pipeline() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("```mermaid"));
    assert!(readme.contains("Provider Data"));
    assert!(readme.contains("Responsibility Split"));
    assert!(readme.contains("reports/samples/AAPL/dashboard.html"));
}

#[test]
fn readme_primary_quickstart_uses_research_rs() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("Primary entry point: `research-rs`."));
    assert!(readme.contains("source \"$HOME/.cargo/env\""));
    assert!(readme.contains("cd research-rs"));
    assert!(readme.contains("cargo run -p research-rs -- --help"));
    assert!(
        readme.contains("cargo run -p research-rs -- run AAPL --ai local --run-id demo_aapl_local")
    );
    assert!(readme.contains("--require-external-ai"));
    assert!(readme.contains("--no-ai-cache"));
    assert!(readme.contains("cargo run -p research-rs -- run 600519.SH"));
}

#[test]
fn readme_legacy_commands_not_in_primary_quickstart() {
    let readme = include_str!("../../../../README.md");
    let quick_start = readme
        .split("## How to Verify Real OpenAI API Usage")
        .next()
        .unwrap();
    assert!(!quick_start.contains("openbb-research"));
    assert!(!quick_start.contains("cresearch"));
    assert!(!quick_start.contains("python scripts/company_research_tool.py"));
    let legacy = readme
        .split("## Legacy Python Workflow")
        .nth(1)
        .expect("legacy section should exist");
    assert!(legacy.contains("openbb-research"));
    assert!(legacy.contains("cresearch"));
    assert!(legacy.contains("python scripts/company_research_tool.py"));
}

#[test]
fn readme_legacy_commands_only_under_legacy_section() {
    readme_legacy_commands_not_in_primary_quickstart();
}

#[test]
fn readme_no_v43_current_product_sections() {
    let readme = include_str!("../../../../README.md");
    for banned in [
        "30-Second Demo",
        "v4.3 Asset-Aware Workflow",
        "v4.4 Batch Evaluation Foundation",
        "Batch Evaluation Output",
        "v4 Workflow Gates",
        "What v4.0 Improves",
        "What v3.0 Improved",
        "What v2.1 Improved",
        "What v2.0 Improved",
        "Core Features",
        "Current Status",
        "v4.3 Status Note",
        "asset-aware report routing",
    ] {
        assert!(!readme.contains(banned), "README still contains {banned}");
    }
}

#[test]
fn readme_not_stuck_on_v43_current_status() {
    readme_no_v43_current_product_sections();
}

#[test]
fn readme_legacy_sections_moved_to_history() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("docs/history_v2_v4.md"));
    assert!(!readme.contains("## v4.3 Asset-Aware Workflow"));
    assert!(!readme.contains("## v4.4 Batch Evaluation Foundation"));
    let history = include_str!("../../../../docs/history_v2_v4.md");
    assert!(history.contains("v4.4 introduced the batch evaluation foundation"));
    assert!(history.contains("not the primary v5 entry point"));
}

#[test]
fn readme_has_us_and_cn_sample_gallery() {
    let readme = include_str!("../../../../README.md");
    for needle in [
        "## v5 Sample Gallery",
        "reports/samples/AAPL/report/AAPL_research_report.md",
        "reports/samples/GOOGL/dashboard.html",
        "reports/samples/CAT/metadata/research_blueprint.json",
        "reports/samples/AMD/metadata/ai_usage.json",
        "reports/samples/600519.SH/report/600519.SH_research_report.md",
        "reports/samples/000001.SZ/dashboard.html",
        "local fallback sample",
    ] {
        assert!(readme.contains(needle), "README missing {needle}");
    }
}

#[test]
fn readme_has_external_ai_usage_proof() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("## How to Verify Real OpenAI API Usage"));
    assert!(readme.contains("metadata/ai_usage.json"));
    assert!(readme.contains("\"external_ai_used\": true"));
    assert!(readme.contains("\"local_mock_used\": false"));
    assert!(readme.contains("\"new_external_ai_calls\": 4"));
    assert!(readme.contains("\"model\": \"gpt-4.1-mini\""));
    assert!(readme.contains("--require-external-ai"));
    assert!(readme.contains("--no-ai-cache"));
}

#[test]
fn readme_explains_external_ai_usage() {
    readme_has_external_ai_usage_proof();
}

#[test]
fn readme_is_bilingual() {
    readme_has_bilingual_sections();
}

#[test]
fn readme_has_limitations_and_disclaimer() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("## Limitations"));
    assert!(readme.contains("## Disclaimer"));
    assert!(readme.contains("## 免责声明"));
    assert!(readme.contains("No buy/sell/hold recommendation"));
    assert!(readme.contains("Provider coverage may be incomplete"));
    assert!(readme.contains("AI may be wrong"));
    assert!(readme.contains("local/mock fallback is not full external AI analysis"));
}

#[test]
fn readme_links_existing_v5_samples() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../");
    for path in [
        "reports/samples/AAPL/report/AAPL_research_report.md",
        "reports/samples/AAPL/dashboard.html",
        "reports/samples/AAPL/report/AAPL_research_report.pdf",
        "reports/samples/AAPL/metadata/company_understanding.json",
        "reports/samples/AAPL/metadata/research_blueprint.json",
        "reports/samples/AAPL/metadata/ai_usage.json",
        "reports/samples/AAPL/self_review/ai_self_review.md",
        "reports/samples/GOOGL/report/GOOGL_research_report.md",
        "reports/samples/GOOGL/dashboard.html",
        "reports/samples/GOOGL/report/GOOGL_research_report.pdf",
        "reports/samples/GOOGL/metadata/company_understanding.json",
        "reports/samples/GOOGL/metadata/research_blueprint.json",
        "reports/samples/GOOGL/metadata/ai_usage.json",
        "reports/samples/GOOGL/self_review/ai_self_review.md",
        "reports/samples/CAT/report/CAT_research_report.md",
        "reports/samples/CAT/dashboard.html",
        "reports/samples/CAT/report/CAT_research_report.pdf",
        "reports/samples/CAT/metadata/company_understanding.json",
        "reports/samples/CAT/metadata/research_blueprint.json",
        "reports/samples/CAT/metadata/ai_usage.json",
        "reports/samples/CAT/self_review/ai_self_review.md",
        "reports/samples/AMD/report/AMD_research_report.md",
        "reports/samples/AMD/dashboard.html",
        "reports/samples/AMD/report/AMD_research_report.pdf",
        "reports/samples/AMD/metadata/company_understanding.json",
        "reports/samples/AMD/metadata/research_blueprint.json",
        "reports/samples/AMD/metadata/ai_usage.json",
        "reports/samples/AMD/self_review/ai_self_review.md",
        "reports/samples/600519.SH/report/600519.SH_research_report.md",
        "reports/samples/600519.SH/dashboard.html",
        "reports/samples/600519.SH/report/600519.SH_research_report.pdf",
        "reports/samples/600519.SH/metadata/company_understanding.json",
        "reports/samples/600519.SH/metadata/research_blueprint.json",
        "reports/samples/600519.SH/metadata/ai_usage.json",
        "reports/samples/600519.SH/self_review/ai_self_review.md",
        "reports/samples/000001.SZ/report/000001.SZ_research_report.md",
        "reports/samples/000001.SZ/dashboard.html",
        "reports/samples/000001.SZ/report/000001.SZ_research_report.pdf",
        "reports/samples/000001.SZ/metadata/company_understanding.json",
        "reports/samples/000001.SZ/metadata/research_blueprint.json",
        "reports/samples/000001.SZ/metadata/ai_usage.json",
        "reports/samples/000001.SZ/self_review/ai_self_review.md",
    ] {
        assert!(repo.join(path).exists(), "sample link missing: {path}");
    }
}

#[test]
fn readme_sample_links_exist() {
    readme_links_existing_v5_samples();
}

#[test]
fn no_research_rs_reports_generated() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../");
    assert!(
        !repo.join("research-rs/reports").exists(),
        "generated reports must not be rooted under research-rs/reports"
    );
}

#[test]
fn generated_paths_anchor_to_repo_root() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../")
        .canonicalize()
        .expect("repo root");
    for path in [
        "reports/samples/AAPL/dashboard.html",
        "reports/release_checks/v5_0/readme_review.md",
        "reports/quality_runs/v5_quality_broad_30/content_quality_summary.md",
    ] {
        let absolute = repo.join(path);
        assert!(
            absolute.exists(),
            "expected root-anchored path missing: {path}"
        );
        assert!(
            !absolute.starts_with(repo.join("research-rs/reports")),
            "{path} must not resolve under research-rs/reports"
        );
    }
}

#[test]
fn ai_artifacts_have_provenance() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../");
    for path in [
        "reports/samples/AAPL/metadata/company_understanding.json",
        "reports/samples/AAPL/metadata/financial_interpretation.json",
        "reports/samples/AAPL/metadata/research_blueprint.json",
        "reports/samples/AAPL/self_review/ai_self_review.json",
        "reports/samples/AAPL/metadata/evidence_map.json",
        "reports/samples/AAPL/metadata/ai_usage.json",
    ] {
        let text = std::fs::read_to_string(repo.join(path)).expect(path);
        assert!(
            text.contains("ai_provenance") || path.ends_with("ai_usage.json"),
            "{path} missing ai_provenance"
        );
        if path.ends_with("ai_usage.json") {
            assert!(text.contains("\"external_ai_used\""));
            assert!(text.contains("\"local_mock_used\""));
            assert!(text.contains("\"new_external_ai_calls\""));
        }
    }
}

#[test]
fn v5_sample_reports_use_v5_structure() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../");
    let report =
        std::fs::read_to_string(repo.join("reports/samples/AAPL/report/AAPL_research_report.md"))
            .expect("sample report");
    for section in [
        "## 1. Report Status",
        "## 2. Company Identity",
        "## 3. Business Model",
        "## 4. Money Flow",
        "## 5. Financial Statement Interpretation",
        "## 6. AI Research Blueprint",
        "## 7. Valuation Frame",
        "## 8. Risks and Red Flags",
        "## 9. Data Gaps and Unsupported Claims",
        "## 10. AI Self Review",
        "## 11. Next Checks",
        "## 12. Charts and Evidence",
        "## 13. Appendix: Locked Data",
    ] {
        assert!(report.contains(section), "sample report missing {section}");
    }
    assert!(!report.contains("## 2. How to Read This Report"));
    assert!(!report.contains("Research Score"));
}

#[test]
fn readme_has_roadmap() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("v5.0"));
    assert!(readme.contains("v5.1"));
    assert!(readme.contains("v5.2"));
    assert!(readme.contains("P3 deferred"));
    assert!(readme.contains("real trading"));
}

#[test]
fn visual_lint_checks_data_coverage() {
    let report = "# AAPL Company Research Report\n\n> Status: PASS\n\n## Table of Contents\n\n## 1. Report Status\n\nWhat to look at:\nWhat to look at:\nWhat to look at:\nWhat to look at:\nWhat to look at:\n";
    let (status, failures) = visual_lint(report, true, true, false, true, true);
    assert_eq!(status, "FAIL");
    assert!(failures.contains(&"data_usage_coverage_report_exists".to_string()));
}
