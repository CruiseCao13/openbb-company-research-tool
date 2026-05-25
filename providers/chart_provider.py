#!/usr/bin/env python3
"""Generate v5 report charts from locked provider_payload.json.

This helper is intentionally small and deterministic. It does not fetch data.
If a chart lacks sufficient data, it writes a data-gap card instead of drawing
an empty or misleading chart.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt


FIGURES = [
    ("Figure_01_price_vs_benchmark.png", "Price / Benchmark Performance", "price"),
    ("Figure_02_drawdown.png", "Drawdown / Risk Path", "drawdown"),
    ("Figure_03_financial_trend.png", "Financial Trend", "financial"),
    ("Figure_04_money_flow.png", "Money Flow / Cash Flow Bridge", "money_flow"),
    ("Figure_05_valuation_frame.png", "Valuation Frame", "valuation"),
]

THEME = {
    "target": "#2563eb",
    "risk": "#dc2626",
    "money": "#059669",
    "valuation": "#7c3aed",
    "grid": "#94a3b8",
    "text": "#0f172a",
}


def _rows(payload: dict[str, Any], section: str, needles: list[str]) -> list[tuple[str, float]]:
    out: list[tuple[str, float]] = []
    for row in payload.get(section, []):
        metric = str(row.get("metric", "")).lower()
        if any(n in metric for n in needles):
            value = row.get("value")
            if isinstance(value, (int, float)):
                out.append((str(row.get("period", "")), float(value)))
    return out[:8]


def _write_gap(path: Path, figure_no: int, title: str, reason: str) -> dict[str, Any]:
    gap_path = path.with_suffix(".md")
    gap_path.write_text(
        f"# Figure {figure_no}. {title}\n\n"
        f"Status: Data gap\n\n"
        f"Reason: {reason}\n\n"
        "Source: provider_payload.json\n",
        encoding="utf-8",
    )
    return {
        "figure": figure_no,
        "title": title,
        "file": gap_path.name,
        "status": "DATA_GAP",
        "source": "provider_payload.json",
        "reason": reason,
    }


def _save_line_chart(path: Path, title: str, subtitle: str, x: list[str], y: list[float], ylabel: str, color: str = THEME["target"]) -> None:
    fig, ax = plt.subplots(figsize=(10, 5.625), dpi=160)
    ax.plot(x, y, color=color, linewidth=2.4, marker="o", markersize=3.8, label=title)
    ax.set_title(title, fontsize=15, weight="bold", color=THEME["text"])
    ax.text(0, 1.025, subtitle, transform=ax.transAxes, fontsize=10, color="#475569")
    ax.set_xlabel("Period")
    ax.set_ylabel(ylabel)
    ax.grid(True, alpha=0.22, color=THEME["grid"])
    ax.legend(loc="best")
    fig.tight_layout()
    fig.savefig(path)
    plt.close(fig)


def generate(payload_path: Path, chart_dir: Path) -> list[dict[str, Any]]:
    payload = json.loads(payload_path.read_text(encoding="utf-8"))
    chart_dir.mkdir(parents=True, exist_ok=True)
    ticker = payload.get("ticker", "")
    provider = payload.get("provider", "")
    currency = payload.get("company_profile", {}).get("currency") or "USD"
    manifest: list[dict[str, Any]] = []

    prices = payload.get("price_history", [])
    price_file = chart_dir / FIGURES[0][0]
    if len(prices) >= 5:
        tail = prices[-120:]
        x = [p["date"] for p in tail]
        y = [float(p["close"]) for p in tail if p.get("close") is not None]
        if len(y) == len(x):
            _save_line_chart(
                price_file,
                f"Figure 1. {ticker} price path",
                f"{ticker} | provider: {provider} | unit: {currency}",
                x,
                y,
                f"Price ({currency})",
            )
            manifest.append({"figure": 1, "title": "Price / Benchmark Performance", "file": price_file.name, "status": "PASS", "source": "provider_payload.json", "data_used": ["price_history.close"], "research_question": "How did the stock's price path behave over the available period?", "why_selected": "Price path is the starting evidence for opportunity cost and volatility context."})
        else:
            manifest.append(_write_gap(price_file, 1, "Price / Benchmark Performance", "price history has missing close values"))
    else:
        manifest.append(_write_gap(price_file, 1, "Price / Benchmark Performance", "not enough price history"))

    drawdown_file = chart_dir / FIGURES[1][0]
    if len(prices) >= 5 and manifest[0]["status"] == "PASS":
        closes = [float(p["close"]) for p in prices[-120:] if p.get("close") is not None]
        dates = [p["date"] for p in prices[-120:] if p.get("close") is not None]
        peak = closes[0]
        dd = []
        for c in closes:
            peak = max(peak, c)
            dd.append((c / peak - 1.0) * 100.0)
        _save_line_chart(drawdown_file, f"Figure 2. {ticker} drawdown path", f"{ticker} | provider: {provider} | unit: %", dates, dd, "Drawdown (%)", THEME["risk"])
        manifest.append({"figure": 2, "title": "Drawdown / Risk Path", "file": drawdown_file.name, "status": "PASS", "source": "provider_payload.json", "data_used": ["price_history.close"], "research_question": "How large were the peak-to-trough losses?", "why_selected": "Drawdown makes the risk path visible instead of hiding it behind return numbers."})
    else:
        manifest.append(_write_gap(drawdown_file, 2, "Drawdown / Risk Path", "not enough valid price history"))

    fin_file = chart_dir / FIGURES[2][0]
    revenue = _rows(payload, "income_statement", ["revenue", "营业收入"])
    if revenue:
        x, y = zip(*revenue[:5])
        _save_line_chart(fin_file, f"Figure 3. {ticker} revenue trend", f"{ticker} | source: provider_payload.json | unit: {currency}", list(x), list(y), f"Revenue ({currency})")
        manifest.append({"figure": 3, "title": "Financial Trend", "file": fin_file.name, "status": "PASS", "source": "provider_payload.json", "data_used": ["income_statement.revenue"], "research_question": "Is the company showing financial progression?", "why_selected": "Revenue trend is a first-pass anchor before interpreting margins and cash flow."})
    else:
        manifest.append(_write_gap(fin_file, 3, "Financial Trend", "revenue trend not available"))

    money_file = chart_dir / FIGURES[3][0]
    cash = _rows(
        payload,
        "cash_flow",
        [
            "operating cash flow",
            "cash from operations",
            "capital expenditure",
            "capex",
            "经营现金流",
            "经营活动产生的现金流量净额",
            "资本开支",
            "购建固定资产",
        ],
    )
    if cash:
        x, y = zip(*cash[:6])
        _save_line_chart(money_file, f"Figure 4. {ticker} money-flow signals", f"{ticker} | source: provider_payload.json | unit: {currency}", list(x), list(y), f"Cash flow ({currency})", THEME["money"])
        manifest.append({"figure": 4, "title": "Money Flow / Cash Flow Bridge", "file": money_file.name, "status": "PASS", "source": "provider_payload.json", "data_used": ["cash_flow.operating_cash_flow", "cash_flow.capex"], "research_question": "Where does cash come from and where is it consumed?", "why_selected": "Money flow is central to judging whether the business funds itself or consumes external capital."})
    else:
        manifest.append(_write_gap(money_file, 4, "Money Flow / Cash Flow Bridge", "cash-flow bridge data not available"))

    val_file = chart_dir / FIGURES[4][0]
    valuation = payload.get("valuation_snapshot", {})
    vals = [(k, v) for k, v in valuation.items() if isinstance(v, (int, float)) and v and v > 0][:5]
    if vals:
        fig, ax = plt.subplots(figsize=(10, 5.625), dpi=160)
        ax.bar([k for k, _ in vals], [float(v) for _, v in vals], color=THEME["valuation"], label="valuation")
        ax.set_title(f"Figure 5. {ticker} valuation frame", fontsize=13, weight="bold")
        ax.text(0, 1.02, f"{ticker} | source: provider_payload.json | unit: x or currency as reported", transform=ax.transAxes, fontsize=9, color="#475569")
        ax.set_ylabel("Reported value")
        ax.legend(loc="best")
        fig.tight_layout()
        fig.savefig(val_file)
        plt.close(fig)
        manifest.append({"figure": 5, "title": "Valuation Frame", "file": val_file.name, "status": "PASS", "source": "provider_payload.json", "data_used": ["valuation_snapshot"], "research_question": "Which valuation lens is available and meaningful?", "why_selected": "Valuation charts should frame expectations, not imply a target price."})
    else:
        manifest.append(_write_gap(val_file, 5, "Valuation Frame", "valuation multiples are not available or not meaningful"))

    (chart_dir / "chart_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--payload", required=True)
    parser.add_argument("--out-dir", required=True)
    args = parser.parse_args()
    generate(Path(args.payload), Path(args.out_dir))


if __name__ == "__main__":
    main()
