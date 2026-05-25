use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct StudioPing {
    status: &'static str,
    message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    app_version: &'static str,
    repo_root: String,
    reports_root: String,
    platform: &'static str,
    studio_mode: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunSummary {
    ticker: String,
    run_id: String,
    market: Option<String>,
    provider: Option<String>,
    status: Option<String>,
    ai_source: Option<String>,
    external_ai_used: Option<bool>,
    local_mock_used: Option<bool>,
    cache_hits: Option<u64>,
    new_external_ai_calls: Option<u64>,
    human_review_required: Option<bool>,
    generated_at: Option<String>,
    report_path_exists: bool,
    dashboard_path_exists: bool,
    pdf_path_exists: bool,
    run_folder: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunDetail {
    ticker: String,
    run_id: String,
    run_folder: String,
    status: DetailStatus,
    ai_usage: DetailAiUsage,
    provider: DetailProvider,
    company: DetailCompany,
    financial_interpretation: DetailFinancialInterpretation,
    blueprint: DetailBlueprint,
    self_review: DetailSelfReview,
    charts: Vec<DetailChart>,
    artifacts: DetailArtifacts,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailStatus {
    overall_status: Option<String>,
    provider_status: Option<String>,
    visual_lint_status: Option<String>,
    pdf_export_status: Option<String>,
    human_review_required: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailAiUsage {
    source: Option<String>,
    external_ai_used: Option<bool>,
    local_mock_used: Option<bool>,
    cache_hits: Option<u64>,
    new_external_ai_calls: Option<u64>,
    model: Option<String>,
    prompt_versions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailProvider {
    provider: Option<String>,
    source: Option<String>,
    provider_adapter: Option<String>,
    package_used: Option<bool>,
    mock: Option<bool>,
    market: Option<String>,
    currency: Option<String>,
    limitations: Vec<String>,
    missing_fields: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailCompany {
    name: Option<String>,
    identity: Option<String>,
    frame: Option<String>,
    not_this: Vec<String>,
    confidence: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailFinancialInterpretation {
    revenue_explanation: Option<String>,
    margin_explanation: Option<String>,
    cash_flow_explanation: Option<String>,
    where_money_comes_from: Option<String>,
    where_money_goes: Option<String>,
    debt_and_financing: Option<String>,
    valuation_method_fit: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailBlueprint {
    core_thesis: Option<String>,
    asset_profile: Option<String>,
    must_analyze: Vec<String>,
    must_not_analyze_as_core: Vec<String>,
    key_questions: Vec<String>,
    red_flags: Vec<String>,
    data_gaps: Vec<String>,
    next_checks: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailSelfReview {
    company_understanding_check: Option<String>,
    framework_fit_check: Option<String>,
    numeric_consistency_check: Option<String>,
    money_flow_check: Option<String>,
    final_confidence: Option<String>,
    human_review_required: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailChart {
    title: String,
    image_path: Option<String>,
    source: Option<String>,
    status: Option<String>,
    explanation: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DetailArtifacts {
    markdown_report_path: Option<String>,
    pdf_report_path: Option<String>,
    dashboard_path: Option<String>,
    ai_usage_path: Option<String>,
    blueprint_path: Option<String>,
    validator_report_path: Option<String>,
    provider_payload_path: Option<String>,
}

#[tauri::command]
fn ping_studio() -> StudioPing {
    StudioPing {
        status: "ok",
        message: "v6 studio shell ready",
    }
}

#[tauri::command]
fn get_app_info() -> Result<AppInfo, String> {
    build_app_info()
}

#[tauri::command]
fn list_runs() -> Result<Vec<RunSummary>, String> {
    let repo_root = discover_repo_root()?;
    let reports_root = repo_root.join("reports");
    validate_reports_root(&repo_root, &reports_root)?;
    list_runs_from_reports_root(&reports_root)
}

#[tauri::command]
fn load_run_detail(ticker: String, run_id: String) -> Result<RunDetail, String> {
    let repo_root = discover_repo_root()?;
    let reports_root = repo_root.join("reports");
    validate_reports_root(&repo_root, &reports_root)?;
    load_run_detail_from_reports_root(&reports_root, &ticker, &run_id)
}

fn build_app_info() -> Result<AppInfo, String> {
    let repo_root = discover_repo_root()?;
    let reports_root = repo_root.join("reports");

    Ok(AppInfo {
        app_version: "v6.0-alpha",
        repo_root: path_to_string(&repo_root),
        reports_root: path_to_string(&reports_root),
        platform: std::env::consts::OS,
        studio_mode: "shell",
    })
}

fn load_run_detail_from_reports_root(
    reports_root: &Path,
    ticker: &str,
    run_id: &str,
) -> Result<RunDetail, String> {
    validate_safe_path_segment(ticker, "ticker")?;
    validate_safe_path_segment(run_id, "run_id")?;

    let run_path = reports_root.join(ticker).join("runs").join(run_id);
    ensure_path_under(reports_root, &run_path)?;

    if !run_path.is_dir() {
        return Err(format!(
            "run folder not found: reports/{ticker}/runs/{run_id}"
        ));
    }

    Ok(build_run_detail(ticker, run_id, &run_path))
}

fn build_run_detail(ticker: &str, run_id: &str, run_path: &Path) -> RunDetail {
    let mut warnings = Vec::new();
    let metadata_dir = run_path.join("metadata");
    let raw_dir = run_path.join("raw");
    let charts_dir = run_path.join("charts");
    let self_review_dir = run_path.join("self_review");

    let report_status = read_json_with_warning(
        &metadata_dir.join("report_status.json"),
        "metadata/report_status.json",
        true,
        &mut warnings,
    );
    let ai_usage = read_json_with_warning(
        &metadata_dir.join("ai_usage.json"),
        "metadata/ai_usage.json",
        true,
        &mut warnings,
    );
    let company_understanding = read_json_with_warning(
        &metadata_dir.join("company_understanding.json"),
        "metadata/company_understanding.json",
        true,
        &mut warnings,
    );
    let financial_interpretation = read_json_with_warning(
        &metadata_dir.join("financial_interpretation.json"),
        "metadata/financial_interpretation.json",
        true,
        &mut warnings,
    );
    let research_blueprint = read_json_with_warning(
        &metadata_dir.join("research_blueprint.json"),
        "metadata/research_blueprint.json",
        true,
        &mut warnings,
    );
    let self_review = read_json_with_warning(
        &self_review_dir.join("ai_self_review.json"),
        "self_review/ai_self_review.json",
        true,
        &mut warnings,
    );
    let provider_payload = read_json_with_warning(
        &raw_dir.join("provider_payload.json"),
        "raw/provider_payload.json",
        true,
        &mut warnings,
    );
    let provider_status = read_json_with_warning(
        &metadata_dir.join("provider_status.json"),
        "metadata/provider_status.json",
        false,
        &mut warnings,
    );
    let data_inventory = read_json_with_warning(
        &metadata_dir.join("data_inventory.json"),
        "metadata/data_inventory.json",
        false,
        &mut warnings,
    );
    let money_flow_map = read_json_with_warning(
        &metadata_dir.join("money_flow_map.json"),
        "metadata/money_flow_map.json",
        false,
        &mut warnings,
    );
    let product_quality_score = read_json_with_warning(
        &metadata_dir.join("product_quality_score.json"),
        "metadata/product_quality_score.json",
        false,
        &mut warnings,
    );
    let chart_table_quality = read_json_with_warning(
        &metadata_dir.join("chart_table_quality.json"),
        "metadata/chart_table_quality.json",
        false,
        &mut warnings,
    );
    let chart_manifest = read_json_with_warning(
        &charts_dir.join("chart_manifest.json"),
        "charts/chart_manifest.json",
        false,
        &mut warnings,
    );

    for relative in [
        "audit/validator_report.md",
        "audit/money_flow_quality_report.md",
        "audit/chart_table_quality_report.md",
        "audit/provider_validation.md",
        "README.md",
    ] {
        if !run_path.join(relative).is_file() {
            warnings.push(format!("optional file missing: {relative}"));
        }
    }

    let artifacts = build_detail_artifacts(run_path, ticker);

    if artifacts.markdown_report_path.is_none() {
        warnings.push("important artifact missing: report markdown".to_string());
    }
    if artifacts.dashboard_path.is_none() {
        warnings.push("important artifact missing: dashboard.html".to_string());
    }
    if artifacts.validator_report_path.is_none() {
        warnings.push("important artifact missing: audit/validator_report.md".to_string());
    }

    RunDetail {
        ticker: ticker.to_string(),
        run_id: run_id.to_string(),
        run_folder: path_to_string(run_path),
        status: build_detail_status(
            report_status.as_ref(),
            provider_status.as_ref(),
            product_quality_score.as_ref(),
            chart_table_quality.as_ref(),
        ),
        ai_usage: build_detail_ai_usage(ai_usage.as_ref()),
        provider: build_detail_provider(
            provider_payload.as_ref(),
            provider_status.as_ref(),
            data_inventory.as_ref(),
        ),
        company: build_detail_company(company_understanding.as_ref(), provider_payload.as_ref()),
        financial_interpretation: build_detail_financial_interpretation(
            financial_interpretation.as_ref(),
            money_flow_map.as_ref(),
        ),
        blueprint: build_detail_blueprint(research_blueprint.as_ref()),
        self_review: build_detail_self_review(self_review.as_ref()),
        charts: build_detail_charts(chart_manifest.as_ref(), &charts_dir),
        artifacts,
        warnings,
    }
}

fn build_detail_status(
    report_status: Option<&Value>,
    provider_status: Option<&Value>,
    product_quality_score: Option<&Value>,
    chart_table_quality: Option<&Value>,
) -> DetailStatus {
    DetailStatus {
        overall_status: report_status.and_then(report_status_value),
        provider_status: first_string(&[
            provider_status.and_then(|json| json_pointer_string(json, "/status")),
            provider_status.and_then(|json| json_pointer_string(json, "/provider_status")),
        ]),
        visual_lint_status: first_string(&[
            report_status.and_then(|json| json_pointer_string(json, "/visual_lint_status")),
            product_quality_score.and_then(|json| json_pointer_string(json, "/visual_lint_status")),
            chart_table_quality.and_then(|json| json_pointer_string(json, "/visual_lint_status")),
        ]),
        pdf_export_status: report_status
            .and_then(|json| json_pointer_string(json, "/pdf_export_status")),
        human_review_required: report_status
            .and_then(|json| json_pointer_bool(json, "/human_review_required")),
    }
}

fn build_detail_ai_usage(ai_usage: Option<&Value>) -> DetailAiUsage {
    let Some(ai_usage) = ai_usage else {
        return DetailAiUsage::default();
    };

    DetailAiUsage {
        source: ai_source(ai_usage),
        external_ai_used: json_pointer_bool(ai_usage, "/external_ai_used"),
        local_mock_used: json_pointer_bool(ai_usage, "/local_mock_used"),
        cache_hits: json_pointer_u64(ai_usage, "/cache_hits"),
        new_external_ai_calls: json_pointer_u64(ai_usage, "/new_external_ai_calls"),
        model: first_string(&[
            json_pointer_string(ai_usage, "/model"),
            ai_usage
                .pointer("/tasks")
                .and_then(Value::as_array)
                .and_then(|tasks| {
                    tasks
                        .iter()
                        .find_map(|task| json_pointer_string(task, "/model"))
                }),
        ]),
        prompt_versions: collect_prompt_versions(ai_usage),
    }
}

fn build_detail_provider(
    provider_payload: Option<&Value>,
    provider_status: Option<&Value>,
    data_inventory: Option<&Value>,
) -> DetailProvider {
    DetailProvider {
        provider: first_string(&[
            provider_status.and_then(|json| json_pointer_string(json, "/provider")),
            provider_payload.and_then(provider_name),
        ]),
        source: first_string(&[
            provider_status.and_then(|json| json_pointer_string(json, "/source")),
            provider_payload.and_then(|json| json_pointer_string(json, "/metadata/source")),
        ]),
        provider_adapter: first_string(&[
            provider_status.and_then(|json| json_pointer_string(json, "/provider_adapter")),
            provider_payload
                .and_then(|json| json_pointer_string(json, "/metadata/provider_adapter")),
        ]),
        package_used: first_bool(&[
            provider_status.and_then(|json| json_pointer_bool(json, "/package_used")),
            provider_payload.and_then(|json| json_pointer_bool(json, "/metadata/package_used")),
        ]),
        mock: first_bool(&[
            provider_status.and_then(|json| json_pointer_bool(json, "/mock")),
            provider_payload.and_then(|json| json_pointer_bool(json, "/metadata/mock")),
        ]),
        market: first_string(&[
            provider_status.and_then(|json| json_pointer_string(json, "/market")),
            provider_payload.and_then(provider_market),
        ]),
        currency: first_string(&[
            provider_status.and_then(|json| json_pointer_string(json, "/currency")),
            provider_payload.and_then(|json| json_pointer_string(json, "/currency")),
            provider_payload.and_then(|json| json_pointer_string(json, "/metadata/currency")),
        ]),
        limitations: collect_strings_from_paths(
            &[provider_status, provider_payload, data_inventory],
            &[
                "/limitations",
                "/provider_limitations",
                "/metadata/provider_limitations",
                "/data_quality_warnings",
                "/metadata/data_quality_warnings",
            ],
        ),
        missing_fields: collect_strings_from_paths(
            &[provider_status, data_inventory, provider_payload],
            &[
                "/missing_fields",
                "/data_coverage/missing_fields",
                "/metadata/missing_fields",
            ],
        ),
    }
}

fn build_detail_company(
    company_understanding: Option<&Value>,
    provider_payload: Option<&Value>,
) -> DetailCompany {
    DetailCompany {
        name: first_string(&[
            company_understanding.and_then(|json| json_pointer_string(json, "/company_name")),
            provider_payload.and_then(|json| json_pointer_string(json, "/company_profile/name")),
            provider_payload.and_then(|json| json_pointer_string(json, "/name")),
        ]),
        identity: company_understanding
            .and_then(|json| json_pointer_string(json, "/company_identity")),
        frame: first_string(&[
            company_understanding
                .and_then(|json| json_pointer_string(json, "/correct_research_frame")),
            company_understanding.and_then(|json| json_pointer_string(json, "/research_frame")),
            company_understanding.and_then(|json| json_pointer_string(json, "/asset_profile")),
        ]),
        not_this: company_understanding
            .map(|json| collect_strings_from_paths(&[Some(json)], &["/not_this"]))
            .unwrap_or_default(),
        confidence: company_understanding.and_then(|json| json_pointer_string(json, "/confidence")),
    }
}

fn build_detail_financial_interpretation(
    financial_interpretation: Option<&Value>,
    money_flow_map: Option<&Value>,
) -> DetailFinancialInterpretation {
    DetailFinancialInterpretation {
        revenue_explanation: financial_interpretation
            .and_then(|json| json_pointer_string(json, "/revenue_explanation")),
        margin_explanation: financial_interpretation
            .and_then(|json| json_pointer_string(json, "/margin_explanation")),
        cash_flow_explanation: financial_interpretation
            .and_then(|json| json_pointer_string(json, "/cash_flow_explanation")),
        where_money_comes_from: first_string(&[
            financial_interpretation
                .and_then(|json| json_pointer_string(json, "/where_money_comes_from")),
            money_flow_map.and_then(|json| json_pointer_string(json, "/where_money_comes_from")),
        ]),
        where_money_goes: first_string(&[
            financial_interpretation
                .and_then(|json| json_pointer_string(json, "/where_money_goes")),
            money_flow_map.and_then(|json| json_pointer_string(json, "/where_money_goes")),
        ]),
        debt_and_financing: financial_interpretation
            .and_then(|json| json_pointer_string(json, "/debt_and_financing")),
        valuation_method_fit: financial_interpretation
            .and_then(|json| json_pointer_string(json, "/valuation_method_fit")),
    }
}

fn build_detail_blueprint(research_blueprint: Option<&Value>) -> DetailBlueprint {
    let Some(research_blueprint) = research_blueprint else {
        return DetailBlueprint::default();
    };

    DetailBlueprint {
        core_thesis: json_pointer_string(research_blueprint, "/core_thesis"),
        asset_profile: json_pointer_string(research_blueprint, "/asset_profile"),
        must_analyze: collect_strings_from_paths(&[Some(research_blueprint)], &["/must_analyze"]),
        must_not_analyze_as_core: collect_strings_from_paths(
            &[Some(research_blueprint)],
            &["/must_not_analyze_as_core", "/must_not_analyze"],
        ),
        key_questions: collect_strings_from_paths(&[Some(research_blueprint)], &["/key_questions"]),
        red_flags: collect_strings_from_paths(&[Some(research_blueprint)], &["/red_flags"]),
        data_gaps: collect_strings_from_paths(&[Some(research_blueprint)], &["/data_gaps"]),
        next_checks: collect_strings_from_paths(&[Some(research_blueprint)], &["/next_checks"]),
    }
}

fn build_detail_self_review(self_review: Option<&Value>) -> DetailSelfReview {
    let Some(self_review) = self_review else {
        return DetailSelfReview::default();
    };

    DetailSelfReview {
        company_understanding_check: json_pointer_string(
            self_review,
            "/company_understanding_check",
        ),
        framework_fit_check: json_pointer_string(self_review, "/framework_fit_check"),
        numeric_consistency_check: json_pointer_string(self_review, "/numeric_consistency_check"),
        money_flow_check: json_pointer_string(self_review, "/money_flow_check"),
        final_confidence: first_string(&[
            json_pointer_string(self_review, "/final_confidence"),
            json_pointer_string(self_review, "/confidence"),
        ]),
        human_review_required: json_pointer_bool(self_review, "/human_review_required"),
    }
}

fn build_detail_charts(chart_manifest: Option<&Value>, charts_dir: &Path) -> Vec<DetailChart> {
    let Some(chart_manifest) = chart_manifest else {
        return Vec::new();
    };

    let charts = chart_manifest
        .as_array()
        .or_else(|| chart_manifest.pointer("/charts").and_then(Value::as_array))
        .or_else(|| chart_manifest.pointer("/figures").and_then(Value::as_array));

    charts
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, chart)| {
            let image_path = first_string(&[
                json_pointer_string(chart, "/image_path"),
                json_pointer_string(chart, "/path"),
                json_pointer_string(chart, "/file"),
            ]);
            DetailChart {
                title: first_string(&[
                    json_pointer_string(chart, "/title"),
                    json_pointer_string(chart, "/name"),
                ])
                .unwrap_or_else(|| format!("Chart {}", index + 1)),
                image_path: image_path.map(|path| path_to_string(&charts_dir.join(path))),
                source: json_pointer_string(chart, "/source"),
                status: json_pointer_string(chart, "/status"),
                explanation: first_string(&[
                    json_pointer_string(chart, "/explanation"),
                    json_pointer_string(chart, "/ai_explanation"),
                ]),
            }
        })
        .collect()
}

fn build_detail_artifacts(run_path: &Path, ticker: &str) -> DetailArtifacts {
    DetailArtifacts {
        markdown_report_path: find_markdown_report(run_path, ticker)
            .map(|path| path_to_string(&path)),
        pdf_report_path: existing_file_path(
            &run_path
                .join("report")
                .join(format!("{ticker}_research_report.pdf")),
        ),
        dashboard_path: existing_file_path(&run_path.join("dashboard.html")),
        ai_usage_path: existing_file_path(&run_path.join("metadata").join("ai_usage.json")),
        blueprint_path: existing_file_path(
            &run_path.join("metadata").join("research_blueprint.json"),
        ),
        validator_report_path: existing_file_path(
            &run_path.join("audit").join("validator_report.md"),
        ),
        provider_payload_path: existing_file_path(
            &run_path.join("raw").join("provider_payload.json"),
        ),
    }
}

fn read_json_with_warning(
    path: &Path,
    label: &str,
    important: bool,
    warnings: &mut Vec<String>,
) -> Option<Value> {
    if !path.is_file() {
        if important {
            warnings.push(format!("important file missing: {label}"));
        }
        return None;
    }

    match fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(value) => Some(value),
            Err(err) => {
                warnings.push(format!("malformed JSON in {label}: {err}"));
                None
            }
        },
        Err(err) => {
            warnings.push(format!("could not read {label}: {err}"));
            None
        }
    }
}

fn list_runs_from_reports_root(reports_root: &Path) -> Result<Vec<RunSummary>, String> {
    if !reports_root.exists() {
        return Ok(Vec::new());
    }

    if !reports_root.is_dir() {
        return Err(format!(
            "reports root is not a directory: {}",
            path_to_string(reports_root)
        ));
    }

    let mut runs = Vec::new();
    let ticker_dirs = read_dir_sorted(reports_root)?;

    for ticker_dir in ticker_dirs {
        let ticker_path = ticker_dir.path();
        if !ticker_path.is_dir() {
            continue;
        }

        let ticker = ticker_dir.file_name().to_string_lossy().into_owned();
        let runs_root = ticker_path.join("runs");
        if !runs_root.is_dir() {
            continue;
        }

        for run_dir in read_dir_sorted(&runs_root)? {
            let run_path = run_dir.path();
            if !run_path.is_dir() {
                continue;
            }

            let run_id = run_dir.file_name().to_string_lossy().into_owned();
            runs.push(build_run_summary(&ticker, &run_id, &run_path));
        }
    }

    runs.sort_by(compare_runs);
    Ok(runs)
}

fn build_run_summary(ticker: &str, run_id: &str, run_path: &Path) -> RunSummary {
    let report_status = read_json_optional(&run_path.join("metadata").join("report_status.json"));
    let ai_usage = read_json_optional(&run_path.join("metadata").join("ai_usage.json"));
    let provider_payload = read_json_optional(&run_path.join("raw").join("provider_payload.json"));

    let market = first_string(&[
        report_status
            .as_ref()
            .and_then(|json| json_pointer_string(json, "/market")),
        provider_payload.as_ref().and_then(provider_market),
    ]);
    let provider = first_string(&[
        report_status
            .as_ref()
            .and_then(|json| json_pointer_string(json, "/provider")),
        provider_payload.as_ref().and_then(provider_name),
    ]);
    let status = report_status.as_ref().and_then(report_status_value);
    let human_review_required = report_status
        .as_ref()
        .and_then(|json| json_pointer_bool(json, "/human_review_required"));
    let generated_at = first_string(&[
        report_status
            .as_ref()
            .and_then(|json| json_pointer_string(json, "/generated_at")),
        ai_usage
            .as_ref()
            .and_then(|json| json_pointer_string(json, "/generated_at")),
    ]);

    RunSummary {
        ticker: ticker.to_string(),
        run_id: run_id.to_string(),
        market,
        provider,
        status,
        ai_source: ai_usage.as_ref().and_then(ai_source),
        external_ai_used: ai_usage
            .as_ref()
            .and_then(|json| json_pointer_bool(json, "/external_ai_used")),
        local_mock_used: ai_usage
            .as_ref()
            .and_then(|json| json_pointer_bool(json, "/local_mock_used")),
        cache_hits: ai_usage
            .as_ref()
            .and_then(|json| json_pointer_u64(json, "/cache_hits")),
        new_external_ai_calls: ai_usage
            .as_ref()
            .and_then(|json| json_pointer_u64(json, "/new_external_ai_calls")),
        human_review_required,
        generated_at,
        report_path_exists: report_markdown_exists(run_path, ticker),
        dashboard_path_exists: run_path.join("dashboard.html").is_file(),
        pdf_path_exists: run_path
            .join("report")
            .join(format!("{ticker}_research_report.pdf"))
            .is_file(),
        run_folder: path_to_string(run_path),
    }
}

fn validate_reports_root(repo_root: &Path, reports_root: &Path) -> Result<(), String> {
    let repo_root = normalize_path(repo_root);
    let reports_root = normalize_path(reports_root);
    if reports_root.starts_with(&repo_root) {
        Ok(())
    } else {
        Err("reports root is outside repo root".to_string())
    }
}

fn read_dir_sorted(path: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let mut entries = fs::read_dir(path)
        .map_err(|err| format!("cannot read directory {}: {err}", path_to_string(path)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            format!(
                "cannot read directory entry in {}: {err}",
                path_to_string(path)
            )
        })?;

    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

fn read_json_optional(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn report_markdown_exists(run_path: &Path, ticker: &str) -> bool {
    find_markdown_report(run_path, ticker).is_some()
}

fn find_markdown_report(run_path: &Path, ticker: &str) -> Option<PathBuf> {
    let report_dir = run_path.join("report");
    let expected = report_dir.join(format!("{ticker}_research_report.md"));
    if expected.is_file() {
        return Some(expected);
    }

    fs::read_dir(report_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "md"))
}

fn existing_file_path(path: &Path) -> Option<String> {
    path.is_file().then(|| path_to_string(path))
}

fn validate_safe_path_segment(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} cannot be empty"));
    }

    if value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.contains("..")
    {
        return Err(format!("{label} contains an unsafe path segment"));
    }

    Ok(())
}

fn ensure_path_under(root: &Path, candidate: &Path) -> Result<(), String> {
    let root = normalize_path(root);
    let candidate = normalize_path(candidate);
    if candidate.starts_with(&root) {
        Ok(())
    } else {
        Err("resolved run folder is outside reports root".to_string())
    }
}

fn report_status_value(json: &Value) -> Option<String> {
    first_string(&[
        json_pointer_string(json, "/overall_status"),
        json_pointer_string(json, "/report_status"),
        json_pointer_string(json, "/status"),
        json_pointer_string(json, "/presentation_status"),
    ])
}

fn ai_source(json: &Value) -> Option<String> {
    first_string(&[
        json_pointer_string(json, "/source"),
        json_pointer_string(json, "/ai_source"),
        json.pointer("/tasks")
            .and_then(Value::as_array)
            .and_then(|tasks| {
                tasks
                    .iter()
                    .find_map(|task| json_pointer_string(task, "/source"))
            }),
    ])
}

fn provider_name(json: &Value) -> Option<String> {
    first_string(&[
        json_pointer_string(json, "/provider"),
        json_pointer_string(json, "/metadata/provider"),
        json_pointer_string(json, "/metadata/source"),
    ])
}

fn provider_market(json: &Value) -> Option<String> {
    first_string(&[
        json_pointer_string(json, "/market"),
        json_pointer_string(json, "/metadata/market"),
        json_pointer_string(json, "/company_profile/market"),
    ])
}

fn json_pointer_string(json: &Value, pointer: &str) -> Option<String> {
    json.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn json_pointer_bool(json: &Value, pointer: &str) -> Option<bool> {
    json.pointer(pointer).and_then(Value::as_bool)
}

fn json_pointer_u64(json: &Value, pointer: &str) -> Option<u64> {
    json.pointer(pointer).and_then(Value::as_u64)
}

fn json_pointer_array_strings(json: &Value, pointer: &str) -> Vec<String> {
    let Some(value) = json.pointer(pointer) else {
        return Vec::new();
    };

    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .or_else(|| {
                        item.as_object().and_then(|object| {
                            ["name", "label", "field", "issue", "description"]
                                .iter()
                                .find_map(|key| object.get(*key)?.as_str())
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(ToOwned::to_owned)
                        })
                    })
            })
            .collect(),
        Value::String(value) if !value.trim().is_empty() => vec![value.trim().to_string()],
        _ => Vec::new(),
    }
}

fn first_string(values: &[Option<String>]) -> Option<String> {
    values.iter().flatten().next().cloned()
}

fn first_bool(values: &[Option<bool>]) -> Option<bool> {
    values.iter().flatten().next().copied()
}

fn collect_strings_from_paths(json_values: &[Option<&Value>], paths: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    for json in json_values.iter().flatten() {
        for path in paths {
            for value in json_pointer_array_strings(json, path) {
                if !values.contains(&value) {
                    values.push(value);
                }
            }
        }
    }
    values
}

fn collect_prompt_versions(ai_usage: &Value) -> Vec<String> {
    let mut versions = collect_strings_from_paths(&[Some(ai_usage)], &["/prompt_versions"]);

    if let Some(tasks) = ai_usage.pointer("/tasks").and_then(Value::as_array) {
        for task in tasks {
            if let Some(version) = json_pointer_string(task, "/prompt_version") {
                if !versions.contains(&version) {
                    versions.push(version);
                }
            }
        }
    }

    versions
}

fn compare_runs(a: &RunSummary, b: &RunSummary) -> std::cmp::Ordering {
    match (&a.generated_at, &b.generated_at) {
        (Some(left), Some(right)) if left != right => right.cmp(left),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => a.ticker.cmp(&b.ticker).then(a.run_id.cmp(&b.run_id)),
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    path.components().collect()
}

fn discover_repo_root() -> Result<PathBuf, String> {
    let current_dir =
        std::env::current_dir().map_err(|err| format!("cannot read current directory: {err}"))?;

    for candidate in current_dir.ancestors() {
        if is_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }

    Err("repo root not found from current directory".to_string())
}

fn is_repo_root(path: &Path) -> bool {
    path.join("research-rs").join("Cargo.toml").is_file()
        && path.join("studio").join("index.html").is_file()
        && path.join("src-tauri").join("Cargo.toml").is_file()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ping_studio,
            get_app_info,
            list_runs,
            load_run_detail
        ])
        .run(tauri::generate_context!())
        .expect("failed to run v6 Tauri Research Studio");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};
    use tempfile::TempDir;

    #[test]
    fn get_app_info_returns_required_fields() {
        let app_info = build_app_info().expect("app info should resolve in the workspace");

        assert_eq!(app_info.app_version, "v6.0-alpha");
        assert!(!app_info.repo_root.is_empty());
        assert!(!app_info.reports_root.is_empty());
        assert!(!app_info.platform.is_empty());
        assert_eq!(app_info.studio_mode, "shell");
    }

    #[test]
    fn get_app_info_reports_root_points_to_repo_reports() {
        let app_info = build_app_info().expect("app info should resolve in the workspace");
        let repo_root = PathBuf::from(&app_info.repo_root);
        let reports_root = PathBuf::from(&app_info.reports_root);

        assert_eq!(reports_root, repo_root.join("reports"));
    }

    #[test]
    fn list_runs_returns_empty_when_reports_missing() {
        let temp_dir = TempDir::new().expect("temp dir");
        let runs = list_runs_from_reports_root(&temp_dir.path().join("reports")).expect("runs");

        assert!(runs.is_empty());
    }

    #[test]
    fn list_runs_reads_basic_report_status() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir
            .path()
            .join("reports")
            .join("AAPL")
            .join("runs")
            .join("demo");
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        write(
            run_path.join("metadata").join("report_status.json"),
            r#"{"overall_status":"PASS","market":"US","provider":"openbb","human_review_required":false,"generated_at":"2026-05-01T00:00:00Z"}"#,
        )
        .expect("report status");

        let runs = list_runs_from_reports_root(&temp_dir.path().join("reports")).expect("runs");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].ticker, "AAPL");
        assert_eq!(runs[0].run_id, "demo");
        assert_eq!(runs[0].status.as_deref(), Some("PASS"));
        assert_eq!(runs[0].market.as_deref(), Some("US"));
        assert_eq!(runs[0].provider.as_deref(), Some("openbb"));
        assert_eq!(runs[0].human_review_required, Some(false));
    }

    #[test]
    fn list_runs_handles_missing_ai_usage() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir
            .path()
            .join("reports")
            .join("CAT")
            .join("runs")
            .join("partial");
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        write(
            run_path.join("metadata").join("report_status.json"),
            r#"{"overall_status":"WARNING"}"#,
        )
        .expect("report status");

        let runs = list_runs_from_reports_root(&temp_dir.path().join("reports")).expect("runs");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status.as_deref(), Some("WARNING"));
        assert_eq!(runs[0].external_ai_used, None);
        assert_eq!(runs[0].local_mock_used, None);
        assert_eq!(runs[0].cache_hits, None);
    }

    #[test]
    fn list_runs_detects_artifact_existence() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir
            .path()
            .join("reports")
            .join("RKLB")
            .join("runs")
            .join("artifacts");
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        create_dir_all(run_path.join("report")).expect("report dir");
        write(run_path.join("dashboard.html"), "<html></html>").expect("dashboard");
        write(
            run_path.join("report").join("RKLB_research_report.md"),
            "# RKLB",
        )
        .expect("report");
        write(
            run_path.join("report").join("RKLB_research_report.pdf"),
            "%PDF",
        )
        .expect("pdf");

        let runs = list_runs_from_reports_root(&temp_dir.path().join("reports")).expect("runs");

        assert!(runs[0].report_path_exists);
        assert!(runs[0].dashboard_path_exists);
        assert!(runs[0].pdf_path_exists);
    }

    #[test]
    fn list_runs_does_not_crash_on_partial_run() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir
            .path()
            .join("reports")
            .join("GOOGL")
            .join("runs")
            .join("partial");
        create_dir_all(&run_path).expect("run dir");

        let runs = list_runs_from_reports_root(&temp_dir.path().join("reports")).expect("runs");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].ticker, "GOOGL");
        assert_eq!(runs[0].run_id, "partial");
        assert_eq!(runs[0].status, None);
    }

    #[test]
    fn list_runs_sorts_stably() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_run_with_generated_at(temp_dir.path(), "AAPL", "old", Some("2026-01-01T00:00:00Z"));
        create_run_with_generated_at(temp_dir.path(), "MSFT", "new", Some("2026-02-01T00:00:00Z"));
        create_run_with_generated_at(temp_dir.path(), "CAT", "no_date", None);

        let runs = list_runs_from_reports_root(&temp_dir.path().join("reports")).expect("runs");
        let keys = runs
            .iter()
            .map(|run| format!("{}:{}", run.ticker, run.run_id))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec!["MSFT:new", "AAPL:old", "CAT:no_date"]);
    }

    #[test]
    fn load_run_detail_rejects_path_traversal() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");

        let error = load_run_detail_from_reports_root(&reports_root, "../AAPL", "demo")
            .expect_err("path traversal should fail");

        assert!(error.contains("unsafe path segment"));
    }

    #[test]
    fn load_run_detail_missing_folder_returns_error() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");

        let error = load_run_detail_from_reports_root(&reports_root, "AAPL", "missing")
            .expect_err("missing run should fail");

        assert!(error.contains("run folder not found"));
    }

    #[test]
    fn load_run_detail_reads_required_metadata() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "detail");

        write_required_detail_metadata(&run_path);

        let detail =
            load_run_detail_from_reports_root(&temp_dir.path().join("reports"), "AAPL", "detail")
                .expect("detail");

        assert_eq!(detail.status.overall_status.as_deref(), Some("PASS"));
        assert_eq!(detail.ai_usage.external_ai_used, Some(true));
        assert_eq!(
            detail.company.identity.as_deref(),
            Some("Consumer technology ecosystem")
        );
        assert_eq!(
            detail.blueprint.core_thesis.as_deref(),
            Some("Hardware plus services")
        );
        assert_eq!(
            detail
                .financial_interpretation
                .where_money_comes_from
                .as_deref(),
            Some("Products and services")
        );
        assert_eq!(
            detail.self_review.framework_fit_check.as_deref(),
            Some("PASS")
        );
    }

    #[test]
    fn load_run_detail_handles_missing_optional_files() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "CAT", "minimal");
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        create_dir_all(run_path.join("self_review")).expect("self review dir");
        create_dir_all(run_path.join("raw")).expect("raw dir");

        write(
            run_path.join("metadata").join("report_status.json"),
            r#"{"overall_status":"WARNING"}"#,
        )
        .expect("report status");
        write(run_path.join("metadata").join("ai_usage.json"), "{}").expect("ai usage");
        write(
            run_path.join("metadata").join("company_understanding.json"),
            "{}",
        )
        .expect("company");
        write(
            run_path
                .join("metadata")
                .join("financial_interpretation.json"),
            "{}",
        )
        .expect("financial");
        write(
            run_path.join("metadata").join("research_blueprint.json"),
            "{}",
        )
        .expect("blueprint");
        write(
            run_path.join("self_review").join("ai_self_review.json"),
            "{}",
        )
        .expect("review");
        write(run_path.join("raw").join("provider_payload.json"), "{}").expect("provider");

        let detail =
            load_run_detail_from_reports_root(&temp_dir.path().join("reports"), "CAT", "minimal")
                .expect("detail");

        assert_eq!(detail.status.overall_status.as_deref(), Some("WARNING"));
        assert!(detail.charts.is_empty());
        assert!(detail
            .warnings
            .iter()
            .any(|warning| warning.contains("optional file missing")));
    }

    #[test]
    fn load_run_detail_collects_warnings() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "GOOGL", "warnings");
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        write(
            run_path.join("metadata").join("report_status.json"),
            "{not json",
        )
        .expect("bad json");

        let detail = build_run_detail("GOOGL", "warnings", &run_path);

        assert!(detail
            .warnings
            .iter()
            .any(|warning| warning.contains("malformed JSON in metadata/report_status.json")));
        assert!(detail
            .warnings
            .iter()
            .any(|warning| warning.contains("important file missing: metadata/ai_usage.json")));
    }

    #[test]
    fn load_run_detail_detects_artifact_paths() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "RKLB", "artifacts");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("audit")).expect("audit dir");
        create_dir_all(run_path.join("report")).expect("report dir");
        write(run_path.join("audit").join("validator_report.md"), "ok").expect("validator");
        write(run_path.join("dashboard.html"), "<html></html>").expect("dashboard");
        write(
            run_path.join("report").join("RKLB_research_report.md"),
            "# RKLB",
        )
        .expect("md");
        write(
            run_path.join("report").join("RKLB_research_report.pdf"),
            "%PDF",
        )
        .expect("pdf");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "RKLB",
            "artifacts",
        )
        .expect("detail");

        assert!(detail.artifacts.markdown_report_path.is_some());
        assert!(detail.artifacts.pdf_report_path.is_some());
        assert!(detail.artifacts.dashboard_path.is_some());
        assert!(detail.artifacts.validator_report_path.is_some());
    }

    #[test]
    fn load_run_detail_does_not_return_raw_provider_payload() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "raw_guard");
        write_required_detail_metadata(&run_path);
        write(
            run_path.join("raw").join("provider_payload.json"),
            r#"{"provider":"openbb","raw_big_secret_like_field":"SHOULD_NOT_RETURN"}"#,
        )
        .expect("provider");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "AAPL",
            "raw_guard",
        )
        .expect("detail");
        let serialized = serde_json::to_string(&detail).expect("serialize");

        assert!(!serialized.contains("SHOULD_NOT_RETURN"));
        assert!(serialized.contains("provider_payload_path"));
    }

    #[test]
    fn load_run_detail_parses_ai_usage_summary() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "JPM", "ai");
        write_required_detail_metadata(&run_path);
        write(
            run_path.join("metadata").join("ai_usage.json"),
            r#"{"source":"external_openai","external_ai_used":true,"local_mock_used":false,"cache_hits":0,"new_external_ai_calls":4,"model":"gpt-4.1-mini","prompt_versions":["company_understanding_v1"],"tasks":[{"prompt_version":"research_blueprint_v1"}]}"#,
        )
        .expect("ai usage");

        let detail =
            load_run_detail_from_reports_root(&temp_dir.path().join("reports"), "JPM", "ai")
                .expect("detail");

        assert_eq!(detail.ai_usage.source.as_deref(), Some("external_openai"));
        assert_eq!(detail.ai_usage.external_ai_used, Some(true));
        assert_eq!(detail.ai_usage.new_external_ai_calls, Some(4));
        assert_eq!(detail.ai_usage.model.as_deref(), Some("gpt-4.1-mini"));
        assert!(detail
            .ai_usage
            .prompt_versions
            .contains(&"company_understanding_v1".to_string()));
        assert!(detail
            .ai_usage
            .prompt_versions
            .contains(&"research_blueprint_v1".to_string()));
    }

    fn create_run_with_generated_at(
        root: &Path,
        ticker: &str,
        run_id: &str,
        generated_at: Option<&str>,
    ) {
        let run_path = root.join("reports").join(ticker).join("runs").join(run_id);
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        let generated_at_json = generated_at
            .map(|value| format!(r#","generated_at":"{value}""#))
            .unwrap_or_default();
        write(
            run_path.join("metadata").join("report_status.json"),
            format!(r#"{{"overall_status":"PASS"{generated_at_json}}}"#),
        )
        .expect("report status");
    }

    fn create_detail_fixture(root: &Path, ticker: &str, run_id: &str) -> PathBuf {
        let run_path = root.join("reports").join(ticker).join("runs").join(run_id);
        create_dir_all(&run_path).expect("run dir");
        run_path
    }

    fn write_required_detail_metadata(run_path: &Path) {
        create_dir_all(run_path.join("metadata")).expect("metadata dir");
        create_dir_all(run_path.join("self_review")).expect("self review dir");
        create_dir_all(run_path.join("raw")).expect("raw dir");
        write(
            run_path.join("metadata").join("report_status.json"),
            r#"{"overall_status":"PASS","human_review_required":false}"#,
        )
        .expect("report status");
        write(
            run_path.join("metadata").join("ai_usage.json"),
            r#"{"source":"external_openai","external_ai_used":true,"local_mock_used":false,"cache_hits":0,"new_external_ai_calls":4,"model":"gpt-4.1-mini"}"#,
        )
        .expect("ai usage");
        write(
            run_path.join("metadata").join("company_understanding.json"),
            r#"{"company_name":"Apple Inc.","company_identity":"Consumer technology ecosystem","correct_research_frame":"Mature Consumer Technology Compounder","not_this":["Bank"],"confidence":"HIGH"}"#,
        )
        .expect("company");
        write(
            run_path.join("metadata").join("financial_interpretation.json"),
            r#"{"revenue_explanation":"Product and service revenue","margin_explanation":"Services mix matters","cash_flow_explanation":"Operating cash flow funds returns","where_money_comes_from":"Products and services","where_money_goes":"COGS, R&D, buybacks","debt_and_financing":"Debt is not operating lifeline","valuation_method_fit":"Earnings and FCF screening"}"#,
        )
        .expect("financial");
        write(
            run_path.join("metadata").join("research_blueprint.json"),
            r#"{"core_thesis":"Hardware plus services","asset_profile":"Consumer tech","must_analyze":["iPhone demand"],"must_not_analyze_as_core":["Bank NIM"],"key_questions":["Services durability"],"red_flags":["China demand"],"data_gaps":["Segment margin"],"next_checks":["Check latest 10-Q"]}"#,
        )
        .expect("blueprint");
        write(
            run_path.join("self_review").join("ai_self_review.json"),
            r#"{"company_understanding_check":"PASS","framework_fit_check":"PASS","numeric_consistency_check":"PASS","money_flow_check":"PASS","final_confidence":"HIGH","human_review_required":false}"#,
        )
        .expect("review");
        write(
            run_path.join("raw").join("provider_payload.json"),
            r#"{"provider":"openbb","market":"US","currency":"USD","company_profile":{"name":"Apple Inc."},"metadata":{"source":"OpenBB","package_used":true,"mock":false,"provider_limitations":["Coverage varies"],"missing_fields":["segment_margin"]}}"#,
        )
        .expect("provider");
    }
}
