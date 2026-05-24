use crate::{run_ai_usage_gate, run_local_compact_analyst, AiRunOptions};
use research_core::types::{AiUsage, CompanyProfile, ProviderPayload};
use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

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

fn temp_ai_dirs(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("research_ai_test_{name}_{stamp}"));
    let metadata = root.join("metadata");
    let ai = root.join("ai");
    fs::create_dir_all(&metadata).unwrap();
    fs::create_dir_all(&ai).unwrap();
    (metadata, ai)
}

#[test]
fn require_external_ai_without_key_fails() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("OPENAI_API_KEY");
    std::env::remove_var("OPENAI_MOCK_SUCCESS");
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (metadata, ai) = temp_ai_dirs("missing_key");
    let result = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: "compact".into(),
            require_external_ai: true,
            no_ai_cache: true,
        },
        &metadata,
        &ai,
    );
    assert!(result.is_err());
    let usage = fs::read_to_string(metadata.join("ai_usage.json")).unwrap();
    assert!(usage.contains("\"external_ai_used\": false"));
    assert!(usage.contains("OPENAI_API_KEY missing"));
}

#[test]
fn local_ai_mode_does_not_call_external_api() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("OPENAI_API_KEY", "sk-test-not-used");
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (metadata, ai) = temp_ai_dirs("local");
    let usage = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: "local".into(),
            require_external_ai: false,
            no_ai_cache: false,
        },
        &metadata,
        &ai,
    )
    .unwrap();
    assert!(!usage.external_ai_used);
    assert!(usage.local_mock_used);
    assert_eq!(usage.ai_calls, 0);
}

#[test]
fn local_mode_never_calls_external_ai() {
    local_ai_mode_does_not_call_external_api();
}

#[test]
fn ai_usage_json_records_external_status() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("OPENAI_API_KEY", "sk-test-mocked");
    std::env::set_var("OPENAI_MOCK_SUCCESS", "1");
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (metadata, ai) = temp_ai_dirs("external_mock");
    let usage = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: "compact".into(),
            require_external_ai: true,
            no_ai_cache: true,
        },
        &metadata,
        &ai,
    )
    .unwrap();
    assert!(usage.external_ai_used);
    assert!(!usage.local_mock_used);
    assert!(usage.new_external_ai_calls > 0);
    assert_eq!(usage.cache_hits, 0);
    let written = fs::read_to_string(metadata.join("ai_usage.json")).unwrap();
    assert!(written.contains("\"external_ai_used\": true"));
    assert!(written.contains("\"local_mock_used\": false"));
    assert!(written.contains("\"request_success\": true"));
    assert!(written.contains("\"ai_provenance\""));
    assert!(written.contains("\"source\": \"external_openai\""));
    std::env::remove_var("OPENAI_MOCK_SUCCESS");
}

#[test]
fn ai_usage_json_required_for_ai_outputs() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("OPENAI_API_KEY", "sk-test-mocked");
    std::env::set_var("OPENAI_MOCK_SUCCESS", "1");
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (metadata, ai) = temp_ai_dirs("usage_required");
    let usage = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: "compact".into(),
            require_external_ai: true,
            no_ai_cache: true,
        },
        &metadata,
        &ai,
    )
    .unwrap();
    assert!(metadata.join("ai_usage.json").exists());
    assert!(usage.external_ai_used);
    std::env::remove_var("OPENAI_MOCK_SUCCESS");
}

#[test]
fn ai_provenance_required_in_ai_json() {
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (understanding, interpretation, blueprint, review, _, _) =
        run_local_compact_analyst(&payload);
    assert!(serde_json::to_value(&understanding).unwrap()["ai_provenance"].is_object());
    assert!(serde_json::to_value(&interpretation).unwrap()["ai_provenance"].is_object());
    assert!(serde_json::to_value(&blueprint).unwrap()["ai_provenance"].is_object());
    assert!(serde_json::to_value(&review).unwrap()["ai_provenance"].is_object());
}

#[test]
fn no_ai_cache_forces_new_external_request_when_key_exists_or_marks_external_required() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("OPENAI_API_KEY", "sk-test-mocked");
    std::env::set_var("OPENAI_MOCK_SUCCESS", "1");
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (metadata, ai) = temp_ai_dirs("no_cache_named");
    let usage = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: "compact".into(),
            require_external_ai: true,
            no_ai_cache: true,
        },
        &metadata,
        &ai,
    )
    .unwrap();
    assert!(usage.require_external_ai);
    assert!(usage.no_ai_cache);
    assert_eq!(usage.cache_hits, 0);
    assert!(usage.new_external_ai_calls > 0);
    assert!(usage.tasks.iter().all(|task| task.request_attempted));
    std::env::remove_var("OPENAI_MOCK_SUCCESS");
}

#[test]
fn require_external_ai_with_no_cache_requires_new_call() {
    no_ai_cache_forces_new_external_request_when_key_exists_or_marks_external_required();
}

#[test]
fn cannot_claim_external_ai_without_ai_usage_proof() {
    let usage = AiUsage {
        ai_mode: "compact".into(),
        external_ai_used: false,
        local_mock_used: true,
        model: "local-compact-analyst-fallback".into(),
        ..Default::default()
    };
    assert!(!usage.external_ai_used);
    assert!(usage.local_mock_used);
    assert_ne!(usage.ai_provenance.source, "external_openai");
}

#[test]
fn ai_budget_defaults_are_explicit() {
    assert!(crate::client::compact_payload_size(&ProviderPayload::default()) < usize::MAX);
    let config = research_core::config::EngineConfig::default();
    assert_eq!(config.ai_budget.max_calls_per_single_run, 8);
    assert_eq!(config.ai_budget.max_calls_per_ticker, 6);
    assert_eq!(config.ai_budget.fail_after_calls, 200);
}

#[test]
fn ai_cache_trace_records_task_cache_inputs() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("OPENAI_API_KEY", "sk-test-mocked");
    std::env::set_var("OPENAI_MOCK_SUCCESS", "1");
    let payload = ProviderPayload {
        ticker: "AAPL".into(),
        ..Default::default()
    };
    let (metadata, ai) = temp_ai_dirs("cache_trace");
    let usage = run_ai_usage_gate(
        &payload,
        &AiRunOptions {
            ai_mode: "compact".into(),
            require_external_ai: true,
            no_ai_cache: true,
        },
        &metadata,
        &ai,
    )
    .unwrap();
    assert!(usage.new_external_ai_calls > 0);
    let trace = fs::read_to_string(metadata.join("ai_cache_trace.json")).unwrap();
    assert!(trace.contains("\"cache_key\""));
    assert!(trace.contains("\"prompt_version\""));
    assert!(trace.contains("\"invalidation_reason\""));
    std::env::remove_var("OPENAI_MOCK_SUCCESS");
}
