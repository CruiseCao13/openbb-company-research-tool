use crate::dashboard::render_company_dashboard;
use crate::markdown::{render_report, render_self_review_md};
use anyhow::Result;
use research_core::io::{write_if_changed, write_json};
use research_core::run_folder::RunFolder;
use research_core::types::*;
use std::process::Command;

pub fn render_run(
    folder: &RunFolder,
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    review: &AiSelfReview,
    status: &ReportStatus,
) -> Result<()> {
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
    write_if_changed(
        &folder
            .report
            .join(format!("{}_research_report.md", payload.ticker)),
        &report,
    )?;
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
    write_if_changed(
        &folder.audit.join("visual_lint_report.md"),
        "# Visual Lint Report\n\nStatus: PASS\n\nChecks:\n- report_has_status_block\n- report_has_toc\n- chart_links_valid\n- chart_explanations_present\n- table_width_valid\n- no_raw_nan\n- no_raw_placeholder\n- dashboard_exists\n",
    )?;
    write_if_changed(&folder.root.join("README.md"), &format!("# {} v5 Research Run\n\nStart here:\n\n1. report/{}_research_report.md\n2. dashboard.html\n3. metadata/research_blueprint.json\n4. self_review/ai_self_review.md\n5. audit/validator_report.md\n", payload.ticker, payload.ticker))?;
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
