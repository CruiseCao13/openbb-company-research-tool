use crate::cache::digest_str;
use crate::error::{ResearchError, ResearchErrorKind};
use crate::paths::{
    ai_cache_dir, batch_runs_dir, quality_runs_dir, release_checks_dir, samples_dir,
    training_cases_dir,
};
use crate::provider::{discover_repo_root, discover_repo_root_from, resolve_provider_script};
use crate::run_folder::RunFolder;
use crate::types::{
    AiSelfReview, CompanyProfile, CompanyUnderstanding, Confidence, FinancialInterpretation,
    ProviderPayload, ResearchBlueprint, RunContext,
};
use crate::validation::{report_status, validate_ai_json, validate_provider_payload};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

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
fn provider_path_resolves_from_repo_root() {
    let root = discover_repo_root().expect("repo root should be discoverable");
    let resolved = discover_repo_root_from(&root).expect("repo root should resolve from itself");
    assert_eq!(resolved, root);
    assert!(resolved.join("providers/provider_common.py").exists());
}

#[test]
fn provider_path_resolves_from_research_rs_dir() {
    let root = discover_repo_root().expect("repo root should be discoverable");
    let research_rs = root.join("research-rs");
    let resolved =
        discover_repo_root_from(&research_rs).expect("repo root should resolve from research-rs");
    assert_eq!(resolved, root);
    let script = resolve_provider_script("providers/provider_common.py")
        .expect("provider script should resolve through repo root");
    assert_eq!(script, root.join("providers/provider_common.py"));
}

#[test]
fn provider_path_env_override_works() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp_root =
        std::env::temp_dir().join(format!("research_engine_root_test_{}", std::process::id()));
    let providers = temp_root.join("providers");
    std::fs::create_dir_all(&providers).unwrap();
    std::fs::write(providers.join("provider_common.py"), "# fixture\n").unwrap();
    let start =
        std::env::temp_dir().join(format!("research_engine_env_start_{}", std::process::id()));
    std::fs::create_dir_all(&start).unwrap();
    std::env::set_var("RESEARCH_ENGINE_ROOT", &temp_root);
    let resolved = discover_repo_root_from(&start).expect("env override should resolve root");
    assert_eq!(resolved, temp_root);
    std::env::remove_var("RESEARCH_ENGINE_ROOT");
}

#[test]
fn provider_path_missing_gives_clear_error() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("RESEARCH_ENGINE_ROOT");
    let start = std::env::temp_dir().join(format!(
        "research_engine_missing_test_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&start).unwrap();
    let err = discover_repo_root_from(&start).unwrap_err().to_string();
    assert!(err.contains("providers/provider_common.py not found"));
    assert!(err.contains("Set RESEARCH_ENGINE_ROOT="));
}

#[test]
fn run_folder_reports_root_is_repo_root_anchored() {
    let root = discover_repo_root().expect("repo root should resolve");
    let ctx = RunContext {
        ticker: "AAPL".into(),
        market: "US".into(),
        provider: "auto".into(),
        ai_mode: "local".into(),
        run_id: "path_test".into(),
        root: "reports".into(),
        force: false,
        pack: false,
        lang: "en".into(),
        mode: "standard".into(),
        require_external_ai: false,
        no_ai_cache: false,
        max_attempts: 2,
        auto_fix: false,
        fail_fast: false,
    };
    let folder = RunFolder::new(&ctx);
    assert_eq!(folder.root, root.join("reports/AAPL/runs/path_test"));
}

#[test]
fn ai_cache_path_anchored_to_repo_root_from_repo_root() {
    let root = discover_repo_root().expect("repo root should resolve");
    let cache = ai_cache_dir().expect("AI cache dir should resolve");
    assert_eq!(cache, root.join("reports/_cache/ai"));
}

#[test]
fn ai_cache_path_anchored_to_repo_root_from_research_rs_dir() {
    let root = discover_repo_root().expect("repo root should resolve");
    let cache = ai_cache_dir().expect("AI cache dir should resolve");
    assert!(!cache.starts_with(root.join("research-rs/reports")));
    assert_eq!(cache, root.join("reports/_cache/ai"));
}

#[test]
fn batch_runs_path_anchored_to_repo_root() {
    let root = discover_repo_root().expect("repo root should resolve");
    assert_eq!(
        batch_runs_dir().expect("batch runs dir should resolve"),
        root.join("reports/batch_runs")
    );
}

#[test]
fn quality_runs_path_anchored_to_repo_root() {
    let root = discover_repo_root().expect("repo root should resolve");
    assert_eq!(
        quality_runs_dir().expect("quality runs dir should resolve"),
        root.join("reports/quality_runs")
    );
}

#[test]
fn samples_path_anchored_to_repo_root() {
    let root = discover_repo_root().expect("repo root should resolve");
    assert_eq!(
        samples_dir().expect("samples dir should resolve"),
        root.join("reports/samples")
    );
}

#[test]
fn release_checks_path_anchored_to_repo_root() {
    let root = discover_repo_root().expect("repo root should resolve");
    assert_eq!(
        release_checks_dir().expect("release checks dir should resolve"),
        root.join("reports/release_checks")
    );
}

#[test]
fn training_cases_path_anchored_to_repo_root() {
    let root = discover_repo_root().expect("repo root should resolve");
    assert_eq!(
        training_cases_dir().expect("training cases dir should resolve"),
        root.join("training_cases")
    );
}

#[test]
fn no_research_rs_reports_created_during_run() {
    let root = discover_repo_root().expect("repo root should resolve");
    for path in [
        ai_cache_dir().unwrap(),
        batch_runs_dir().unwrap(),
        quality_runs_dir().unwrap(),
        samples_dir().unwrap(),
        release_checks_dir().unwrap(),
        training_cases_dir().unwrap(),
    ] {
        assert!(
            !path.starts_with(root.join("research-rs/reports")),
            "{} should not be rooted under research-rs/reports",
            path.display()
        );
    }
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
