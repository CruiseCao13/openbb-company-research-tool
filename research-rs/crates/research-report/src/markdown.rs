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

fn chart_block(figure: usize, title: &str, file: &str, status: &str) -> String {
    let link = if file.ends_with(".png") {
        format!("![Figure {figure}. {title}](../charts/{file})")
    } else {
        format!("[Figure {figure}. {title}](../charts/{file})")
    };
    let (look_at, meaning, not_overread, next_check) = match figure {
        1 => (
            "Compare the company price path with the benchmark over the stated period, using the index level rather than the ending price alone.",
            "This can show whether the stock has created relative price outperformance or lagged the opportunity-cost benchmark.",
            "A price chart cannot prove the stock is cheap, cannot validate the business model, and cannot replace company-specific cash-flow work.",
            "Check whether the same period also shows drawdown, revenue, margin, or cash-flow evidence that explains the price path.",
        ),
        2 => (
            "Look for the depth and duration of drawdowns from prior highs.",
            "This helps separate smooth compounding from a return path that depends on tolerating large interim losses.",
            "Drawdown does not prove solvency risk or business failure; it only describes historical market path risk.",
            "Compare drawdowns with financing needs, debt maturity, and business milestones before treating market stress as fundamental stress.",
        ),
        3 => (
            "Read revenue, operating profit, free cash flow, and margin together instead of treating revenue growth as enough.",
            "The useful question is whether growth is converting into operating profit and cash generation.",
            "A financial trend chart cannot prove segment quality, customer concentration, or management guidance accuracy when those data are missing.",
            "Verify segment drivers, gross margin bridge, and one-off items in the filing or provider source.",
        ),
        4 => (
            "Focus on operating cash flow, capital spending, financing flows, dividends, and buybacks when available.",
            "This is the evidence path for whether the business funds itself or depends on external capital.",
            "A cash-flow bridge cannot prove future runway without backlog, debt terms, and committed spending data.",
            "Reconcile cash-flow rows with debt, share issuance, working capital, and project or R&D spending.",
        ),
        _ => (
            "Check which valuation metric is actually present and whether it fits the company profile.",
            "The chart is useful only when the chosen metric matches the business model and profitability stage.",
            "A multiple does not prove fair value by itself, and negative or non-meaningful multiples should not be normalized into a bullish or bearish claim.",
            "Use the valuation method named in the research blueprint, then verify the missing data that would make that method meaningful.",
        ),
    };
    format!(
        r#"### Figure {figure}. {title}

{link}

Source: provider_payload.json  
Status: {status}

What to look at:
{look_at}

What it means:
{meaning}

What not to overread:
{not_overread}

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
            1,
            "Price / Benchmark Performance",
            "Figure_01_price_vs_benchmark.png",
            "PASS or DATA_GAP",
        ),
        chart_block(
            2,
            "Drawdown / Risk Path",
            "Figure_02_drawdown.png",
            "PASS or DATA_GAP",
        ),
        chart_block(
            3,
            "Financial Trend",
            "Figure_03_financial_trend.png",
            "PASS or DATA_GAP",
        ),
        chart_block(
            4,
            "Money Flow / Cash Flow Bridge",
            "Figure_04_money_flow.png",
            "PASS or DATA_GAP",
        ),
        chart_block(
            5,
            "Valuation Frame",
            "Figure_05_valuation_frame.png",
            "PASS or DATA_GAP",
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
    format!(
        r#"# {ticker} Company Research Report

> Version: v5.0  
> Company: {name}  
> Market: {market}  
> Provider: {provider}  
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
13. Appendix: Locked Data

## 1. Report Status

| Item | Value |
|---|---|
| Overall status | {status_value} |
| Provider status | {provider_status} |
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
Source: provider_payload.json and financial_interpretation.json  
How to read this table: each row links a money-flow signal to why it matters.

| Flow | Signal | Unit | Why it matters |
|---|---|---|---|
{financial_snapshot}

**Where money comes from:** {money_from}

**Where money goes:** {money_goes}

This matters because growth is not automatically valuable. The report needs to distinguish operating cash generation from financing, reinvestment, R&D, capex, working capital, buybacks, and debt service.

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

## 13. Appendix: Locked Data

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
| Price points | {price_count} |
| Income rows | {income_count} |
| Balance sheet rows | {balance_count} |
| Cash-flow rows | {cash_count} |

"#,
        ticker = payload.ticker,
        name = name,
        market = payload.market,
        provider = payload.provider,
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
    format!(
        r#"# {ticker} 公司研究报告

> 状态：{status_value}  
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
13. 附录：锁定数据

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

## 13. 附录：锁定数据

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
| Price rows | {price_count} |

"#,
        ticker = payload.ticker,
        name = name,
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
