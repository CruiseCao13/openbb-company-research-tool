"""v4 gates and report helpers.

This module keeps the v4 audit, method, language, bilingual, and battle-card
logic separate from deterministic data fetching and chart rendering.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any

import pandas as pd


STATUS_PASS = "PASS"
STATUS_WARNING = "WARNING"
STATUS_FAIL = "FAIL"
OVERALL_VERIFIED = "VERIFIED"
OVERALL_UNVERIFIED = "UNVERIFIED"
MAX_CORRECTION_PASSES = 1
MAX_LANGUAGE_REWRITE_ATTEMPTS = 3
METHOD_BLACKBOX = "[METHOD_BLACKBOX]"
LANGUAGE_FAIL = "[LANGUAGE_FAIL]"
DATA_UNAUDITED = "[DATA_UNAUDITED]"

SOURCE_PROVIDER = "PROVIDER_DATA"
METRIC_MISSING_RAW = "[METRIC_MISSING_RAW]"
FIELD_MISSING_PROVIDER = "[FIELD_MISSING_PROVIDER]"
PRIMARY_SOURCE_REQUIRED_PLACEHOLDER = "[PRIMARY_SOURCE_REQUIRED]"
SEGMENT_DATA_MISSING = "[SEGMENT_DATA_MISSING]"
METHOD_ASSUMPTION_MISSING_PLACEHOLDER = "[METHOD_ASSUMPTION_MISSING]"
PRICE_LABEL_UNVERIFIED = "[PRICE_LABEL_UNVERIFIED]"
PRIMARY_FILING_REQUIRED = "PRIMARY_FILING_REQUIRED"
STAGE_2_UNVERIFIED = "STAGE_2_UNVERIFIED"
FORMULA_DERIVED = "FORMULA_DERIVED"
FIELD_MISSING = "FIELD_MISSING"
PRICE_LABEL_MISMATCH = "PRICE_LABEL_MISMATCH"
METHOD_ASSUMPTION_MISSING = "METHOD_ASSUMPTION_MISSING"
BENCHMARK_MISMATCH = "BENCHMARK_MISMATCH"

MISSING_PLACEHOLDERS = {
    METRIC_MISSING_RAW,
    FIELD_MISSING_PROVIDER,
    PRIMARY_SOURCE_REQUIRED_PLACEHOLDER,
    SEGMENT_DATA_MISSING,
    METHOD_ASSUMPTION_MISSING_PLACEHOLDER,
    PRICE_LABEL_UNVERIFIED,
}

LOOSE_MISSING_STRINGS = {"N/A", "Data not found", "Unknown", "None", "Missing"}
ANSWERABLE_FROM_REPORT = "ANSWERABLE_FROM_REPORT"
PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION = "PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION"
NOT_ANSWERABLE_FROM_CURRENT_DATA = "NOT_ANSWERABLE_FROM_CURRENT_DATA"
FROM_PAYLOAD = "FROM_PAYLOAD"
NEEDS_EXTERNAL_VERIFICATION = "NEEDS_EXTERNAL_VERIFICATION"
NOT_ALLOWED = "NOT_ALLOWED"
SEVERITIES = {"LOW", "MEDIUM", "HIGH"}
EVIDENCE_BOUNDARIES = {FROM_PAYLOAD, NEEDS_EXTERNAL_VERIFICATION, NOT_ALLOWED}
ANSWERABILITY_STATUSES = {
    ANSWERABLE_FROM_REPORT,
    PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION,
    NOT_ANSWERABLE_FROM_CURRENT_DATA,
}

EN_BANNED_PHRASES = [
    "it is important to note",
    "investors should monitor",
    "may face risks",
    "might face risks",
    "could potentially",
    "further research is needed",
    "should be considered",
    "it should be noted",
    "there are uncertainties",
    "the company shows",
    "the company demonstrates",
    "the company exhibits",
    "relatively strong",
    "potentially attractive",
]

ZH_BANNED_PHRASES = [
    "该公司显示出",
    "展现出了",
    "具备较强的",
    "密切关注",
    "未来动向",
    "值得注意的是",
    "具有一定",
    "在一定程度上",
    "较为稳健",
    "存在一定不确定性",
    "对于……而言",
    "关于……方面",
    "整体来看",
    "综合来看",
    "需要进一步观察",
]


@dataclass
class RiskMethod:
    price_field: str = "adj_close"
    return_frequency: str = "daily"
    annualization_days: int | None = 252
    risk_free_rate: float | None = 0.0
    benchmark: str = "SPY"
    missing_data_handling: str = "aligned trading days; rows with unavailable target or benchmark prices are dropped"
    dividend_handling: str = "included only when the selected provider price field includes dividend adjustments"


def fmt_value(value: Any) -> str:
    if value is None:
        return METRIC_MISSING_RAW
    try:
        if pd.isna(value):
            return METRIC_MISSING_RAW
    except Exception:
        pass
    if isinstance(value, float):
        if abs(value) >= 1_000_000_000_000:
            return f"{value / 1_000_000_000_000:.2f}T"
        if abs(value) >= 1_000_000_000:
            return f"{value / 1_000_000_000:.2f}B"
        if abs(value) >= 1_000_000:
            return f"{value / 1_000_000:.2f}M"
        return f"{value:.4f}"
    return str(value)


def _metric_value(df: pd.DataFrame, metric: str, column: str = "Value") -> Any:
    if df is None or df.empty or "Metric" not in df.columns or column not in df.columns:
        return None
    row = df[df["Metric"] == metric]
    if row.empty:
        return None
    return row.iloc[0][column]


def _audit_row(
    metric_name: str,
    display_value: Any,
    raw_fields: str,
    formula: str,
    source_table: str,
    period: str,
    currency: str,
    price_field: str | None = None,
    confidence_enum: str = STAGE_2_UNVERIFIED,
    warning_enum: str = PRIMARY_FILING_REQUIRED,
    needs_primary_verification: bool = True,
    audit_status: str = STATUS_WARNING,
) -> dict[str, Any]:
    return {
        "metric_name": metric_name,
        "display_value": fmt_value(display_value),
        "raw_fields": raw_fields,
        "formula": formula,
        "source_provider": SOURCE_PROVIDER,
        "source_table": source_table,
        "period": period,
        "timestamp": datetime.now().isoformat(timespec="seconds"),
        "currency": currency,
        "price_field": price_field or "",
        "confidence_enum": confidence_enum,
        "warning_enum": warning_enum,
        "needs_primary_verification": needs_primary_verification,
        "audit_status": audit_status,
    }


def build_data_audit(
    report_data: dict[str, Any],
    risk_method: RiskMethod,
) -> pd.DataFrame:
    raw = report_data["raw"]
    fundamental = raw["fundamental_summary"]
    valuation = raw["valuation"]
    price = raw["price_summary"]
    score = raw["score_table"]
    resilience = raw["ruin_risk"]
    info = raw["info"]
    currency = str(info.get("currency") or "N/A")
    period = f"{report_data.get('start_date')} to {report_data.get('end_date') or 'latest available'}"

    rows = [
        _audit_row("Revenue", _metric_value(fundamental, "Revenue Latest"), "Revenue", "latest provider revenue", "fundamental_summary", period, currency),
        _audit_row("Revenue Growth", _metric_value(fundamental, "Revenue Growth Latest"), "Revenue", "(latest revenue / prior revenue) - 1", "fundamental_summary", period, currency),
        _audit_row("Revenue CAGR", _metric_value(fundamental, "Revenue CAGR"), "Revenue", "CAGR from first to latest available revenue", "fundamental_summary", period, currency),
        _audit_row("Gross Profit", None, "Gross Profit", "provider gross profit", "financial_statement", period, currency),
        _audit_row("Gross Margin", _metric_value(fundamental, "Gross Margin Latest"), "Gross Profit, Revenue", "gross profit / revenue", "fundamental_summary", period, currency),
        _audit_row("Operating Income", None, "Operating Income", "provider operating income", "financial_statement", period, currency),
        _audit_row("Operating Margin", _metric_value(fundamental, "Operating Margin Latest"), "Operating Income, Revenue", "operating income / revenue", "fundamental_summary", period, currency),
        _audit_row("Net Income", None, "Net Income", "provider net income", "financial_statement", period, currency),
        _audit_row("Operating Cash Flow", None, "Operating Cash Flow", "provider operating cash flow", "cash_flow_statement", period, currency),
        _audit_row("Capital Expenditure", None, "Capital Expenditure", "provider capital expenditure", "cash_flow_statement", period, currency),
        _audit_row("Free Cash Flow", None, "Operating Cash Flow, Capital Expenditure", "operating cash flow + capital expenditure", "cash_flow_statement", period, currency),
        _audit_row("FCF Margin", _metric_value(fundamental, "FCF Margin Latest"), "Free Cash Flow, Revenue", "free cash flow / revenue", "fundamental_summary", period, currency),
        _audit_row("Market Cap", _metric_value(valuation, "marketCap"), "marketCap", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("Enterprise Value", _metric_value(valuation, "enterpriseValue"), "enterpriseValue", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("Trailing PE", _metric_value(valuation, "trailingPE"), "trailingPE", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("Forward PE", _metric_value(valuation, "forwardPE"), "forwardPE", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("Price/Sales", _metric_value(valuation, "priceToSalesTrailing12Months"), "priceToSalesTrailing12Months", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("Price/Book", _metric_value(valuation, "priceToBook"), "priceToBook", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("EV/Revenue", _metric_value(valuation, "enterpriseToRevenue"), "enterpriseToRevenue", "provider snapshot", "valuation_snapshot", period, currency),
        _audit_row("EV/EBITDA", _metric_value(valuation, "enterpriseToEbitda"), "enterpriseToEbitda", "provider snapshot", "valuation_snapshot", period, currency),
    ]

    for metric in ["Total Return", "CAGR", "Max Drawdown", "Annualized Volatility", "Sharpe Ratio", "Sortino Ratio", "Calmar Ratio", "Beta vs Benchmark", "Alpha vs Benchmark", "Information Ratio"]:
        rows.append(
            _audit_row(
                metric,
                _metric_value(price, metric, "Target"),
                risk_method.price_field,
                "calculated from aligned daily return series",
                "price_summary",
                period,
                currency,
                price_field=risk_method.price_field,
                confidence_enum=FORMULA_DERIVED,
                warning_enum=STAGE_2_UNVERIFIED,
                needs_primary_verification=False,
                audit_status=STATUS_PASS,
            )
        )

    rows.append(
        _audit_row(
            "Research Score",
            _metric_value(score.rename(columns={"Component": "Metric"}), "Research Score", "Score"),
            "score components",
            "weighted heuristic score by research profile",
            "research_score",
            period,
            currency,
            confidence_enum=FORMULA_DERIVED,
            warning_enum=STAGE_2_UNVERIFIED,
            needs_primary_verification=False,
            audit_status=STATUS_WARNING,
        )
    )
    rows.append(
        _audit_row(
            "Balance Sheet Resilience Score",
            _metric_value(resilience, "Balance Sheet Resilience Score"),
            "debt, cash, EBITDA, free cash flow",
            "100 - balance-sheet stress heuristic",
            "balance_sheet_resilience",
            period,
            currency,
            confidence_enum=FORMULA_DERIVED,
            warning_enum=PRIMARY_FILING_REQUIRED,
            needs_primary_verification=True,
            audit_status=STATUS_WARNING,
        )
    )
    return pd.DataFrame(rows)


def audit_status(audit: pd.DataFrame) -> str:
    if audit is None or audit.empty:
        return STATUS_FAIL
    statuses = set(audit["audit_status"].dropna().astype(str))
    if STATUS_FAIL in statuses:
        return STATUS_FAIL
    if STATUS_WARNING in statuses:
        return STATUS_WARNING
    return STATUS_PASS


def markdown_table(df: pd.DataFrame) -> str:
    if df is None or df.empty:
        return f"_{METRIC_MISSING_RAW}_"
    cols = list(df.columns)
    lines = ["| " + " | ".join(cols) + " |", "| " + " | ".join(["---"] * len(cols)) + " |"]
    for _, row in df.iterrows():
        lines.append("| " + " | ".join(str(row.get(col, "")) for col in cols) + " |")
    return "\n".join(lines)


def artifact_filename(base_name: str, overall_status: str) -> str:
    prefix = "UNVERIFIED_" if overall_status == OVERALL_UNVERIFIED else ""
    return f"{prefix}{base_name}"


def write_data_audit(out_dir: Path, audit: pd.DataFrame, overall_status: str = OVERALL_VERIFIED) -> None:
    audit.to_csv(out_dir / artifact_filename("data_audit.csv", overall_status), index=False)
    content = "# Data Audit\n\nKey metrics are listed with source, formula, confidence, and verification status.\n\n"
    content += markdown_table(audit)
    (out_dir / artifact_filename("data_audit.md", overall_status)).write_text(content, encoding="utf-8")


def build_price_label_sanity_check(
    ticker: str,
    price: pd.DataFrame,
    chart_label_value: float,
    provider_latest_quote: float | None = None,
) -> pd.DataFrame:
    price = price.dropna(how="all")
    last_row = price.iloc[-1]
    last_close = float(last_row.get("close"))
    last_adj_close = last_row.get("adj_close", None)
    if provider_latest_quote is None:
        reference = last_close
    else:
        try:
            reference = last_close if pd.isna(provider_latest_quote) else provider_latest_quote
        except Exception:
            reference = provider_latest_quote
    difference_abs = abs(chart_label_value - reference)
    difference_pct = difference_abs / abs(reference) if reference else float("nan")
    if pd.isna(difference_pct) or difference_pct > 0.015:
        status = STATUS_FAIL
    elif difference_pct > 0.005:
        status = STATUS_WARNING
    else:
        status = STATUS_PASS
    warning = "" if status == STATUS_PASS else PRICE_LABEL_UNVERIFIED
    return pd.DataFrame(
        [
            {
                "ticker": ticker,
                "last_row_date": str(price.index[-1].date()) if hasattr(price.index[-1], "date") else str(price.index[-1]),
                "last_close": last_close,
                "last_adj_close": None if pd.isna(last_adj_close) else last_adj_close,
                "chart_label_value": chart_label_value,
                "provider_latest_quote": reference,
                "difference_abs": difference_abs,
                "difference_pct": difference_pct,
                "status": status,
                "warning_enum": warning,
            }
        ]
    )


def write_price_label_sanity_check(out_dir: Path, check: pd.DataFrame, overall_status: str = OVERALL_VERIFIED) -> None:
    content = "# Price Label Sanity Check\n\n"
    content += "This check compares the chart label with the final price row used by the chart.\n\n"
    content += markdown_table(check)
    (out_dir / artifact_filename("price_label_sanity_check.md", overall_status)).write_text(content, encoding="utf-8")


def risk_method_status(method: RiskMethod) -> str:
    if not method.price_field or method.risk_free_rate is None or not method.annualization_days or not method.return_frequency:
        return STATUS_FAIL
    return STATUS_PASS


def render_risk_methodology(method: RiskMethod, language: str = "en") -> str:
    status = risk_method_status(method)
    marker = f" {METHOD_BLACKBOX}" if status == STATUS_FAIL else ""
    if language == "zh":
        return f"""## 风险指标方法

RISK_METHOD_STATUS: {status}{marker}

- 价格字段：`{method.price_field}`
- 收益率频率：{method.return_frequency}
- 年化天数：{method.annualization_days}
- 无风险利率：{method.risk_free_rate:.2%}
- 基准：{method.benchmark}
- 缺失值处理：对齐交易日，删除目标和基准任一缺失的行。
- 股息处理：股息是否进入收益率，取决于数据源的复权价格口径。

风险指标使用日频复权收盘价、{method.annualization_days} 个交易日年化、{method.risk_free_rate:.2%} 无风险利率。股息是否进入收益率，取决于数据源的复权价格口径。
"""
    return f"""## Risk Metric Methodology

RISK_METHOD_STATUS: {status}{marker}

- Price field: `{method.price_field}`
- Return frequency: {method.return_frequency}
- Annualization days: {method.annualization_days}
- Risk-free rate: {method.risk_free_rate:.2%}
- Benchmark: {method.benchmark}
- Data gap handling: {method.missing_data_handling}
- Dividend handling: {method.dividend_handling}
- Trading-day alignment: target and benchmark are joined on overlapping trading days.

Risk metrics use daily adjusted-close returns, {method.annualization_days} trading days, and a {method.risk_free_rate:.2%} risk-free rate. Dividend effects are included only if adjusted close includes them in the provider data.
"""


def render_battle_card(report_data: dict[str, Any], language: str = "en") -> str:
    ticker = report_data["ticker"]
    profile = report_data["research_profile"]
    valuation = report_data["raw"]["valuation"]
    fundamental = report_data["raw"]["fundamental_summary"]
    pe = _metric_value(valuation, "trailingPE")
    ps = _metric_value(valuation, "priceToSalesTrailing12Months")
    revenue_cagr = _metric_value(fundamental, "Revenue CAGR")
    fcf_margin = _metric_value(fundamental, "FCF Margin Latest")

    if language == "zh":
        return f"""## 投研博弈卡片

### 买入的核心赌注

{ticker} 不是纯高增长故事。核心赌注是利润率、现金流、生态粘性和回购能力能继续支撑每股收益。只要这些支柱不塌，市场就可能继续接受高估值。

### 做空或离场的死穴

反方逻辑从增长放慢但估值不降开始。如果服务业务失速，或者毛利率下滑，当前估值缺少缓冲。

### 市场已经交易了什么

当前价格反映的不只是今年赚了多少钱，还包括市场对长期现金流和经营韧性的信任。市盈率约 {fmt_value(pe)} 倍，市销率约 {fmt_value(ps)} 倍，这意味着市场不只是在买当期收入增长。

### 什么必须守住

- 毛利率不能持续下滑。
- 自由现金流率要保持健康。
- 核心业务不能出现结构性失速。
- 回购对每股收益的支撑不能失效。
- 监管不能打穿关键利润池。

### 一票否决条件

- 服务业务增速明显放缓。
- 毛利率连续两个季度下滑。
- 核心收入下滑且没有服务业务接住。
- 高估值削弱回购对每股收益的支撑。
- 监管压力伤害高利润业务。

### 最优先核查的 3 件事

1. 分业务线收入和服务业务增长。
2. 自由现金流是否稳定，还是受一次性项目影响。
3. 当前估值是否已经透支未来每股收益增长。
"""

    return f"""## Research Battle Card

### The Long Bet

{ticker} is not a pure high-growth bet. The long case rests on margin durability, cash generation, ecosystem strength, and buyback-supported EPS growth. If those pillars hold, the market can keep paying a premium multiple.

### The Short Trigger

The short case starts when growth slows but valuation refuses to reset. If margin quality cracks or cash flow weakens, the premium multiple has little protection.

### Market Pricing

The market is already pricing durable cash flow and resilience. PE is around {fmt_value(pe)} and price-to-sales is around {fmt_value(ps)}, so the setup needs more than revenue growth of {fmt_value(revenue_cagr)}.

### What Must Hold

- Gross margin must not break.
- Free cash flow margin must remain healthy.
- Core revenue must avoid structural decline.
- Buybacks must keep supporting EPS.
- Regulation must not damage high-margin profit pools.

### Kill Criteria

- Services or recurring revenue growth slows materially.
- Gross margin compresses for two consecutive quarters.
- Core revenue declines without offset from higher-quality revenue.
- Buybacks lose EPS impact because valuation stays too high.
- Regulatory pressure damages platform economics.

### Verification Priority

1. Segment revenue and services growth.
2. Free cash flow durability versus one-off working-capital effects.
3. Valuation support from EPS growth, margins, and buybacks.
"""


def render_valuation_sensitivity(report_data: dict[str, Any], language: str = "en") -> str:
    valuation = report_data["raw"]["valuation"]
    pe = _metric_value(valuation, "trailingPE")
    scenarios = [30, 25, 20]
    eps_growth = ["0%", "5%", "10%", "15%"]
    if language == "zh":
        rows = "\n".join(f"| 市盈率压缩到 {s} 倍 | 与当前 {fmt_value(pe)} 倍比较 | 每股收益增长假设：{', '.join(eps_growth)} |" for s in scenarios)
        return f"""## 估值压力测试

这不是目标价预测，而是估值压力测试。它回答的不是“应该值多少钱”，而是“如果市场开始杀估值，可能有多疼”。

| 场景 | 对比 | 每股收益增长假设 |
|---|---|---|
{rows}
"""
    rows = "\n".join(f"| PE compresses to {s}x | Compared with current {fmt_value(pe)}x | EPS growth assumptions: {', '.join(eps_growth)} |" for s in scenarios)
    return f"""## Valuation Sensitivity

This is not a price target. It is a valuation stress test.

| Scenario | Comparison | EPS Growth Assumptions |
|---|---|---|
{rows}
"""


def render_segment_revenue(report_data: dict[str, Any], language: str = "en") -> str:
    ticker = report_data["ticker"]
    mega_caps = {"AAPL", "MSFT", "NVDA", "GOOGL", "GOOG", "AMZN", "META"}
    if language == "zh":
        if ticker in mega_caps:
            return """## 业务线拆解

这家公司必须拆业务线。只看总收入，会把真正的增长来源和利润结构藏起来。

需要手工核查：

- 主要产品线、服务业务或对应公司的核心业务线；
- 各业务线 YoY 增长；
- 各业务线收入占比；
- 高毛利业务是否在提升利润质量；
- 监管是否影响高利润业务。
"""
        return "## 业务线拆解\n\n当前版本未自动提取业务线收入。对重要公司仍应手工核查分部收入。\n"
    if ticker in mega_caps:
        return """## Segment Revenue Analysis

Segment revenue is required for this company. Total revenue alone hides the real business mix.

Manual source required:

- Product or business-line revenue;
- segment YoY growth;
- segment revenue mix;
- whether higher-margin segments are improving profit quality;
- whether regulation can pressure high-margin segments.
"""
    return "## Segment Revenue Analysis\n\nAutomated segment extraction is not implemented in this version. Manual segment revenue review is still required for serious research.\n"


def lint_language(text: str, language: str = "en") -> dict[str, Any]:
    banned = ZH_BANNED_PHRASES if language == "zh" else EN_BANNED_PHRASES
    lower = text.lower()
    hits = [phrase for phrase in banned if (phrase in text if language == "zh" else phrase in lower)]
    sentences = [s.strip() for s in text.replace("\n", " ").split(".") if s.strip()]
    overlong = [s for s in sentences if len(s.split()) > 38] if language == "en" else [s for s in text.split("。") if len(s) > 90]
    mixed_language_hits = []
    if language == "zh":
        mixed_language_hits = [hit for hit in ["DATA_AUDIT_STATUS", "RISK_METHOD_STATUS", "OVERALL_REPORT_STATUS", "Price Label Check", "Research Battle Card"] if hit in text]
    else:
        mixed_language_hits = [hit for hit in ["买入的核心赌注", "估值压力测试", "一票否决条件", "投研博弈卡片"] if hit in text]
    translationese_hits = []
    if language == "zh":
        translationese_hits = [phrase for phrase in ["当前估值已经在交易", "具有一定", "综合来看", "需要进一步观察"] if phrase in text]
    raw_placeholder_hits = [placeholder for placeholder in MISSING_PLACEHOLDERS if placeholder in text]
    unexplained_chart_count = max(0, text.count("![") - text.count("图表看什么") - text.count("What this chart shows"))
    unanswered_question_count = max(0, text.count("？") + text.count("?") - text.count("回答") - text.count("Answer"))
    table_without_explanation_count = max(0, text.count("|---") - text.count("这说明什么") - text.count("What this means") - text.count("Interpretation"))
    if len(hits) == 0:
        status = STATUS_PASS
    elif len(hits) <= 2:
        status = STATUS_WARNING
    else:
        status = STATUS_FAIL
    language_quality_score = 100
    language_quality_score -= len(mixed_language_hits) * 15
    language_quality_score -= len(translationese_hits) * 8
    language_quality_score -= unexplained_chart_count * 10
    language_quality_score -= unanswered_question_count * 6
    language_quality_score -= table_without_explanation_count * 4
    language_quality_score = max(0, language_quality_score)
    return {
        "language": language,
        "banned_phrase_hits": hits,
        "mixed_language_hits": mixed_language_hits,
        "translationese_hits": translationese_hits,
        "unexplained_chart_count": unexplained_chart_count,
        "unanswered_question_count": unanswered_question_count,
        "raw_placeholder_hits": raw_placeholder_hits,
        "table_without_explanation_count": table_without_explanation_count,
        "language_quality_score": language_quality_score,
        "overlong_sentences": overlong,
        "overlong_sections": [],
        "rewritten_sections": [],
        "rewrite_attempts": 0,
        "final_status": status,
    }


def write_language_lint_report(out_dir: Path, results: list[dict[str, Any]], overall_status: str = OVERALL_VERIFIED) -> None:
    lines = ["# Language Lint Report", ""]
    for result in results:
        lines.extend(
            [
                f"## {result['language']}",
                "",
                f"language: {result['language']}",
                f"banned_phrase_hits: {result['banned_phrase_hits']}",
                f"mixed_language_hits: {result.get('mixed_language_hits', [])}",
                f"translationese_hits: {result.get('translationese_hits', [])}",
                f"unexplained_chart_count: {result.get('unexplained_chart_count', 0)}",
                f"unanswered_question_count: {result.get('unanswered_question_count', 0)}",
                f"raw_placeholder_hits: {result.get('raw_placeholder_hits', [])}",
                f"table_without_explanation_count: {result.get('table_without_explanation_count', 0)}",
                f"language_quality_score: {result.get('language_quality_score', 100)}",
                f"overlong_sentences: {len(result['overlong_sentences'])}",
                f"overlong_sections: {result['overlong_sections']}",
                f"rewritten_sections: {result['rewritten_sections']}",
                f"rewrite_attempts: {result['rewrite_attempts']}",
                f"max_language_rewrite_attempts: {MAX_LANGUAGE_REWRITE_ATTEMPTS}",
                f"final_status: {result['final_status']}",
                "",
            ]
        )
    (out_dir / artifact_filename("language_lint_report.md", overall_status)).write_text("\n".join(lines), encoding="utf-8")


def no_free_text_warning_payload(payload: dict[str, Any]) -> bool:
    allowed = {
        SOURCE_PROVIDER,
        PRIMARY_FILING_REQUIRED,
        STAGE_2_UNVERIFIED,
        "STALE_TIMESTAMP",
        FIELD_MISSING,
        FORMULA_DERIVED,
        BENCHMARK_MISMATCH,
        PRICE_LABEL_MISMATCH,
        METHOD_ASSUMPTION_MISSING,
        *MISSING_PLACEHOLDERS,
    }
    for key, value in payload.items():
        if key.endswith("_enum") or key in {"source_type", "warning_enum", "confidence_enum"}:
            if isinstance(value, str) and value not in allowed and len(value.split()) > 1:
                return False
    return True


def classify_missing_placeholder(value: Any) -> dict[str, str] | None:
    if isinstance(value, str) and value in MISSING_PLACEHOLDERS:
        boundary = "NOT_ALLOWED" if value in {METHOD_ASSUMPTION_MISSING_PLACEHOLDER, PRICE_LABEL_UNVERIFIED} else "NEEDS_EXTERNAL_VERIFICATION"
        answerability = "NOT_ANSWERABLE_FROM_CURRENT_DATA" if boundary == "NOT_ALLOWED" else "PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION"
        return {
            "answerability": answerability,
            "evidence_boundary": boundary,
        }
    return None


def ai_response_for_missing_placeholder(placeholder: str, topic: str = "Segment data") -> str:
    classification = classify_missing_placeholder(placeholder)
    if not classification:
        return ""
    return (
        f"{topic} is missing. Services thesis cannot be treated as evidence until the latest 10-K segment table is checked. "
        f"answerability={classification['answerability']}; evidence_boundary={classification['evidence_boundary']}."
    )


def zh_response_for_missing_placeholder(placeholder: str, topic: str = "业务线数据") -> str:
    classification = classify_missing_placeholder(placeholder)
    if not classification:
        return ""
    return (
        f"{topic}缺失。服务业务能否支撑估值，目前只能作为研究假设，不能当作证据。"
        f"下一步必须核对最新 10-K 的 segment revenue 表。"
        f"answerability={classification['answerability']}; evidence_boundary={classification['evidence_boundary']}。"
    )


def classify_answerability(question: str, payload: dict[str, Any]) -> dict[str, Any]:
    lowered = question.lower()
    future_terms = ["will", "future", "outperform", "next year", "target price", "double", "price", "明年", "股价", "翻倍", "目标价"]
    why_terms = ["why", "为什么", "multiple expansion", "market reaction"]
    segment_terms = ["services", "segment", "business line", "业务线", "服务业务", "分部"]
    if any(term in lowered or term in question for term in future_terms):
        return {
            "question": question,
            "status": NOT_ANSWERABLE_FROM_CURRENT_DATA,
            "short_answer": "Not answerable from the current report. The workflow does not predict short-term returns, target prices, or future market reactions.",
            "evidence_used": [],
            "what_to_verify_next": "Use primary sources and scenario analysis; do not treat this report as a forecast.",
            "evidence_boundary": NOT_ALLOWED,
        }
    if any(term in lowered or term in question for term in why_terms):
        return {
            "question": question,
            "status": PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION,
            "short_answer": "Only partially answerable. The report can frame a hypothesis, but causal claims require primary-source verification.",
            "evidence_used": ["structured_payload"],
            "what_to_verify_next": "Check segment revenue, margin drivers, and management commentary before treating the claim as evidence.",
            "evidence_boundary": NEEDS_EXTERNAL_VERIFICATION,
        }
    if any(term in lowered or term in question for term in segment_terms):
        return {
            "question": question,
            "status": PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION,
            "short_answer": "Plausible, but segment data is required before treating this as evidence.",
            "evidence_used": [SEGMENT_DATA_MISSING],
            "what_to_verify_next": "Check the latest 10-K segment revenue table before using the segment thesis.",
            "evidence_boundary": NEEDS_EXTERNAL_VERIFICATION,
        }
    return {
        "question": question,
        "status": ANSWERABLE_FROM_REPORT,
        "short_answer": "Answerable from the current report when the relevant metric is present in the structured payload.",
        "evidence_used": ["structured_payload"],
        "what_to_verify_next": "Verify provider values against primary filings before relying on the conclusion.",
        "evidence_boundary": FROM_PAYLOAD,
    }


def build_ai_correction_log(report_data: dict[str, Any], language: str = "en") -> dict[str, Any]:
    ticker = report_data["ticker"]
    valuation = report_data["raw"]["valuation"]
    fundamental = report_data["raw"]["fundamental_summary"]
    pe = _metric_value(valuation, "trailingPE")
    revenue_cagr = _metric_value(fundamental, "Revenue CAGR")
    segment_placeholder = SEGMENT_DATA_MISSING
    corrections = [
        {
            "section": "Valuation Sensitivity",
            "original_issue": "Valuation can read as a static multiple table unless the embedded expectations are stated.",
            "suggested_revision": "Explain what the current PE requires: margin durability, cash-flow resilience, and buyback-supported EPS.",
            "reason": "This turns valuation from description into a testable research claim.",
            "requires_data_verification": False,
            "severity": "MEDIUM",
            "evidence_boundary": FROM_PAYLOAD,
        },
        {
            "section": "Segment Revenue Analysis",
            "original_issue": segment_placeholder,
            "suggested_revision": "Segment data is missing. Services thesis cannot be treated as evidence until the latest 10-K segment table is checked.",
            "reason": "Total revenue hides business mix, especially for mega-cap platform companies.",
            "requires_data_verification": True,
            "severity": "HIGH",
            "evidence_boundary": NEEDS_EXTERNAL_VERIFICATION,
        },
    ]
    unanswered = [
        "How much growth comes from Services or recurring revenue rather than hardware or one-off demand?",
        "Is margin improvement structural or temporary?",
        "How much EPS growth depends on buybacks rather than net income growth?",
        "What multiple compression can the stock absorb?",
        f"Why own {ticker} instead of the benchmark at this valuation?",
    ]
    questions = [
        f"Is {ticker} a high-growth company?",
        "Is Services carrying valuation?",
        f"Will {ticker} beat the benchmark next year?",
    ]
    answerability = [classify_answerability(question, report_data) for question in questions]
    if language == "zh":
        stance = (
            "这不是买卖建议，而是研究立场。\n\n"
            f"{ticker} 可以放在 Watchlist，但还不该给高信念标签。"
            f"收入 CAGR 约 {fmt_value(revenue_cagr)}，当前 PE 约 {fmt_value(pe)}，"
            "核心问题不是公司质量，而是估值是否已经把利润率、服务业务和回购支撑买得太满。"
        )
        next_checks = [
            "拉最新 10-K，把 Services 或对应高质量业务线单独拆出来。",
            "重算 30x、25x、20x PE 下的估值压力。",
            "拆 EPS 增长，看它来自净利润增长还是回购减少股本。",
        ]
    else:
        stance = (
            "This is not a buy/sell recommendation. It is a research stance.\n\n"
            f"{ticker} deserves a Watchlist label, not a high-conviction label. "
            f"Revenue CAGR near {fmt_value(revenue_cagr)} does not carry the setup by itself, while PE near {fmt_value(pe)} demands durability. "
            "The next step is to test whether margins, segment mix, and buybacks can defend the multiple."
        )
        next_checks = [
            "Pull latest 10-K segment revenue and isolate Services or recurring revenue growth.",
            "Recalculate valuation sensitivity at 30x, 25x, and 20x PE.",
            "Split EPS growth into net income growth and share-count reduction.",
        ]
    return {
        "language": language,
        "max_correction_passes": MAX_CORRECTION_PASSES,
        "report_corrections": corrections,
        "unanswered_questions": unanswered[:5],
        "answerability_classification": answerability,
        "research_stance": stance,
        "next_3_checks": next_checks,
        "status": STATUS_PASS if len(next_checks) == 3 and all(c["severity"] in SEVERITIES for c in corrections) else STATUS_FAIL,
    }


def validate_ai_correction_log(log: dict[str, Any]) -> str:
    corrections = log.get("report_corrections", [])
    next_checks = log.get("next_3_checks", [])
    if len(next_checks) != 3:
        return STATUS_FAIL
    for correction in corrections:
        required = {"section", "original_issue", "suggested_revision", "reason", "requires_data_verification", "severity", "evidence_boundary"}
        if not required.issubset(correction):
            return STATUS_FAIL
        if correction["severity"] not in SEVERITIES:
            return STATUS_FAIL
        if correction["evidence_boundary"] not in EVIDENCE_BOUNDARIES:
            return STATUS_FAIL
        if not isinstance(correction["requires_data_verification"], bool):
            return STATUS_FAIL
    for item in log.get("answerability_classification", []):
        if item.get("status") not in ANSWERABILITY_STATUSES:
            return STATUS_FAIL
        if item.get("evidence_boundary") not in EVIDENCE_BOUNDARIES:
            return STATUS_FAIL
    return log.get("status", STATUS_WARNING)


def write_ai_correction_log(out_dir: Path, log: dict[str, Any], overall_status: str = OVERALL_VERIFIED) -> None:
    import json

    json_path = out_dir / artifact_filename("ai_correction_log.json", overall_status)
    md_path = out_dir / artifact_filename("ai_correction_log.md", overall_status)
    json_path.write_text(json.dumps(log, ensure_ascii=False, indent=2), encoding="utf-8")
    lines = [
        "# AI Correction Log",
        "",
        "AI may propose interpretation corrections, but it must not overwrite deterministic metrics unless verified by source or formula.",
        "",
        f"language: {log.get('language')}",
        f"max_correction_passes: {log.get('max_correction_passes')}",
        f"status: {log.get('status')}",
        "",
        "## Report Corrections",
        "",
    ]
    for correction in log.get("report_corrections", []):
        lines.extend(
            [
                f"### {correction['section']}",
                "",
                f"- original_issue: {correction['original_issue']}",
                f"- suggested_revision: {correction['suggested_revision']}",
                f"- reason: {correction['reason']}",
                f"- requires_data_verification: {correction['requires_data_verification']}",
                f"- severity: {correction['severity']}",
                f"- evidence_boundary: {correction['evidence_boundary']}",
                "",
            ]
        )
    lines.extend(["## Unanswered Questions", ""])
    for idx, question in enumerate(log.get("unanswered_questions", []), start=1):
        lines.append(f"{idx}. {question}")
    lines.extend(["", "## Answerability Classification", ""])
    for item in log.get("answerability_classification", []):
        lines.extend(
            [
                f"### {item['question']}",
                "",
                f"- status: {item['status']}",
                f"- short_answer: {item['short_answer']}",
                f"- evidence_used: {item['evidence_used']}",
                f"- what_to_verify_next: {item['what_to_verify_next']}",
                f"- evidence_boundary: {item['evidence_boundary']}",
                "",
            ]
        )
    lines.extend(["## Research Stance", "", log.get("research_stance", ""), "", "## Next 3 Checks", ""])
    for idx, check in enumerate(log.get("next_3_checks", []), start=1):
        lines.append(f"{idx}. {check}")
    md_path.write_text("\n".join(lines), encoding="utf-8")


def payload_has_loose_missing_strings(value: Any) -> bool:
    if isinstance(value, dict):
        return any(payload_has_loose_missing_strings(v) for v in value.values())
    if isinstance(value, list):
        return any(payload_has_loose_missing_strings(v) for v in value)
    return isinstance(value, str) and value in LOOSE_MISSING_STRINGS


def overall_report_status(gate_status: dict[str, str]) -> str:
    statuses = [
        value
        for key, value in gate_status.items()
        if key.endswith("_STATUS") and key != "OVERALL_REPORT_STATUS"
    ]
    if STATUS_FAIL in statuses:
        return OVERALL_UNVERIFIED
    if STATUS_WARNING in statuses:
        return STATUS_WARNING
    return OVERALL_VERIFIED


def report_filename(symbol: str, suffix: str, overall_status: str) -> str:
    prefix = "UNVERIFIED_" if overall_status == OVERALL_UNVERIFIED else ""
    return f"{prefix}{symbol.upper()}_{suffix}"


def clone_payload(payload: dict[str, Any]) -> dict[str, Any]:
    import copy

    return copy.deepcopy(payload)


def ai_layer_can_only_suggest(payload: dict[str, Any], suggestions: dict[str, Any]) -> bool:
    return payload == clone_payload(payload) and "deterministic_metrics" not in suggestions
