use serde::Serialize;
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
        .invoke_handler(tauri::generate_handler![ping_studio, get_app_info])
        .run(tauri::generate_context!())
        .expect("failed to run v6 Tauri Research Studio");
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
