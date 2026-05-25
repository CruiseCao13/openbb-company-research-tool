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
    let report_dir = run_path.join("report");
    if report_dir
        .join(format!("{ticker}_research_report.md"))
        .is_file()
    {
        return true;
    }

    fs::read_dir(report_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
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

fn first_string(values: &[Option<String>]) -> Option<String> {
    values.iter().flatten().next().cloned()
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
            list_runs
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
}
