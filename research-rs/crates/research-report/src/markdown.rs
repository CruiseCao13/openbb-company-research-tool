use research_core::types::*;

fn bullet(items: &[String]) -> String {
    if items.is_empty() {
        "- Not available from locked data.\n".to_string()
    } else {
        items.iter().map(|x| format!("- {x}\n")).collect()
    }
}

fn table_rows(items: &[(&str, String, &str, &str)]) -> String {
    items
        .iter()
        .map(|(a, b, c, d)| format!("| {a} | {b} | {c} | {d} |\n"))
        .collect()
}

fn provider_limitations(payload: &ProviderPayload) -> String {
    if payload.metadata.provider_limitations.is_empty() {
        "- None recorded by provider.\n".to_string()
    } else {
        bullet(&payload.metadata.provider_limitations)
    }
}

fn missing_provider_fields(payload: &ProviderPayload) -> String {
    if payload.missing_fields.is_empty() {
        "None recorded by provider.".to_string()
    } else {
        payload.missing_fields.join("; ")
    }
}

fn latest_metric_line(rows: &[StatementRow], label: &str, needles: &[&str]) -> String {
    rows.iter()
        .find(|row| {
            let metric = row.metric.to_lowercase();
            row.value.is_some()
                && needles
                    .iter()
                    .any(|needle| metric.contains(&needle.to_lowercase()))
        })
        .map(|row| {
            format!(
                "- {label}: {} {} = {:.1} {}\n",
                row.metric,
                row.period,
                row.value.unwrap_or_default(),
                row.unit
            )
        })
        .unwrap_or_else(|| format!("- {label}: missing from locked data\n"))
}

fn money_flow_fact_anchors(payload: &ProviderPayload) -> String {
    [
        latest_metric_line(
            &payload.income_statement,
            "Revenue / operating income base",
            &["revenue", "total revenue", "营业收入"],
        ),
        latest_metric_line(
            &payload.cash_flow,
            "Operating cash flow",
            &["operating cash flow", "cash from operations", "经营现金流"],
        ),
        latest_metric_line(
            &payload.cash_flow,
            "Capex / reinvestment",
            &["capital expenditure", "capex", "资本开支", "购建固定资产"],
        ),
        latest_metric_line(
            &payload.cash_flow,
            "Free cash flow",
            &["free cash flow", "fcf", "自由现金流"],
        ),
        latest_metric_line(
            &payload.balance_sheet,
            "Debt / balance-sheet pressure",
            &["debt", "borrowings", "有息负债", "负债合计"],
        ),
        latest_metric_line(
            &payload.balance_sheet,
            "Working-capital check",
            &["inventory", "receivable", "存货", "应收"],
        ),
    ]
    .join("")
}

fn has_metric(rows: &[StatementRow], needles: &[&str]) -> bool {
    rows.iter().any(|row| {
        let metric = row.metric.to_lowercase();
        row.value.is_some()
            && needles
                .iter()
                .any(|needle| metric.contains(&needle.to_lowercase()))
    })
}

fn framework_coverage_rows(payload: &ProviderPayload) -> String {
    let has_profile = !payload.company_profile.name.is_empty()
        || !payload.company_profile.description.is_empty()
        || !payload.company_profile.industry.is_empty();
    let has_revenue = has_metric(
        &payload.income_statement,
        &["revenue", "total revenue", "营业收入"],
    );
    let has_gross = has_metric(
        &payload.income_statement,
        &["gross profit", "毛利率", "毛利"],
    );
    let has_operating_profit = has_metric(
        &payload.income_statement,
        &["operating income", "operating profit", "营业利润"],
    );
    let has_net_profit = has_metric(
        &payload.income_statement,
        &["net income", "归母净利润", "净利润"],
    );
    let has_ocf = has_metric(
        &payload.cash_flow,
        &["operating cash flow", "cash from operations", "经营现金"],
    );
    let has_capex = has_metric(
        &payload.cash_flow,
        &["capex", "capital expenditure", "资本开支"],
    );
    let has_fcf = has_metric(&payload.cash_flow, &["free cash flow", "fcf", "自由现金流"]);
    let has_balance = has_metric(
        &payload.balance_sheet,
        &["cash", "debt", "存货", "应收", "负债"],
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
    let rows = [
        (
            "Business Model",
            if has_profile { "PASS" } else { "DATA_GAP" },
            if has_profile {
                "profile and company_fact_sheet"
            } else {
                "company profile"
            },
        ),
        (
            "Revenue Growth",
            if has_revenue { "WARNING" } else { "DATA_GAP" },
            "YoY/QoQ/guidance/expectation checks",
        ),
        (
            "Gross Margin",
            if has_gross { "WARNING" } else { "DATA_GAP" },
            "pricing power, cost control, product mix",
        ),
        (
            "Operating Profit",
            if has_operating_profit {
                "WARNING"
            } else {
                "DATA_GAP"
            },
            "operating leverage and expense ratios",
        ),
        (
            "Net Profit",
            if has_net_profit {
                "WARNING"
            } else {
                "DATA_GAP"
            },
            "EPS, dilution, one-time items",
        ),
        (
            "Cash Flow",
            if has_ocf && has_capex && has_fcf {
                "PASS"
            } else if has_ocf {
                "WARNING"
            } else {
                "DATA_GAP"
            },
            "OCF/capex/FCF/financing need",
        ),
        (
            "Balance Sheet",
            if has_balance { "WARNING" } else { "DATA_GAP" },
            "current assets/liabilities and goodwill",
        ),
        (
            "Key Business Metrics",
            "DATA_GAP",
            "sector KPIs and KPI-to-cash-flow link",
        ),
        ("Guidance", "DATA_GAP", "revenue, margin, full-year outlook"),
        (
            "Market Expectations",
            "DATA_GAP",
            "actual vs expectation and revisions",
        ),
        (
            "Valuation",
            if has_valuation { "WARNING" } else { "DATA_GAP" },
            "implied growth, margin of safety, downside if growth misses",
        ),
    ];
    rows.iter()
        .map(|(section, status, missing)| format!("| {section} | {status} | {missing} |\n"))
        .collect()
}

fn latest_value_for_fields(payload: &ProviderPayload, fields: &[&str]) -> String {
    for field in fields {
        let value = match *field {
            "price_history.close" => payload.price_history.iter().rev().find_map(|point| {
                point
                    .close
                    .map(|close| format!("{} close = {:.2}", point.date, close))
            }),
            "income_statement.revenue" => payload.income_statement.iter().find_map(|row| {
                let metric = row.metric.to_lowercase();
                if metric.contains("revenue") || metric.contains("营业收入") {
                    row.value.map(|value| {
                        format!("{} {} = {:.1} {}", row.period, row.metric, value, row.unit)
                    })
                } else {
                    None
                }
            }),
            "cash_flow.operating_cash_flow" => payload.cash_flow.iter().find_map(|row| {
                let metric = row.metric.to_lowercase();
                if metric.contains("operating cash") || metric.contains("经营现金") {
                    row.value.map(|value| {
                        format!("{} {} = {:.1} {}", row.period, row.metric, value, row.unit)
                    })
                } else {
                    None
                }
            }),
            "cash_flow.capex" => payload.cash_flow.iter().find_map(|row| {
                let metric = row.metric.to_lowercase();
                if metric.contains("capex") || metric.contains("capital expenditure") {
                    row.value.map(|value| {
                        format!("{} {} = {:.1} {}", row.period, row.metric, value, row.unit)
                    })
                } else {
                    None
                }
            }),
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
    "locked field is missing or data-limited".to_string()
}

fn chart_company_observation(
    payload: &ProviderPayload,
    figure: usize,
    title: &str,
    fields: &[&str],
    frame: &str,
) -> (String, String, String, String) {
    let ticker = &payload.ticker;
    let latest = latest_value_for_fields(payload, fields);
    let field_list = fields.join(", ");
    match figure {
        1 => (
            format!("{title} uses {field_list} for {ticker}; latest locked observation: {latest}. For the {frame} frame, price path is only context for opportunity cost and volatility, not proof of business quality."),
            format!("It can support a market-path discussion for {ticker}'s {frame} case when paired with drawdown and operating evidence."),
            format!("It cannot prove valuation, business quality, or whether {ticker}'s research frame is correct."),
            format!("Check whether {ticker}'s revenue, cash flow, and drawdown explain the same period rather than reading the price line alone."),
        ),
        2 => (
            format!("{title} uses {field_list} for {ticker}; latest locked observation: {latest}. For the {frame} frame, drawdown describes path risk and financing pressure context."),
            format!("It can support a risk-path discussion for {ticker}'s {frame} case, especially where volatility affects financing, confidence, or patience."),
            format!("It cannot prove {ticker}'s solvency, product-market fit, credit quality, or permanent impairment by itself."),
            format!("Compare {ticker}'s drawdown periods with cash-flow, debt, provider gaps, and frame-specific milestones."),
        ),
        3 => (
            format!("{title} uses {field_list} for {ticker}; latest locked observation: {latest}. For the {frame} frame, the question is whether reported scale connects to the company-specific revenue engine."),
            format!("It can support a first-pass revenue or financial progression claim for {ticker} when the source field is available."),
            format!("It cannot prove {ticker}'s segment mix, customer concentration, pricing power, or margin durability unless those facts are present."),
            format!("Verify {ticker}'s segment drivers, margin bridge, and one-off items against filings or provider detail."),
        ),
        4 => (
            format!("{title} uses {field_list} for {ticker}; latest locked observation: {latest}. For the {frame} frame, the chart should be read through OCF, capex, FCF, working capital, and financing need."),
            format!("It can support whether {ticker} appears self-funding or cash-consuming within locked data limits for the {frame} frame."),
            format!("It cannot prove {ticker}'s future cash runway, contract durability, bank capital quality, insurance solvency, or shareholder-return capacity without the missing supporting fields."),
            format!("Reconcile {ticker}'s OCF, capex, debt, inventory/receivables, and data gaps before making a money-flow conclusion."),
        ),
        _ => (
            format!("{title} uses {field_list} for {ticker}; latest locked observation: {latest}. For the {frame} frame, valuation evidence must match the company's profitability stage and asset type."),
            format!("It can support which valuation lens is available for {ticker}'s {frame} frame."),
            format!("It cannot create an investment recommendation or fair-value conclusion for {ticker} on its own."),
            format!("Use {ticker}'s blueprint valuation method, then verify missing inputs before treating any multiple as meaningful."),
        ),
    }
}

fn chart_block(
    payload: &ProviderPayload,
    figure: usize,
    title: &str,
    file: &str,
    status: &str,
    fields: &[&str],
    frame: &str,
) -> String {
    let link = if file.ends_with(".png") {
        format!("![Figure {figure}. {title}](../charts/{file})")
    } else {
        format!("[Figure {figure}. {title}](../charts/{file})")
    };
    let (observation, supports, cannot_prove, next_check) =
        chart_company_observation(payload, figure, title, fields, frame);
    format!(
        r#"### Figure {figure}. {title}

{link}

Source: provider_payload.json  
Status: {status}

Company-specific observation:
{observation}

What this chart can support:
{supports}

What not to overread:
{cannot_prove}

Next check:
{next_check}

"#
    )
}

pub fn render_report(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    review: &AiSelfReview,
    status: &ReportStatus,
) -> String {
    let name = if payload.company_profile.name.is_empty() {
        payload.ticker.clone()
    } else {
        payload.company_profile.name.clone()
    };
    let chart_manifest = [
        chart_block(
            payload,
            1,
            "Price / Benchmark Performance",
            "Figure_01_price_vs_benchmark.png",
            "PASS or DATA_GAP",
            &["price_history.close"],
            &blueprint.asset_profile,
        ),
        chart_block(
            payload,
            2,
            "Drawdown / Risk Path",
            "Figure_02_drawdown.png",
            "PASS or DATA_GAP",
            &["price_history.close"],
            &blueprint.asset_profile,
        ),
        chart_block(
            payload,
            3,
            "Financial Trend",
            "Figure_03_financial_trend.png",
            "PASS or DATA_GAP",
            &["income_statement.revenue"],
            &blueprint.asset_profile,
        ),
        chart_block(
            payload,
            4,
            "Money Flow / Cash Flow Bridge",
            "Figure_04_money_flow.png",
            "PASS or DATA_GAP",
            &["cash_flow.operating_cash_flow", "cash_flow.capex"],
            &blueprint.asset_profile,
        ),
        chart_block(
            payload,
            5,
            "Valuation Frame",
            "Figure_05_valuation_frame.png",
            "PASS or DATA_GAP",
            &["valuation_snapshot"],
            &blueprint.asset_profile,
        ),
    ]
    .join("\n");
    let financial_snapshot = table_rows(&[
        (
            "Research frame",
            blueprint.asset_profile.clone(),
            "frame",
            "Controls which metrics matter",
        ),
        (
            "Money source",
            interpretation.where_money_comes_from.clone(),
            "text",
            "Shows whether operations or financing matter",
        ),
        (
            "Money use",
            interpretation.where_money_goes.clone(),
            "text",
            "Shows reinvestment and cash pressure",
        ),
    ]);
    let money_flow_fact_anchors = money_flow_fact_anchors(payload);
    let framework_coverage_rows = framework_coverage_rows(payload);
    let company_specific_question = blueprint.key_questions.first().cloned().unwrap_or_else(|| {
        format!(
            "Which missing field would most change the money-flow read for {}?",
            payload.ticker
        )
    });
    format!(
        r#"# {ticker} Company Research Report

> Version: v5.0
> Company: {name}
> Market: {market}
> Provider: {provider}
> Provider Source: {provider_source}
> Provider Adapter: {provider_adapter}
> Provider Package Used: {provider_package_used}
> Provider Mock: {provider_mock}
> Status: {status_value}
> AI Confidence: {confidence:?}
> AI Source: {ai_source}
> External AI Used: {external_ai_used}
> Local Mock Used: {local_mock_used}
> AI Calls: {new_external_ai_calls}
> Cache Hits: {ai_cache_hits}
> Model: {ai_model}
> Prompt Versions: {prompt_version}
> Research Frame: {asset_profile}  
> Human Review Required: {human_review}  
> Note: This report is for first-pass research only. It is not investment advice.

## Table of Contents

1. Report Status
2. Company Identity
3. Business Model
4. Money Flow: Where Money Comes From and Where It Goes
5. Financial Statement Interpretation
6. AI Research Blueprint
7. Valuation Frame
8. Risks and Red Flags
9. Data Gaps and Unsupported Claims
10. AI Self Review
11. Next Checks
12. Charts and Evidence
13. Financial Report Framework Coverage
14. Appendix: Locked Data

## 1. Report Status

| Item | Value |
|---|---|
| Overall status | {status_value} |
| Provider status | {provider_status} |
| Provider used | {provider} |
| Provider source | {provider_source} |
| Provider adapter | {provider_adapter} |
| Provider package used | {provider_package_used} |
| Provider mock | {provider_mock} |
| Missing provider fields | {missing_provider_fields} |
| Visual lint | {visual_lint_status} |
| PDF export | {pdf_export_status} |
| AI mode | {ai_mode} |
| AI source | {ai_source} |
| External AI used | {external_ai_used} |
| Local mock used | {local_mock_used} |
| New external AI calls | {new_external_ai_calls} |
| AI calls | {ai_calls} |
| Cache hits | {cache_hits} |
| Model | {ai_model} |
| Prompt versions | {prompt_version} |
| Human review required | {human_review} |

The status separates locked data availability from interpretation confidence. A warning means the report can be useful as a screening memo, but the unsupported sections need human review.

Table 1. Research status snapshot  
Unit: status / text  
Source: metadata/report_status.json  
How to read this table: use it to decide whether this report is usable as a first-pass memo or needs manual review.

Provider limitations:

{provider_limitations}

## 2. Company Identity

**Identity:** {identity}

**Correct research frame:** {frame}

**What this company is not:**  
{not_this}

## 3. Business Model

{business_model}

Revenue engines currently identified:

{revenue_engines}

Profit pool:

{profit_pool}

## 4. Money Flow: Where Money Comes From and Where It Goes

Table 2. Money flow summary  
Unit: text  
Source: provider_payload.json, metadata/company_fact_sheet.json, metadata/revenue_engine_map.json, metadata/cost_structure_map.json, metadata/capital_allocation_map.json, metadata/money_flow_mechanism.json, and financial_interpretation.json
How to read this table: each row links a money-flow signal to why it matters.

| Flow | Signal | Unit | Why it matters |
|---|---|---|---|
{financial_snapshot}

**Where money comes from:** {money_from}

**Where money goes:** {money_goes}

Company-specific locked-data anchors:

{money_flow_fact_anchors}

Company-specific question:

- {company_specific_question}

Money-flow interpretation must stay inside these anchors and explicit data gaps. If a revenue engine, cost item, shareholder return, or financing claim is not in the fact-map artifacts, treat it as a manual check rather than a conclusion.

## 5. Financial Statement Interpretation

**Revenue:** {revenue_explanation}

**Margins:** {margin_explanation}

**Cash flow:** {cash_flow_explanation}

**Capex / R&D pressure:** {capex_rnd}

**Debt and financing:** {debt}

**Shareholder return quality:** {shareholder}

## 6. AI Research Blueprint

**Core thesis:** {core_thesis}

**Asset profile:** {asset_profile}

**Secondary profile:** {secondary_profile}

Must analyze:

{must_analyze}

Must not analyze as core:

{must_not}

Key questions:

{questions}

## 7. Valuation Frame

{valuation}

The report does not provide a target price, buy/sell recommendation, or short-term price prediction.

## 8. Risks and Red Flags

{red_flags}

## 9. Data Gaps and Unsupported Claims

Data gaps:

{data_gaps}

Unsupported claims flagged by AI self-review:

{unsupported}

## 10. AI Self Review

| Check | Status |
|---|---|
| Company understanding | {cu_check:?} |
| Framework fit | {ff_check:?} |
| Numeric consistency | {num_check:?} |
| Money flow | {money_check:?} |
| Final confidence | {confidence:?} |

Wrong-framework risks:

{wrong_risk}

## 11. Next Checks

{next_checks}

## 12. Charts and Evidence

{chart_manifest}

## 13. Financial Report Framework Coverage

Core question: can this company consistently generate cash flow?

Detailed audit: `audit/financial_report_framework_coverage.md`

| Framework section | Status | Major missing items / next human checks |
|---|---|---|
{framework_coverage_rows}

Guidance and market expectations are marked as data gaps unless locked provider data explicitly supports them. Valuation coverage is limited to method fit and missing inputs; it does not create an investment recommendation.

## 14. Appendix: Locked Data

Table 3. Locked data coverage  
Unit: count / text  
Source: raw/provider_payload.json  
How to read this table: it tells you which locked data exists before relying on interpretation.

| Field | Value |
|---|---|
| Ticker | {ticker} |
| Sector | {sector} |
| Industry | {industry} |
| Currency | {currency} |
| Provider source | {provider_source} |
| Provider adapter | {provider_adapter} |
| Provider package used | {provider_package_used} |
| Provider mock | {provider_mock} |
| Missing provider fields | {missing_provider_fields} |
| Price points | {price_count} |
| Income rows | {income_count} |
| Balance sheet rows | {balance_count} |
| Cash-flow rows | {cash_count} |

"#,
        ticker = payload.ticker,
        name = name,
        market = payload.market,
        provider = payload.provider,
        provider_source = payload.metadata.source,
        provider_adapter = payload.metadata.provider_adapter,
        provider_package_used = payload.metadata.package_used,
        provider_mock = payload.metadata.mock,
        provider_limitations = provider_limitations(payload),
        missing_provider_fields = missing_provider_fields(payload),
        status_value = status.overall_status,
        provider_status = status.provider_status,
        visual_lint_status = status.visual_lint_status,
        pdf_export_status = status.pdf_export_status,
        ai_mode = status.ai_mode,
        ai_source = understanding.ai_provenance.source,
        external_ai_used = understanding.ai_provenance.external_ai_used,
        local_mock_used = understanding.ai_provenance.local_mock_used,
        new_external_ai_calls = status.ai_calls,
        ai_cache_hits = status.cache_hits,
        ai_model = understanding.ai_provenance.model,
        prompt_version = understanding.ai_provenance.prompt_version,
        ai_calls = status.ai_calls,
        cache_hits = status.cache_hits,
        human_review = status.human_review_required,
        identity = understanding.company_identity,
        frame = understanding.correct_research_frame,
        not_this = bullet(&understanding.not_this),
        business_model = understanding.business_model,
        revenue_engines = bullet(&understanding.revenue_engines),
        profit_pool = understanding.profit_pool,
        money_from = interpretation.where_money_comes_from,
        money_goes = interpretation.where_money_goes,
        revenue_explanation = interpretation.revenue_explanation,
        margin_explanation = interpretation.margin_explanation,
        cash_flow_explanation = interpretation.cash_flow_explanation,
        capex_rnd = interpretation.capex_or_rnd_pressure,
        debt = interpretation.debt_and_financing,
        shareholder = interpretation.shareholder_return_quality,
        core_thesis = blueprint.core_thesis,
        asset_profile = blueprint.asset_profile,
        secondary_profile = blueprint.secondary_profile,
        must_analyze = bullet(&blueprint.must_analyze),
        must_not = bullet(&blueprint.must_not_analyze_as_core),
        questions = bullet(&blueprint.key_questions),
        valuation = blueprint.valuation_frame,
        red_flags = bullet(&blueprint.red_flags),
        data_gaps = bullet(&blueprint.data_gaps),
        unsupported = bullet(&review.unsupported_claims),
        cu_check = review.company_understanding_check,
        ff_check = review.framework_fit_check,
        num_check = review.numeric_consistency_check,
        money_check = review.money_flow_check,
        confidence = review.final_confidence,
        wrong_risk = bullet(&review.wrong_framework_risk),
        next_checks = bullet(&blueprint.next_checks),
        sector = payload.company_profile.sector,
        industry = payload.company_profile.industry,
        currency = payload.company_profile.currency,
        price_count = payload.price_history.len(),
        income_count = payload.income_statement.len(),
        balance_count = payload.balance_sheet.len(),
        cash_count = payload.cash_flow.len(),
        framework_coverage_rows = framework_coverage_rows,
    )
}

pub fn render_report_zh(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    review: &AiSelfReview,
    status: &ReportStatus,
) -> String {
    let name = if payload.company_profile.name.is_empty() {
        payload.ticker.clone()
    } else {
        payload.company_profile.name.clone()
    };
    let framework_coverage_rows = framework_coverage_rows(payload);
    format!(
        r#"# {ticker} 公司研究报告

> 状态：{status_value}
> Provider：{provider}
> Provider Source：{provider_source}
> Provider Adapter：{provider_adapter}
> Provider Package Used：{provider_package_used}
> Provider Mock：{provider_mock}
> AI 置信度：{confidence:?}
> AI 来源：{ai_source}
> 是否使用外部 OpenAI API：{external_ai_used}
> 是否使用本地 fallback：{local_mock_used}
> 新外部 AI 调用：{new_external_ai_calls}
> AI 缓存命中：{ai_cache_hits}
> 模型：{ai_model}
> Prompt 版本：{prompt_version}
> 研究框架：{asset_profile}  
> 是否需要人工复核：{human_review}  
> 生成说明：本报告不是投资建议。

## 目录

1. 报告状态
2. 公司身份
3. 商业模式
4. 资金流向：钱从哪里来，去了哪里
5. 财报解释
6. AI 研究蓝图
7. 估值框架
8. 风险与红旗
9. 数据缺口与未验证判断
10. AI 自我复核
11. 下一步核查
12. 图表与证据
13. 财报阅读框架覆盖情况
14. 附录：锁定数据

## 1. 报告状态

Table 1. 报告状态快照  
单位：状态 / 文本  
来源：metadata/report_status.json  
How to read this table：先看是否需要人工复核，再看研究框架是否可信。

| 项目 | 内容 |
|---|---|
| 公司 | {name} |
| 总体状态 | {status_value} |
| 数据源状态 | {provider_status} |
| Provider used | {provider} |
| Provider source | {provider_source} |
| Provider adapter | {provider_adapter} |
| Provider package used | {provider_package_used} |
| Provider mock | {provider_mock} |
| Missing provider fields | {missing_provider_fields} |
| 视觉检查 | {visual_lint_status} |
| PDF 导出 | {pdf_export_status} |
| AI 模式 | {ai_mode} |
| AI 来源 | {ai_source} |
| 使用外部 OpenAI API | {external_ai_used} |
| 使用本地 fallback | {local_mock_used} |
| 新外部 AI 调用 | {new_external_ai_calls} |
| AI 缓存命中 | {ai_cache_hits} |
| 模型 | {ai_model} |
| Prompt 版本 | {prompt_version} |

## 2. 公司身份

{identity}

它不应该被当成：

{not_this}

## 3. 商业模式

{business_model}

收入来源：

{revenue_engines}

## 4. 资金流向：钱从哪里来，去了哪里

Table 2. 资金流摘要  
单位：文本  
来源：provider_payload.json 和 financial_interpretation.json  
How to read this table：看经营造血、再投资、融资压力是否匹配公司画像。

| 流向 | 信号 | 为什么重要 |
|---|---|---|
| 钱从哪里来 | {money_from} | 判断增长是否靠经营造血 |
| 钱去了哪里 | {money_goes} | 判断现金是否被 capex、研发、债务或回报股东消耗 |

## 5. 财报解释

收入：{revenue_explanation}

利润率：{margin_explanation}

现金流：{cash_flow_explanation}

## 6. AI 研究蓝图

核心主线：{core_thesis}

必须核查：

{must_analyze}

不能作为核心框架：

{must_not}

## 7. 估值框架

{valuation}

本报告不给目标价，不给买卖建议，也不预测短期股价。

## 8. 风险与红旗

{red_flags}

## 9. 数据缺口与未验证判断

{data_gaps}

## 10. AI 自我复核

| 检查 | 状态 |
|---|---|
| 公司理解 | {cu_check:?} |
| 框架匹配 | {ff_check:?} |
| 数字一致性 | {num_check:?} |
| 资金流 | {money_check:?} |

## 11. 下一步核查

{next_checks}

## 12. 图表与证据

### Figure 1. 价格 / 基准表现

![Figure 1](../charts/Figure_01_price_vs_benchmark.png)

来源：provider_payload.json

What to look at：看价格路径和时间范围。  
What it means：它能帮助判断持有过程中的价格表现。  
What not to overread：它不能证明未来会继续上涨或跑赢。  
Next check：结合估值、现金流和业务数据复核。

### Figure 2. 回撤 / 风险路径

![Figure 2](../charts/Figure_02_drawdown.png)

来源：provider_payload.json

What to look at：看从阶段高点跌下来的幅度。  
What it means：它反映持有过程中的账户压力。  
What not to overread：回撤不是破产风险，也不是买卖信号。  
Next check：结合基本面恶化或市场周期复核。

### Figure 3. 财务趋势

![Figure 3](../charts/Figure_03_financial_trend.png)

来源：provider_payload.json

What to look at：看收入或行业适配指标的方向。  
What it means：它能提示增长或周期位置。  
What not to overread：单一趋势不能替代完整财报分析。  
Next check：查最新年报和分部数据。

### Figure 4. 资金流 / 现金流桥

![Figure 4](../charts/Figure_04_money_flow.png)

来源：provider_payload.json

What to look at：看经营现金流、资本开支和融资压力。  
What it means：它帮助判断业务是否造血。  
What not to overread：缺少项目时不能强行推断。  
Next check：核查现金流量表明细。

### Figure 5. 估值框架

![Figure 5](../charts/Figure_05_valuation_frame.png)

来源：provider_payload.json

What to look at：看当前可用的估值入口。  
What it means：它只说明市场如何定价部分指标。  
What not to overread：估值倍数不是目标价。  
Next check：确认该倍数是否适合当前资产类型。

## 13. 财报阅读框架覆盖情况

核心问题：一家公司是否具备持续创造现金流的能力？

详细审计文件：`audit/financial_report_framework_coverage.md`

| 框架部分 | 状态 | 主要缺口 / 下一步人工核查 |
|---|---|---|
{framework_coverage_rows}

未来指引和市场预期如果没有锁定数据支持，必须标为数据缺口。估值只讨论方法适配和缺失输入，不生成投资建议。

## 14. 附录：锁定数据

Table 3. 锁定数据覆盖  
单位：条数 / 文本  
来源：raw/provider_payload.json  
How to read this table：先看数据覆盖，再决定解释可信度。

| 字段 | 内容 |
|---|---|
| Ticker | {ticker} |
| Sector | {sector} |
| Industry | {industry} |
| Currency | {currency} |
| Provider source | {provider_source} |
| Provider adapter | {provider_adapter} |
| Provider package used | {provider_package_used} |
| Provider mock | {provider_mock} |
| Missing provider fields | {missing_provider_fields} |
| Price rows | {price_count} |

"#,
        ticker = payload.ticker,
        name = name,
        provider = payload.provider,
        provider_source = payload.metadata.source,
        provider_adapter = payload.metadata.provider_adapter,
        provider_package_used = payload.metadata.package_used,
        provider_mock = payload.metadata.mock,
        missing_provider_fields = missing_provider_fields(payload),
        status_value = status.overall_status,
        provider_status = status.provider_status,
        visual_lint_status = status.visual_lint_status,
        pdf_export_status = status.pdf_export_status,
        ai_mode = status.ai_mode,
        human_review = if status.human_review_required {
            "是"
        } else {
            "否"
        },
        confidence = blueprint.confidence,
        ai_source = understanding.ai_provenance.source,
        external_ai_used = understanding.ai_provenance.external_ai_used,
        local_mock_used = understanding.ai_provenance.local_mock_used,
        new_external_ai_calls = status.ai_calls,
        ai_cache_hits = status.cache_hits,
        ai_model = understanding.ai_provenance.model,
        prompt_version = understanding.ai_provenance.prompt_version,
        asset_profile = blueprint.asset_profile,
        identity = understanding.company_identity,
        not_this = bullet(&understanding.not_this),
        business_model = understanding.business_model,
        revenue_engines = bullet(&understanding.revenue_engines),
        money_from = interpretation.where_money_comes_from,
        money_goes = interpretation.where_money_goes,
        revenue_explanation = interpretation.revenue_explanation,
        margin_explanation = interpretation.margin_explanation,
        cash_flow_explanation = interpretation.cash_flow_explanation,
        core_thesis = blueprint.core_thesis,
        must_analyze = bullet(&blueprint.must_analyze),
        must_not = bullet(&blueprint.must_not_analyze_as_core),
        valuation = blueprint.valuation_frame,
        red_flags = bullet(&blueprint.red_flags),
        data_gaps = bullet(&blueprint.data_gaps),
        cu_check = review.company_understanding_check,
        ff_check = review.framework_fit_check,
        num_check = review.numeric_consistency_check,
        money_check = review.money_flow_check,
        next_checks = bullet(&blueprint.next_checks),
        sector = payload.company_profile.sector,
        industry = payload.company_profile.industry,
        currency = payload.company_profile.currency,
        price_count = payload.price_history.len(),
        framework_coverage_rows = framework_coverage_rows,
    )
}

pub fn render_self_review_md(review: &AiSelfReview) -> String {
    format!(
        "# AI Self Review\n\n| Check | Status |\n|---|---|\n| Company understanding | {:?} |\n| Framework fit | {:?} |\n| Numeric consistency | {:?} |\n| Money flow | {:?} |\n| Final confidence | {:?} |\n| Human review required | {} |\n\n## Unsupported Claims\n\n{}\n\n## Wrong-Framework Risk\n\n{}\n",
        review.company_understanding_check,
        review.framework_fit_check,
        review.numeric_consistency_check,
        review.money_flow_check,
        review.final_confidence,
        review.human_review_required,
        bullet(&review.unsupported_claims),
        bullet(&review.wrong_framework_risk),
    )
}
