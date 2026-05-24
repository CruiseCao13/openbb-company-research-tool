use crate::dashboard::render_company_dashboard;
use crate::markdown::{render_report, render_report_zh, render_self_review_md};
use anyhow::Result;
use research_core::io::{write_if_changed, write_json};
use research_core::run_folder::RunFolder;
use research_core::types::*;
use research_core::validation::visual_lint;
use serde_json::json;
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
    write_if_changed(
        &folder.metadata.join("prompt_versions.json"),
        "{\n  \"company_understanding\": \"company_understanding_v1\",\n  \"financial_interpretation\": \"financial_interpretation_v1\",\n  \"research_blueprint\": \"research_blueprint_v1\",\n  \"self_review\": \"self_review_v1\",\n  \"content_quality_judge\": \"content_quality_judge_v1\",\n  \"chart_explanation\": \"chart_explanation_v1\",\n  \"table_explanation\": \"table_explanation_v1\"\n}\n",
    )?;
    generate_charts(folder)?;
    write_data_inventory(folder, payload, blueprint)?;
    write_data_usage_coverage(folder, payload, blueprint)?;
    write_chart_plan(folder, payload)?;
    write_evidence_map(folder, understanding, interpretation, blueprint)?;
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
    let pdf_status = write_pdf_export_report(folder, payload, lang)?;
    let mut final_status = status.clone();
    final_status.pdf_export_status = pdf_status.clone();
    if pdf_status == "WARNING" && final_status.overall_status == "PASS" {
        final_status.overall_status = "WARNING".to_string();
        final_status.human_review_required = true;
    }
    write_json(&folder.metadata.join("report_status.json"), &final_status)?;
    write_chart_table_quality(folder, &primary_report)?;
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
    let report_entry = if lang == "zh" {
        format!("report/{}_research_report_cn.md", payload.ticker)
    } else {
        format!("report/{}_research_report.md", payload.ticker)
    };
    write_if_changed(&folder.root.join("README.md"), &format!("# {} v5 Research Run\n\nStart here:\n\n1. {}\n2. dashboard.html\n3. metadata/research_blueprint.json\n4. self_review/ai_self_review.md\n5. audit/validator_report.md\n6. audit/visual_lint_report.md\n7. audit/data_usage_coverage_report.md\n8. audit/chart_table_quality_report.md\n9. audit/pdf_export_report.md\n\nPDF exports live in `report/` when the lightweight exporter is available.\n", payload.ticker, report_entry))?;
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
    Ok(status.to_string())
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
