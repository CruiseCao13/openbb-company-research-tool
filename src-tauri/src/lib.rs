use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    audit_trail: Vec<AuditTrailStage>,
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
    image_exists: bool,
    source: Option<String>,
    status: Option<String>,
    why_selected: Option<String>,
    what_to_look_at: Option<String>,
    what_it_means: Option<String>,
    what_not_to_overread: Option<String>,
    next_check: Option<String>,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditTrailStage {
    stage: String,
    label: String,
    status: String,
    source: Option<String>,
    message: Option<String>,
    artifact_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ArtifactActionResult {
    ok: bool,
    action: &'static str,
    path: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TrainingRunSummary {
    run_id: String,
    path: String,
    has_quality_matrix: bool,
    has_issue_distribution: bool,
    has_training_cases: bool,
    generated_at: Option<String>,
    summary_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct QualityMatrix {
    run_id: String,
    rows: Vec<QualityMatrixRow>,
    issue_distribution: Vec<IssueDistributionItem>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct QualityMatrixRow {
    ticker: String,
    market: Option<String>,
    company_name: Option<String>,
    status: Option<String>,
    quality_score: Option<f64>,
    grade: Option<String>,
    issue_types: Vec<String>,
    hard_failures: Vec<String>,
    ai_source: Option<String>,
    provider_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IssueDistributionItem {
    issue_type: String,
    count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtifactAction {
    Open,
    Reveal,
}

#[derive(Debug)]
struct ValidatedArtifactPath {
    canonical_path: PathBuf,
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

#[tauri::command]
fn list_training_runs() -> Result<Vec<TrainingRunSummary>, String> {
    let repo_root = discover_repo_root()?;
    let training_root = repo_root.join("reports").join("training_runs");
    list_training_runs_from_root(&training_root)
}

#[tauri::command]
fn load_quality_matrix(run_id: String) -> Result<QualityMatrix, String> {
    let repo_root = discover_repo_root()?;
    let training_root = repo_root.join("reports").join("training_runs");
    load_quality_matrix_from_root(&training_root, &run_id)
}

#[tauri::command]
fn open_artifact(path: String) -> Result<ArtifactActionResult, String> {
    let repo_root = discover_repo_root()?;
    let reports_root = repo_root.join("reports");
    let artifact = validate_artifact_path(&reports_root, &path, ArtifactAction::Open)?;
    open_path_with_os(&artifact.canonical_path)?;

    Ok(ArtifactActionResult {
        ok: true,
        action: "open",
        path: path_to_string(&artifact.canonical_path),
        message: "artifact open request sent".to_string(),
    })
}

#[tauri::command]
fn reveal_in_folder(path: String) -> Result<ArtifactActionResult, String> {
    let repo_root = discover_repo_root()?;
    let reports_root = repo_root.join("reports");
    let artifact = validate_artifact_path(&reports_root, &path, ArtifactAction::Reveal)?;
    reveal_path_with_os(&artifact.canonical_path)?;

    Ok(ArtifactActionResult {
        ok: true,
        action: "reveal",
        path: path_to_string(&artifact.canonical_path),
        message: "artifact reveal request sent".to_string(),
    })
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

fn list_training_runs_from_root(training_root: &Path) -> Result<Vec<TrainingRunSummary>, String> {
    if !training_root.exists() {
        return Ok(Vec::new());
    }

    if !training_root.is_dir() {
        return Err(format!(
            "training runs root is not a directory: {}",
            path_to_string(training_root)
        ));
    }

    let mut runs = Vec::new();
    for entry in read_dir_sorted(training_root)? {
        let run_path = entry.path();
        if !run_path.is_dir() {
            continue;
        }
        let run_id = entry.file_name().to_string_lossy().into_owned();
        runs.push(build_training_run_summary(&run_id, &run_path));
    }

    runs.sort_by(
        |left, right| match (&left.generated_at, &right.generated_at) {
            (Some(left_date), Some(right_date)) if left_date != right_date => {
                right_date.cmp(left_date)
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => left.run_id.cmp(&right.run_id),
        },
    );

    Ok(runs)
}

fn build_training_run_summary(run_id: &str, run_path: &Path) -> TrainingRunSummary {
    let summary = read_json_optional(&run_path.join("training_summary.json"))
        .or_else(|| read_json_optional(&run_path.join("final_acceptance.json")));

    TrainingRunSummary {
        run_id: run_id.to_string(),
        path: path_to_string(run_path),
        has_quality_matrix: run_path.join("quality_matrix.json").is_file()
            || run_path.join("quality_matrix.csv").is_file(),
        has_issue_distribution: run_path.join("issue_distribution.json").is_file()
            || run_path.join("issue_distribution.md").is_file(),
        has_training_cases: run_path.join("training_cases_generated.jsonl").is_file()
            || run_path
                .join("external_correction_cases_generated.jsonl")
                .is_file(),
        generated_at: first_string(&[
            summary
                .as_ref()
                .and_then(|json| json_pointer_string(json, "/generated_at")),
            summary
                .as_ref()
                .and_then(|json| json_pointer_string(json, "/completed_at")),
        ]),
        summary_status: summary.as_ref().and_then(report_status_value),
    }
}

fn load_quality_matrix_from_root(
    training_root: &Path,
    run_id: &str,
) -> Result<QualityMatrix, String> {
    validate_safe_path_segment(run_id, "run_id")?;
    let run_path = training_root.join(run_id);
    ensure_path_under(training_root, &run_path)?;

    if !run_path.exists() {
        return Ok(QualityMatrix {
            run_id: run_id.to_string(),
            rows: Vec::new(),
            issue_distribution: Vec::new(),
            warnings: vec![format!("training run not found: {run_id}")],
        });
    }

    let mut warnings = Vec::new();
    let rows = if run_path.join("quality_matrix.json").is_file() {
        parse_quality_matrix_json(&run_path.join("quality_matrix.json"))?
    } else if run_path.join("quality_matrix.csv").is_file() {
        parse_quality_matrix_csv(&run_path.join("quality_matrix.csv"), &mut warnings)?
    } else {
        warnings.push("quality_matrix.json/csv missing".to_string());
        Vec::new()
    };

    let issue_distribution = parse_issue_distribution(&run_path, &rows, &mut warnings)?;

    Ok(QualityMatrix {
        run_id: run_id.to_string(),
        rows,
        issue_distribution,
        warnings,
    })
}

fn parse_quality_matrix_json(path: &Path) -> Result<Vec<QualityMatrixRow>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("could not read quality matrix JSON: {err}"))?;
    let json: Value = serde_json::from_str(&raw)
        .map_err(|err| format!("malformed quality_matrix.json: {err}"))?;
    let rows = json
        .as_array()
        .or_else(|| json.pointer("/rows").and_then(Value::as_array))
        .or_else(|| json.pointer("/quality_matrix").and_then(Value::as_array))
        .ok_or_else(|| "quality_matrix.json does not contain rows".to_string())?;

    Ok(rows.iter().filter_map(row_from_json).collect())
}

fn row_from_json(row: &Value) -> Option<QualityMatrixRow> {
    let ticker = first_string(&[
        json_pointer_string(row, "/ticker"),
        json_pointer_string(row, "/symbol"),
    ])?;

    Some(QualityMatrixRow {
        ticker,
        market: json_pointer_string(row, "/market"),
        company_name: first_string(&[
            json_pointer_string(row, "/company_name"),
            json_pointer_string(row, "/name"),
        ]),
        status: first_string(&[
            json_pointer_string(row, "/status"),
            json_pointer_string(row, "/overall_status"),
            json_pointer_string(row, "/report_status"),
        ]),
        quality_score: first_f64(&[
            json_pointer_f64(row, "/quality_score"),
            json_pointer_f64(row, "/total_quality_score"),
            json_pointer_f64(row, "/score"),
        ]),
        grade: json_pointer_string(row, "/grade"),
        issue_types: collect_strings_from_paths(&[Some(row)], &["/issue_types", "/issues"]),
        hard_failures: collect_strings_from_paths(&[Some(row)], &["/hard_failures"]),
        ai_source: first_string(&[
            json_pointer_string(row, "/ai_source"),
            json_pointer_string(row, "/source"),
        ]),
        provider_status: json_pointer_string(row, "/provider_status"),
    })
}

fn parse_quality_matrix_csv(
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<QualityMatrixRow>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("could not read quality matrix CSV: {err}"))?;
    let mut lines = raw.lines().filter(|line| !line.trim().is_empty());
    let Some(header_line) = lines.next() else {
        warnings.push("quality_matrix.csv is empty".to_string());
        return Ok(Vec::new());
    };
    let headers = split_csv_line(header_line)
        .into_iter()
        .map(|header| header.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let mut rows = Vec::new();

    for line in lines {
        let cells = split_csv_line(line);
        let value = |names: &[&str]| -> Option<String> {
            names.iter().find_map(|name| {
                headers
                    .iter()
                    .position(|header| header == name)
                    .and_then(|index| cells.get(index))
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
            })
        };

        let Some(ticker) = value(&["ticker", "symbol"]) else {
            warnings.push("CSV row skipped because ticker column is missing or empty".to_string());
            continue;
        };

        rows.push(QualityMatrixRow {
            ticker,
            market: value(&["market"]),
            company_name: value(&["company_name", "name"]),
            status: value(&["status", "overall_status", "report_status"]),
            quality_score: value(&["quality_score", "total_quality_score", "score"])
                .and_then(|value| value.parse::<f64>().ok()),
            grade: value(&["grade"]),
            issue_types: split_list_value(value(&["issue_types", "issues"]).as_deref()),
            hard_failures: split_list_value(value(&["hard_failures"]).as_deref()),
            ai_source: value(&["ai_source", "source"]),
            provider_status: value(&["provider_status"]),
        });
    }

    Ok(rows)
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                cells.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    cells.push(current.trim().to_string());
    cells
}

fn split_list_value(value: Option<&str>) -> Vec<String> {
    value
        .into_iter()
        .flat_map(|value| value.split([';', '|', ',']))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_issue_distribution(
    run_path: &Path,
    rows: &[QualityMatrixRow],
    warnings: &mut Vec<String>,
) -> Result<Vec<IssueDistributionItem>, String> {
    if run_path.join("issue_distribution.json").is_file() {
        let raw = fs::read_to_string(run_path.join("issue_distribution.json"))
            .map_err(|err| format!("could not read issue_distribution.json: {err}"))?;
        let json: Value = serde_json::from_str(&raw)
            .map_err(|err| format!("malformed issue_distribution.json: {err}"))?;
        if let Some(items) = json.as_array() {
            return Ok(items
                .iter()
                .filter_map(|item| {
                    Some(IssueDistributionItem {
                        issue_type: first_string(&[
                            json_pointer_string(item, "/issue_type"),
                            json_pointer_string(item, "/issue"),
                        ])?,
                        count: json_pointer_u64(item, "/count").unwrap_or(1),
                    })
                })
                .collect());
        }
        if let Some(object) = json.as_object() {
            return Ok(object
                .iter()
                .filter_map(|(issue_type, count)| {
                    count.as_u64().map(|count| IssueDistributionItem {
                        issue_type: issue_type.clone(),
                        count,
                    })
                })
                .collect());
        }
        warnings.push("issue_distribution.json shape is unsupported".to_string());
    }

    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for row in rows {
        for issue in row.issue_types.iter().chain(row.hard_failures.iter()) {
            *counts.entry(issue.clone()).or_default() += 1;
        }
    }

    Ok(counts
        .into_iter()
        .map(|(issue_type, count)| IssueDistributionItem { issue_type, count })
        .collect())
}

fn validate_artifact_path(
    reports_root: &Path,
    path: &str,
    action: ArtifactAction,
) -> Result<ValidatedArtifactPath, String> {
    if path.trim().is_empty() {
        return Err("artifact path cannot be empty".to_string());
    }

    let raw_path = PathBuf::from(path);
    if path.contains('\0')
        || raw_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err("artifact path contains unsafe traversal".to_string());
    }

    let reports_root = reports_root
        .canonicalize()
        .map_err(|err| format!("reports root cannot be resolved: {err}"))?;

    let candidate = if raw_path.is_absolute() {
        raw_path
    } else {
        reports_root.join(raw_path)
    };

    if !candidate.exists() {
        return Err(format!(
            "artifact path does not exist: {}",
            path_to_string(&candidate)
        ));
    }

    let canonical_path = candidate
        .canonicalize()
        .map_err(|err| format!("artifact path cannot be resolved: {err}"))?;

    if !canonical_path.starts_with(&reports_root) {
        return Err("artifact path is outside reports root".to_string());
    }

    match action {
        ArtifactAction::Open => {
            if !canonical_path.is_file() {
                return Err("open_artifact requires a file path".to_string());
            }
            if !is_allowed_artifact_file(&canonical_path, &reports_root) {
                return Err("artifact file type or location is not allowed".to_string());
            }
        }
        ArtifactAction::Reveal => {
            if !(canonical_path.is_file() || canonical_path.is_dir()) {
                return Err("reveal_in_folder requires an existing file or directory".to_string());
            }
        }
    }

    Ok(ValidatedArtifactPath { canonical_path })
}

fn is_allowed_artifact_file(path: &Path, reports_root: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(reports_root) else {
        return false;
    };

    let components = relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();

    if components.len() < 4 {
        return false;
    }

    let Some(parent) = components.get(3).copied() else {
        return false;
    };
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if components.len() == 4 && file_name == "dashboard.html" {
        return true;
    }

    match parent {
        "report" => matches!(extension, "md" | "pdf"),
        "metadata" => extension == "json",
        "raw" => file_name == "provider_payload.json",
        "audit" => extension == "md",
        "self_review" => extension == "md",
        "charts" => extension == "png",
        "pack" => extension == "zip",
        _ => false,
    }
}

fn open_path_with_os(path: &Path) -> Result<(), String> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()
    } else if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).status()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(path).status()
    } else {
        return Err("opening artifacts is unsupported on this platform".to_string());
    }
    .map_err(|err| format!("failed to launch OS open command: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("OS open command failed with status: {status}"))
    }
}

fn reveal_path_with_os(path: &Path) -> Result<(), String> {
    let status = if cfg!(target_os = "macos") {
        Command::new("open")
            .args(["-R", &path_to_string(path)])
            .status()
    } else if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(format!(
                "/select,{}",
                path_to_string(path).replace('/', "\\")
            ))
            .status()
    } else if cfg!(target_os = "linux") {
        let target = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };
        Command::new("xdg-open").arg(target).status()
    } else {
        return Err("revealing artifacts is unsupported on this platform".to_string());
    }
    .map_err(|err| format!("failed to launch OS reveal command: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("OS reveal command failed with status: {status}"))
    }
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

    let status = build_detail_status(
        report_status.as_ref(),
        provider_status.as_ref(),
        product_quality_score.as_ref(),
        chart_table_quality.as_ref(),
    );
    let ai_usage_detail = build_detail_ai_usage(ai_usage.as_ref());
    let provider_detail = build_detail_provider(
        provider_payload.as_ref(),
        provider_status.as_ref(),
        data_inventory.as_ref(),
    );
    let audit_trail = build_audit_trail(
        run_path,
        &artifacts,
        &status,
        &ai_usage_detail,
        &provider_detail,
        report_status.as_ref(),
        ai_usage.as_ref(),
        provider_payload.as_ref(),
    );

    RunDetail {
        ticker: ticker.to_string(),
        run_id: run_id.to_string(),
        run_folder: path_to_string(run_path),
        status,
        ai_usage: ai_usage_detail,
        provider: provider_detail,
        company: build_detail_company(company_understanding.as_ref(), provider_payload.as_ref()),
        financial_interpretation: build_detail_financial_interpretation(
            financial_interpretation.as_ref(),
            money_flow_map.as_ref(),
        ),
        blueprint: build_detail_blueprint(research_blueprint.as_ref()),
        self_review: build_detail_self_review(self_review.as_ref()),
        charts: build_detail_charts(chart_manifest.as_ref(), &charts_dir),
        artifacts,
        audit_trail,
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
            let image_path_value = first_string(&[
                json_pointer_string(chart, "/image_path"),
                json_pointer_string(chart, "/path"),
                json_pointer_string(chart, "/file"),
            ]);
            let image_path = image_path_value
                .as_deref()
                .and_then(|path| safe_chart_image_path(charts_dir, path));
            let image_exists = image_path.as_ref().is_some_and(|path| path.is_file());
            DetailChart {
                title: first_string(&[
                    json_pointer_string(chart, "/title"),
                    json_pointer_string(chart, "/name"),
                ])
                .unwrap_or_else(|| format!("Chart {}", index + 1)),
                image_path: image_path.map(|path| path_to_string(&path)),
                image_exists,
                source: json_pointer_string(chart, "/source"),
                status: first_string(&[
                    json_pointer_string(chart, "/status"),
                    (!image_exists).then(|| "WARNING".to_string()),
                ]),
                why_selected: first_string(&[
                    json_pointer_string(chart, "/why_selected"),
                    json_pointer_string(chart, "/purpose"),
                ]),
                what_to_look_at: first_string(&[
                    json_pointer_string(chart, "/what_to_look_at"),
                    json_pointer_string(chart, "/how_to_read"),
                ]),
                what_it_means: first_string(&[
                    json_pointer_string(chart, "/what_it_means"),
                    json_pointer_string(chart, "/explanation"),
                    json_pointer_string(chart, "/ai_explanation"),
                ]),
                what_not_to_overread: first_string(&[
                    json_pointer_string(chart, "/what_not_to_overread"),
                    json_pointer_string(chart, "/limitation"),
                    json_pointer_string(chart, "/limitations"),
                    json_pointer_string(chart, "/warning"),
                ]),
                next_check: json_pointer_string(chart, "/next_check"),
            }
        })
        .collect()
}

fn safe_chart_image_path(charts_dir: &Path, path: &str) -> Option<PathBuf> {
    if path.trim().is_empty() {
        return None;
    }
    let raw_path = PathBuf::from(path);
    if raw_path.is_absolute()
        || raw_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }

    let candidate = charts_dir.join(raw_path);
    ensure_path_under(charts_dir, &candidate).ok()?;
    Some(candidate)
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

fn build_audit_trail(
    run_path: &Path,
    artifacts: &DetailArtifacts,
    status: &DetailStatus,
    ai_usage: &DetailAiUsage,
    provider: &DetailProvider,
    report_status: Option<&Value>,
    ai_usage_json: Option<&Value>,
    provider_payload: Option<&Value>,
) -> Vec<AuditTrailStage> {
    let metadata_dir = run_path.join("metadata");
    let audit_dir = run_path.join("audit");
    let self_review_dir = run_path.join("self_review");
    let report_dir = run_path.join("report");
    let pack_dir = run_path.join("pack");

    let provider_artifact = artifacts.provider_payload_path.clone();
    let provider_missing = !provider.missing_fields.is_empty()
        || provider
            .limitations
            .iter()
            .any(|item| contains_any_ci(item, &["data gap", "missing", "limited"]));
    let provider_status = if provider_payload.is_none() {
        "fail"
    } else if provider_missing || status_contains(status.provider_status.as_deref(), "warn") {
        "warning"
    } else {
        "pass"
    };

    let locked_validation_status =
        match report_status.and_then(|json| json_pointer_bool(json, "/provider_payload_valid")) {
            Some(true) => "pass",
            Some(false) => "fail",
            None if audit_dir.join("provider_validation.md").is_file() => "unknown",
            None => "warning",
        };

    vec![
        AuditTrailStage {
            stage: "provider_fetch".to_string(),
            label: "Provider Fetch".to_string(),
            status: provider_status.to_string(),
            source: first_string(&[
                provider.provider.clone(),
                provider.source.clone(),
                provider.provider_adapter.clone(),
            ]),
            message: Some(provider_stage_message(provider)),
            artifact_path: provider_artifact,
        },
        AuditTrailStage {
            stage: "locked_data_validation".to_string(),
            label: "Locked Data Validation".to_string(),
            status: locked_validation_status.to_string(),
            source: Some("metadata/report_status.json + audit/provider_validation.md".to_string()),
            message: Some(
                "Uses completed-run metadata and provider validation artifacts; no live validation stream."
                    .to_string(),
            ),
            artifact_path: existing_file_path(&audit_dir.join("provider_validation.md")),
        },
        build_ai_audit_stage(
            "ai_company_understanding",
            "AI Company Understanding",
            &metadata_dir.join("company_understanding.json"),
            ai_usage,
            ai_usage_json,
            "company_understanding",
        ),
        build_ai_audit_stage(
            "ai_financial_interpretation",
            "AI Financial Interpretation",
            &metadata_dir.join("financial_interpretation.json"),
            ai_usage,
            ai_usage_json,
            "financial_interpretation",
        ),
        build_ai_audit_stage(
            "ai_research_blueprint",
            "AI Research Blueprint",
            &metadata_dir.join("research_blueprint.json"),
            ai_usage,
            ai_usage_json,
            "research_blueprint",
        ),
        AuditTrailStage {
            stage: "report_rendering".to_string(),
            label: "Report Rendering".to_string(),
            status: if artifacts.markdown_report_path.is_some() {
                status_to_audit_status(status.overall_status.as_deref())
            } else {
                "fail".to_string()
            },
            source: Some("report/*.md + metadata/report_status.json".to_string()),
            message: Some(
                artifacts
                    .markdown_report_path
                    .as_ref()
                    .map(|_| "Markdown report artifact exists.".to_string())
                    .unwrap_or_else(|| "Markdown report artifact is missing.".to_string()),
            ),
            artifact_path: artifacts.markdown_report_path.clone(),
        },
        AuditTrailStage {
            stage: "ai_self_review".to_string(),
            label: "AI Self Review".to_string(),
            status: ai_stage_status(
                self_review_dir.join("ai_self_review.json").is_file(),
                ai_usage,
                ai_usage_json,
                "self_review",
            ),
            source: ai_stage_source(ai_usage),
            message: Some(ai_stage_message(
                self_review_dir.join("ai_self_review.json").is_file(),
                ai_usage,
                "self_review",
            )),
            artifact_path: existing_file_path(&self_review_dir.join("ai_self_review.md")),
        },
        AuditTrailStage {
            stage: "validator_lint".to_string(),
            label: "Validator / Lint".to_string(),
            status: if audit_dir.join("validator_report.md").is_file() {
                status_to_audit_status(status.visual_lint_status.as_deref())
            } else {
                "warning".to_string()
            },
            source: Some("audit/validator_report.md + visual_lint_status".to_string()),
            message: Some(
                status
                    .visual_lint_status
                    .clone()
                    .unwrap_or_else(|| "Validator report status is unknown.".to_string()),
            ),
            artifact_path: artifacts.validator_report_path.clone(),
        },
        AuditTrailStage {
            stage: "pdf_pack_export".to_string(),
            label: "PDF / Pack / Export".to_string(),
            status: export_stage_status(status, artifacts, &pack_dir),
            source: Some("report/*.pdf + pack/*.zip + pdf_export_status".to_string()),
            message: Some(export_stage_message(status, artifacts, &pack_dir)),
            artifact_path: artifacts
                .pdf_report_path
                .clone()
                .or_else(|| find_first_file_with_extension(&pack_dir, "zip").map(|path| path_to_string(&path)))
                .or_else(|| existing_file_path(&report_dir)),
        },
    ]
}

fn build_ai_audit_stage(
    stage: &str,
    label: &str,
    artifact_path: &Path,
    ai_usage: &DetailAiUsage,
    ai_usage_json: Option<&Value>,
    task_name: &str,
) -> AuditTrailStage {
    let exists = artifact_path.is_file();
    AuditTrailStage {
        stage: stage.to_string(),
        label: label.to_string(),
        status: ai_stage_status(exists, ai_usage, ai_usage_json, task_name),
        source: ai_stage_source(ai_usage),
        message: Some(ai_stage_message(exists, ai_usage, task_name)),
        artifact_path: existing_file_path(artifact_path),
    }
}

fn ai_stage_status(
    artifact_exists: bool,
    ai_usage: &DetailAiUsage,
    ai_usage_json: Option<&Value>,
    task_name: &str,
) -> String {
    if !artifact_exists {
        return "warning".to_string();
    }
    if task_request_success(ai_usage_json, task_name) == Some(false) {
        return "fail".to_string();
    }
    if ai_usage.local_mock_used == Some(true) {
        return "warning".to_string();
    }
    if ai_usage.cache_hits.unwrap_or(0) > 0 {
        return "cached".to_string();
    }
    if ai_usage.external_ai_used == Some(true) {
        return "pass".to_string();
    }
    "unknown".to_string()
}

fn ai_stage_source(ai_usage: &DetailAiUsage) -> Option<String> {
    if ai_usage.local_mock_used == Some(true) {
        Some("local_mock".to_string())
    } else if ai_usage.external_ai_used == Some(true) {
        Some(
            ai_usage
                .source
                .clone()
                .unwrap_or_else(|| "external_openai".to_string()),
        )
    } else {
        ai_usage.source.clone()
    }
}

fn ai_stage_message(artifact_exists: bool, ai_usage: &DetailAiUsage, task_name: &str) -> String {
    if !artifact_exists {
        return format!("{task_name} artifact is missing.");
    }
    if ai_usage.local_mock_used == Some(true) {
        return format!("{task_name} used local/mock AI; treat as non-external analysis.");
    }
    if ai_usage.cache_hits.unwrap_or(0) > 0 {
        return format!("{task_name} may include cached AI output.");
    }
    if ai_usage.external_ai_used == Some(true) {
        return format!("{task_name} artifact exists with external AI provenance.");
    }
    format!("{task_name} artifact exists; AI source is unknown.")
}

fn task_request_success(ai_usage: Option<&Value>, task_name: &str) -> Option<bool> {
    ai_usage?
        .pointer("/tasks")
        .and_then(Value::as_array)?
        .iter()
        .find(|task| {
            first_string(&[
                json_pointer_string(task, "/task"),
                json_pointer_string(task, "/name"),
                json_pointer_string(task, "/stage"),
            ])
            .is_some_and(|name| name.contains(task_name))
        })
        .and_then(|task| json_pointer_bool(task, "/request_success"))
}

fn provider_stage_message(provider: &DetailProvider) -> String {
    let mut parts = Vec::new();
    if let Some(provider_name) = &provider.provider {
        parts.push(format!("Provider: {provider_name}."));
    }
    if let Some(source) = &provider.source {
        parts.push(format!("Source: {source}."));
    }
    if provider.mock == Some(true) {
        parts.push("Mock provider data is flagged.".to_string());
    }
    if !provider.missing_fields.is_empty() {
        parts.push(format!(
            "Missing fields: {}.",
            provider.missing_fields.join(", ")
        ));
    }
    if parts.is_empty() {
        "Provider metadata is unavailable.".to_string()
    } else {
        parts.join(" ")
    }
}

fn status_to_audit_status(status: Option<&str>) -> String {
    let Some(status) = status else {
        return "unknown".to_string();
    };
    let normalized = status.to_ascii_lowercase();
    if normalized.contains("fail") {
        "fail".to_string()
    } else if normalized.contains("warn")
        || normalized.contains("review")
        || normalized.contains("degraded")
    {
        "warning".to_string()
    } else if normalized.contains("pass") {
        "pass".to_string()
    } else if normalized.contains("skip") {
        "skipped".to_string()
    } else {
        "unknown".to_string()
    }
}

fn export_stage_status(
    status: &DetailStatus,
    artifacts: &DetailArtifacts,
    pack_dir: &Path,
) -> String {
    if status_contains(status.pdf_export_status.as_deref(), "fail") {
        return "fail".to_string();
    }
    if status_contains(status.pdf_export_status.as_deref(), "warn") {
        return "warning".to_string();
    }
    if artifacts.pdf_report_path.is_some()
        || find_first_file_with_extension(pack_dir, "zip").is_some()
    {
        return status_to_audit_status(status.pdf_export_status.as_deref())
            .replace("unknown", "pass");
    }
    "unknown".to_string()
}

fn export_stage_message(
    status: &DetailStatus,
    artifacts: &DetailArtifacts,
    pack_dir: &Path,
) -> String {
    let pdf = if artifacts.pdf_report_path.is_some() {
        "PDF present"
    } else {
        "PDF missing or unavailable"
    };
    let pack = if find_first_file_with_extension(pack_dir, "zip").is_some() {
        "pack zip present"
    } else {
        "pack zip not found"
    };
    let pdf_status = status
        .pdf_export_status
        .clone()
        .unwrap_or_else(|| "pdf status unknown".to_string());
    format!("{pdf}; {pack}; {pdf_status}.")
}

fn status_contains(status: Option<&str>, needle: &str) -> bool {
    status
        .map(|value| value.to_ascii_lowercase().contains(needle))
        .unwrap_or(false)
}

fn contains_any_ci(value: &str, needles: &[&str]) -> bool {
    let normalized = value.to_ascii_lowercase();
    needles.iter().any(|needle| normalized.contains(needle))
}

fn find_first_file_with_extension(dir: &Path, extension: &str) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.is_file() && path.extension().is_some_and(|ext| ext == extension))
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

fn json_pointer_f64(json: &Value, pointer: &str) -> Option<f64> {
    json.pointer(pointer).and_then(Value::as_f64)
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

fn first_f64(values: &[Option<f64>]) -> Option<f64> {
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
            load_run_detail,
            list_training_runs,
            load_quality_matrix,
            open_artifact,
            reveal_in_folder
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

    #[test]
    fn open_artifact_rejects_path_traversal() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        create_dir_all(&reports_root).expect("reports");

        let error = validate_artifact_path(&reports_root, "../../etc/passwd", ArtifactAction::Open)
            .expect_err("traversal should fail");

        assert!(error.contains("unsafe traversal"));
    }

    #[test]
    fn open_artifact_rejects_outside_reports() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        create_dir_all(&reports_root).expect("reports");
        let outside = temp_dir.path().join("outside.md");
        write(&outside, "outside").expect("outside");

        let error = validate_artifact_path(
            &reports_root,
            &path_to_string(&outside),
            ArtifactAction::Open,
        )
        .expect_err("outside reports should fail");

        assert!(error.contains("outside reports root"));
    }

    #[test]
    fn open_artifact_rejects_missing_file() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        create_dir_all(&reports_root).expect("reports");

        let error = validate_artifact_path(
            &reports_root,
            &path_to_string(
                &reports_root
                    .join("AAPL")
                    .join("runs")
                    .join("missing")
                    .join("report")
                    .join("x.md"),
            ),
            ArtifactAction::Open,
        )
        .expect_err("missing file should fail");

        assert!(error.contains("does not exist"));
    }

    #[test]
    fn open_artifact_accepts_report_markdown_under_reports() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        let report_path = reports_root
            .join("AAPL")
            .join("runs")
            .join("demo")
            .join("report")
            .join("AAPL_research_report.md");
        create_dir_all(report_path.parent().expect("parent")).expect("report dir");
        write(&report_path, "# AAPL").expect("report");

        let validated = validate_artifact_path(
            &reports_root,
            &path_to_string(&report_path),
            ArtifactAction::Open,
        )
        .expect("valid report");

        assert_eq!(
            validated.canonical_path,
            report_path.canonicalize().expect("canonical")
        );
    }

    #[test]
    fn reveal_in_folder_accepts_run_folder_under_reports() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        let run_path = reports_root.join("AAPL").join("runs").join("demo");
        create_dir_all(&run_path).expect("run dir");

        let validated = validate_artifact_path(
            &reports_root,
            &path_to_string(&run_path),
            ArtifactAction::Reveal,
        )
        .expect("valid reveal");

        assert_eq!(
            validated.canonical_path,
            run_path.canonicalize().expect("canonical")
        );
    }

    #[test]
    fn reveal_in_folder_rejects_outside_reports() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        create_dir_all(&reports_root).expect("reports");
        let outside = temp_dir.path().join("outside");
        create_dir_all(&outside).expect("outside");

        let error = validate_artifact_path(
            &reports_root,
            &path_to_string(&outside),
            ArtifactAction::Reveal,
        )
        .expect_err("outside reveal should fail");

        assert!(error.contains("outside reports root"));
    }

    #[test]
    fn artifact_commands_do_not_read_file_contents() {
        let temp_dir = TempDir::new().expect("temp dir");
        let reports_root = temp_dir.path().join("reports");
        let provider_path = reports_root
            .join("AAPL")
            .join("runs")
            .join("demo")
            .join("raw")
            .join("provider_payload.json");
        create_dir_all(provider_path.parent().expect("parent")).expect("raw dir");
        write(&provider_path, "SHOULD_NOT_BE_READ").expect("provider");

        let validated = validate_artifact_path(
            &reports_root,
            &path_to_string(&provider_path),
            ArtifactAction::Open,
        )
        .expect("valid provider payload");

        let serialized = serde_json::to_string(&ArtifactActionResult {
            ok: true,
            action: "open",
            path: path_to_string(&validated.canonical_path),
            message: "artifact open request sent".to_string(),
        })
        .expect("serialize");

        assert!(!serialized.contains("SHOULD_NOT_BE_READ"));
        assert!(serialized.contains("provider_payload.json"));
    }

    #[test]
    fn load_run_detail_builds_audit_trail() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "audit");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("audit")).expect("audit dir");
        create_dir_all(run_path.join("report")).expect("report dir");
        write(
            run_path.join("audit").join("validator_report.md"),
            "validator",
        )
        .expect("validator");
        write(
            run_path.join("report").join("AAPL_research_report.md"),
            "# AAPL",
        )
        .expect("report");

        let detail =
            load_run_detail_from_reports_root(&temp_dir.path().join("reports"), "AAPL", "audit")
                .expect("detail");
        let stages = detail
            .audit_trail
            .iter()
            .map(|stage| stage.stage.as_str())
            .collect::<Vec<_>>();

        assert_eq!(detail.audit_trail.len(), 9);
        assert!(stages.contains(&"provider_fetch"));
        assert!(stages.contains(&"ai_company_understanding"));
        assert!(stages.contains(&"validator_lint"));
    }

    #[test]
    fn audit_trail_marks_missing_optional_stage_unknown_or_warning() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "CAT", "missing_audit");
        write_required_detail_metadata(&run_path);

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "CAT",
            "missing_audit",
        )
        .expect("detail");
        let validator = detail
            .audit_trail
            .iter()
            .find(|stage| stage.stage == "validator_lint")
            .expect("validator stage");
        let export = detail
            .audit_trail
            .iter()
            .find(|stage| stage.stage == "pdf_pack_export")
            .expect("export stage");

        assert_eq!(validator.status, "warning");
        assert_eq!(export.status, "unknown");
    }

    #[test]
    fn audit_trail_marks_ai_source_external() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "JPM", "external_audit");
        write_required_detail_metadata(&run_path);

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "JPM",
            "external_audit",
        )
        .expect("detail");
        let company_stage = detail
            .audit_trail
            .iter()
            .find(|stage| stage.stage == "ai_company_understanding")
            .expect("company stage");

        assert_eq!(company_stage.status, "pass");
        assert_eq!(company_stage.source.as_deref(), Some("external_openai"));
    }

    #[test]
    fn audit_trail_marks_local_mock_warning() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "local_audit");
        write_required_detail_metadata(&run_path);
        write(
            run_path.join("metadata").join("ai_usage.json"),
            r#"{"source":"local_mock","external_ai_used":false,"local_mock_used":true,"cache_hits":0,"new_external_ai_calls":0}"#,
        )
        .expect("ai usage");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "AAPL",
            "local_audit",
        )
        .expect("detail");
        let ai_stages = detail
            .audit_trail
            .iter()
            .filter(|stage| stage.stage.starts_with("ai_"))
            .collect::<Vec<_>>();

        assert!(!ai_stages.is_empty());
        assert!(ai_stages.iter().all(|stage| stage.status == "warning"));
        assert!(ai_stages
            .iter()
            .all(|stage| stage.source.as_deref() == Some("local_mock")));
    }

    #[test]
    fn audit_trail_includes_artifact_paths_when_present() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "RKLB", "artifact_audit");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("audit")).expect("audit dir");
        create_dir_all(run_path.join("report")).expect("report dir");
        write(
            run_path.join("audit").join("validator_report.md"),
            "validator",
        )
        .expect("validator");
        write(
            run_path.join("report").join("RKLB_research_report.md"),
            "# RKLB",
        )
        .expect("report");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "RKLB",
            "artifact_audit",
        )
        .expect("detail");

        assert!(detail.audit_trail.iter().any(|stage| stage
            .artifact_path
            .as_deref()
            .is_some_and(|path| path.ends_with("provider_payload.json"))));
        assert!(detail.audit_trail.iter().any(|stage| stage
            .artifact_path
            .as_deref()
            .is_some_and(|path| path.ends_with("validator_report.md"))));
    }

    #[test]
    fn audit_trail_does_not_require_live_events() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "GOOGL", "static_audit");
        write_required_detail_metadata(&run_path);

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "GOOGL",
            "static_audit",
        )
        .expect("detail");
        let serialized = serde_json::to_string(&detail.audit_trail).expect("serialize");

        assert!(!serialized.contains("running"));
        assert!(!serialized.contains("event_stream"));
    }

    #[test]
    fn audit_trail_marks_provider_data_gap_warning() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "600519.SH", "gap_audit");
        write_required_detail_metadata(&run_path);
        write(
            run_path.join("metadata").join("provider_status.json"),
            r#"{"provider":"eastmoney_public","status":"WARNING","missing_fields":["dividend"],"market":"CN_A","currency":"CNY"}"#,
        )
        .expect("provider status");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "600519.SH",
            "gap_audit",
        )
        .expect("detail");
        let provider_stage = detail
            .audit_trail
            .iter()
            .find(|stage| stage.stage == "provider_fetch")
            .expect("provider stage");

        assert_eq!(provider_stage.status, "warning");
        assert!(provider_stage
            .message
            .as_deref()
            .is_some_and(|message| message.contains("Missing fields")));
    }

    #[test]
    fn audit_trail_does_not_parse_full_report_markdown() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "report_guard");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("report")).expect("report dir");
        write(
            run_path.join("report").join("AAPL_research_report.md"),
            "REPORT_BODY_SHOULD_NOT_RETURN",
        )
        .expect("report");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "AAPL",
            "report_guard",
        )
        .expect("detail");
        let serialized = serde_json::to_string(&detail).expect("serialize");

        assert!(!serialized.contains("REPORT_BODY_SHOULD_NOT_RETURN"));
        assert!(serialized.contains("AAPL_research_report.md"));
    }

    #[test]
    fn load_run_detail_reads_chart_manifest() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "charts");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("charts")).expect("charts dir");
        write(run_path.join("charts").join("price.png"), "png").expect("chart image");
        write(
            run_path.join("charts").join("chart_manifest.json"),
            r#"{"charts":[{"title":"Price vs Benchmark","image_path":"price.png","source":"locked price history","status":"PASS","why_selected":"Checks price path","what_to_look_at":"relative return","what_it_means":"historical price path","what_not_to_overread":"not valuation proof","next_check":"compare drawdown"}]}"#,
        )
        .expect("chart manifest");

        let detail =
            load_run_detail_from_reports_root(&temp_dir.path().join("reports"), "AAPL", "charts")
                .expect("detail");

        assert_eq!(detail.charts.len(), 1);
        assert_eq!(detail.charts[0].title, "Price vs Benchmark");
        assert!(detail.charts[0].image_exists);
        assert_eq!(
            detail.charts[0].source.as_deref(),
            Some("locked price history")
        );
        assert_eq!(
            detail.charts[0].what_not_to_overread.as_deref(),
            Some("not valuation proof")
        );
    }

    #[test]
    fn chart_grid_handles_missing_manifest() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "CAT", "no_manifest");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("charts")).expect("charts dir");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "CAT",
            "no_manifest",
        )
        .expect("detail");

        assert!(detail.charts.is_empty());
    }

    #[test]
    fn chart_grid_marks_missing_image() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "RKLB", "missing_chart");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("charts")).expect("charts dir");
        write(
            run_path.join("charts").join("chart_manifest.json"),
            r#"[{"title":"Money Flow","image_path":"missing.png","source":"cash flow"}]"#,
        )
        .expect("chart manifest");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "RKLB",
            "missing_chart",
        )
        .expect("detail");

        assert_eq!(detail.charts.len(), 1);
        assert!(!detail.charts[0].image_exists);
        assert_eq!(detail.charts[0].status.as_deref(), Some("WARNING"));
    }

    #[test]
    fn chart_metadata_includes_source_or_warning() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "GOOGL", "chart_warning");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("charts")).expect("charts dir");
        write(
            run_path.join("charts").join("chart_manifest.json"),
            r#"[{"title":"Missing Source","image_path":"missing.png"}]"#,
        )
        .expect("chart manifest");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "GOOGL",
            "chart_warning",
        )
        .expect("detail");

        assert!(detail.charts[0].source.is_none());
        assert_eq!(detail.charts[0].status.as_deref(), Some("WARNING"));
    }

    #[test]
    fn chart_artifact_path_stays_under_reports() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = create_detail_fixture(temp_dir.path(), "AAPL", "chart_escape");
        write_required_detail_metadata(&run_path);
        create_dir_all(run_path.join("charts")).expect("charts dir");
        write(
            run_path.join("charts").join("chart_manifest.json"),
            r#"[{"title":"Unsafe","image_path":"../outside.png","source":"bad"}]"#,
        )
        .expect("chart manifest");

        let detail = load_run_detail_from_reports_root(
            &temp_dir.path().join("reports"),
            "AAPL",
            "chart_escape",
        )
        .expect("detail");

        assert_eq!(detail.charts.len(), 1);
        assert!(detail.charts[0].image_path.is_none());
        assert!(!detail.charts[0].image_exists);
        assert_eq!(detail.charts[0].status.as_deref(), Some("WARNING"));
    }

    #[test]
    fn list_training_runs_returns_empty_when_missing() {
        let temp_dir = TempDir::new().expect("temp dir");
        let runs =
            list_training_runs_from_root(&temp_dir.path().join("reports").join("training_runs"))
                .expect("training runs");

        assert!(runs.is_empty());
    }

    #[test]
    fn list_training_runs_detects_quality_matrix() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir
            .path()
            .join("reports")
            .join("training_runs")
            .join("regression_01");
        create_dir_all(&run_path).expect("run dir");
        write(run_path.join("quality_matrix.json"), "[]").expect("matrix");
        write(run_path.join("issue_distribution.md"), "# Issues").expect("issues");
        write(run_path.join("training_cases_generated.jsonl"), "{}\n").expect("cases");

        let runs =
            list_training_runs_from_root(&temp_dir.path().join("reports").join("training_runs"))
                .expect("training runs");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run_id, "regression_01");
        assert!(runs[0].has_quality_matrix);
        assert!(runs[0].has_issue_distribution);
        assert!(runs[0].has_training_cases);
    }

    #[test]
    fn load_quality_matrix_reads_json() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir.path().join("training_runs").join("json_run");
        create_dir_all(&run_path).expect("run dir");
        write(
            run_path.join("quality_matrix.json"),
            r#"{"rows":[{"ticker":"AAPL","market":"US","company_name":"Apple","status":"PASS","quality_score":88,"grade":"GOOD","issue_types":["weak_money_flow"],"hard_failures":[],"ai_source":"external_openai","provider_status":"PASS"}]}"#,
        )
        .expect("matrix");

        let matrix =
            load_quality_matrix_from_root(&temp_dir.path().join("training_runs"), "json_run")
                .expect("matrix");

        assert_eq!(matrix.rows.len(), 1);
        assert_eq!(matrix.rows[0].ticker, "AAPL");
        assert_eq!(matrix.rows[0].quality_score, Some(88.0));
        assert_eq!(matrix.issue_distribution[0].issue_type, "weak_money_flow");
    }

    #[test]
    fn load_quality_matrix_reads_csv_fallback() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir.path().join("training_runs").join("csv_run");
        create_dir_all(&run_path).expect("run dir");
        write(
            run_path.join("quality_matrix.csv"),
            "ticker,market,company_name,status,quality_score,grade,issue_types,hard_failures,ai_source,provider_status\nRKLB,US,Rocket Lab,WARNING,72,ACCEPTABLE,weak_money_flow|generic_chart_explanation,,local_mock,PASS\n",
        )
        .expect("matrix");

        let matrix =
            load_quality_matrix_from_root(&temp_dir.path().join("training_runs"), "csv_run")
                .expect("matrix");

        assert_eq!(matrix.rows.len(), 1);
        assert_eq!(matrix.rows[0].ticker, "RKLB");
        assert_eq!(matrix.rows[0].issue_types.len(), 2);
    }

    #[test]
    fn load_quality_matrix_handles_missing_columns() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir.path().join("training_runs").join("minimal_csv");
        create_dir_all(&run_path).expect("run dir");
        write(run_path.join("quality_matrix.csv"), "ticker\nCAT\n").expect("matrix");

        let matrix =
            load_quality_matrix_from_root(&temp_dir.path().join("training_runs"), "minimal_csv")
                .expect("matrix");

        assert_eq!(matrix.rows[0].ticker, "CAT");
        assert_eq!(matrix.rows[0].quality_score, None);
        assert!(matrix.warnings.is_empty());
    }

    #[test]
    fn load_quality_matrix_rejects_path_traversal() {
        let temp_dir = TempDir::new().expect("temp dir");
        let error = load_quality_matrix_from_root(&temp_dir.path().join("training_runs"), "../bad")
            .expect_err("path traversal should fail");

        assert!(error.contains("unsafe path segment"));
    }

    #[test]
    fn quality_matrix_path_stays_under_reports() {
        let temp_dir = TempDir::new().expect("temp dir");
        let training_root = temp_dir.path().join("training_runs");
        let error = load_quality_matrix_from_root(&training_root, "bad/../escape")
            .expect_err("unsafe segment should fail");

        assert!(error.contains("unsafe path segment"));
    }

    #[test]
    fn issue_distribution_parsed_when_available() {
        let temp_dir = TempDir::new().expect("temp dir");
        let run_path = temp_dir.path().join("training_runs").join("issues");
        create_dir_all(&run_path).expect("run dir");
        write(run_path.join("quality_matrix.json"), "[]").expect("matrix");
        write(
            run_path.join("issue_distribution.json"),
            r#"[{"issue_type":"wrong_framework","count":2},{"issue_type":"weak_money_flow","count":3}]"#,
        )
        .expect("issues");

        let matrix =
            load_quality_matrix_from_root(&temp_dir.path().join("training_runs"), "issues")
                .expect("matrix");

        assert_eq!(matrix.issue_distribution.len(), 2);
        assert_eq!(matrix.issue_distribution[0].count, 2);
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
