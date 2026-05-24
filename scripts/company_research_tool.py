#!/usr/bin/env python3
"""
OpenBB Company Research Tool

A command-line tool for generating standardized company research data packs.

Core goals:
- Pull price and financial data
- Compare a ticker with a chosen benchmark
- Calculate risk, return, growth, profitability, and cash-flow metrics
- Generate charts, CSV files, and a Markdown research report
- Keep the output useful for human investment research, not automated trading

Important:
This tool is NOT a buy/sell recommendation engine.
"""

from __future__ import annotations

__version__ = "4.3.0"

import argparse
from datetime import datetime
import math
import os
import shutil
import sys
import tempfile
import textwrap
import time
import zipfile
from pathlib import Path
from typing import Any

os.environ.setdefault("MPLCONFIGDIR", str(Path(tempfile.gettempdir()) / "openbb_company_research_tool_mpl"))

import pandas as pd
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

try:
    import plotly.graph_objects as go
    from plotly.subplots import make_subplots
except Exception:
    go = None
    make_subplots = None

try:
    from openbb import obb
except Exception:
    obb = None

try:
    import yfinance as yf
except Exception as exc:
    raise SystemExit(
        "yfinance import failed. Install dependencies first:\n"
        "  pip install -r requirements.txt\n\n"
        f"Original error: {exc}"
    )

try:
    from ai_review import (
        DEFAULT_AI_MODEL,
        build_ai_review_payload,
        call_ai_review,
        render_ai_review_markdown,
        render_ai_review_skipped,
    )
except Exception:
    DEFAULT_AI_MODEL = "gpt-4o-mini"
    build_ai_review_payload = None
    call_ai_review = None
    render_ai_review_markdown = None
    render_ai_review_skipped = None

try:
    import terminal_ui
except Exception:
    terminal_ui = None

try:
    from asset_aware import (
        build_asset_profile,
        build_report_blocks,
        lifecycle_logic_check,
        apply_interpretation_patch,
        write_patch_artifacts,
        company_specificity_status,
        fallback_status,
        overall_status_v43,
        rollup_data_verification_status,
        rollup_thesis_verification_status,
        overall_status_from_verification,
        organize_report_pack,
        write_run_readme,
        write_json,
        write_framework_gap_analysis,
        write_improvement_suggestions,
        write_regression_test_suggestions,
        write_system_self_review,
        write_lifecycle_logic_report,
        presentation_gate,
        copy_pack_to_latest,
        build_asset_aware_ai_correction_log,
    )
except Exception:
    build_asset_profile = None
    build_report_blocks = None
    lifecycle_logic_check = None
    apply_interpretation_patch = None
    write_patch_artifacts = None
    company_specificity_status = None
    fallback_status = None
    overall_status_v43 = None
    rollup_data_verification_status = None
    rollup_thesis_verification_status = None
    overall_status_from_verification = None
    organize_report_pack = None
    write_run_readme = None
    write_json = None
    write_framework_gap_analysis = None
    write_improvement_suggestions = None
    write_regression_test_suggestions = None
    write_system_self_review = None
    write_lifecycle_logic_report = None
    presentation_gate = None
    copy_pack_to_latest = None
    build_asset_aware_ai_correction_log = None

try:
    from v4_workflow import (
        STATUS_FAIL,
        STATUS_PASS,
        STATUS_WARNING,
        RiskMethod,
        audit_status,
        build_ai_correction_log,
        build_data_audit,
        build_price_label_sanity_check,
        lint_language,
        overall_report_status,
        report_filename,
        render_battle_card,
        render_risk_methodology,
        render_segment_revenue,
        render_valuation_sensitivity,
        risk_method_status,
        validate_ai_correction_log,
        write_ai_correction_log,
        write_data_audit,
        write_language_lint_report,
        write_price_label_sanity_check,
    )
except Exception:
    STATUS_PASS = "PASS"
    STATUS_WARNING = "WARNING"
    STATUS_FAIL = "FAIL"
    def overall_report_status(gate_status: dict[str, str]) -> str:
        return "UNVERIFIED" if STATUS_FAIL in gate_status.values() else STATUS_WARNING if STATUS_WARNING in gate_status.values() else "VERIFIED"
    def report_filename(symbol: str, suffix: str, overall_status: str) -> str:
        return f"{'UNVERIFIED_' if overall_status == 'UNVERIFIED' else ''}{symbol.upper()}_{suffix}"
    RiskMethod = None


# =============================================================================
# Utility
# =============================================================================

def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def reset_dir(path: Path) -> None:
    if not path.exists():
        ensure_dir(path)
        return

    for child in path.iterdir():
        if child.is_dir():
            shutil.rmtree(child)
        else:
            child.unlink()


def safe_symbol(symbol: str) -> str:
    return symbol.upper().replace("/", "_").replace(".", "_")


def clean_columns(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()
    if isinstance(df.columns, pd.MultiIndex):
        df.columns = [str(c[0]) for c in df.columns]
    df.columns = [str(c).strip().lower().replace(" ", "_") for c in df.columns]
    return df


def drop_empty_rows(df: pd.DataFrame) -> pd.DataFrame:
    if df is None or df.empty:
        return pd.DataFrame()
    return df.dropna(how="all")


MISSING_VALUE_PLACEHOLDER = "[METRIC_MISSING_RAW]"


def fmt_number(value: Any) -> str:
    if value is None:
        return MISSING_VALUE_PLACEHOLDER
    try:
        if pd.isna(value):
            return MISSING_VALUE_PLACEHOLDER
        value = float(value)
    except Exception:
        return str(value)

    abs_value = abs(value)
    if abs_value >= 1_000_000_000_000:
        return f"{value / 1_000_000_000_000:.2f}T"
    if abs_value >= 1_000_000_000:
        return f"{value / 1_000_000_000:.2f}B"
    if abs_value >= 1_000_000:
        return f"{value / 1_000_000:.2f}M"
    if abs_value >= 1_000:
        return f"{value:,.2f}"
    return f"{value:.2f}"


def fmt_percent(value: Any) -> str:
    if value is None:
        return MISSING_VALUE_PLACEHOLDER
    try:
        if pd.isna(value):
            return MISSING_VALUE_PLACEHOLDER
        return f"{float(value):.2%}"
    except Exception:
        return MISSING_VALUE_PLACEHOLDER


def fmt_score(value: Any) -> str:
    if value is None:
        return MISSING_VALUE_PLACEHOLDER
    try:
        if pd.isna(value):
            return MISSING_VALUE_PLACEHOLDER
        return f"{float(value):.2f} / 100"
    except Exception:
        return MISSING_VALUE_PLACEHOLDER


PERCENT_METRICS = {
    "Total Return",
    "CAGR",
    "1Y Return",
    "6M Return",
    "3M Return",
    "Max Drawdown",
    "Annualized Volatility",
    "Alpha vs Benchmark",
    "Tracking Error",
    "Upside Capture",
    "Downside Capture",
    "Revenue CAGR",
    "Revenue Growth Latest",
    "Gross Margin Latest",
    "Gross Margin Change",
    "Operating Margin Latest",
    "Operating Margin Change",
    "Net Margin",
    "FCF Margin Latest",
    "FCF Margin Change",
    "Revenue Growth YoY",
    "Gross Margin",
    "Operating Margin",
    "FCF Margin",
    "grossMargins",
    "operatingMargins",
    "profitMargins",
    "returnOnEquity",
    "returnOnAssets",
    "revenueGrowth",
    "earningsGrowth",
    "heldPercentInsiders",
    "heldPercentInstitutions",
    "Weight",
    "Loan / Value",
}

RATIO_METRICS = {
    "Sharpe Ratio",
    "Sortino Ratio",
    "Calmar Ratio",
    "Beta vs Benchmark",
    "Correlation vs Benchmark",
    "Information Ratio",
    "beta",
    "trailingPE",
    "forwardPE",
    "priceToSalesTrailing12Months",
    "priceToBook",
    "enterpriseToRevenue",
    "enterpriseToEbitda",
}

COUNT_METRICS = {
    "Positive Net Income Years",
    "Positive FCF Years",
}

SHARE_COUNT_METRICS = {
    "sharesOutstanding",
    "floatShares",
}

SCORE_METRICS = {
    "Growth Score",
    "Profitability Score",
    "Quality Trend Score",
    "Risk Control Score",
    "Benchmark Score",
    "Valuation Sanity Score",
    "Research Score",
    "Balance Sheet Resilience Score",
}

RISK_METRICS = {
    "Net Debt / EBITDA",
    "Debt / FCF",
    "Cash Runway Years",
}

CURRENCY_METRICS = {
    "marketCap",
    "enterpriseValue",
    "operatingCashflow",
    "freeCashflow",
    "totalCash",
    "totalDebt",
    "fiftyTwoWeekLow",
    "fiftyTwoWeekHigh",
    "Revenue",
    "Gross Profit",
    "Operating Income",
    "Net Income",
    "Operating Cash Flow",
    "Capital Expenditure",
    "Free Cash Flow",
    "Net Debt",
    "EBITDA",
    "Portfolio Value",
    "Margin Loan",
    "Equity Cushion",
}

FUND_QUOTE_TYPES = {"ETF", "FUND", "MUTUALFUND", "INDEX"}
CHART_COLORS = {
    "target": "#2563eb",
    "benchmark": "#64748b",
    "positive": "#059669",
    "negative": "#dc2626",
    "accent": "#7c3aed",
    "grid": "#d8dee9",
    "text": "#111827",
}

plt.rcParams.update(
    {
        "font.family": "DejaVu Sans",
        "axes.titlesize": 15,
        "axes.titleweight": "bold",
        "axes.labelsize": 10,
        "xtick.labelsize": 9,
        "ytick.labelsize": 9,
        "legend.fontsize": 9,
        "figure.facecolor": "white",
        "axes.facecolor": "white",
    }
)


def metric_value_kind(metric: str) -> str:
    """Return display type based on an explicit metric-name registry."""
    metric = str(metric)

    if metric in PERCENT_METRICS:
        return "percent"
    if metric in RATIO_METRICS:
        return "ratio"
    if metric in COUNT_METRICS:
        return "integer"
    if metric in SHARE_COUNT_METRICS:
        return "share_count"
    if metric in SCORE_METRICS:
        return "score"
    if metric in RISK_METRICS:
        return "ratio"
    if metric in CURRENCY_METRICS:
        return "currency"

    return "number"


def format_value_by_metric(metric: str, value: Any) -> str:
    """Format table values according to the metric row."""
    kind = metric_value_kind(metric)

    if value is None:
        return MISSING_VALUE_PLACEHOLDER

    try:
        if pd.isna(value):
            return MISSING_VALUE_PLACEHOLDER
    except Exception:
        pass

    if metric in {"trailingPE", "forwardPE"}:
        try:
            if float(value) <= 0:
                return "不适用 / 盈利未建立"
        except Exception:
            pass
    if metric == "enterpriseToEbitda":
        try:
            if float(value) <= 0:
                return "不适用 / EBITDA 为负"
        except Exception:
            pass

    if kind == "percent":
        return fmt_percent(value)

    if kind == "score":
        return fmt_score(value)

    if kind == "ratio":
        try:
            return f"{float(value):.2f}"
        except Exception:
            return str(value)

    if kind == "integer":
        try:
            return str(int(round(float(value))))
        except Exception:
            return str(value)

    if kind == "share_count":
        try:
            return f"{int(round(float(value))):,}"
        except Exception:
            return str(value)

    return fmt_number(value)


def is_fund_like(info: dict[str, Any]) -> bool:
    return str(info.get("quoteType", "")).upper() in FUND_QUOTE_TYPES


def default_run_id(symbol: str, benchmark: str, start_date: str, end_date: str | None) -> str:
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    parts = [timestamp, f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}", f"start_{start_date}"]
    if end_date:
        parts.append(f"end_{end_date}")
    return "_".join(parts)


def output_dir_for_run(
    output: Path,
    symbol: str,
    benchmark: str,
    start_date: str,
    end_date: str | None,
    archive: bool,
    run_id: str | None,
) -> Path:
    symbol_dir = output / safe_symbol(symbol)
    if archive or run_id:
        return symbol_dir / "runs" / (run_id or default_run_id(symbol, benchmark, start_date, end_date))
    return symbol_dir / "runs" / default_run_id(symbol, benchmark, start_date, end_date)


def copy_run_to_latest(run_dir: Path, latest_dir: Path) -> None:
    reset_dir(latest_dir)
    for child in run_dir.iterdir():
        target = latest_dir / child.name
        if child.is_dir():
            shutil.copytree(child, target)
        else:
            shutil.copy2(child, target)


def markdown_table(
    df: pd.DataFrame,
    max_rows: int = 30,
    percent_columns: set[str] | None = None,
    score_columns: set[str] | None = None,
) -> str:
    if df is None or df.empty:
        return f"_{MISSING_VALUE_PLACEHOLDER}_"

    percent_columns = percent_columns or set()
    score_columns = score_columns or set()

    view = df.copy().head(max_rows)

    if not isinstance(view.index, pd.RangeIndex):
        view = view.reset_index()

    headers = [str(c) for c in view.columns]
    lines = [
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join(["---"] * len(headers)) + " |",
    ]

    has_metric_column = "Metric" in view.columns

    for _, row in view.iterrows():
        values = []
        row_metric = str(row["Metric"]) if has_metric_column else None

        for col in view.columns:
            col_name = str(col)
            value = row[col]

            if isinstance(value, pd.Timestamp):
                values.append(str(value.date()))
                continue

            if col_name in score_columns:
                values.append(fmt_score(value))
                continue

            if has_metric_column and col_name != "Metric":
                values.append(format_value_by_metric(row_metric, value))
                continue

            if col_name in percent_columns:
                values.append(fmt_percent(value))
                continue

            if isinstance(value, (int, float)) and not isinstance(value, bool):
                values.append(fmt_number(value))
                continue

            values.append(str(value))

        lines.append("| " + " | ".join(values) + " |")

    return "\n".join(lines)

def save_text(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")


# =============================================================================
# Price analysis
# =============================================================================

def fetch_price_history(symbol: str, start_date: str, end_date: str | None = None) -> pd.DataFrame:
    symbol = symbol.upper()

    if obb is not None:
        try:
            kwargs = {
                "symbol": symbol,
                "start_date": start_date,
                "provider": "yfinance",
            }
            if end_date:
                kwargs["end_date"] = end_date

            result = obb.equity.price.historical(**kwargs)
            df = result.to_df().copy()

            if "date" in df.columns:
                df = df.set_index("date")

            df.index = pd.to_datetime(df.index)
            df = df.sort_index()
            df = clean_columns(df)

            if "close" not in df.columns:
                raise ValueError("OpenBB returned price data without a close column.")

            return df

        except Exception as exc:
            print(f"[WARN] OpenBB price failed for {symbol}: {exc}")
            print(f"[WARN] Falling back to yfinance for {symbol}.")

    df = yf.download(
        symbol,
        start=start_date,
        end=end_date,
        auto_adjust=False,
        progress=False,
    )

    if df.empty:
        raise ValueError(f"{symbol}: no price data returned.")

    df.index = pd.to_datetime(df.index)
    df = df.sort_index()
    df = clean_columns(df)

    if "close" not in df.columns:
        raise ValueError(f"{symbol}: missing close column.")

    return df


def select_price_series(price: pd.DataFrame, field: str) -> pd.Series:
    field = field.lower()
    if field == "adj_close" and "adj_close" in price.columns:
        return price["adj_close"]
    if field == "adj_close" and "adjusted_close" in price.columns:
        return price["adjusted_close"]
    if "close" not in price.columns:
        raise ValueError("Price data is missing close column.")
    return price["close"]


def daily_returns(close: pd.Series) -> pd.Series:
    return close.dropna().pct_change().dropna()


def total_return(close: pd.Series) -> float:
    close = close.dropna()
    if len(close) < 2:
        return float("nan")
    return float(close.iloc[-1] / close.iloc[0] - 1)


def cagr(close: pd.Series) -> float:
    close = close.dropna()
    if len(close) < 2:
        return float("nan")

    days = (close.index[-1] - close.index[0]).days
    if days <= 0:
        return float("nan")

    return float((close.iloc[-1] / close.iloc[0]) ** (365 / days) - 1)


def annualized_volatility(close: pd.Series, annualization_days: int = 252) -> float:
    r = daily_returns(close)
    if r.empty:
        return float("nan")
    return float(r.std() * math.sqrt(annualization_days))


def max_drawdown(close: pd.Series) -> float:
    close = close.dropna()
    if close.empty:
        return float("nan")
    running_max = close.cummax()
    dd = close / running_max - 1
    return float(dd.min())


def rolling_return(close: pd.Series, trading_days: int) -> float:
    close = close.dropna()
    if len(close) <= trading_days:
        return float("nan")
    return float(close.iloc[-1] / close.iloc[-trading_days] - 1)


def sharpe_ratio(close: pd.Series, risk_free_rate: float = 0.0, annualization_days: int = 252) -> float:
    vol = annualized_volatility(close, annualization_days)
    if vol == 0 or pd.isna(vol):
        return float("nan")
    return float((cagr(close) - risk_free_rate) / vol)


def sortino_ratio(close: pd.Series, risk_free_rate: float = 0.0, annualization_days: int = 252) -> float:
    r = daily_returns(close)
    if r.empty:
        return float("nan")
    downside = r[r < 0].std() * math.sqrt(annualization_days)
    if downside == 0 or pd.isna(downside):
        return float("nan")
    return float((cagr(close) - risk_free_rate) / downside)


def calmar_ratio(close: pd.Series) -> float:
    dd = abs(max_drawdown(close))
    if dd == 0 or pd.isna(dd):
        return float("nan")
    return float(cagr(close) / dd)


def beta_alpha(target: pd.Series, benchmark: pd.Series, risk_free_rate: float = 0.0, annualization_days: int = 252) -> tuple[float, float]:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan"), float("nan")

    var_b = r["benchmark"].var()
    if var_b == 0 or pd.isna(var_b):
        return float("nan"), float("nan")

    beta = float(r["target"].cov(r["benchmark"]) / var_b)
    target_ann = r["target"].mean() * annualization_days
    bench_ann = r["benchmark"].mean() * annualization_days
    alpha = float(target_ann - (risk_free_rate + beta * (bench_ann - risk_free_rate)))
    return beta, alpha


def correlation(target: pd.Series, benchmark: pd.Series) -> float:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan")
    return float(r["target"].corr(r["benchmark"]))


def tracking_error(target: pd.Series, benchmark: pd.Series, annualization_days: int = 252) -> float:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan")
    diff = r["target"] - r["benchmark"]
    return float(diff.std() * math.sqrt(annualization_days))


def information_ratio(target: pd.Series, benchmark: pd.Series, annualization_days: int = 252) -> float:
    excess = cagr(target) - cagr(benchmark)
    te = tracking_error(target, benchmark, annualization_days)
    if te == 0 or pd.isna(te):
        return float("nan")
    return float(excess / te)


def capture_ratios(target: pd.Series, benchmark: pd.Series) -> tuple[float, float]:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan"), float("nan")

    up = r[r["benchmark"] > 0]
    down = r[r["benchmark"] < 0]

    upside = (
        float(up["target"].mean() / up["benchmark"].mean())
        if not up.empty and up["benchmark"].mean() != 0
        else float("nan")
    )
    downside = (
        float(down["target"].mean() / down["benchmark"].mean())
        if not down.empty and down["benchmark"].mean() != 0
        else float("nan")
    )

    return upside, downside


def chart_subtitle(start: pd.Timestamp, end: pd.Timestamp, benchmark: str) -> str:
    return f"Period: {start.date()} to {end.date()} | Benchmark: {benchmark} | Source: OpenBB / yfinance"


def style_finance_axis(ax: Any) -> None:
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.spines["left"].set_color("#cbd5e1")
    ax.spines["bottom"].set_color("#cbd5e1")
    ax.grid(True, color=CHART_COLORS["grid"], linewidth=0.8, alpha=0.7)
    ax.tick_params(colors="#334155")
    ax.yaxis.label.set_color("#334155")
    ax.xaxis.label.set_color("#334155")


def add_subtitle(ax: Any, subtitle: str) -> None:
    ax.text(
        0,
        1.02,
        subtitle,
        transform=ax.transAxes,
        ha="left",
        va="bottom",
        fontsize=9,
        color="#475569",
    )


def annotate_last_value(ax: Any, series: pd.Series, label: str, color: str, percent: bool = False) -> None:
    series = series.dropna()
    if series.empty:
        return
    x = series.index[-1]
    y = series.iloc[-1]
    text = f"{label}: {fmt_percent(y)}" if percent else f"{label}: {fmt_number(y)}"
    ax.annotate(
        text,
        xy=(x, y),
        xytext=(8, 0),
        textcoords="offset points",
        color=color,
        fontsize=9,
        va="center",
        bbox=dict(boxstyle="round,pad=0.25", fc="white", ec=color, alpha=0.9),
    )


def build_price_summary(target_close: pd.Series, benchmark_close: pd.Series, risk_free_rate: float, annualization_days: int = 252) -> pd.DataFrame:
    beta, alpha = beta_alpha(target_close, benchmark_close, risk_free_rate, annualization_days)
    upside, downside = capture_ratios(target_close, benchmark_close)

    rows = [
        ("Total Return", total_return(target_close), total_return(benchmark_close)),
        ("CAGR", cagr(target_close), cagr(benchmark_close)),
        ("1Y Return", rolling_return(target_close, 252), rolling_return(benchmark_close, 252)),
        ("6M Return", rolling_return(target_close, 126), rolling_return(benchmark_close, 126)),
        ("3M Return", rolling_return(target_close, 63), rolling_return(benchmark_close, 63)),
        ("Max Drawdown", max_drawdown(target_close), max_drawdown(benchmark_close)),
        ("Annualized Volatility", annualized_volatility(target_close, annualization_days), annualized_volatility(benchmark_close, annualization_days)),
        ("Sharpe Ratio", sharpe_ratio(target_close, risk_free_rate, annualization_days), sharpe_ratio(benchmark_close, risk_free_rate, annualization_days)),
        ("Sortino Ratio", sortino_ratio(target_close, risk_free_rate, annualization_days), sortino_ratio(benchmark_close, risk_free_rate, annualization_days)),
        ("Calmar Ratio", calmar_ratio(target_close), calmar_ratio(benchmark_close)),
        ("Beta vs Benchmark", beta, None),
        ("Alpha vs Benchmark", alpha, None),
        ("Correlation vs Benchmark", correlation(target_close, benchmark_close), None),
        ("Tracking Error", tracking_error(target_close, benchmark_close, annualization_days), None),
        ("Information Ratio", information_ratio(target_close, benchmark_close, annualization_days), None),
        ("Upside Capture", upside, None),
        ("Downside Capture", downside, None),
    ]

    data = []
    for metric, target, bench in rows:
        diff = target - bench if bench is not None and not pd.isna(target) and not pd.isna(bench) else None
        data.append({"Metric": metric, "Target": target, "Benchmark": bench, "Difference": diff})

    return pd.DataFrame(data)


def plot_normalized_performance(normalized: pd.DataFrame, target: str, benchmark: str, path: Path) -> None:
    target_return = normalized[target].iloc[-1] / normalized[target].iloc[0] - 1
    benchmark_return = normalized[benchmark].iloc[-1] / normalized[benchmark].iloc[0] - 1
    title = (
        f"{target} outperformed {benchmark}, but risk metrics still matter"
        if target_return > benchmark_return
        else f"{target} lagged {benchmark} over the selected period"
    )
    fig, ax = plt.subplots(figsize=(13.5, 7))
    ax.plot(normalized.index, normalized[target], label=target, color=CHART_COLORS["target"], linewidth=2.2)
    ax.plot(normalized.index, normalized[benchmark], label=benchmark, color=CHART_COLORS["benchmark"], linewidth=2.0)
    ax.set_title(title, loc="left", color=CHART_COLORS["text"], pad=24)
    add_subtitle(ax, chart_subtitle(normalized.index[0], normalized.index[-1], benchmark))
    ax.set_xlabel("Date")
    ax.set_ylabel("Normalized Price: Start = 100")
    annotate_last_value(ax, normalized[target], f"{target} total return", CHART_COLORS["target"])
    annotate_last_value(ax, normalized[benchmark], f"{benchmark} total return", CHART_COLORS["benchmark"])
    style_finance_axis(ax)
    ax.legend(loc="upper left", frameon=False, ncols=2)
    plt.tight_layout()
    plt.savefig(path, dpi=220, bbox_inches="tight")
    plt.close()


def plot_actual_close_price(close: pd.DataFrame, target: str, benchmark: str, path: Path) -> None:
    fig, ax = plt.subplots(figsize=(13.5, 7))
    ax.plot(close.index, close[target], label=target, color=CHART_COLORS["target"], linewidth=2.2)
    ax.plot(close.index, close[benchmark], label=benchmark, color=CHART_COLORS["benchmark"], linewidth=2.0)
    ax.set_title(f"{target} and {benchmark}: actual close price", loc="left", color=CHART_COLORS["text"], pad=24)
    add_subtitle(ax, chart_subtitle(close.index[0], close.index[-1], benchmark))
    ax.set_xlabel("Date")
    ax.set_ylabel("Close Price")
    annotate_last_value(ax, close[target], f"{target} latest", CHART_COLORS["target"])
    annotate_last_value(ax, close[benchmark], f"{benchmark} latest", CHART_COLORS["benchmark"])
    style_finance_axis(ax)
    ax.legend(loc="upper left", frameon=False, ncols=2)
    plt.tight_layout()
    plt.savefig(path, dpi=220, bbox_inches="tight")
    plt.close()


def plot_drawdown(close: pd.DataFrame, target: str, benchmark: str, path: Path) -> None:
    dd = build_drawdown_frame(close, target, benchmark)
    target_mdd = dd[target].min()
    benchmark_mdd = dd[benchmark].min()
    title = (
        f"{target} outperformed with deeper drawdowns"
        if target_mdd < benchmark_mdd
        else f"{target} had a shallower drawdown profile than {benchmark}"
    )
    fig, ax = plt.subplots(figsize=(13.5, 6.5))
    ax.plot(dd.index, dd[target], label=target, color=CHART_COLORS["target"], linewidth=2.2)
    ax.plot(dd.index, dd[benchmark], label=benchmark, color=CHART_COLORS["benchmark"], linewidth=2.0)
    ax.fill_between(dd.index, dd[target], 0, color=CHART_COLORS["target"], alpha=0.08)
    ax.set_title(title, loc="left", color=CHART_COLORS["text"], pad=24)
    add_subtitle(ax, chart_subtitle(close.index[0], close.index[-1], benchmark))
    ax.set_xlabel("Date")
    ax.set_ylabel("Drawdown")
    ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda y, _: f"{y:.0%}"))
    annotate_last_value(ax, pd.Series([target_mdd], index=[dd[target].idxmin()]), f"{target} max drawdown", CHART_COLORS["negative"], percent=True)
    annotate_last_value(ax, pd.Series([benchmark_mdd], index=[dd[benchmark].idxmin()]), f"{benchmark} max drawdown", CHART_COLORS["benchmark"], percent=True)
    style_finance_axis(ax)
    ax.legend(loc="lower left", frameon=False, ncols=2)
    plt.tight_layout()
    plt.savefig(path, dpi=220, bbox_inches="tight")
    plt.close()


def build_drawdown_frame(close: pd.DataFrame, target: str, benchmark: str) -> pd.DataFrame:
    dd = pd.DataFrame(index=close.index)
    for col in [target, benchmark]:
        running_max = close[col].cummax()
        dd[col] = close[col] / running_max - 1
    return dd


def write_interactive_price_dashboard(
    close: pd.DataFrame,
    normalized: pd.DataFrame,
    target: str,
    benchmark: str,
    path: Path,
) -> None:
    if go is None or make_subplots is None:
        save_text(
            path,
            "<html><body><h1>Interactive chart unavailable</h1>"
            "<p>Install plotly to generate interactive charts: pip install plotly</p></body></html>",
        )
        return

    drawdown = build_drawdown_frame(close, target, benchmark)
    fig = make_subplots(
        rows=3,
        cols=1,
        shared_xaxes=True,
        vertical_spacing=0.07,
        subplot_titles=(
            "Actual close price",
            "Normalized performance: start = 100",
            "Drawdown from previous peak",
        ),
    )

    colors = {target: CHART_COLORS["target"], benchmark: CHART_COLORS["benchmark"]}
    for name in [target, benchmark]:
        fig.add_trace(
            go.Scatter(
                x=close.index,
                y=close[name],
                name=f"{name} close",
                mode="lines",
                line=dict(color=colors[name], width=2.4),
                hovertemplate=f"{name}<br>Date=%{{x|%Y-%m-%d}}<br>Close=%{{y:.2f}}<extra></extra>",
            ),
            row=1,
            col=1,
        )
        fig.add_trace(
            go.Scatter(
                x=normalized.index,
                y=normalized[name],
                name=f"{name} normalized",
                mode="lines",
                line=dict(color=colors[name], width=2.4),
                hovertemplate=f"{name}<br>Date=%{{x|%Y-%m-%d}}<br>Index=%{{y:.2f}}<extra></extra>",
                showlegend=False,
            ),
            row=2,
            col=1,
        )
        fig.add_trace(
            go.Scatter(
                x=drawdown.index,
                y=drawdown[name],
                name=f"{name} drawdown",
                mode="lines",
                line=dict(color=colors[name], width=2.4),
                hovertemplate=f"{name}<br>Date=%{{x|%Y-%m-%d}}<br>Drawdown=%{{y:.2%}}<extra></extra>",
                showlegend=False,
            ),
            row=3,
            col=1,
        )

    fig.update_layout(
        title={
            "text": f"{target} vs {benchmark}: price, relative performance, and drawdown",
            "x": 0.02,
            "xanchor": "left",
        },
        hovermode="x unified",
        template="simple_white",
        height=980,
        margin=dict(l=70, r=40, t=90, b=50),
        legend=dict(orientation="h", yanchor="bottom", y=1.03, xanchor="left", x=0),
        font=dict(family="Arial, sans-serif", size=13, color=CHART_COLORS["text"]),
        plot_bgcolor="white",
        paper_bgcolor="white",
    )
    fig.update_yaxes(title_text="Close Price", row=1, col=1)
    fig.update_yaxes(title_text="Start = 100", row=2, col=1)
    fig.update_yaxes(title_text="Drawdown", tickformat=".0%", row=3, col=1)
    fig.update_xaxes(
        rangeslider=dict(visible=True, thickness=0.04),
        rangeselector=dict(
            buttons=list(
                [
                    dict(count=3, label="3M", step="month", stepmode="backward"),
                    dict(count=6, label="6M", step="month", stepmode="backward"),
                    dict(count=1, label="1Y", step="year", stepmode="backward"),
                    dict(step="all", label="All"),
                ]
            )
        ),
        row=3,
        col=1,
    )
    fig.update_xaxes(showgrid=True, gridcolor="#e5e7eb")
    fig.update_yaxes(showgrid=True, gridcolor="#e5e7eb", zerolinecolor="#cbd5e1")
    fig.write_html(path, include_plotlyjs="cdn", full_html=True)


def write_metric_radar_chart(score_table: pd.DataFrame, path: Path) -> None:
    if go is None:
        save_text(
            path,
            "<html><body><h1>Score radar unavailable</h1>"
            "<p>Install plotly to generate interactive radar charts: pip install plotly</p></body></html>",
        )
        return

    components = score_table[score_table["Component"] != "Research Score"].copy()
    if components.empty:
        save_text(path, "<html><body><h1>No score components available</h1></body></html>")
        return

    categories = components["Component"].tolist()
    values = components["Score"].fillna(0).tolist()
    categories.append(categories[0])
    values.append(values[0])

    fig = go.Figure(
        data=[
            go.Scatterpolar(
                r=values,
                theta=categories,
                fill="toself",
                name="Research Radar",
            )
        ]
    )
    fig.update_layout(
        title={"text": "Research score components", "x": 0.03, "xanchor": "left"},
        template="simple_white",
        polar=dict(radialaxis=dict(visible=True, range=[0, 100])),
        height=620,
        font=dict(family="Arial, sans-serif", size=13, color=CHART_COLORS["text"]),
        paper_bgcolor="white",
    )
    fig.write_html(path, include_plotlyjs="cdn", full_html=True)


def plot_score_components(score_table: pd.DataFrame, path: Path) -> None:
    components = score_table[score_table["Component"] != "Research Score"].copy()
    if components.empty:
        return
    components = components.sort_values("Score")
    colors = [CHART_COLORS["negative"] if v < 45 else CHART_COLORS["accent"] if v < 60 else CHART_COLORS["positive"] for v in components["Score"]]
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.barh(components["Component"], components["Score"], color=colors)
    ax.set_title("What supports or weakens the research score", loc="left", color=CHART_COLORS["text"], pad=22)
    add_subtitle(ax, "Score components are heuristics, not investment recommendations")
    ax.set_xlabel("Score: 0-100")
    ax.set_xlim(0, 100)
    for i, value in enumerate(components["Score"]):
        ax.text(min(value + 1.5, 98), i, f"{value:.1f}", va="center", fontsize=9, color="#334155")
    style_finance_axis(ax)
    plt.tight_layout()
    plt.savefig(path, dpi=220, bbox_inches="tight")
    plt.close()


def plot_growth_quality(trends: pd.DataFrame, path: Path) -> None:
    if trends is None or trends.empty:
        return
    cols = [c for c in ["Revenue Growth YoY", "Gross Margin", "Operating Margin", "FCF Margin"] if c in trends]
    if not cols:
        return
    view = trends[cols].apply(pd.to_numeric, errors="coerce").dropna(how="all")
    if view.empty:
        return
    fig, ax = plt.subplots(figsize=(12.5, 6))
    palette = [CHART_COLORS["target"], CHART_COLORS["positive"], CHART_COLORS["accent"], CHART_COLORS["negative"]]
    for idx, col in enumerate(cols):
        series = view[col].dropna()
        if series.empty:
            continue
        ax.plot(series.index, series, marker="o", linewidth=2.0, label=col, color=palette[idx % len(palette)])
    ax.set_title("Growth quality: revenue, margins, and cash conversion", loc="left", color=CHART_COLORS["text"], pad=22)
    add_subtitle(ax, "Use this to check whether growth is supported by economics and cash flow")
    ax.set_xlabel("Fiscal Period")
    ax.set_ylabel("Ratio")
    ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda y, _: f"{y:.0%}"))
    style_finance_axis(ax)
    ax.legend(loc="best", frameon=False)
    plt.tight_layout()
    plt.savefig(path, dpi=220, bbox_inches="tight")
    plt.close()


def plot_ruin_risk_snapshot(ruin_risk: pd.DataFrame, path: Path) -> None:
    if ruin_risk is None or ruin_risk.empty:
        return
    view = ruin_risk[ruin_risk["Metric"].isin(["Net Debt / EBITDA", "Debt / FCF", "Cash Runway Years", "Balance Sheet Resilience Score"])].copy()
    if view.empty:
        return
    view["Display Value"] = pd.to_numeric(view["Value"], errors="coerce").fillna(0)
    colors = [CHART_COLORS["positive"] if m == "Balance Sheet Resilience Score" else CHART_COLORS["benchmark"] for m in view["Metric"]]
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.bar(view["Metric"], view["Display Value"], color=colors)
    ax.set_title("Balance sheet resilience: debt and cash-flow stress", loc="left", color=CHART_COLORS["text"], pad=22)
    add_subtitle(ax, "Higher resilience score means stronger balance-sheet resilience; this is not stock-price risk")
    ax.set_ylabel("Value")
    for i, value in enumerate(view["Display Value"]):
        ax.text(i, value, f"{value:.2f}", ha="center", va="bottom", fontsize=9, color="#334155")
    style_finance_axis(ax)
    plt.xticks(rotation=20, ha="right")
    plt.tight_layout()
    plt.savefig(path, dpi=220, bbox_inches="tight")
    plt.close()


# =============================================================================
# Financial analysis
# =============================================================================

def statement_to_time_series(statement: pd.DataFrame) -> pd.DataFrame:
    if statement is None or statement.empty:
        return pd.DataFrame()

    df = statement.copy()
    df.columns = pd.to_datetime(df.columns, errors="coerce")
    df = df.loc[:, df.columns.notna()]
    df = df.T.sort_index()
    df.columns = [str(c).strip() for c in df.columns]
    return df


def pick_series(df: pd.DataFrame, names: list[str]) -> pd.Series:
    if df is None or df.empty:
        return pd.Series(dtype="float64")

    name_map = {
        c.lower().replace(" ", "").replace("_", ""): c
        for c in df.columns
    }

    for name in names:
        key = name.lower().replace(" ", "").replace("_", "")
        if key in name_map:
            return pd.to_numeric(df[name_map[key]], errors="coerce")

    return pd.Series(index=df.index, dtype="float64")


def safe_ratio(numerator: pd.Series, denominator: pd.Series) -> pd.Series:
    denominator = denominator.replace({0: pd.NA})
    return numerator / denominator


def metric_cagr(series: pd.Series) -> float:
    s = series.dropna()
    if len(s) < 2:
        return float("nan")

    years = max((s.index[-1] - s.index[0]).days / 365, 1e-9)
    if s.iloc[0] <= 0 or s.iloc[-1] <= 0:
        return float("nan")

    return float((s.iloc[-1] / s.iloc[0]) ** (1 / years) - 1)


def metric_change(series: pd.Series) -> float:
    s = series.dropna()
    if len(s) < 2:
        return float("nan")
    return float(s.iloc[-1] - s.iloc[0])


def fetch_company_info(symbol: str) -> dict[str, Any]:
    last_exc: Exception | None = None
    ticker = yf.Ticker(symbol)
    for attempt in range(2):
        try:
            info = ticker.get_info()
            if info:
                info.setdefault("symbol", symbol.upper())
                return info
        except Exception as exc:
            last_exc = exc
            if attempt == 0:
                time.sleep(1)
    try:
        fast_info = dict(getattr(ticker, "fast_info", {}) or {})
        if fast_info:
            return {
                "symbol": symbol.upper(),
                "shortName": fast_info.get("lastPrice") and symbol.upper(),
                "currency": fast_info.get("currency"),
                "marketCap": fast_info.get("marketCap"),
            }
    except Exception:
        pass
    if last_exc is not None:
        print(f"[WARN] yfinance company info failed for {symbol}: {last_exc}")
    return {"symbol": symbol.upper()}


def build_company_profile(info: dict[str, Any]) -> pd.DataFrame:
    fields = [
        "shortName", "longName", "symbol", "quoteType", "sector", "industry",
        "country", "exchange", "currency", "marketCap", "enterpriseValue",
        "beta", "website",
    ]
    return pd.DataFrame(
        [{"Field": field, "Value": info.get(field)} for field in fields if info.get(field) is not None]
    )


def build_valuation_snapshot(info: dict[str, Any]) -> pd.DataFrame:
    groups = {
        "Market Size": ["marketCap", "enterpriseValue"],
        "Valuation Multiples": [
            "trailingPE", "forwardPE", "priceToSalesTrailing12Months",
            "priceToBook", "enterpriseToRevenue", "enterpriseToEbitda",
        ],
        "Profitability and Growth": [
            "grossMargins", "operatingMargins", "profitMargins",
            "returnOnEquity", "returnOnAssets", "revenueGrowth", "earningsGrowth",
        ],
        "Cash Flow and Debt": ["operatingCashflow", "freeCashflow", "totalCash", "totalDebt"],
        "Ownership": ["sharesOutstanding", "floatShares", "heldPercentInsiders", "heldPercentInstitutions"],
        "Price Range": ["fiftyTwoWeekLow", "fiftyTwoWeekHigh"],
    }
    rows = []
    for group, fields in groups.items():
        rows.extend(
            {"Group": group, "Metric": field, "Value": info.get(field)}
            for field in fields
            if info.get(field) is not None
        )
    return pd.DataFrame(
        rows,
        columns=["Group", "Metric", "Value"],
    )


def valuation_group_sections(valuation: pd.DataFrame) -> str:
    if valuation is None or valuation.empty or "Group" not in valuation.columns:
        return markdown_table(valuation, max_rows=50)

    def readable_valuation_table(df: pd.DataFrame) -> pd.DataFrame:
        table = df[["Metric", "Value"]].copy()

        def display(row: pd.Series) -> object:
            metric = str(row.get("Metric", ""))
            value = row.get("Value")
            try:
                value_float = float(value)
            except Exception:
                return value
            if metric in {"trailingPE", "forwardPE"} and value_float <= 0:
                return "Not applicable / profitability not established"
            if metric == "enterpriseToEbitda" and value_float <= 0:
                return "Not applicable / EBITDA negative"
            return value

        table["Value"] = table.apply(display, axis=1)
        return table

    sections = []
    for group, group_df in valuation.groupby("Group", sort=False):
        sections.append(f"### {group}")
        sections.append(markdown_table(readable_valuation_table(group_df), max_rows=50))
    return "\n\n".join(sections)


ZH_STATUS_LABELS = {
    "PASS": "通过",
    "WARNING": "有警告",
    "WARNING_DEGRADED": "降级警告",
    "FAIL": "未通过",
    "VERIFIED": "已验证",
    "WARNING_OVERALL": "需要复核",
    "UNVERIFIED": "未验证",
    "Watchlist": "观察名单",
    "High Priority Research": "高优先级研究",
    "Research More": "继续研究",
    "FOMO Risk / Weak Evidence": "证据不足，容易受情绪影响",
    "Avoid for Now / Data Weak": "暂不适合深入，数据或证据偏弱",
    "Mature Compounder": "成熟复利型公司",
    "Profitable Growth": "盈利成长型公司",
    "Speculative Growth": "投机成长型公司",
    "Unprofitable Growth": "未盈利成长型公司",
    "Unknown / Data-Limited Screening": "未知或数据不足，仅限初筛",
    "Hybrid Growth Compounder": "混合成长复利型公司",
    "Financials": "金融类公司",
    "Capital-Intensive Semiconductor Turnaround": "资本开支重的半导体制造转型",
    "Insurance-like Screening": "保险类初筛",
    "REIT-like Screening": "REIT 类初筛",
    "Consumer / Retail": "消费 / 零售",
    "Utilities / Infrastructure": "公用事业 / 基础设施",
    "Shipping / Airlines / Transport": "航运 / 航空 / 运输",
    "Cyclical": "周期型公司",
    "Cyclical / Asset Heavy": "周期或重资产公司",
    "ETF / Fund": "基金或 ETF",
    "Data Limited": "数据不足",
    "General Equity": "普通股票",
}


ZH_METRIC_LABELS = {
    "Metric": "指标",
    "Value": "数值",
    "Target": "标的",
    "Benchmark": "基准",
    "Difference": "差异",
    "Component": "组成部分",
    "Score": "分数",
    "Weight": "权重",
    "Revenue CAGR": "收入复合增速",
    "Revenue Growth Latest": "最新收入增速",
    "Gross Margin Latest": "最新毛利率",
    "Gross Margin Change": "毛利率变化",
    "Operating Margin Latest": "最新经营利润率",
    "Operating Margin Change": "经营利润率变化",
    "FCF Margin Latest": "最新自由现金流率",
    "FCF Margin Change": "自由现金流率变化",
    "Positive Net Income Years": "净利润为正年份",
    "Positive FCF Years": "自由现金流为正年份",
    "Total Return": "总收益",
    "CAGR": "年化收益",
    "Max Drawdown": "最大回撤",
    "Annualized Volatility": "年化波动率",
    "Sharpe Ratio": "夏普比率",
    "Sortino Ratio": "索提诺比率",
    "Calmar Ratio": "卡玛比率",
    "Beta vs Benchmark": "相对基准 Beta",
    "Alpha vs Benchmark": "相对基准 Alpha",
    "Tracking Error": "跟踪误差",
    "Information Ratio": "信息比率",
    "Upside Capture": "上涨捕获率",
    "Downside Capture": "下跌捕获率",
    "Balance Sheet Resilience Score": "资产负债表韧性分数",
    "Net Debt / EBITDA": "净债务 / EBITDA",
    "Debt / FCF": "债务 / 自由现金流",
    "Cash Runway Years": "现金可支撑年限",
    "Growth Score": "成长分数",
    "Profitability Score": "盈利能力分数",
    "Quality Trend Score": "质量趋势分数",
    "Risk Control Score": "风险控制分数",
    "Benchmark Score": "基准比较分数",
    "Valuation Sanity Score": "估值合理性分数",
    "Research Score": "研究分数",
    "Net Debt": "净债务",
    "EBITDA": "EBITDA",
    "marketCap": "市值",
    "enterpriseValue": "企业价值",
    "trailingPE": "滚动市盈率",
    "forwardPE": "预期市盈率",
    "priceToSalesTrailing12Months": "市销率",
    "priceToBook": "市净率",
    "enterpriseToRevenue": "企业价值 / 收入",
    "enterpriseToEbitda": "企业价值 / EBITDA",
    "grossMargins": "毛利率",
    "operatingMargins": "经营利润率",
    "profitMargins": "净利率",
    "returnOnEquity": "净资产收益率",
    "returnOnAssets": "资产收益率",
    "revenueGrowth": "收入增速",
    "earningsGrowth": "利润增速",
    "operatingCashflow": "经营现金流",
    "freeCashflow": "自由现金流",
    "totalCash": "现金总额",
    "totalDebt": "债务总额",
    "sharesOutstanding": "总股本",
    "floatShares": "流通股本",
    "heldPercentInsiders": "内部人持股比例",
    "heldPercentInstitutions": "机构持股比例",
    "fiftyTwoWeekLow": "52 周低点",
    "fiftyTwoWeekHigh": "52 周高点",
}


def zh_status(value: str) -> str:
    if value == "WARNING":
        return ZH_STATUS_LABELS["WARNING"]
    return ZH_STATUS_LABELS.get(value, value)


def zh_overall_status(value: str) -> str:
    if value == "WARNING":
        return ZH_STATUS_LABELS["WARNING_OVERALL"]
    return ZH_STATUS_LABELS.get(value, value)


def zh_term(term: str, term_style: str = "pure") -> str:
    label = ZH_METRIC_LABELS.get(term, term)
    if term_style == "bilingual" and label != term:
        return f"{label}（{term}）"
    return label


def zh_profile_value(value: Any) -> str:
    mapping = {
        "Aerospace / Space Systems": "航天 / 空间系统",
        "PARTIAL": "部分覆盖",
        "FULL": "完整覆盖",
        "SCREENING_ONLY": "仅限初筛",
        "UNKNOWN": "未知",
        "LOW": "低",
        "MEDIUM": "中",
        "HIGH": "高",
        "PS / EV Revenue / burn / dilution": "市销率 / 企业价值收入倍数 / 现金消耗 / 稀释",
        "PE / FCF sensitivity": "市盈率 / 自由现金流敏感性",
        "P/B / ROE": "市净率 / 净资产收益率",
        "Negative or unproven FCF": "自由现金流为负或尚未证明",
        "Positive FCF": "自由现金流为正",
        "backlog/order conversion": "backlog / 订单转化",
        "dilution plan": "潜在稀释计划",
    }
    if isinstance(value, list):
        return "、".join(zh_profile_value(item) for item in value)
    return mapping.get(str(value), str(value))


def localized_metric_table(
    df: pd.DataFrame,
    term_style: str = "pure",
    max_rows: int = 30,
    percent_columns: set[str] | None = None,
    score_columns: set[str] | None = None,
) -> str:
    if df is None or df.empty:
        return f"_{MISSING_VALUE_PLACEHOLDER}_"
    view = df.copy()
    if set(["Metric", "Value"]).issubset(view.columns):
        rows = []
        for _, row in view.head(max_rows).iterrows():
            metric = str(row["Metric"])
            rows.append({"指标": zh_term(metric, term_style), "数值": format_value_by_metric(metric, row["Value"])})
        return markdown_table(pd.DataFrame(rows), max_rows=max_rows)
    if set(["Component", "Score"]).issubset(view.columns):
        rows = []
        for _, row in view.head(max_rows).iterrows():
            component = str(row["Component"])
            rows.append({
                "组成部分": zh_term(component, term_style),
                "分数": fmt_score(row["Score"]),
                "权重": fmt_percent(row["Weight"]) if "Weight" in row else MISSING_VALUE_PLACEHOLDER,
            })
        return markdown_table(pd.DataFrame(rows), max_rows=max_rows)
    rename_cols = {col: ZH_METRIC_LABELS.get(str(col), str(col)) for col in view.columns}
    view = view.rename(columns=rename_cols)
    translated_percent_columns = {ZH_METRIC_LABELS.get(col, col) for col in (percent_columns or set())}
    translated_score_columns = {ZH_METRIC_LABELS.get(col, col) for col in (score_columns or set())}
    return markdown_table(view, max_rows=max_rows, percent_columns=translated_percent_columns, score_columns=translated_score_columns)


def chinese_verdict(symbol: str, benchmark: str, report_data: dict[str, Any]) -> str:
    raw = report_data["raw"]
    price_summary = raw["price_summary"]
    fundamental_summary = raw["fundamental_summary"]
    valuation = raw["valuation"]
    total = get_metric(price_summary, "Total Return", "Target")
    bench = get_metric(price_summary, "Total Return", "Benchmark")
    rev_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    pe = get_metric(valuation, "trailingPE")
    return (
        f"{symbol} 更像成熟现金流公司，不是高增长故事。"
        f"它在本周期总收益为 {fmt_percent(total)}，高于 {benchmark} 的 {fmt_percent(bench)}，"
        f"但收入复合增速只有约 {fmt_percent(rev_cagr)}，当前市盈率约 {fmt_number(pe)} 倍。"
        "所以这份报告的核心结论不是“公司差”，而是“估值已经要求利润率、现金流和回购继续守住”。"
    )


def clean_report_placeholders(text: str, language: str = "en") -> str:
    replacement = "当前数据未提供" if language == "zh" else "Not provided by the current data"
    for placeholder in [
        "[METRIC_MISSING_RAW]",
        "[FIELD_MISSING_PROVIDER]",
        "[PRIMARY_SOURCE_REQUIRED]",
        "[SEGMENT_DATA_MISSING]",
        "[METHOD_ASSUMPTION_MISSING]",
        "[PRICE_LABEL_UNVERIFIED]",
    ]:
        text = text.replace(placeholder, replacement)
    return text


def status_card_table(symbol: str, benchmark: str, start_date: str, end_date: str | None, rating: str, category: str, gate_status: dict[str, str]) -> str:
    rows = pd.DataFrame(
        [
            {"Item": "Target", "Value": symbol},
            {"Item": "Benchmark", "Value": benchmark},
            {"Item": "Period", "Value": f"{start_date} to {end_date or 'latest available'}"},
            {"Item": "Research Status", "Value": rating},
            {"Item": "Research Profile", "Value": category},
            {"Item": "Report Status", "Value": gate_status.get("OVERALL_REPORT_STATUS", "VERIFIED")},
            {"Item": "Data Verification", "Value": gate_status.get("DATA_VERIFICATION_STATUS", STATUS_PASS)},
            {"Item": "Thesis Verification", "Value": gate_status.get("THESIS_VERIFICATION_STATUS", STATUS_PASS)},
            {"Item": "Data Audit", "Value": gate_status.get("DATA_AUDIT_STATUS", STATUS_PASS)},
            {"Item": "Risk Method", "Value": gate_status.get("RISK_METHOD_STATUS", STATUS_PASS)},
            {"Item": "AI Analyst Gate", "Value": gate_status.get("AI_ANALYST_REVIEW_STATUS", STATUS_PASS)},
            {"Item": "Language Check", "Value": gate_status.get("LANGUAGE_LINT_STATUS", STATUS_PASS)},
            {"Item": "Price Label Check", "Value": gate_status.get("PRICE_LABEL_CHECK_STATUS", STATUS_PASS)},
        ]
    )
    return markdown_table(rows, max_rows=20)


def zh_status_card_table(symbol: str, benchmark: str, start_date: str, end_date: str | None, rating: str, category: str, gate_status: dict[str, str]) -> str:
    rows = pd.DataFrame(
        [
            {"项目": "标的", "内容": symbol},
            {"项目": "基准", "内容": benchmark},
            {"项目": "期间", "内容": f"{start_date} 至 {end_date or '最新可得数据'}"},
            {"项目": "研究状态", "内容": ZH_STATUS_LABELS.get(rating, rating)},
            {"项目": "研究类型", "内容": ZH_STATUS_LABELS.get(category, category)},
            {"项目": "报告状态", "内容": zh_overall_status(gate_status.get("OVERALL_REPORT_STATUS", "VERIFIED"))},
            {"项目": "数字验证", "内容": zh_status(gate_status.get("DATA_VERIFICATION_STATUS", STATUS_PASS))},
            {"项目": "主线验证", "内容": zh_status(gate_status.get("THESIS_VERIFICATION_STATUS", STATUS_PASS))},
            {"项目": "数据审计", "内容": zh_status(gate_status.get("DATA_AUDIT_STATUS", STATUS_PASS))},
            {"项目": "风险方法", "内容": zh_status(gate_status.get("RISK_METHOD_STATUS", STATUS_PASS))},
            {"项目": "AI 二次复核", "内容": zh_status(gate_status.get("AI_ANALYST_REVIEW_STATUS", STATUS_PASS))},
            {"项目": "语言检查", "内容": zh_status(gate_status.get("LANGUAGE_LINT_STATUS", STATUS_PASS))},
            {"项目": "价格标签校验", "内容": zh_status(gate_status.get("PRICE_LABEL_CHECK_STATUS", STATUS_PASS))},
        ]
    )
    return markdown_table(rows, max_rows=20)


def english_chart_walkthrough(symbol: str, benchmark: str, charts: dict[str, str]) -> str:
    return f"""## 8. Chart Walkthrough

### Actual Close Price

![{symbol} vs {benchmark} actual close price]({charts["actual"]})

**What this chart shows:** This chart shows the raw closing price path for {symbol} and {benchmark}. It is useful for seeing trend shape, gaps, highs, and drawdown locations.

**What the report reads from it:** {symbol} has moved more aggressively than the benchmark during this window. The chart sets up the later question: whether the extra movement produced enough extra return.

**How not to misread it:** This is not a valuation chart. A higher price line does not mean a security is more expensive, and a lower price line does not mean it is cheaper.

**What to check next:** Use normalized performance and drawdown before judging relative performance.

### Normalized Performance

![{symbol} vs {benchmark} normalized performance]({charts["normalized"]})

**What this chart shows:** Both series start at 100, so the chart compares cumulative return over the same period.

**What the report reads from it:** If {symbol} ends above {benchmark}, it outperformed in raw return terms. That is only the first layer of the research question.

**How not to misread it:** Outperformance does not prove the stock is cheap, safe, or likely to keep outperforming.

**What to check next:** Compare the return spread with volatility, drawdown, Sharpe, and the business evidence.

### Drawdown

![{symbol} vs {benchmark} drawdown]({charts["drawdown"]})

**What this chart shows:** Drawdown measures how far the asset fell from its previous peak.

**What the report reads from it:** Deeper drawdowns mean the holding path was harder, even if final returns were strong.

**How not to misread it:** Drawdown is not bankruptcy risk. It measures investor pain, not whether the company can survive.

**What to check next:** Connect drawdown with balance-sheet resilience and valuation pressure.
"""


def chinese_chart_walkthrough(symbol: str, benchmark: str, charts: dict[str, str]) -> str:
    return f"""## 8. 图表解读

### 实际收盘价

![{symbol} 与 {benchmark} 实际收盘价]({charts["actual"]})

**图表看什么：** 这张图展示 {symbol} 和 {benchmark} 的实际收盘价走势。它适合看价格趋势、阶段高点和回撤位置，但不能直接比较两者收益率，因为价格基数不同。

**读出来的结论：** {symbol} 在这个周期里的价格运动更明显，持有体验也更不平稳。只看实际价格线，容易高估“涨得多”的意义。

**不要误读：** 实际价格图不是估值图。股价更高不代表更贵，股价更低也不代表更便宜。

**下一步怎么查：** 继续看归一化表现和回撤，再结合风险指标判断这段超额收益是否值得。

### 归一化表现

![{symbol} 与 {benchmark} 归一化表现]({charts["normalized"]})

**图表看什么：** 这张图把 {symbol} 和 {benchmark} 都从 100 开始计算，目的是比较同一周期内谁的累计收益更高。

**读出来的结论：** 如果 {symbol} 的线明显更高，说明它过去这段时间跑赢基准。但这不是免费午餐，还要看波动、回撤和估值。

**不要误读：** 跑赢基准不等于现在值得买。它只能说明过去这段时间收益更高，不能证明未来还能继续跑赢。

**下一步怎么查：** 对照最大回撤、夏普比率和业务质量，看收益是不是用更高风险换来的。

### 回撤

![{symbol} 与 {benchmark} 回撤]({charts["drawdown"]})

**图表看什么：** 这张图看的是从阶段高点跌下来多少。它不直接判断公司质量，只展示持有过程中的账户压力。

**读出来的结论：** 如果 {symbol} 的回撤比基准更深，说明它虽然可能赚得更多，但持有人要承受更大的波动和心理压力。

**不要误读：** 回撤不是破产风险。回撤深不等于公司会出问题，它说明买入价格、仓位和持仓纪律很重要。

**下一步怎么查：** 把回撤和资产负债表韧性、现金流、估值压力放在一起看。
"""


def english_key_questions(symbol: str, benchmark: str, price_summary: pd.DataFrame, fundamental_summary: pd.DataFrame, valuation: pd.DataFrame) -> str:
    total = get_metric(price_summary, "Total Return", "Target")
    bench = get_metric(price_summary, "Total Return", "Benchmark")
    dd = get_metric(price_summary, "Max Drawdown", "Target")
    bench_dd = get_metric(price_summary, "Max Drawdown", "Benchmark")
    rev_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    pe = get_metric(valuation, "trailingPE")
    ps = get_metric(valuation, "priceToSalesTrailing12Months")
    return f"""## 6. Key Questions and Answers

### Question: Why not simply own {benchmark}?

**Answer:** {symbol} has to justify the extra single-stock risk. If it beat {benchmark}, the next question is whether the extra return paid enough for deeper drawdowns and less diversification.

**Evidence:** Total return was {fmt_percent(total)} for {symbol} versus {fmt_percent(bench)} for {benchmark}. Max drawdown was {fmt_percent(dd)} for {symbol} versus {fmt_percent(bench_dd)} for {benchmark}.

**Boundary:** This only describes the selected historical period. It does not predict future outperformance.

### Question: Is {symbol} a high-growth company?

**Answer:** Not from this data. The current profile is closer to a mature cash-flow company than a high-growth story.

**Evidence:** Revenue CAGR is about {fmt_percent(rev_cagr)}, while profitability and cash-flow metrics carry more of the research case.

**Boundary:** Slow revenue growth does not make the company weak. It changes the question from growth acceleration to margin durability, cash generation, and valuation support.

### Question: Where is valuation pressure?

**Answer:** The pressure is in the amount of future stability already embedded in the multiple. A mature company at a high PE needs strong margins, cash flow, and buybacks to defend the valuation.

**Evidence:** Trailing PE is about {fmt_number(pe)}x and price-to-sales is about {fmt_number(ps)}x.

**Boundary:** Expensive does not mean imminent downside. The next check is whether segment mix, margins, and EPS growth can support the current multiple.
"""


def chinese_key_questions(symbol: str, benchmark: str, price_summary: pd.DataFrame, fundamental_summary: pd.DataFrame, valuation: pd.DataFrame) -> str:
    total = get_metric(price_summary, "Total Return", "Target")
    bench = get_metric(price_summary, "Total Return", "Benchmark")
    dd = get_metric(price_summary, "Max Drawdown", "Target")
    bench_dd = get_metric(price_summary, "Max Drawdown", "Benchmark")
    rev_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    pe = get_metric(valuation, "trailingPE")
    ps = get_metric(valuation, "priceToSalesTrailing12Months")
    return f"""## 6. 关键问题与回答

### 问题：为什么不直接买 {benchmark}？

**回答：** 如果只看收益，{symbol} 在这个周期里可能比 {benchmark} 更强。但这不自动证明它更适合持有，因为单一个股通常要承担更高波动、更深回撤和更强的估值风险。

**证据：** {symbol} 的总收益为 {fmt_percent(total)}，{benchmark} 为 {fmt_percent(bench)}。{symbol} 的最大回撤为 {fmt_percent(dd)}，{benchmark} 为 {fmt_percent(bench_dd)}。

**边界：** 这只能说明过去这段时间的风险收益特征，不能预测未来。下一步要判断的是：未来是否还有足够超额收益，来补偿更高的个股风险。

### 问题：{symbol} 是高增长公司吗？

**回答：** 不是。它现在更像成熟现金流公司，而不是高速增长公司。

**证据：** 报告中的收入复合增速约为 {fmt_percent(rev_cagr)}。真正支撑研究价值的，不是收入爆发，而是利润率、自由现金流、回购和估值能否维持。

**边界：** 这不代表公司差。它只是说明当前投资逻辑不是“收入爆发”，而是“高利润率、强现金流和估值韧性”。

### 问题：当前估值贵在哪里？

**回答：** 贵在市场已经提前支付了很多未来稳定性的价格。对一家收入增速不快的成熟公司来说，约 {fmt_number(pe)} 倍市盈率需要很强的利润率、现金流和回购来支撑。

**证据：** 报告显示市盈率约 {fmt_number(pe)} 倍，市销率约 {fmt_number(ps)} 倍，而收入复合增速并不高。

**边界：** 估值高不等于马上会跌。真正要验证的是：服务业务、毛利率和回购能不能继续撑住这个倍数。
"""


def fetch_money_source_and_flow(symbol: str, out_dir: Path, years: int) -> pd.DataFrame:
    ticker = yf.Ticker(symbol)

    income = statement_to_time_series(ticker.financials)
    cashflow = statement_to_time_series(ticker.cashflow)
    balance = statement_to_time_series(ticker.balance_sheet)

    if not income.empty:
        income.to_csv(out_dir / f"{safe_symbol(symbol)}_income_statement_yfinance.csv")
    if not cashflow.empty:
        cashflow.to_csv(out_dir / f"{safe_symbol(symbol)}_cash_flow_yfinance.csv")
    if not balance.empty:
        balance.to_csv(out_dir / f"{safe_symbol(symbol)}_balance_sheet_yfinance.csv")

    idx = income.index.union(cashflow.index).union(balance.index).sort_values()
    trends = pd.DataFrame(index=idx)

    revenue = pick_series(income, ["Total Revenue", "Operating Revenue"])
    gross_profit = pick_series(income, ["Gross Profit"])
    operating_income = pick_series(income, ["Operating Income", "Operating Income Loss"])
    net_income = pick_series(income, ["Net Income", "Net Income Common Stockholders"])
    operating_cash_flow = pick_series(cashflow, ["Operating Cash Flow", "Total Cash From Operating Activities"])
    capex = pick_series(cashflow, ["Capital Expenditure", "Capital Expenditures", "Capital Expenditure Reported"])
    free_cash_flow = pick_series(cashflow, ["Free Cash Flow"])

    if free_cash_flow.dropna().empty and not operating_cash_flow.dropna().empty:
        free_cash_flow = operating_cash_flow.add(capex, fill_value=0)

    trends["Revenue"] = revenue.reindex(idx)
    trends["Gross Profit"] = gross_profit.reindex(idx)
    trends["Operating Income"] = operating_income.reindex(idx)
    trends["Net Income"] = net_income.reindex(idx)
    trends["Operating Cash Flow"] = operating_cash_flow.reindex(idx)
    trends["Capital Expenditure"] = capex.reindex(idx)
    trends["Free Cash Flow"] = free_cash_flow.reindex(idx)

    trends["Revenue Growth YoY"] = trends["Revenue"].pct_change()
    trends["Gross Margin"] = safe_ratio(trends["Gross Profit"], trends["Revenue"])
    trends["Operating Margin"] = safe_ratio(trends["Operating Income"], trends["Revenue"])
    trends["Net Margin"] = safe_ratio(trends["Net Income"], trends["Revenue"])
    trends["FCF Margin"] = safe_ratio(trends["Free Cash Flow"], trends["Revenue"])

    trends = drop_empty_rows(trends)

    if years > 0:
        trends = trends.tail(years)

    trends.to_csv(out_dir / f"{safe_symbol(symbol)}_money_source_and_flow.csv")
    return trends


def build_fundamental_summary(trends: pd.DataFrame) -> pd.DataFrame:
    if trends is None or trends.empty:
        return pd.DataFrame(columns=["Metric", "Value"])

    def latest(col: str) -> float:
        if col not in trends or trends[col].dropna().empty:
            return float("nan")
        return float(trends[col].dropna().iloc[-1])

    rows = [
        ("Revenue CAGR", metric_cagr(trends["Revenue"]) if "Revenue" in trends else float("nan")),
        ("Revenue Growth Latest", latest("Revenue Growth YoY")),
        ("Gross Margin Latest", latest("Gross Margin")),
        ("Gross Margin Change", metric_change(trends["Gross Margin"]) if "Gross Margin" in trends else float("nan")),
        ("Operating Margin Latest", latest("Operating Margin")),
        ("Operating Margin Change", metric_change(trends["Operating Margin"]) if "Operating Margin" in trends else float("nan")),
        ("FCF Margin Latest", latest("FCF Margin")),
        ("FCF Margin Change", metric_change(trends["FCF Margin"]) if "FCF Margin" in trends else float("nan")),
        ("Positive Net Income Years", int((trends["Net Income"].dropna() > 0).sum()) if "Net Income" in trends else 0),
        ("Positive FCF Years", int((trends["Free Cash Flow"].dropna() > 0).sum()) if "Free Cash Flow" in trends else 0),
    ]
    return pd.DataFrame(rows, columns=["Metric", "Value"])


def latest_trend_value(trends: pd.DataFrame, column: str) -> float:
    if trends is None or trends.empty or column not in trends or trends[column].dropna().empty:
        return float("nan")
    return float(trends[column].dropna().iloc[-1])


def safe_info_float(info: dict[str, Any], key: str) -> float:
    try:
        value = info.get(key)
        if value is None or pd.isna(value):
            return float("nan")
        return float(value)
    except Exception:
        return float("nan")


def classify_research_category(info: dict[str, Any], fundamental_summary: pd.DataFrame) -> str:
    quote_type = str(info.get("quoteType", "")).upper()
    sector = str(info.get("sector", "")).lower()
    revenue_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    fcf_margin = get_metric(fundamental_summary, "FCF Margin Latest")
    operating_margin = get_metric(fundamental_summary, "Operating Margin Latest")
    market_cap = safe_info_float(info, "marketCap")

    if quote_type in FUND_QUOTE_TYPES:
        return "ETF / Fund"
    if "financial" in sector:
        return "Financials"
    if any(word in sector for word in ["energy", "utilities", "basic materials"]):
        return "Cyclical / Asset Heavy"
    if pd.isna(revenue_cagr) and pd.isna(fcf_margin):
        return "Data Limited"
    if not pd.isna(revenue_cagr) and revenue_cagr >= 0.20 and (pd.isna(fcf_margin) or fcf_margin <= 0):
        return "Speculative Growth"
    if not pd.isna(revenue_cagr) and revenue_cagr >= 0.15 and not pd.isna(fcf_margin) and fcf_margin > 0:
        return "Profitable Growth"
    if market_cap >= 100_000_000_000 and not pd.isna(operating_margin) and operating_margin > 0.15:
        return "Mature Compounder"
    return "General Equity"


def score_weights_for_category(category: str) -> dict[str, float]:
    profiles = {
        "Mature Compounder": {
            "Growth Score": 0.14,
            "Profitability Score": 0.28,
            "Quality Trend Score": 0.18,
            "Risk Control Score": 0.18,
            "Benchmark Score": 0.14,
            "Valuation Sanity Score": 0.08,
        },
        "Speculative Growth": {
            "Growth Score": 0.30,
            "Profitability Score": 0.10,
            "Quality Trend Score": 0.14,
            "Risk Control Score": 0.20,
            "Benchmark Score": 0.14,
            "Valuation Sanity Score": 0.12,
        },
        "Profitable Growth": {
            "Growth Score": 0.26,
            "Profitability Score": 0.22,
            "Quality Trend Score": 0.16,
            "Risk Control Score": 0.14,
            "Benchmark Score": 0.14,
            "Valuation Sanity Score": 0.08,
        },
        "Cyclical / Asset Heavy": {
            "Growth Score": 0.12,
            "Profitability Score": 0.20,
            "Quality Trend Score": 0.12,
            "Risk Control Score": 0.26,
            "Benchmark Score": 0.12,
            "Valuation Sanity Score": 0.18,
        },
        "Financials": {
            "Growth Score": 0.10,
            "Profitability Score": 0.18,
            "Quality Trend Score": 0.12,
            "Risk Control Score": 0.26,
            "Benchmark Score": 0.16,
            "Valuation Sanity Score": 0.18,
        },
    }
    return profiles.get(
        category,
        {
            "Growth Score": 0.22,
            "Profitability Score": 0.22,
            "Quality Trend Score": 0.16,
            "Risk Control Score": 0.16,
            "Benchmark Score": 0.16,
            "Valuation Sanity Score": 0.08,
        },
    )


def build_ruin_risk_snapshot(info: dict[str, Any], trends: pd.DataFrame) -> pd.DataFrame:
    total_debt = safe_info_float(info, "totalDebt")
    total_cash = safe_info_float(info, "totalCash")
    ebitda = safe_info_float(info, "ebitda")
    fcf = safe_info_float(info, "freeCashflow")
    if pd.isna(fcf):
        fcf = latest_trend_value(trends, "Free Cash Flow")

    net_debt = total_debt - total_cash if not pd.isna(total_debt) and not pd.isna(total_cash) else float("nan")
    net_debt_to_ebitda = net_debt / ebitda if not pd.isna(net_debt) and not pd.isna(ebitda) and ebitda != 0 else float("nan")
    debt_to_fcf = total_debt / fcf if not pd.isna(total_debt) and not pd.isna(fcf) and fcf > 0 else float("nan")
    cash_runway = total_cash / abs(fcf) if not pd.isna(total_cash) and not pd.isna(fcf) and fcf < 0 else float("nan")

    stress_score = 50.0
    if not pd.isna(net_debt_to_ebitda):
        stress_score += clamp(net_debt_to_ebitda * 12, 0, 35)
    if not pd.isna(debt_to_fcf):
        stress_score += clamp((debt_to_fcf - 3) * 8, 0, 25)
    if not pd.isna(cash_runway):
        stress_score += 25 if cash_runway < 2 else 10 if cash_runway < 4 else 0
    if not pd.isna(fcf) and fcf < 0 and pd.isna(cash_runway):
        stress_score += 20
    stress_score = clamp(stress_score)
    resilience_score = clamp(100 - stress_score)

    rows = [
        ("Net Debt", net_debt, "Total debt minus cash. Negative is net cash."),
        ("EBITDA", ebitda, "Provider EBITDA, when available."),
        ("Net Debt / EBITDA", net_debt_to_ebitda, "Debt-load proxy. Higher values deserve manual stress testing."),
        ("Debt / FCF", debt_to_fcf, "Debt compared with free cash flow. Not useful when FCF is negative."),
        ("Cash Runway Years", cash_runway, "Approximate years of cash runway when FCF is negative."),
        ("Balance Sheet Resilience Score", resilience_score, "Higher score = stronger balance sheet resilience. This score measures balance-sheet resilience, not stock-price volatility."),
    ]
    return pd.DataFrame(rows, columns=["Metric", "Value", "Interpretation"])


def build_margin_stress(
    account_equity: float | None,
    margin_loan: float | None,
    stress_drops: list[float],
) -> pd.DataFrame:
    if account_equity is None or margin_loan is None or account_equity <= 0:
        return pd.DataFrame(columns=["Scenario", "Portfolio Value", "Margin Loan", "Equity Cushion", "Loan / Value"])

    rows = []
    for drop in stress_drops:
        portfolio_value = account_equity * (1 - drop)
        cushion = portfolio_value - margin_loan
        loan_to_value = margin_loan / portfolio_value if portfolio_value > 0 else float("nan")
        rows.append(
            {
                "Scenario": f"{fmt_percent(-drop)} portfolio shock",
                "Portfolio Value": portfolio_value,
                "Margin Loan": margin_loan,
                "Equity Cushion": cushion,
                "Loan / Value": loan_to_value,
            }
        )
    return pd.DataFrame(rows)


# =============================================================================
# Score
# =============================================================================

def clamp(value: float, lo: float = 0.0, hi: float = 100.0) -> float:
    if pd.isna(value):
        return 0.0
    return max(lo, min(hi, value))


def get_metric(df: pd.DataFrame, metric: str, column: str = "Value") -> float:
    try:
        row = df[df["Metric"] == metric]
        if row.empty:
            return float("nan")
        value = row.iloc[0][column]
        if value is None:
            return float("nan")
        return float(value)
    except Exception:
        return float("nan")


def get_component_score(score_table: pd.DataFrame, component: str) -> float:
    try:
        row = score_table[score_table["Component"] == component]
        if row.empty and component == "Research Score":
            row = score_table[score_table["Component"] == "Research Potential Score"]
        if row.empty and component == "Research Potential Score":
            row = score_table[score_table["Component"] == "Research Score"]
        if row.empty:
            return float("nan")
        return float(row.iloc[0]["Score"])
    except Exception:
        return float("nan")


def growth_score(revenue_cagr: float, latest_growth: float) -> float:
    scores = []
    if not pd.isna(revenue_cagr):
        scores.append(clamp(30 + revenue_cagr * 300))
    if not pd.isna(latest_growth):
        scores.append(clamp(30 + latest_growth * 250))
    return sum(scores) / len(scores) if scores else 0.0


def profitability_score(gross_margin: float, operating_margin: float, fcf_margin: float) -> float:
    scores = []
    if not pd.isna(gross_margin):
        scores.append(clamp(gross_margin * 150))
    if not pd.isna(operating_margin):
        scores.append(clamp(40 + operating_margin * 200))
    if not pd.isna(fcf_margin):
        scores.append(clamp(40 + fcf_margin * 200))
    return sum(scores) / len(scores) if scores else 0.0


def quality_trend_score(gross_margin_change: float, operating_margin_change: float, fcf_margin_change: float) -> float:
    scores = []
    for value in [gross_margin_change, operating_margin_change, fcf_margin_change]:
        if not pd.isna(value):
            scores.append(clamp(50 + value * 300))
    return sum(scores) / len(scores) if scores else 0.0


def risk_control_score(max_dd: float, volatility: float, beta: float) -> float:
    scores = []
    if not pd.isna(max_dd):
        scores.append(clamp(100 + max_dd * 160))  # drawdown is negative
    if not pd.isna(volatility):
        scores.append(clamp(100 - volatility * 180))
    if not pd.isna(beta):
        scores.append(clamp(100 - max(0.0, beta - 1.0) * 35))
    return sum(scores) / len(scores) if scores else 0.0


def benchmark_score(excess_cagr: float, information_ratio_value: float, sharpe_diff: float) -> float:
    scores = []
    if not pd.isna(excess_cagr):
        scores.append(clamp(50 + excess_cagr * 250))
    if not pd.isna(information_ratio_value):
        scores.append(clamp(50 + information_ratio_value * 25))
    if not pd.isna(sharpe_diff):
        scores.append(clamp(50 + sharpe_diff * 35))
    return sum(scores) / len(scores) if scores else 0.0


def valuation_sanity_score(info: dict[str, Any]) -> float:
    # Simple penalty system. This is NOT intrinsic valuation.
    score = 70.0

    checks = [
        ("priceToSalesTrailing12Months", 5.0, 3.0),
        ("enterpriseToRevenue", 6.0, 3.0),
        ("trailingPE", 30.0, 0.6),
        ("forwardPE", 30.0, 0.4),
        ("enterpriseToEbitda", 20.0, 0.7),
    ]

    for key, threshold, penalty in checks:
        try:
            value = info.get(key)
            if value is not None and not pd.isna(value) and float(value) > threshold:
                score -= (float(value) - threshold) * penalty
        except Exception:
            continue

    return clamp(score)


def build_research_score(price_summary: pd.DataFrame, fundamental_summary: pd.DataFrame, info: dict[str, Any]) -> pd.DataFrame:
    if is_fund_like(info) or fundamental_summary is None or fundamental_summary.empty:
        return pd.DataFrame(
            [
                {"Component": "Research Score", "Score": float("nan"), "Weight": 1.0, "Profile": "ETF / Fund"},
            ]
        )

    revenue_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    latest_growth = get_metric(fundamental_summary, "Revenue Growth Latest")
    gross_margin = get_metric(fundamental_summary, "Gross Margin Latest")
    operating_margin = get_metric(fundamental_summary, "Operating Margin Latest")
    fcf_margin = get_metric(fundamental_summary, "FCF Margin Latest")
    gross_margin_change = get_metric(fundamental_summary, "Gross Margin Change")
    operating_margin_change = get_metric(fundamental_summary, "Operating Margin Change")
    fcf_margin_change = get_metric(fundamental_summary, "FCF Margin Change")

    max_dd = get_metric(price_summary, "Max Drawdown", "Target")
    volatility = get_metric(price_summary, "Annualized Volatility", "Target")
    beta = get_metric(price_summary, "Beta vs Benchmark", "Target")
    excess_cagr = get_metric(price_summary, "CAGR", "Difference")
    information_ratio_value = get_metric(price_summary, "Information Ratio", "Target")
    sharpe_diff = get_metric(price_summary, "Sharpe Ratio", "Difference")

    components = {
        "Growth Score": growth_score(revenue_cagr, latest_growth),
        "Profitability Score": profitability_score(gross_margin, operating_margin, fcf_margin),
        "Quality Trend Score": quality_trend_score(gross_margin_change, operating_margin_change, fcf_margin_change),
        "Risk Control Score": risk_control_score(max_dd, volatility, beta),
        "Benchmark Score": benchmark_score(excess_cagr, information_ratio_value, sharpe_diff),
        "Valuation Sanity Score": valuation_sanity_score(info),
    }

    category = classify_research_category(info, fundamental_summary)
    weights = score_weights_for_category(category)

    total = sum(components[k] * weights[k] for k in components)
    rows = [{"Component": k, "Score": components[k], "Weight": weights[k], "Profile": category} for k in components]
    rows.append({"Component": "Research Score", "Score": total, "Weight": 1.0, "Profile": category})
    return pd.DataFrame(rows)


def rating_from_score(score: float) -> str:
    if pd.isna(score):
        return "Data Insufficient"
    if score >= 75:
        return "High Priority Research"
    if score >= 60:
        return "Watchlist"
    if score >= 45:
        return "Research More"
    if score >= 30:
        return "FOMO Risk / Weak Evidence"
    return "Avoid for Now / Data Weak"


# =============================================================================
# Interpretation
# =============================================================================

def business_summary(info: dict[str, Any], max_chars: int = 1500) -> str:
    summary = info.get("longBusinessSummary")
    if not summary:
        return "_No automatic business summary available. Add manual narrative from 10-K / company IR._"
    summary = " ".join(str(summary).split())
    if len(summary) > max_chars:
        return summary[:max_chars].rstrip() + "..."
    return summary


def benchmark_explanation(benchmark: str) -> str:
    benchmark = benchmark.upper()
    if benchmark == "SPY":
        return "SPY is a broad S&P 500 benchmark. It tests whether the stock deserves capital compared with a simple broad-market ETF."
    if benchmark == "VOO":
        return "VOO is a low-cost S&P 500 ETF and a practical long-term alternative for many investors."
    if benchmark == "QQQ":
        return "QQQ is a technology/growth-heavy benchmark, useful for large technology and growth stocks."
    return f"{benchmark} is the chosen benchmark. The comparison measures opportunity cost versus holding {benchmark}."


def generate_key_takeaways(
    symbol: str,
    benchmark: str,
    price_summary: pd.DataFrame,
    fundamental_summary: pd.DataFrame,
    score_table: pd.DataFrame,
    info: dict[str, Any],
) -> list[str]:
    takeaways = []

    if is_fund_like(info):
        takeaways.append(
            f"{symbol} appears to be a fund-like instrument, so company financial statement analysis was skipped."
        )

    total_diff = get_metric(price_summary, "Total Return", "Difference")
    cagr_diff = get_metric(price_summary, "CAGR", "Difference")
    sharpe_diff = get_metric(price_summary, "Sharpe Ratio", "Difference")
    mdd_diff = get_metric(price_summary, "Max Drawdown", "Difference")
    revenue_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    gross_margin = get_metric(fundamental_summary, "Gross Margin Latest")
    fcf_margin = get_metric(fundamental_summary, "FCF Margin Latest")
    valuation_score = get_component_score(score_table, "Valuation Sanity Score")
    total_score = get_component_score(score_table, "Research Score")
    category = classify_research_category(info, fundamental_summary)
    ps = safe_info_float(info, "priceToSalesTrailing12Months")
    pe = safe_info_float(info, "trailingPE")

    if not pd.isna(total_diff) and not pd.isna(cagr_diff):
        if total_diff > 0 and cagr_diff > 0:
            if not pd.isna(revenue_cagr) and revenue_cagr < 0.05:
                takeaways.append(
                    f"{symbol} beat {benchmark}, but the outperformance was not supported by fast revenue growth; the thesis depends more on margins, cash flow, buybacks, and market willingness to keep paying a premium multiple."
                )
            else:
                takeaways.append(
                    f"{symbol} beat {benchmark} on both total return and annualized compounding, so the first question is whether the business quality justifies the extra single-stock risk."
                )
        elif total_diff < 0 and cagr_diff < 0:
            takeaways.append(
                f"{symbol} lagged {benchmark} on both total return and annualized compounding, which weakens the opportunity-cost case unless the forward thesis has changed materially."
            )
        else:
            takeaways.append(
                f"{symbol}'s relative performance versus {benchmark} was mixed, so the benchmark comparison should be treated as inconclusive rather than supportive."
            )
    elif not pd.isna(total_diff):
        takeaways.append(
            f"{symbol} {'outperformed' if total_diff > 0 else 'underperformed'} {benchmark} in total return over the selected period."
        )
    elif not pd.isna(cagr_diff):
        takeaways.append(
            f"{symbol}'s CAGR was {'higher' if cagr_diff > 0 else 'lower'} than {benchmark} over the selected period."
        )

    raw_return_up_risk_efficiency_down = (
        not pd.isna(cagr_diff) and not pd.isna(sharpe_diff) and cagr_diff > 0 and sharpe_diff < 0
    )
    raw_return_up_deeper_drawdown = (
        not pd.isna(cagr_diff) and not pd.isna(mdd_diff) and cagr_diff > 0 and mdd_diff < 0
    )

    if raw_return_up_risk_efficiency_down:
        takeaways.append(
            "The stock delivered better raw return, but weaker Sharpe efficiency means the investor was paid less cleanly for each unit of volatility."
        )
    elif not pd.isna(sharpe_diff):
        if sharpe_diff > 0:
            takeaways.append("Risk-adjusted performance was stronger than the benchmark based on Sharpe ratio.")
        else:
            takeaways.append("Risk-adjusted performance was weaker than the benchmark based on Sharpe ratio.")

    if raw_return_up_deeper_drawdown:
        takeaways.append(
            "The outperformance came with deeper drawdowns, so position sizing and holding-period discipline matter more than the headline return suggests."
        )
    elif not pd.isna(mdd_diff):
        if mdd_diff < 0:
            takeaways.append("The stock had a deeper max drawdown than the benchmark, meaning higher downside risk.")
        else:
            takeaways.append("The stock had a shallower max drawdown than the benchmark.")

    if not pd.isna(revenue_cagr) and not pd.isna(gross_margin) and not pd.isna(fcf_margin):
        takeaways.append(
            f"The business profile is {category}: revenue CAGR is {fmt_percent(revenue_cagr)}, gross margin is {fmt_percent(gross_margin)}, and FCF margin is {fmt_percent(fcf_margin)}. That points to {'cash-flow quality' if fcf_margin > 0 else 'a need to prove cash conversion'}, not just top-line momentum."
        )
    elif not pd.isna(revenue_cagr):
        takeaways.append(f"Revenue CAGR is {fmt_percent(revenue_cagr)}, but the report needs more complete margin and cash-flow context before treating that growth as high quality.")

    if not pd.isna(revenue_cagr) and not pd.isna(gross_margin) and not pd.isna(fcf_margin):
        if revenue_cagr > 0.20 and gross_margin > 0.50 and fcf_margin > 0:
            takeaways.append("Growth quality looks stronger because revenue growth, gross margin, and free cash flow margin are all positive signals.")
        elif revenue_cagr > 0.20 and fcf_margin <= 0:
            takeaways.append("High growth has not yet translated into positive free cash flow margin, so growth quality needs manual verification.")

    if not pd.isna(valuation_score):
        if valuation_score < 45:
            valuation_context = []
            if not pd.isna(pe):
                valuation_context.append(f"PE around {pe:.1f}x")
            if not pd.isna(ps):
                valuation_context.append(f"PS around {ps:.1f}x")
            suffix = f" ({', '.join(valuation_context)})" if valuation_context else ""
            takeaways.append(
                f"Valuation is the main friction point{suffix}; future returns depend on the company maintaining margin quality and the market continuing to accept a premium multiple."
            )
        elif valuation_score >= 65:
            takeaways.append("Valuation does not look like the primary problem in this first pass, but it still needs peer and history checks.")

    if not pd.isna(total_score):
        takeaways.append(
            f"The research score is {fmt_score(total_score)} ({rating_from_score(total_score)}), which should be read as a triage label rather than a conclusion."
        )

    return takeaways[:8]


def explain_score(score_table: pd.DataFrame) -> str:
    if score_table is None or score_table.empty:
        return "_No score explanation available._"

    components = score_table[score_table["Component"] != "Research Score"].copy()
    if components.empty:
        return "_No score components available._"

    best = components.sort_values("Score", ascending=False).head(2)
    worst = components.sort_values("Score", ascending=True).head(2)

    lines = []
    lines.append("Main score support:")
    for _, row in best.iterrows():
        lines.append(f"- {row['Component']}: {fmt_score(row['Score'])}")

    lines.append("")
    lines.append("Main score drag:")
    for _, row in worst.iterrows():
        lines.append(f"- {row['Component']}: {fmt_score(row['Score'])}")

    return "\n".join(lines)


def score_methodology_section(info: dict[str, Any]) -> str:
    if is_fund_like(info):
        return (
            "Research Score is not calculated for ETF / fund / index-like instruments in this version. "
            "Fund analysis should focus on expense ratio, holdings, benchmark methodology, exposure, liquidity, and tracking error."
        )

    return """This score is a heuristic screening score weighted by research profile.
It is not a valuation model, not a prediction model, and not a buy/sell signal.

- Growth Score: revenue CAGR and latest revenue growth.
- Profitability Score: gross margin, operating margin, and FCF margin.
- Quality Trend Score: changes in gross margin, operating margin, and FCF margin.
- Risk Control Score: max drawdown, volatility, and beta.
- Benchmark Score: excess CAGR, information ratio, and Sharpe difference.
- Valuation Sanity Score: penalty-based check using PE, PS, EV/Revenue, and EV/EBITDA."""


def one_line_verdict(
    symbol: str,
    benchmark: str,
    price_summary: pd.DataFrame,
    fundamental_summary: pd.DataFrame,
    score_table: pd.DataFrame,
    info: dict[str, Any],
) -> str:
    total_score = get_component_score(score_table, "Research Score")
    rating = rating_from_score(total_score)

    if is_fund_like(info):
        return (
            f"{symbol} looks fund-like, so treat this as a price and benchmark-risk review rather than a company fundamentals report."
        )

    cagr_diff = get_metric(price_summary, "CAGR", "Difference")
    sharpe_diff = get_metric(price_summary, "Sharpe Ratio", "Difference")
    max_dd = get_metric(price_summary, "Max Drawdown", "Target")
    revenue_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    gross_margin = get_metric(fundamental_summary, "Gross Margin Latest")
    fcf_margin = get_metric(fundamental_summary, "FCF Margin Latest")
    valuation_score = get_component_score(score_table, "Valuation Sanity Score")

    if not pd.isna(revenue_cagr) and revenue_cagr < 0.05 and not pd.isna(fcf_margin) and fcf_margin > 0:
        company_phrase = "a mature cash-flow business rather than a high-growth story"
    elif not pd.isna(revenue_cagr) and revenue_cagr >= 0.20 and not pd.isna(fcf_margin) and fcf_margin <= 0:
        company_phrase = "a growth story that still needs to prove cash conversion"
    elif not pd.isna(fcf_margin) and fcf_margin > 0:
        company_phrase = "a cash-generative business"
    else:
        company_phrase = "a data-limited research case"

    if pd.isna(cagr_diff):
        benchmark_phrase = f"needs more evidence before its opportunity cost versus {benchmark} can be judged"
    elif cagr_diff > 0 and (pd.isna(sharpe_diff) or sharpe_diff >= 0):
        benchmark_phrase = f"has backed up its story with stronger performance than {benchmark} in this period"
    elif cagr_diff > 0:
        benchmark_phrase = f"beat {benchmark} on return, but the risk-adjusted picture is less clean"
    else:
        benchmark_phrase = f"has not yet justified the extra work versus simply holding {benchmark} in this period"

    risk_phrase = ""
    if not pd.isna(max_dd) and max_dd <= -0.40:
        risk_phrase = " The deep drawdown history means downside risk needs special attention."

    valuation_phrase = ""
    if not pd.isna(valuation_score) and valuation_score < 45:
        valuation_phrase = " Valuation is the main constraint on the first-pass setup."

    return f"{symbol} looks like {company_phrase}; it {benchmark_phrase}, so the current research status is {rating}.{valuation_phrase}{risk_phrase}"


def build_data_warnings(
    symbol: str,
    benchmark: str,
    info: dict[str, Any],
    benchmark_info: dict[str, Any],
    target_price: pd.DataFrame,
    trends: pd.DataFrame,
) -> list[str]:
    warnings = []

    target_currency = info.get("currency")
    benchmark_currency = benchmark_info.get("currency")
    if target_currency and benchmark_currency and target_currency != benchmark_currency:
        warnings.append(
            f"Currency mismatch detected. Target currency: {target_currency}. Benchmark currency: {benchmark_currency}. Relative performance may be misleading."
        )

    price_days = len(target_price.dropna()) if target_price is not None else 0
    if price_days < 252:
        warnings.append(
            f"Price history has fewer than roughly 1 trading year of observations ({price_days} rows). Benchmark comparison confidence is lower."
        )

    if is_fund_like(info):
        warnings.append(
            f"{symbol} appears to be ETF / fund / index-like based on quoteType={info.get('quoteType')}. Company financial statement analysis was skipped."
        )

    for key in ["heldPercentInsiders", "heldPercentInstitutions"]:
        value = info.get(key)
        try:
            if value is not None and not pd.isna(value) and float(value) > 1:
                warnings.append(
                    f"{key} is above 100%. This may reflect data provider timing or methodology differences. Manual verification required."
                )
        except Exception:
            pass

    for key in ["trailingPE", "forwardPE", "enterpriseToEbitda"]:
        value = info.get(key)
        try:
            if value is not None and not pd.isna(value) and float(value) < 0:
                warnings.append(
                    f"{key} is negative. This often indicates losses, unusual accounting, or data-provider methodology differences."
                )
        except Exception:
            pass

    if not is_fund_like(info):
        financial_years = len(trends.dropna(how="all")) if trends is not None else 0
        if financial_years < 3:
            warnings.append(
                f"Financial statement history has fewer than 3 usable years ({financial_years}). Score reliability is lower."
            )
        if trends is None or trends.empty or "Revenue" not in trends or trends["Revenue"].dropna().empty:
            warnings.append("Revenue data is missing. Growth analysis and score reliability are limited.")
        if trends is None or trends.empty or "Free Cash Flow" not in trends or trends["Free Cash Flow"].dropna().empty:
            warnings.append("Free cash flow data is missing. Cash conversion analysis requires manual verification.")

    return warnings


def build_sanity_checks(
    symbol: str,
    benchmark: str,
    info: dict[str, Any],
    benchmark_info: dict[str, Any],
    target_price: pd.DataFrame,
    trends: pd.DataFrame,
    ruin_risk: pd.DataFrame,
) -> pd.DataFrame:
    rows = []

    def add(severity: str, check: str, finding: str, action: str) -> None:
        rows.append(
            {
                "Severity": severity,
                "Check": check,
                "Finding": finding,
                "Action": action,
            }
        )

    target_currency = info.get("currency")
    benchmark_currency = benchmark_info.get("currency")
    if target_currency and benchmark_currency and target_currency != benchmark_currency:
        add(
            "HIGH",
            "Currency mismatch",
            f"{symbol}: {target_currency}; {benchmark}: {benchmark_currency}.",
            "Do not treat relative performance as clean without FX adjustment.",
        )

    financial_years = len(trends.dropna(how="all")) if trends is not None else 0
    if not is_fund_like(info) and financial_years < 3:
        add(
            "HIGH",
            "Short financial history",
            f"Only {financial_years} usable financial years found.",
            "Treat score as fragile and verify filings manually.",
        )

    price_days = len(target_price.dropna()) if target_price is not None else 0
    if price_days < 252:
        add(
            "MEDIUM",
            "Short price history",
            f"Only {price_days} price rows found.",
            "Benchmark, beta, drawdown, and Sharpe may be unstable.",
        )

    if not is_fund_like(info):
        revenue = latest_trend_value(trends, "Revenue")
        fcf = latest_trend_value(trends, "Free Cash Flow")
        operating_cf = latest_trend_value(trends, "Operating Cash Flow")
        capex = latest_trend_value(trends, "Capital Expenditure")

        if pd.isna(revenue):
            add(
                "HIGH",
                "Missing revenue",
                "Revenue is missing from the provider financial statements.",
                "Do not rely on growth score until revenue is verified.",
            )

        if pd.isna(fcf):
            add(
                "HIGH",
                "Missing FCF",
                "Free cash flow is missing from provider data.",
                "Verify OCF, capex, and FCF from filings.",
            )
        elif not pd.isna(operating_cf) and not pd.isna(capex):
            reconstructed_fcf = operating_cf + capex
            if abs(fcf) > 1 and abs(reconstructed_fcf - fcf) / max(abs(fcf), 1) > 0.10:
                add(
                    "HIGH",
                    "FCF consistency",
                    "Provider FCF differs from OCF plus capex by more than 10%.",
                    "Verify cash-flow statement manually.",
                )

        ruin_score = get_metric(ruin_risk, "Balance Sheet Resilience Score")
        if not pd.isna(ruin_score) and ruin_score <= 25:
            add(
                "HIGH",
                "Ruin risk",
                f"Balance Sheet Resilience Score is {fmt_number(ruin_score)}.",
                "Stress-test debt, cash burn, and refinancing risk.",
            )

    if is_fund_like(info):
        add(
            "MEDIUM",
            "Fund-like instrument",
            f"quoteType={info.get('quoteType')}; company fundamentals were skipped.",
            "Analyze expense ratio, holdings, liquidity, tracking error, and exposure.",
        )

    if not rows:
        add(
            "INFO",
            "No triggered sanity failure",
            "No automatic high-risk consistency failure was detected.",
            "Still verify important numbers with primary sources.",
        )

    return pd.DataFrame(rows)


def sanity_checks_section(sanity_checks: pd.DataFrame) -> str:
    return markdown_table(sanity_checks, max_rows=30)


def warnings_section(warnings: list[str]) -> str:
    if not warnings:
        return "No additional warning rule was triggered. Review the sanity checks above before relying on the data."
    return "\n".join(f"- Warning: {warning}" for warning in warnings)


def data_confidence_section(
    trends: pd.DataFrame,
    valuation: pd.DataFrame,
    profile: pd.DataFrame,
    target_price: pd.DataFrame,
    info: dict[str, Any],
) -> str:
    price_days = len(target_price.dropna()) if target_price is not None else 0
    price_conf = "Medium-High" if price_days >= 252 else "Low / Short History"
    financial_conf = "Not Applicable for Fund" if is_fund_like(info) else "Medium" if trends is not None and not trends.empty else "Low / Missing"
    valuation_conf = "Medium" if valuation is not None and not valuation.empty else "Low / Missing"
    profile_conf = "Medium" if profile is not None and not profile.empty else "Low / Missing"

    return f"""| Data Area | Confidence | Notes |
|---|---|---|
| Price Data | {price_conf} | {price_days} rows available. Usually usable for historical comparison, but may be delayed or adjusted by provider. |
| Company Profile | {profile_conf} | Good for quick context, but business description should be verified with company filings. |
| Financial Statements | {financial_conf} | Useful for screening; verify important numbers with 10-K / 10-Q. |
| Valuation Snapshot | {valuation_conf} | Useful for first-pass valuation risk, not enough for final judgment. |
| Segment Revenue | Manual Required | Usually requires SEC filings or company IR. |
"""


def status_from_score(score: float, strong: float = 65, weak: float = 40, reverse: bool = False) -> str:
    if pd.isna(score):
        return "Unknown"
    if reverse:
        if score >= strong:
            return "Weak"
        if score <= weak:
            return "Strong"
        return "Medium"
    if score >= strong:
        return "Strong"
    if score <= weak:
        return "Weak"
    return "Medium"


def plain_status_from_score(score: float) -> str:
    status = status_from_score(score)
    return "Moderate" if status == "Medium" else status


def valuation_status(score: float) -> str:
    if pd.isna(score):
        return "Unknown"
    if score < 45:
        return "Expensive"
    if score >= 70:
        return "Reasonable"
    return "Needs Review"


def risk_level_from_ruin_score(score: float) -> str:
    if pd.isna(score):
        return "Unknown"
    if score >= 45:
        return "Low"
    if score >= 25:
        return "Medium"
    return "High"


def stock_risk_status(risk_score: float) -> str:
    if pd.isna(risk_score):
        return "Unknown"
    if risk_score >= 70:
        return "Low / Moderate"
    if risk_score >= 45:
        return "Medium"
    return "High"


def beginner_summary_table(
    fundamental_summary: pd.DataFrame,
    score_table: pd.DataFrame,
    ruin_risk: pd.DataFrame,
    sanity_checks: pd.DataFrame,
) -> pd.DataFrame:
    growth_score_value = get_component_score(score_table, "Growth Score")
    profitability_score_value = get_component_score(score_table, "Profitability Score")
    valuation_score_value = get_component_score(score_table, "Valuation Sanity Score")
    risk_score_value = get_component_score(score_table, "Risk Control Score")
    ruin_score = get_metric(ruin_risk, "Balance Sheet Resilience Score")
    revenue_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    fcf_margin = get_metric(fundamental_summary, "FCF Margin Latest")

    high_severity = 0
    if sanity_checks is not None and not sanity_checks.empty and "Severity" in sanity_checks:
        high_severity = int((sanity_checks["Severity"] == "HIGH").sum())

    rows = [
        {
            "Area": "Business Quality",
            "Status": plain_status_from_score(profitability_score_value),
            "Plain-English Meaning": "The company appears cash-generative and profitable." if not pd.isna(fcf_margin) and fcf_margin > 0 else "Cash conversion is not yet clearly proven.",
        },
        {
            "Area": "Growth",
            "Status": plain_status_from_score(growth_score_value),
            "Plain-English Meaning": f"Revenue growth is {fmt_percent(revenue_cagr)} in this data window." if not pd.isna(revenue_cagr) else "Revenue growth data is incomplete.",
        },
        {
            "Area": "Valuation",
            "Status": valuation_status(valuation_score_value),
            "Plain-English Meaning": "The stock needs strong future execution to justify the current multiple." if not pd.isna(valuation_score_value) and valuation_score_value < 45 else "Valuation does not look like the main first-pass issue.",
        },
        {
            "Area": "Balance Sheet Risk",
            "Status": risk_level_from_ruin_score(ruin_score),
            "Plain-English Meaning": "Debt or cash-flow fragility deserves manual review." if not pd.isna(ruin_score) and ruin_score <= 25 else "Debt and cash-flow fragility do not appear to be the main first-pass risk.",
        },
        {
            "Area": "Stock Risk",
            "Status": stock_risk_status(risk_score_value),
            "Plain-English Meaning": "The stock can still have painful drawdowns even when the business is strong.",
        },
        {
            "Area": "Data Confidence",
            "Status": "Weak" if high_severity else "Medium",
            "Plain-English Meaning": "High-severity sanity checks were triggered." if high_severity else "Good enough for screening, but important numbers still need primary-source verification.",
        },
    ]
    return pd.DataFrame(rows)


def how_to_read_report_section() -> str:
    return """This report is designed for first-pass research.

You do not need to understand every financial term at the beginning. Start with five questions:

1. What does the company sell?
2. Is revenue growing?
3. Does the company turn revenue into real cash?
4. Is the balance sheet fragile?
5. Is the stock already priced for perfection?

If a metric looks unfamiliar, read the plain-English note below each section first, then use the table as evidence.

Metric guide: [docs/metric_guide.md](../../../docs/metric_guide.md)"""


def price_plain_english(symbol: str, benchmark: str, price_summary: pd.DataFrame) -> str:
    total_diff = get_metric(price_summary, "Total Return", "Difference")
    sharpe_diff = get_metric(price_summary, "Sharpe Ratio", "Difference")
    mdd_diff = get_metric(price_summary, "Max Drawdown", "Difference")

    if not pd.isna(total_diff) and total_diff > 0:
        result = f"{symbol} made more money than {benchmark} during this period."
    elif not pd.isna(total_diff):
        result = f"{symbol} made less money than {benchmark} during this period."
    else:
        result = "The return comparison is incomplete."

    risk_parts = []
    if not pd.isna(sharpe_diff) and sharpe_diff < 0:
        risk_parts.append("Its risk-adjusted return was weaker, which means the extra return was not as efficient as it first appears.")
    if not pd.isna(mdd_diff) and mdd_diff < 0:
        risk_parts.append("It also had a deeper drawdown, so investors had to tolerate more pain along the way.")
    if not risk_parts:
        risk_parts.append("The risk metrics do not show an obvious penalty versus the benchmark in this first pass.")

    return f"{result} {' '.join(risk_parts)} The key question is not just which line went up more, but whether the extra return was worth the extra volatility and drawdown."


def growth_plain_english(fundamental_summary: pd.DataFrame) -> str:
    revenue_cagr = get_metric(fundamental_summary, "Revenue CAGR")
    fcf_margin = get_metric(fundamental_summary, "FCF Margin Latest")
    gross_margin = get_metric(fundamental_summary, "Gross Margin Latest")
    if pd.isna(revenue_cagr):
        return "Growth data is incomplete, so the business trend needs manual verification."
    if revenue_cagr < 0.05 and not pd.isna(fcf_margin) and fcf_margin > 0:
        return f"Growth is not the main story here. Revenue CAGR is {fmt_percent(revenue_cagr)}, so the case depends more on profitability, cash flow, capital returns, and valuation discipline."
    if revenue_cagr >= 0.20 and not pd.isna(fcf_margin) and fcf_margin <= 0:
        return "Revenue is growing quickly, but the company has not yet proven that growth turns into free cash flow."
    if not pd.isna(gross_margin) and not pd.isna(fcf_margin):
        return f"The business shows {fmt_percent(gross_margin)} gross margin and {fmt_percent(fcf_margin)} FCF margin. That helps judge whether growth is supported by real economics."
    return "Use this section to check whether revenue growth is supported by margins and cash conversion."


def ruin_plain_english(ruin_risk: pd.DataFrame) -> str:
    ruin_score = get_metric(ruin_risk, "Balance Sheet Resilience Score")
    if pd.isna(ruin_score):
        return "Ruin risk data is incomplete. Debt, cash, EBITDA, and free cash flow should be checked manually."
    if ruin_score <= 25:
        return "This section shows elevated financial fragility. The next step is to check debt maturities, cash burn, refinancing risk, and dilution risk."
    return "This section is not about daily stock movement. It asks whether the business could face serious financial stress if growth slows, cash flow weakens, or refinancing becomes difficult."


def cash_flow_plain_english(trends: pd.DataFrame) -> str:
    if trends is None or trends.empty:
        return "Cash-flow data is missing or not applicable. Do not infer business quality from an empty table."
    fcf = latest_trend_value(trends, "Free Cash Flow")
    revenue = latest_trend_value(trends, "Revenue")
    if not pd.isna(fcf) and fcf > 0:
        return "The company generated positive free cash flow in the latest available period. That cash can support reinvestment, buybacks, debt reduction, or dividends."
    if not pd.isna(revenue) and (pd.isna(fcf) or fcf <= 0):
        return "Revenue exists, but free cash flow is weak or missing. For beginners, this means sales have not clearly turned into surplus cash."
    return "Use this section to connect the business model to real cash generation."


def valuation_plain_english(info: dict[str, Any], score_table: pd.DataFrame) -> str:
    valuation_score = get_component_score(score_table, "Valuation Sanity Score")
    pe = safe_info_float(info, "trailingPE")
    ps = safe_info_float(info, "priceToSalesTrailing12Months")
    details = []
    if not pd.isna(pe):
        details.append(f"PE is around {pe:.1f}x")
    if not pd.isna(ps):
        details.append(f"price-to-sales is around {ps:.1f}x")
    prefix = f"{', and '.join(details)}. " if details else ""
    if not pd.isna(valuation_score) and valuation_score < 45:
        return prefix + "The market is already pricing in a lot of future success, so the company needs strong execution to justify the valuation."
    return prefix + "Valuation does not look like the main first-pass problem, but it still needs peer and history checks."


def margin_plain_english(margin_stress: pd.DataFrame) -> str:
    if margin_stress is None or margin_stress.empty:
        return "No personal account inputs were provided. This section stays empty unless you add account equity and margin loan values."
    worst = margin_stress.iloc[-1]
    return f"Under the largest stress scenario shown here, the equity cushion would be {fmt_number(worst['Equity Cushion'])} and loan/value would be {fmt_percent(worst['Loan / Value'])}. This is about your own balance sheet, not the company."


def df_records(df: pd.DataFrame | None) -> list[dict[str, Any]]:
    if df is None or df.empty:
        return []
    clean = df.copy()
    clean = clean.where(pd.notna(clean), None)
    return clean.to_dict(orient="records")


def generated_file_list(out_dir: Path) -> list[str]:
    if not out_dir.exists():
        return []
    return sorted(path.name for path in out_dir.iterdir() if path.is_file())


def manual_verification_items() -> list[str]:
    return [
        "Revenue source and segment breakdown",
        "Gross margin trend",
        "Operating income quality",
        "Free cash flow calculation",
        "Debt and dilution",
        "Stock-based compensation",
        "One-time gains/losses",
        "Management guidance",
        "SEC 10-K / 10-Q",
        "Company IR materials",
        "Sanity Checks HIGH severity items",
        "Ruin Risk debt and cash-burn assumptions",
    ]


def final_research_questions(symbol: str, benchmark: str) -> list[str]:
    return [
        f"Why not simply buy {benchmark}?",
        f"Has {symbol} earned its extra risk?",
        "Is growth real or narrative-driven?",
        "Is profit quality improving?",
        "Is free cash flow healthy?",
        "Is valuation already pricing in too much future success?",
        "If the stock falls 30%-50%, does the thesis still hold?",
        "If the stock falls 70%, does the business survive without destructive dilution?",
        "Is this company being judged against the right lifecycle and sector peers?",
    ]


def generate_report_data_dict(
    symbol: str,
    benchmark: str,
    start_date: str,
    end_date: str | None,
    out_dir: Path,
    profile: pd.DataFrame,
    valuation: pd.DataFrame,
    trends: pd.DataFrame,
    fundamental_summary: pd.DataFrame,
    price_summary: pd.DataFrame,
    score_table: pd.DataFrame,
    info: dict[str, Any],
    target_price: pd.DataFrame,
    warnings: list[str],
    sanity_checks: pd.DataFrame,
    ruin_risk: pd.DataFrame,
    margin_stress: pd.DataFrame,
    actual_chart_name: str,
    chart_name: str,
    drawdown_chart_name: str,
    score_components_chart_name: str,
    growth_quality_chart_name: str | None,
    ruin_risk_chart_name: str,
    interactive_chart_name: str,
    radar_chart_name: str,
    ai_review_markdown: str | None = None,
) -> dict[str, Any]:
    total_score = get_component_score(score_table, "Research Score")
    rating = rating_from_score(total_score)
    category = classify_research_category(info, fundamental_summary)
    beginner_summary = beginner_summary_table(fundamental_summary, score_table, ruin_risk, sanity_checks)
    key_takeaways = generate_key_takeaways(symbol, benchmark, price_summary, fundamental_summary, score_table, info)

    return {
        "ticker": symbol,
        "symbol": symbol,
        "benchmark": benchmark,
        "version": __version__,
        "start_date": start_date,
        "end_date": end_date,
        "research_profile": category,
        "research_status": rating,
        "one_line_verdict": one_line_verdict(symbol, benchmark, price_summary, fundamental_summary, score_table, info),
        "key_takeaways": key_takeaways,
        "beginner_summary": df_records(beginner_summary),
        "price_metrics": df_records(price_summary),
        "growth_quality_metrics": df_records(fundamental_summary),
        "valuation_snapshot": df_records(valuation),
        "ruin_risk_metrics": df_records(ruin_risk),
        "research_score": {"score": None if pd.isna(total_score) else float(total_score), "status": rating},
        "score_components": df_records(score_table),
        "sanity_checks": df_records(sanity_checks),
        "data_confidence": data_confidence_section(trends, valuation, profile, target_price, info),
        "manual_verification": manual_verification_items(),
        "final_research_questions": final_research_questions(symbol, benchmark),
        "generated_files": generated_file_list(out_dir),
        "company_profile": df_records(profile),
        "business_model_and_cash_flow": df_records(trends),
        "raw": {
            "profile": profile,
            "valuation": valuation,
            "trends": trends,
            "fundamental_summary": fundamental_summary,
            "price_summary": price_summary,
            "score_table": score_table,
            "target_price": target_price,
            "warnings": warnings,
            "sanity_checks": sanity_checks,
            "ruin_risk": ruin_risk,
            "margin_stress": margin_stress,
            "info": info,
        },
        "charts": {
            "actual": actual_chart_name,
            "normalized": chart_name,
            "drawdown": drawdown_chart_name,
            "score_components": score_components_chart_name,
            "growth_quality": growth_quality_chart_name,
            "ruin_risk": ruin_risk_chart_name,
            "interactive": interactive_chart_name,
            "radar": radar_chart_name,
        },
        "ai_review_markdown": ai_review_markdown,
    }


# =============================================================================
# Report
# =============================================================================

PRICE_PERCENT_COLS = {"Target", "Benchmark", "Difference"}
FUND_PERCENT_COLS = {"Value"}
SCORE_COLS = {"Score"}


def write_report(
    symbol: str,
    benchmark: str,
    start_date: str,
    end_date: str | None,
    out_dir: Path,
    profile: pd.DataFrame,
    valuation: pd.DataFrame,
    trends: pd.DataFrame,
    fundamental_summary: pd.DataFrame,
    price_summary: pd.DataFrame,
    score_table: pd.DataFrame,
    info: dict[str, Any],
    target_price: pd.DataFrame,
    warnings: list[str],
    sanity_checks: pd.DataFrame,
    ruin_risk: pd.DataFrame,
    margin_stress: pd.DataFrame,
    actual_chart_name: str,
    chart_name: str,
    drawdown_chart_name: str,
    score_components_chart_name: str,
    growth_quality_chart_name: str | None,
    ruin_risk_chart_name: str,
    interactive_chart_name: str,
    radar_chart_name: str,
    ai_review_markdown: str | None = None,
    v4_sections: dict[str, str] | None = None,
    gate_status: dict[str, str] | None = None,
    asset_profile_data: dict[str, Any] | None = None,
) -> Path:
    report_data = generate_report_data_dict(
        symbol=symbol,
        benchmark=benchmark,
        start_date=start_date,
        end_date=end_date,
        out_dir=out_dir,
        profile=profile,
        valuation=valuation,
        trends=trends,
        fundamental_summary=fundamental_summary,
        price_summary=price_summary,
        score_table=score_table,
        info=info,
        target_price=target_price,
        warnings=warnings,
        sanity_checks=sanity_checks,
        ruin_risk=ruin_risk,
        margin_stress=margin_stress,
        actual_chart_name=actual_chart_name,
        chart_name=chart_name,
        drawdown_chart_name=drawdown_chart_name,
        score_components_chart_name=score_components_chart_name,
        growth_quality_chart_name=growth_quality_chart_name,
        ruin_risk_chart_name=ruin_risk_chart_name,
        interactive_chart_name=interactive_chart_name,
        radar_chart_name=radar_chart_name,
        ai_review_markdown=ai_review_markdown,
    )
    if asset_profile_data:
        report_data["asset_profile"] = asset_profile_data
        report_data["research_profile"] = asset_profile_data.get("primary_profile", report_data["research_profile"])

    total_score = report_data["research_score"]["score"]
    rating = report_data["research_status"]

    v4_sections = v4_sections or {}
    verdict = v4_sections.get("one_line_verdict") or report_data["one_line_verdict"]
    takeaways = report_data["key_takeaways"]
    takeaways_md = "\n".join([f"- {item}" for item in takeaways]) if takeaways else "_No automatic takeaways available._"
    beginner_summary = pd.DataFrame(report_data["beginner_summary"])

    category = report_data["research_profile"]
    charts = report_data["charts"]
    if category == "Capital-Intensive Semiconductor Turnaround":
        business_quality_intro = "**Conclusion:** This section is not asking whether the company has a generic growth story. For a capital-intensive semiconductor turnaround, revenue must be read together with gross-margin recovery, capex pressure, manufacturing execution, and free-cash-flow pressure."
        business_quality_interp = "**Interpretation:** Margin and FCF lines are transition evidence. If revenue improves but gross margin, capex burden, or free cash flow do not improve, the turnaround is still unproven."
    elif category in {"Speculative Growth", "Unprofitable Growth"}:
        business_quality_intro = "**Conclusion:** This section asks whether revenue growth is becoming better economics. For speculative growth, growth only matters if it starts improving gross margin, operating loss, and cash burn."
        business_quality_interp = "**Interpretation:** Revenue growth without improving unit economics is not enough. The next check is whether losses and free-cash-flow burn are narrowing."
    elif category == "Unknown / Data-Limited Screening":
        business_quality_intro = "**Conclusion:** This section is a first-pass data screen, not a full business-quality judgment. The company-specific research frame still needs verification."
        business_quality_interp = "**Interpretation:** These metrics show what public provider data can support. They do not replace industry-specific operating metrics."
    else:
        business_quality_intro = "**Conclusion:** This section checks whether the company converts revenue into profit and cash. A mature company with slow revenue growth can still be high quality if margins and free cash flow remain strong."
        business_quality_interp = "**Interpretation:** Revenue growth tells only part of the story. Margin stability and free-cash-flow conversion show whether the business model is doing useful economic work."
    report_status_card = status_card_table(symbol, benchmark, start_date, end_date, rating, category, gate_status or {})
    key_questions_section = v4_sections.get("key_questions") or english_key_questions(symbol, benchmark, price_summary, fundamental_summary, valuation)
    chart_walkthrough_section = english_chart_walkthrough(symbol, benchmark, charts)
    ai_review_section = ""
    manual_section_number = 14
    next_section_number = 15
    final_section_number = 16
    files_section_number = 17
    if report_data.get("ai_review_markdown"):
        ai_review_section = f"\n---\n\n{report_data['ai_review_markdown'].strip()}\n"
        manual_section_number = 15
        next_section_number = 16
        final_section_number = 17
        files_section_number = 18

    gate_status = gate_status or {
        "DATA_AUDIT_STATUS": STATUS_PASS,
        "RISK_METHOD_STATUS": STATUS_PASS,
        "AI_ANALYST_REVIEW_STATUS": STATUS_PASS,
        "LANGUAGE_LINT_STATUS": STATUS_PASS,
    }
    battle_card_section = v4_sections.get("battle_card", "")
    risk_method_section = v4_sections.get("risk_methodology", "")
    valuation_sensitivity_section = v4_sections.get("valuation_sensitivity", "")
    segment_revenue_section = v4_sections.get("segment_revenue", "")
    next_checks_section = v4_sections.get("next_checks", "")
    core_view = v4_sections.get("core_view") or (
        f"{symbol} should be read as a first-pass research case, not as a finished investment conclusion. "
        "The report asks whether business quality, cash generation, valuation, and risk are consistent with the current market story."
    )

    def without_first_heading(text: str) -> str:
        lines = (text or "").splitlines()
        if lines and lines[0].startswith("## "):
            return "\n".join(lines[1:]).strip()
        return (text or "").strip()

    generated_at = datetime.now().strftime("%Y-%m-%d %H:%M")
    fallback_next_checks = "\n".join(
        [
            "1. Latest 10-K / 10-Q revenue breakdown",
            "2. Whether growth comes from volume, price, services, or accounting effects",
            "3. Whether free cash flow is stable or one-off",
            "4. Whether valuation is justified by future growth",
            "5. Whether the company has hidden dilution, debt, or margin pressure",
        ]
    )
    asset_profile_section = ""
    if report_data.get("asset_profile"):
        ap = report_data["asset_profile"]
        asset_profile_section = f"""## 3. Asset Profile

| Field | Value |
|---|---|
| Primary Profile | {ap.get('primary_profile')} |
| Secondary Profile | {ap.get('secondary_profile') or 'None'} |
| Framework Coverage | {ap.get('framework_coverage_level')} |
| Profile Confidence | {ap.get('thesis_spine_confidence')} |
| Valuation Method Fit | {ap.get('valuation_method_fit')} |
| Cash Flow Profile | {ap.get('cash_flow_profile')} |
| Data Deficits | {', '.join(ap.get('data_deficit_flags') or []) or 'None detected automatically'} |

**What this means:** The report shell is reusable, but the research logic is routed through this asset profile. If framework coverage is partial or screening-only, the report should be read as a starting point rather than a complete industry note.
"""

    report = f"""# {symbol} Equity Research Report

> Version: v{__version__}  
> Ticker: {symbol}  
> Benchmark: {benchmark}  
> Period: {start_date} to {end_date or 'latest available'}  
> Report Status: {gate_status.get('OVERALL_REPORT_STATUS', 'PASS')}  
> Generated: {generated_at}  
> Note: This report is for first-pass research only. It is not investment advice.

---

## Table of Contents

1. Report Status Card
2. One-line Verdict
3. Asset Profile
4. Thesis Spine
5. AI Analyst Red Flags
6. Research Battle Card
7. Key Questions and Answers
8. Chart Walkthrough
9. Business Quality
10. Risk and Resilience
11. Valuation Stress Test
12. Segment Revenue Gap
13. Data Audit and Methodology
14. Next Research Steps
15. Boundary
16. Appendix A: Metric Definitions and Units
17. Appendix B: Data Deficits and Manual Checks

---

## 1. Report Status Card

{report_status_card}

This status card is a reading guide. A warning does not mean the report is unusable; it means one or more assumptions, data fields, or labels need review before the report becomes decision-grade evidence. Numbers can be auditable while the research thesis still remains unverified.

---

## 2. One-line Verdict

{verdict}

---

{asset_profile_section}

---

## 4. Thesis Spine

{core_view}

---

## 5. AI Analyst Red Flags

The bounded AI analyst gate reviews locked data, detects profile mismatch and thin reasoning, and writes patches into interpretation-layer report blocks. It cannot change revenue, cash flow, valuation multiples, risk metrics, or the research score, and it cannot turn missing data into facts.

---

## 6. Research Battle Card

{without_first_heading(battle_card_section)}

---

{key_questions_section}

---

{chart_walkthrough_section}

### Price Evidence Table

This table turns the chart movement into comparable return and risk metrics. Read it after the charts, because the numbers explain whether the visual outperformance came with extra volatility or deeper drawdowns.

{markdown_table(price_summary, max_rows=50, percent_columns=PRICE_PERCENT_COLS)}

**What this means:** The table is evidence for the benchmark comparison. It is not a forecast, a valuation model, or a buy/sell signal.

---

## 9. Business Quality

{business_quality_intro}

{markdown_table(fundamental_summary, max_rows=30, percent_columns=FUND_PERCENT_COLS)}

{business_quality_interp}

---

## 10. Risk and Resilience

![{symbol} balance-sheet resilience]({ruin_risk_chart_name})

**What this chart shows:** This chart separates balance-sheet and cash-flow stress from ordinary stock-price volatility.

**What the report reads from it:** A higher Balance Sheet Resilience Score means stronger financial resilience. Debt and cash-flow ratios explain whether the company has room to absorb stress.

**How not to misread it:** This is not a drawdown chart and not a short-term price-risk model.

{markdown_table(ruin_risk, max_rows=20)}

Balance Sheet Resilience Score direction: higher score = stronger balance sheet resilience. This score measures balance-sheet resilience, not stock-price volatility.

---

## 11. Valuation Stress Test

**Conclusion:** Valuation is not a target price exercise here. It is a pressure test: if the market pays a lower multiple, how much business performance is needed to offset that pressure?

{without_first_heading(valuation_sensitivity_section)}

### Valuation Snapshot

This table shows the provider valuation snapshot. Treat it as screening data because provider snapshots can change with refresh time and market close.

{valuation_group_sections(valuation)}

**Interpretation:** High multiples require durable earnings, margin discipline, and cash-flow support. They do not prove downside is imminent.

---

## 12. Segment Revenue Gap

{without_first_heading(segment_revenue_section)}

---

## 13. Data Audit and Methodology

{without_first_heading(risk_method_section)}

### Data Confidence

{data_confidence_section(trends, valuation, profile, target_price, info)}

### Sanity Checks

{sanity_checks_section(sanity_checks)}

### Automatic Data Warnings

{warnings_section(warnings)}

---

## 14. Next Research Steps

{without_first_heading(next_checks_section) or fallback_next_checks}

---

## 15. Boundary

This report is a structured first-pass research workflow.

It does **not** provide:

- Buy / sell recommendation
- Target price
- Guaranteed return
- Automatic investment decision

The score is a **research prioritization score**, not a prediction.

---

## 16. Appendix A: Metric Definitions and Units

| Metric | Unit | Meaning |
|---|---|---|
| Revenue | USD | Total company sales reported by provider data. |
| Gross / Operating / FCF Margin | % | Margin ratios used to judge business economics and cash conversion. |
| PE / PS / EV Revenue | x | Valuation multiples; applicability depends on asset profile. |
| Drawdown / Volatility | % | Historical price-risk metrics, not business survival metrics. |
| Balance Sheet Resilience Score | 0-100 | Higher score means stronger balance-sheet resilience. |

## 17. Appendix B: Data Deficits and Manual Checks

{chr(10).join(f"- {item}" for item in (report_data.get("asset_profile", {}).get("data_deficit_flags") or ["No major profile-specific deficit was detected automatically."]))}

---

## 2. How to Read This Report

{how_to_read_report_section()}

---

## 3. Key Takeaways

{takeaways_md}

---

{battle_card_section}

---

## 4. Beginner Summary

{markdown_table(beginner_summary, max_rows=10)}

---

## 5. Data Confidence

{data_confidence_section(trends, valuation, profile, target_price, info)}

### Sanity Checks

{sanity_checks_section(sanity_checks)}

### Automatic Data Warnings

{warnings_section(warnings)}

---

## 6. Company Profile

{markdown_table(profile, max_rows=30)}

### Automatic Business Summary

{business_summary(info)}

### Manual Narrative Needed

You still need to manually answer:

- What does the company actually sell?
- Who pays the company?
- What is the main revenue source?
- What is the growth story?
- Is the story supported by financial data?
- What can break the thesis?

---

## 7. Price vs Benchmark

Benchmark explanation:

> {benchmark_explanation(benchmark)}

### Interactive HTML

[Open interactive price dashboard]({interactive_chart_name})

The HTML chart supports hover, zoom, range selection, and exact-date inspection.

![{symbol} vs {benchmark} Actual Close Price]({actual_chart_name})

Actual close price chart shows the raw closing prices from the data provider.
Use this to inspect absolute price levels, gaps, and broad trend shape before comparing relative returns.

How to read this chart: it shows price level, not valuation. A higher line does not mean the stock is cheaper or safer.

![{symbol} vs {benchmark}]({chart_name})

Performance chart uses normalized price.
The first available price in the selected period is set to 100.
This allows comparison of relative performance, not absolute stock price.

How to read this chart: both lines start at 100. If one line ends higher, it performed better during this period. This does not prove it is a better investment today.

![{symbol} vs {benchmark} Drawdown]({drawdown_chart_name})

Drawdown shows the decline from the previous peak.
0% means no drawdown.
-20% means the asset fell 20% from its previous high.

How to read this chart: drawdown shows pain. A -30% drawdown means an investor buying near the previous peak would have seen the position fall by about 30%.

{markdown_table(price_summary, max_rows=50, percent_columns=PRICE_PERCENT_COLS)}

### How to Read This

- **Total Return / CAGR**: raw performance.
- **Excess Return**: whether the stock outperformed the benchmark.
- **Max Drawdown**: deepest historical decline in this period.
- **Sharpe / Sortino / Calmar**: risk-adjusted performance.
- **Beta**: sensitivity to benchmark movement.
- **Information Ratio**: excess return per unit of tracking risk.
- **Upside / Downside Capture**: whether the stock captures more upside or downside than benchmark.

### Plain-English Meaning

{price_plain_english(symbol, benchmark, price_summary)}

---

{risk_method_section}

---

## 8. Growth and Quality Summary

{markdown_table(fundamental_summary, max_rows=30, percent_columns=FUND_PERCENT_COLS)}

### Core Questions

- Is revenue growing?
- Is growth accelerating or slowing?
- Is gross margin stable or improving?
- Is operating margin improving?
- Is free cash flow improving?

### Plain-English Meaning

{growth_plain_english(fundamental_summary)}

---

## 9. Ruin Risk

![{symbol} Ruin Risk]({ruin_risk_chart_name})

How to read this chart: this is not about day-to-day stock movement. It asks whether the business could face financial stress if growth slows, cash flow weakens, or refinancing becomes difficult.

{markdown_table(ruin_risk, max_rows=20)}

This section tries to separate normal price volatility from business fragility. Historical drawdown is not the same as ruin risk.

Balance Sheet Resilience Score direction: higher score = stronger balance sheet resilience.
This score measures balance-sheet resilience, not stock-price volatility.

### Plain-English Meaning

{ruin_plain_english(ruin_risk)}

---

## 10. Business Model and Cash Flow

{f"![{symbol} Growth and Quality Trend]({growth_quality_chart_name})" if growth_quality_chart_name else ""}

How to read this chart: rising revenue is useful, but the important question is whether the company keeps enough cash after costs, operations, and capital spending.

{markdown_table(
    trends,
    max_rows=20,
    percent_columns={"Revenue Growth YoY", "Gross Margin", "Operating Margin", "Net Margin", "FCF Margin"},
)}

### Interpretation

- Revenue shows money coming in.
- Gross profit shows whether product/service economics work.
- Operating income shows whether the operating model works.
- Net income shows accounting profit.
- Operating cash flow shows whether business operations generate cash.
- Free cash flow shows whether cash remains after capital expenditure.

### Plain-English Meaning

{cash_flow_plain_english(trends)}

---

{segment_revenue_section}

---

## 11. Personal Margin Stress

{markdown_table(margin_stress, max_rows=20, percent_columns={"Loan / Value"}) if margin_stress is not None and not margin_stress.empty else "_No account-level margin inputs provided. Add `--account-equity` and `--margin-loan` to generate a personal stress table._"}

This optional section is not about the company. It tests whether your own balance sheet can survive stress.

### Plain-English Meaning

{margin_plain_english(margin_stress)}

---

## 12. Valuation Snapshot

{valuation_group_sections(valuation)}

High valuation requires stronger growth, margin expansion, and cash flow evidence.

### Plain-English Meaning

{valuation_plain_english(info, score_table)}

---

{valuation_sensitivity_section}

---

## 13. Research Score

[Open interactive score radar]({radar_chart_name})

![{symbol} Research Score Components]({score_components_chart_name})

How to read this chart: the bars show which parts of the screening model help or hurt the score. They do not say whether the stock is cheap or safe.

{markdown_table(score_table, max_rows=20, percent_columns={"Weight"}, score_columns=SCORE_COLS)}

### Beginner Warning

A high score is not a buy signal. A low score is not a sell signal. The score only helps prioritize further research under this model.

### Why This Score?

{score_methodology_section(info)}

{explain_score(score_table)}

### Score Meaning

- 75–100: High Priority Research
- 60–75: Watchlist
- 45–60: Research More
- 30–45: FOMO Risk / Weak Evidence
- 0–30: Avoid for Now / Data Weak

This score is transparent but imperfect. It is used to prioritize research, not to make investment decisions.

---

{ai_review_section}
## {manual_section_number}. Manual Verification

Before making any serious judgment, verify:

- Revenue source and segment breakdown
- Gross margin trend
- Operating income quality
- Free cash flow calculation
- Debt and dilution
- Stock-based compensation
- One-time gains/losses
- Management guidance
- SEC 10-K / 10-Q
- Company IR materials
- Sanity Checks HIGH severity items
- Ruin Risk debt and cash-burn assumptions

---

## {next_section_number}. What to Check Next

Before making any serious judgment, manually check:

1. Latest 10-K / 10-Q revenue breakdown
2. Whether growth comes from volume, price, services, or accounting effects
3. Whether free cash flow is stable or one-off
4. Whether valuation is justified by future growth
5. Whether the company has hidden dilution, debt, or margin pressure

---

## {final_section_number}. Final Research Questions

- Why not simply buy `{benchmark}`?
- Has `{symbol}` earned its extra risk?
- Is growth real or narrative-driven?
- Is profit quality improving?
- Is free cash flow healthy?
- Is valuation already pricing in too much future success?
- If the stock falls 30%-50%, does the thesis still hold?
- If the stock falls 70%, does the business survive without destructive dilution?
- Is this company being judged against the right lifecycle and sector peers?

---

## {files_section_number}. Generated Files

This folder contains CSV, chart, and Markdown outputs generated by the tool.
"""
    legacy_marker = "\n## 2. How to Read This Report"
    if legacy_marker in report:
        report = report.split(legacy_marker, 1)[0].rstrip()
        report += f"""

## Generated Files

This folder contains the Markdown reports, audit logs, chart images, interactive dashboard, and CSV exports generated for this run.
"""
    report = clean_report_placeholders(report, "en")

    path = out_dir / report_filename(safe_symbol(symbol), "research_report.md", gate_status.get("OVERALL_REPORT_STATUS", "VERIFIED"))
    save_text(path, report)
    return path


def write_chinese_report(
    report_data: dict[str, Any],
    out_dir: Path,
    v4_sections: dict[str, str],
    gate_status: dict[str, str],
    term_style: str = "pure",
) -> Path:
    symbol = report_data["ticker"]
    benchmark = report_data["benchmark"]
    raw = report_data["raw"]
    price_summary = raw["price_summary"]
    fundamental_summary = raw["fundamental_summary"]
    score_table = raw["score_table"]
    valuation = raw["valuation"]
    ruin_risk = raw["ruin_risk"]
    charts = report_data["charts"]
    score = report_data["research_score"]["score"]
    status_card = zh_status_card_table(symbol, benchmark, report_data["start_date"], report_data["end_date"], report_data["research_status"], report_data["research_profile"], gate_status)
    questions = v4_sections.get("key_questions_zh") or chinese_key_questions(symbol, benchmark, price_summary, fundamental_summary, valuation)
    charts_section = chinese_chart_walkthrough(symbol, benchmark, charts)
    verdict = v4_sections.get("one_line_verdict_zh") or chinese_verdict(symbol, benchmark, report_data)
    core_view = v4_sections.get("core_view_zh") or "这份报告用于第一轮研究，不负责替用户给买卖结论。核心任务是判断当前证据是否足够支持继续研究，以及哪些问题必须人工复核。"
    generated_at = datetime.now().strftime("%Y-%m-%d %H:%M")
    ap = report_data.get("asset_profile", {})
    profile_rows = pd.DataFrame(
        [
            {"项目": "主要画像", "内容": ZH_STATUS_LABELS.get(ap.get("primary_profile", ""), ap.get("primary_profile", ""))},
            {"项目": "次要画像", "内容": zh_profile_value(ap.get("secondary_profile") or "无")},
            {"项目": "研究框架覆盖程度", "内容": zh_profile_value(ap.get("framework_coverage_level", "未知"))},
            {"项目": "画像置信度", "内容": zh_profile_value(ap.get("thesis_spine_confidence", "未知"))},
            {"项目": "适用估值框架", "内容": zh_profile_value(ap.get("valuation_method_fit", "未知"))},
            {"项目": "数据缺口", "内容": zh_profile_value(ap.get("data_deficit_flags") or []) or "自动检查未发现重大缺口"},
        ]
    )
    battle_card_zh = v4_sections.get("battle_card_zh", "").strip()
    if battle_card_zh.startswith("## "):
        battle_card_zh = "\n".join(battle_card_zh.splitlines()[1:]).strip()
    next_checks_zh = v4_sections.get("next_checks_zh", "").strip() or "## 13. 下一步研究清单\n\n1. 核查最新财报。\n2. 核查业务驱动因素。\n3. 核查估值方法。"
    if next_checks_zh.startswith("## "):
        next_checks_zh = "\n".join(next_checks_zh.splitlines()[1:]).strip()
    profile_name = ap.get("primary_profile", report_data["research_profile"])
    if profile_name == "Capital-Intensive Semiconductor Turnaround":
        business_quality_intro = "**结论：这里不是看公司有没有科技故事，而是看收入能不能和毛利率修复、资本开支压力、制造执行和自由现金流改善一起出现。**"
        business_quality_interp = "这说明什么：对资本开支重的半导体转型公司，收入改善只是第一层。毛利率、capex、free cash flow、foundry 和 data center 业务线才决定转型有没有经营证据。"
    elif profile_name in {"Speculative Growth", "Unprofitable Growth"}:
        business_quality_intro = "**结论：这里看的不是收入有没有增长，而是增长有没有开始变成更好的经济性。**"
        business_quality_interp = "这说明什么：如果收入增长没有带来毛利率改善、亏损收窄或现金消耗下降，成长故事还不能算经营证据。"
    elif profile_name == "Unknown / Data-Limited Screening":
        business_quality_intro = "**结论：这里是第一轮数据初筛，不是完整业务质量判断。**"
        business_quality_interp = "这说明什么：这些指标只能说明公共数据里能看到什么。行业专属经营指标缺失时，不能把普通财务表格当作完整结论。"
    else:
        business_quality_intro = "**结论：这里看的不是公司有没有故事，而是收入能不能变成利润和现金。** 对成熟公司来说，收入不高速增长也不一定是问题；真正要看的是毛利率、经营利润率和自由现金流率能不能守住。"
        business_quality_interp = "这说明什么：如果收入增长慢，但利润率和自由现金流率很强，研究重点就不是“爆发式增长”，而是“高质量现金流能不能继续支撑估值”。"

    content = f"""# {symbol} 股票研究报告

> 版本：v{__version__}  
> 标的：{symbol}  
> 基准：{benchmark}  
> 周期：{report_data["start_date"]} 至 {report_data["end_date"] or "最新可得数据"}  
> 报告状态：{zh_overall_status(gate_status.get("OVERALL_REPORT_STATUS", "PASS"))}  
> 生成时间：{generated_at}  
> 说明：本报告用于第一轮研究，不构成买卖建议。

---

## 目录

1. 报告状态卡片
2. 一句话结论
3. 资产画像
4. 报告主线
5. AI 二次复核红旗
6. 投研博弈卡片
7. 关键问题与回答
8. 图表解读
9. 业务质量
10. 风险与韧性
11. 估值压力测试
12. 业务线拆解：当前缺口
13. 数据审计与方法说明
14. 下一步研究清单
15. 结论边界
16. 附录 A：指标定义与单位
17. 附录 B：数据缺口与人工核查项

---

## 1. 报告状态卡片

{status_card}

这张状态卡告诉你这份报告现在能不能直接使用。出现“有警告”不代表报告作废，而是提醒你：相关数字、方法或数据标签还需要复核，不能直接当成最终证据。数字层可以是可审计的，但研究主线仍可能未验证。

---

## 2. 一句话结论

{verdict}

---

## 3. 资产画像

{markdown_table(profile_rows, max_rows=10)}

这说明什么：报告外壳可以复用，但研究逻辑必须跟资产画像一致。如果研究框架覆盖程度不是完整覆盖，这份报告只能作为初筛，不能当完整行业研究。

---

## 4. 报告主线

{core_view}

---

## 5. AI 二次复核红旗

AI 二次复核层会审查锁定数据、识别画像错配和推理过薄的问题，并在解释层文本中写入修正补丁。它不能修改收入、现金流、估值倍数、风险指标或研究评分，也不能把缺失数据补成事实。详细记录见 `../ai/ai_correction_log.md` 和 `../ai/patch_diff_log.md`。

---

## 6. 投研博弈卡片

{battle_card_zh}

---

{questions}

---

{charts_section}

---

## 9. 业务质量

{business_quality_intro}

{localized_metric_table(fundamental_summary, term_style=term_style, max_rows=30, percent_columns=FUND_PERCENT_COLS)}

{business_quality_interp}

---

## 10. 风险与韧性

![{symbol} 资产负债表韧性]({charts["ruin_risk"]})

**图表看什么：** 这张图看的是债务、现金流和资产负债表承压能力，不是股价短期波动。

**读出来的结论：** 资产负债表韧性分数越高，说明公司越有能力承受现金流波动、债务压力或融资环境变化。

**不要误读：** 财务韧性强不等于股价不会跌。估值太高时，即使公司基本面不脆，股价也可能因为杀估值而回撤。

{localized_metric_table(ruin_risk, term_style=term_style, max_rows=20)}

这说明什么：这部分回答的是“公司抗不抗压”，不是“股票会不会跌”。两者不能混为一谈。

---

{v4_sections.get("valuation_sensitivity_zh", "").strip()}

下面的估值快照来自数据供应商，只能作为初筛输入。真正严肃的判断，仍要回到最新财报和公司公告。

{localized_metric_table(valuation[["Metric", "Value"]] if "Metric" in valuation.columns else valuation, term_style=term_style, max_rows=50)}

这说明什么：估值倍数越高，市场对未来稳定性的要求越高。高估值不等于马上下跌，但它会降低犯错空间。

---

## 12. 业务线拆解：当前缺口

{v4_sections.get("segment_revenue_zh", "").replace("## 业务线拆解", "").strip()}

---

## 13. 数据审计与方法说明

### 风险指标方法

风险指标使用日频价格序列、252 个交易日年化和命令行传入的无风险利率。收益、回撤、波动和基准比较都依赖同一段对齐后的交易日数据。

### 初筛数据边界

这份报告使用公开数据供应商和自动计算结果。供应商快照可能滞后、缺字段或口径不同；重要数字必须回到原始财报、公司公告或监管文件复核。

---

## 14. 下一步研究清单

{next_checks_zh}

---

## 15. 结论边界

当前研究分数为 {fmt_score(score)}。它只说明这家公司值得按上述路径继续核查，不代表应该买入或卖出，也不代表未来收益。

---

## 16. 附录 A：指标定义与单位

| 指标 | 单位 | 含义 |
|---|---|---|
| 收入 | 美元 | 数据供应商提供的公司销售收入。 |
| 毛利率 / 经营利润率 / 自由现金流率 | % | 判断商业模式、盈利能力和现金转化质量。 |
| 市盈率 / 市销率 / 企业价值 / 收入 | 倍 | 估值倍数，是否适用取决于资产画像。 |
| 回撤 / 波动率 | % | 历史价格风险，不等于公司生存风险。 |
| 资产负债表韧性分数 | 0-100 | 分数越高，资产负债表韧性越强。 |

## 17. 附录 B：数据缺口与人工核查项

{chr(10).join(f"- {zh_profile_value(item)}" for item in (ap.get("data_deficit_flags") or ["自动检查未发现重大画像相关缺口。"]))}
"""
    content = clean_report_placeholders(content, "zh")
    path = out_dir / report_filename(safe_symbol(symbol), "research_report_cn.md", gate_status.get("OVERALL_REPORT_STATUS", "VERIFIED"))
    save_text(path, content)
    return path


# =============================================================================
# Main workflow
# =============================================================================

def run_one(
    symbol: str,
    benchmark: str,
    start_date: str,
    end_date: str | None,
    years: int,
    output: Path,
    risk_free_rate: float,
    archive: bool = False,
    run_id: str | None = None,
    account_equity: float | None = None,
    margin_loan: float | None = None,
    ai_review: bool = False,
    ai_model: str = DEFAULT_AI_MODEL,
    ai_review_depth: str = "basic",
    ai_timeout: int = 60,
    ai_max_output_tokens: int = 1200,
    audit_data: bool = False,
    cn: bool = False,
    language: str = "en",
    term_style: str = "pure",
    price_field: str = "adj_close",
    annualization_days: int = 252,
    profile_hint: str | None = None,
) -> dict[str, Any]:
    symbol = symbol.upper()
    benchmark = benchmark.upper()

    out_dir = output_dir_for_run(output, symbol, benchmark, start_date, end_date, archive, run_id)
    if out_dir.exists():
        for generated_name in ["report", "charts", "data", "audit", "ai", "dashboard", "metadata", "self_review", "README.md"]:
            generated_path = out_dir / generated_name
            if generated_path.is_dir():
                shutil.rmtree(generated_path)
            elif generated_path.exists():
                generated_path.unlink()
        for stale_zip in out_dir.glob("*_research_pack.zip"):
            stale_zip.unlink()
    ensure_dir(out_dir)
    pack_dirs = organize_report_pack(out_dir, symbol) if organize_report_pack is not None else {
        "report": out_dir,
        "charts": out_dir,
        "data": out_dir,
        "audit": out_dir,
        "ai": out_dir,
        "dashboard": out_dir,
        "metadata": out_dir,
        "self_review": out_dir,
    }
    report_dir = pack_dirs["report"]
    charts_dir = pack_dirs["charts"]
    data_dir = pack_dirs["data"]
    audit_dir = pack_dirs["audit"]
    ai_dir = pack_dirs["ai"]
    dashboard_dir = pack_dirs["dashboard"]
    metadata_dir = pack_dirs["metadata"]
    self_review_dir = pack_dirs["self_review"]

    if terminal_ui is not None:
        terminal_ui.print_run_config(symbol, benchmark, ai_review, archive_enabled=True, model=ai_model)
    else:
        print(f"\n=== Building research pack: {symbol} vs {benchmark} ===")

    target_price = fetch_price_history(symbol, start_date, end_date)
    benchmark_price = fetch_price_history(benchmark, start_date, end_date)
    if terminal_ui is not None:
        terminal_ui.step_done("[1/8] Fetching market data")

    target_price.to_csv(data_dir / f"{safe_symbol(symbol)}_price_history.csv")
    benchmark_price.to_csv(data_dir / f"{safe_symbol(benchmark)}_price_history.csv")

    close = pd.DataFrame({
        symbol: select_price_series(target_price, price_field),
        benchmark: select_price_series(benchmark_price, price_field),
    }).dropna()

    if close.empty:
        raise ValueError(f"No overlapping price data for {symbol} and {benchmark}.")

    normalized = close / close.iloc[0] * 100
    normalized.to_csv(data_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_normalized.csv")

    actual_chart_path = charts_dir / "Figure_01_price_actual.png"
    chart_path = charts_dir / "Figure_02_price_normalized.png"
    drawdown_chart_path = charts_dir / "Figure_03_drawdown.png"
    interactive_chart_path = dashboard_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_interactive_dashboard.html"
    radar_chart_path = dashboard_dir / f"{safe_symbol(symbol)}_research_score_radar.html"

    plot_actual_close_price(close, symbol, benchmark, actual_chart_path)
    plot_normalized_performance(normalized, symbol, benchmark, chart_path)
    plot_drawdown(close, symbol, benchmark, drawdown_chart_path)
    write_interactive_price_dashboard(close, normalized, symbol, benchmark, interactive_chart_path)

    price_summary = build_price_summary(close[symbol], close[benchmark], risk_free_rate, annualization_days)
    price_summary.to_csv(
        data_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_price_summary.csv",
        index=False,
    )

    info = fetch_company_info(symbol)
    if profile_hint:
        info["_profile_hint"] = profile_hint
    benchmark_info = fetch_company_info(benchmark)
    profile = build_company_profile(info)
    valuation = build_valuation_snapshot(info)

    profile.to_csv(data_dir / f"{safe_symbol(symbol)}_company_profile.csv", index=False)
    valuation.to_csv(data_dir / f"{safe_symbol(symbol)}_valuation_snapshot.csv", index=False)

    if is_fund_like(info):
        trends = pd.DataFrame()
        if terminal_ui is not None:
            terminal_ui.step_warn("[2/8] Loading fundamentals", f"{symbol} appears fund-like; financial statements skipped")
        else:
            print(f"[WARN] {symbol} appears to be a fund-like instrument. Skipping company financial statements.")
    else:
        trends = fetch_money_source_and_flow(symbol, data_dir, years)
        if terminal_ui is not None:
            terminal_ui.step_done("[2/8] Loading fundamentals")

    fundamental_summary = build_fundamental_summary(trends)
    fundamental_summary.to_csv(data_dir / f"{safe_symbol(symbol)}_fundamental_summary.csv", index=False)

    score_table = build_research_score(price_summary, fundamental_summary, info)
    score_table.to_csv(data_dir / f"{safe_symbol(symbol)}_research_potential_score.csv", index=False)
    score_components_chart_path = charts_dir / "Figure_05_score_components.png"

    ruin_risk = build_ruin_risk_snapshot(info, trends)
    ruin_risk.to_csv(data_dir / f"{safe_symbol(symbol)}_ruin_risk_snapshot.csv", index=False)
    ruin_risk_chart_path = charts_dir / "Figure_06_ruin_risk.png"

    growth_quality_chart_path = charts_dir / "Figure_04_growth_quality.png"

    margin_stress = build_margin_stress(account_equity, margin_loan, [0.20, 0.30, 0.50, 0.70])
    if not margin_stress.empty:
        margin_stress.to_csv(data_dir / f"{safe_symbol(symbol)}_personal_margin_stress.csv", index=False)
    if terminal_ui is not None:
        terminal_ui.step_done("[3/8] Calculating metrics")

    data_warnings = build_data_warnings(
        symbol=symbol,
        benchmark=benchmark,
        info=info,
        benchmark_info=benchmark_info,
        target_price=target_price,
        trends=trends,
    )
    sanity_checks = build_sanity_checks(
        symbol=symbol,
        benchmark=benchmark,
        info=info,
        benchmark_info=benchmark_info,
        target_price=target_price,
        trends=trends,
        ruin_risk=ruin_risk,
    )
    sanity_checks.to_csv(data_dir / f"{safe_symbol(symbol)}_sanity_checks.csv", index=False)
    if terminal_ui is not None:
        terminal_ui.step_done("[4/8] Running sanity checks")

    write_metric_radar_chart(score_table, radar_chart_path)
    plot_score_components(score_table, score_components_chart_path)
    plot_ruin_risk_snapshot(ruin_risk, ruin_risk_chart_path)
    plot_growth_quality(trends, growth_quality_chart_path)
    growth_quality_chart_name = growth_quality_chart_path.name if growth_quality_chart_path.exists() else None
    if terminal_ui is not None:
        terminal_ui.step_done("[5/8] Rendering charts")
        terminal_ui.step_done("[6/8] Building report data")

    risk_method = RiskMethod(
        price_field=price_field,
        annualization_days=annualization_days,
        risk_free_rate=risk_free_rate,
        benchmark=benchmark,
    ) if RiskMethod is not None else None

    chart_refs = {
        "actual": f"../charts/{actual_chart_path.name}",
        "normalized": f"../charts/{chart_path.name}",
        "drawdown": f"../charts/{drawdown_chart_path.name}",
        "score_components": f"../charts/{score_components_chart_path.name}",
        "growth_quality": f"../charts/{growth_quality_chart_path.name}" if growth_quality_chart_path.exists() else None,
        "ruin_risk": f"../charts/{ruin_risk_chart_path.name}",
        "interactive": f"../dashboard/{interactive_chart_path.name}",
        "radar": f"../dashboard/{radar_chart_path.name}",
    }

    report_data = generate_report_data_dict(
        symbol=symbol,
        benchmark=benchmark,
        start_date=start_date,
        end_date=end_date,
        out_dir=report_dir,
        profile=profile,
        valuation=valuation,
        trends=trends,
        fundamental_summary=fundamental_summary,
        price_summary=price_summary,
        score_table=score_table,
        info=info,
        target_price=target_price,
        warnings=data_warnings,
        sanity_checks=sanity_checks,
        ruin_risk=ruin_risk,
        margin_stress=margin_stress,
        actual_chart_name=chart_refs["actual"],
        chart_name=chart_refs["normalized"],
        drawdown_chart_name=chart_refs["drawdown"],
        score_components_chart_name=chart_refs["score_components"],
        growth_quality_chart_name=chart_refs["growth_quality"],
        ruin_risk_chart_name=chart_refs["ruin_risk"],
        interactive_chart_name=chart_refs["interactive"],
        radar_chart_name=chart_refs["radar"],
    )

    asset_profile = build_asset_profile(info, fundamental_summary, valuation, trends, ruin_risk) if build_asset_profile is not None else None
    if asset_profile is not None:
        report_data["asset_profile"] = asset_profile.to_dict()
        report_data["research_profile"] = asset_profile.primary_profile

    data_audit_status = STATUS_PASS
    data_audit = None
    if audit_data and risk_method is not None:
        data_audit = build_data_audit(report_data, risk_method)
        data_audit_status = audit_status(data_audit)

    price_check_target = build_price_label_sanity_check(
        symbol,
        target_price,
        float(select_price_series(target_price, price_field).dropna().iloc[-1]),
        safe_info_float(info, "currentPrice"),
    )
    price_check_benchmark = build_price_label_sanity_check(
        benchmark,
        benchmark_price,
        float(select_price_series(benchmark_price, price_field).dropna().iloc[-1]),
        safe_info_float(benchmark_info, "currentPrice"),
    )
    price_label_check = pd.concat([price_check_target, price_check_benchmark], ignore_index=True)
    if STATUS_FAIL in set(price_label_check["status"]):
        data_audit_status = STATUS_FAIL
    elif STATUS_WARNING in set(price_label_check["status"]) and data_audit_status == STATUS_PASS:
        data_audit_status = STATUS_WARNING
    price_label_status = STATUS_FAIL if STATUS_FAIL in set(price_label_check["status"]) else STATUS_WARNING if STATUS_WARNING in set(price_label_check["status"]) else STATUS_PASS

    patch_log = {"patch_status": "NOT_NEEDED", "patch_records": [], "patch_attempts": 0}
    patched_blocks_en = []
    patched_blocks_zh = []
    lifecycle_report = {"status": STATUS_PASS, "failure_reasons": []}
    company_specificity = {"COMPANY_SPECIFICITY_STATUS": STATUS_PASS, "framework_coverage_level": "FULL", "patch_status": "NOT_NEEDED"}
    if asset_profile is not None and build_report_blocks is not None:
        draft_blocks_en = build_report_blocks(report_data, asset_profile, "en")
        draft_blocks_zh = build_report_blocks(report_data, asset_profile, "zh")
        block_map = {block["block_id"]: block["content"] for block in draft_blocks_en + draft_blocks_zh}
        lifecycle_report = lifecycle_logic_check(asset_profile, block_map) if lifecycle_logic_check is not None else lifecycle_report
        patched_blocks_en, patch_log_en = apply_interpretation_patch(draft_blocks_en, asset_profile, lifecycle_report.get("failure_reasons", [])) if apply_interpretation_patch is not None else (draft_blocks_en, patch_log)
        patched_blocks_zh, patch_log_zh = apply_interpretation_patch(draft_blocks_zh, asset_profile, lifecycle_report.get("failure_reasons", [])) if apply_interpretation_patch is not None else (draft_blocks_zh, patch_log)
        patch_log = {
            "patch_status": "APPLIED" if patch_log_en.get("patch_status") == "APPLIED" or patch_log_zh.get("patch_status") == "APPLIED" else "NOT_NEEDED",
            "patch_materiality_status": "MATERIAL_PATCH_APPLIED"
            if "MATERIAL_PATCH_APPLIED" in {patch_log_en.get("patch_materiality_status"), patch_log_zh.get("patch_materiality_status")}
            else "FALLBACK_PATCH_APPLIED"
            if "FALLBACK_PATCH_APPLIED" in {patch_log_en.get("patch_materiality_status"), patch_log_zh.get("patch_materiality_status")}
            else "NO_MATERIAL_CHANGE",
            "patch_attempts": max(patch_log_en.get("patch_attempts", 0), patch_log_zh.get("patch_attempts", 0)),
            "material_patch_count": patch_log_en.get("material_patch_count", 0) + patch_log_zh.get("material_patch_count", 0),
            "fallback_patch_count": patch_log_en.get("fallback_patch_count", 0) + patch_log_zh.get("fallback_patch_count", 0),
            "patch_records": patch_log_en.get("patch_records", []) + patch_log_zh.get("patch_records", []),
        }
        company_specificity = company_specificity_status(asset_profile, lifecycle_report, patch_log) if company_specificity_status is not None else company_specificity

    def block_content(blocks: list[dict[str, Any]], block_id: str) -> str:
        for block in blocks:
            if block.get("block_id") == block_id:
                return block.get("content", "")
        return ""

    v4_sections = {
        "battle_card": block_content(patched_blocks_en, "battle_card") or render_battle_card(report_data, "en"),
        "battle_card_zh": block_content(patched_blocks_zh, "battle_card") or render_battle_card(report_data, "zh"),
        "key_questions": block_content(patched_blocks_en, "key_questions"),
        "key_questions_zh": block_content(patched_blocks_zh, "key_questions"),
        "one_line_verdict": block_content(patched_blocks_en, "one_line_verdict"),
        "one_line_verdict_zh": block_content(patched_blocks_zh, "one_line_verdict"),
        "core_view": block_content(patched_blocks_en, "core_view"),
        "core_view_zh": block_content(patched_blocks_zh, "core_view"),
        "valuation_sensitivity": block_content(patched_blocks_en, "valuation") or render_valuation_sensitivity(report_data, "en"),
        "valuation_sensitivity_zh": block_content(patched_blocks_zh, "valuation") or render_valuation_sensitivity(report_data, "zh"),
        "next_checks": block_content(patched_blocks_en, "next_checks"),
        "next_checks_zh": block_content(patched_blocks_zh, "next_checks"),
        "risk_methodology": render_risk_methodology(risk_method, "en") if risk_method is not None else "",
        "risk_methodology_zh": render_risk_methodology(risk_method, "zh") if risk_method is not None else "",
        "segment_revenue": render_segment_revenue(report_data, "en"),
        "segment_revenue_zh": render_segment_revenue(report_data, "zh"),
    }
    language_preview = "\n".join(v4_sections.values())
    lint_en = lint_language(language_preview, "en")
    lint_zh = lint_language(language_preview, "zh") if cn else {"language": "zh", "banned_phrase_hits": [], "overlong_sentences": [], "overlong_sections": [], "rewritten_sections": [], "rewrite_attempts": 0, "final_status": STATUS_PASS}
    language_status = STATUS_FAIL if STATUS_FAIL in {lint_en["final_status"], lint_zh["final_status"]} else STATUS_WARNING if STATUS_WARNING in {lint_en["final_status"], lint_zh["final_status"]} else STATUS_PASS
    risk_status = risk_method_status(risk_method) if risk_method is not None else STATUS_FAIL
    if asset_profile is not None and build_asset_aware_ai_correction_log is not None:
        ai_correction_log = build_asset_aware_ai_correction_log(asset_profile, symbol, benchmark, "en")
    else:
        ai_correction_log = build_ai_correction_log(report_data, "en")
    ai_analyst_status = validate_ai_correction_log(ai_correction_log)
    gate_status = {
        "DATA_AUDIT_STATUS": data_audit_status,
        "RISK_METHOD_STATUS": risk_status,
        "AI_ANALYST_REVIEW_STATUS": ai_analyst_status,
        "LANGUAGE_LINT_STATUS": language_status,
        "PRICE_LABEL_CHECK_STATUS": price_label_status,
        "LIFECYCLE_LOGIC_STATUS": lifecycle_report.get("status", STATUS_PASS),
        "COMPANY_SPECIFICITY_STATUS": company_specificity.get("COMPANY_SPECIFICITY_STATUS", STATUS_PASS),
        "PATCH_STATUS": patch_log.get("patch_status", "NOT_NEEDED"),
        "PATCH_MATERIALITY_STATUS": patch_log.get("patch_materiality_status", "NO_MATERIAL_CHANGE"),
        "FALLBACK_STATUS": fallback_status(asset_profile) if asset_profile is not None and fallback_status is not None else "NONE",
        "FALLBACK_USED_COUNT": asset_profile.fallback_used_count if asset_profile is not None else 0,
        "FRAMEWORK_COVERAGE_LEVEL": asset_profile.framework_coverage_level if asset_profile is not None else "UNKNOWN",
        "ASSET_PROFILE": asset_profile.primary_profile if asset_profile is not None else classify_research_category(info, fundamental_summary),
    }
    if rollup_data_verification_status is not None and rollup_thesis_verification_status is not None and overall_status_from_verification is not None:
        gate_status["DATA_VERIFICATION_STATUS"] = rollup_data_verification_status(gate_status)
        gate_status["THESIS_VERIFICATION_STATUS"] = rollup_thesis_verification_status(gate_status)
        gate_status["OVERALL_REPORT_STATUS"] = overall_status_from_verification(
            gate_status["DATA_VERIFICATION_STATUS"],
            gate_status["THESIS_VERIFICATION_STATUS"],
        )
    elif overall_status_v43 is not None:
        gate_status["OVERALL_REPORT_STATUS"] = overall_status_v43(gate_status, fallback_count=asset_profile.fallback_used_count if asset_profile is not None else 0)
    else:
        gate_status["OVERALL_REPORT_STATUS"] = overall_report_status(gate_status)

    overall_status = gate_status["OVERALL_REPORT_STATUS"]
    if data_audit is not None:
        write_data_audit(audit_dir, data_audit, overall_status)
    write_price_label_sanity_check(audit_dir, price_label_check, overall_status)
    write_ai_correction_log(ai_dir, ai_correction_log, overall_status)
    if write_patch_artifacts is not None:
        write_patch_artifacts(ai_dir, patch_log, patched_blocks_en + patched_blocks_zh)
    write_language_lint_report(audit_dir, [lint_en, lint_zh], overall_status)
    if write_json is not None:
        if asset_profile is not None:
            write_json(metadata_dir / "asset_profile.json", asset_profile.to_dict())
        write_json(audit_dir / "lifecycle_logic_report.json", lifecycle_report)
        write_json(audit_dir / "company_specificity_report.json", company_specificity)
        write_json(metadata_dir / "report_status.json", gate_status)
    if asset_profile is not None and write_lifecycle_logic_report is not None:
        write_lifecycle_logic_report(audit_dir / "lifecycle_logic_report.md", lifecycle_report, asset_profile)

    ai_review_markdown = None
    if ai_review:
        if build_ai_review_payload is None or call_ai_review is None or render_ai_review_markdown is None or render_ai_review_skipped is None:
            reason = "AI Review module could not be loaded."
            ai_review_markdown = "## AI Review\n\nAI Review was requested but skipped.\n\nReason: AI Review module could not be loaded.\n\nThe deterministic report was still generated successfully.\n"
            if terminal_ui is not None:
                terminal_ui.print_ai_review_status("skipped", model=ai_model, error=reason)
        else:
            payload = build_ai_review_payload(report_data, depth=ai_review_depth)
            if terminal_ui is not None:
                with terminal_ui.ai_review_spinner():
                    review = call_ai_review(payload, model=ai_model, timeout=ai_timeout, max_output_tokens=ai_max_output_tokens)
            else:
                print("Calling OpenAI API, this may take a few seconds...")
                review = call_ai_review(payload, model=ai_model, timeout=ai_timeout, max_output_tokens=ai_max_output_tokens)
            error = getattr(call_ai_review, "last_error", None)
            if review is None:
                ai_review_markdown = render_ai_review_skipped(error or "AI Review was not available.")
                if terminal_ui is not None:
                    terminal_ui.print_ai_review_status("skipped", model=ai_model, error=error)
            else:
                ai_review_markdown = render_ai_review_markdown(review)
                if terminal_ui is not None:
                    terminal_ui.print_ai_review_status("completed", model=ai_model)
    elif terminal_ui is not None:
        terminal_ui.print_ai_review_status("disabled")

    report_path = write_report(
        symbol=symbol,
        benchmark=benchmark,
        start_date=start_date,
        end_date=end_date,
        out_dir=report_dir,
        profile=profile,
        valuation=valuation,
        trends=trends,
        fundamental_summary=fundamental_summary,
        price_summary=price_summary,
        score_table=score_table,
        info=info,
        target_price=target_price,
        warnings=data_warnings,
        sanity_checks=sanity_checks,
        ruin_risk=ruin_risk,
        margin_stress=margin_stress,
        actual_chart_name=chart_refs["actual"],
        chart_name=chart_refs["normalized"],
        drawdown_chart_name=chart_refs["drawdown"],
        score_components_chart_name=chart_refs["score_components"],
        growth_quality_chart_name=chart_refs["growth_quality"],
        ruin_risk_chart_name=chart_refs["ruin_risk"],
        interactive_chart_name=chart_refs["interactive"],
        radar_chart_name=chart_refs["radar"],
        ai_review_markdown=ai_review_markdown,
        v4_sections=v4_sections,
        gate_status=gate_status,
        asset_profile_data=asset_profile.to_dict() if asset_profile is not None else None,
    )
    final_lint_results = [lint_language(report_path.read_text(encoding="utf-8"), "en")]
    if cn or language in {"zh", "both"}:
        chinese_report_path = write_chinese_report(report_data, report_dir, v4_sections, gate_status, term_style=term_style)
        final_lint_results.append(lint_language(chinese_report_path.read_text(encoding="utf-8"), "zh"))
    else:
        final_lint_results.append(lint_zh)
    write_language_lint_report(audit_dir, final_lint_results, gate_status["OVERALL_REPORT_STATUS"])

    if write_json is not None:
        write_json(
            metadata_dir / "run_metadata.json",
            {
                "ticker": symbol,
                "benchmark": benchmark,
                "run_id": out_dir.name,
                "version": __version__,
                "generated_at": datetime.now().isoformat(timespec="seconds"),
                "report_path": str(report_path.relative_to(out_dir)),
                "chinese_report_path": str(chinese_report_path.relative_to(out_dir)) if "chinese_report_path" in locals() else None,
            },
        )
        write_json(metadata_dir / "report_status.json", gate_status)

    if asset_profile is not None:
        self_review_data = {
            "ticker": symbol,
            "run_id": out_dir.name,
            "asset_profile": asset_profile.to_dict(),
            "lifecycle_logic_status": lifecycle_report.get("status", STATUS_PASS),
            "company_specificity_status": gate_status.get("COMPANY_SPECIFICITY_STATUS"),
            "patch_status": patch_log.get("patch_status"),
        }
        if write_system_self_review is not None:
            write_system_self_review(self_review_dir / "system_self_review.md", self_review_data)
        if write_framework_gap_analysis is not None:
            write_framework_gap_analysis(self_review_dir / "framework_gap_analysis.md", asset_profile)
        if write_improvement_suggestions is not None:
            write_improvement_suggestions(self_review_dir / "improvement_suggestions.md", asset_profile)
        if write_regression_test_suggestions is not None:
            write_regression_test_suggestions(self_review_dir / "regression_test_suggestions.md", asset_profile)

    if write_run_readme is not None:
        write_run_readme(out_dir, safe_symbol(symbol), benchmark, gate_status)
    if presentation_gate is not None:
        presentation = presentation_gate(out_dir)
        gate_status["PRESENTATION_STATUS"] = presentation.get("PRESENTATION_STATUS", STATUS_PASS)
        if write_json is not None:
            write_json(audit_dir / "presentation_report.json", presentation)
            write_json(metadata_dir / "report_status.json", gate_status)

    latest_dir = output / safe_symbol(symbol) / "latest"
    if copy_pack_to_latest is not None:
        copy_pack_to_latest(out_dir, latest_dir)
    else:
        copy_run_to_latest(out_dir, latest_dir)
    if terminal_ui is not None:
        terminal_ui.step_done("[8/8] Writing outputs")
        if gate_status.get("OVERALL_REPORT_STATUS") == "UNVERIFIED":
            terminal_ui.step_error(
                "WARNING: One or more validation gates failed.",
                "Generated report is marked UNVERIFIED. Check data_audit.md, ai_correction_log.md, language_lint_report.md, price_label_sanity_check.md",
            )
    elif gate_status.get("OVERALL_REPORT_STATUS") == "UNVERIFIED":
        print("WARNING: One or more validation gates failed.")
        print("Generated report is marked UNVERIFIED.")
        print("Check: data_audit.md, ai_correction_log.md, language_lint_report.md, price_label_sanity_check.md")

    score = get_component_score(score_table, "Research Score")
    rating = rating_from_score(score)

    if terminal_ui is not None:
        terminal_ui.print_final_outputs(report_path, interactive_chart_path, out_dir)
        print(f"Score: {fmt_score(score)} ({rating})")
    else:
        print(f"Done: {symbol}")
        print(f"Report: {report_path}")
        print(f"Score: {fmt_score(score)} ({rating})")

    return {
        "symbol": symbol,
        "benchmark": benchmark,
        "score": score,
        "rating": rating,
        "report": str(report_path),
        "folder": str(out_dir),
        "latest_folder": str(latest_dir),
        "profile": classify_research_category(info, fundamental_summary),
        "total_return": get_metric(price_summary, "Total Return", "Target"),
        "benchmark_return": get_metric(price_summary, "Total Return", "Benchmark"),
        "excess_return": get_metric(price_summary, "Total Return", "Difference"),
        "cagr": get_metric(price_summary, "CAGR", "Target"),
        "max_drawdown": get_metric(price_summary, "Max Drawdown", "Target"),
        "volatility": get_metric(price_summary, "Annualized Volatility", "Target"),
        "revenue_cagr": get_metric(fundamental_summary, "Revenue CAGR"),
        "gross_margin_latest": get_metric(fundamental_summary, "Gross Margin Latest"),
        "fcf_margin_latest": get_metric(fundamental_summary, "FCF Margin Latest"),
    }


def write_cross_ticker_comparison(
    rows: list[dict[str, Any]],
    output: Path,
    archive: bool = False,
    run_id: str | None = None,
) -> None:
    if not rows:
        return

    comparison_run_id = run_id or datetime.now().strftime("%Y%m%d_%H%M%S")
    out_dir = output / "_comparison" / "runs" / comparison_run_id
    ensure_dir(out_dir)

    df = pd.DataFrame(rows).sort_values("score", ascending=False, na_position="last")
    df.to_csv(out_dir / "cross_ticker_comparison.csv", index=False)

    report = f"""# Cross Ticker Comparison

This table compares all tickers generated in the same run.

The score is a research-priority score, not a buy/sell signal.

{markdown_table(
    df,
    max_rows=100,
    percent_columns={
        "total_return",
        "benchmark_return",
        "excess_return",
        "cagr",
        "max_drawdown",
        "volatility",
        "revenue_cagr",
        "gross_margin_latest",
        "fcf_margin_latest",
    },
    score_columns={"score"},
)}

## How to Use

- High score means the ticker may deserve deeper research.
- Low score means weak evidence, high risk, poor data, or poor fundamentals.
- Always verify with primary sources before making any decision.
"""
    save_text(out_dir / "cross_ticker_comparison.md", report)
    copy_run_to_latest(out_dir, output / "_comparison" / "latest")


def pack_report_folder(run_dir: Path) -> Path:
    """Create a zip archive for a v4.3 report run folder."""
    run_dir = run_dir.expanduser().resolve()
    if not run_dir.exists() or not run_dir.is_dir():
        raise FileNotFoundError(f"Run folder not found: {run_dir}")
    ticker = run_dir.parents[1].name if len(run_dir.parents) >= 2 and run_dir.parent.name == "runs" else run_dir.name.split("_")[0].upper()
    zip_path = run_dir / f"{safe_symbol(ticker)}_research_pack.zip"
    required = ["README.md", "report", "charts", "data", "audit", "ai", "dashboard", "metadata", "self_review"]
    missing = [item for item in required if not (run_dir / item).exists()]
    if missing:
        raise ValueError(f"Cannot pack incomplete run folder. Missing: {', '.join(missing)}")
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(run_dir.rglob("*")):
            if path == zip_path or path.is_dir():
                continue
            archive.write(path, path.relative_to(run_dir))
    return zip_path


def parse_args() -> argparse.Namespace:
    examples = """
Examples:

  # Basic asset-aware research pack
  openbb-research AAPL

  # Chinese report only
  openbb-research AAPL --zh

  # Multiple tickers ranked together
  openbb-research AAPL TSLA RKLB

  # Use VOO as benchmark
  openbb-research TSLA --benchmark VOO

  # Use QQQ for technology/growth comparison
  openbb-research NVDA MSFT --benchmark QQQ

  # Compare one stock against another stock
  openbb-research TSLA --benchmark AAPL --start 2020-01-01

  # Custom risk-free rate
  openbb-research AAPL --risk-free-rate 0.04

  # Optional personal margin stress table
  openbb-research AAPL --account-equity 100000 --margin-loan 25000

  # Optional AI review layer
  openbb-research AAPL --ai-review

  # Generate English and Chinese reports
  openbb-research RKLB --both --full

  # Generate and zip the organized report pack
  openbb-research TICKER --pack
  openbb-research RKLB --both --full --pack

  # Pack an existing run folder
  openbb-research pack RUN_FOLDER
  openbb-research pack reports/RKLB/runs/manual_review_rklb_v43

  # Batch evaluation and local self-training cases
  openbb-research batch eval_sets/smoke_12.yaml
  openbb-research batch eval_sets/broad_200.yaml --both --full --pack --no-ai
  openbb-research batch eval_sets/broad_200.yaml --ai-review-failures --max-ai-reviews 40 --resume

  # Use your own run id
  openbb-research AAPL --run-id test_2023_start
"""
    parser = argparse.ArgumentParser(
        prog="openbb-research",
        description="Generate an asset-aware first-pass equity research pack with benchmark comparison, audit logs, charts, and English/Chinese reports.",
        epilog=textwrap.dedent(examples),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("symbols", nargs="+", help="Target ticker(s), e.g. AAPL TSLA RKLB.")
    parser.add_argument("--benchmark", default="SPY", help="Benchmark ticker. Default: SPY. Examples: VOO, QQQ, AAPL.")
    parser.add_argument("--start", default="2023-01-01", help="Start date, YYYY-MM-DD. Default: 2023-01-01.")
    parser.add_argument("--end", default=None, help="Optional end date, YYYY-MM-DD. Default: latest available.")
    parser.add_argument("--years", type=int, default=5, help="Financial years to include. Default: 5.")
    parser.add_argument("--output", default="reports", help="Output folder. Default: reports.")
    parser.add_argument("--risk-free-rate", type=float, default=0.0, help="Annual risk-free rate for risk-adjusted metrics. Example: 0.04.")
    parser.add_argument("--price-field", choices=["close", "adj_close"], default="adj_close", help="Price field for return and risk metrics. Default: adj_close with close fallback.")
    parser.add_argument("--annualization-days", type=int, default=252, help="Trading days used for annualized risk metrics. Default: 252.")
    parser.add_argument("--archive", action="store_true", help="Compatibility flag. v2 archives every run and refreshes latest automatically.")
    parser.add_argument("--run-id", default=None, help="Optional archive folder name under reports/TICKER/runs/.")
    parser.add_argument("--account-equity", type=float, default=None, help="Optional account equity for personal margin stress testing.")
    parser.add_argument("--margin-loan", type=float, default=None, help="Optional margin loan balance for personal margin stress testing.")
    parser.add_argument("--ai-review", action="store_true", help="Add optional OpenAI AI Review section to the report.")
    parser.add_argument("--ai-model", default=os.getenv("OPENAI_MODEL", DEFAULT_AI_MODEL), help=f"AI model for --ai-review. Default: OPENAI_MODEL or {DEFAULT_AI_MODEL}.")
    parser.add_argument("--ai-review-depth", choices=["basic", "deep"], default="basic", help="AI review depth. Default: basic.")
    parser.add_argument("--ai-timeout", type=int, default=60, help="OpenAI API timeout in seconds. Default: 60.")
    parser.add_argument("--ai-max-output-tokens", type=int, default=1200, help="Max tokens for AI review output. Default: 1200.")
    parser.add_argument("--audit-data", dest="audit_data", action="store_true", default=True, help="Generate data audit files. Default: enabled in v4.3.")
    parser.add_argument("--no-audit-data", dest="audit_data", action="store_false", help="Disable data audit file generation.")
    parser.add_argument("--language", choices=["en", "zh", "both"], default="both", help="Report language. Default: both.")
    parser.add_argument("--en", dest="language", action="store_const", const="en", help="Generate English report only.")
    parser.add_argument("--zh", dest="language", action="store_const", const="zh", help="Generate Chinese report only.")
    parser.add_argument("--both", dest="language", action="store_const", const="both", help="Generate both English and Chinese reports.")
    parser.add_argument("--term-style", choices=["pure", "bilingual"], default="pure", help="Term display style for localized reports. Default: pure.")
    parser.add_argument("--cn", "--chinese", dest="language", action="store_const", const="zh", help="Generate an independent Chinese Markdown report.")
    parser.add_argument("--quick", action="store_true", help="Fast mode placeholder. Keeps core report generation but skips optional AI review unless requested.")
    parser.add_argument("--full", action="store_true", help="Full research mode. In v4.3 this is the default behavior.")
    parser.add_argument("--pack", action="store_true", help="Create TICKER_research_pack.zip in each run folder after generation.")
    parser.add_argument("--no-rich", action="store_true", help="Disable Rich terminal UI and use plain output.")
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    args = parser.parse_args()
    args.cn = args.language in {"zh", "both"}
    return args


def main() -> None:
    if len(sys.argv) >= 2 and sys.argv[1] == "batch":
        try:
            from batch_eval import main as batch_main
            batch_main(sys.argv[2:])
        except Exception as exc:
            print(f"[ERROR] Batch evaluation failed: {exc}")
            raise SystemExit(1)
        return

    if len(sys.argv) >= 2 and sys.argv[1] == "pack":
        if len(sys.argv) < 3:
            print("Usage: openbb-research pack RUN_FOLDER")
            raise SystemExit(2)
        try:
            zip_path = pack_report_folder(Path(sys.argv[2]))
        except Exception as exc:
            print(f"[ERROR] Failed to pack report folder: {exc}")
            raise SystemExit(1)
        print(f"Packed report folder: {zip_path}")
        return

    args = parse_args()
    output = Path(args.output)
    ensure_dir(output)

    if terminal_ui is not None:
        terminal_ui.configure(enabled=not args.no_rich)
        terminal_ui.print_app_banner(__version__)

    rows = []
    for symbol in args.symbols:
        try:
            row = run_one(
                symbol=symbol,
                benchmark=args.benchmark,
                start_date=args.start,
                end_date=args.end,
                years=args.years,
                output=output,
                risk_free_rate=args.risk_free_rate,
                archive=args.archive,
                run_id=args.run_id,
                account_equity=args.account_equity,
                margin_loan=args.margin_loan,
                ai_review=args.ai_review,
                ai_model=args.ai_model,
                ai_review_depth=args.ai_review_depth,
                ai_timeout=args.ai_timeout,
                ai_max_output_tokens=args.ai_max_output_tokens,
                audit_data=args.audit_data,
                cn=args.language in {"zh", "both"},
                language=args.language,
                term_style=args.term_style,
                price_field=args.price_field,
                annualization_days=args.annualization_days,
            )
            if args.pack:
                zip_path = pack_report_folder(Path(row["folder"]))
                row["pack_zip"] = str(zip_path)
                print(f"Pack zip: {zip_path}")
            rows.append(row)
        except Exception as exc:
            print(f"\n[ERROR] Failed for {symbol}: {exc}")

    write_cross_ticker_comparison(rows, output, archive=args.archive, run_id=args.run_id)


if __name__ == "__main__":
    main()
