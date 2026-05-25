#!/usr/bin/env python3
"""A-share provider adapter for v5.

The adapter prefers the local AKShare package when present, but it does not
pretend AKShare coverage exists when the dependency is missing.  In that case
it uses the same public Eastmoney endpoints AKShare commonly wraps and records
that fallback explicitly in metadata.  The output remains a real provider
payload, not a mock fixture.
"""

from __future__ import annotations

import gzip
import importlib.util
import json
import os
import signal
import time
from datetime import datetime, timezone
from typing import Any
from urllib.request import Request, urlopen

SCHEMA_VERSION = "v5.0.0"
PROVIDER_VERSION = "v5-a-share-provider-repair"
EASTMONEY_LIMITATIONS = [
    "Public endpoint; field coverage and schema may change",
    "Not a guaranteed official data contract",
    "Validate important values against exchange filings or company reports",
]
REQUEST_HEADERS = {
    "User-Agent": "Mozilla/5.0",
    "Accept-Encoding": "identity",
    "Referer": "https://emweb.securities.eastmoney.com/",
}
AKSHARE_CALL_TIMEOUT_SECONDS = 8


class ProviderCallTimeout(TimeoutError):
    """Raised when an optional package provider call exceeds its budget."""


def _call_with_timeout(label: str, func, timeout_seconds: int = AKSHARE_CALL_TIMEOUT_SECONDS) -> Any:
    if not hasattr(signal, "SIGALRM"):
        return func()

    def _handle_timeout(_signum: int, _frame: Any) -> None:
        raise ProviderCallTimeout(f"{label} timed out after {timeout_seconds}s")

    previous_handler = signal.getsignal(signal.SIGALRM)
    signal.signal(signal.SIGALRM, _handle_timeout)
    previous_timer = signal.setitimer(signal.ITIMER_REAL, timeout_seconds)
    try:
        return func()
    finally:
        signal.setitimer(signal.ITIMER_REAL, previous_timer[0], previous_timer[1])
        signal.signal(signal.SIGALRM, previous_handler)


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _base_payload(
    ticker: str,
    provider: str,
    *,
    source: str | None = None,
    package_used: bool = False,
    provider_adapter: str | None = None,
    provider_limitations: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "ticker": ticker.upper(),
        "market": "CN_A",
        "provider": provider,
        "provider_status": "PASS",
        "fetched_at": _now(),
        "company_profile": {
            "name": ticker.upper(),
            "sector": "",
            "industry": "",
            "description": "",
            "exchange": "",
            "currency": "CNY",
        },
        "price_history": [],
        "income_statement": [],
        "balance_sheet": [],
        "cash_flow": [],
        "valuation_snapshot": {},
        "segments": [],
        "data_coverage": {},
        "missing_fields": [],
        "metadata": {
            "data_quality_warnings": [],
            "source": source or provider,
            "provider_version": PROVIDER_VERSION,
            "package_used": package_used,
            "mock": False,
            "provider_adapter": provider_adapter or provider,
            "provider_limitations": provider_limitations or [],
        },
    }


def _error_payload(ticker: str, provider: str, error_type: str, message: str) -> dict[str, Any]:
    payload = _base_payload(ticker, provider, source="provider failure", provider_adapter=provider)
    payload["provider_status"] = "PROVIDER_ERROR"
    payload["data_coverage"] = {
        "company_profile": False,
        "price_history": False,
        "income_statement": False,
        "balance_sheet": False,
        "cash_flow": False,
    }
    payload["missing_fields"] = [
        "company_profile",
        "price_history",
        "income_statement",
        "balance_sheet",
        "cash_flow",
    ]
    payload["metadata"]["data_quality_warnings"].append(message)
    payload["error"] = {
        "error_type": error_type,
        "error_message": message,
        "stage": "provider_fetch",
    }
    return payload


def _eastmoney_code(ticker: str) -> str:
    ticker = ticker.upper()
    if ticker.endswith(".SH"):
        return f"SH{ticker[:6]}"
    if ticker.endswith(".SZ"):
        return f"SZ{ticker[:6]}"
    raise ValueError(f"unsupported A-share ticker format: {ticker}")


def _secid(ticker: str) -> str:
    ticker = ticker.upper()
    if ticker.endswith(".SH"):
        return f"1.{ticker[:6]}"
    if ticker.endswith(".SZ"):
        return f"0.{ticker[:6]}"
    raise ValueError(f"unsupported A-share ticker format: {ticker}")


def _sina_symbol(ticker: str) -> str:
    ticker = ticker.upper()
    if ticker.endswith(".SH"):
        return f"sh{ticker[:6]}"
    if ticker.endswith(".SZ"):
        return f"sz{ticker[:6]}"
    raise ValueError(f"unsupported A-share ticker format: {ticker}")


def _fetch_json(url: str) -> dict[str, Any]:
    return json.loads(_fetch_text(url))


def _fetch_text(url: str) -> str:
    last_error: Exception | None = None
    for attempt in range(3):
        try:
            req = Request(url, headers={**REQUEST_HEADERS, "Connection": "close"})
            with urlopen(req, timeout=20) as response:
                body = response.read()
                if response.headers.get("Content-Encoding") == "gzip" or body[:2] == b"\x1f\x8b":
                    body = gzip.decompress(body)
            return body.decode("utf-8")
        except Exception as exc:  # pragma: no cover - network retry path
            last_error = exc
            if attempt < 2:
                time.sleep(0.8 * (attempt + 1))
    raise last_error or RuntimeError("provider request failed")


def _fetch_jsonp(url: str) -> dict[str, Any]:
    text = _fetch_text(url)
    start = text.find("(")
    end = text.rfind(")")
    if start == -1 or end == -1 or end <= start:
        raise ValueError("JSONP wrapper not found")
    return json.loads(text[start + 1 : end])


def _ticker_code(ticker: str) -> str:
    return ticker.upper()[:6]


def _num(value: Any) -> float | None:
    if value is None or value == "":
        return None
    try:
        parsed = float(value)
    except (TypeError, ValueError):
        return None
    if parsed != parsed:
        return None
    return parsed


def _add_row(rows: list[dict[str, Any]], period: str, metric: str, value: Any, unit: str) -> None:
    rows.append(
        {
            "period": period[:10] if period else "",
            "metric": metric,
            "value": _num(value),
            "unit": unit,
        }
    )


def _first(items: Any) -> dict[str, Any]:
    if isinstance(items, list) and items:
        first = items[0]
        if isinstance(first, dict):
            return first
    return {}


def _fetch_profile(payload: dict[str, Any], ticker: str) -> None:
    code = _eastmoney_code(ticker)
    survey = _fetch_json(
        f"https://emweb.securities.eastmoney.com/PC_HSF10/CompanySurvey/PageAjax?code={code}"
    )
    business = _fetch_json(
        f"https://emweb.securities.eastmoney.com/PC_HSF10/BusinessAnalysis/PageAjax?code={code}"
    )
    profile = _first(survey.get("jbzl"))
    scope = _first(business.get("zyfw")).get("BUSINESS_SCOPE") or ""
    sector_path = profile.get("EM2016") or ""
    industry = profile.get("INDUSTRYCSRC1") or sector_path
    payload["company_profile"] = {
        "name": profile.get("ORG_NAME") or profile.get("SECURITY_NAME_ABBR") or ticker.upper(),
        "sector": sector_path.split("-")[0] if sector_path else "",
        "industry": industry,
        "description": scope,
        "exchange": profile.get("TRADE_MARKET") or "",
        "currency": "CNY",
    }
    if payload["segments"]:
        return
    for segment in business.get("zygcfx") or []:
        if isinstance(segment, dict):
            payload["segments"].append(
                {
                    "period": str(segment.get("REPORT_DATE") or "")[:10],
                    "name": segment.get("ITEM_NAME") or "",
                    "revenue": _num(segment.get("MAIN_BUSINESS_INCOME")),
                    "cost": _num(segment.get("MAIN_BUSINESS_COST")),
                    "gross_profit": _num(segment.get("MAIN_BUSINESS_RPOFIT")),
                    "unit": "CNY",
                }
            )


def _fetch_profile_akshare(payload: dict[str, Any], ticker: str, ak: Any) -> bool:
    frame = _call_with_timeout(
        "akshare.stock_individual_info_em",
        lambda: ak.stock_individual_info_em(symbol=_ticker_code(ticker)),
    )
    if frame is None or getattr(frame, "empty", True):
        return False
    values = {
        str(row["item"]): row["value"]
        for _, row in frame.iterrows()
        if "item" in frame.columns and "value" in frame.columns
    }
    name = str(values.get("股票简称") or payload["company_profile"].get("name") or ticker.upper())
    industry = str(values.get("行业") or payload["company_profile"].get("industry") or "")
    payload["company_profile"].update(
        {
            "name": name,
            "sector": industry.split("Ⅱ")[0] if industry else payload["company_profile"].get("sector", ""),
            "industry": industry,
            "currency": "CNY",
        }
    )
    market_cap = _num(values.get("总市值"))
    if market_cap is not None:
        payload["valuation_snapshot"]["market_cap"] = market_cap
    return True


def _fetch_segments_akshare(payload: dict[str, Any], ticker: str, ak: Any) -> bool:
    frame = _call_with_timeout(
        "akshare.stock_zygc_em",
        lambda: ak.stock_zygc_em(symbol=_eastmoney_code(ticker)),
    )
    if frame is None or getattr(frame, "empty", True):
        return False
    for _, row in frame.head(40).iterrows():
        payload["segments"].append(
            {
                "period": str(row.get("报告日期") or "")[:10],
                "name": row.get("主营构成") or "",
                "revenue": _num(row.get("主营收入")),
                "cost": _num(row.get("主营成本")),
                "gross_profit": _num(row.get("主营利润")),
                "unit": "CNY",
            }
        )
    return True


def _fetch_price_history_akshare(payload: dict[str, Any], ticker: str, ak: Any) -> bool:
    frame = ak.stock_zh_a_hist(
        symbol=_ticker_code(ticker),
        period="daily",
        start_date="20200101",
        end_date="20500101",
        adjust="qfq",
    )
    if frame is None or getattr(frame, "empty", True):
        return False
    date_col = "日期"
    close_col = "收盘"
    volume_col = "成交量"
    for _, row in frame.tail(260).iterrows():
        payload["price_history"].append(
            {
                "date": str(row.get(date_col) or "")[:10],
                "close": _num(row.get(close_col)),
                "volume": _num(row.get(volume_col)),
            }
        )
    return True


def _fetch_price_history(payload: dict[str, Any], ticker: str) -> None:
    _fetch_price_history_sina(payload, ticker)
    if payload["price_history"]:
        return
    url = (
        "https://push2his.eastmoney.com/api/qt/stock/kline/get?"
        f"secid={_secid(ticker)}&fields1=f1,f2,f3,f4,f5,f6&"
        "fields2=f51,f52,f53,f56&"
        "klt=101&fqt=1&beg=20200101&end=20500101"
    )
    try:
        data = _fetch_json(url).get("data") or {}
    except Exception:
        try:
            jsonp_url = (
                "https://push2his.eastmoney.com/api/qt/stock/kline/get?"
                f"cb=jQuery&secid={_secid(ticker)}&ut=fa5fd1943c7b386f172d6893dbfba10b&"
                "fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f56&"
                "klt=101&fqt=1&beg=20200101&end=20500101"
            )
            data = _fetch_jsonp(jsonp_url).get("data") or {}
        except Exception:
            _fetch_price_history_sina(payload, ticker)
            return
    for line in (data.get("klines") or [])[-260:]:
        fields = str(line).split(",")
        if len(fields) < 4:
            continue
        payload["price_history"].append(
            {
                "date": fields[0],
                "close": _num(fields[2]),
                "volume": _num(fields[3]),
            }
        )


def _fetch_price_history_sina(payload: dict[str, Any], ticker: str) -> None:
    url = (
        "https://quotes.sina.cn/cn/api/json_v2.php/CN_MarketData.getKLineData?"
        f"symbol={_sina_symbol(ticker)}&scale=240&ma=no&datalen=260"
    )
    rows = _fetch_json(url)
    if not isinstance(rows, list):
        return
    for row in rows[-260:]:
        if not isinstance(row, dict):
            continue
        payload["price_history"].append(
            {
                "date": str(row.get("day") or "")[:10],
                "close": _num(row.get("close")),
                "volume": _num(row.get("volume")),
            }
        )
    payload["metadata"]["data_quality_warnings"].append(
        "price_history_from_sina_public_endpoint; Eastmoney price endpoint is unstable in this environment"
    )


def _fetch_financials(payload: dict[str, Any], ticker: str) -> None:
    code = _eastmoney_code(ticker)
    data = _fetch_json(
        f"https://emweb.securities.eastmoney.com/PC_HSF10/NewFinanceAnalysis/ZYZBAjaxNew?type=0&code={code}"
    )
    rows = data.get("data") or []
    for row in rows[:8]:
        if not isinstance(row, dict):
            continue
        period = str(row.get("REPORT_DATE") or row.get("REPORT_DATE_NAME") or "")
        currency = row.get("CURRENCY") or "CNY"
        _add_row(payload["income_statement"], period, "营业收入", row.get("TOTALOPERATEREVE"), currency)
        _add_row(payload["income_statement"], period, "毛利", row.get("MLR"), currency)
        _add_row(payload["income_statement"], period, "归母净利润", row.get("PARENTNETPROFIT"), currency)
        _add_row(payload["income_statement"], period, "扣非归母净利润", row.get("KCFJCXSYJLR"), currency)
        _add_row(payload["income_statement"], period, "毛利率", row.get("XSMLL"), "%")
        _add_row(payload["income_statement"], period, "净利率", row.get("XSJLL"), "%")
        _add_row(payload["income_statement"], period, "ROE", row.get("ROEJQ"), "%")
        _add_row(payload["income_statement"], period, "ROA/总资产净利率", row.get("ZZCJLL"), "%")

        _add_row(payload["balance_sheet"], period, "负债合计", row.get("LIABILITY"), currency)
        _add_row(payload["balance_sheet"], period, "资产负债率", row.get("ZCFZL"), "%")
        _add_row(payload["balance_sheet"], period, "流动比率", row.get("LD"), "x")
        _add_row(payload["balance_sheet"], period, "速动比率", row.get("SD"), "x")
        _add_row(payload["balance_sheet"], period, "现金比率", row.get("CASH_RATIO"), "x")
        _add_row(payload["balance_sheet"], period, "有息负债率", row.get("INTEREST_DEBT_RATIO"), "%")
        _add_row(payload["balance_sheet"], period, "存货周转率", row.get("CHZZL"), "x")
        _add_row(payload["balance_sheet"], period, "存货周转天数", row.get("CHZZTS"), "days")
        _add_row(payload["balance_sheet"], period, "应收账款周转率", row.get("YSZKZZL"), "x")
        _add_row(payload["balance_sheet"], period, "应收账款周转天数", row.get("YSZKZZTS"), "days")
        _add_row(payload["balance_sheet"], period, "贷款总额", row.get("GROSSLOANS"), currency)
        _add_row(payload["balance_sheet"], period, "贷款及垫款", row.get("LOAN_ADVANCES"), currency)
        _add_row(payload["balance_sheet"], period, "存款总额", row.get("TOTALDEPOSITS"), currency)
        _add_row(payload["balance_sheet"], period, "不良贷款率", row.get("NONPERLOAN"), "%")
        _add_row(payload["balance_sheet"], period, "不良贷款余额", row.get("NON_PERFORMING_LOAN"), currency)
        _add_row(payload["balance_sheet"], period, "拨备覆盖率", row.get("BLDKBBL"), "%")
        _add_row(payload["balance_sheet"], period, "资本充足率", row.get("NEWCAPITALADER"), "%")
        _add_row(payload["balance_sheet"], period, "一级资本充足率", row.get("FIRST_ADEQUACY_RATIO"), "%")
        _add_row(payload["balance_sheet"], period, "净息差", row.get("NET_INTEREST_MARGIN"), "%")

        _add_row(payload["cash_flow"], period, "每股经营现金流", row.get("MGJYXJJE"), "CNY/share")
        _add_row(payload["cash_flow"], period, "经营现金流/营业收入", row.get("JYXJLYYSR"), "x")
        _add_row(payload["cash_flow"], period, "现金流量比率", row.get("XJLLB"), "x")
        _add_row(payload["cash_flow"], period, "FCFF估算(后向)", row.get("FCFF_BACK"), currency)

    latest = _first(rows)
    if latest:
        payload["valuation_snapshot"] = {
            "roe": _num(latest.get("ROEJQ")),
            "roa": _num(latest.get("ZZCJLL")),
            "net_interest_margin": _num(latest.get("NET_INTEREST_MARGIN")),
            "capital_adequacy_ratio": _num(latest.get("NEWCAPITALADER")),
            "npl_ratio": _num(latest.get("NONPERLOAN")),
            "asset_liability_ratio": _num(latest.get("ZCFZL")),
        }


def _metric_present(rows: list[dict[str, Any]], needles: list[str]) -> bool:
    for row in rows:
        metric = str(row.get("metric") or "").lower()
        if row.get("value") is not None and any(needle.lower() in metric for needle in needles):
            return True
    return False


def _finalize_coverage(payload: dict[str, Any]) -> None:
    profile_text = " ".join(
        str(payload["company_profile"].get(key) or "")
        for key in ("name", "sector", "industry", "description")
    )
    is_bank = any(word in profile_text.lower() for word in ["bank", "银行", "货币金融"])
    coverage = {
        "company_profile": bool(
            payload["company_profile"].get("name")
            and payload["company_profile"].get("description")
        ),
        "price_history": any(p.get("close") is not None for p in payload["price_history"]),
        "income_statement": any(r.get("value") is not None for r in payload["income_statement"]),
        "balance_sheet": any(r.get("value") is not None for r in payload["balance_sheet"]),
        "cash_flow": any(r.get("value") is not None for r in payload["cash_flow"]),
        "currency": payload["company_profile"].get("currency") in {"CNY", "RMB"},
        "market": payload.get("market") == "CN_A",
        "营业收入": _metric_present(payload["income_statement"], ["营业收入"]),
        "归母净利润": _metric_present(payload["income_statement"], ["归母净利润"]),
        "经营现金流": _metric_present(payload["cash_flow"], ["经营现金"]),
        "毛利率": _metric_present(payload["income_statement"], ["毛利率"]),
        "净利率": _metric_present(payload["income_statement"], ["净利率"]),
        "ROE": _metric_present(payload["income_statement"], ["ROE"]),
        "NIM": _metric_present(payload["balance_sheet"], ["净息差"]),
        "不良贷款": _metric_present(payload["balance_sheet"], ["不良贷款"]),
        "资本充足率": _metric_present(payload["balance_sheet"], ["资本充足率"]),
        "存款": _metric_present(payload["balance_sheet"], ["存款总额"]),
        "贷款": _metric_present(payload["balance_sheet"], ["贷款"]),
        "存货": _metric_present(payload["balance_sheet"], ["存货"]),
        "应收账款": _metric_present(payload["balance_sheet"], ["应收账款"]),
        "现金比率": _metric_present(payload["balance_sheet"], ["现金比率"]),
        "货币资金余额": _metric_present(payload["balance_sheet"], ["货币资金"]),
        "分红": False,
    }
    minimum = ["company_profile", "price_history", "income_statement", "balance_sheet", "cash_flow", "currency", "market"]
    if is_bank:
        relevant = minimum + ["营业收入", "归母净利润", "经营现金流", "ROE", "ROA", "NIM", "不良贷款", "资本充足率", "存款", "贷款"]
    else:
        relevant = minimum + [
            "营业收入",
            "归母净利润",
            "经营现金流",
            "毛利率",
            "净利率",
            "ROE",
            "存货",
            "应收账款",
            "货币资金余额",
            "分红",
        ]
    coverage["ROA"] = _metric_present(payload["income_statement"], ["ROA", "总资产净利率"])
    missing = [field for field in relevant if not coverage.get(field)]
    payload["data_coverage"] = coverage
    payload["missing_fields"] = missing
    required = ["company_profile", "price_history", "income_statement", "balance_sheet", "cash_flow"]
    if any(not coverage[field] for field in required):
        payload["error"] = {
            "error_type": "ProviderDataGap",
            "error_message": "A-share provider returned incomplete minimum required fields.",
            "stage": "provider_fetch",
        }
        payload["provider_status"] = "PROVIDER_ERROR"
    else:
        payload["provider_status"] = "PASS"


def fetch(ticker: str) -> dict[str, Any]:
    ticker = ticker.upper()
    if os.environ.get("RESEARCH_USE_AKSHARE_PACKAGE") == "1" and importlib.util.find_spec("akshare") is not None:
        payload = _fetch_with_akshare_package(ticker)
        if payload.get("error") is None:
            return payload
    return _fetch_with_eastmoney_public(ticker)


def _fetch_with_eastmoney_public(ticker: str) -> dict[str, Any]:
    provider = "eastmoney_public"
    last_error: Exception | None = None
    for attempt in range(1):
        payload = _base_payload(
            ticker,
            provider,
            source="Eastmoney public endpoint",
            package_used=False,
            provider_adapter="akshare_compatible_fallback",
            provider_limitations=EASTMONEY_LIMITATIONS,
        )
        payload["metadata"]["data_quality_warnings"].append(
            "used_eastmoney_public_fallback; no mock data was generated"
        )
        if importlib.util.find_spec("akshare") is not None:
            payload["metadata"]["data_quality_warnings"].append(
                "akshare_package_installed_but_not_used; RESEARCH_USE_AKSHARE_PACKAGE=1 enables package endpoint attempts"
            )
        try:
            _fetch_profile(payload, ticker)
            _fetch_price_history(payload, ticker)
            _fetch_financials(payload, ticker)
            _finalize_coverage(payload)
            if payload["missing_fields"]:
                payload["metadata"]["data_quality_warnings"].append(
                    "provider_optional_fields_missing: see missing_fields for non-core or sector-specific A-share fields not returned"
                )
            if attempt:
                payload["metadata"]["data_quality_warnings"].append(
                    f"provider_fetch_retry_succeeded_after_{attempt}_failed_attempts"
                )
            return payload
        except Exception as exc:
            last_error = exc
            if attempt < 2:
                time.sleep(1.0 * (attempt + 1))
    return _error_payload(
        ticker,
        provider,
        type(last_error).__name__ if last_error else "ProviderError",
        str(last_error) if last_error else "A-share provider failed",
    )


def _fetch_with_akshare_package(ticker: str) -> dict[str, Any]:
    provider = "akshare_package"
    last_error: Exception | None = None
    for attempt in range(3):
        payload = _base_payload(
            ticker,
            provider,
            source="AKShare package",
            package_used=True,
            provider_adapter="akshare_package",
            provider_limitations=[
                "AKShare package is installed and called",
                "Selected fields may fall back to Eastmoney public endpoints if an AKShare endpoint is unavailable",
                "Validate important values against exchange filings or company reports",
            ],
        )
        try:
            ak = __import__("akshare")
            package_calls = []
            if _fetch_profile_akshare(payload, ticker, ak):
                package_calls.append("stock_individual_info_em")
            try:
                if _fetch_segments_akshare(payload, ticker, ak):
                    package_calls.append("stock_zygc_em")
            except Exception as exc:
                payload["metadata"]["data_quality_warnings"].append(
                    f"akshare_segment_endpoint_failed; used Eastmoney segment fallback if available: {type(exc).__name__}: {exc}"
                )
            payload["metadata"]["data_quality_warnings"].append(
                "akshare_price_endpoint_skipped; used Eastmoney price fallback to avoid provider timeout"
            )
            if not package_calls:
                raise RuntimeError("AKShare package imported but no package endpoint returned usable data")
            payload["metadata"]["provider_limitations"].append(
                "AKShare endpoints called: " + ", ".join(package_calls)
            )
            _fetch_profile(payload, ticker)
            if not payload["price_history"]:
                _fetch_price_history(payload, ticker)
            _fetch_financials(payload, ticker)
            _finalize_coverage(payload)
            if payload["missing_fields"]:
                payload["metadata"]["data_quality_warnings"].append(
                    "provider_optional_fields_missing: see missing_fields for non-core or sector-specific A-share fields not returned"
                )
            if attempt:
                payload["metadata"]["data_quality_warnings"].append(
                    f"provider_fetch_retry_succeeded_after_{attempt}_failed_attempts"
                )
            return payload
        except Exception as exc:
            last_error = exc
            if attempt < 2:
                time.sleep(1.0 * (attempt + 1))
    payload = _error_payload(
        ticker,
        provider,
        type(last_error).__name__ if last_error else "ProviderError",
        str(last_error) if last_error else "AKShare package provider failed",
    )
    payload["metadata"]["package_used"] = True
    payload["metadata"]["provider_adapter"] = "akshare_package"
    payload["metadata"]["source"] = "AKShare package"
    return payload
