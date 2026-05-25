use crate::{run_ai_usage_gate, run_local_compact_analyst, AiRunOptions};
use research_core::types::{AiUsage, CheckStatus, CompanyProfile, ProviderPayload};
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

#[test]
fn cat_like_payload_is_industrial_machinery_not_insurance() {
    let payload = ProviderPayload {
        ticker: "CAT".to_string(),
        company_profile: CompanyProfile {
            name: "Caterpillar Inc.".to_string(),
            sector: "Industrials".to_string(),
            industry: "Farm & Heavy Construction Machinery".to_string(),
            description:
                "Manufactures construction and mining equipment, diesel engines, turbines, and related parts and services."
                    .to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, _, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Industrial Machinery"));
    assert!(!understanding
        .correct_research_frame
        .to_lowercase()
        .contains("insurance"));
    assert!(understanding
        .not_this
        .iter()
        .any(|item| item.to_lowercase().contains("insurance")));
}

#[test]
fn jpm_like_payload_is_bank_not_consumer_retail() {
    let payload = ProviderPayload {
        ticker: "JPM".to_string(),
        company_profile: CompanyProfile {
            name: "JPMorgan Chase & Co.".to_string(),
            sector: "Financial Services".to_string(),
            industry: "Banks - Diversified".to_string(),
            description:
                "Provides consumer banking, commercial banking, investment banking, markets, asset management, cards, deposits, and lending."
                    .to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding.correct_research_frame.contains("Bank-like"));
    assert!(!understanding
        .correct_research_frame
        .to_lowercase()
        .contains("consumer / retail"));
    assert!(blueprint
        .must_not_analyze_as_core
        .iter()
        .any(|item| item.to_lowercase().contains("industrial")));
}

#[test]
fn isrg_like_payload_is_medtech_not_biotech() {
    let payload = ProviderPayload {
        ticker: "ISRG".to_string(),
        company_profile: CompanyProfile {
            name: "Intuitive Surgical, Inc.".to_string(),
            sector: "Healthcare".to_string(),
            industry: "Medical Devices".to_string(),
            description: "Develops robotic-assisted surgical systems, instruments, accessories, and services for hospitals.".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Medical Devices"));
    assert!(!understanding
        .correct_research_frame
        .to_lowercase()
        .contains("biotech"));
    assert!(blueprint
        .must_not_analyze_as_core
        .iter()
        .any(|item| item.to_lowercase().contains("biotech")));
}

fn lunr_payload() -> ProviderPayload {
    ProviderPayload {
        ticker: "LUNR".to_string(),
        company_profile: CompanyProfile {
            name: "Intuitive Machines, Inc.".to_string(),
            sector: "Industrials".to_string(),
            industry: "Aerospace & Defense".to_string(),
            description: "Space exploration company supporting lunar missions, NASA programs, landers, mission services, and cislunar infrastructure.".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn cn_payload(
    ticker: &str,
    name: &str,
    sector: &str,
    industry: &str,
    description: &str,
) -> ProviderPayload {
    ProviderPayload {
        ticker: ticker.to_string(),
        market: "CN_A".to_string(),
        company_profile: CompanyProfile {
            name: name.to_string(),
            sector: sector.to_string(),
            industry: industry.to_string(),
            description: description.to_string(),
            currency: "CNY".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn cn_catl_battery_not_unknown() {
    let payload = cn_payload(
        "300750.SZ",
        "宁德时代新能源科技股份有限公司",
        "电气设备",
        "制造业-电气机械和器材制造业",
        "锂离子电池、动力电池、储能电池、电池管理系统及新能源行业投资。",
    );
    let (understanding, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("New Energy / Battery Manufacturing"));
    assert!(!understanding.correct_research_frame.contains("Unknown"));
    assert!(interpretation.where_money_comes_from.contains("动力电池"));
    assert!(blueprint
        .must_analyze
        .iter()
        .any(|item| item.to_lowercase().contains("capex")));
}

#[test]
fn cn_catl_not_bank_or_consumer() {
    let payload = cn_payload(
        "300750.SZ",
        "宁德时代新能源科技股份有限公司",
        "电气设备",
        "制造业-电气机械和器材制造业",
        "动力电池、储能电池、电池管理系统及可充电电池包。",
    );
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    let text = format!(
        "{} {}",
        understanding.correct_research_frame,
        blueprint.must_not_analyze_as_core.join(" ")
    )
    .to_lowercase();
    assert!(!understanding.correct_research_frame.contains("Bank"));
    assert!(text.contains("bank"));
    assert!(text.contains("consumer"));
}

#[test]
fn cn_pingan_insurance_not_unknown() {
    let payload = cn_payload(
        "601318.SH",
        "中国平安保险(集团)股份有限公司",
        "金融",
        "金融业-保险业",
        "投资保险企业，开展保险资金运用业务，开展国内、国际保险业务。",
    );
    let (understanding, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Insurance / Integrated Financials"));
    assert!(!understanding.correct_research_frame.contains("Unknown"));
    assert!(interpretation.where_money_comes_from.contains("保费"));
    assert!(blueprint
        .must_analyze
        .iter()
        .any(|item| item.to_lowercase().contains("premium")));
}

#[test]
fn cn_pingan_not_industrial_fcf() {
    let payload = cn_payload(
        "601318.SH",
        "中国平安保险(集团)股份有限公司",
        "金融",
        "金融业-保险业",
        "寿险、财产保险、保险资金运用和综合金融服务。",
    );
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .not_this
        .iter()
        .any(|item| item.to_lowercase().contains("industrial free cash flow")));
    assert!(blueprint
        .must_not_analyze_as_core
        .iter()
        .any(|item| item.to_lowercase().contains("industrial fcf")));
}

#[test]
fn cn_hengrui_pharma_not_unknown() {
    let payload = cn_payload(
        "600276.SH",
        "江苏恒瑞医药股份有限公司",
        "医药生物",
        "制造业-医药制造业",
        "片剂、抗肿瘤药、原料药、创新药研发、生产和销售。",
    );
    let (understanding, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Pharma / Innovative Drug Portfolio"));
    assert!(!understanding.correct_research_frame.contains("Unknown"));
    assert!(interpretation.where_money_comes_from.contains("药"));
    assert!(blueprint
        .must_analyze
        .iter()
        .any(|item| item.to_lowercase().contains("r&d")));
}

#[test]
fn cn_hengrui_not_early_biotech_only() {
    let payload = cn_payload(
        "600276.SH",
        "江苏恒瑞医药股份有限公司",
        "医药生物",
        "制造业-医药制造业",
        "抗肿瘤药、创新药、药品研发和商业化销售。",
    );
    let (understanding, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .not_this
        .iter()
        .any(|item| item.contains("early biotech")));
    assert!(blueprint
        .must_not_analyze_as_core
        .iter()
        .any(|item| item.contains("early biotech")));
    assert!(!interpretation
        .valuation_method_fit
        .to_lowercase()
        .contains("cash runway only"));
}

#[test]
fn cn_zijin_mining_not_unknown() {
    let payload = cn_payload(
        "601899.SH",
        "紫金矿业集团股份有限公司",
        "有色金属",
        "采矿业-有色金属矿采选业",
        "金矿采选、金冶炼、铜矿采选、铜冶炼和矿产资源勘查。",
    );
    let (understanding, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Mining / Nonferrous Metals"));
    assert!(!understanding.correct_research_frame.contains("Unknown"));
    assert!(interpretation.where_money_comes_from.contains("黄金"));
    assert!(blueprint
        .must_analyze
        .iter()
        .any(|item| item.to_lowercase().contains("commodity")));
}

#[test]
fn cn_zijin_not_biotech_or_software() {
    let payload = cn_payload(
        "601899.SH",
        "紫金矿业集团股份有限公司",
        "有色金属",
        "采矿业-有色金属矿采选业",
        "矿产资源勘查、金矿采选、铜矿采选和有色金属冶炼。",
    );
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .not_this
        .iter()
        .any(|item| item.to_lowercase().contains("biotech")));
    assert!(understanding
        .not_this
        .iter()
        .any(|item| item.to_lowercase().contains("software")));
    assert!(blueprint
        .must_not_analyze_as_core
        .iter()
        .any(|item| item.to_lowercase().contains("software")));
}

#[test]
fn a_share_sector_frame_guard_uses_cn_ticker_and_profile() {
    let payload = cn_payload(
        "300750.SZ",
        "Example Battery Co.",
        "制造业",
        "制造业",
        "generic manufacturing description",
    );
    let (understanding, _, _, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding
        .correct_research_frame
        .contains("Battery Manufacturing"));
}

#[test]
fn a_share_unknown_only_when_profile_missing_or_conflicting() {
    let payload = ProviderPayload {
        ticker: "399999.SZ".to_string(),
        market: "CN_A".to_string(),
        company_profile: CompanyProfile {
            name: "Unknown A-share Co.".to_string(),
            currency: "CNY".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, _, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding.correct_research_frame.contains("Unknown"));
    assert!(understanding.human_review_required);
}

#[test]
fn lunr_not_telecom() {
    let payload = lunr_payload();
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(!understanding
        .correct_research_frame
        .contains("Telecom / Infrastructure Cash Flow"));
    assert!(!blueprint.asset_profile.contains("Telecom"));
}

#[test]
fn lunr_forbids_wireless_broadband_subscriber_churn() {
    let payload = lunr_payload();
    let (understanding, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    let text = format!(
        "{} {} {} {} {} {} {} {}",
        understanding.company_identity,
        understanding.business_model,
        understanding.revenue_engines.join(" "),
        understanding.key_risks.join(" "),
        interpretation.where_money_comes_from,
        interpretation.where_money_goes,
        blueprint.asset_profile,
        blueprint.must_analyze.join(" ")
    )
    .to_lowercase();
    for forbidden in [
        "wireless service revenue",
        "broadband / network revenue",
        "subscriber churn",
        "regulated telecom",
    ] {
        assert!(
            !text.contains(forbidden),
            "found forbidden term {forbidden}"
        );
    }
}

#[test]
fn lunr_money_flow_is_space_specific_or_data_limited() {
    let payload = lunr_payload();
    let (_, interpretation, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    let text = format!(
        "{} {} {}",
        interpretation.where_money_comes_from,
        interpretation.where_money_goes,
        blueprint.must_analyze.join(" ")
    )
    .to_lowercase();
    assert!(text.contains("nasa") || text.contains("mission") || text.contains("contract"));
    assert!(text.contains("cash runway") || text.contains("financing"));
    assert!(!text.contains("money comes from operating revenue when available"));
}

#[test]
fn self_review_flags_wrong_frame() {
    let payload = lunr_payload();
    let mut understanding = crate::company_understanding::understand_company(&payload);
    understanding.correct_research_frame = "Telecom / Infrastructure Cash Flow".into();
    understanding.company_identity = "LUNR is a telecom infrastructure cash flow company.".into();
    understanding.revenue_engines = vec!["wireless service revenue".into()];
    let interpretation =
        crate::financial_interpretation::interpret_financials(&payload, &understanding);
    let mut blueprint =
        crate::research_blueprint::build_blueprint(&payload, &understanding, &interpretation);
    blueprint.asset_profile = "Telecom / Infrastructure Cash Flow".into();
    blueprint.must_analyze = vec!["subscriber churn".into(), "broadband revenue".into()];
    let review = crate::self_review::review(&payload, &understanding, &interpretation, &blueprint);
    assert_eq!(review.framework_fit_check, CheckStatus::FAIL);
    assert!(review.human_review_required);
    assert!(review
        .required_rewrite_sections
        .iter()
        .any(|section| section == "Money Flow"));
}

#[test]
fn self_review_flags_hallucinated_revenue_engine() {
    self_review_flags_wrong_frame();
}

#[test]
fn self_review_flags_generic_chart_explanation() {
    let prompt = include_str!("../prompts/self_review_v1.md");
    assert!(prompt.contains("generic chart explanations"));
    assert!(prompt.contains("specific metric or data gap"));
}

#[test]
fn compact_payload_includes_company_description() {
    let payload = lunr_payload();
    let prompt = crate::client::compact_prompt(
        &payload,
        "company_understanding",
        "company_understanding_v1",
    );
    assert!(prompt.contains("Intuitive Machines"));
    assert!(prompt.contains("Aerospace & Defense"));
    assert!(prompt.contains("lunar missions"));
    assert!(prompt.contains("cislunar infrastructure"));
}

#[test]
fn compact_payload_missing_description_forces_data_limited() {
    let payload = ProviderPayload {
        ticker: "MISSING".into(),
        company_profile: CompanyProfile {
            name: "Missing Description Co.".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (understanding, _, blueprint, _, _, _) = run_local_compact_analyst(&payload);
    assert!(understanding.correct_research_frame.contains("Unknown"));
    assert!(understanding.human_review_required);
    assert!(blueprint.human_review_required);
}

#[test]
fn lunr_payload_contains_space_or_lunar_if_provider_has_it() {
    let payload = lunr_payload();
    let prompt = crate::client::compact_prompt(
        &payload,
        "company_understanding",
        "company_understanding_v1",
    )
    .to_lowercase();
    assert!(prompt.contains("space") || prompt.contains("lunar"));
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
