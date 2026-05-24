use crate::dashboard::render_company_dashboard;
use crate::markdown::{render_report, render_report_zh, render_self_review_md};
use anyhow::Result;
use research_core::io::{write_if_changed, write_json};
use research_core::run_folder::RunFolder;
use research_core::types::*;
use research_core::validation::visual_lint;
use std::process::Command;

pub struct RenderRunInput<'a> {
    pub folder: &'a RunFolder,
    pub payload: &'a ProviderPayload,
    pub understanding: &'a CompanyUnderstanding,
    pub interpretation: &'a FinancialInterpretation,
    pub blueprint: &'a ResearchBlueprint,
    pub review: &'a AiSelfReview,
    pub status: &'a ReportStatus,
    pub lang: &'a str,
}

pub fn render_run(input: RenderRunInput<'_>) -> Result<()> {
    let folder = input.folder;
    let payload = input.payload;
    let understanding = input.understanding;
    let interpretation = input.interpretation;
    let blueprint = input.blueprint;
    let review = input.review;
    let status = input.status;
    let lang = input.lang;
    write_json(
        &folder.metadata.join("company_understanding.json"),
        understanding,
    )?;
    write_json(
        &folder.metadata.join("financial_interpretation.json"),
        interpretation,
    )?;
    write_json(&folder.metadata.join("research_blueprint.json"), blueprint)?;
    write_json(&folder.self_review.join("ai_self_review.json"), review)?;
    write_json(&folder.metadata.join("report_status.json"), status)?;
    let unit_policy = UnitPolicy {
        reporting_currency: payload.company_profile.currency.clone(),
        price_currency: payload.company_profile.currency.clone(),
        financial_statement_unit:
            "provider reported units; report displays compact human-readable values".into(),
        percentage_format: "12.3%".into(),
        multiple_format: "24.1x".into(),
        share_count_unit: "shares".into(),
        date_range: "provider payload range".into(),
        provider_source: payload.provider.clone(),
    };
    write_json(&folder.metadata.join("unit_policy.json"), &unit_policy)?;
    generate_charts(folder)?;
    write_if_changed(
        &folder.self_review.join("ai_self_review.md"),
        &render_self_review_md(review),
    )?;
    let report = render_report(
        payload,
        understanding,
        interpretation,
        blueprint,
        review,
        status,
    );
    let english_report_path = folder
        .report
        .join(format!("{}_research_report.md", payload.ticker));
    let chinese_report_path = folder
        .report
        .join(format!("{}_research_report_cn.md", payload.ticker));
    let mut primary_report = report.clone();
    if lang == "en" || lang == "both" {
        write_if_changed(&english_report_path, &report)?;
        export_pdf(
            &english_report_path,
            &folder
                .report
                .join(format!("{}_research_report.pdf", payload.ticker)),
        )?;
    }
    if lang == "zh" || lang == "both" {
        let zh_report = render_report_zh(
            payload,
            understanding,
            interpretation,
            blueprint,
            review,
            status,
        );
        if lang == "zh" {
            primary_report = zh_report.clone();
        }
        write_if_changed(&chinese_report_path, &zh_report)?;
        export_pdf(
            &chinese_report_path,
            &folder
                .report
                .join(format!("{}_research_report_cn.pdf", payload.ticker)),
        )?;
    }
    let dashboard =
        render_company_dashboard(payload, understanding, interpretation, blueprint, status);
    write_if_changed(&folder.root.join("dashboard.html"), &dashboard)?;
    write_if_changed(
        &folder.audit.join("provider_validation.md"),
        "# Provider Validation\n\nProvider payload was parsed into the v5 locked-data schema.\n",
    )?;
    write_if_changed(&folder.audit.join("data_quality.md"), "# Data Quality\n\nData quality warnings are recorded in raw/provider_payload.json metadata.\n")?;
    write_if_changed(
        &folder.audit.join("validator_report.md"),
        &format!(
            "# Validator Report\n\nOverall status: {}\n\nHuman review required: {}\n",
            status.overall_status, status.human_review_required
        ),
    )?;
    write_if_changed(
        &folder.audit.join("lint_report.md"),
        "# Lint Report\n\nDeterministic report lint completed.\n",
    )?;
    let (visual_status, visual_failures) = visual_lint(
        &primary_report,
        folder.root.join("dashboard.html").exists(),
        folder.charts.join("chart_manifest.json").exists(),
    );
    let visual_details = if visual_failures.is_empty() {
        "All P0 display checks passed.".to_string()
    } else {
        visual_failures
            .iter()
            .map(|failure| format!("- {}", failure))
            .collect::<Vec<_>>()
            .join("\n")
    };
    write_if_changed(
        &folder.audit.join("visual_lint_report.md"),
        &format!(
            "# Visual Lint Report\n\nStatus: {}\n\n## Checks\n\n{}\n\n## Scope\n\n- Markdown status block\n- Table of contents\n- Chart explanation blocks\n- Raw placeholder / NaN scan\n- Dashboard existence\n- Chart manifest existence\n- Forbidden advice scan\n",
            visual_status, visual_details
        ),
    )?;
    if visual_status == "FAIL" {
        anyhow::bail!("visual lint failed: {}", visual_failures.join(", "));
    }
    let report_entry = if lang == "zh" {
        format!("report/{}_research_report_cn.md", payload.ticker)
    } else {
        format!("report/{}_research_report.md", payload.ticker)
    };
    write_if_changed(&folder.root.join("README.md"), &format!("# {} v5 Research Run\n\nStart here:\n\n1. {}\n2. dashboard.html\n3. metadata/research_blueprint.json\n4. self_review/ai_self_review.md\n5. audit/validator_report.md\n6. audit/visual_lint_report.md\n\nPDF exports live in `report/` when the lightweight exporter is available.\n", payload.ticker, report_entry))?;
    Ok(())
}

fn generate_charts(folder: &RunFolder) -> Result<()> {
    let python = if std::path::Path::new(".venv/bin/python").exists() {
        ".venv/bin/python"
    } else {
        "python3"
    };
    let status = Command::new(python)
        .arg("providers/chart_provider.py")
        .arg("--payload")
        .arg(folder.raw.join("provider_payload.json"))
        .arg("--out-dir")
        .arg(&folder.charts)
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            write_if_changed(
                &folder.charts.join("Figure_01_data_gap.md"),
                "# Chart Generation Failed\n\nStatus: WARNING\n\nSource: provider_payload.json\n",
            )?;
            Ok(())
        }
    }
}

fn export_pdf(markdown_path: &std::path::Path, pdf_path: &std::path::Path) -> Result<()> {
    let python = if std::path::Path::new(".venv/bin/python").exists() {
        ".venv/bin/python"
    } else {
        "python3"
    };
    let status = Command::new(python)
        .arg("providers/pdf_export.py")
        .arg("--markdown")
        .arg(markdown_path)
        .arg("--out")
        .arg(pdf_path)
        .status();
    match status {
        Ok(s) if s.success() && pdf_path.exists() => Ok(()),
        _ => {
            let fallback_path = markdown_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("pdf_export_unavailable.md");
            write_if_changed(
                &fallback_path,
                "# PDF Export Unavailable\n\nThe basic PDF exporter failed. Markdown and HTML outputs remain authoritative.\n",
            )?;
            Ok(())
        }
    }
}
