#!/usr/bin/env python3
"""Unified v5 provider bridge.

The Rust engine calls this script and reads a stable provider_payload.json.
The script never silently fails: it writes either a valid payload or a payload
with an explicit error object.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def _base_payload(ticker: str, market: str, provider: str) -> dict[str, Any]:
    return {
        "ticker": ticker.upper(),
        "market": market.upper(),
        "provider": provider,
        "fetched_at": datetime.now(timezone.utc).isoformat(),
        "company_profile": {
            "name": ticker.upper(),
            "sector": "",
            "industry": "",
            "description": "",
            "exchange": "",
            "currency": "",
        },
        "price_history": [],
        "income_statement": [],
        "balance_sheet": [],
        "cash_flow": [],
        "valuation_snapshot": {},
        "segments": [],
        "metadata": {
            "data_quality_warnings": [],
            "source": provider,
            "provider_version": "v5-provider-bridge",
        },
    }


def _error_payload(ticker: str, market: str, provider: str, error_type: str, message: str) -> dict[str, Any]:
    payload = _base_payload(ticker, market, provider)
    payload["metadata"]["data_quality_warnings"].append(message)
    payload["error"] = {
        "error_type": error_type,
        "error_message": message,
        "stage": "provider_fetch",
    }
    return payload


def _statement_rows(frame: Any, unit: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    if frame is None or getattr(frame, "empty", True):
        return rows
    # yfinance financial frames: index metrics, columns periods.
    for metric in list(frame.index)[:40]:
        for period in list(frame.columns)[:5]:
            value = frame.loc[metric, period]
            try:
                if value != value:
                    parsed = None
                else:
                    parsed = float(value)
            except Exception:
                parsed = None
            rows.append({"period": str(period)[:10], "metric": str(metric), "value": parsed, "unit": unit})
    return rows


def fetch_yfinance(ticker: str, market: str, provider: str) -> dict[str, Any]:
    try:
        import yfinance as yf
    except Exception as exc:  # pragma: no cover - depends on environment
        return _error_payload(ticker, market, provider, "ImportError", f"yfinance unavailable: {exc}")

    try:
        tk = yf.Ticker(ticker)
        info = {}
        try:
            info = tk.get_info() or {}
        except Exception:
            info = {}
        payload = _base_payload(ticker, market, "yfinance" if provider == "auto" else provider)
        payload["company_profile"] = {
            "name": info.get("longName") or info.get("shortName") or ticker.upper(),
            "sector": info.get("sector") or "",
            "industry": info.get("industry") or "",
            "description": info.get("longBusinessSummary") or "",
            "exchange": info.get("exchange") or "",
            "currency": info.get("currency") or "USD",
        }
        hist = tk.history(period="5y", auto_adjust=False)
        if hist is not None and not hist.empty:
            for date, row in hist.tail(260).iterrows():
                payload["price_history"].append({
                    "date": str(date.date()),
                    "close": None if row.get("Close") != row.get("Close") else float(row.get("Close")),
                    "volume": None if row.get("Volume") != row.get("Volume") else float(row.get("Volume")),
                })
        payload["income_statement"] = _statement_rows(getattr(tk, "financials", None), payload["company_profile"]["currency"])
        payload["balance_sheet"] = _statement_rows(getattr(tk, "balance_sheet", None), payload["company_profile"]["currency"])
        payload["cash_flow"] = _statement_rows(getattr(tk, "cashflow", None), payload["company_profile"]["currency"])
        payload["valuation_snapshot"] = {
            "marketCap": info.get("marketCap"),
            "trailingPE": info.get("trailingPE"),
            "forwardPE": info.get("forwardPE"),
            "priceToSalesTrailing12Months": info.get("priceToSalesTrailing12Months"),
            "priceToBook": info.get("priceToBook"),
            "enterpriseToRevenue": info.get("enterpriseToRevenue"),
            "enterpriseToEbitda": info.get("enterpriseToEbitda"),
        }
        if not payload["income_statement"]:
            payload["metadata"]["data_quality_warnings"].append("income_statement_missing")
        if not payload["cash_flow"]:
            payload["metadata"]["data_quality_warnings"].append("cash_flow_missing")
        return payload
    except Exception as exc:
        return _error_payload(ticker, market, provider, type(exc).__name__, str(exc))


def fetch_cn_placeholder(ticker: str, market: str, provider: str) -> dict[str, Any]:
    try:
        if provider in {"auto", "akshare"}:
            import akshare as ak  # noqa: F401
            # Keep this conservative for v5 foundation. The provider is detected,
            # but full A-share normalization is a follow-up adapter task.
            payload = _base_payload(ticker, market, "akshare")
            payload["company_profile"]["name"] = ticker.upper()
            payload["company_profile"]["currency"] = "CNY"
            payload["metadata"]["data_quality_warnings"].append("akshare_available_but_v5_adapter_is_screening_only")
            return payload
    except Exception:
        pass
    return _error_payload(
        ticker,
        market,
        provider,
        "ProviderUnavailable",
        "A-share provider adapter is unavailable or screening-only in this environment.",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ticker", required=True)
    parser.add_argument("--market", default="US")
    parser.add_argument("--provider", default="auto")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    ticker = args.ticker.upper()
    market = args.market.upper()
    if market in {"CN", "CN_A", "A"} or ticker.endswith((".SH", ".SZ")):
        payload = fetch_cn_placeholder(ticker, "CN_A", args.provider)
    else:
        payload = fetch_yfinance(ticker, "US", args.provider)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()

