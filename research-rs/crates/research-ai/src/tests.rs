use crate::run_local_compact_analyst;
use research_core::types::{CompanyProfile, ProviderPayload};

#[test]
fn google_like_payload_is_not_financials() {
    let payload = ProviderPayload {
        ticker: "GOOGL".to_string(),
        company_profile: CompanyProfile {
            name: "Alphabet Inc.".to_string(),
            sector: "Communication Services".to_string(),
            industry: "Internet Content & Information".to_string(),
            description: "Google Search advertising and cloud services.".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Platform Internet"));
    assert!(!blueprint.asset_profile.contains("Financial"));
}

#[test]
fn zim_like_payload_is_transport_not_biotech() {
    let payload = ProviderPayload {
        ticker: "ZIM".to_string(),
        company_profile: CompanyProfile {
            name: "ZIM Integrated Shipping".to_string(),
            sector: "Industrials".to_string(),
            industry: "Marine Shipping".to_string(),
            description: "Container shipping, vessels, freight rates, fleet utilization."
                .to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding.correct_research_frame.contains("Shipping"));
    assert!(blueprint
        .must_not_analyze_as_core
        .iter()
        .any(|x| x.contains("biotech")));
}
