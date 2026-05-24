use crate::cache::digest_str;
use crate::error::{ResearchError, ResearchErrorKind};
use crate::types::{
    AiSelfReview, CompanyProfile, CompanyUnderstanding, Confidence, FinancialInterpretation,
    ProviderPayload, ResearchBlueprint,
};
use crate::validation::{report_status, validate_ai_json, validate_provider_payload};

#[test]
fn cache_key_is_stable() {
    assert_eq!(digest_str("AAPL:v5"), digest_str("AAPL:v5"));
    assert_ne!(digest_str("AAPL:v5"), digest_str("MSFT:v5"));
}

#[test]
fn provider_error_taxonomy_has_user_action() {
    let err = ResearchError::provider_failure(
        "AAPL",
        "auto",
        "provider_fetch",
        "temporary timeout".to_string(),
    );
    assert_eq!(err.kind, ResearchErrorKind::ProviderError);
    assert!(err.recoverable);
    assert!(err.suggested_next_action.contains("--force"));
}

#[test]
fn provider_payload_validation_catches_missing_ticker() {
    let payload = ProviderPayload {
        ticker: "".to_string(),
        company_profile: CompanyProfile {
            name: "Example".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let failures = validate_provider_payload(&payload);
    assert!(failures.iter().any(|f| f.contains("ticker_missing")));
}

#[test]
fn low_confidence_requires_human_review() {
    let review = AiSelfReview {
        final_confidence: Confidence::LOW,
        human_review_required: false,
        ..Default::default()
    };
    let status = report_status(&[], &[], &review, "PASS".into(), "compact".into(), 0, 0);
    assert_eq!(status.overall_status, "WARNING");
    assert!(status.human_review_required);
}

#[test]
fn missing_not_this_boundary_is_ai_failure() {
    let understanding = CompanyUnderstanding {
        company_identity: "Company identity".into(),
        correct_research_frame: "Frame".into(),
        business_model: "Business model".into(),
        not_this: vec![],
        ..Default::default()
    };
    let interpretation = FinancialInterpretation {
        where_money_comes_from: "Operations".into(),
        where_money_goes: "Capex".into(),
        ..Default::default()
    };
    let blueprint = ResearchBlueprint {
        core_thesis: "This is a specific enough thesis for validation checks.".into(),
        asset_profile: "Mature Compounder".into(),
        must_analyze: vec!["margin durability".into()],
        ..Default::default()
    };
    let review = AiSelfReview::default();
    let failures = validate_ai_json(&understanding, &interpretation, &blueprint, &review);
    assert!(failures.contains(&"missing_not_this_boundary".to_string()));
}
