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
fn cannot_claim_external_ai_without_ai_usage_proof() {
    let report_source = include_str!("markdown.rs");
    assert!(report_source.contains("External AI Used"));
    assert!(report_source.contains("Local Mock Used"));
    assert!(!report_source.contains("AI analyst finished"));
}

#[test]
fn readme_is_bilingual() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("What It Does"));
    assert!(readme.contains("它做什么"));
    assert!(readme.contains("免责声明"));
}

#[test]
fn readme_has_mermaid_pipeline() {
    let readme = include_str!("../../../../README.md");
    assert!(readme.contains("```mermaid"));
    assert!(readme.contains("Provider Data"));
    assert!(readme.contains("Responsibility map"));
    assert!(readme.contains("reports/samples/AAPL/dashboard.html"));
}

#[test]
fn visual_lint_checks_data_coverage() {
    let report = "# AAPL Company Research Report\n\n> Status: PASS\n\n## Table of Contents\n\n## 1. Report Status\n\nWhat to look at:\nWhat to look at:\nWhat to look at:\nWhat to look at:\nWhat to look at:\n";
    let (status, failures) = visual_lint(report, true, true, false, true, true);
    assert_eq!(status, "FAIL");
    assert!(failures.contains(&"data_usage_coverage_report_exists".to_string()));
}
