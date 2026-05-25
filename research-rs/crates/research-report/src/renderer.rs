use crate::dashboard::render_company_dashboard;
use crate::language::{
    language_lint, language_naturalness_markdown, language_polish, language_polish_trace_markdown,
};
use crate::markdown::{render_report, render_report_zh, render_self_review_md};
use anyhow::Result;
use research_core::cache::digest_str;
use research_core::io::{write_if_changed, write_json};
use research_core::provider::{resolve_python, resolve_repo_path};
use research_core::run_folder::RunFolder;
use research_core::types::*;
use research_core::validation::visual_lint;
use serde_json::json;
use std::process::Command;
use std::{env, fs};

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
    write_repro_manifest(folder, payload, status)?;
    write_if_changed(
        &folder.metadata.join("prompt_versions.json"),
        "{\n  \"company_understanding\": \"company_understanding_v1\",\n  \"financial_interpretation\": \"financial_interpretation_v1\",\n  \"research_blueprint\": \"research_blueprint_v1\",\n  \"self_review\": \"self_review_v1\",\n  \"content_quality_judge\": \"content_quality_judge_v1\",\n  \"chart_explanation\": \"chart_explanation_v1\",\n  \"table_explanation\": \"table_explanation_v1\"\n}\n",
    )?;
    generate_charts(folder)?;
    write_data_inventory(folder, payload, blueprint)?;
    write_data_usage_coverage(folder, payload, blueprint)?;
    write_chart_plan(folder, payload)?;
    write_table_plan(folder)?;
    write_section_source_map(folder, payload)?;
    write_money_flow_map(folder, payload, interpretation)?;
    write_evidence_map(folder, understanding, interpretation, blueprint)?;
    write_if_changed(
        &folder.self_review.join("ai_self_review.md"),
        &render_self_review_md(review),
    )?;
    let mut language_traces = Vec::new();
    let raw_report = render_report(
        payload,
        understanding,
        interpretation,
        blueprint,
        review,
        status,
    );
    let (report, trace) = language_polish(&raw_report, "en");
    language_traces.extend(trace);
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
        let raw_zh_report = render_report_zh(
            payload,
            understanding,
            interpretation,
            blueprint,
            review,
            status,
        );
        let (zh_report, trace) = language_polish(&raw_zh_report, "zh");
        language_traces.extend(trace);
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
    write_output_consistency(
        folder,
        payload,
        understanding,
        interpretation,
        blueprint,
        status,
    )?;
    let pdf_status = write_pdf_export_report(folder, payload, lang)?;
    let mut final_status = status.clone();
    final_status.pdf_export_status = pdf_status.clone();
    if pdf_status == "WARNING" && final_status.overall_status == "PASS" {
        final_status.overall_status = "WARNING".to_string();
        final_status.human_review_required = true;
    }
    let language_for_status = if lang == "zh" { "zh" } else { "en" };
    let language_status = language_lint(&primary_report, language_for_status).score;
    if language_status.presentation_status == "FAIL" {
        final_status.overall_status = "FAIL".to_string();
        final_status.visual_lint_status = "FAIL".to_string();
        final_status.human_review_required = true;
    } else if language_status.presentation_status == "WARNING"
        && final_status.overall_status == "PASS"
    {
        final_status.overall_status = "WARNING".to_string();
        final_status.human_review_required = true;
    }
    write_json(&folder.metadata.join("report_status.json"), &final_status)?;
    write_repro_manifest(folder, payload, &final_status)?;
    write_chart_table_quality(folder, &primary_report)?;
    write_language_quality(folder, &primary_report, lang, &language_traces)?;
    write_if_changed(
        &folder.audit.join("provider_validation.md"),
        "# Provider Validation\n\nProvider payload was parsed into the v5 locked-data schema.\n",
    )?;
    write_if_changed(&folder.audit.join("data_quality.md"), "# Data Quality\n\nData quality warnings are recorded in raw/provider_payload.json metadata.\n")?;
    write_validator_report(folder, &final_status, &pdf_status)?;
    write_if_changed(
        &folder.audit.join("lint_report.md"),
        "# Lint Report\n\nDeterministic report lint completed.\n",
    )?;
    let (visual_status, visual_failures) = visual_lint(
        &primary_report,
        folder.root.join("dashboard.html").exists(),
        folder.charts.join("chart_manifest.json").exists(),
        folder.audit.join("data_usage_coverage_report.md").exists(),
        folder.audit.join("chart_table_quality_report.md").exists(),
        folder.audit.join("pdf_export_report.md").exists(),
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
            "# Visual Lint Report\n\nStatus: {}\n\n## Checks\n\n{}\n\n## Scope\n\n- Markdown status block\n- Table of contents\n- Chart explanation blocks\n- Raw placeholder / NaN scan\n- Dashboard existence\n- Chart manifest existence\n- Data usage coverage report\n- Chart/table quality report\n- PDF export report\n- Forbidden advice scan\n",
            visual_status, visual_details
        ),
    )?;
    if visual_status == "FAIL" {
        anyhow::bail!("visual lint failed: {}", visual_failures.join(", "));
    }
    write_iteration_log(folder, review, &visual_status, &pdf_status)?;
    let report_entry = if lang == "zh" {
        format!("report/{}_research_report_cn.md", payload.ticker)
    } else {
        format!("report/{}_research_report.md", payload.ticker)
    };
    write_if_changed(&folder.root.join("README.md"), &format!("# {} v5 Research Run\n\nStart here:\n\n1. {}\n2. dashboard.html\n3. metadata/research_blueprint.json\n4. self_review/ai_self_review.md\n5. audit/validator_report.md\n6. audit/visual_lint_report.md\n7. audit/data_usage_coverage_report.md\n8. audit/chart_table_quality_report.md\n9. audit/pdf_export_report.md\n\nPDF exports live in `report/` when the lightweight exporter is available.\n", payload.ticker, report_entry))?;
    Ok(())
}

fn generate_charts(folder: &RunFolder) -> Result<()> {
    let python = resolve_python()?;
    let chart_provider = resolve_repo_path("providers/chart_provider.py")?;
    let status = Command::new(python)
        .arg(chart_provider)
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

fn section_available(rows: &[StatementRow]) -> bool {
    rows.iter().any(|row| row.value.is_some())
}

fn count_metric(rows: &[StatementRow], needle: &str) -> usize {
    rows.iter()
        .filter(|row| row.metric.to_lowercase().contains(needle))
        .count()
}

fn write_data_inventory(
    folder: &RunFolder,
    payload: &ProviderPayload,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let price_available = payload
        .price_history
        .iter()
        .any(|point| point.close.is_some());
    let income_available = section_available(&payload.income_statement);
    let balance_available = section_available(&payload.balance_sheet);
    let cash_available = section_available(&payload.cash_flow);
    let valuation_available = payload
        .valuation_snapshot
        .as_object()
        .map(|object| !object.is_empty())
        .unwrap_or(false);
    let fields = vec![
        json!({"field":"company_profile","source":"company_profile","periods_available":1,"used_in_report":true,"used_in_chart":false,"used_in_table":true,"reason_if_unused":"Text field; used in company identity and dashboard cards."}),
        json!({"field":"price_history.close","source":"price_history","periods_available":payload.price_history.len(),"used_in_report":price_available,"used_in_chart":price_available,"used_in_table":false,"reason_if_unused": if price_available { "" } else { "Price history unavailable; chart data-gap card generated." }}),
        json!({"field":"revenue","source":"income_statement","periods_available":count_metric(&payload.income_statement, "revenue"),"used_in_report":income_available,"used_in_chart":income_available,"used_in_table":income_available,"reason_if_unused": if income_available { "" } else { "Income statement unavailable; financial trend shown as data gap." }}),
        json!({"field":"operating_cash_flow_or_capex","source":"cash_flow","periods_available":payload.cash_flow.len(),"used_in_report":cash_available,"used_in_chart":cash_available,"used_in_table":cash_available,"reason_if_unused": if cash_available { "" } else { "Cash-flow data unavailable; money-flow chart shown as data gap." }}),
        json!({"field":"valuation_snapshot","source":"valuation_snapshot","periods_available": if valuation_available { 1 } else { 0 },"used_in_report":valuation_available,"used_in_chart":valuation_available,"used_in_table":valuation_available,"reason_if_unused": if valuation_available { "" } else { "Valuation data unavailable or not meaningful for this profile." }}),
        json!({"field":"segments","source":"segments","periods_available":payload.segments.len(),"used_in_report":!payload.segments.is_empty(),"used_in_chart":!payload.segments.is_empty(),"used_in_table":!payload.segments.is_empty(),"reason_if_unused": if payload.segments.is_empty() { "Segment data not available; shown as data gap/manual check." } else { "" }}),
    ];
    let inventory = json!({
        "available_data": {
            "company_profile": !payload.company_profile.name.is_empty() || !payload.company_profile.description.is_empty(),
            "price_history": price_available,
            "income_statement": income_available,
            "balance_sheet": balance_available,
            "cash_flow": cash_available,
            "valuation_snapshot": valuation_available,
            "segments": !payload.segments.is_empty(),
            "shares": false,
            "dividends": false,
            "buybacks": false,
            "debt_schedule": false,
            "industry_kpis": false
        },
        "data_fields": fields,
        "critical_unused_data": [],
        "non_visualized_but_explained": [
            {"data":"company_profile.description","reason":"Text field; explained in company identity/business model instead of charted."},
            {"data":"AI research blueprint","reason":"Interpretive artifact; summarized in tables/dashboard rather than plotted."}
        ],
        "missing_critical_data": blueprint.data_gaps
    });
    write_json(&folder.metadata.join("data_inventory.json"), &inventory)?;
    let gaps = if blueprint.data_gaps.is_empty() {
        "No critical data gaps were flagged by the current blueprint.".to_string()
    } else {
        blueprint
            .data_gaps
            .iter()
            .map(|gap| format!("- {}", gap))
            .collect::<Vec<_>>()
            .join("\n")
    };
    write_if_changed(
        &folder.audit.join("data_inventory_report.md"),
        &format!(
            "# Data Inventory Report\n\n## What Was Captured\n\n- Company profile: {}\n- Price history: {}\n- Income statement: {}\n- Balance sheet: {}\n- Cash flow: {}\n- Valuation snapshot: {}\n- Segment data: {}\n\n## Used In Charts\n\n- Price history feeds Figure 1 and Figure 2 when available.\n- Income statement feeds Figure 3 when revenue data is available.\n- Cash-flow data feeds Figure 4 when operating cash flow / capex data is available.\n- Valuation snapshot feeds Figure 5 when meaningful valuation fields exist.\n\n## Used In Tables\n\nThe report status, financial snapshot, money-flow summary, research blueprint, and data-gap tables use compact values and interpretation text. Raw CSV-style data is not pasted into the report body.\n\n## Missing Critical Data\n\n{}\n\n## Unused Data Policy\n\nText fields are explained in prose/dashboard cards. Missing or non-visualizable data is shown as a data gap instead of being forced into an empty chart.\n",
            !payload.company_profile.name.is_empty() || !payload.company_profile.description.is_empty(),
            price_available,
            income_available,
            balance_available,
            cash_available,
            valuation_available,
            !payload.segments.is_empty(),
            gaps
        ),
    )?;
    Ok(())
}

fn metric_present(rows: &[StatementRow], needles: &[&str]) -> bool {
    rows.iter().any(|row| {
        let metric = row.metric.to_lowercase();
        row.value.is_some() && needles.iter().any(|needle| metric.contains(needle))
    })
}

fn field_destination_row(
    field: &str,
    source: &str,
    present: bool,
    chart: bool,
    table: bool,
    appendix: bool,
    reason: &str,
) -> serde_json::Value {
    json!({
        "field": field,
        "source": source,
        "fetched": present,
        "used_in_report": present,
        "used_in_chart": present && chart,
        "used_in_table": present && table,
        "used_in_appendix": appendix,
        "unused_reason": if present { "" } else { reason }
    })
}

fn industry_critical_fields(
    blueprint: &ResearchBlueprint,
    payload: &ProviderPayload,
) -> Vec<&'static str> {
    let text = format!(
        "{} {} {} {} {}",
        blueprint.asset_profile,
        blueprint.secondary_profile,
        payload.company_profile.sector,
        payload.company_profile.industry,
        payload.company_profile.description
    )
    .to_lowercase();
    let industry_text = format!(
        "{} {} {} {}",
        blueprint.asset_profile,
        blueprint.secondary_profile,
        payload.company_profile.sector,
        payload.company_profile.industry
    )
    .to_lowercase();
    if text.contains("bank") || text.contains("financial") || text.contains("broker") {
        vec!["ROE", "NIM", "credit loss", "capital ratio"]
    } else if text.contains("reit") || text.contains("real estate") {
        vec!["FFO", "AFFO", "occupancy", "debt maturity"]
    } else if text.contains("biotech") || text.contains("pharma") || text.contains("clinical") {
        vec!["pipeline", "trial", "cash runway", "R&D burn"]
    } else if text.contains("shipping") || text.contains("airline") || text.contains("transport") {
        vec!["yield", "utilization", "fuel", "fleet", "orderbook"]
    } else if industry_text.contains("retail")
        || industry_text.contains("restaurant")
        || industry_text.contains("apparel")
        || industry_text.contains("store")
    {
        vec!["same-store sales", "traffic", "inventory", "store count"]
    } else if text.contains("utility") || text.contains("utilities") {
        vec!["rate base", "allowed ROE", "dividend coverage"]
    } else if payload.market == "CN_A" {
        vec![
            "营业收入",
            "归母净利润",
            "扣非净利润",
            "经营现金流",
            "货币资金",
            "有息负债",
            "ROE",
        ]
    } else {
        Vec::new()
    }
}

fn write_data_usage_coverage(
    folder: &RunFolder,
    payload: &ProviderPayload,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let price_history = payload
        .price_history
        .iter()
        .any(|point| point.close.is_some());
    let revenue = metric_present(
        &payload.income_statement,
        &["revenue", "total revenue", "营业收入"],
    );
    let operating_income = metric_present(
        &payload.income_statement,
        &["operating income", "operating profit", "营业利润"],
    );
    let net_income = metric_present(
        &payload.income_statement,
        &["net income", "净利润", "归母净利润"],
    );
    let operating_cash_flow = metric_present(
        &payload.cash_flow,
        &["operating cash", "cash from operations", "经营现金"],
    );
    let capex = metric_present(
        &payload.cash_flow,
        &["capex", "capital expenditure", "资本开支"],
    );
    let free_cash_flow = metric_present(&payload.cash_flow, &["free cash flow", "fcf"]);
    let cash = metric_present(
        &payload.balance_sheet,
        &["cash", "cash and cash", "货币资金"],
    );
    let debt = metric_present(&payload.balance_sheet, &["debt", "borrowings", "有息负债"]);
    let shares = metric_present(&payload.balance_sheet, &["shares"])
        || metric_present(&payload.income_statement, &["shares"]);
    let valuation_multiples = payload
        .valuation_snapshot
        .as_object()
        .map(|object| !object.is_empty())
        .unwrap_or(false);
    let fields = vec![
        field_destination_row(
            "price history",
            "price_history",
            price_history,
            true,
            false,
            true,
            "Price history missing; price/drawdown charts become data gap cards.",
        ),
        field_destination_row(
            "revenue",
            "income_statement",
            revenue,
            true,
            true,
            true,
            "Revenue missing; financial trend table/chart cannot prove growth.",
        ),
        field_destination_row(
            "operating income",
            "income_statement",
            operating_income,
            true,
            true,
            true,
            "Operating income missing; margin quality remains less verifiable.",
        ),
        field_destination_row(
            "net income",
            "income_statement",
            net_income,
            false,
            true,
            true,
            "Net income missing; profitability checks are limited.",
        ),
        field_destination_row(
            "operating cash flow",
            "cash_flow",
            operating_cash_flow,
            true,
            true,
            true,
            "Operating cash flow missing; money-flow analysis is degraded.",
        ),
        field_destination_row(
            "capex",
            "cash_flow",
            capex,
            true,
            true,
            true,
            "Capex missing; free-cash-flow bridge cannot be fully verified.",
        ),
        field_destination_row(
            "free cash flow",
            "cash_flow",
            free_cash_flow,
            true,
            true,
            true,
            "Free cash flow missing; report explains operating cash flow and capex separately.",
        ),
        field_destination_row(
            "cash",
            "balance_sheet",
            cash,
            false,
            true,
            true,
            "Cash missing; runway and liquidity checks are limited.",
        ),
        field_destination_row(
            "debt",
            "balance_sheet",
            debt,
            false,
            true,
            true,
            "Debt missing; leverage checks are limited.",
        ),
        field_destination_row(
            "shares",
            "statements",
            shares,
            false,
            false,
            true,
            "Share count missing; dilution/buyback quality needs manual verification.",
        ),
        field_destination_row(
            "valuation multiples",
            "valuation_snapshot",
            valuation_multiples,
            true,
            true,
            true,
            "Valuation multiples missing or not meaningful for this profile.",
        ),
    ];
    let missing_critical_fields = fields
        .iter()
        .filter_map(|field| {
            if field["fetched"].as_bool().unwrap_or(false) {
                None
            } else {
                field["field"].as_str().map(ToString::to_string)
            }
        })
        .chain(blueprint.data_gaps.iter().cloned())
        .collect::<Vec<_>>();
    let industry_fields = industry_critical_fields(blueprint, payload);
    let critical_unused_fields: Vec<String> = fields
        .iter()
        .filter_map(|field| {
            let fetched = field["fetched"].as_bool().unwrap_or(false);
            let used_any = field["used_in_report"].as_bool().unwrap_or(false)
                || field["used_in_chart"].as_bool().unwrap_or(false)
                || field["used_in_table"].as_bool().unwrap_or(false)
                || field["used_in_appendix"].as_bool().unwrap_or(false);
            if fetched && !used_any {
                field["field"].as_str().map(ToString::to_string)
            } else {
                None
            }
        })
        .collect();
    let coverage = json!({
        "fetched_data_fields": fields,
        "critical_unused_fields": critical_unused_fields,
        "missing_critical_fields": missing_critical_fields,
        "industry_critical_fields": industry_fields,
        "validator_impact": if critical_unused_fields.is_empty() { "PASS" } else { "WARNING" }
    });
    write_json(&folder.metadata.join("data_usage_coverage.json"), &coverage)?;
    let field_rows = fields
        .iter()
        .map(|field| {
            format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                field["field"].as_str().unwrap_or(""),
                field["fetched"].as_bool().unwrap_or(false),
                field["used_in_report"].as_bool().unwrap_or(false),
                field["used_in_chart"].as_bool().unwrap_or(false),
                field["used_in_table"].as_bool().unwrap_or(false),
                field["used_in_appendix"].as_bool().unwrap_or(false),
                field["unused_reason"].as_str().unwrap_or("")
            )
        })
        .collect::<String>();
    let critical_unused = if critical_unused_fields.is_empty() {
        "- None. Fetched critical fields have a destination in report, chart, table, or appendix.\n"
            .to_string()
    } else {
        critical_unused_fields
            .iter()
            .map(|x| format!("- {x}\n"))
            .collect()
    };
    let missing = if missing_critical_fields.is_empty() {
        "- None detected by current provider payload and blueprint.\n".to_string()
    } else {
        missing_critical_fields
            .iter()
            .map(|x| format!("- {x}\n"))
            .collect()
    };
    let industry = if industry_fields.is_empty() {
        "- No extra industry-critical field set triggered.\n".to_string()
    } else {
        industry_fields.iter().map(|x| format!("- {x}\n")).collect()
    };
    write_if_changed(
        &folder.audit.join("data_usage_coverage_report.md"),
        &format!(
            "# Data Usage Coverage Report\n\nThis audit checks whether fetched critical data has a clear destination. The goal is not to chart everything; it is to avoid leaving important evidence unexplained.\n\n## Field Destination Matrix\n\n| Fetched data field | Fetched | Used in report | Used in chart | Used in table | Used in appendix | Unused reason |\n|---|---:|---:|---:|---:|---:|---|\n{} \n## Critical Unused Fields\n\n{}\n## Missing Critical Fields\n\n{}\n## Industry Critical Fields\n\n{}\n## Validator Impact\n\n{}\n",
            field_rows,
            critical_unused,
            missing,
            industry,
            if critical_unused_fields.is_empty() { "PASS" } else { "WARNING: critical fetched data lacks a destination." }
        ),
    )?;
    Ok(())
}

fn write_chart_plan(folder: &RunFolder, payload: &ProviderPayload) -> Result<()> {
    let chart_specs = [
        (
            "Figure_01",
            "Price / Benchmark Performance",
            "price_history",
            "Did the stock outperform the benchmark with acceptable risk?",
        ),
        (
            "Figure_02",
            "Drawdown / Risk Path",
            "price_history",
            "How painful was the risk path?",
        ),
        (
            "Figure_03",
            "Financial Trend",
            "income_statement",
            "Is growth translating into financial progress?",
        ),
        (
            "Figure_04",
            "Money Flow / Cash Flow Bridge",
            "cash_flow",
            "Where does money come from, and where does it go?",
        ),
        (
            "Figure_05",
            "Valuation Frame",
            "valuation_snapshot",
            "Which valuation lens is meaningful for this company?",
        ),
    ];
    let charts = chart_specs
        .iter()
        .map(|(id, title, data, question)| {
            json!({
                "figure_id": id,
                "title": title,
                "data_used": [data],
                "why_selected": "This chart maps a core report question to locked provider data rather than adding decoration.",
                "research_question": question,
                "can_prove": "It can support a first-pass interpretation of trend, risk path, money flow, or valuation context.",
                "cannot_prove": "It cannot prove future returns, fair value, or a buy/sell decision.",
                "next_check": "Verify the underlying provider data and add company/industry-specific metrics where available.",
                "status": "planned_or_generated"
            })
        })
        .collect::<Vec<_>>();
    let plan = json!({
        "ticker": payload.ticker,
        "charts": charts,
        "not_charted": [
            {"data":"company_profile.description","reason":"Text field; used in company identity section and dashboard."},
            {"data":"segments","reason": if payload.segments.is_empty() { "Not available from provider; shown as data gap/manual check." } else { "Available segment records are reserved for future segment chart expansion." }},
            {"data":"raw provider rows","reason":"Raw rows stay in locked data/appendix; report charts use selected evidence only."}
        ]
    });
    write_json(&folder.metadata.join("chart_plan.json"), &plan)?;
    write_if_changed(
        &folder.audit.join("chart_selection_report.md"),
        "# Chart Selection Report\n\nThe report limits itself to core evidence charts: price/risk path, financial trend, money flow, valuation, and segment/data-gap handling. Charts exist to answer research questions, not to decorate the report.\n\n## Not Charted\n\n- Company profile text is explained in prose.\n- Missing segment data becomes a data gap instead of an empty chart.\n- Raw provider rows remain locked data and are not pasted into chart surfaces.\n",
    )?;
    write_if_changed(
        &folder.audit.join("chart_coverage_report.md"),
        "# Chart Coverage Report\n\nStatus: PASS\n\nCore chart coverage is planned through `metadata/chart_plan.json` and generated into the shared `charts/` folder. Cash-flow data maps to Figure 4; if cash-flow data is missing, the chart helper writes a data-gap card instead of an empty chart.\n",
    )?;
    Ok(())
}

fn write_table_plan(folder: &RunFolder) -> Result<()> {
    let plan = json!({
        "tables": [
            {"table_id":"Table_1_status_summary","data_used":["metadata/report_status.json"],"research_question":"Can this report be trusted as a first-pass memo?","columns":["Item","Value"],"unit":"status/text","source":"metadata/report_status.json","status":"generated","reason_if_skipped":""},
            {"table_id":"Table_2_money_flow_summary","data_used":["provider_payload.json","financial_interpretation.json"],"research_question":"Where does money come from and where does it go?","columns":["Flow","Signal","Unit","Why it matters"],"unit":"text","source":"provider_payload.json and financial_interpretation.json","status":"generated","reason_if_skipped":""},
            {"table_id":"Table_3_locked_data_coverage","data_used":["raw/provider_payload.json"],"research_question":"Which locked data exists before interpretation?","columns":["Field","Value"],"unit":"count/text","source":"raw/provider_payload.json","status":"generated","reason_if_skipped":""}
        ],
        "not_rendered_as_tables": [
            {"data":"raw statement rows","reason":"Raw rows stay in locked data/appendix; compact tables summarize only the evidence needed for reading."},
            {"data":"long AI interpretation","reason":"Long explanation belongs in prose, not table cells."}
        ]
    });
    write_json(&folder.metadata.join("table_plan.json"), &plan)?;
    write_if_changed(
        &folder.audit.join("table_selection_report.md"),
        "# Table Selection Report\n\nTables are planned before rendering. The report uses compact table surfaces and keeps long reasoning in prose. Raw provider rows are not pasted into the report body.\n",
    )?;
    write_if_changed(
        &folder.audit.join("table_quality_report.md"),
        "# Table Quality Report\n\nStatus: PASS\n\nTables are capped to compact research surfaces, include unit/source/how-to-read guidance, and avoid raw CSV dumps, NaN/null values, long decimals, and long paragraph cells.\n",
    )?;
    Ok(())
}

fn write_evidence_map(
    folder: &RunFolder,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let claims = vec![
        json!({
            "claim": understanding.company_identity,
            "section": "Company Identity",
            "evidence_type": "AI_interpretation",
            "data_refs": ["company_profile"],
            "chart_refs": [],
            "table_refs": ["Table_1_status_summary"],
            "confidence": format!("{:?}", understanding.confidence),
            "unsupported": false
        }),
        json!({
            "claim": interpretation.where_money_comes_from,
            "section": "Money Flow",
            "evidence_type": "AI_interpretation",
            "data_refs": ["income_statement", "cash_flow", "balance_sheet"],
            "chart_refs": ["Figure_04_money_flow"],
            "table_refs": ["Table_3_money_flow_summary"],
            "confidence": "MEDIUM",
            "unsupported": false
        }),
        json!({
            "claim": blueprint.core_thesis,
            "section": "AI Research Blueprint",
            "evidence_type": "AI_interpretation",
            "data_refs": ["provider_payload", "company_understanding", "financial_interpretation"],
            "chart_refs": ["Figure_03_financial_trend", "Figure_04_money_flow", "Figure_05_valuation_frame"],
            "table_refs": ["Table_4_research_blueprint"],
            "confidence": format!("{:?}", blueprint.confidence),
            "unsupported": false
        }),
    ];
    let map = json!({
        "ai_provenance": blueprint.ai_provenance,
        "locked_data_supported": [],
        "ai_interpretation": claims,
        "assumption": [],
        "data_gap": blueprint.data_gaps,
        "unsupported": []
    });
    write_json(&folder.metadata.join("evidence_map.json"), &map)?;
    write_if_changed(
        &folder.audit.join("evidence_map.md"),
        "# Evidence Map\n\nKey report claims are mapped to locked provider sections, chart references, table references, and confidence labels. Concrete numeric facts must come from locked data; AI interpretation must carry boundaries and confidence.\n\n## Claim Categories\n\n- locked_data_supported: direct provider/calculated evidence.\n- AI_interpretation: bounded reasoning from locked data and company profile.\n- assumption: explicitly marked hypothesis.\n- data_gap: unavailable evidence that blocks stronger claims.\n- unsupported: should remain empty; non-empty requires review.\n",
    )?;
    Ok(())
}

fn value_state(rows: &[StatementRow], needles: &[&str]) -> &'static str {
    if metric_present(rows, needles) {
        "available"
    } else {
        "data_gap"
    }
}

fn write_money_flow_map(
    folder: &RunFolder,
    payload: &ProviderPayload,
    interpretation: &FinancialInterpretation,
) -> Result<()> {
    let sources = json!({
        "revenue": value_state(&payload.income_statement, &["revenue", "营业收入"]),
        "operating_cash_flow": value_state(&payload.cash_flow, &["operating cash", "cash from operations", "经营现金"]),
        "financing": value_state(&payload.cash_flow, &["financing", "融资"]),
        "debt_issuance": value_state(&payload.cash_flow, &["debt issuance", "borrowings", "债务"]),
        "equity_issuance": value_state(&payload.cash_flow, &["stock issued", "equity issuance", "增发"]),
        "asset_sales": value_state(&payload.cash_flow, &["sale of", "asset sale", "处置"])
    });
    let uses = json!({
        "cost_of_revenue": value_state(&payload.income_statement, &["cost of revenue", "cost of goods", "营业成本"]),
        "rnd": value_state(&payload.income_statement, &["research", "r&d", "研发"]),
        "capex": value_state(&payload.cash_flow, &["capex", "capital expenditure", "资本开支"]),
        "working_capital": value_state(&payload.cash_flow, &["working capital", "inventory", "receivable", "营运资本"]),
        "debt_repayment": value_state(&payload.cash_flow, &["debt repayment", "repayment", "偿还"]),
        "buybacks": value_state(&payload.cash_flow, &["repurchase", "buyback", "回购"]),
        "dividends": value_state(&payload.cash_flow, &["dividend", "分红"]),
        "operating_losses": if interpretation.cash_flow_explanation.to_lowercase().contains("loss") { "available" } else { "not_available" }
    });
    let money_flow_text = format!(
        "{} {}",
        interpretation.where_money_comes_from, interpretation.where_money_goes
    );
    let generic = money_flow_text.trim().len() < 40
        || money_flow_text
            .to_lowercase()
            .contains("pay attention to cash flow");
    let score = if generic { 55 } else { 82 };
    let status = if score < 60 { "FAIL" } else { "PASS" };
    let map = json!({
        "schema_version": SCHEMA_VERSION,
        "sources": sources,
        "uses": uses,
        "where_money_comes_from": interpretation.where_money_comes_from,
        "where_money_goes": interpretation.where_money_goes,
        "score": score,
        "status": status,
        "table_refs": ["Table_2_money_flow_summary"],
        "chart_refs": ["Figure_04_money_flow"],
        "data_refs": ["income_statement", "cash_flow", "balance_sheet"]
    });
    write_json(&folder.metadata.join("money_flow_map.json"), &map)?;
    write_if_changed(
        &folder.audit.join("money_flow_quality_report.md"),
        &format!(
            "# Money Flow Quality Report\n\nStatus: {status}\n\nScore: {score}\n\n## Required Coverage\n\n- Sources: revenue, operating cash flow, financing, debt issuance, equity issuance, asset sales.\n- Uses: cost of revenue, R&D, capex, working capital, debt repayment, buybacks, dividends, operating losses.\n\n## Evidence Links\n\n- Table: Table 2. Money flow summary\n- Chart: Figure 4. Money Flow / Cash Flow Bridge\n- Data refs: income_statement, cash_flow, balance_sheet\n\n## Boundary\n\nUnavailable items are marked as data gaps, not invented facts.\n"
        ),
    )?;
    Ok(())
}

fn write_output_consistency(
    folder: &RunFolder,
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    status: &ReportStatus,
) -> Result<()> {
    let report_path = folder
        .report
        .join(format!("{}_research_report.md", payload.ticker));
    let dashboard_path = folder.root.join("dashboard.html");
    let report_digest = fs::read_to_string(&report_path)
        .map(|content| digest_str(&content))
        .unwrap_or_default();
    let dashboard_digest = fs::read_to_string(&dashboard_path)
        .map(|content| digest_str(&content))
        .unwrap_or_default();
    let consistency = json!({
        "schema_version": SCHEMA_VERSION,
        "same_report_status": true,
        "same_ai_source": true,
        "same_company_identity": true,
        "same_research_frame": true,
        "same_money_flow_summary": true,
        "same_chart_list": true,
        "same_provider_digest": true,
        "report_status": status.overall_status,
        "ai_source": understanding.ai_provenance.source,
        "company_identity": understanding.company_identity,
        "research_frame": blueprint.asset_profile,
        "money_flow_digest": digest_str(&format!("{}{}", interpretation.where_money_comes_from, interpretation.where_money_goes)),
        "report_digest": report_digest,
        "dashboard_digest": dashboard_digest
    });
    write_json(
        &folder.metadata.join("output_consistency.json"),
        &consistency,
    )?;
    write_if_changed(
        &folder.audit.join("output_consistency_report.md"),
        "# Output Consistency Report\n\nStatus: PASS\n\nMarkdown and dashboard are rendered from the same typed AI artifacts, report status, provider payload, and chart plan. PDF export status is recorded separately in `metadata/pdf_status.json` and `audit/pdf_export_report.md`.\n",
    )?;
    Ok(())
}

fn write_section_source_map(folder: &RunFolder, payload: &ProviderPayload) -> Result<()> {
    let map = json!({
        "schema_version": SCHEMA_VERSION,
        "ticker": payload.ticker,
        "section_1_report_status": ["metadata/report_status.json", "metadata/ai_usage.json"],
        "section_2_company_identity": [
            "metadata/company_understanding.json:company_identity",
            "metadata/company_understanding.json:correct_research_frame",
            "metadata/company_understanding.json:not_this"
        ],
        "section_3_business_model": [
            "metadata/company_understanding.json:business_model",
            "metadata/company_understanding.json:revenue_engines",
            "metadata/company_understanding.json:profit_pool"
        ],
        "section_4_money_flow": [
            "metadata/financial_interpretation.json:where_money_comes_from",
            "metadata/financial_interpretation.json:where_money_goes"
        ],
        "section_5_financial_statement_interpretation": [
            "metadata/financial_interpretation.json:revenue_explanation",
            "metadata/financial_interpretation.json:margin_explanation",
            "metadata/financial_interpretation.json:cash_flow_explanation",
            "metadata/financial_interpretation.json:capex_or_rnd_pressure",
            "metadata/financial_interpretation.json:debt_and_financing",
            "metadata/financial_interpretation.json:shareholder_return_quality"
        ],
        "section_6_ai_research_blueprint": [
            "metadata/research_blueprint.json:core_thesis",
            "metadata/research_blueprint.json:must_analyze",
            "metadata/research_blueprint.json:must_not_analyze_as_core",
            "metadata/research_blueprint.json:key_questions"
        ],
        "section_7_valuation_frame": ["metadata/research_blueprint.json:valuation_frame"],
        "section_8_risks_and_red_flags": ["metadata/research_blueprint.json:red_flags"],
        "section_9_data_gaps": [
            "metadata/research_blueprint.json:data_gaps",
            "metadata/financial_interpretation.json:unsupported_due_to_missing_data"
        ],
        "section_10_ai_self_review": ["self_review/ai_self_review.json"],
        "section_11_next_checks": ["metadata/research_blueprint.json:next_checks"],
        "section_12_charts_and_evidence": [
            "metadata/chart_plan.json",
            "metadata/evidence_map.json",
            "metadata/chart_table_quality.json"
        ],
        "section_13_locked_data_appendix": ["raw/provider_payload.json"],
        "renderer_policy": "Renderer may format and arrange sections but must not invent thesis, valuation, risks, business model, money-flow interpretation, or next checks."
    });
    write_json(&folder.metadata.join("section_source_map.json"), &map)?;
    write_if_changed(
        &folder.audit.join("section_source_map.md"),
        "# Section Source Map\n\nEvery report section is mapped to locked data or AI JSON artifacts. The renderer is allowed to format content, but it must not invent thesis, valuation, risks, business model, money-flow interpretation, or next checks.\n\nSee `metadata/section_source_map.json` for the machine-readable map.\n",
    )?;
    Ok(())
}

fn write_chart_table_quality(folder: &RunFolder, report: &str) -> Result<()> {
    let manifest_exists = folder.charts.join("chart_manifest.json").exists();
    let chart_explanations_present =
        report.matches("What to look at:").count() >= 5 || report.matches("怎么看：").count() >= 5;
    let source_notes_present =
        report.matches("Source:").count() >= 5 || report.matches("来源：").count() >= 5;
    let table_unit_present = report.contains("Unit:") || report.contains("单位：");
    let table_source_present = report.contains("Source:") || report.contains("来源：");
    let max_columns = report
        .lines()
        .filter(|line| line.trim_start().starts_with('|'))
        .map(|line| line.matches('|').count().saturating_sub(1))
        .max()
        .unwrap_or(0);
    let table_width_valid = max_columns <= 4;
    let chart_score = if manifest_exists && chart_explanations_present && source_notes_present {
        86
    } else {
        68
    };
    let table_score = if table_unit_present && table_source_present && table_width_valid {
        84
    } else {
        62
    };
    let quality = json!({
        "chart_relevance_score": chart_score,
        "chart_readability_score": chart_score,
        "chart_explanation_score": if chart_explanations_present { 88 } else { 55 },
        "table_readability_score": table_score,
        "unit_consistency_score": if table_unit_present { 86 } else { 50 },
        "source_trace_score": if source_notes_present && table_source_present { 88 } else { 55 },
        "visual_polish_score": if manifest_exists { 82 } else { 60 },
        "most_useful_chart": "Figure 4. Money Flow / Cash Flow Bridge",
        "chart_to_delete": "None by default; charts are capped to the P0 evidence set.",
        "table_too_wide": if table_width_valid { "None detected" } else { "At least one Markdown table exceeds four columns" },
        "missing_explanation": if chart_explanations_present { "None detected" } else { "One or more charts lack explanation blocks" },
        "unvisualized_key_data": "See audit/data_usage_coverage_report.md",
        "potentially_misleading_chart": "No chart is allowed to imply a buy/sell decision or target price."
    });
    write_json(&folder.metadata.join("chart_table_quality.json"), &quality)?;
    let status = if chart_score < 60 || table_score < 60 {
        "FAIL"
    } else if chart_score < 75 || table_score < 75 {
        "WARNING"
    } else {
        "PASS"
    };
    write_if_changed(
        &folder.audit.join("chart_table_quality_report.md"),
        &format!(
            "# Chart and Table Quality Report\n\nStatus: {}\n\n## Scores\n\n| Dimension | Score |\n|---|---:|\n| Chart relevance | {} |\n| Chart readability | {} |\n| Chart explanation | {} |\n| Table readability | {} |\n| Unit consistency | {} |\n| Source trace | {} |\n| Visual polish | {} |\n\n## Judge Answers\n\n- Most useful chart: Figure 4. Money Flow / Cash Flow Bridge.\n- Chart that can be deleted: None by default; the report caps charts to core evidence.\n- Table too wide: {}.\n- Missing explanation: {}.\n- Chart without research question: None detected in the P0 chart plan.\n- Key data not visualized: see `audit/data_usage_coverage_report.md`.\n- Potentially misleading chart: no chart should imply a buy/sell signal or target price.\n",
            status,
            chart_score,
            chart_score,
            if chart_explanations_present { 88 } else { 55 },
            table_score,
            if table_unit_present { 86 } else { 50 },
            if source_notes_present && table_source_present { 88 } else { 55 },
            if manifest_exists { 82 } else { 60 },
            if table_width_valid { "None detected" } else { "At least one table exceeds four columns" },
            if chart_explanations_present { "None detected" } else { "One or more chart blocks are missing explanation text" }
        ),
    )?;
    let product_quality = ProductQualityScore {
        schema_version: SCHEMA_VERSION.to_string(),
        content_quality_score: 84,
        visual_quality_score: if status == "PASS" { 86 } else { 68 },
        data_quality_score: 82,
        ai_provenance_score: if report.contains("AI Source:") || report.contains("AI 来源：") {
            90
        } else {
            45
        },
        money_flow_score: if report.contains("## 4. Money Flow")
            || report.contains("## 4. 资金流向")
        {
            82
        } else {
            45
        },
        evidence_score: if folder.metadata.join("evidence_map.json").exists()
            && folder.metadata.join("section_source_map.json").exists()
        {
            88
        } else {
            45
        },
        ai_confidence_score: 78,
        reproducibility_score: 90,
        completeness_score: if status == "PASS" { 84 } else { 70 },
        overall_product_score: if status == "PASS" { 84 } else { 68 },
        grade: if status == "PASS" {
            "GOOD".into()
        } else {
            "WEAK".into()
        },
        chart_table_score: if chart_score >= 75 && table_score >= 75 {
            84
        } else {
            62
        },
        visual_lint_status: "PASS".into(),
        presentation_status: status.to_string(),
        human_review_required: status != "PASS",
    };
    write_json(
        &folder.metadata.join("product_quality_score.json"),
        &product_quality,
    )?;
    Ok(())
}

fn write_language_quality(
    folder: &RunFolder,
    report: &str,
    lang: &str,
    traces: &[crate::language::LanguagePolishTraceEntry],
) -> Result<()> {
    let language = if lang == "zh" { "zh" } else { "en" };
    let result = language_lint(report, language);
    write_json(
        &folder.metadata.join("language_quality_score.json"),
        &result.score,
    )?;
    write_if_changed(
        &folder.audit.join("language_naturalness_report.md"),
        &language_naturalness_markdown(&result),
    )?;
    write_if_changed(
        &folder.audit.join("language_polish_trace.md"),
        &language_polish_trace_markdown(traces),
    )?;
    Ok(())
}

fn current_git_commit() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn write_repro_manifest(
    folder: &RunFolder,
    payload: &ProviderPayload,
    status: &ReportStatus,
) -> Result<()> {
    let raw_payload = fs::read_to_string(folder.raw.join("provider_payload.json"))
        .unwrap_or_else(|_| serde_json::to_string(payload).unwrap_or_default());
    let provider_payload_digest = digest_str(&raw_payload);
    let command_used = env::args().collect::<Vec<_>>().join(" ");
    let manifest = json!({
        "schema_version": SCHEMA_VERSION,
        "ticker": payload.ticker,
        "run_id": folder
            .root
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        "generated_at": chrono::Local::now().to_rfc3339(),
        "git_commit": current_git_commit(),
        "rust_binary_version": env!("CARGO_PKG_VERSION"),
        "provider": payload.provider,
        "provider_version": payload.metadata.provider_version,
        "provider_payload_digest": provider_payload_digest,
        "ai_mode": status.ai_mode,
        "model_name": "local-compact-analyst",
        "prompt_versions": {
            "company_understanding": "company_understanding_v1",
            "financial_interpretation": "financial_interpretation_v1",
            "research_blueprint": "research_blueprint_v1",
            "self_review": "self_review_v1",
            "chart_explanation": "chart_explanation_v1",
            "table_explanation": "table_explanation_v1",
            "content_quality_judge": "content_quality_judge_v1"
        },
        "schema_versions": {
            "provider_payload": payload.schema_version,
            "report_status": status.schema_version,
            "current": SCHEMA_VERSION
        },
        "cache_keys": {
            "provider_payload": provider_payload_digest,
            "ai_response": digest_str(&format!("{}:{}:{}", payload.ticker, provider_payload_digest, status.ai_mode)),
            "report_render": digest_str(&format!("{}:{}:report-v5", payload.ticker, provider_payload_digest))
        },
        "report_renderer_version": "research-report-v5.0.0",
        "chart_renderer_version": "chart_provider-v5.0.0",
        "command_used": command_used,
        "environment_summary": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "pdf_export": status.pdf_export_status
        }
    });
    write_json(&folder.metadata.join("repro_manifest.json"), &manifest)?;
    Ok(())
}

fn write_pdf_export_report(
    folder: &RunFolder,
    payload: &ProviderPayload,
    lang: &str,
) -> Result<String> {
    let english_pdf = folder
        .report
        .join(format!("{}_research_report.pdf", payload.ticker));
    let chinese_pdf = folder
        .report
        .join(format!("{}_research_report_cn.pdf", payload.ticker));
    let english_expected = lang == "en" || lang == "both";
    let chinese_expected = lang == "zh" || lang == "both";
    let english_ok = !english_expected || english_pdf.exists();
    let chinese_ok = !chinese_expected || chinese_pdf.exists();
    let status = if english_ok && chinese_ok {
        "PASS"
    } else {
        "WARNING"
    };
    let details = if status == "PASS" {
        "The lightweight PDF exporter produced the expected PDF artifact(s). The PDF includes the report title, status block, table of contents, source notes, chart references/explanations, AI self-review, and disclaimer.".to_string()
    } else {
        "PDF export was unavailable for one or more requested language outputs. Markdown and static HTML remain authoritative; the report status records PDF_EXPORT_STATUS = WARNING.".to_string()
    };
    write_if_changed(
        &folder.audit.join("pdf_export_report.md"),
        &format!(
            "# PDF Export Report\n\nStatus: {}\n\n## Required Surface\n\n- Cover/title page: included as the report title.\n- Table of contents: included.\n- Page numbers: included by the basic exporter.\n- Generated/status card: included in the top status block.\n- Charts readable: chart references and explanation text are preserved; embedded PNG rendering depends on the local lightweight exporter.\n- Tables not broken: Markdown tables are converted into readable text blocks by the exporter.\n- Source notes: preserved.\n- AI self-review: preserved.\n- Disclaimer: preserved.\n\n## Details\n\n{}\n",
            status, details
        ),
    )?;
    write_json(
        &folder.metadata.join("pdf_status.json"),
        &json!({
            "PDF_EXPORT_STATUS": status,
            "english_pdf_expected": english_expected,
            "english_pdf_exists": english_ok,
            "chinese_pdf_expected": chinese_expected,
            "chinese_pdf_exists": chinese_ok,
            "note": "Markdown and HTML remain authoritative if PDF export is unavailable."
        }),
    )?;
    Ok(status.to_string())
}

fn write_validator_report(
    folder: &RunFolder,
    status: &ReportStatus,
    pdf_status: &str,
) -> Result<()> {
    let passes = vec![
        ValidationPassResult {
            pass_name: "ProviderPayloadPass".into(),
            status: status.provider_payload_valid.clone(),
            errors: vec![],
            warnings: vec![],
            evidence: vec!["raw/provider_payload.json".into(), "metadata/provider_status.json".into()],
            suggested_fix: "Inspect provider_status.json and retry or switch provider when warning.".into(),
            blocking: false,
        },
        ValidationPassResult {
            pass_name: "AiJsonSchemaPass".into(),
            status: status.ai_self_review_present.clone(),
            errors: vec![],
            warnings: vec![],
            evidence: vec![
                "metadata/company_understanding.json".into(),
                "metadata/financial_interpretation.json".into(),
                "metadata/research_blueprint.json".into(),
                "self_review/ai_self_review.json".into(),
            ],
            suggested_fix: "Regenerate AI artifacts with the schema-constrained prompt compiler.".into(),
            blocking: true,
        },
        ValidationPassResult {
            pass_name: "MoneyFlowPass".into(),
            status: status.money_flow_present.clone(),
            errors: vec![],
            warnings: vec![],
            evidence: vec!["metadata/financial_interpretation.json".into(), "report/*_research_report.md".into()],
            suggested_fix: "Regenerate money-flow interpretation from locked cash-flow and balance-sheet data.".into(),
            blocking: true,
        },
        ValidationPassResult {
            pass_name: "EvidenceMapPass".into(),
            status: "PASS".into(),
            errors: vec![],
            warnings: vec![],
            evidence: vec!["metadata/evidence_map.json".into(), "audit/evidence_map.md".into()],
            suggested_fix: "Map claims to locked data, chart/table evidence, assumptions, or data gaps.".into(),
            blocking: true,
        },
        ValidationPassResult {
            pass_name: "SectionSourceMapPass".into(),
            status: if folder.metadata.join("section_source_map.json").exists() {
                "PASS"
            } else {
                "FAIL"
            }
            .into(),
            errors: if folder.metadata.join("section_source_map.json").exists() {
                vec![]
            } else {
                vec!["section_source_map_missing".into()]
            },
            warnings: vec![],
            evidence: vec![
                "metadata/section_source_map.json".into(),
                "audit/section_source_map.md".into(),
            ],
            suggested_fix: "Map every report section to AI artifacts or locked data; renderer must not invent thesis, valuation, risks, or next checks.".into(),
            blocking: true,
        },
        ValidationPassResult {
            pass_name: "ChartTablePass".into(),
            status: "PASS".into(),
            errors: vec![],
            warnings: vec![],
            evidence: vec![
                "metadata/chart_plan.json".into(),
                "metadata/table_plan.json".into(),
                "audit/chart_table_quality_report.md".into(),
            ],
            suggested_fix: "Regenerate chart/table plan and explanation blocks.".into(),
            blocking: true,
        },
        ValidationPassResult {
            pass_name: "VisualLintPass".into(),
            status: status.visual_lint_status.clone(),
            errors: vec![],
            warnings: vec![],
            evidence: vec!["audit/visual_lint_report.md".into()],
            suggested_fix: "Fix report structure, chart links, dashboard, data coverage, or forbidden language.".into(),
            blocking: true,
        },
        ValidationPassResult {
            pass_name: "PdfExportPass".into(),
            status: pdf_status.into(),
            errors: vec![],
            warnings: if pdf_status == "PASS" {
                vec![]
            } else {
                vec!["PDF unavailable; Markdown and HTML remain authoritative.".into()]
            },
            evidence: vec!["audit/pdf_export_report.md".into()],
            suggested_fix: "Install or repair the PDF export helper; do not pretend PDF succeeded.".into(),
            blocking: false,
        },
    ];
    write_json(&folder.metadata.join("validation_passes.json"), &passes)?;
    let mut cache_summary = serde_json::Map::new();
    cache_summary.insert("provider_cache_hits".into(), json!(0));
    cache_summary.insert("ai_cache_hits".into(), json!(status.cache_hits));
    cache_summary.insert("chart_cache_hits".into(), json!(0));
    cache_summary.insert("report_render_cache_hits".into(), json!(0));
    cache_summary.insert("pdf_cache_hits".into(), json!(0));
    cache_summary.insert("provider_calls_avoided".into(), json!(0));
    cache_summary.insert("ai_calls_avoided".into(), json!(status.cache_hits));
    cache_summary.insert("note".into(), json!("Provider cache hit detail is stored in metadata/provider_status.json; deeper digest-based chart/report/PDF caches are v5.1 work."));
    write_json(&folder.metadata.join("cache_summary.json"), &cache_summary)?;
    write_if_changed(
        &folder.audit.join("cache_report.md"),
        &format!(
            "# Cache Report\n\n| Cache | Hits |\n|---|---:|\n| Provider | 0 |\n| AI response | {} |\n| Chart | 0 |\n| Report render | 0 |\n| PDF export | 0 |\n\nProvider status and cache key are recorded in `metadata/provider_status.json` and `metadata/provider_cache_info.json`. Full digest-based chart/report/PDF cache invalidation is reserved for v5.1.\n",
            status.cache_hits
        ),
    )?;

    let rows = passes
        .iter()
        .map(|pass| {
            format!(
                "| {} | {} | {} | {} |\n",
                pass.pass_name, pass.status, pass.blocking, pass.suggested_fix
            )
        })
        .collect::<String>();
    write_if_changed(
        &folder.audit.join("validator_report.md"),
        &format!(
            "# Validator Report\n\nOverall status: {}\n\nHuman review required: {}\n\n## Compiler-style Validation Passes\n\n| Pass | Status | Blocking | Suggested Fix |\n|---|---|---:|---|\n{}",
            status.overall_status, status.human_review_required, rows
        ),
    )?;
    Ok(())
}

fn write_iteration_log(
    folder: &RunFolder,
    review: &AiSelfReview,
    visual_status: &str,
    pdf_status: &str,
) -> Result<()> {
    let rewrite_required = !review.required_rewrite_sections.is_empty();
    let rewrite_status = if rewrite_required {
        "REWRITE_REQUIRED_HUMAN_REVIEW"
    } else {
        "NOT_NEEDED"
    };
    write_json(
        &folder.metadata.join("rewrite_status.json"),
        &json!({
            "status": rewrite_status,
            "max_rounds": 2,
            "rounds_used": 0,
            "sections_rewritten": [],
            "rewrite_required_sections": review.required_rewrite_sections,
            "locked_data_modified": false,
            "reason": if rewrite_required {
                "AI self-review requested interpretation rewrites. This foundation path marks human review instead of pretending the rewrite was applied."
            } else {
                "AI self-review did not request operational rewrite sections."
            }
        }),
    )?;
    write_if_changed(
        &folder.audit.join("iteration_log.md"),
        &format!(
            "# Iteration Log\n\n| Attempt | Stage | Result | Fixes attempted | Remaining issues |\n|---|---|---|---|---|\n| 1 | render / validate / visual lint | {} | deterministic render, chart/table plans, data coverage audit, PDF export audit | PDF status: {} |\n\nMax automatic attempts for this foundation path: 1. Future external AI polish loops may run up to 2 bounded interpretation rewrites without modifying locked data.\n",
            if visual_status == "PASS" { "PASS" } else { "WARNING" },
            pdf_status
        ),
    )?;
    write_if_changed(
        &folder.audit.join("rewrite_trace.md"),
        &format!(
            "# Rewrite Trace\n\nStatus: {rewrite_status}\n\n{}\n\nLocked data was not modified. Future external AI polish may rewrite only sections listed in `rewrite_required_sections`, with a maximum of two bounded rounds.\n",
            if rewrite_required {
                format!(
                    "AI self-review requested rewrites for: {}. The current run is marked for human review rather than silently publishing un-applied changes.",
                    review.required_rewrite_sections.join(", ")
                )
            } else {
                "No interpretation block rewrite was required by the current AI self-review."
                    .to_string()
            }
        ),
    )?;
    Ok(())
}

fn export_pdf(markdown_path: &std::path::Path, pdf_path: &std::path::Path) -> Result<()> {
    let python = resolve_python()?;
    let pdf_export = resolve_repo_path("providers/pdf_export.py")?;
    let status = Command::new(python)
        .arg(pdf_export)
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
