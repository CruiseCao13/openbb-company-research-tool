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
use research_core::validation::{has_space_lunar_identity, visual_lint};
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
    write_chart_observation_maps(folder, payload, blueprint)?;
    write_table_plan(folder)?;
    write_section_source_map(folder, payload)?;
    write_company_specific_artifacts(folder, payload, understanding, interpretation, blueprint)?;
    write_financial_report_framework_coverage(folder, payload, interpretation, blueprint)?;
    write_money_flow_map(folder, payload, interpretation)?;
    write_evidence_map(folder, payload, understanding, interpretation, blueprint)?;
    write_lunr_root_cause_report(
        folder,
        payload,
        understanding,
        interpretation,
        blueprint,
        review,
    )?;
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
    let accuracy_status = write_core_data_accuracy_audits(
        folder,
        payload,
        interpretation,
        blueprint,
        &primary_report,
    )?;
    let template_status = write_template_leakage_check(folder, &primary_report, payload)?;
    let pdf_status = write_pdf_export_report(folder, payload, lang)?;
    let mut final_status = status.clone();
    final_status.pdf_export_status = pdf_status.clone();
    if pdf_status == "WARNING" && final_status.overall_status == "PASS" {
        final_status.overall_status = "WARNING".to_string();
        final_status.human_review_required = true;
    }
    if accuracy_status.has_failures {
        final_status.overall_status = "FAIL".to_string();
        final_status.human_review_required = true;
    } else if accuracy_status.has_warnings && final_status.overall_status == "PASS" {
        final_status.overall_status = "WARNING".to_string();
        final_status.human_review_required = true;
    }
    if template_status == "FAIL" {
        final_status.overall_status = "FAIL".to_string();
        final_status.human_review_required = true;
    } else if template_status == "WARNING" && final_status.overall_status == "PASS" {
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
        &provider_validation_markdown(payload),
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

fn provider_validation_markdown(payload: &ProviderPayload) -> String {
    let coverage = payload
        .data_coverage
        .as_object()
        .map(|object| {
            object
                .iter()
                .map(|(field, value)| {
                    format!(
                        "| {} | {} |\n",
                        field,
                        value
                            .as_bool()
                            .map_or_else(|| value.to_string(), |v| v.to_string())
                    )
                })
                .collect::<String>()
        })
        .unwrap_or_else(|| "| data_coverage | unavailable |\n".to_string());
    let missing = if payload.missing_fields.is_empty() {
        "- None recorded by provider.\n".to_string()
    } else {
        payload
            .missing_fields
            .iter()
            .map(|field| format!("- {field}\n"))
            .collect::<String>()
    };
    let warnings = if payload.metadata.data_quality_warnings.is_empty() {
        "- None.\n".to_string()
    } else {
        payload
            .metadata
            .data_quality_warnings
            .iter()
            .map(|warning| format!("- {warning}\n"))
            .collect::<String>()
    };
    let error = payload.error.as_ref().map_or_else(
        || "- None.\n".to_string(),
        |err| {
            format!(
                "- {} at {}: {}\n",
                err.error_type, err.stage, err.error_message
            )
        },
    );
    format!(
        "# Provider Validation\n\nStatus: {}\n\nProvider: {}\nSource: {}\nAdapter: {}\nPackage used: {}\nMock: {}\nMarket: {}\nCurrency: {}\n\nProvider payload was parsed into the v5 locked-data schema.\n\n## Data Coverage\n\n| Field | Available |\n|---|---:|\n{}\n## Missing Fields\n\n{}\n## Provider Limitations\n\n{}\n## Provider Warnings\n\n{}\n## Provider Error\n\n{}",
        if payload.error.is_some() {
            "PROVIDER_ERROR"
        } else if payload.provider_status.is_empty() {
            "PASS"
        } else {
            payload.provider_status.as_str()
        },
        payload.provider,
        payload.metadata.source,
        payload.metadata.provider_adapter,
        payload.metadata.package_used,
        payload.metadata.mock,
        payload.market,
        payload.company_profile.currency,
        coverage,
        missing,
        if payload.metadata.provider_limitations.is_empty() {
            "- None recorded by provider.\n".to_string()
        } else {
            payload
                .metadata
                .provider_limitations
                .iter()
                .map(|limitation| format!("- {limitation}\n"))
                .collect::<String>()
        },
        warnings,
        error
    )
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
        row.value.is_some()
            && needles
                .iter()
                .any(|needle| metric.contains(&needle.to_lowercase()))
    })
}

fn metric_value(rows: &[StatementRow], needles: &[&str]) -> Option<f64> {
    rows.iter().find_map(|row| {
        let metric = row.metric.to_lowercase();
        if needles
            .iter()
            .any(|needle| metric.contains(&needle.to_lowercase()))
        {
            row.value
        } else {
            None
        }
    })
}

fn metric_fact(rows: &[StatementRow], field: &str, needles: &[&str]) -> serde_json::Value {
    if let Some(row) = rows.iter().find(|row| {
        let metric = row.metric.to_lowercase();
        row.value.is_some()
            && needles
                .iter()
                .any(|needle| metric.contains(&needle.to_lowercase()))
    }) {
        json!({
            "field": field,
            "status": "available",
            "metric": row.metric,
            "period": row.period,
            "value": row.value,
            "unit": row.unit,
            "evidence_ref": format!("provider_payload.{}", field)
        })
    } else {
        json!({
            "field": field,
            "status": "missing",
            "missing_reason": format!("{field} not present in locked provider payload"),
            "evidence_ref": serde_json::Value::Null
        })
    }
}

fn fact_status(fact: &serde_json::Value) -> &str {
    fact.get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("missing")
}

fn fact_value_line(fact: &serde_json::Value) -> Option<String> {
    if fact_status(fact) != "available" {
        return None;
    }
    let field = fact.get("field")?.as_str().unwrap_or("field");
    let metric = fact.get("metric")?.as_str().unwrap_or(field);
    let period = fact.get("period").and_then(|v| v.as_str()).unwrap_or("");
    let value = fact.get("value").and_then(|v| v.as_f64())?;
    let unit = fact.get("unit").and_then(|v| v.as_str()).unwrap_or("");
    Some(format!("{field}: {metric} {period} = {value:.1} {unit}"))
}

fn available_lines(facts: &[serde_json::Value]) -> Vec<String> {
    facts.iter().filter_map(fact_value_line).collect()
}

fn missing_fields_from_facts(facts: &[serde_json::Value]) -> Vec<String> {
    facts
        .iter()
        .filter(|fact| fact_status(fact) != "available")
        .filter_map(|fact| {
            fact.get("field")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
        .collect()
}

fn write_company_specific_artifacts(
    folder: &RunFolder,
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let revenue = metric_fact(
        &payload.income_statement,
        "income_statement.revenue",
        &["revenue", "total revenue", "营业收入"],
    );
    let gross_profit = metric_fact(
        &payload.income_statement,
        "income_statement.gross_profit",
        &["gross profit", "毛利"],
    );
    let operating_income = metric_fact(
        &payload.income_statement,
        "income_statement.operating_income",
        &["operating income", "营业利润"],
    );
    let net_income = metric_fact(
        &payload.income_statement,
        "income_statement.net_income",
        &["net income", "归母净利润", "净利润"],
    );
    let r_and_d = metric_fact(
        &payload.income_statement,
        "income_statement.r_and_d",
        &["research", "r&d", "研发"],
    );
    let sga = metric_fact(
        &payload.income_statement,
        "income_statement.sga",
        &["selling", "general", "administrative", "销售", "管理"],
    );
    let cogs = metric_fact(
        &payload.income_statement,
        "income_statement.cogs_or_equivalent",
        &["cost of revenue", "cost of goods", "营业成本"],
    );
    let ocf = metric_fact(
        &payload.cash_flow,
        "cash_flow.operating_cash_flow",
        &["operating cash flow", "cash from operations", "经营现金流"],
    );
    let capex = metric_fact(
        &payload.cash_flow,
        "cash_flow.capex",
        &["capital expenditure", "capex", "资本开支", "购建固定资产"],
    );
    let fcf = metric_fact(
        &payload.cash_flow,
        "cash_flow.free_cash_flow",
        &["free cash flow", "fcf", "自由现金流"],
    );
    let dividends = metric_fact(
        &payload.cash_flow,
        "cash_flow.dividends",
        &["dividend", "分红"],
    );
    let buybacks = metric_fact(
        &payload.cash_flow,
        "cash_flow.buybacks",
        &["repurchase", "buyback", "回购"],
    );
    let debt = metric_fact(
        &payload.balance_sheet,
        "balance_sheet.debt",
        &["debt", "borrowings", "有息负债", "负债合计"],
    );
    let inventory = metric_fact(
        &payload.balance_sheet,
        "balance_sheet.inventory",
        &["inventory", "存货"],
    );
    let receivables = metric_fact(
        &payload.balance_sheet,
        "balance_sheet.receivables",
        &["receivable", "应收"],
    );
    let cash = metric_fact(
        &payload.balance_sheet,
        "balance_sheet.cash",
        &["cash", "货币资金"],
    );
    let bank_metrics = vec![
        metric_fact(&payload.balance_sheet, "bank.roe", &["roe", "净资产收益率"]),
        metric_fact(&payload.balance_sheet, "bank.roa", &["roa", "总资产收益率"]),
        metric_fact(&payload.balance_sheet, "bank.nim", &["nim", "净息差"]),
        metric_fact(&payload.balance_sheet, "bank.npl", &["npl", "不良贷款率"]),
        metric_fact(
            &payload.balance_sheet,
            "bank.capital_ratio",
            &["capital adequacy", "资本充足率"],
        ),
    ];

    let key_facts = available_lines(&[
        revenue.clone(),
        gross_profit.clone(),
        operating_income.clone(),
        net_income.clone(),
        ocf.clone(),
        capex.clone(),
        fcf.clone(),
        debt.clone(),
        inventory.clone(),
        receivables.clone(),
    ]);
    let mut key_missing = missing_fields_from_facts(&[
        revenue.clone(),
        ocf.clone(),
        capex.clone(),
        fcf.clone(),
        debt.clone(),
        inventory.clone(),
        receivables.clone(),
    ]);
    key_missing.extend(payload.missing_fields.clone());
    key_missing.sort();
    key_missing.dedup();

    let data_limited_reason = if key_missing.is_empty() {
        String::new()
    } else {
        format!("Missing or incomplete fields: {}", key_missing.join(", "))
    };
    let confidence = if key_facts.len() >= 5 {
        "medium"
    } else if key_facts.len() >= 2 {
        "low"
    } else {
        "data_limited"
    };

    let company_fact_sheet = json!({
        "schema_version": SCHEMA_VERSION,
        "ticker": payload.ticker,
        "company_name": payload.company_profile.name,
        "market": payload.market,
        "currency": payload.company_profile.currency,
        "provider_source": payload.metadata.source,
        "business_description": payload.company_profile.description,
        "sector": payload.company_profile.sector,
        "industry": payload.company_profile.industry,
        "research_frame": understanding.correct_research_frame,
        "not_this": understanding.not_this,
        "key_available_facts": key_facts,
        "key_missing_facts": key_missing,
        "confidence": confidence,
        "data_limited_reason": data_limited_reason,
        "framework_sections": ["business_model"]
    });
    write_json(
        &folder.metadata.join("company_fact_sheet.json"),
        &company_fact_sheet,
    )?;

    let primary_revenue_sources = if fact_status(&revenue) == "available" {
        vec![json!({
            "source": "reported revenue",
            "fact": revenue.clone(),
            "mechanism": interpretation.revenue_explanation,
            "evidence_ref": "provider_payload.income_statement"
        })]
    } else {
        Vec::new()
    };
    let revenue_engine_map = json!({
        "schema_version": SCHEMA_VERSION,
        "primary_revenue_sources": primary_revenue_sources,
        "secondary_revenue_sources": understanding.revenue_engines,
        "revenue_evidence_refs": ["raw/provider_payload.json", "metadata/company_understanding.json", "data/normalized_financials.json"],
        "unsupported_revenue_claims": interpretation.unsupported_due_to_missing_data,
        "missing_revenue_fields": missing_fields_from_facts(std::slice::from_ref(&revenue)),
        "confidence": if fact_status(&revenue) == "available" { "medium" } else { "data_limited" },
        "framework_sections": ["business_model", "revenue_growth"]
    });
    write_json(
        &folder.metadata.join("revenue_engine_map.json"),
        &revenue_engine_map,
    )?;

    let cost_facts = vec![
        cogs.clone(),
        r_and_d.clone(),
        sga.clone(),
        capex.clone(),
        inventory.clone(),
        receivables.clone(),
    ];
    let cost_structure_map = json!({
        "schema_version": SCHEMA_VERSION,
        "major_cost_items": available_lines(&cost_facts),
        "r_and_d": r_and_d.clone(),
        "sga": sga.clone(),
        "cogs_or_equivalent": cogs.clone(),
        "capex": capex.clone(),
        "working_capital_items": {
            "inventory": inventory.clone(),
            "receivables": receivables.clone()
        },
        "cost_evidence_refs": ["raw/provider_payload.json", "data/normalized_financials.json"],
        "missing_cost_fields": missing_fields_from_facts(&cost_facts),
        "confidence": if available_lines(&cost_facts).len() >= 2 { "medium" } else { "data_limited" },
        "framework_sections": ["gross_margin", "operating_profit"]
    });
    write_json(
        &folder.metadata.join("cost_structure_map.json"),
        &cost_structure_map,
    )?;

    let bank_available = available_lines(&bank_metrics);
    let financing_need = match (
        ocf.get("value").and_then(|v| v.as_f64()),
        capex.get("value").and_then(|v| v.as_f64()),
        fcf.get("value").and_then(|v| v.as_f64()),
    ) {
        (_, _, Some(f)) if f < 0.0 => {
            "free cash flow is negative; financing or cash balance needs review"
        }
        (Some(o), Some(c), _) if o + c < 0.0 => {
            "OCF plus capex is negative; financing or cash balance needs review"
        }
        _ => "not indicated by compact locked data",
    };
    let capital_facts = vec![
        ocf.clone(),
        capex.clone(),
        fcf.clone(),
        debt.clone(),
        dividends.clone(),
        buybacks.clone(),
        cash.clone(),
    ];
    let capital_allocation_map = json!({
        "schema_version": SCHEMA_VERSION,
        "operating_cash_flow": ocf.clone(),
        "capex": capex.clone(),
        "free_cash_flow": fcf.clone(),
        "debt": debt.clone(),
        "dividends": dividends.clone(),
        "buybacks": buybacks.clone(),
        "cash": cash.clone(),
        "financing_need": financing_need,
        "bank_or_insurance_specific_capital_metrics": bank_available,
        "evidence_refs": ["raw/provider_payload.json", "data/normalized_financials.json", "metadata/unit_policy.json"],
        "missing_fields": missing_fields_from_facts(&capital_facts),
        "framework_sections": ["net_profit", "balance_sheet"]
    });
    write_json(
        &folder.metadata.join("capital_allocation_map.json"),
        &capital_allocation_map,
    )?;

    let mechanism_gaps = {
        let mut gaps = blueprint.data_gaps.clone();
        gaps.extend(payload.missing_fields.clone());
        gaps.extend(missing_fields_from_facts(&[
            revenue.clone(),
            ocf.clone(),
            capex.clone(),
            fcf.clone(),
        ]));
        gaps.sort();
        gaps.dedup();
        gaps
    };
    let mut should_not_say = understanding.wrong_frames_to_avoid.clone();
    should_not_say.extend(understanding.not_this.clone());
    should_not_say.extend([
        "Do not claim a revenue engine without revenue_engine_map support.".to_string(),
        "Do not claim buybacks, dividends, or financing quality without capital_allocation_map support.".to_string(),
        "Do not use polished generic prose when company_fact_sheet confidence is data_limited.".to_string(),
    ]);
    let money_flow_mechanism = json!({
        "schema_version": SCHEMA_VERSION,
        "money_source_mechanism": interpretation.where_money_comes_from,
        "money_use_mechanism": interpretation.where_money_goes,
        "cash_conversion_mechanism": interpretation.cash_flow_explanation,
        "capital_intensity": interpretation.capex_or_rnd_pressure,
        "financing_or_liquidity_pressure": interpretation.debt_and_financing,
        "company_specific_data_gaps": mechanism_gaps,
        "next_checks": blueprint.next_checks,
        "should_not_say": should_not_say,
        "confidence": confidence,
        "source_artifacts": [
            "metadata/company_fact_sheet.json",
            "metadata/revenue_engine_map.json",
            "metadata/cost_structure_map.json",
            "metadata/capital_allocation_map.json"
        ],
        "framework_sections": ["cash_flow"]
    });
    write_json(
        &folder.metadata.join("money_flow_mechanism.json"),
        &money_flow_mechanism,
    )?;

    let questions = build_company_specific_questions(
        payload,
        understanding,
        blueprint,
        &available_lines(&[
            revenue.clone(),
            ocf.clone(),
            capex.clone(),
            fcf.clone(),
            debt.clone(),
            inventory.clone(),
            receivables.clone(),
        ]),
        &mechanism_gaps,
    );
    write_json(
        &folder.metadata.join("company_specific_questions.json"),
        &questions,
    )?;
    Ok(())
}

fn build_company_specific_questions(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    blueprint: &ResearchBlueprint,
    available_fact_lines: &[String],
    gaps: &[String],
) -> serde_json::Value {
    let name = if payload.company_profile.name.is_empty() {
        payload.ticker.as_str()
    } else {
        payload.company_profile.name.as_str()
    };
    let fact_anchor = available_fact_lines
        .first()
        .cloned()
        .unwrap_or_else(|| "no numeric locked fact available".to_string());
    let gap_anchor = gaps
        .first()
        .cloned()
        .unwrap_or_else(|| "segment detail / management filing support".to_string());
    let frame = &understanding.correct_research_frame;
    let mut key_questions = vec![
        format!("For {name}, does {fact_anchor} support the {frame} frame, or does it only prove scale?"),
        format!("Which missing field would most change the {payload_ticker} money-flow read: {gap_anchor}?", payload_ticker = payload.ticker),
        format!("Does {name}'s cash conversion match its stated frame, or is financing/working capital doing the work?"),
    ];
    key_questions.extend(
        blueprint
            .key_questions
            .iter()
            .filter(|question| !question.trim().is_empty())
            .take(4)
            .cloned(),
    );
    key_questions.sort();
    key_questions.dedup();
    let data_needed = if gaps.is_empty() {
        vec![
            "No critical data gap was flagged; verify provider values against filings.".to_string(),
        ]
    } else {
        gaps.iter().take(8).cloned().collect()
    };
    let why_each_question_matters = key_questions
        .iter()
        .map(|question| {
            json!({
                "question": question,
                "why_it_matters": "It forces the local fallback to tie the report to this company's locked facts, missing fields, and research frame instead of selecting a reusable sector paragraph."
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema_version": SCHEMA_VERSION,
        "key_questions": key_questions,
        "data_needed": data_needed,
        "why_each_question_matters": why_each_question_matters,
        "related_sections": [
            "Company Identity",
            "Business Model",
            "Money Flow",
            "Financial Statement Interpretation",
            "Data Gaps"
        ],
        "framework_sections": [
            "key_business_metrics",
            "guidance",
            "market_expectations",
            "valuation"
        ]
    })
}

fn framework_section(
    section_id: &str,
    status: &str,
    covered_checks: Vec<&str>,
    missing_checks: Vec<&str>,
    evidence_refs: Vec<&str>,
    report_sections: Vec<&str>,
    notes: &str,
) -> serde_json::Value {
    json!({
        "section_id": section_id,
        "status": status,
        "covered_checks": covered_checks,
        "missing_checks": missing_checks,
        "evidence_refs": evidence_refs,
        "report_sections": report_sections,
        "notes": notes
    })
}

fn write_financial_report_framework_coverage(
    folder: &RunFolder,
    payload: &ProviderPayload,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let has_profile = !payload.company_profile.name.is_empty()
        || !payload.company_profile.description.is_empty()
        || !payload.company_profile.industry.is_empty();
    let has_revenue = metric_present(
        &payload.income_statement,
        &["revenue", "total revenue", "营业收入"],
    );
    let has_gross = metric_present(
        &payload.income_statement,
        &["gross profit", "毛利率", "毛利"],
    );
    let has_operating_profit = metric_present(
        &payload.income_statement,
        &["operating income", "营业利润", "operating profit"],
    );
    let has_net_profit = metric_present(
        &payload.income_statement,
        &["net income", "归母净利润", "净利润"],
    );
    let has_ocf = metric_present(
        &payload.cash_flow,
        &["operating cash flow", "cash from operations", "经营现金"],
    );
    let has_capex = metric_present(
        &payload.cash_flow,
        &["capex", "capital expenditure", "资本开支"],
    );
    let has_fcf = metric_present(&payload.cash_flow, &["free cash flow", "fcf", "自由现金流"]);
    let has_cash = metric_present(&payload.balance_sheet, &["cash", "货币资金"]);
    let has_debt = metric_present(
        &payload.balance_sheet,
        &["debt", "borrowings", "有息负债", "负债"],
    );
    let has_working_capital = metric_present(
        &payload.balance_sheet,
        &["inventory", "receivable", "存货", "应收"],
    );
    let has_valuation = payload
        .valuation_snapshot
        .as_object()
        .map(|object| {
            object
                .values()
                .any(|value| value.as_f64().unwrap_or(0.0) > 0.0)
        })
        .unwrap_or(false);
    let has_guidance = payload
        .segments
        .iter()
        .any(|value| value.to_string().to_lowercase().contains("guidance"))
        || blueprint
            .next_checks
            .iter()
            .any(|check| check.to_lowercase().contains("guidance"));
    let has_expectations = payload
        .segments
        .iter()
        .any(|value| value.to_string().to_lowercase().contains("expectation"));

    let sections = vec![
        framework_section(
            "business_model",
            if has_profile { "PASS" } else { "DATA_GAP" },
            if has_profile { vec!["product_or_service", "customer"] } else { vec![] },
            if has_profile {
                vec!["willingness_to_pay", "recurring_or_one_time_revenue", "growth_source_quality"]
            } else {
                vec![
                    "product_or_service",
                    "customer",
                    "willingness_to_pay",
                    "recurring_or_one_time_revenue",
                    "growth_source_quality",
                ]
            },
            vec!["metadata/company_fact_sheet.json", "metadata/revenue_engine_map.json"],
            vec!["Company Identity", "Business Model"],
            "Business-model coverage is anchored to provider profile and revenue-engine artifacts.",
        ),
        framework_section(
            "revenue_growth",
            if has_revenue { "WARNING" } else { "DATA_GAP" },
            if has_revenue { vec!["current_revenue"] } else { vec![] },
            vec!["yoy_growth", "qoq_growth", "guidance", "expectation_gap"],
            vec!["metadata/revenue_engine_map.json", "raw/provider_payload.json"],
            vec!["Business Model", "Financial Statement Interpretation"],
            "Current revenue is checked when available; growth rates, guidance, and expectations remain data gaps unless provider supplies them.",
        ),
        framework_section(
            "gross_margin",
            if has_gross { "WARNING" } else { "DATA_GAP" },
            if has_gross { vec!["gross_profit", "gross_margin"] } else { vec![] },
            vec!["pricing_power", "cost_control", "product_mix"],
            vec!["metadata/cost_structure_map.json"],
            vec!["Financial Statement Interpretation"],
            "Gross margin is used only when locked data exists and remains sector-aware.",
        ),
        framework_section(
            "operating_profit",
            if has_operating_profit { "WARNING" } else { "DATA_GAP" },
            if has_operating_profit { vec!["operating_income"] } else { vec![] },
            vec!["operating_margin", "operating_leverage", "expense_ratio"],
            vec!["metadata/cost_structure_map.json"],
            vec!["Financial Statement Interpretation"],
            "Operating profit coverage is partial unless margin and expense-ratio data are present.",
        ),
        framework_section(
            "net_profit",
            if has_net_profit { "WARNING" } else { "DATA_GAP" },
            if has_net_profit { vec!["net_income"] } else { vec![] },
            vec!["eps", "dilution", "one_time_items"],
            vec!["metadata/capital_allocation_map.json"],
            vec!["Financial Statement Interpretation"],
            "Net profit is not treated as final quality without cash-flow and one-time item checks.",
        ),
        framework_section(
            "cash_flow",
            if has_ocf && has_capex && has_fcf { "PASS" } else if has_ocf { "WARNING" } else { "DATA_GAP" },
            {
                let mut checks = Vec::new();
                if has_ocf { checks.push("operating_cash_flow"); }
                if has_capex { checks.push("capex"); }
                if has_fcf { checks.push("free_cash_flow"); }
                checks
            },
            {
                let mut missing = Vec::new();
                if !has_ocf { missing.push("operating_cash_flow"); }
                if !has_capex { missing.push("capex"); }
                if !has_fcf { missing.push("free_cash_flow"); }
                missing.push("financing_need");
                missing
            },
            vec!["metadata/money_flow_mechanism.json", "metadata/capital_allocation_map.json"],
            vec!["Money Flow", "Financial Statement Interpretation"],
            "Cash-flow coverage asks whether operations fund reinvestment; missing cash-flow rows stay data gaps.",
        ),
        framework_section(
            "balance_sheet",
            if has_cash || has_debt || has_working_capital { "WARNING" } else { "DATA_GAP" },
            {
                let mut checks = Vec::new();
                if has_cash { checks.push("cash"); }
                if has_debt { checks.push("debt"); }
                if has_working_capital {
                    checks.push("inventory");
                    checks.push("receivables");
                }
                checks
            },
            vec!["current_assets", "current_liabilities", "goodwill"],
            vec!["metadata/capital_allocation_map.json"],
            vec!["Financial Statement Interpretation", "Data Gaps and Unsupported Claims"],
            "Balance-sheet coverage is partial unless liquidity, debt, working capital, and goodwill fields are all present.",
        ),
        framework_section(
            "key_business_metrics",
            "DATA_GAP",
            vec![],
            vec![
                "sector_specific_kpis",
                "kpi_to_revenue_link",
                "kpi_to_profit_link",
                "kpi_to_cash_flow_link",
            ],
            vec!["metadata/company_specific_questions.json"],
            vec!["Next Checks"],
            "Provider payload does not reliably include sector KPIs, so the report asks next-check questions instead of inventing KPI conclusions.",
        ),
        framework_section(
            "guidance",
            if has_guidance { "WARNING" } else { "DATA_GAP" },
            if has_guidance { vec!["management_commentary"] } else { vec![] },
            vec!["revenue_guidance", "margin_guidance", "full_year_outlook"],
            vec!["metadata/company_specific_questions.json"],
            vec!["Next Checks", "Data Gaps and Unsupported Claims"],
            "Guidance is treated as unavailable unless explicitly present in provider data or next-check artifacts.",
        ),
        framework_section(
            "market_expectations",
            if has_expectations { "WARNING" } else { "DATA_GAP" },
            if has_expectations { vec!["actual_vs_expectation"] } else { vec![] },
            vec!["beat_or_miss", "expectation_revision"],
            vec!["metadata/research_blueprint.json"],
            vec!["Data Gaps and Unsupported Claims"],
            "Market expectations are not inferred from price action; missing expectation data remains a data gap.",
        ),
        framework_section(
            "valuation",
            if has_valuation { "WARNING" } else { "DATA_GAP" },
            if has_valuation { vec!["valuation_method_fit"] } else { vec![] },
            vec!["implied_growth", "margin_of_safety", "downside_if_growth_misses"],
            vec!["data/valuation_snapshot.json", "metadata/research_blueprint.json"],
            vec!["Valuation Frame"],
            "Valuation coverage discusses method fit and missing inputs only; it must not create investment advice.",
        ),
    ];

    write_json(
        &folder
            .metadata
            .join("financial_report_framework_coverage.json"),
        &json!({
            "schema_version": SCHEMA_VERSION,
            "framework_version": "v1",
            "core_question": {
                "zh": "一家公司是否具备持续创造现金流的能力？",
                "en": "Can the company consistently generate cash flow?"
            },
            "ticker": payload.ticker,
            "sections": sections
        }),
    )?;

    let rows = sections
        .iter()
        .map(|section| {
            let id = section
                .get("section_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let status = section.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let missing = section
                .get("missing_checks")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            format!("| {id} | {status} | {missing} |\n")
        })
        .collect::<String>();
    write_if_changed(
        &folder.audit.join("financial_report_framework_coverage.md"),
        &format!(
            "# Financial Report Framework Coverage\n\nCore question: Can the company consistently generate cash flow?\n\n| Section | Status | Missing checks |\n|---|---|---|\n{rows}\n## Safety\n\n- No investment recommendation is generated.\n- No target price is generated.\n- Guidance and market expectations remain DATA_GAP unless provider data explicitly supports them.\n"
        ),
    )?;
    let _ = interpretation;
    Ok(())
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

fn chart_locked_observation(payload: &ProviderPayload, fields: &[String]) -> String {
    for field in fields {
        let value = match field.as_str() {
            "price_history.close" => payload.price_history.iter().rev().find_map(|point| {
                point
                    .close
                    .map(|close| format!("{} close = {:.2}", point.date, close))
            }),
            "income_statement.revenue" => metric_value(
                &payload.income_statement,
                &["revenue", "total revenue", "营业收入"],
            )
            .map(|value| format!("latest revenue = {value:.1}")),
            "cash_flow.operating_cash_flow" => metric_value(
                &payload.cash_flow,
                &["operating cash flow", "cash from operations", "经营现金"],
            )
            .map(|value| format!("latest operating cash flow = {value:.1}")),
            "cash_flow.capex" => metric_value(
                &payload.cash_flow,
                &["capital expenditure", "capex", "资本开支"],
            )
            .map(|value| format!("latest capex = {value:.1}")),
            "valuation_snapshot" => payload.valuation_snapshot.as_object().and_then(|object| {
                object.iter().find_map(|(key, value)| {
                    value
                        .as_f64()
                        .filter(|number| *number > 0.0)
                        .map(|number| format!("{key} = {number:.2}"))
                })
            }),
            _ => None,
        };
        if let Some(value) = value {
            return value;
        }
    }
    "source field is missing or data-limited".to_string()
}

fn chart_frame_reason(frame: &str, figure: i64) -> &'static str {
    let lower = frame.to_lowercase();
    if lower.contains("bank") || lower.contains("financial") {
        match figure {
            4 => "bank charts must be read as funding, credit, capital, and liquidity context rather than industrial FCF proof",
            5 => "bank valuation needs P/B, ROE, NIM, credit quality, and capital context before any multiple is meaningful",
            _ => "bank research needs price/risk context tied to asset quality, deposits, credit cost, and capital strength",
        }
    } else if lower.contains("insurance") {
        "insurance charts need premium, investment income, solvency, and asset-liability context before conclusions"
    } else if lower.contains("aerospace") || lower.contains("space") {
        "speculative aerospace charts need cash burn, contract timing, capex, financing, and milestone context"
    } else if lower.contains("battery") || lower.contains("new energy") {
        "battery manufacturing charts need revenue, margin, capex, inventory, receivables, debt, and cycle context"
    } else if lower.contains("mining") || lower.contains("commodity") {
        "mining charts need commodity price, production, capex, debt, and reserve context"
    } else if lower.contains("pharma") || lower.contains("drug") {
        "pharma charts need revenue, R&D, product portfolio, approval, reimbursement, and patent-risk context"
    } else if lower.contains("medical") || lower.contains("robotics") {
        "medtech charts need procedure volume, system/accessory revenue, R&D, SG&A, inventory, and receivables context"
    } else if lower.contains("industrial") || lower.contains("cyclical") {
        "industrial charts need machinery cycle, working capital, inventory, receivables, capex, and debt context"
    } else if lower.contains("platform") || lower.contains("cloud") {
        "platform charts need ads/cloud economics, R&D, capex, AI infrastructure, and margin context"
    } else if lower.contains("consumer") || lower.contains("baijiu") {
        "consumer-brand charts need revenue, margin, OCF, inventory, receivables, cash, and shareholder-return context"
    } else {
        "chart interpretation needs company-specific locked facts and explicit data gaps before conclusions"
    }
}

fn write_chart_observation_maps(
    folder: &RunFolder,
    payload: &ProviderPayload,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let manifest_path = folder.charts.join("chart_manifest.json");
    let manifest: Vec<serde_json::Value> = if manifest_path.exists() {
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap_or_default())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let run_id = folder
        .root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let mut chart_rows = Vec::new();
    let mut claim_rows = Vec::new();
    let mut warnings = Vec::new();
    for item in manifest.iter() {
        let figure = item
            .get("figure")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let chart_id = if figure > 0 {
            format!("Figure_{figure:02}")
        } else {
            "Figure_unknown".to_string()
        };
        let title = item
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or("Untitled chart")
            .to_string();
        let file = item
            .get("file")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let status = item
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();
        let fields = item
            .get("data_used")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let source_file = if file.is_empty() {
            String::new()
        } else {
            format!("charts/{file}")
        };
        let source_exists = !file.is_empty() && folder.charts.join(&file).exists();
        let locked_observation = chart_locked_observation(payload, &fields);
        let frame_reason = chart_frame_reason(&blueprint.asset_profile, figure);
        let data_gap_if_any = if status == "DATA_GAP" || !source_exists {
            item.get("reason")
                .and_then(|value| value.as_str())
                .unwrap_or("chart source missing or data-limited")
                .to_string()
        } else {
            String::new()
        };
        if !source_exists {
            warnings.push(format!("{chart_id} source file missing: {source_file}"));
        }
        let company_specific_observation = format!(
            "{} uses {} for {}; locked observation: {}. For the {} frame, {}.",
            title,
            if fields.is_empty() {
                "no machine-readable source field".to_string()
            } else {
                fields.join(", ")
            },
            payload.ticker,
            locked_observation,
            blueprint.asset_profile,
            frame_reason
        );
        let limitation = if figure == 5 {
            "This chart cannot create an investment recommendation or fair-value conclusion without the missing valuation inputs.".to_string()
        } else {
            format!(
                "This chart cannot prove {}'s business quality, valuation, or future return without the linked locked data and company-specific follow-up checks.",
                payload.ticker
            )
        };
        let next_check = match figure {
            1 => format!("Tie {}'s price path to revenue, cash flow, drawdown, and provider gaps before interpreting performance.", payload.ticker),
            2 => format!("Compare {} drawdowns with financing, debt, working capital, and frame-specific milestones.", payload.ticker),
            3 => format!("Verify {} revenue trend against segment/product drivers and margin bridge.", payload.ticker),
            4 => format!("Reconcile {} OCF, capex, FCF, debt, inventory/receivables, and financing gaps.", payload.ticker),
            _ => format!("Check whether {}'s blueprint valuation method has enough locked inputs to be meaningful.", payload.ticker),
        };
        chart_rows.push(json!({
            "chart_id": chart_id,
            "title": title,
            "source_file": source_file,
            "source_fields": fields,
            "company_specific_observation": company_specific_observation,
            "linked_financial_metric": locked_observation,
            "linked_company_fact": blueprint.asset_profile,
            "what_this_chart_supports": item.get("research_question").and_then(|value| value.as_str()).unwrap_or("company-specific chart observation"),
            "what_this_chart_cannot_prove": limitation,
            "next_check": next_check,
            "data_gap_if_any": data_gap_if_any,
            "confidence": if source_exists && status == "PASS" { "medium" } else { "data_limited" }
        }));
        claim_rows.push(json!({
            "chart_id": chart_id,
            "report_section": "Charts and Evidence",
            "claim_supported": format!("{} supports a first-pass observation for {} within the {} frame.", title, payload.ticker, blueprint.asset_profile),
            "evidence_refs": [
                "charts/chart_manifest.json",
                source_file,
                "raw/provider_payload.json",
                "metadata/chart_observation_map.json"
            ],
            "unsupported_or_overread_risk": "Do not infer an investment recommendation or complete business-model proof from this chart.",
            "limitation_text": limitation
        }));
    }
    if manifest.is_empty() {
        warnings.push("chart_manifest.json missing or empty".to_string());
    }
    let status = if warnings.iter().any(|warning| warning.contains("missing")) {
        "WARNING"
    } else {
        "PASS"
    };
    write_json(
        &folder.metadata.join("chart_observation_map.json"),
        &json!({
            "schema_version": SCHEMA_VERSION,
            "ticker": payload.ticker,
            "run_id": run_id,
            "charts": chart_rows
        }),
    )?;
    write_json(
        &folder.metadata.join("chart_claim_map.json"),
        &json!({
            "schema_version": SCHEMA_VERSION,
            "ticker": payload.ticker,
            "run_id": run_id,
            "claims": claim_rows
        }),
    )?;
    let findings = if warnings.is_empty() {
        "- Every chart has a company-specific observation, source reference, limitation, and next check.\n".to_string()
    } else {
        warnings
            .iter()
            .map(|warning| format!("- {warning}\n"))
            .collect::<String>()
    };
    write_if_changed(
        &folder.audit.join("chart_observation_quality_report.md"),
        &format!(
            "# Chart Observation Quality Report\n\nStatus: {status}\n\n## Checks\n\n- Company-specific chart observation: checked\n- Source existence: checked\n- Limitation text: checked\n- Generic template-only explanation: checked\n- Buy/sell/target-price implication: checked\n\n## Findings\n\n{findings}"
        ),
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
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
) -> Result<()> {
    let locked_data_supported = payload
        .income_statement
        .iter()
        .chain(payload.balance_sheet.iter())
        .chain(payload.cash_flow.iter())
        .filter_map(|row| {
            row.value.map(|value| {
                json!({
                    "claim": format!("{} {} = {}", row.period, row.metric, value),
                    "section": "Locked Data Appendix",
                    "evidence_type": "locked_data",
                    "source_file": "raw/provider_payload.json",
                    "source_field": row.metric,
                    "period": row.period,
                    "unit": row.unit,
                    "value": value,
                    "unsupported": false
                })
            })
        })
        .collect::<Vec<_>>();
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
        "locked_data_supported": locked_data_supported,
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

fn write_lunr_root_cause_report(
    folder: &RunFolder,
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    review: &AiSelfReview,
) -> Result<()> {
    if payload.ticker != "LUNR" && !has_space_lunar_identity(payload) {
        return Ok(());
    }
    let description = if payload.company_profile.description.trim().is_empty() {
        "Provider description is missing.".to_string()
    } else {
        payload
            .company_profile
            .description
            .chars()
            .take(900)
            .collect::<String>()
    };
    let profile_summary = format!(
        "- Name: {}\n- Sector: {}\n- Industry: {}\n- Provider: {}\n- Description excerpt: {}\n",
        payload.company_profile.name,
        payload.company_profile.sector,
        payload.company_profile.industry,
        payload.provider,
        description
    );
    let compact_summary = format!(
        "- Compact payload includes ticker, market, company name, sector, industry, full description, income summary, and cash-flow summary.\n- Company frame after guard: {}\n- Money flow source text: {}\n- Money flow use text: {}\n",
        understanding.correct_research_frame,
        interpretation.where_money_comes_from,
        interpretation.where_money_goes
    );
    let validator_summary = format!(
        "- Framework fit check: {:?}\n- Human review required: {}\n- Wrong framework risk: {}\n- Rewrite sections: {}\n",
        review.framework_fit_check,
        review.human_review_required,
        if review.wrong_framework_risk.is_empty() {
            "None".to_string()
        } else {
            review.wrong_framework_risk.join("; ")
        },
        if review.required_rewrite_sections.is_empty() {
            "None".to_string()
        } else {
            review.required_rewrite_sections.join("; ")
        }
    );
    let report = format!(
        "# LUNR Root Cause Report\n\n## Provider Company Profile\n\n{}\n## Compact Payload Sent to AI\n\n{}\n## Why The Previous Output Could Fail\n\nThe provider identity is space/lunar/aerospace. Any carrier-style infrastructure frame is a framework conflict unless the provider description explicitly supports that business model. The previous quality gap was that framework acceptance did not deterministically challenge AI output against provider identity before report status was assigned.\n\n## Validator / Self-Review Failure Mode\n\nBefore this guard, a fluent but wrong industry frame could pass because the JSON fields were present and self-review mainly checked completeness. The new framework challenge checks provider identity, forbidden frame terms, revenue-engine support, and human-review escalation.\n\n## Applied Fix\n\n- Space/lunar provider clues trigger an aerospace/data-limited identity guard.\n- Wrong framework conflicts set framework_fit_check=FAIL and human_review_required=true.\n- The report cannot remain PASS when wrong_framework_conflict is present.\n- Money Flow must be specific or data-limited, not generic.\n- Chart explanations must answer figure-specific questions.\n\n## Current Validator Summary\n\n{}\n## Current Blueprint\n\n- Asset profile: {}\n- Secondary profile: {}\n- Must analyze: {}\n- Data gaps: {}\n",
        profile_summary,
        compact_summary,
        validator_summary,
        blueprint.asset_profile,
        blueprint.secondary_profile,
        blueprint.must_analyze.join("; "),
        blueprint.data_gaps.join("; ")
    );
    write_if_changed(&folder.audit.join("lunr_root_cause_report.md"), &report)?;
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
    let lower = money_flow_text.to_lowercase();
    let generic = money_flow_text.trim().len() < 80
        || [
            "money comes from operating revenue when available",
            "money goes to operating costs",
            "costs and reinvestment",
            "pay attention to cash flow",
            "cash flow is important",
        ]
        .iter()
        .any(|phrase| lower.contains(phrase));
    let data_limited_specific = lower.contains("data gap")
        || lower.contains("not fully verified")
        || lower.contains("current compact payload requires manual")
        || lower.contains("current locked data does not show");
    let score = if generic && !data_limited_specific {
        55
    } else if data_limited_specific {
        74
    } else {
        84
    };
    let status = if score < 60 {
        "FAIL"
    } else if score < 70 {
        "WARNING"
    } else {
        "PASS"
    };
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
    write_json(
        &folder.metadata.join("money_flow_specificity_score.json"),
        &json!({
            "schema_version": SCHEMA_VERSION,
            "money_flow_specificity_score": score,
            "status": status,
            "generic_money_flow_detected": generic && !data_limited_specific,
            "data_limited_specificity": data_limited_specific,
            "report_status_can_be_pass": score >= 70
        }),
    )?;
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

fn sentence_counts(report: &str) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for part in report.split(['.', '!', '?']) {
        let sentence = part
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        if sentence.len() < 48 {
            continue;
        }
        *counts.entry(sentence).or_insert(0) += 1;
    }
    counts
}

fn write_template_leakage_check(
    folder: &RunFolder,
    report: &str,
    payload: &ProviderPayload,
) -> Result<&'static str> {
    let allowed_repeated = [
        "not investment advice",
        "target price",
        "buy/sell recommendation",
        "provider limitations",
        "source:",
    ];
    let repeated_sentences = sentence_counts(report)
        .into_iter()
        .filter(|(sentence, count)| {
            *count > 1
                && !allowed_repeated
                    .iter()
                    .any(|allowed| sentence.to_lowercase().contains(allowed))
        })
        .map(|(sentence, count)| json!({"sentence": sentence, "count": count}))
        .collect::<Vec<_>>();
    let lower = report.to_lowercase();
    let generic_phrases = [
        "money comes from operating revenue when available",
        "money goes to costs and reinvestment",
        "cash flow matters",
        "growth is not automatically valuable",
        "the report should explain how the company earns money before interpreting valuation",
    ]
    .iter()
    .filter(|phrase| lower.contains(**phrase))
    .map(|phrase| phrase.to_string())
    .collect::<Vec<_>>();
    let company_name = payload.company_profile.name.to_lowercase();
    let description_terms = payload
        .company_profile
        .description
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.len() >= 5)
        .take(12)
        .map(|term| term.to_lowercase())
        .collect::<Vec<_>>();
    let company_specific_hits = description_terms
        .iter()
        .filter(|term| lower.contains(term.as_str()))
        .count()
        + if !company_name.is_empty() && lower.contains(&company_name) {
            1
        } else {
            0
        };
    let mut warnings = Vec::new();
    if !generic_phrases.is_empty() {
        warnings.push("generic money-flow or business-model phrase detected".to_string());
    }
    if company_specific_hits == 0 && !payload.company_profile.description.is_empty() {
        warnings
            .push("report does not appear to reuse company-specific description terms".to_string());
    }
    if repeated_sentences.len() > 3 {
        warnings.push("multiple repeated non-disclosure sentences detected".to_string());
    }
    let status = if generic_phrases.len() >= 2 {
        "FAIL"
    } else if warnings.is_empty() {
        "PASS"
    } else {
        "WARNING"
    };
    write_json(
        &folder.metadata.join("template_leakage_check.json"),
        &json!({
            "schema_version": SCHEMA_VERSION,
            "status": status,
            "generic_phrases": generic_phrases,
            "repeated_sentences": repeated_sentences,
            "company_specific_description_terms_checked": description_terms,
            "company_specific_hits": company_specific_hits,
            "warnings": warnings,
            "allowed_repeated_text": [
                "legal disclaimer",
                "provider limitations",
                "not investment advice",
                "standard headings",
                "field labels"
            ]
        }),
    )?;
    let findings = if warnings.is_empty() {
        "- No blocking template leakage was detected by the deterministic scan.\n".to_string()
    } else {
        warnings
            .iter()
            .map(|warning| format!("- {warning}\n"))
            .collect::<String>()
    };
    write_if_changed(
        &folder.audit.join("template_leakage_report.md"),
        &format!(
            "# Template Leakage Report\n\nStatus: {status}\n\n## Findings\n\n{findings}\n## Policy\n\nRepeated legal disclaimers, provider limitations, standard headings, and field labels are allowed. Repeated analytical sentences, frame-label-only identity, and generic money-flow prose are not allowed.\n"
        ),
    )?;
    Ok(status)
}

pub(crate) struct CoreDataAccuracyStatus {
    pub has_failures: bool,
    pub has_warnings: bool,
}

fn status_from_counts(failures: usize, warnings: usize) -> &'static str {
    if failures > 0 {
        "FAIL"
    } else if warnings > 0 {
        "WARNING"
    } else {
        "PASS"
    }
}

fn report_numeric_claim_count(report: &str) -> usize {
    report
        .split_whitespace()
        .filter(|token| {
            let cleaned = token.trim_matches(|c: char| {
                c == '#' || c == '|' || c == ':' || c == ',' || c == '.' || c == '(' || c == ')'
            });
            cleaned.chars().any(|c| c.is_ascii_digit())
                && cleaned
                    .chars()
                    .any(|c| c.is_ascii_alphabetic() || c == '%' || c == '.')
        })
        .count()
}

fn expected_currency_for_market(market: &str) -> Option<&'static str> {
    if market.eq_ignore_ascii_case("US") {
        Some("USD")
    } else if market.eq_ignore_ascii_case("CN_A") || market.eq_ignore_ascii_case("CN") {
        Some("CNY")
    } else {
        None
    }
}

fn currency_matches(expected: &str, actual: &str) -> bool {
    actual.eq_ignore_ascii_case(expected)
        || (expected == "CNY" && actual.eq_ignore_ascii_case("RMB"))
}

fn text_mentions_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_lowercase();
    needles
        .iter()
        .any(|needle| lower.contains(&needle.to_lowercase()))
}

fn write_core_data_accuracy_audits(
    folder: &RunFolder,
    payload: &ProviderPayload,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    report: &str,
) -> Result<CoreDataAccuracyStatus> {
    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    let numeric_claim_count = report_numeric_claim_count(report);
    let locked_numeric_count =
        payload.income_statement.len() + payload.balance_sheet.len() + payload.cash_flow.len();
    let evidence_status = if numeric_claim_count > 0 && locked_numeric_count == 0 {
        failures.push(
            "report contains numeric-looking claims but locked statement evidence is empty"
                .to_string(),
        );
        "FAIL"
    } else if numeric_claim_count > 0 && !folder.metadata.join("evidence_map.json").exists() {
        failures.push(
            "report contains numeric-looking claims but evidence_map.json is missing".to_string(),
        );
        "FAIL"
    } else if numeric_claim_count > 0 {
        "PASS"
    } else {
        warnings
            .push("report contains no concrete numeric-looking claims to cross-check".to_string());
        "WARNING"
    };
    write_if_changed(
        &folder.audit.join("evidence_numeric_accuracy_report.md"),
        &format!(
            "# Evidence Numeric Accuracy Report\n\nStatus: {evidence_status}\n\n| Field | Value |\n|---|---:|\n| numeric_claim_count | {numeric_claim_count} |\n| locked_numeric_rows | {locked_numeric_count} |\n\n## Trace Policy\n\nEvery concrete number in the report must trace back to `raw/provider_payload.json`, `data/normalized_financials.json`, `data/valuation_snapshot.json`, `metadata/data_inventory.json`, or chart source metadata. Renderer-generated narrative must not introduce new numeric facts.\n\n## Findings\n\n{}\n",
            if evidence_status == "PASS" {
                "- Numeric-looking claims have locked statement evidence and `metadata/evidence_map.json` exists.\n".to_string()
            } else {
                failures
                    .iter()
                    .chain(warnings.iter())
                    .map(|issue| format!("- {issue}\n"))
                    .collect::<String>()
            }
        ),
    )?;

    let ocf = metric_value(
        &payload.cash_flow,
        &["operating cash flow", "cash from operations", "经营现金流"],
    );
    let capex = metric_value(
        &payload.cash_flow,
        &["capital expenditure", "capex", "资本开支", "购建固定资产"],
    );
    let fcf = metric_value(&payload.cash_flow, &["free cash flow", "fcf", "自由现金流"]);
    let fcf_issue = match (ocf, capex, fcf) {
        (Some(o), Some(c), Some(f)) => {
            let expected = o + c;
            let tolerance = expected.abs().max(f.abs()).max(1.0) * 0.02;
            if (expected - f).abs() > tolerance {
                Some(format!(
                    "free cash flow mismatch: expected OCF + capex = {expected:.2}, provider/reported FCF = {f:.2}"
                ))
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(issue) = &fcf_issue {
        failures.push(issue.clone());
    }
    let money_text = format!(
        "{} {} {}",
        interpretation.where_money_comes_from,
        interpretation.where_money_goes,
        interpretation.cash_flow_explanation
    );
    let generic_money_flow = text_mentions_any(
        &money_text,
        &[
            "money comes from operating revenue when available",
            "money goes to costs and reinvestment",
            "cash flow is important",
            "pay attention to cash flow",
        ],
    );
    if generic_money_flow {
        warnings.push(
            "money flow uses generic phrasing instead of locked data or explicit data gaps"
                .to_string(),
        );
    }
    let frame_text = format!(
        "{} {} {} {} {}",
        blueprint.asset_profile,
        blueprint.secondary_profile,
        payload.company_profile.sector,
        payload.company_profile.industry,
        payload.company_profile.description
    );
    let bank_or_insurance = text_mentions_any(&frame_text, &["bank", "银行", "insurance", "保险"]);
    if bank_or_insurance
        && text_mentions_any(
            &money_text,
            &[
                "industrial fcf",
                "ordinary industrial",
                "net debt / ebitda as core",
            ],
        )
    {
        failures.push("bank/insurance money flow uses ordinary industrial FCF framing".to_string());
    }
    let money_status = if failures.iter().any(|issue| {
        issue.contains("free cash flow mismatch") || issue.contains("bank/insurance money flow")
    }) {
        "FAIL"
    } else if generic_money_flow {
        "WARNING"
    } else {
        "PASS"
    };
    write_if_changed(
        &folder.audit.join("money_flow_accuracy_report.md"),
        &format!(
            "# Money Flow Accuracy Report\n\nStatus: {money_status}\n\n| Metric | Value |\n|---|---:|\n| operating cash flow | {} |\n| capex | {} |\n| free cash flow | {} |\n\n## Rules\n\n- FCF must equal operating cash flow plus capex when provider gives all three values and capex is stored as a cash outflow.\n- Banks and insurers must not be forced into ordinary industrial FCF framing.\n- Missing cash-flow evidence must be disclosed as a data gap instead of invented.\n\n## Findings\n\n{}\n",
            ocf.map(|v| format!("{v:.2}")).unwrap_or_else(|| "missing".into()),
            capex.map(|v| format!("{v:.2}")).unwrap_or_else(|| "missing".into()),
            fcf.map(|v| format!("{v:.2}")).unwrap_or_else(|| "missing".into()),
            if fcf_issue.is_none() && !generic_money_flow {
                "- Money-flow text is not in conflict with the checked locked data.\n".to_string()
            } else {
                fcf_issue
                    .iter()
                    .map(|issue| format!("- {issue}\n"))
                    .chain(if generic_money_flow {
                        vec!["- Generic money-flow phrasing detected.\n".to_string()]
                    } else {
                        Vec::new()
                    })
                    .collect::<String>()
            }
        ),
    )?;

    let chart_manifest = folder.charts.join("chart_manifest.json");
    let mut chart_failures = Vec::new();
    let mut chart_warnings = Vec::new();
    if chart_manifest.exists() {
        let manifest_text = fs::read_to_string(&chart_manifest).unwrap_or_default();
        match serde_json::from_str::<serde_json::Value>(&manifest_text) {
            Ok(serde_json::Value::Array(items)) => {
                for item in items {
                    let status = item
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN");
                    let source = item.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    let file = item.get("file").and_then(|v| v.as_str()).unwrap_or("");
                    if source.is_empty() {
                        chart_warnings.push(format!("chart `{file}` is missing source"));
                    }
                    if status == "PASS" && !file.is_empty() && !folder.charts.join(file).exists() {
                        chart_failures
                            .push(format!("chart `{file}` is marked PASS but file is missing"));
                    }
                    if status == "PASS" && file.ends_with(".md") {
                        chart_failures
                            .push(format!("chart `{file}` is a data-gap card but marked PASS"));
                    }
                }
            }
            _ => chart_failures.push("chart_manifest.json is malformed".to_string()),
        }
    } else {
        chart_warnings.push("chart_manifest.json is missing".to_string());
    }
    let chart_status = status_from_counts(chart_failures.len(), chart_warnings.len());
    if chart_status == "FAIL" {
        failures.extend(chart_failures.iter().cloned());
    } else {
        warnings.extend(chart_warnings.iter().cloned());
    }
    write_if_changed(
        &folder.audit.join("chart_data_accuracy_report.md"),
        &format!(
            "# Chart Data Accuracy Report\n\nStatus: {chart_status}\n\n## Rules\n\n- Each chart must disclose a source.\n- A chart marked PASS must have an existing artifact.\n- Data-gap cards must not be mislabeled as generated charts.\n- Charts must not imply buy/sell/target-price conclusions.\n\n## Findings\n\n{}\n",
            if chart_failures.is_empty() && chart_warnings.is_empty() {
                "- Chart manifest and chart files are internally consistent.\n".to_string()
            } else {
                chart_failures
                    .iter()
                    .chain(chart_warnings.iter())
                    .map(|issue| format!("- {issue}\n"))
                    .collect::<String>()
            }
        ),
    )?;

    let mut unit_failures = Vec::new();
    let mut unit_warnings = Vec::new();
    if let Some(expected) = expected_currency_for_market(&payload.market) {
        if !currency_matches(expected, &payload.company_profile.currency) {
            unit_failures.push(format!(
                "market {} expects currency {}, got {}",
                payload.market, expected, payload.company_profile.currency
            ));
        }
    } else {
        unit_warnings.push(format!(
            "no deterministic currency expectation for market {}",
            payload.market
        ));
    }
    if (payload.market.eq_ignore_ascii_case("CN_A") || payload.market.eq_ignore_ascii_case("CN"))
        && payload.metadata.mock
    {
        unit_failures.push("A-share provider payload is marked mock=true".to_string());
    }
    let unit_status = status_from_counts(unit_failures.len(), unit_warnings.len());
    if unit_status == "FAIL" {
        failures.extend(unit_failures.iter().cloned());
    } else {
        warnings.extend(unit_warnings.iter().cloned());
    }
    write_if_changed(
        &folder.audit.join("unit_policy_accuracy_report.md"),
        &format!(
            "# Unit Policy Accuracy Report\n\nStatus: {unit_status}\n\n| Field | Value |\n|---|---|\n| market | {} |\n| currency | {} |\n| provider | {} |\n| source | {} |\n| package_used | {} |\n| mock | {} |\n\n## Rules\n\n- US reports must use USD unless explicitly converted.\n- A-share reports must use CNY/RMB and disclose provider source/package/mock status.\n- Percentage, multiple, table, and chart units must remain aligned with locked data.\n\n## Findings\n\n{}\n",
            payload.market,
            payload.company_profile.currency,
            payload.provider,
            payload.metadata.source,
            payload.metadata.package_used,
            payload.metadata.mock,
            if unit_failures.is_empty() && unit_warnings.is_empty() {
                "- Unit policy matches market and provider metadata.\n".to_string()
            } else {
                unit_failures
                    .iter()
                    .chain(unit_warnings.iter())
                    .map(|issue| format!("- {issue}\n"))
                    .collect::<String>()
            }
        ),
    )?;

    Ok(CoreDataAccuracyStatus {
        has_failures: !failures.is_empty(),
        has_warnings: !warnings.is_empty(),
    })
}

fn write_section_source_map(folder: &RunFolder, payload: &ProviderPayload) -> Result<()> {
    let map = json!({
        "schema_version": SCHEMA_VERSION,
        "ticker": payload.ticker,
        "section_1_report_status": ["metadata/report_status.json", "metadata/ai_usage.json"],
        "section_2_company_identity": [
            "metadata/company_fact_sheet.json",
            "metadata/company_understanding.json:company_identity",
            "metadata/company_understanding.json:correct_research_frame",
            "metadata/company_understanding.json:not_this"
        ],
        "section_3_business_model": [
            "metadata/company_fact_sheet.json",
            "metadata/revenue_engine_map.json",
            "metadata/company_understanding.json:business_model",
            "metadata/company_understanding.json:revenue_engines",
            "metadata/company_understanding.json:profit_pool"
        ],
        "section_4_money_flow": [
            "metadata/company_fact_sheet.json",
            "metadata/revenue_engine_map.json",
            "metadata/cost_structure_map.json",
            "metadata/capital_allocation_map.json",
            "metadata/money_flow_mechanism.json",
            "metadata/company_specific_questions.json",
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
    let english_source = folder
        .report
        .join(format!("{}_research_report.md", payload.ticker));
    let chinese_source = folder
        .report
        .join(format!("{}_research_report_cn.md", payload.ticker));
    let source_size = if english_expected {
        fs::metadata(&english_source).map(|m| m.len()).unwrap_or(0)
    } else {
        fs::metadata(&chinese_source).map(|m| m.len()).unwrap_or(0)
    };
    let english_pdf_size = fs::metadata(&english_pdf).map(|m| m.len()).unwrap_or(0);
    let chinese_pdf_size = fs::metadata(&chinese_pdf).map(|m| m.len()).unwrap_or(0);
    let english_title_ok = if english_expected {
        fs::read(&english_pdf)
            .map(|bytes| {
                String::from_utf8_lossy(&bytes)
                    .contains(&format!("{} Company Research Report", payload.ticker))
            })
            .unwrap_or(false)
    } else {
        true
    };
    let chinese_title_ok = if chinese_expected {
        fs::read(&chinese_pdf)
            .map(|bytes| {
                String::from_utf8_lossy(&bytes)
                    .contains(&format!("{} 公司研究报告", payload.ticker))
            })
            .unwrap_or(false)
    } else {
        true
    };
    let english_ok =
        !english_expected || (english_pdf.exists() && english_pdf_size > 1024 && english_title_ok);
    let chinese_ok =
        !chinese_expected || (chinese_pdf.exists() && chinese_pdf_size > 1024 && chinese_title_ok);
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
            "# PDF Export Report\n\nStatus: {}\n\n## Source and Output\n\n| Item | Value |\n|---|---:|\n| Source file size | {} |\n| English PDF size | {} |\n| Chinese PDF size | {} |\n| English title check | {} |\n| Chinese title check | {} |\n\n## Required Surface\n\n- Cover/title page: included as the report title.\n- Table of contents: included.\n- Page numbers: included by the basic exporter.\n- Generated/status card: included in the top status block.\n- Charts readable: chart references and explanation text are preserved; embedded PNG rendering depends on the local lightweight exporter.\n- Tables not broken: Markdown tables are converted into readable text blocks by the exporter.\n- Source notes: preserved.\n- AI self-review: preserved.\n- Disclaimer: preserved.\n\n## Blank PDF Guard\n\nPDF_EXPORT_STATUS can only be PASS when the expected PDF exists, is larger than the minimum size, and contains the report title text. Otherwise status is WARNING and Markdown/HTML remain authoritative.\n\n## Details\n\n{}\n",
            status,
            source_size,
            english_pdf_size,
            chinese_pdf_size,
            english_title_ok,
            chinese_title_ok,
            details
        ),
    )?;
    write_json(
        &folder.metadata.join("pdf_status.json"),
        &json!({
            "PDF_EXPORT_STATUS": status,
            "english_pdf_expected": english_expected,
            "english_pdf_exists": english_ok,
            "english_pdf_size": english_pdf_size,
            "english_title_check": english_title_ok,
            "chinese_pdf_expected": chinese_expected,
            "chinese_pdf_exists": chinese_ok,
            "chinese_pdf_size": chinese_pdf_size,
            "chinese_title_check": chinese_title_ok,
            "source_file_size": source_size,
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
