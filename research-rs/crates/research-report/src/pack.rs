use anyhow::Result;
use chrono::Local;
use research_core::io::write_json;
use research_core::run_folder::RunFolder;
use research_core::types::{PackManifest, ReportStatus};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

fn add_dir(zip: &mut zip::ZipWriter<File>, base: &Path, path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            add_dir(zip, base, &p)?;
        } else {
            let mut f = File::open(&p)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let name = p.strip_prefix(base)?.to_string_lossy().replace('\\', "/");
            zip.start_file(name, SimpleFileOptions::default())?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}

pub fn pack_run(folder: &RunFolder, ticker: &str) -> Result<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_files(&folder.root, &folder.root, &mut files)?;
    let status_path = folder.metadata.join("report_status.json");
    let status: Option<ReportStatus> = fs::read_to_string(&status_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok());
    let manifest = PackManifest {
        ticker: ticker.to_string(),
        run_id: folder
            .root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        generated_at: Local::now().to_rfc3339(),
        files: files.clone(),
        report_status: status
            .as_ref()
            .map(|s| s.overall_status.clone())
            .unwrap_or_else(|| "UNKNOWN".into()),
        ai_mode: status
            .as_ref()
            .map(|s| s.ai_mode.clone())
            .unwrap_or_else(|| "compact".into()),
        provider: status
            .as_ref()
            .map(|s| s.provider_status.clone())
            .unwrap_or_else(|| "UNKNOWN".into()),
        has_dashboard: folder.root.join("dashboard.html").exists(),
        has_charts: folder.charts.exists(),
        has_self_review: folder.self_review.join("ai_self_review.md").exists(),
    };
    write_json(&folder.pack.join("pack_manifest.json"), &manifest)?;
    let out = folder.pack.join(format!("{}_research_pack.zip", ticker));
    let file = File::create(&out)?;
    let mut zip = zip::ZipWriter::new(file);
    add_dir(&mut zip, &folder.root, &folder.root)?;
    zip.finish()?;
    Ok(out)
}

fn collect_files(base: &Path, path: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            if p.file_name().map(|x| x == "pack").unwrap_or(false) {
                continue;
            }
            collect_files(base, &p, files)?;
        } else {
            files.push(p.strip_prefix(base)?.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}
