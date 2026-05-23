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

__version__ = "3.0.0"

import argparse
from datetime import datetime
import math
import os
import shutil
import tempfile
import textwrap
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


def fmt_number(value: Any) -> str:
    if value is None:
        return "N/A"
    try:
        if pd.isna(value):
            return "N/A"
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
    return f"{value:.4f}"


def fmt_percent(value: Any) -> str:
    if value is None:
        return "N/A"
    try:
        if pd.isna(value):
            return "N/A"
        return f"{float(value):.2%}"
    except Exception:
        return "N/A"


def fmt_score(value: Any) -> str:
    if value is None:
        return "N/A"
    try:
        if pd.isna(value):
            return "N/A"
        return f"{float(value):.2f} / 100"
    except Exception:
        return "N/A"


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
    "Ruin Risk Score",
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
        return "N/A"

    try:
        if pd.isna(value):
            return "N/A"
    except Exception:
        pass

    if kind == "percent":
        return fmt_percent(value)

    if kind == "score":
        return fmt_score(value)

    if kind == "ratio":
        try:
            return f"{float(value):.4f}"
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
        return "_No data available._"

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


def annualized_volatility(close: pd.Series) -> float:
    r = daily_returns(close)
    if r.empty:
        return float("nan")
    return float(r.std() * math.sqrt(252))


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


def sharpe_ratio(close: pd.Series, risk_free_rate: float = 0.0) -> float:
    vol = annualized_volatility(close)
    if vol == 0 or pd.isna(vol):
        return float("nan")
    return float((cagr(close) - risk_free_rate) / vol)


def sortino_ratio(close: pd.Series, risk_free_rate: float = 0.0) -> float:
    r = daily_returns(close)
    if r.empty:
        return float("nan")
    downside = r[r < 0].std() * math.sqrt(252)
    if downside == 0 or pd.isna(downside):
        return float("nan")
    return float((cagr(close) - risk_free_rate) / downside)


def calmar_ratio(close: pd.Series) -> float:
    dd = abs(max_drawdown(close))
    if dd == 0 or pd.isna(dd):
        return float("nan")
    return float(cagr(close) / dd)


def beta_alpha(target: pd.Series, benchmark: pd.Series, risk_free_rate: float = 0.0) -> tuple[float, float]:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan"), float("nan")

    var_b = r["benchmark"].var()
    if var_b == 0 or pd.isna(var_b):
        return float("nan"), float("nan")

    beta = float(r["target"].cov(r["benchmark"]) / var_b)
    target_ann = r["target"].mean() * 252
    bench_ann = r["benchmark"].mean() * 252
    alpha = float(target_ann - (risk_free_rate + beta * (bench_ann - risk_free_rate)))
    return beta, alpha


def correlation(target: pd.Series, benchmark: pd.Series) -> float:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan")
    return float(r["target"].corr(r["benchmark"]))


def tracking_error(target: pd.Series, benchmark: pd.Series) -> float:
    df = pd.DataFrame({"target": target, "benchmark": benchmark}).dropna()
    r = df.pct_change().dropna()
    if r.empty:
        return float("nan")
    diff = r["target"] - r["benchmark"]
    return float(diff.std() * math.sqrt(252))


def information_ratio(target: pd.Series, benchmark: pd.Series) -> float:
    excess = cagr(target) - cagr(benchmark)
    te = tracking_error(target, benchmark)
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


def build_price_summary(target_close: pd.Series, benchmark_close: pd.Series, risk_free_rate: float) -> pd.DataFrame:
    beta, alpha = beta_alpha(target_close, benchmark_close, risk_free_rate)
    upside, downside = capture_ratios(target_close, benchmark_close)

    rows = [
        ("Total Return", total_return(target_close), total_return(benchmark_close)),
        ("CAGR", cagr(target_close), cagr(benchmark_close)),
        ("1Y Return", rolling_return(target_close, 252), rolling_return(benchmark_close, 252)),
        ("6M Return", rolling_return(target_close, 126), rolling_return(benchmark_close, 126)),
        ("3M Return", rolling_return(target_close, 63), rolling_return(benchmark_close, 63)),
        ("Max Drawdown", max_drawdown(target_close), max_drawdown(benchmark_close)),
        ("Annualized Volatility", annualized_volatility(target_close), annualized_volatility(benchmark_close)),
        ("Sharpe Ratio", sharpe_ratio(target_close, risk_free_rate), sharpe_ratio(benchmark_close, risk_free_rate)),
        ("Sortino Ratio", sortino_ratio(target_close, risk_free_rate), sortino_ratio(benchmark_close, risk_free_rate)),
        ("Calmar Ratio", calmar_ratio(target_close), calmar_ratio(benchmark_close)),
        ("Beta vs Benchmark", beta, None),
        ("Alpha vs Benchmark", alpha, None),
        ("Correlation vs Benchmark", correlation(target_close, benchmark_close), None),
        ("Tracking Error", tracking_error(target_close, benchmark_close), None),
        ("Information Ratio", information_ratio(target_close, benchmark_close), None),
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
    fig, ax = plt.subplots(figsize=(12.5, 6))
    palette = [CHART_COLORS["target"], CHART_COLORS["positive"], CHART_COLORS["accent"], CHART_COLORS["negative"]]
    for idx, col in enumerate(cols):
        ax.plot(trends.index, trends[col], marker="o", linewidth=2.0, label=col, color=palette[idx % len(palette)])
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
    view = ruin_risk[ruin_risk["Metric"].isin(["Net Debt / EBITDA", "Debt / FCF", "Cash Runway Years", "Ruin Risk Score"])].copy()
    if view.empty:
        return
    view["Display Value"] = pd.to_numeric(view["Value"], errors="coerce").fillna(0)
    colors = [CHART_COLORS["negative"] if m == "Ruin Risk Score" else CHART_COLORS["benchmark"] for m in view["Metric"]]
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.bar(view["Metric"], view["Display Value"], color=colors)
    ax.set_title("Ruin risk: balance sheet and cash-flow fragility", loc="left", color=CHART_COLORS["text"], pad=22)
    add_subtitle(ax, "Higher values deserve manual review; this is not a bankruptcy model")
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
    try:
        return yf.Ticker(symbol).get_info()
    except Exception as exc:
        print(f"[WARN] yfinance company info failed for {symbol}: {exc}")
        return {}


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

    sections = []
    for group, group_df in valuation.groupby("Group", sort=False):
        sections.append(f"### {group}")
        sections.append(markdown_table(group_df[["Metric", "Value"]], max_rows=50))
    return "\n\n".join(sections)


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

    risk_score = 50.0
    if not pd.isna(net_debt_to_ebitda):
        risk_score += clamp(net_debt_to_ebitda * 12, 0, 35)
    if not pd.isna(debt_to_fcf):
        risk_score += clamp((debt_to_fcf - 3) * 8, 0, 25)
    if not pd.isna(cash_runway):
        risk_score += 25 if cash_runway < 2 else 10 if cash_runway < 4 else 0
    if not pd.isna(fcf) and fcf < 0 and pd.isna(cash_runway):
        risk_score += 20
    risk_score = clamp(risk_score)

    rows = [
        ("Net Debt", net_debt, "Total debt minus cash. Negative is net cash."),
        ("EBITDA", ebitda, "Provider EBITDA, when available."),
        ("Net Debt / EBITDA", net_debt_to_ebitda, "Debt-load proxy. Higher values deserve manual stress testing."),
        ("Debt / FCF", debt_to_fcf, "Debt compared with free cash flow. Not useful when FCF is negative."),
        ("Cash Runway Years", cash_runway, "Approximate years of cash runway when FCF is negative."),
        ("Ruin Risk Score", risk_score, "Heuristic fragility score. Higher means more balance-sheet or cash-burn pressure."),
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

        ruin_score = get_metric(ruin_risk, "Ruin Risk Score")
        if not pd.isna(ruin_score) and ruin_score >= 75:
            add(
                "HIGH",
                "Ruin risk",
                f"Ruin Risk Score is {fmt_number(ruin_score)}.",
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
    if score >= 75:
        return "High"
    if score >= 45:
        return "Medium"
    return "Low"


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
    ruin_score = get_metric(ruin_risk, "Ruin Risk Score")
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
            "Plain-English Meaning": "Debt or cash-flow fragility deserves manual review." if not pd.isna(ruin_score) and ruin_score >= 75 else "Debt and cash-flow fragility do not appear to be the main first-pass risk.",
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
    ruin_score = get_metric(ruin_risk, "Ruin Risk Score")
    if pd.isna(ruin_score):
        return "Ruin risk data is incomplete. Debt, cash, EBITDA, and free cash flow should be checked manually."
    if ruin_score >= 75:
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

    total_score = report_data["research_score"]["score"]
    rating = report_data["research_status"]

    verdict = report_data["one_line_verdict"]
    takeaways = report_data["key_takeaways"]
    takeaways_md = "\n".join([f"- {item}" for item in takeaways]) if takeaways else "_No automatic takeaways available._"
    beginner_summary = pd.DataFrame(report_data["beginner_summary"])

    category = report_data["research_profile"]
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

    report = f"""# {symbol} Research Report

> Target: `{symbol}`  
> Benchmark: `{benchmark}`  
> Period: `{start_date}` to `{end_date or "latest available"}`  
> Research Status: **{rating}**  
> Research Profile: **{category}**  
> Version: `{__version__}`

---

## 0. Boundary

This report is a structured first-pass research workflow.

It does **not** provide:

- Buy / sell recommendation
- Target price
- Guaranteed return
- Automatic investment decision

The score is a **research prioritization score**, not a prediction.

---

## 1. One-line Verdict

{verdict}

---

## 2. How to Read This Report

{how_to_read_report_section()}

---

## 3. Key Takeaways

{takeaways_md}

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

    path = out_dir / f"{safe_symbol(symbol)}_research_report.md"
    save_text(path, report)
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
) -> dict[str, Any]:
    symbol = symbol.upper()
    benchmark = benchmark.upper()

    out_dir = output_dir_for_run(output, symbol, benchmark, start_date, end_date, archive, run_id)
    ensure_dir(out_dir)

    if terminal_ui is not None:
        terminal_ui.print_run_config(symbol, benchmark, ai_review, archive_enabled=True, model=ai_model)
    else:
        print(f"\n=== Building research pack: {symbol} vs {benchmark} ===")

    target_price = fetch_price_history(symbol, start_date, end_date)
    benchmark_price = fetch_price_history(benchmark, start_date, end_date)
    if terminal_ui is not None:
        terminal_ui.step_done("[1/8] Fetching market data")

    target_price.to_csv(out_dir / f"{safe_symbol(symbol)}_price_history.csv")
    benchmark_price.to_csv(out_dir / f"{safe_symbol(benchmark)}_price_history.csv")

    close = pd.DataFrame({
        symbol: target_price["close"],
        benchmark: benchmark_price["close"],
    }).dropna()

    if close.empty:
        raise ValueError(f"No overlapping price data for {symbol} and {benchmark}.")

    normalized = close / close.iloc[0] * 100
    normalized.to_csv(out_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_normalized.csv")

    actual_chart_path = out_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_actual_close_price_chart.png"
    chart_path = out_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_performance_chart.png"
    drawdown_chart_path = out_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_drawdown_chart.png"
    interactive_chart_path = out_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_interactive_dashboard.html"
    radar_chart_path = out_dir / f"{safe_symbol(symbol)}_research_score_radar.html"

    plot_actual_close_price(close, symbol, benchmark, actual_chart_path)
    plot_normalized_performance(normalized, symbol, benchmark, chart_path)
    plot_drawdown(close, symbol, benchmark, drawdown_chart_path)
    write_interactive_price_dashboard(close, normalized, symbol, benchmark, interactive_chart_path)

    price_summary = build_price_summary(close[symbol], close[benchmark], risk_free_rate)
    price_summary.to_csv(
        out_dir / f"{safe_symbol(symbol)}_vs_{safe_symbol(benchmark)}_price_summary.csv",
        index=False,
    )

    info = fetch_company_info(symbol)
    benchmark_info = fetch_company_info(benchmark)
    profile = build_company_profile(info)
    valuation = build_valuation_snapshot(info)

    profile.to_csv(out_dir / f"{safe_symbol(symbol)}_company_profile.csv", index=False)
    valuation.to_csv(out_dir / f"{safe_symbol(symbol)}_valuation_snapshot.csv", index=False)

    if is_fund_like(info):
        trends = pd.DataFrame()
        if terminal_ui is not None:
            terminal_ui.step_warn("[2/8] Loading fundamentals", f"{symbol} appears fund-like; financial statements skipped")
        else:
            print(f"[WARN] {symbol} appears to be a fund-like instrument. Skipping company financial statements.")
    else:
        trends = fetch_money_source_and_flow(symbol, out_dir, years)
        if terminal_ui is not None:
            terminal_ui.step_done("[2/8] Loading fundamentals")

    fundamental_summary = build_fundamental_summary(trends)
    fundamental_summary.to_csv(out_dir / f"{safe_symbol(symbol)}_fundamental_summary.csv", index=False)

    score_table = build_research_score(price_summary, fundamental_summary, info)
    score_table.to_csv(out_dir / f"{safe_symbol(symbol)}_research_potential_score.csv", index=False)
    score_components_chart_path = out_dir / f"{safe_symbol(symbol)}_research_score_components.png"

    ruin_risk = build_ruin_risk_snapshot(info, trends)
    ruin_risk.to_csv(out_dir / f"{safe_symbol(symbol)}_ruin_risk_snapshot.csv", index=False)
    ruin_risk_chart_path = out_dir / f"{safe_symbol(symbol)}_ruin_risk_snapshot.png"

    growth_quality_chart_path = out_dir / f"{safe_symbol(symbol)}_growth_quality_trend.png"

    margin_stress = build_margin_stress(account_equity, margin_loan, [0.20, 0.30, 0.50, 0.70])
    if not margin_stress.empty:
        margin_stress.to_csv(out_dir / f"{safe_symbol(symbol)}_personal_margin_stress.csv", index=False)
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
    sanity_checks.to_csv(out_dir / f"{safe_symbol(symbol)}_sanity_checks.csv", index=False)
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

    ai_review_markdown = None
    if ai_review:
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
            warnings=data_warnings,
            sanity_checks=sanity_checks,
            ruin_risk=ruin_risk,
            margin_stress=margin_stress,
            actual_chart_name=actual_chart_path.name,
            chart_name=chart_path.name,
            drawdown_chart_name=drawdown_chart_path.name,
            score_components_chart_name=score_components_chart_path.name,
            growth_quality_chart_name=growth_quality_chart_name,
            ruin_risk_chart_name=ruin_risk_chart_path.name,
            interactive_chart_name=interactive_chart_path.name,
            radar_chart_name=radar_chart_path.name,
        )
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
        out_dir=out_dir,
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
        actual_chart_name=actual_chart_path.name,
        chart_name=chart_path.name,
        drawdown_chart_name=drawdown_chart_path.name,
        score_components_chart_name=score_components_chart_path.name,
        growth_quality_chart_name=growth_quality_chart_name,
        ruin_risk_chart_name=ruin_risk_chart_path.name,
        interactive_chart_name=interactive_chart_path.name,
        radar_chart_name=radar_chart_path.name,
        ai_review_markdown=ai_review_markdown,
    )

    latest_dir = output / safe_symbol(symbol) / "latest"
    copy_run_to_latest(out_dir, latest_dir)
    if terminal_ui is not None:
        terminal_ui.step_done("[8/8] Writing outputs")

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


def parse_args() -> argparse.Namespace:
    examples = """
Examples:

  # Basic: AAPL vs SPY
  cresearch AAPL

  # Multiple tickers ranked together
  cresearch AAPL TSLA RKLB

  # Use VOO as benchmark
  cresearch TSLA --benchmark VOO

  # Use QQQ for technology/growth comparison
  cresearch NVDA MSFT --benchmark QQQ

  # Compare one stock against another stock
  cresearch TSLA --benchmark AAPL --start 2020-01-01

  # Custom risk-free rate
  cresearch AAPL --risk-free-rate 0.04

  # Optional personal margin stress table
  cresearch AAPL --account-equity 100000 --margin-loan 25000

  # Optional AI review layer
  cresearch AAPL --ai-review

  # Every run is archived by default; latest is refreshed automatically
  cresearch AAPL

  # Use your own run id
  cresearch AAPL --run-id test_2023_start
"""
    parser = argparse.ArgumentParser(
        prog="cresearch",
        description="Generate a company research data pack with benchmark comparison, charts, financial metrics, and Markdown report.",
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
    parser.add_argument("--archive", action="store_true", help="Compatibility flag. v2 archives every run and refreshes latest automatically.")
    parser.add_argument("--run-id", default=None, help="Optional archive folder name under reports/TICKER/runs/.")
    parser.add_argument("--account-equity", type=float, default=None, help="Optional account equity for personal margin stress testing.")
    parser.add_argument("--margin-loan", type=float, default=None, help="Optional margin loan balance for personal margin stress testing.")
    parser.add_argument("--ai-review", action="store_true", help="Add optional OpenAI AI Review section to the report.")
    parser.add_argument("--ai-model", default=os.getenv("OPENAI_MODEL", DEFAULT_AI_MODEL), help=f"AI model for --ai-review. Default: OPENAI_MODEL or {DEFAULT_AI_MODEL}.")
    parser.add_argument("--ai-review-depth", choices=["basic", "deep"], default="basic", help="AI review depth. Default: basic.")
    parser.add_argument("--ai-timeout", type=int, default=60, help="OpenAI API timeout in seconds. Default: 60.")
    parser.add_argument("--ai-max-output-tokens", type=int, default=1200, help="Max tokens for AI review output. Default: 1200.")
    parser.add_argument("--no-rich", action="store_true", help="Disable Rich terminal UI and use plain output.")
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    output = Path(args.output)
    ensure_dir(output)

    if terminal_ui is not None:
        terminal_ui.configure(enabled=not args.no_rich)
        terminal_ui.print_app_banner(__version__)

    rows = []
    for symbol in args.symbols:
        try:
            rows.append(
                run_one(
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
                )
            )
        except Exception as exc:
            print(f"\n[ERROR] Failed for {symbol}: {exc}")

    write_cross_ticker_comparison(rows, output, archive=args.archive, run_id=args.run_id)


if __name__ == "__main__":
    main()
