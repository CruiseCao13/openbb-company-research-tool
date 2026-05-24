use crate::cache::digest_str;
use crate::types::{CompanyProfile, ProviderPayload};
use crate::validation::validate_provider_payload;

#[test]
fn cache_key_is_stable() {
    assert_eq!(digest_str("AAPL:v5"), digest_str("AAPL:v5"));
    assert_ne!(digest_str("AAPL:v5"), digest_str("MSFT:v5"));
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
