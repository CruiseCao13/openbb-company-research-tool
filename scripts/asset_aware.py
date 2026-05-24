"""Asset-aware research routing and v4.3 report helpers.

The functions in this module keep the research frame separate from market-data
calculation. Deterministic data stays locked; interpretation blocks can be
patched when the asset profile and report wording do not match.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from datetime import datetime
import json
import math
from pathlib import Path
import re
import shutil
from typing import Any

import pandas as pd


STATUS_PASS = "PASS"
STATUS_WARNING = "WARNING"
STATUS_FAIL = "FAIL"
STATUS_WARNING_DEGRADED = "WARNING_DEGRADED"
STATUS_UNVERIFIED = "UNVERIFIED"


def _safe_float(value: Any) -> float:
    try:
        if value is None or pd.isna(value):
            return float("nan")
        return float(value)
    except Exception:
        return float("nan")


def _metric(df: pd.DataFrame, metric: str, column: str = "Value") -> float:
    try:
        row = df[df["Metric"] == metric]
        if row.empty:
            return float("nan")
        return _safe_float(row.iloc[0][column])
    except Exception:
        return float("nan")


def _valuation(df: pd.DataFrame, metric: str) -> float:
    return _metric(df, metric, "Value")


def _fmt_pct(value: float) -> str:
    return "not applicable from current data" if math.isnan(value) else f"{value:.2%}"


def _fmt_x(value: float) -> str:
    if math.isnan(value):
        return "not applicable from current data"
    if value <= 0:
        return "not applicable / profitability not established"
    return f"{value:.2f}x"


def _fmt_num(value: float) -> str:
    return "not applicable from current data" if math.isnan(value) else f"{value:.2f}"


@dataclass
class AssetProfile:
    lifecycle_profile: str
    primary_profile: str
    secondary_profile: str
    profitability_status: str
    sector_type: str
    sector_confidence: str
    valuation_method_fit: str
    cash_flow_profile: str
    capital_intensity: str
    dilution_risk: str
    cyclicality_risk: str
    financial_company_flag: bool
    dominant_metric_set: list[str]
    invalid_metric_set: list[str]
    data_deficit_flags: list[str]
    research_stance_anchor: str
    report_thesis_spine: str
    thesis_spine_confidence: str
    framework_coverage_level: str
    fallback_used_count: int = 0
    fallback_used_sections: list[str] = field(default_factory=list)
    business_model_clues: list[str] = field(default_factory=list)
    suggested_framework_extension: str = ""

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def build_asset_profile(
    info: dict[str, Any],
    fundamental_summary: pd.DataFrame,
    valuation_snapshot: pd.DataFrame,
    trends: pd.DataFrame,
    ruin_risk: pd.DataFrame,
) -> AssetProfile:
    sector = str(info.get("sector") or "").lower()
    industry = str(info.get("industry") or "").lower()
    summary = str(info.get("longBusinessSummary") or "").lower()
    short_name = str(info.get("shortName") or "").lower()
    long_name = str(info.get("longName") or "").lower()
    quote_type = str(info.get("quoteType") or "").upper()
    clues_text = " ".join([sector, industry, summary, short_name, long_name])

    revenue_cagr = _metric(fundamental_summary, "Revenue CAGR")
    latest_growth = _metric(fundamental_summary, "Revenue Growth Latest")
    gross_margin = _metric(fundamental_summary, "Gross Margin Latest")
    operating_margin = _metric(fundamental_summary, "Operating Margin Latest")
    fcf_margin = _metric(fundamental_summary, "FCF Margin Latest")
    positive_net_income_years = _metric(fundamental_summary, "Positive Net Income Years")
    positive_fcf_years = _metric(fundamental_summary, "Positive FCF Years")
    pe = _valuation(valuation_snapshot, "trailingPE")
    ps = _valuation(valuation_snapshot, "priceToSalesTrailing12Months")
    ev_revenue = _valuation(valuation_snapshot, "enterpriseToRevenue")
    cash_runway = _metric(ruin_risk, "Cash Runway Years")
    latest_revenue = _metric(trends, "Revenue", "Latest") if not trends.empty else float("nan")
    latest_capex = _metric(trends, "Capital Expenditure", "Latest") if not trends.empty else float("nan")
    capex_intensity = abs(latest_capex) / latest_revenue if not math.isnan(latest_capex) and latest_revenue and latest_revenue > 0 else float("nan")

    insurance_like = any(word in clues_text for word in ["insurance", "reinsurance", "underwriting", "premiums", "claims", "catastrophe"])
    reit_like = "reit" in clues_text or "real estate investment trust" in clues_text
    utility_like = any(word in clues_text for word in ["utility", "utilities", "electric utility", "regulated utility", "rate base", "power generation", "water utility"])
    transport_like = any(word in clues_text for word in ["shipping", "airline", "freight", "logistics", "transport", "fleet", "load factor", "cruise"])
    consumer_retail_terms = ["retail", "restaurant", "same-store", "same store", "store", "apparel", "coffee", "quick service", "home improvement"]
    consumer_retail_like = any(word in clues_text for word in consumer_retail_terms)
    financial = any(word in sector for word in ["financial"]) or any(word in industry for word in ["bank", "broker", "asset management"]) or (insurance_like and "financial" in sector)
    fund_like = quote_type in {"ETF", "FUND", "MUTUALFUND", "INDEX"}
    speculative_signal = (
        (not math.isnan(revenue_cagr) and revenue_cagr >= 0.15)
        and (math.isnan(fcf_margin) or fcf_margin <= 0 or math.isnan(operating_margin) or operating_margin <= 0)
    ) or (
        (not math.isnan(latest_growth) and latest_growth >= 0.20)
        and (math.isnan(fcf_margin) or fcf_margin <= 0)
    )
    mature_signal = (
        not math.isnan(operating_margin)
        and operating_margin > 0.15
        and not math.isnan(fcf_margin)
        and fcf_margin > 0.08
        and positive_net_income_years >= 3
        and positive_fcf_years >= 3
    )
    hybrid_signal = (
        not math.isnan(revenue_cagr)
        and revenue_cagr >= 0.15
        and not math.isnan(operating_margin)
        and operating_margin > 0.12
        and not math.isnan(fcf_margin)
        and fcf_margin > 0.05
    )
    cyclical_signal = any(word in clues_text for word in ["oil", "gas", "mining", "steel", "commodity", "metals", "agriculture", "construction equipment"])
    biotech_like = any(word in clues_text for word in ["biotechnology", "clinical", "therapeutic", "drug candidate", "pharmaceutical"])
    aerospace_like = any(word in clues_text for word in ["space", "launch", "aerospace", "satellite", "defense"])
    semiconductor_core = any(word in industry for word in ["semiconductor", "chip"]) or any(word in sector for word in ["semiconductor"])
    semiconductor_clues = [
        "semiconductor",
        "chip",
        "processor",
        "foundry",
        "manufacturing",
        "wafer",
        "data center",
        "client computing",
        "accelerator",
        "integrated device",
        "nvidia",
    ]
    semiconductor_hit_count = sum(1 for word in semiconductor_clues if word in clues_text)
    semiconductor_like = semiconductor_core or semiconductor_hit_count >= 3 or "nvidia" in clues_text
    ai_semiconductor_terms = [
        "artificial intelligence",
        " ai ",
        "gpu",
        "graphics processing",
        "accelerated computing",
        "accelerator",
        "cuda",
        "data center",
        "datacenter",
        "hyperscaler",
        "networking",
        "nvidia",
    ]
    ai_semiconductor_like = semiconductor_like and any(term in f" {clues_text} " for term in ai_semiconductor_terms)
    semiconductor_turnaround_signal = semiconductor_like and (
        (not math.isnan(capex_intensity) and capex_intensity >= 0.12)
        or (not math.isnan(fcf_margin) and fcf_margin < 0.08)
        or (not math.isnan(operating_margin) and operating_margin < 0.15)
        or (not math.isnan(gross_margin) and gross_margin < 0.45)
        or (not math.isnan(revenue_cagr) and revenue_cagr < 0.05)
    )

    missing_core = []
    for name, value in [
        ("Revenue CAGR", revenue_cagr),
        ("Operating Margin", operating_margin),
        ("FCF Margin", fcf_margin),
        ("Price/Sales", ps),
    ]:
        if math.isnan(value):
            missing_core.append(name)

    business_model_clues = []
    if aerospace_like:
        business_model_clues.append("Aerospace / space systems")
    if biotech_like:
        business_model_clues.append("Biotech / clinical-stage risk")
    if reit_like:
        business_model_clues.append("REIT-like structure")
    if insurance_like:
        business_model_clues.append("Insurance-like underwriting")
    if reit_like:
        business_model_clues.append("REIT-like real estate income")
    if utility_like:
        business_model_clues.append("Utilities / regulated infrastructure")
    if transport_like:
        business_model_clues.append("Transport / shipping cycle")
    if consumer_retail_like and not semiconductor_like:
        business_model_clues.append("Consumer / retail operating model")
    if financial and not insurance_like:
        business_model_clues.append("Financial services")
    if cyclical_signal:
        business_model_clues.append("Cyclical / commodity-sensitive")
    if semiconductor_like:
        business_model_clues.append("Semiconductor / manufacturing turnaround")
    if ai_semiconductor_like:
        business_model_clues.append("AI semiconductor / data-center platform")

    if fund_like:
        primary = "Unknown / Data-Limited Screening"
        secondary = "Fund-like Instrument"
        coverage = "SCREENING_ONLY"
        confidence = "LOW"
        thesis = "Fund-like instrument. Company financial-statement analysis is not the right frame; holdings, fees, liquidity, and benchmark exposure matter more."
        anchor = "Screening-only / fund-like instrument; avoid company-operating-metric conclusions."
        dominant = ["holdings", "expense ratio", "liquidity", "tracking"]
        invalid = ["ordinary FCF margin", "net debt / EBITDA", "company PE thesis"]
    elif insurance_like:
        primary = "Insurance-like Screening"
        secondary = "Financials"
        coverage = "SCREENING_ONLY"
        confidence = "MEDIUM" if "insurance" in industry or "insurance" in summary else "LOW"
        thesis = "Insurance-like company. The core question is underwriting quality, combined ratio, reserve adequacy, float, investment income, catastrophe exposure, and premium growth quality."
        anchor = "Insurance Screening / ordinary industrial cash-flow framing is insufficient; focus on underwriting, reserves, float, and investment income."
        dominant = ["combined ratio", "underwriting margin", "reserve adequacy", "investment income", "float", "catastrophe exposure", "premium growth quality"]
        invalid = ["ordinary FCF margin as core frame", "EV / EBITDA as core frame", "ordinary cash runway"]
        missing_core.extend(["combined ratio", "underwriting margin", "reserve adequacy", "investment income", "float", "catastrophe exposure"])
    elif reit_like:
        primary = "REIT-like Screening"
        secondary = "Real Estate Income"
        coverage = "SCREENING_ONLY"
        confidence = "MEDIUM"
        thesis = "REIT-like company. Ordinary EPS and FCF screens are not enough; FFO, AFFO, occupancy, rent spread, same-store NOI, leverage, debt maturity, and cap rates drive the research frame."
        anchor = "REIT Screening / focus on FFO, AFFO, occupancy, debt maturity, cap rates, and property-type risk."
        dominant = ["FFO", "AFFO", "occupancy", "rent spread", "same-store NOI", "debt maturity", "cap rate", "property type"]
        invalid = ["ordinary PE conclusion", "ordinary FCF quality conclusion", "EV / EBITDA as core frame"]
        missing_core.extend(["FFO", "AFFO", "occupancy", "rent spread", "same-store NOI", "debt maturity", "cap rate", "property type"])
    elif financial:
        primary = "Financials"
        secondary = "Data-Limited Financial Screening" if missing_core else ""
        coverage = "PARTIAL"
        confidence = "MEDIUM" if "financial" in sector else "LOW"
        thesis = "Financial company. The key question is not ordinary free cash flow, but whether ROE is supported by asset quality, NIM, credit discipline, funding cost, and capital adequacy."
        anchor = "Financial Screening / valuation depends on ROE, asset quality, NIM, credit losses, and capital adequacy."
        dominant = ["ROE", "P/B", "NIM", "credit losses", "capital adequacy"]
        invalid = ["ordinary FCF margin", "net debt / EBITDA", "ordinary cash runway", "EV / EBITDA as core frame"]
        missing_core.extend(["NIM", "credit losses", "capital ratio"])
    elif biotech_like:
        primary = "Unknown / Data-Limited Screening"
        secondary = "Biotech-like Screening"
        coverage = "SCREENING_ONLY"
        confidence = "LOW"
        thesis = "Possible clinical or biotech-like company. Standard PE, PS, and FCF screens cannot explain the full case without pipeline, trial, regulatory, cash runway, and dilution data."
        anchor = "Screening-only / biotech-like; focus on cash runway and missing pipeline evidence."
        dominant = ["cash runway", "R&D burn", "pipeline stage", "trial milestones", "dilution"]
        invalid = ["ordinary PE conclusion", "ordinary FCF quality conclusion", "short-term revenue growth quality"]
        missing_core.extend(["pipeline stage", "clinical trial status", "regulatory milestones"])
    elif ai_semiconductor_like and hybrid_signal:
        primary = "Hybrid AI Semiconductor Compounder"
        secondary = "AI Semiconductor / Data Center Growth Compounder"
        coverage = "PARTIAL"
        confidence = "MEDIUM"
        thesis = "AI semiconductor growth compounder. The core question is whether AI data-center revenue, gross-margin sustainability, supply capacity, customer concentration, hyperscaler capex, export controls, and platform ecosystem strength can keep supporting the premium multiple."
        anchor = "AI Semiconductor Compounder / profitable high-growth chip platform; focus on AI data-center revenue, capacity, gross margins, customer concentration, hyperscaler capex, export controls, ecosystem durability, and valuation premium risk."
        dominant = [
            "AI data center revenue",
            "gross margin sustainability",
            "supply / capacity constraint",
            "customer concentration",
            "hyperscaler capex cycle",
            "export control risk",
            "networking / accelerator ecosystem",
            "valuation premium risk",
        ]
        invalid = ["ordinary mature low-growth thesis", "simple PE compression without growth durability", "generic semiconductor framing"]
        missing_core.extend(
            [
                "AI data center revenue and margin",
                "supply / capacity constraint",
                "customer concentration",
                "hyperscaler capex cycle",
                "export control exposure",
                "networking / accelerator ecosystem",
                "CUDA / platform ecosystem evidence",
                "valuation premium sensitivity",
            ]
        )
    elif semiconductor_turnaround_signal:
        primary = "Capital-Intensive Semiconductor Turnaround"
        secondary = "Hybrid / Technology Manufacturing"
        coverage = "PARTIAL"
        confidence = "MEDIUM" if semiconductor_core else "LOW"
        thesis = "Capital-intensive semiconductor turnaround. The core question is not whether PE looks low, but whether process execution, foundry progress, data-center competitiveness, gross-margin recovery, and free-cash-flow pressure can improve together."
        anchor = "Semiconductor Turnaround / capital-intensive manufacturing; focus on foundry execution, capex pressure, gross margin recovery, segment competitiveness, and free cash flow pressure."
        dominant = ["foundry execution", "capex intensity", "gross margin recovery", "data center competitiveness", "process roadmap", "free cash flow pressure"]
        invalid = ["simple mature compounder thesis", "low PE means cheap", "ordinary software margin thesis", "buyback-supported EPS as core thesis"]
        missing_core.extend(
            [
                "foundry revenue / margin",
                "foundry operating loss",
                "data center segment revenue and margin",
                "client computing segment trend",
                "capex roadmap",
                "free cash flow bridge",
                "process node progress",
                "manufacturing yield / utilization",
                "AI accelerator competitiveness",
                "inventory pressure",
                "gross margin bridge",
                "segment-level operating income",
            ]
        )
    elif speculative_signal:
        primary = "Speculative Growth" if not math.isnan(revenue_cagr) or not math.isnan(latest_growth) else "Unprofitable Growth"
        secondary = "Aerospace / Space Systems" if aerospace_like else ""
        coverage = "PARTIAL" if aerospace_like else "FULL"
        confidence = "MEDIUM"
        thesis = "High-growth but unprofitable/speculative growth. The key question is not whether revenue is growing, but whether revenue growth can convert into gross margin improvement, operating loss narrowing, lower cash burn, and a credible path to profitability."
        anchor = "Speculative Watchlist / high-growth but unprofitable; focus on growth quality, burn, runway, dilution, and path to profitability."
        dominant = ["revenue growth", "gross margin trend", "operating loss", "FCF burn", "cash runway", "PS / EV Revenue"]
        invalid = ["PE compression", "buyback-supported EPS", "mature cash-flow thesis", "Services thesis"]
        missing_core.extend(["backlog/order conversion", "dilution plan"])
    elif hybrid_signal:
        primary = "Hybrid Growth Compounder"
        secondary = "Profitable Growth"
        coverage = "FULL"
        confidence = "MEDIUM"
        thesis = "Mature profitability with growth-premium expectations. The key question is whether high growth can continue long enough to justify the premium multiple without margin or demand-cycle reversal."
        anchor = "Hybrid Growth Compounder / profitable growth with premium valuation; test growth durability and margin reversal risk."
        dominant = ["revenue growth", "operating margin", "FCF margin", "valuation multiple", "demand durability"]
        invalid = ["pure mature low-growth framing", "pure speculative burn framing"]
    elif cyclical_signal:
        primary = "Cyclical"
        secondary = "Asset Heavy"
        coverage = "PARTIAL"
        confidence = "MEDIUM"
        thesis = "Cyclical company. The key question is not whether current PE is low, but whether current earnings are near a cycle peak and whether valuation still works on normalized earnings."
        anchor = "Cyclical Screening / valuation must be judged on normalized earnings, not peak-cycle profit."
        dominant = ["cycle position", "normalized earnings", "through-cycle margin", "capex discipline", "balance sheet"]
        invalid = ["low PE means cheap", "current earnings breakout proves quality"]
        missing_core.extend(["normalized earnings", "cycle position"])
    elif utility_like:
        primary = "Utilities / Infrastructure"
        secondary = "Regulated Asset Base"
        coverage = "PARTIAL"
        confidence = "MEDIUM"
        thesis = "Utilities or infrastructure company. The core question is regulated return, rate base growth, debt cost, capex plan, dividend coverage, and allowed ROE."
        anchor = "Utilities Screening / focus on regulated asset base, allowed ROE, rate cases, debt cost, and dividend coverage."
        dominant = ["regulated asset base", "allowed ROE", "rate case", "capex plan", "debt cost", "dividend coverage"]
        invalid = ["ordinary high-growth thesis", "software margin thesis"]
        missing_core.extend(["regulated asset base", "allowed ROE", "rate case", "dividend coverage"])
    elif transport_like:
        primary = "Shipping / Airlines / Transport"
        secondary = "Cycle-Sensitive Transport"
        coverage = "PARTIAL"
        confidence = "MEDIUM"
        thesis = "Transport or shipping-like company. The core question is rate cycle, utilization, fuel cost, fleet age, orderbook, leverage, and demand-cycle sensitivity."
        anchor = "Transport Cycle Screening / focus on freight rate or yield, utilization, fuel cost, fleet age, orderbook, leverage, and cycle sensitivity."
        dominant = ["freight rate / yield", "load factor / utilization", "fuel cost", "fleet age", "orderbook", "leverage", "cycle sensitivity"]
        invalid = ["stable SaaS growth thesis", "ordinary mature compounder thesis"]
        missing_core.extend(["freight rate / yield", "load factor / utilization", "fuel cost", "fleet age", "orderbook"])
    elif consumer_retail_like and not mature_signal:
        primary = "Consumer / Retail"
        secondary = "Store / Brand Economics"
        coverage = "PARTIAL"
        confidence = "MEDIUM"
        thesis = "Consumer or retail company. The core question is same-store sales, traffic, ticket, inventory, gross margin, store count, brand strength, and pricing power."
        anchor = "Consumer / Retail Screening / focus on same-store sales, traffic, ticket, inventory, gross margin, store growth, and pricing power."
        dominant = ["same-store sales", "traffic", "ticket", "inventory", "gross margin", "store count", "brand / pricing power"]
        invalid = ["pure software thesis", "ordinary industrial cycle thesis"]
        missing_core.extend(["same-store sales", "traffic", "ticket", "inventory", "store count"])
    elif mature_signal:
        primary = "Mature Compounder"
        secondary = ""
        coverage = "FULL"
        confidence = "HIGH"
        thesis = "Mature cash-flow compounder. The key question is not explosive revenue growth, but whether margin durability, free cash flow stability, business mix, and buybacks can continue to support premium valuation."
        anchor = "Watchlist / mature cash-flow quality, but valuation demands margin durability, FCF stability, and buyback discipline."
        dominant = ["margin durability", "FCF stability", "business mix", "buybacks", "PE / FCF sensitivity"]
        invalid = ["burn runway as core thesis", "dilution-first framing"]
    elif len(missing_core) >= 3:
        primary = "Unknown / Data-Limited Screening"
        secondary = ""
        coverage = "UNKNOWN"
        confidence = "LOW"
        thesis = "Current public data is not enough to choose a reliable company-specific research frame. This report is screening-only until business model, industry drivers, and core metrics are manually verified."
        anchor = "Unknown / Data-Limited Screening; do not force a mature, speculative, financial, or cyclical template."
        dominant = ["data availability", "business model verification", "manual framework selection"]
        invalid = ["confident PE thesis", "confident growth thesis", "confident mature compounder thesis"]
    else:
        primary = "Unknown / Data-Limited Screening"
        secondary = ""
        coverage = "SCREENING_ONLY"
        confidence = "LOW"
        thesis = "The company does not cleanly match the built-in research frames. The report can screen price, valuation, and basic financials, but it cannot claim a complete industry-specific thesis."
        anchor = "Screening-only / framework uncertainty; expose missing company-specific drivers."
        dominant = ["basic financials", "benchmark return", "valuation snapshot", "manual business model review"]
        invalid = ["template-specific conclusion without evidence"]

    profitability = "Profitable" if positive_net_income_years >= 2 and not math.isnan(operating_margin) and operating_margin > 0 else "Unprofitable / not established"
    cash_flow = "Positive FCF" if positive_fcf_years >= 2 and not math.isnan(fcf_margin) and fcf_margin > 0 else "Negative or unproven FCF"
    valuation_fit = "PS / EV Revenue / burn / dilution" if primary in {"Speculative Growth", "Unprofitable Growth"} else "cash runway / R&D burn / pipeline milestones / dilution" if secondary == "Biotech-like Screening" else "turnaround / margin recovery / capex pressure / segment verification" if primary == "Capital-Intensive Semiconductor Turnaround" else "AI data-center growth / margin durability / valuation premium sensitivity" if primary == "Hybrid AI Semiconductor Compounder" else "P/B / ROE" if primary == "Financials" else "combined ratio / float / investment income" if primary == "Insurance-like Screening" else "P/FFO / AFFO yield / NAV / cap rate" if primary == "REIT-like Screening" else "normalized earnings" if primary == "Cyclical" else "regulated asset base / allowed ROE / dividend coverage" if primary == "Utilities / Infrastructure" else "rate cycle / utilization / fuel / leverage" if primary == "Shipping / Airlines / Transport" else "same-store sales / margin / inventory / brand" if primary == "Consumer / Retail" else "PE / FCF sensitivity" if primary in {"Mature Compounder", "Hybrid Growth Compounder"} else "screening-only"
    capital_intensity = "High / execution dependent" if aerospace_like or cyclical_signal or primary == "Capital-Intensive Semiconductor Turnaround" else "Unknown" if len(missing_core) >= 3 else "Moderate"
    dilution_risk = "Elevated" if cash_flow.startswith("Negative") and (math.isnan(cash_runway) or cash_runway < 4) else "Normal / not primary"
    cyclicality = "High" if cyclical_signal else "Industry-specific" if aerospace_like else "Not primary"

    deficits = sorted(set(flag for flag in missing_core if flag))
    fallback_count = 0
    fallback_sections: list[str] = []
    if coverage in {"SCREENING_ONLY", "UNKNOWN"}:
        fallback_count += 1
        fallback_sections.append("research_framework")
    if deficits:
        fallback_count += 1
        fallback_sections.append("data_deficit_narrative")

    extension = ""
    if biotech_like:
        extension = "Biotech Research Profile"
    elif insurance_like:
        extension = "Insurance Research Profile"
    elif aerospace_like and primary in {"Speculative Growth", "Unprofitable Growth"}:
        extension = "Aerospace / Space Systems Research Profile"
    elif primary == "Capital-Intensive Semiconductor Turnaround":
        extension = "Semiconductor / Manufacturing Turnaround Research Profile"
    elif primary == "Hybrid AI Semiconductor Compounder":
        extension = "AI Semiconductor / Data Center Growth Compounder Research Profile"
    elif reit_like:
        extension = "REIT Research Profile"
    elif utility_like:
        extension = "Utilities / Infrastructure Research Profile"
    elif transport_like:
        extension = "Transport / Shipping / Airlines Research Profile"
    elif consumer_retail_like:
        extension = "Consumer / Retail Research Profile"
    elif coverage in {"SCREENING_ONLY", "UNKNOWN"}:
        extension = "Company-specific industry framework"

    return AssetProfile(
        lifecycle_profile=primary,
        primary_profile=primary,
        secondary_profile=secondary,
        profitability_status=profitability,
        sector_type=sector or "unknown",
        sector_confidence=confidence,
        valuation_method_fit=valuation_fit,
        cash_flow_profile=cash_flow,
        capital_intensity=capital_intensity,
        dilution_risk=dilution_risk,
        cyclicality_risk=cyclicality,
        financial_company_flag=financial,
        dominant_metric_set=dominant,
        invalid_metric_set=invalid,
        data_deficit_flags=deficits,
        research_stance_anchor=anchor,
        report_thesis_spine=thesis,
        thesis_spine_confidence=confidence,
        framework_coverage_level=coverage,
        fallback_used_count=fallback_count,
        fallback_used_sections=fallback_sections,
        business_model_clues=business_model_clues,
        suggested_framework_extension=extension,
    )


def build_key_questions(profile: AssetProfile, ticker: str, benchmark: str, language: str = "en") -> str:
    speculative = profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}
    semiconductor = profile.primary_profile == "Capital-Intensive Semiconductor Turnaround"
    financial = profile.primary_profile == "Financials"
    insurance = profile.primary_profile == "Insurance-like Screening"
    reit = profile.primary_profile == "REIT-like Screening"
    biotech = profile.secondary_profile == "Biotech-like Screening"
    consumer = profile.primary_profile == "Consumer / Retail"
    utility = profile.primary_profile == "Utilities / Infrastructure"
    transport = profile.primary_profile == "Shipping / Airlines / Transport"
    cyclical = profile.primary_profile == "Cyclical"
    unknown = profile.primary_profile == "Unknown / Data-Limited Screening"
    if language == "zh":
        if speculative:
            questions = [
                ("增长质量有没有改善？", "当前重点不是收入有没有增长，而是增长能不能转成毛利率改善、亏损收窄和现金消耗下降。", "资产画像显示公司更接近高增长但盈利未稳的类型。", "如果 backlog、订单转化或项目执行数据缺失，这个判断只能作为研究假设。"),
                ("现金还能支撑多久？", "现金 runway 和融资稀释是核心风险，不应被成熟公司估值语言掩盖。", "自由现金流为负或尚未稳定时，现金余额、债务和融资需求比 PE 更重要。", "需要手工核查最新 10-Q、10-K、投资者材料中的现金消耗、融资计划和订单转化。"),
                (f"为什么不直接买 {benchmark}？", f"{ticker} 必须证明单一高风险资产值得承担。", "个股波动、回撤和现金消耗风险通常高于宽基基准。", "这不是未来收益预测，只是说明研究门槛更高。"),
            ]
        elif semiconductor:
            questions = [
                ("这是不是普通成熟科技股？", "不是。它更像资本开支重、毛利率承压、制造和 foundry 转型需要兑现的半导体转型案例。", "资产画像命中半导体、制造转型、自由现金流压力或利润率修复信号。", "这不是说转型一定成功；必须核查业务线、制程进度和现金流桥。"),
                ("当前估值应该怎么看？", "核心不是 PE 低不低，而是先进制程、foundry 进展、数据中心竞争位置、毛利率修复和自由现金流压力能否同时改善。", "半导体转型公司的盈利质量可能被周期、资本开支和一次性修复扭曲。", "需要手工核查 foundry 收入 / 亏损、capex roadmap、segment operating income 和毛利率 bridge。"),
                (f"为什么不直接买 {benchmark}？", f"{ticker} 必须证明制造转型和资本开支压力能换来更强竞争力。", "单一个股承担制程、产能利用率、竞争和现金流风险。", "历史表现不能证明转型会兑现。"),
            ]
        elif financial:
            questions = [
                ("ROE 是由质量支撑，还是由杠杆撑出来？", "金融公司不能用普通工业公司的自由现金流框架判断。", "金融画像要求看 ROE、资产质量、净息差、信用损失和资本充足率。", "如果 NIM、信用损失和资本率缺失，本报告只能降级为金融初筛。"),
                ("估值该看什么？", "P/B 与 ROE 的匹配度比普通 PE/FCF 叙事更关键。", "金融公司资产负债表本身就是业务引擎。", "必须核查监管资本、资产质量和拨备覆盖。"),
                ("最大风险是什么？", "信用周期、融资成本和资产质量恶化比普通经营现金流更重要。", "行业框架决定风险语言。", "缺行业专属指标时不能写成完整投研结论。"),
            ]
        elif cyclical:
            questions = [
                ("当前利润是不是周期高点？", "周期股不能因为当前 PE 低就直接说便宜。", "周期画像要求用正常化利润和穿越周期的资产负债表看估值。", "必须核查周期位置、价格变量、订单或商品价格。"),
                ("估值怎么判断？", "应使用正常化利润、周期中位 margin 或 through-cycle 指标。", "当前利润可能高估长期盈利能力。", "缺正常化利润时只能做初筛。"),
                ("什么会推翻判断？", "需求或价格周期反转会打穿当前盈利。", "周期股风险来自均值回归。", "下一步要看行业供需和资本开支纪律。"),
            ]
        elif biotech:
            questions = [
                ("普通财务指标能不能解释核心价值？", "不能完整解释。生物科技或 AI 药物发现平台的核心变量是 pipeline、临床里程碑、监管路径、R&D burn、现金 runway、合作收入和稀释风险。", "资产画像显示当前更接近 biotech-like screening，而不是普通 SaaS 或成熟现金流公司。", "没有 pipeline 和临床数据前，PS、PE、FCF 都只能做入口，不能当完整估值框架。"),
                ("现金 runway 是否足够？", "现金 runway 是第一层硬约束，因为研发消耗可能在商业化前持续多年。", "数据缺口包含 pipeline / trial / regulatory 相关指标。", "必须核查现金余额、R&D burn、经营现金流和未来 12-24 个月融资需求。"),
                ("合作收入能不能证明平台有效？", "不能只看收入增长，要拆合作收入、预付款、里程碑付款和可持续性。", "Biotech-like 公司收入可能来自合作或里程碑，不等于产品商业化已验证。", "需要查合作方、合同结构、milestone timing 和临床进展。"),
            ]
        elif reit:
            questions = [
                ("普通 EPS / FCF 能不能作为核心？", "不能。REIT-like 公司应该看 FFO、AFFO、出租率、租金价差、same-store NOI、债务期限和 cap rate。", "资产画像识别为 REIT-like screening。", "如果这些数据缺失，本报告只能初筛。"),
                ("分红和杠杆是否安全？", "要看 AFFO 覆盖、利息成本、债务到期和物业类型。", "REIT 的现金分配和融资成本高度相关。", "必须查债务期限表、租户结构和租金到期。"),
                ("估值怎么看？", "P/FFO、AFFO yield、NAV 折溢价和 cap rate 比普通 PE 更重要。", "REIT 的会计盈利不等同于可分配现金流。", "缺行业数据时不能给完整估值判断。"),
            ]
        elif insurance:
            questions = [
                ("承保是否赚钱？", "保险公司核心不是普通经营现金流，而是 combined ratio、underwriting margin 和 reserve adequacy。", "资产画像识别为 insurance-like screening。", "缺承保和准备金数据时不能下完整结论。"),
                ("浮存金和投资收益是否可靠？", "要看 float、investment income、资产久期和利率敏感性。", "保险公司利润来自承保和投资两条线。", "必须查投资组合、准备金和巨灾风险。"),
                ("最大风险是什么？", "准备金不足、巨灾暴露和赔付周期恶化会打穿看似便宜的估值。", "行业专属指标决定风险语言。", "普通 PE 只能做入口。"),
            ]
        elif consumer:
            questions = [
                ("增长来自客流还是涨价？", "消费零售要拆 same-store sales、traffic、ticket、门店数和库存。", "资产画像识别为 consumer / retail。", "只看总收入会掩盖客流和价格结构。"),
                ("毛利率压力来自哪里？", "库存、折扣、供应链和品牌定价权会影响毛利率。", "消费零售的利润质量和库存周期相关。", "必须查同店销售、库存周转和门店扩张。"),
                ("估值依赖什么？", "品牌力、门店经济、同店销售和现金流稳定性共同决定估值。", "PE/FCF 可作为入口，但不能替代经营指标。", "下一步要查公司披露的 retail KPIs。"),
            ]
        elif utility:
            questions = [
                ("收益是否受监管支持？", "公用事业要看 regulated asset base、allowed ROE、rate case、capex plan 和 debt cost。", "资产画像识别为 utilities / infrastructure。", "缺监管和 rate base 数据时只能初筛。"),
                ("资本开支是否压制自由现金流？", "大型 capex 可能换来 rate base 增长，也可能增加债务压力。", "该类公司常有重资本开支和利率敏感性。", "必须查 rate case、融资成本和分红覆盖。"),
                ("估值怎么看？", "估值要结合 allowed ROE、债务成本、股息覆盖和监管环境。", "普通成长股框架不适用。", "下一步查监管文件和资本开支计划。"),
            ]
        elif transport:
            questions = [
                ("当前盈利是否处在周期位置？", "运输、航运和航空要看 freight rate / yield、utilization、fuel cost、fleet age 和 orderbook。", "资产画像识别为 transport / shipping cycle。", "只看当前利润可能误判周期。"),
                ("燃油和运价如何影响利润？", "燃油成本、运价和载客率 / 利用率共同决定利润弹性。", "行业高度受周期和成本影响。", "必须查运价、利用率、燃油对冲和债务。"),
                ("估值怎么看？", "要用周期框架看 normalized earnings、leverage 和 fleet/orderbook。", "低 PE 不自动等于便宜。", "下一步查行业供需和订单周期。"),
            ]
        elif unknown:
            questions = [
                ("系统是否理解这家公司？", "当前只能做第一轮初筛，不能当完整行业研究。", "资产画像置信度低或框架覆盖不足。", "下一步必须人工确认业务模式、核心资产和行业专属指标。"),
                ("哪些指标可能不适用？", "普通 PE、FCF 或 PS 可能都不够解释公司价值。", "未知行业需要先判断估值方法。", "报告必须避免套成熟复利或投机成长模板。"),
                ("下一步查什么？", "先查 10-K、10-Q、投资者材料和行业专属经营指标。", "数据缺口会影响研究框架选择。", "查清前只能保持 screening-only。"),
            ]
        else:
            questions = [
                ("公司是否仍在复利式地产生高质量现金流？", "关键不是收入爆发，而是利润率和自由现金流能否守住。", "成熟复利画像强调 margin、FCF、业务结构和回购。", "如果业务线或回购拆分缺失，需要人工复核。"),
                ("当前估值是否透支稳定性？", "高估值要求持续利润质量和现金流支撑。", "市盈率、自由现金流率和业务质量共同决定风险。", "估值高不等于马上下跌，但犯错空间更小。"),
                (f"为什么不直接买 {benchmark}？", "个股必须用更高质量或更高回报证明额外风险值得承担。", "基准比较提供机会成本。", "历史跑赢不能证明未来继续跑赢。"),
            ]
        rows = []
        for q, a, e, b in questions:
            rows.append(f"### 问题：{q}\n\n**回答：** {a}\n\n**证据：** {e}\n\n**边界：** {b}")
        return "## 7. 关键问题与回答\n\n" + "\n\n".join(rows)

    if speculative:
        questions = [
            ("Is growth quality improving?", "The question is not whether revenue is growing; it is whether growth can translate into gross margin improvement, narrower losses, lower burn, and a credible path to profitability.", "The asset profile is speculative or unprofitable growth.", "Backlog, order conversion, execution cadence, and cash runway require manual verification."),
            ("How long can cash support the current burn?", "Cash runway, financing access, and dilution matter more than PE-based valuation.", "Negative or unproven FCF shifts the research frame toward burn and dilution.", "The report does not know management's future financing plan unless it is in primary sources."),
            (f"Why own {ticker} instead of {benchmark}?", "The single-name case must justify higher execution and financing risk.", "Benchmark comparison shows the opportunity cost.", "This is not a forecast of future outperformance."),
        ]
    elif semiconductor:
        questions = [
            ("Is this a normal mature tech company?", "No. The right frame is a capital-intensive semiconductor turnaround: manufacturing execution, foundry progress, data-center competitiveness, gross-margin recovery, and free-cash-flow pressure must improve together.", "The asset profile matched semiconductor and manufacturing-turnaround signals.", "This does not prove the turnaround will work; segment, process, and cash-flow evidence still need verification."),
            ("How should valuation be interpreted?", "The question is not whether PE looks low. Earnings quality can be distorted by cycle pressure, capex, restructuring, and margin recovery.", "The profile marks valuation as turnaround / margin recovery / capex pressure / segment verification.", "Foundry revenue and margin, capex roadmap, segment operating income, and gross-margin bridge need manual review."),
            (f"Why own {ticker} instead of {benchmark}?", "The stock must prove that manufacturing transition risk and capex pressure can produce stronger competitive positioning.", "Single-name risk includes process execution, utilization, competition, and FCF pressure.", "Past price action does not validate the turnaround."),
        ]
    elif financial:
        questions = [
            ("Is ROE supported by quality or leverage?", "Financial companies need asset-quality, NIM, credit-loss, and capital checks rather than ordinary industrial FCF framing.", "The asset profile is Financials.", "If those metrics are missing, the report is financial screening only."),
            ("Which valuation method fits?", "P/B and ROE alignment matter more than ordinary EV/EBITDA or FCF margin.", "The balance sheet is the business engine for financials.", "Regulatory capital and credit quality need primary-source review."),
            ("What can break the case?", "Credit deterioration, funding cost pressure, and capital weakness matter most.", "The risk frame is financial-sector-specific.", "Without sector data, the report must not overstate confidence."),
        ]
    elif cyclical:
        questions = [
            ("Are current earnings near a cycle peak?", "A low current PE is not enough for cyclical companies; normalized earnings matter.", "The asset profile is Cyclical.", "Cycle position and normalized margin require manual review."),
            ("How should valuation be judged?", "Use normalized earnings, through-cycle margin, and cycle stress rather than one-year earnings.", "Cyclical profits can mean-revert.", "The report cannot prove normalized value without cycle data."),
            ("What can break the case?", "Demand or commodity-price reversal can compress earnings quickly.", "Cycle sensitivity is the dominant risk.", "Manual industry checks are required."),
        ]
    elif biotech:
        questions = [
            ("Can ordinary financial metrics explain the core value?", "Not fully. A biotech-like or AI drug discovery platform depends on pipeline stage, trial milestones, regulatory path, R&D burn, cash runway, partnership revenue quality, and dilution risk.", "The profile is biotech-like screening rather than ordinary SaaS or mature cash flow.", "Without pipeline and clinical data, PS, PE, and FCF are only entry points, not a complete valuation framework."),
            ("Is the cash runway enough?", "Cash runway is a hard constraint because R&D burn may continue for years before commercialization.", "The data deficit list includes pipeline / trial / regulatory evidence.", "Verify cash balance, R&D burn, operating cash flow, and 12-24 month financing needs."),
            ("Does partnership revenue validate the platform?", "Revenue growth alone is weak evidence. Split partner revenue, upfront payments, milestone payments, and recurring economics.", "Biotech-like revenue can be collaboration-driven rather than product commercialization.", "Review partner contracts, milestone timing, and trial progress."),
        ]
    elif reit:
        questions = [
            ("Can ordinary EPS / FCF be the core frame?", "No. REIT-like companies need FFO, AFFO, occupancy, rent spread, same-store NOI, debt maturity, interest cost, cap rates, and property type.", "The profile is REIT-like screening.", "If those data are missing, this is only screening."),
            ("Are distributions and leverage safe?", "AFFO coverage, debt maturity, tenant mix, and interest cost matter more than ordinary net income.", "REIT cash distribution and financing costs are tightly linked.", "Check debt maturity schedules, lease structure, and property-level metrics."),
            ("How should valuation be read?", "P/FFO, AFFO yield, NAV discount/premium, and cap rates are more relevant than ordinary PE.", "Accounting earnings do not equal distributable cash flow.", "Missing REIT metrics block a full valuation view."),
        ]
    elif insurance:
        questions = [
            ("Is underwriting profitable?", "The core is combined ratio, underwriting margin, reserve adequacy, and premium quality, not ordinary operating cash flow.", "The profile is insurance-like screening.", "Without underwriting and reserve data, the report cannot be a full insurance note."),
            ("Are float and investment income reliable?", "Float, investment income, asset duration, and interest-rate sensitivity drive the second earnings engine.", "Insurance profits come from underwriting and investments.", "Verify investment portfolio, reserves, and catastrophe exposure."),
            ("What can break the case?", "Reserve inadequacy, catastrophe losses, and claims inflation can overwhelm a superficially cheap multiple.", "Sector-specific metrics define the risk language.", "Ordinary PE is only an entry point."),
        ]
    elif consumer:
        questions = [
            ("Is growth from traffic or price?", "Consumer and retail research needs same-store sales, traffic, ticket, store count, and inventory.", "The profile is Consumer / Retail.", "Total revenue alone can hide traffic and pricing mix."),
            ("Where is margin pressure coming from?", "Inventory, discounting, supply chain cost, and brand pricing power drive gross margin.", "Retail profit quality is tied to inventory and store economics.", "Verify same-store sales, inventory turnover, and store expansion."),
            ("What supports valuation?", "Brand strength, store economics, same-store sales, and cash flow durability support the multiple.", "PE and FCF are entry points, not replacements for operating KPIs.", "Check company-disclosed retail KPIs."),
        ]
    elif utility:
        questions = [
            ("Are returns supported by regulation?", "Utilities need regulated asset base, allowed ROE, rate cases, capex plan, debt cost, and dividend coverage.", "The profile is Utilities / Infrastructure.", "Without regulatory data, this is screening-only."),
            ("Does capex suppress free cash flow?", "Large capex can build rate base but also raise financing pressure.", "This profile is capital intensive and rate-sensitive.", "Verify rate cases, funding cost, and dividend coverage."),
            ("How should valuation be read?", "Allowed ROE, debt cost, dividend coverage, and regulatory climate matter more than generic growth framing.", "The ordinary growth-stock frame does not fit.", "Review regulatory filings and capex plans."),
        ]
    elif transport:
        questions = [
            ("Are earnings cycle-supported?", "Transport and shipping need freight rate / yield, utilization, fuel cost, fleet age, orderbook, leverage, and cycle sensitivity.", "The profile is Transport / Shipping Cycle.", "Current earnings may misstate normalized profitability."),
            ("How do rates and fuel affect profit?", "Fuel cost, rate/yield, and utilization determine operating leverage.", "The industry is cycle and cost sensitive.", "Verify rates, utilization, fuel hedging, and leverage."),
            ("How should valuation be judged?", "Use normalized earnings, leverage, fleet/orderbook, and cycle position.", "Low PE is not automatically cheap.", "Check supply-demand and order-cycle data."),
        ]
    elif unknown:
        questions = [
            ("Does the system understand this company?", "Only partially. The report should be treated as screening-only until the industry frame is manually verified.", "Framework coverage is low or unknown.", "Do not treat generic financial tables as a complete research conclusion."),
            ("Which common metrics may be misleading?", "PE, PS, FCF margin, or net debt metrics may not fit the business model.", "Unknown framework coverage blocks confident valuation framing.", "Manual industry metric selection comes next."),
            ("What should be checked next?", "Read the latest filings and investor materials to identify the company's revenue engine, core assets, and industry-specific operating metrics.", "Data deficits affect the research frame.", "The report should not force a known template."),
        ]
    else:
        questions = [
            ("Is the company still compounding quality cash flow?", "The mature-compounder case depends on margin durability, FCF stability, business mix, and capital returns.", "The asset profile is Mature Compounder or similar.", "Segment and buyback decomposition still need verification."),
            ("Is valuation pricing in too much stability?", "A premium multiple requires durable earnings and cash flow.", "PE/FCF sensitivity fits this profile.", "Expensive does not mean imminent downside."),
            (f"Why own {ticker} over {benchmark}?", "The single-name case must justify concentration risk versus the benchmark.", "Benchmark comparison frames opportunity cost.", "Past outperformance does not predict future outperformance."),
        ]
    rows = []
    for q, a, e, b in questions:
        rows.append(f"### Question: {q}\n\n**Answer:** {a}\n\n**Evidence:** {e}\n\n**Boundary:** {b}")
    return "## 7. Key Questions and Answers\n\n" + "\n\n".join(rows)


def build_battle_card(profile: AssetProfile, ticker: str, language: str = "en") -> str:
    speculative = profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}
    semiconductor = profile.primary_profile == "Capital-Intensive Semiconductor Turnaround"
    if language == "zh":
        if speculative:
            return f"""## 6. 投研博弈卡片

### 买入的核心赌注

{ticker} 的多头逻辑不是“现在已经很赚钱”，而是收入增长、毛利率改善、现金消耗下降和项目执行能够连成一条通往盈利的路径。只有这些信号同时改善，高增长故事才有机会从预期变成经营证据。

### 做空或离场的死穴

反方逻辑从现金消耗和融资稀释开始。如果收入增长没有转成更好的毛利率和更低亏损，估值很容易从“成长溢价”变成“故事折价”。

### 市场已经交易了什么

市场交易的是未来执行，而不是当前利润。对这类公司，市销率、企业价值 / 收入、订单转化、现金 runway 和稀释风险比市盈率更重要。

### 什么必须守住

- 收入增长不能只停留在故事层面。
- 毛利率需要改善。
- 经营亏损和自由现金流消耗要收窄。
- 现金 runway 不能被融资压力打穿。
- 项目、订单或产能执行要兑现。

### 一票否决条件

- 收入增长放缓但现金消耗没有改善。
- 毛利率恶化。
- 现金 runway 快速缩短。
- 大幅股权融资稀释老股东。
- 关键项目或订单转化明显低于预期。

### 最优先核查

1. 拆业务线收入、毛利率和订单转化。
2. 计算自由现金流消耗和现金 runway，评估融资或稀释风险。
3. 核查项目进度、交付节奏、backlog 和盈利路径。
"""
        if semiconductor:
            return f"""## 6. 投研博弈卡片

### 买入的核心赌注

{ticker} 不是普通高现金流成熟公司，也不是普通成长股。多头赌的是半导体制造转型能兑现：先进制程推进、foundry 业务改善、数据中心竞争力恢复、毛利率修复和自由现金流压力缓解要同时发生。

### 做空或离场的死穴

反方逻辑从资本开支和毛利率开始。如果制造转型继续烧钱、foundry 亏损扩大，或者数据中心竞争位置没有修复，低 PE 或周期修复叙事都不够支撑估值。

### 市场已经交易了什么

市场交易的不是一个普通科技成熟股，而是转型能否落地。当前估值需要用 margin recovery、capex burden、foundry progress、segment operating income 和 free cash flow bridge 一起检验。

### 什么必须守住

- 先进制程和制造节点不能继续失速。
- foundry 收入和亏损路径要改善。
- 数据中心 / AI 竞争位置不能继续被削弱。
- 毛利率修复需要有 segment 和 utilization 证据。
- 资本开支不能长期压垮自由现金流。

### 一票否决条件

- foundry 亏损扩大且没有收入兑现。
- 毛利率修复失败。
- 数据中心竞争继续恶化。
- 资本开支继续压制自由现金流且融资压力上升。
- 制程路线图或产能利用率明显低于管理层叙事。

### 最优先核查

1. 拆 foundry、data center、client computing 的收入和 operating income。
2. 核查 capex roadmap、free cash flow bridge 和补贴 / 政府资金影响。
3. 跟踪 process node 进度、制造良率 / 利用率和 AI accelerator 竞争力。
"""
        if profile.secondary_profile == "Biotech-like Screening":
            return f"""## 6. 投研博弈卡片

### 买入的核心赌注

{ticker} 不是普通 SaaS 成长股，也不是已经稳定产现金的成熟公司。多头逻辑取决于 pipeline 推进、监管里程碑、R&D 效率、现金 runway、合作收入质量和稀释控制。

### 做空或离场的死穴

反方逻辑是现金压力先于临床或平台证据到来。如果 pipeline 里程碑延后、R&D burn 居高不下，或者融资稀释加速，普通收入增长不足以支撑主线。

### 市场已经交易了什么

市场交易的是平台可选性和未来里程碑，不是当前盈利。市销率只能作为情绪入口，真正的研究负担在 pipeline、现金 runway 和监管进展。

### 什么必须守住

- 主要 pipeline 项目必须有可信临床路径。
- R&D burn 必须处于可融资范围内。
- 现金 runway 必须覆盖下一轮关键里程碑。
- 合作收入要能证明外部验证，而不是一次性会计收入。
- 稀释风险必须透明且可控。

### 最优先核查

1. 核查 pipeline 候选项目、适应症、临床阶段和下一次里程碑时间。
2. 核查 FDA / EMA 监管路径、试验披露节奏和审批风险。
3. 重算现金余额、R&D burn、经营现金流和未来 12-24 个月融资需求。
4. 核查合作收入质量：合作方、预付款、里程碑付款和可持续性。
5. 核查稀释风险：股本变化、SBC、ATM / 增发历史和债务融资可能性。
"""
        if profile.primary_profile == "Unknown / Data-Limited Screening":
            return f"""## 6. 投研博弈卡片

### 当前能判断什么

{ticker} 目前只能做第一轮数据初筛。系统还没有足够证据选择完整行业研究框架，因此不能把它写成成熟复利股、投机成长股、金融股或周期股。

### 当前不能判断什么

本报告不能完整判断公司核心资产、行业专属指标和适用估值方法。任何确定性结论都需要先补业务模式和行业数据。

### 下一步必须守住

- 明确公司到底靠什么赚钱。
- 明确最重要的行业经营指标。
- 判断 PE、PS、P/B、EV/EBITDA 哪些适用，哪些会误导。
- 补齐关键数据缺口。
- 再决定是否进入更深研究。

### 最优先核查

1. 最新 10-K / 10-Q 和投资者材料中的业务说明。
2. 行业专属经营指标和核心资产。
3. 估值方法是否适用，以及缺失数据会改变哪些判断。
"""
        return f"""## 6. 投研博弈卡片

### 买入的核心赌注

{ticker} 的核心赌注是利润率、自由现金流、业务结构和资本回报能继续支撑高质量复利。重点不是收入爆发，而是稳定经营质量能否撑住估值。

### 做空或离场的死穴

反方逻辑从估值过满开始。如果利润率下滑、自由现金流变弱，或回购对每股收益的支撑下降，市场给高估值的理由会变少。

### 市场已经交易了什么

市场已经为稳定性和现金流质量付了价格。接下来要验证的是这种稳定性能否继续，而不是简单判断公司好坏。

### 什么必须守住

- 毛利率和经营利润率不能持续恶化。
- 自由现金流要稳定。
- 业务结构不能明显变差。
- 回购或资本回报不能掩盖基本面放缓。
- 估值不能脱离利润质量。

### 一票否决条件

- 利润率连续走弱。
- 自由现金流质量恶化。
- 业务线增长或结构明显弱化。
- 高估值下每股收益增长失去支撑。
- 监管或竞争削弱核心利润池。

### 最优先核查

1. 分业务线收入和利润贡献。
2. 拆每股收益增长来自净利润还是股本减少。
3. 测试估值对利润率和自由现金流变化的敏感性。
"""

    if speculative:
        return f"""## 6. Research Battle Card

### The Long Bet

{ticker} is an execution-and-financing growth case. The long case requires revenue growth to turn into gross-margin improvement, narrowing losses, lower cash burn, and a credible path to profitability.

### The Short Trigger

The short trigger is growth without operating leverage. If revenue rises but burn, dilution risk, and project execution risk do not improve, the valuation can lose support quickly.

### Market Pricing

The market is pricing future execution rather than current profit. For this profile, PS, EV/Revenue, burn, runway, dilution, backlog, and order conversion matter more than earnings-multiple framing.

### What Must Hold

- Revenue growth must remain real and executable.
- Gross margin must improve.
- Operating losses and FCF burn must narrow.
- Cash runway must not become a forced-financing problem.
- Orders, backlog, projects, or production cadence must convert into revenue.

### Kill Criteria

- Revenue growth slows while burn remains high.
- Gross margin deteriorates.
- Cash runway shortens materially.
- Equity financing creates heavy dilution.
- Key projects or order conversion miss the implied execution path.

### Verification Priority

1. Split business-line revenue, gross margin, and order conversion.
2. Calculate FCF burn and cash runway; assess refinancing or dilution risk.
3. Track project progress, production cadence, backlog conversion, and path to profitability.
"""
    if semiconductor:
        return f"""## 6. Research Battle Card

### The Long Bet

{ticker} is not a simple mature compounder or a plain growth stock. The long case is a semiconductor manufacturing turnaround: process execution, foundry progress, data-center competitiveness, gross-margin recovery, and free-cash-flow pressure must improve together.

### The Short Trigger

The short trigger starts with capex and gross margin. If foundry losses widen, manufacturing transition keeps consuming cash, or data-center competitiveness does not recover, a low-PE or cyclical-recovery story is not enough.

### Market Pricing

The market is not just pricing a normal profitable technology company. It is pricing whether the turnaround can become operating evidence across margin recovery, capex burden, foundry progress, segment operating income, and free cash flow bridge.

### What Must Hold

- Process-node execution must stop slipping.
- Foundry revenue and losses must move in the right direction.
- Data center / AI competitiveness must stabilize or improve.
- Gross-margin recovery needs segment and utilization evidence.
- Capex must not keep suppressing free cash flow indefinitely.

### Kill Criteria

- Foundry losses widen without revenue conversion.
- Gross-margin recovery fails.
- Data-center competitiveness keeps deteriorating.
- Capex keeps pressuring free cash flow while financing risk rises.
- Process roadmap or utilization falls short of management's transition story.

### Verification Priority

1. Split foundry, data center, and client computing revenue and operating income.
2. Check capex roadmap, free cash flow bridge, and subsidy / government-funding impact.
3. Track process-node progress, manufacturing yield / utilization, and AI accelerator competitiveness.
"""
    if profile.secondary_profile == "Biotech-like Screening":
        return f"""## 6. Research Battle Card

### The Long Bet

{ticker} is not a normal SaaS growth stock and not a cash-generative mature company. The long case depends on pipeline progress, regulatory milestones, R&D productivity, cash runway, partnership economics, and dilution control.

### The Short Trigger

The short trigger is clinical or platform evidence that fails to arrive before cash pressure rises. If pipeline milestones slip, R&D burn stays high, or financing dilution accelerates, ordinary revenue growth is not enough.

### Market Pricing

The market is pricing platform optionality and future milestones, not current earnings. PS is only a sentiment entry point; pipeline, cash runway, and regulatory progress carry the research burden.

### What Must Hold

- Lead pipeline programs must keep credible clinical paths.
- R&D burn must remain financeable.
- Cash runway must cover the next milestone window.
- Partnership revenue must signal real external validation.
- Dilution risk must stay visible and controlled.

### Verification Priority

1. Verify pipeline candidates, indications, clinical phase, and next milestone timing.
2. Check FDA / EMA regulatory path, trial disclosure cadence, and approval risk.
3. Recalculate cash balance, R&D burn, operating cash flow, and 12-24 month financing need.
4. Verify partnership revenue quality: partner, upfront payments, milestone payments, and durability.
5. Verify dilution risk: share-count change, SBC, ATM or equity issuance history, and debt financing risk.
"""
    if profile.primary_profile == "Unknown / Data-Limited Screening":
        return f"""## 6. Research Battle Card

### What Can Be Judged

{ticker} can only be treated as first-pass screening right now. The system does not have enough evidence to choose a full industry-specific framework.

### What Cannot Be Judged

The report cannot safely claim a mature, speculative, financial, or cyclical thesis without additional business-model and industry-specific data.

### What Must Hold

- The business model must be manually identified.
- The core industry metrics must be selected.
- Inapplicable valuation methods must be rejected.
- Data gaps must be closed before a stronger stance is written.
- The final framework must match the company, not a template.

### Verification Priority

1. Read the latest filings and investor deck to identify the revenue engine.
2. Identify the three most important industry operating metrics.
3. Decide which valuation methods are valid before interpreting the multiples.
"""
    return f"""## 6. Research Battle Card

### The Long Bet

{ticker} is a mature quality case. The long case rests on margin durability, free cash flow stability, business mix, and capital returns supporting premium valuation.

### The Short Trigger

The short trigger is premium valuation without enough earnings or cash-flow durability. If margins weaken or cash flow quality fades, the multiple has less protection.

### Market Pricing

The market is paying for durability, not just the latest year of revenue. PE and FCF sensitivity are relevant because the company is already profitable and cash-generative.

### What Must Hold

- Margins must remain durable.
- Free cash flow must remain stable.
- Business mix must not deteriorate.
- Buybacks or capital returns must not mask weak fundamentals.
- Valuation must stay tied to earnings quality.

### Kill Criteria

- Margins compress for multiple periods.
- Free cash flow quality deteriorates.
- Segment mix weakens.
- EPS growth depends too heavily on buybacks.
- Premium valuation remains high while growth slows.

### Verification Priority

1. Split business segments and margin contribution.
2. Split EPS growth between net income growth and share-count reduction.
3. Test PE / FCF yield sensitivity against margin and buyback durability.
"""


def build_valuation_section(profile: AssetProfile, report_data: dict[str, Any], language: str = "en") -> str:
    valuation = pd.DataFrame(report_data["valuation_snapshot"])
    pe = _valuation(valuation, "trailingPE")
    ps = _valuation(valuation, "priceToSalesTrailing12Months")
    ev_rev = _valuation(valuation, "enterpriseToRevenue")
    speculative = profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}
    biotech = profile.secondary_profile == "Biotech-like Screening"
    if language == "zh":
        if speculative:
            return f"""## 11. 估值压力测试

**结论：这类公司不能用盈利倍数作为核心估值框架。** 当前重点是市销率、企业价值 / 收入、收入增长、现金消耗、现金 runway 和潜在股权稀释。

对未盈利成长股，收入增长不是免费的保护伞。如果估值倍数下杀，而公司又因为持续烧钱需要融资，股东承受的是估值压缩和股权稀释的双重压力。

| 场景 | 收入增长假设 | 稀释假设 | 研究含义 |
| --- | --- | --- | --- |
| 销售倍数压缩到 80 倍 | 20% / 40% / 60% | 0% / 5% / 10% / 20% | 测试高增长能否抵消轻度估值压缩 |
| 销售倍数压缩到 50 倍 | 20% / 40% / 60% | 0% / 5% / 10% / 20% | 测试市场情绪降温时的压力 |
| 销售倍数压缩到 30 倍 | 20% / 40% / 60% | 0% / 5% / 10% / 20% | 测试成长股估值重定价风险 |

当前市销率约 {_fmt_x(ps)}，企业价值 / 收入约 {_fmt_x(ev_rev)}。这不是目标价预测，而是检查“收入增长、估值倍数和稀释”三者如何共同影响股东结果。
"""
        if profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
            return f"""## 11. 估值压力测试

**结论：这类半导体转型公司不能只问 PE 低不低。** 估值要同时看毛利率修复、资本开支压力、foundry 亏损路径、数据中心竞争力和自由现金流桥。

| 压力点 | 需要核查的数据 | 研究含义 |
| --- | --- | --- |
| 毛利率修复 | gross margin bridge、产能利用率、产品结构 | 判断利润率改善是结构性修复还是短期波动 |
| 资本开支压力 | capex roadmap、free cash flow bridge、补贴影响 | 判断制造转型是否继续压制股东现金流 |
| foundry 兑现 | foundry revenue、foundry operating loss、客户进展 | 判断代工战略是经营证据还是仍停留在叙事 |
| 数据中心竞争 | data center revenue / margin、AI accelerator 竞争力 | 判断高价值业务是否恢复竞争位置 |

当前 trailing PE 约 {_fmt_x(pe)}，市销率约 {_fmt_x(ps)}。这些倍数只能作为入口，不能替代半导体转型验证。
"""
        if biotech:
            return """## 11. 估值压力测试

**结论：biotech-like 公司不能用普通 PE、PS 或 FCF 框架解释核心价值。** 真正变量是 pipeline 阶段、临床里程碑、监管路径、R&D burn、现金 runway、合作收入质量和稀释风险。

| 压力点 | 需要核查的数据 | 研究含义 |
| --- | --- | --- |
| Pipeline 质量 | 主要项目、适应症、临床阶段、下一次数据时间 | 判断平台价值是否有临床证据 |
| 监管路径 | FDA / EMA 节点、试验设计、审批风险 | 判断里程碑是否可执行 |
| 现金 runway | 现金余额、R&D burn、经营现金流、融资需求 | 判断是否存在稀释或融资压力 |
| 合作收入 | 合作方、预付款、里程碑付款、收入持续性 | 判断收入是不是平台验证，而不是一次性会计收入 |

市销率只能作为市场预期入口，不是完整估值框架。
"""
        if profile.primary_profile == "REIT-like Screening":
            return """## 11. 估值压力测试

**结论：REIT-like 公司不能用普通 EPS / FCF 作为核心。** 应核查 FFO、AFFO、出租率、租金价差、same-store NOI、债务期限、利息成本和 cap rate。

缺少这些指标时，本报告只能做初筛，不能形成完整房地产收入资产估值判断。
"""
        if profile.primary_profile == "Insurance-like Screening":
            return """## 11. 估值压力测试

**结论：保险公司估值不能只看普通 PE。** 承保利润、combined ratio、准备金充足性、float、投资收益、巨灾暴露和保费增长质量决定研究框架。

缺承保和准备金数据时，只能把估值倍数当入口，不能当结论。
"""
        if profile.primary_profile == "Utilities / Infrastructure":
            return """## 11. 估值压力测试

**结论：公用事业和基础设施资产要看监管回报和债务成本。** 估值应围绕 rate base、allowed ROE、rate case、capex plan、debt cost 和 dividend coverage 展开。
"""
        if profile.primary_profile == "Shipping / Airlines / Transport":
            return """## 11. 估值压力测试

**结论：运输和航运类公司要用周期框架看估值。** 当前盈利需要和运价 / yield、利用率、燃油成本、fleet age、orderbook、杠杆和正常化利润一起判断。
"""
        if profile.primary_profile == "Consumer / Retail":
            return """## 11. 估值压力测试

**结论：消费零售估值要和门店经济、品牌力和库存周期一起看。** same-store sales、traffic、ticket、库存、毛利率和门店数比单一 PE 更能解释质量。
"""
        if profile.primary_profile == "Financials":
            return """## 11. 估值压力测试

**结论：金融公司不应以普通自由现金流或 EV/EBITDA 作为核心框架。** 估值应围绕 P/B、ROE、资产质量、净息差、信用损失和资本充足率展开。

当前报告缺少部分金融行业专属指标，因此只能作为金融初筛。下一步必须核查监管资本、拨备覆盖、资产质量和资金成本。
"""
        if profile.primary_profile == "Cyclical":
            return """## 11. 估值压力测试

**结论：周期股不能因为当前市盈率低就直接判断便宜。** 估值必须回到正常化利润、周期位置、through-cycle margin 和资产负债表韧性。

当前报告只能提示周期压力测试方向，不能替代行业供需、商品价格或订单周期分析。
"""
        if profile.primary_profile == "Unknown / Data-Limited Screening":
            return """## 11. 估值压力测试

**结论：当前无法可靠选择完整估值框架。** 系统只能展示基础倍数和数据缺口，不能把 PE、PS、P/B 或 EV/EBITDA 中任何一个直接当成核心结论。

下一步必须先确认业务模式、行业专属指标和主要价值驱动，再决定估值方法。
"""
        return f"""## 11. 估值压力测试

**结论：成熟现金流公司最大的估值风险通常不是突然亏损，而是市场不再愿意给高倍数。**

| 场景 | 对比 | 每股收益增长假设 |
| --- | --- | --- |
| 市盈率压缩到 30 倍 | 与当前 {_fmt_x(pe)} 比较 | 0% / 5% / 10% / 15% |
| 市盈率压缩到 25 倍 | 与当前 {_fmt_x(pe)} 比较 | 0% / 5% / 10% / 15% |
| 市盈率压缩到 20 倍 | 与当前 {_fmt_x(pe)} 比较 | 0% / 5% / 10% / 15% |

这不是目标价预测，而是估值压力测试。它回答的不是“应该值多少钱”，而是“如果市场开始杀估值，业务表现需要多强才扛得住”。
"""
    if speculative:
        return f"""## 11. Valuation Stress Test

**Conclusion: earnings-multiple pressure is not the right core frame for this company.** For speculative or unprofitable growth, the relevant frame is PS, EV/Revenue, revenue growth, burn, runway, dilution, and path to profitability.

For unprofitable growth companies, revenue growth alone is not enough. If valuation multiples compress and the company must raise equity, shareholders face both multiple compression and dilution.

| Scenario | Revenue Growth Assumptions | Dilution Assumptions | Research Meaning |
| --- | --- | --- | --- |
| Sales multiple compresses to 80x | 20% / 40% / 60% | 0% / 5% / 10% / 20% | Tests whether growth offsets mild multiple compression |
| Sales multiple compresses to 50x | 20% / 40% / 60% | 0% / 5% / 10% / 20% | Tests pressure when market enthusiasm cools |
| Sales multiple compresses to 30x | 20% / 40% / 60% | 0% / 5% / 10% / 20% | Tests growth-stock re-rating risk |

Current PS is about {_fmt_x(ps)} and EV/Revenue is about {_fmt_x(ev_rev)}. This is not a price target; it is a stress test for the interaction among revenue growth, multiple compression, and dilution.
"""
    if profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
        return f"""## 11. Valuation Stress Test

**Conclusion: this profile cannot be judged by asking whether PE looks low.** Valuation needs a turnaround frame: gross-margin recovery, capex pressure, foundry loss trajectory, data-center competitiveness, and free cash flow bridge.

| Pressure Point | Data to Verify | Research Meaning |
| --- | --- | --- |
| Gross-margin recovery | gross-margin bridge, utilization, product mix | Tests whether margin recovery is structural or temporary |
| Capex pressure | capex roadmap, free cash flow bridge, subsidy impact | Tests whether manufacturing transition keeps suppressing shareholder cash flow |
| Foundry execution | foundry revenue, foundry operating loss, customer progress | Tests whether the foundry strategy is becoming operating evidence |
| Data-center competitiveness | data center revenue / margin, AI accelerator position | Tests whether high-value segments are recovering competitive position |

Current trailing PE is about {_fmt_x(pe)} and PS is about {_fmt_x(ps)}. These multiples are only entry points; they do not replace semiconductor turnaround verification.
"""
    if biotech:
        return """## 11. Valuation Stress Test

**Conclusion: ordinary PE, PS, and FCF do not explain the core value of a biotech-like company.** The real variables are pipeline stage, clinical milestones, regulatory path, R&D burn, cash runway, partnership revenue quality, and dilution risk.

| Pressure Point | Data to Verify | Research Meaning |
| --- | --- | --- |
| Pipeline quality | lead programs, indications, trial stage, next data readout | Tests whether platform value has clinical evidence |
| Regulatory path | FDA / EMA milestones, trial design, approval risk | Tests whether milestones are executable |
| Cash runway | cash balance, R&D burn, operating cash flow, financing need | Tests dilution or refinancing pressure |
| Partnership revenue | partner, upfront payments, milestones, recurring economics | Tests whether revenue validates the platform or is one-off accounting revenue |

PS is only a market-expectation entry point, not a complete valuation framework.
"""
    if profile.primary_profile == "REIT-like Screening":
        return """## 11. Valuation Stress Test

**Conclusion: REIT-like companies should not be judged by ordinary EPS / FCF alone.** Verify FFO, AFFO, occupancy, rent spread, same-store NOI, debt maturity, interest cost, property type, and cap rates.
"""
    if profile.primary_profile == "Insurance-like Screening":
        return """## 11. Valuation Stress Test

**Conclusion: insurance valuation cannot stop at ordinary PE.** Underwriting profit, combined ratio, reserve adequacy, float, investment income, catastrophe exposure, and premium growth quality define the frame.
"""
    if profile.primary_profile == "Utilities / Infrastructure":
        return """## 11. Valuation Stress Test

**Conclusion: utilities and infrastructure assets need a regulated-return frame.** Rate base, allowed ROE, rate cases, capex plan, debt cost, and dividend coverage matter more than generic growth language.
"""
    if profile.primary_profile == "Shipping / Airlines / Transport":
        return """## 11. Valuation Stress Test

**Conclusion: transport and shipping valuation is cycle-sensitive.** Freight rate or yield, utilization, fuel cost, fleet age, orderbook, leverage, and normalized earnings determine the stress frame.
"""
    if profile.primary_profile == "Consumer / Retail":
        return """## 11. Valuation Stress Test

**Conclusion: consumer and retail valuation must be checked against store economics and brand strength.** Same-store sales, traffic, ticket, inventory, gross margin, store count, and pricing power explain quality better than one multiple.
"""
    if profile.primary_profile == "Financials":
        return """## 11. Valuation Stress Test

**Conclusion: ordinary FCF or EV/EBITDA is not the core frame for financial companies.** The relevant frame is P/B, ROE, asset quality, NIM, credit losses, funding cost, and capital adequacy.

The current report is financial screening only if those sector metrics are missing.
"""
    if profile.primary_profile == "Cyclical":
        return """## 11. Valuation Stress Test

**Conclusion: a low current PE does not automatically mean a cyclical is cheap.** Valuation must be tested against normalized earnings, cycle position, through-cycle margin, and balance-sheet resilience.
"""
    if profile.primary_profile == "Unknown / Data-Limited Screening":
        return """## 11. Valuation Stress Test

**Conclusion: the current data does not support a full valuation framework.** The report can show screening multiples, but it should not promote PE, PS, P/B, or EV/EBITDA as the right answer before the business model and industry metrics are verified.
"""
    return f"""## 11. Valuation Stress Test

**Conclusion: for mature cash-flow companies, valuation risk usually comes from multiple compression rather than immediate solvency pressure.**

| Scenario | Comparison | EPS Growth Assumptions |
| --- | --- | --- |
| PE compresses to 30x | Compared with current {_fmt_x(pe)} | 0% / 5% / 10% / 15% |
| PE compresses to 25x | Compared with current {_fmt_x(pe)} | 0% / 5% / 10% / 15% |
| PE compresses to 20x | Compared with current {_fmt_x(pe)} | 0% / 5% / 10% / 15% |

This is not a price target. It is a valuation stress test.
"""


def profile_forbidden_terms(profile: AssetProfile) -> list[str]:
    if profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}:
        return [
            "mature cash-flow company",
            "mature cash-flow business",
            "Services thesis",
            "buyback-supported EPS",
            "PE compression",
            "high-quality cash flow supports valuation",
            "recurring Services growth",
            "margins and buybacks defend the multiple",
            "成熟现金流公司",
            "服务业务",
            "回购支撑",
            "市盈率压缩",
        ]
    if profile.primary_profile == "Financials":
        return ["ordinary FCF margin", "net debt / EBITDA as core", "普通自由现金流作为核心"]
    if profile.primary_profile == "Cyclical":
        return ["low PE means cheap", "PE is low so it is cheap", "市盈率低所以便宜"]
    if profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
        return [
            "simple mature compounder",
            "ordinary mature compounder",
            "low PE means cheap",
            "Services thesis",
            "buyback-supported EPS",
            "对成熟公司来说",
            "成熟复利股",
            "市盈率低所以便宜",
        ]
    if profile.primary_profile == "Unknown / Data-Limited Screening":
        return ["high-conviction", "complete thesis", "高信念", "完整投研结论"]
    return []


BOUNDARY_NEGATION_MARKERS = [
    "not ",
    "no ",
    "cannot",
    "can't",
    "should not",
    "do not",
    "does not",
    "is not",
    "are not",
    "不能",
    "不应",
    "不可",
    "不得",
    "不是",
    "不代表",
    "不能把",
    "不应视为",
]

MISLEADING_COMPLETENESS_CLAIMS = [
    "can be treated as a complete research conclusion",
    "complete investment research conclusion",
    "complete industry research conclusion",
    "directly as a complete research basis",
    "low pe means cheap",
    "low pe proves undervaluation",
    "本报告可以作为完整投研结论",
    "本报告已经形成完整行业研究",
    "可直接作为完整研究依据",
    "低 PE 证明低估",
    "低市盈率证明低估",
]


def _is_negated_boundary_use(content_lower: str, start: int) -> bool:
    window = content_lower[max(0, start - 90): start]
    return any(marker.lower() in window for marker in BOUNDARY_NEGATION_MARKERS)


def lifecycle_logic_check(profile: AssetProfile, blocks: dict[str, str]) -> dict[str, Any]:
    content = "\n".join(blocks.values())
    content_lower = content.lower()
    failures = []
    for term in profile_forbidden_terms(profile):
        term_lower = term.lower()
        start = content_lower.find(term_lower)
        if start >= 0 and not _is_negated_boundary_use(content_lower, start):
            failures.append(
                {
                    "failure_id": f"FORBIDDEN_TERM_{len(failures)+1}",
                    "failed_check": "profile/body conflict",
                    "affected_sections": ["interpretation"],
                    "forbidden_terms": [term],
                    "expected_profile_logic": profile.primary_profile,
                    "severity": "HIGH",
                }
            )
    for claim in MISLEADING_COMPLETENESS_CLAIMS:
        claim_lower = claim.lower()
        start = content_lower.find(claim_lower)
        if start >= 0 and not _is_negated_boundary_use(content_lower, start):
            failures.append(
                {
                    "failure_id": f"MISLEADING_CLAIM_{len(failures)+1}",
                    "failed_check": "misleading completeness claim",
                    "affected_sections": ["interpretation"],
                    "forbidden_terms": [claim],
                    "expected_profile_logic": "screening report boundary",
                    "severity": "HIGH",
                }
            )
    status = STATUS_FAIL if failures else STATUS_PASS
    return {"status": status, "failure_reasons": failures}


def build_report_blocks(report_data: dict[str, Any], profile: AssetProfile, language: str = "en") -> list[dict[str, Any]]:
    ticker = report_data["ticker"]
    benchmark = report_data["benchmark"]
    if language == "zh":
        if profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}:
            verdict = f"{ticker} 应按投机成长股处理：重点不是现在赚了多少钱，而是增长质量、现金消耗、现金 runway、稀释风险和盈利路径是否同步改善。"
            core = "高增长但盈利尚未稳定。核心问题不是有没有增长，而是收入增长能否转成毛利率改善、亏损收窄、现金消耗下降和可信的盈利路径。"
        elif profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
            verdict = f"{ticker} 应按半导体制造转型案例处理：重点不是 PE 低不低，而是 foundry、制程、毛利率、资本开支和自由现金流压力能否一起改善。"
            core = "资本开支重的半导体转型公司。核心问题不是普通成熟公司稳定性，而是先进制程推进、代工业务兑现、数据中心竞争位置、毛利率修复和自由现金流压力能否同时改善。"
        elif profile.secondary_profile == "Biotech-like Screening":
            verdict = f"{ticker} 应按 biotech-like 初筛处理：核心不是普通收入增长，而是 pipeline、临床 / 监管里程碑、R&D burn、现金 runway、合作收入质量和稀释风险。"
            core = "Biotech-like / 平台药物发现公司不能用普通 PE、PS、FCF 框架解释核心价值。当前主线只能是：pipeline 是否有证据，现金能否撑到下一轮里程碑，合作收入是否证明平台有效，以及稀释风险是否可控。"
        elif profile.primary_profile == "Unknown / Data-Limited Screening":
            verdict = f"{ticker} 只能作为第一轮数据初筛，当前系统还不能可靠选择完整行业研究框架。"
            core = "当前数据不足以支持完整行业判断。报告能做基础筛查，但不能把普通财务表格包装成完整投研结论。"
        elif profile.primary_profile == "Financials":
            verdict = f"{ticker} 应按金融类公司初筛，重点是 ROE、资产质量、净息差、信用损失和资本充足率。"
            core = "金融公司不能用普通工业公司的自由现金流框架替代。核心问题是 ROE 是否由资产质量和资本结构支撑。"
        elif profile.primary_profile == "Cyclical":
            verdict = f"{ticker} 应按周期公司处理，不能因为当前市盈率低就直接判断便宜。"
            core = "周期股核心不是当前利润有多高，而是利润是否处在周期高点，以及正常化利润下估值是否仍然成立。"
        else:
            verdict = f"{ticker} 更像成熟现金流复利型公司，重点是利润率、自由现金流、业务结构和资本回报能否继续支撑估值。"
            core = "成熟现金流复利型公司。核心问题不是收入爆发，而是利润率、自由现金流、业务结构和回购能否继续支撑较高估值。"
    else:
        verdict = profile.research_stance_anchor
        core = profile.report_thesis_spine
    blocks = [
        {
            "block_id": "one_line_verdict",
            "block_type": "EDITABLE_ONE_LINE_VERDICT",
            "section_name": "One-line Verdict" if language == "en" else "一句话结论",
            "language": language,
            "content": verdict,
            "locked_data_refs": ["asset_profile.primary_profile", "asset_profile.research_stance_anchor"],
            "editable": True,
        },
        {
            "block_id": "core_view",
            "block_type": "EDITABLE_CORE_VIEW",
            "section_name": "Core View" if language == "en" else "报告主线",
            "language": language,
            "content": core,
            "locked_data_refs": ["asset_profile.report_thesis_spine"],
            "editable": True,
        },
        {
            "block_id": "battle_card",
            "block_type": "EDITABLE_BATTLE_CARD",
            "section_name": "Research Battle Card" if language == "en" else "投研博弈卡片",
            "language": language,
            "content": build_battle_card(profile, ticker, language),
            "locked_data_refs": ["asset_profile.primary_profile", "asset_profile.dominant_metric_set"],
            "editable": True,
        },
        {
            "block_id": "key_questions",
            "block_type": "EDITABLE_QA",
            "section_name": "Key Questions and Answers" if language == "en" else "关键问题与回答",
            "language": language,
            "content": build_key_questions(profile, ticker, benchmark, language),
            "locked_data_refs": ["asset_profile.primary_profile", "asset_profile.data_deficit_flags"],
            "editable": True,
        },
        {
            "block_id": "valuation",
            "block_type": "EDITABLE_VALUATION_EXPLANATION",
            "section_name": "Valuation Stress Test" if language == "en" else "估值压力测试",
            "language": language,
            "content": build_valuation_section(profile, report_data, language),
            "locked_data_refs": ["valuation_snapshot", "asset_profile.valuation_method_fit"],
            "editable": True,
        },
        {
            "block_id": "next_checks",
            "block_type": "EDITABLE_NEXT_CHECK",
            "section_name": "Next Research Steps" if language == "en" else "下一步研究清单",
            "language": language,
            "content": build_next_checks(profile, language),
            "locked_data_refs": ["asset_profile.data_deficit_flags", "asset_profile.business_model_clues"],
            "editable": True,
        },
        {
            "block_id": "status_card",
            "block_type": "LOCKED_AUDIT_STATUS",
            "section_name": "Status Card",
            "language": language,
            "content": "",
            "locked_data_refs": ["gate_status", "asset_profile"],
            "editable": False,
        },
    ]
    return blocks


def build_next_checks(profile: AssetProfile, language: str = "en") -> str:
    speculative = profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}
    biotech = profile.secondary_profile == "Biotech-like Screening"
    if language == "zh":
        if speculative:
            checks = [
                "拆业务线收入、毛利率和订单转化，确认增长是否有可执行需求支撑。",
                "计算自由现金流消耗和现金 runway，评估再融资或股权稀释风险。",
                "核查项目进度、产能或交付节奏，以及管理层给出的盈利路径。",
            ]
        elif profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
            checks = [
                "拆 foundry、data center、client computing 的收入、毛利率和 operating income。",
                "核查 capex roadmap、free cash flow bridge、补贴 / 政府资金和制造利用率。",
                "跟踪 process node 进度、AI accelerator 竞争力、库存压力和毛利率 bridge。",
            ]
        elif biotech:
            checks = [
                "核查 pipeline：主要项目、适应症、临床阶段和下一次数据 / 里程碑时间。",
                "核查 FDA / EMA 节点、试验数据披露节奏和审批风险。",
                "核查现金余额、R&D burn、经营现金流、未来 12-24 个月融资需求和稀释风险。",
                "核查合作收入质量：合作方、预付款、里程碑付款和收入持续性。",
                "核查稀释风险：股本变化、SBC、ATM / 股权融资历史和债务融资可能性。",
            ]
        elif profile.primary_profile == "Unknown / Data-Limited Screening":
            checks = [
                "先确认公司靠什么赚钱，以及核心资产是什么。",
                "列出该行业最重要的 3 个经营指标。",
                "判断当前估值方法是否适用；不适用时降级为初筛。",
            ]
        elif profile.primary_profile == "REIT-like Screening":
            checks = ["核查 FFO、AFFO、出租率和 same-store NOI。", "核查债务期限、利息成本和物业类型。", "核查 cap rate、租金价差和租户集中度。"]
        elif profile.primary_profile == "Insurance-like Screening":
            checks = ["核查 combined ratio、underwriting margin 和 reserve adequacy。", "核查 float、investment income 和资产久期。", "核查巨灾暴露、保费增长质量和赔付通胀。"]
        elif profile.primary_profile == "Utilities / Infrastructure":
            checks = ["核查 regulated asset base、allowed ROE 和 rate case。", "核查 capex plan、debt cost 和 dividend coverage。", "核查监管环境和利率敏感性。"]
        elif profile.primary_profile == "Shipping / Airlines / Transport":
            checks = ["核查 freight rate / yield、load factor / utilization。", "核查 fuel cost、fleet age、orderbook 和 leverage。", "核查正常化利润和周期位置。"]
        elif profile.primary_profile == "Consumer / Retail":
            checks = ["核查 same-store sales、traffic、ticket 和 store count。", "核查 inventory、gross margin 和促销压力。", "核查品牌定价权和门店经济。"]
        elif profile.primary_profile == "Financials":
            checks = [
                "核查 ROE、P/B、净息差、信用损失和资本充足率。",
                "确认资产质量和融资成本是否恶化。",
                "不要用普通工业公司自由现金流框架替代金融框架。",
            ]
        elif profile.primary_profile == "Cyclical":
            checks = [
                "估算正常化利润，而不是只看当前 PE。",
                "检查周期位置、商品价格或订单周期。",
                "测试 through-cycle 资产负债表韧性。",
            ]
        else:
            checks = [
                "拆业务线收入和利润贡献。",
                "拆每股收益增长来自净利润还是股本减少。",
                "测试市盈率和自由现金流收益率对利润率变化的敏感性。",
            ]
        return "## 13. 下一步研究清单\n\n" + "\n".join(f"{idx}. {item}" for idx, item in enumerate(checks, 1))
    if speculative:
        checks = [
            "Split business-line revenue, gross margin, and order conversion to test whether growth is executable.",
            "Calculate FCF burn and cash runway; assess refinancing or dilution risk.",
            "Track project progress, production cadence, backlog conversion, and path to profitability.",
        ]
    elif profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
        checks = [
            "Split foundry, data center, and client computing revenue, gross margin, and operating income.",
            "Verify capex roadmap, free cash flow bridge, subsidies / government funding, and manufacturing utilization.",
            "Track process-node progress, AI accelerator competitiveness, inventory pressure, and gross-margin bridge.",
        ]
    elif biotech:
        checks = [
            "Verify pipeline: lead programs, indications, trial stage, and next data / milestone timing.",
            "Verify FDA / EMA milestones, trial disclosure cadence, and regulatory risk.",
            "Verify cash balance, R&D burn, operating cash flow, 12-24 month financing need, and dilution risk.",
            "Verify partnership revenue quality: partner, upfront payments, milestone payments, and durability.",
            "Verify dilution risk: share-count change, SBC, ATM or equity issuance history, and debt financing risk.",
        ]
    elif profile.primary_profile == "Unknown / Data-Limited Screening":
        checks = [
            "Identify how the company makes money and what its core assets are.",
            "List the three most important industry operating metrics.",
            "Decide which valuation method fits before interpreting valuation multiples.",
        ]
    elif profile.primary_profile == "REIT-like Screening":
        checks = ["Verify FFO, AFFO, occupancy, and same-store NOI.", "Check debt maturity, interest cost, and property type.", "Review cap rates, rent spread, and tenant concentration."]
    elif profile.primary_profile == "Insurance-like Screening":
        checks = ["Verify combined ratio, underwriting margin, and reserve adequacy.", "Check float, investment income, and asset duration.", "Review catastrophe exposure, premium growth quality, and claims inflation."]
    elif profile.primary_profile == "Utilities / Infrastructure":
        checks = ["Verify regulated asset base, allowed ROE, and rate cases.", "Check capex plan, debt cost, and dividend coverage.", "Review regulatory environment and interest-rate sensitivity."]
    elif profile.primary_profile == "Shipping / Airlines / Transport":
        checks = ["Verify freight rate / yield and load factor / utilization.", "Check fuel cost, fleet age, orderbook, and leverage.", "Estimate normalized earnings and cycle position."]
    elif profile.primary_profile == "Consumer / Retail":
        checks = ["Verify same-store sales, traffic, ticket, and store count.", "Check inventory, gross margin, and promotion pressure.", "Review brand pricing power and store economics."]
    elif profile.primary_profile == "Insurance-like Screening":
        suspected = "Insurance / reinsurance underwriting business"
        why = (
            "Insurance companies require underwriting and reserve metrics. Ordinary industrial FCF, EV/EBITDA, "
            "or a simple PE screen does not explain combined ratio, float economics, reserve adequacy, investment income, or catastrophe exposure."
        )
        can_judge = ["basic price and benchmark behavior", "basic valuation snapshot", "whether insurance-sector data is missing"]
        cannot_judge = ["underwriting profitability", "reserve adequacy", "catastrophe exposure", "float quality", "claims inflation"]
        missing = profile.data_deficit_flags or ["combined ratio", "underwriting margin", "reserve adequacy", "investment income", "catastrophe exposure"]
        extension = "Insurance Research Profile"
    elif profile.primary_profile == "Financials":
        checks = [
            "Verify ROE, P/B, NIM, credit losses, and capital adequacy.",
            "Check whether asset quality or funding cost is deteriorating.",
            "Avoid substituting industrial FCF logic for financial-sector analysis.",
        ]
    elif profile.primary_profile == "Cyclical":
        checks = [
            "Estimate normalized earnings instead of relying on current PE.",
            "Check cycle position, commodity price, demand, or order-cycle exposure.",
            "Stress-test through-cycle balance-sheet resilience.",
        ]
    else:
        checks = [
            "Split business segments and margin contribution.",
            "Split EPS growth between net income growth and share-count reduction.",
            "Test PE / FCF yield sensitivity against margin and buyback durability.",
        ]
    return "## 13. Next Research Steps\n\n" + "\n".join(f"{idx}. {item}" for idx, item in enumerate(checks, 1))


LOCKED_BLOCK_TYPES = {
    "LOCKED_METRIC",
    "LOCKED_COMPUTED_TABLE",
    "LOCKED_AUDIT_STATUS",
    "LOCKED_CHART_PATH",
    "LOCKED_PRICE_DATA",
    "LOCKED_SCENARIO_ASSUMPTION",
}


def apply_interpretation_patch(
    blocks: list[dict[str, Any]],
    profile: AssetProfile,
    failure_reasons: list[dict[str, Any]] | None = None,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    patched = []
    patch_records = []
    material_count = 0
    fallback_count = 0
    failure_reasons = failure_reasons or []
    failures_by_section = {
        section
        for failure in failure_reasons
        for section in failure.get("affected_sections", [])
    }
    for block in blocks:
        block_copy = dict(block)
        old_text = block_copy["content"]
        if not block_copy.get("editable", False) or block_copy["block_type"] in LOCKED_BLOCK_TYPES:
            patched.append(block_copy)
            continue
        new_text = old_text
        reason = ""
        if "interpretation" in failures_by_section or profile.primary_profile in {"Speculative Growth", "Unprofitable Growth", "Unknown / Data-Limited Screening"}:
            if block_copy["block_id"] == "one_line_verdict":
                reason = "Align one-line verdict with asset profile."
            elif block_copy["block_id"] == "core_view":
                reason = "Align core view with thesis spine."
            elif block_copy["block_id"] in {"battle_card", "key_questions", "valuation", "next_checks"}:
                reason = "Use asset-profile-specific interpretation block."
        if profile.framework_coverage_level != "FULL" and block_copy["block_id"] == "core_view":
            concept_text = ", ".join(profile.dominant_metric_set[:4]) or profile.primary_profile
            boundary_note = (
                f"修正提示：当前研究框架不是完整覆盖，必须先核查 {concept_text}，才能把主线当作证据。"
                if block_copy.get("language") == "zh"
                else f"Patch note: framework coverage is not full; verify {concept_text} before treating the thesis as evidence."
            )
            if boundary_note not in new_text:
                new_text = f"{new_text}\n\n{boundary_note}"
                reason = "Expose partial framework coverage in the final interpretation."
        if new_text != old_text or reason:
            block_copy["content"] = new_text
            old_norm = old_text.strip()
            new_norm = new_text.strip()
            material = old_norm != new_norm
            fallback_patch = material and profile.framework_coverage_level in {"SCREENING_ONLY", "UNKNOWN"}
            if fallback_patch:
                materiality_label = "FALLBACK_PATCH_APPLIED"
                fallback_count += 1
            elif material:
                materiality_label = "MATERIAL_PATCH_APPLIED"
                material_count += 1
            else:
                materiality_label = "CONFIRMED_NO_CHANGE"
            patch_records.append(
                {
                    "block_id": block_copy["block_id"],
                    "section_name": block_copy["section_name"],
                    "old_text": old_text,
                    "new_text": new_text,
                    "materiality": materiality_label,
                    "material_change": material,
                    "reason": reason or "Profile-specific block selected.",
                    "data_refs_used": block_copy.get("locked_data_refs", []),
                    "derived_claims": [],
                    "derivation_formula": "",
                    "fixed_failure_ids": [failure.get("failure_id") for failure in failure_reasons],
                    "patch_attempt_id": 1,
                }
            )
        patched.append(block_copy)
    material_records = material_count + fallback_count
    return patched, {
        "patch_status": "APPLIED" if material_records else "NOT_NEEDED",
        "patch_materiality_status": "MATERIAL_PATCH_APPLIED" if material_count else "FALLBACK_PATCH_APPLIED" if fallback_count else "NO_MATERIAL_CHANGE",
        "patch_attempts": 1 if patch_records else 0,
        "material_patch_count": material_count,
        "fallback_patch_count": fallback_count,
        "patch_records": patch_records,
    }


def write_patch_artifacts(out_dir: Path, patch_log: dict[str, Any], patched_blocks: list[dict[str, Any]]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "correction_patch.json").write_text(json.dumps(patch_log, ensure_ascii=False, indent=2), encoding="utf-8")
    (out_dir / "patched_report_blocks.json").write_text(json.dumps(patched_blocks, ensure_ascii=False, indent=2), encoding="utf-8")
    lines = ["# Patch Diff Log", ""]
    for record in patch_log.get("patch_records", []):
        lines.extend(
            [
                f"## {record['block_id']}",
                "",
                f"Section: {record['section_name']}",
                f"Materiality: {record.get('materiality', 'CONFIRMED_NO_CHANGE')}",
                "",
                "### Old Text",
                "",
                record["old_text"] or "_empty_",
                "",
                "### New Text",
                "",
                record["new_text"] or "_empty_",
                "",
                f"Reason: {record['reason']}",
                f"Data refs used: {record['data_refs_used']}",
                "",
            ]
        )
    if not patch_log.get("patch_records"):
        lines.append("No interpretation patch was required.")
    (out_dir / "patch_diff_log.md").write_text("\n".join(lines), encoding="utf-8")


def build_asset_aware_ai_correction_log(profile: AssetProfile, ticker: str, benchmark: str, language: str = "en") -> dict[str, Any]:
    speculative = profile.primary_profile in {"Speculative Growth", "Unprofitable Growth"}
    semiconductor = profile.primary_profile == "Capital-Intensive Semiconductor Turnaround"
    unknown = profile.primary_profile == "Unknown / Data-Limited Screening"
    if language == "zh":
        if speculative:
            corrections = [
                {
                    "section": "估值压力测试",
                    "original_issue": "未盈利成长股不能用成熟公司盈利倍数框架解释。",
                    "suggested_revision": "改用收入倍数、现金消耗、现金 runway、稀释和盈利路径来解释估值压力。",
                    "reason": "资产画像显示盈利尚未建立，盈利倍数不是核心框架。",
                    "requires_data_verification": False,
                    "severity": "HIGH",
                    "evidence_boundary": "FROM_PAYLOAD",
                },
                {
                    "section": "业务线拆解",
                    "original_issue": "[SEGMENT_DATA_MISSING]",
                    "suggested_revision": "业务线数据缺失。增长质量、订单转化和盈利路径必须查最新 10-K、10-Q 或投资者材料。",
                    "reason": "没有业务线和订单数据，不能把增长假设当证据。",
                    "requires_data_verification": True,
                    "severity": "HIGH",
                    "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
                },
            ]
            unanswered = ["收入增长是否能转成毛利率改善？", "现金 runway 是否足够支撑当前烧钱？", "订单或 backlog 是否能兑现成收入？"]
            stance = f"这不是买卖建议，而是研究立场。\n\n{ticker} 应按投机成长股处理。核心不是现在赚了多少钱，而是增长质量、现金消耗、融资稀释和盈利路径是否同时改善。"
            checks = ["拆业务线收入、毛利率和订单转化。", "计算自由现金流消耗和现金 runway。", "核查 backlog、项目进度和盈利路径。"]
        elif semiconductor:
            corrections = [
                {
                    "section": "资产画像",
                    "original_issue": "半导体制造转型公司不能降级成泛泛 technology screening-only。",
                    "suggested_revision": "使用 Capital-Intensive Semiconductor Turnaround 框架，围绕 foundry、capex、毛利率修复、制程和自由现金流压力组织正文。",
                    "reason": "公司简介和财务特征指向资本开支重的半导体制造转型，而不是普通成熟复利或未知行业。",
                    "requires_data_verification": False,
                    "severity": "HIGH",
                    "evidence_boundary": "FROM_PAYLOAD",
                },
                {
                    "section": "数据缺口",
                    "original_issue": "[SEGMENT_DATA_MISSING]",
                    "suggested_revision": "列出 foundry revenue / margin、data center segment、capex roadmap、process node、manufacturing utilization 和 gross margin bridge 缺口。",
                    "reason": "这些数据决定制造转型、毛利率修复和估值框架是否成立。",
                    "requires_data_verification": True,
                    "severity": "HIGH",
                    "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
                },
            ]
            unanswered = ["foundry 业务是否真的改善？", "资本开支是否继续压制自由现金流？", "毛利率修复有没有制程和产能利用率证据？"]
            stance = f"这不是买卖建议，而是研究立场。\n\n{ticker} 应按半导体制造转型案例处理。下一步不是讨论 PE 低不低，而是验证 foundry、制程、数据中心竞争、毛利率修复和自由现金流压力。"
            checks = ["拆 foundry / data center / client computing。", "核查 capex roadmap 和 free cash flow bridge。", "跟踪 process node、制造利用率、AI accelerator 竞争和毛利率 bridge。"]
        elif unknown:
            corrections = [
                {
                    "section": "资产画像",
                    "original_issue": "[FIELD_MISSING_PROVIDER]",
                    "suggested_revision": "降级为仅限初筛，不得套成熟复利、投机成长、金融或周期模板。",
                    "reason": "框架覆盖不足时，完整投研结论会误导用户。",
                    "requires_data_verification": True,
                    "severity": "HIGH",
                    "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
                }
            ]
            unanswered = ["公司到底靠什么赚钱？", "最重要的行业指标是什么？", "哪种估值方法适用？"]
            stance = f"这不是买卖建议，而是研究立场。\n\n{ticker} 当前只能作为数据初筛。系统还不能可靠选择完整行业框架。"
            checks = ["确认业务模式和核心资产。", "列出行业专属经营指标。", "决定哪些估值方法适用。"]
        else:
            corrections = [
                {
                    "section": "估值压力测试",
                    "original_issue": "成熟公司估值讨论需要说明市场正在支付什么稳定性。",
                    "suggested_revision": "围绕利润率、自由现金流、业务结构和资本回报解释估值压力。",
                    "reason": "资产画像支持成熟复利框架。",
                    "requires_data_verification": False,
                    "severity": "MEDIUM",
                    "evidence_boundary": "FROM_PAYLOAD",
                }
            ]
            unanswered = ["业务结构是否还能支撑利润率？", "每股收益增长来自净利润还是股本减少？", "为什么不直接买基准？"]
            stance = f"这不是买卖建议，而是研究立场。\n\n{ticker} 的核心是成熟现金流质量和估值纪律。"
            checks = ["拆业务线收入和利润贡献。", "拆 EPS 增长来源。", "测试估值对利润率和现金流变化的敏感性。"]
    else:
        if speculative:
            corrections = [
                {
                    "section": "Valuation Stress Test",
                    "original_issue": "Unprofitable growth should not be explained with mature earnings-multiple logic.",
                    "suggested_revision": "Use sales multiples, burn, runway, dilution, and path-to-profitability framing.",
                    "reason": "The asset profile does not support a mature profitable-company valuation frame.",
                    "requires_data_verification": False,
                    "severity": "HIGH",
                    "evidence_boundary": "FROM_PAYLOAD",
                },
                {
                    "section": "Segment Revenue Gap",
                    "original_issue": "[SEGMENT_DATA_MISSING]",
                    "suggested_revision": "Segment data is missing. Growth quality and order conversion require latest filings or investor materials.",
                    "reason": "Without segment or backlog data, growth cannot be treated as executable demand evidence.",
                    "requires_data_verification": True,
                    "severity": "HIGH",
                    "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
                },
            ]
            unanswered = ["Can growth convert into margin improvement?", "How long can cash support burn?", "Do orders or backlog convert into revenue?"]
            stance = f"This is not a buy/sell recommendation. It is a research stance.\n\n{ticker} should be treated as a speculative growth case. The research burden is growth quality, burn, runway, dilution, and path to profitability."
            checks = ["Split business-line revenue, gross margin, and order conversion.", "Calculate FCF burn and cash runway.", "Verify backlog, project progress, and path to profitability."]
        elif semiconductor:
            corrections = [
                {
                    "section": "Asset Profile",
                    "original_issue": "A semiconductor manufacturing turnaround should not be downgraded into generic technology screening-only.",
                    "suggested_revision": "Use the Capital-Intensive Semiconductor Turnaround frame and organize the report around foundry execution, capex pressure, gross-margin recovery, process roadmap, and free cash flow pressure.",
                    "reason": "Business clues and financial pattern point to capital-intensive semiconductor transition, not a plain mature or unknown template.",
                    "requires_data_verification": False,
                    "severity": "HIGH",
                    "evidence_boundary": "FROM_PAYLOAD",
                },
                {
                    "section": "Data Deficits",
                    "original_issue": "[SEGMENT_DATA_MISSING]",
                    "suggested_revision": "Expose missing foundry revenue / margin, data center segment trend, capex roadmap, process-node progress, manufacturing utilization, and gross-margin bridge.",
                    "reason": "Those missing metrics determine whether the turnaround, margin recovery, and valuation frame are supportable.",
                    "requires_data_verification": True,
                    "severity": "HIGH",
                    "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
                },
            ]
            unanswered = ["Is foundry progress becoming operating evidence?", "Does capex keep suppressing free cash flow?", "Is gross-margin recovery supported by process and utilization data?"]
            stance = f"This is not a buy/sell recommendation. It is a research stance.\n\n{ticker} should be treated as a semiconductor manufacturing turnaround. The next step is not to debate whether PE is low; it is to verify foundry execution, process progress, data-center competitiveness, gross-margin recovery, and free cash flow pressure."
            checks = ["Split foundry / data center / client computing.", "Verify capex roadmap and free cash flow bridge.", "Track process-node progress, manufacturing utilization, AI accelerator competitiveness, and gross-margin bridge."]
        elif unknown:
            corrections = [
                {
                    "section": "Asset Profile",
                    "original_issue": "[FIELD_MISSING_PROVIDER]",
                    "suggested_revision": "Downgrade to screening-only and avoid mature, speculative, financial, or cyclical thesis claims.",
                    "reason": "Low framework coverage makes a complete thesis unsafe.",
                    "requires_data_verification": True,
                    "severity": "HIGH",
                    "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
                }
            ]
            unanswered = ["How does the company make money?", "Which industry metrics matter most?", "Which valuation method fits?"]
            stance = f"This is not a buy/sell recommendation. It is a research stance.\n\n{ticker} is screening-only until the business model and industry framework are verified."
            checks = ["Identify the revenue engine and core assets.", "List the industry-specific operating metrics.", "Decide which valuation method applies."]
        else:
            corrections = [
                {
                    "section": "Valuation Stress Test",
                    "original_issue": "Mature-company valuation needs explicit durability assumptions.",
                    "suggested_revision": "Frame valuation around margin durability, FCF stability, business mix, and capital returns.",
                    "reason": "The asset profile supports mature compounder logic.",
                    "requires_data_verification": False,
                    "severity": "MEDIUM",
                    "evidence_boundary": "FROM_PAYLOAD",
                }
            ]
            unanswered = ["Can business mix keep supporting margins?", "How much EPS growth comes from buybacks?", f"Why own {ticker} over {benchmark}?"]
            stance = f"This is not a buy/sell recommendation. It is a research stance.\n\n{ticker} is a mature cash-flow quality case where valuation discipline matters."
            checks = ["Split segment revenue and margin contribution.", "Split EPS growth between net income and share-count reduction.", "Stress-test valuation against margin and FCF durability."]

    answerability = [
        {
            "question": question,
            "status": "PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION" if idx < 2 else "NOT_ANSWERABLE_FROM_CURRENT_DATA" if "next year" in question.lower() else "PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION",
            "short_answer": "Current report data frames the issue, but primary-source verification is required.",
            "evidence_used": "asset_profile and deterministic report payload",
            "what_to_verify_next": checks[min(idx, len(checks) - 1)],
            "evidence_boundary": "NEEDS_EXTERNAL_VERIFICATION",
        }
        for idx, question in enumerate(unanswered[:3])
    ]
    return {
        "language": language,
        "max_correction_passes": 1,
        "report_corrections": corrections,
        "unanswered_questions": unanswered[:5],
        "answerability_classification": answerability,
        "research_stance": stance,
        "next_3_checks": checks[:3],
        "status": STATUS_PASS,
    }


def company_specificity_status(profile: AssetProfile, logic_report: dict[str, Any], patch_log: dict[str, Any]) -> dict[str, Any]:
    failures = logic_report.get("failure_reasons", [])
    if failures and patch_log.get("patch_status") != "APPLIED":
        status = STATUS_FAIL
    elif profile.framework_coverage_level in {"SCREENING_ONLY", "UNKNOWN"}:
        status = STATUS_WARNING
    elif failures:
        status = STATUS_WARNING
    else:
        status = STATUS_PASS
    return {
        "COMPANY_SPECIFICITY_STATUS": status,
        "framework_coverage_level": profile.framework_coverage_level,
        "failure_reasons": failures,
        "patch_status": patch_log.get("patch_status"),
    }


def fallback_status(profile: AssetProfile) -> str:
    if profile.fallback_used_count > 2:
        return STATUS_WARNING_DEGRADED
    if profile.fallback_used_count > 0:
        return "USED"
    return "NONE"


def overall_status_v43(statuses: dict[str, str], fallback_count: int = 0) -> str:
    if STATUS_FAIL in statuses.values():
        return STATUS_UNVERIFIED
    if fallback_count > 2:
        return STATUS_WARNING_DEGRADED
    if STATUS_WARNING in statuses.values() or fallback_count > 0:
        return STATUS_WARNING
    return STATUS_PASS


def rollup_data_verification_status(statuses: dict[str, str]) -> str:
    data_keys = ["DATA_AUDIT_STATUS", "RISK_METHOD_STATUS", "PRICE_LABEL_CHECK_STATUS"]
    values = [statuses.get(key, STATUS_PASS) for key in data_keys]
    if STATUS_FAIL in values:
        return STATUS_FAIL
    if STATUS_WARNING in values or STATUS_WARNING_DEGRADED in values:
        return STATUS_WARNING
    return STATUS_PASS


def rollup_thesis_verification_status(statuses: dict[str, str]) -> str:
    thesis_keys = ["AI_ANALYST_REVIEW_STATUS", "LANGUAGE_LINT_STATUS", "LIFECYCLE_LOGIC_STATUS", "COMPANY_SPECIFICITY_STATUS"]
    values = [statuses.get(key, STATUS_PASS) for key in thesis_keys]
    coverage = statuses.get("FRAMEWORK_COVERAGE_LEVEL", "FULL")
    if STATUS_FAIL in values:
        return STATUS_FAIL
    if coverage in {"UNKNOWN", "SCREENING_ONLY"}:
        return STATUS_UNVERIFIED
    if STATUS_WARNING in values or STATUS_WARNING_DEGRADED in values or coverage == "PARTIAL":
        return STATUS_WARNING
    return STATUS_PASS


def overall_status_from_verification(data_status: str, thesis_status: str) -> str:
    if data_status == STATUS_FAIL or thesis_status in {STATUS_UNVERIFIED, STATUS_FAIL}:
        return STATUS_UNVERIFIED
    if data_status == STATUS_WARNING or thesis_status == STATUS_WARNING:
        return STATUS_WARNING
    return STATUS_PASS


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


def write_framework_gap_analysis(path: Path, profile: AssetProfile) -> None:
    if profile.primary_profile == "Hybrid AI Semiconductor Compounder":
        suspected = "AI semiconductor / data-center growth compounder"
        why = (
            "Generic semiconductor or mature-compounder framing is too thin for AI accelerator platforms. "
            "The report must test AI data-center revenue, gross-margin sustainability, supply and capacity limits, "
            "customer concentration, hyperscaler capex cyclicality, export controls, networking / accelerator ecosystem strength, "
            "and valuation premium risk."
        )
        can_judge = [
            "basic price and benchmark behavior",
            "high-level profitability and revenue growth",
            "whether the company deserves a growth-premium research frame",
            "which AI-semiconductor metrics must be manually verified",
        ]
        cannot_judge = [
            "AI data-center segment durability without segment revenue and margin details",
            "customer concentration risk without customer / hyperscaler disclosure",
            "supply capacity and lead-time constraints without operational disclosures",
            "export-control exposure without region and product detail",
            "CUDA / platform ecosystem durability without product and developer evidence",
        ]
        missing = profile.data_deficit_flags or [
            "AI data center revenue and margin",
            "gross margin sustainability bridge",
            "supply / capacity constraint",
            "customer concentration",
            "hyperscaler capex cycle",
            "export control exposure",
            "networking / accelerator ecosystem",
            "CUDA / platform ecosystem evidence",
            "valuation premium sensitivity",
        ]
        extension = "AI Semiconductor / Data Center Growth Compounder Research Profile"
    elif profile.primary_profile == "Capital-Intensive Semiconductor Turnaround":
        suspected = "Semiconductor / integrated device manufacturing / foundry turnaround"
        why = (
            "Ordinary mature-compounder logic does not capture capex intensity, manufacturing transition, "
            "process-node execution, foundry losses, data-center competitiveness, and gross-margin recovery."
        )
        can_judge = [
            "basic price and benchmark behavior",
            "high-level profitability pressure",
            "whether public provider data is enough for first-pass screening",
            "which semiconductor-specific metrics must be manually verified",
        ]
        cannot_judge = [
            "foundry turnaround quality without segment revenue and losses",
            "process-node execution without roadmap and yield / utilization data",
            "data-center or AI accelerator competitiveness without segment details",
            "gross-margin recovery quality without a gross-margin bridge",
        ]
        missing = profile.data_deficit_flags
        extension = "Semiconductor / Manufacturing Turnaround Research Profile"
    elif "Biotech" in profile.secondary_profile or profile.suggested_framework_extension == "Biotech Research Profile":
        suspected = "Clinical-stage biotechnology / pharmaceutical development"
        why = (
            "Traditional PE, PS, and ordinary FCF quality frameworks do not capture pipeline stage, "
            "trial milestones, regulatory approval risk, R&D burn, cash runway, and dilution risk."
        )
        can_judge = ["cash balance and burn indicators", "basic price volatility", "data availability", "financing pressure"]
        cannot_judge = ["pipeline quality", "clinical trial success probability", "FDA approval risk", "drug market size", "competitive medical profile"]
        missing = profile.data_deficit_flags or ["pipeline stage", "clinical trial status", "FDA milestone timeline", "R&D runway", "drug candidate concentration"]
        extension = "Biotech Research Profile"
    elif "REIT" in profile.secondary_profile or profile.suggested_framework_extension == "REIT Research Profile":
        suspected = "REIT / real-estate income vehicle"
        why = (
            "Ordinary net income, PE, and industrial FCF metrics can mislead for REIT-like companies. "
            "Funds from operations, occupancy, same-property NOI, lease duration, debt maturity, and interest-rate sensitivity matter more."
        )
        can_judge = ["basic price and benchmark behavior", "basic balance-sheet availability", "public valuation snapshot"]
        cannot_judge = ["FFO / AFFO quality", "property-level NOI", "lease rollover risk", "debt maturity wall", "rate sensitivity"]
        missing = profile.data_deficit_flags or ["FFO / AFFO", "occupancy", "same-property NOI", "lease maturity", "debt maturity schedule"]
        extension = "REIT Research Profile"
    elif profile.primary_profile == "Financials":
        suspected = "Financial services / banking / insurance / brokerage"
        why = (
            "Financial companies require sector metrics that public generic financial statements may not expose. "
            "Ordinary industrial FCF, net debt / EBITDA, and EV/EBITDA do not explain asset quality, underwriting, credit, funding, or capital adequacy."
        )
        can_judge = ["basic valuation snapshot", "price versus benchmark", "whether financial-sector data is missing"]
        cannot_judge = ["NIM quality", "credit losses", "capital adequacy", "insurance underwriting quality", "reserve adequacy"]
        missing = profile.data_deficit_flags or ["NIM", "credit losses", "capital ratio", "underwriting ratio", "reserve adequacy"]
        extension = "Financial Research Profile"
    elif profile.primary_profile == "Utilities / Infrastructure":
        suspected = "Utilities / regulated infrastructure"
        why = "Utilities and infrastructure companies need regulated-return metrics that ordinary growth or industrial screens do not capture."
        can_judge = ["basic price behavior", "balance-sheet availability", "high-level cash-flow pressure"]
        cannot_judge = ["allowed ROE", "rate base quality", "rate-case outcome", "dividend coverage through the capex cycle"]
        missing = profile.data_deficit_flags or ["regulated asset base", "allowed ROE", "rate case", "dividend coverage"]
        extension = "Utilities / Infrastructure Research Profile"
    elif profile.primary_profile == "Shipping / Airlines / Transport":
        suspected = "Transport / shipping / airlines cycle-sensitive company"
        why = "Transport economics depend on rate/yield cycles, utilization, fuel cost, fleet age, orderbook, leverage, and normalized earnings."
        can_judge = ["basic price and benchmark behavior", "current public financial direction", "whether transport-cycle data is missing"]
        cannot_judge = ["normalized earnings", "cycle position", "fleet economics", "fuel sensitivity", "orderbook pressure"]
        missing = profile.data_deficit_flags or ["freight rate / yield", "load factor / utilization", "fuel cost", "fleet age", "orderbook"]
        extension = "Transport / Shipping / Airlines Research Profile"
    elif profile.primary_profile == "Consumer / Retail":
        suspected = "Consumer / retail / restaurant operating model"
        why = "Consumer and retail companies require store economics and brand metrics; total revenue alone can hide traffic, price, inventory, and margin pressure."
        can_judge = ["basic revenue and margin direction", "price versus benchmark", "provider valuation snapshot"]
        cannot_judge = ["same-store sales quality", "traffic versus ticket split", "inventory health", "store economics", "brand pricing power"]
        missing = profile.data_deficit_flags or ["same-store sales", "traffic", "ticket", "inventory", "store count"]
        extension = "Consumer / Retail Research Profile"
    else:
        suspected = profile.sector_type
        why = profile.report_thesis_spine
        can_judge = [
            "basic price and benchmark behavior",
            "basic financial availability",
            "high-level valuation snapshot",
            "data gaps and manual verification needs",
        ]
        cannot_judge = [
            "industry-specific operating quality when specialized metrics are missing",
            "full company-specific thesis when framework confidence is low",
            "future stock performance or target price",
        ]
        missing = profile.data_deficit_flags
        extension = profile.suggested_framework_extension or "No immediate extension suggested."
    content = f"""# Framework Gap Analysis

## Suspected Industry

{suspected}

## Coverage Level

{profile.framework_coverage_level}

## Why Existing Framework Is Not Enough

{why}

## What This Report Can Judge

{chr(10).join(f"- {item}" for item in can_judge)}

## What This Report Cannot Judge

{chr(10).join(f"- {item}" for item in cannot_judge)}

## Missing Metrics

{chr(10).join(f"- {item}" for item in missing) if missing else "- No major framework-specific missing metric was detected automatically."}

## Suggested Framework Extension

{extension}
"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def write_lifecycle_logic_report(path: Path, lifecycle_report: dict[str, Any], profile: AssetProfile) -> None:
    failures = lifecycle_report.get("failure_reasons", [])
    failed_checks = failures or [
        {
            "failure_id": "NONE",
            "failed_check": "No lifecycle logic failure detected.",
            "affected_sections": [],
            "forbidden_terms": [],
            "expected_profile_logic": profile.primary_profile,
            "severity": "INFO",
        }
    ]
    content = f"""# Lifecycle Logic Report

## Status

{lifecycle_report.get("status", STATUS_PASS)}

## Failed Checks

{chr(10).join(f"- {item.get('failure_id')}: {item.get('failed_check')} ({item.get('severity')})" for item in failed_checks)}

## Why Failed

{("The report contained wording or valuation logic that conflicts with the detected asset profile." if failures else "No profile/body conflict was detected.")}

## Affected Sections

{chr(10).join(f"- {section}" for item in failures for section in item.get("affected_sections", [])) if failures else "- None"}

## Profile Conflict

- Expected profile logic: {profile.primary_profile}
- Thesis spine: {profile.report_thesis_spine}
- Invalid metric set: {", ".join(profile.invalid_metric_set) if profile.invalid_metric_set else "None"}

## Valuation Method Conflict

- Expected valuation frame: {profile.valuation_method_fit}
- Status: {"Review required" if failures else "No conflict detected automatically"}

## Template Contamination Check

{chr(10).join(f"- Forbidden term hit: {term}" for item in failures for term in item.get("forbidden_terms", [])) if failures else "- No forbidden template terms detected."}

## Missing Data Check

{chr(10).join(f"- {item}" for item in profile.data_deficit_flags) if profile.data_deficit_flags else "- No profile-specific missing data flags were registered."}

## Suggested Fix

Use the asset-profile-specific thesis spine, valuation frame, Q&A, red flags, and next checks. Do not reuse a mature, speculative, financial, or cyclical interpretation unless the profile supports it.
"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def write_improvement_suggestions(path: Path, profile: AssetProfile) -> None:
    suggestions = [
        f"Suggested framework: {profile.suggested_framework_extension or profile.primary_profile}",
        f"Add metrics for: {', '.join(profile.dominant_metric_set)}",
        f"Reject invalid metrics: {', '.join(profile.invalid_metric_set)}",
        "Add profile-specific Q&A and next-check tests.",
    ]
    content = "# Improvement Suggestions\n\n" + "\n".join(f"- {item}" for item in suggestions)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def write_regression_test_suggestions(path: Path, profile: AssetProfile) -> None:
    slug = profile.primary_profile.lower().replace(" ", "_").replace("/", "_").replace("-", "_")
    content = f"""# Regression Test Suggestions

## Suggested Test 1

Name:
test_{slug}_payload_uses_matching_research_frame

Purpose:
Ensure companies with this profile do not receive another profile's thesis language.

Expected:
- primary_profile = {profile.primary_profile}
- valuation method matches `{profile.valuation_method_fit}`
- invalid metric set is not used as core interpretation
- company specificity status is PASS or explicit WARNING

## Suggested Test 2

Name:
test_{slug}_data_deficits_are_visible

Purpose:
Ensure missing sector-specific metrics are exposed with manual verification steps.
"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def write_system_self_review(path: Path, data: dict[str, Any]) -> None:
    profile = AssetProfile(**data["asset_profile"])
    summary = {
        "ticker": data.get("ticker"),
        "run_id": data.get("run_id"),
        "detected_asset_profile": profile.primary_profile,
        "profile_confidence": profile.thesis_spine_confidence,
        "framework_coverage_level": profile.framework_coverage_level,
        "selected_research_framework": profile.primary_profile,
        "why_this_framework": profile.dominant_metric_set,
        "supporting_evidence": profile.business_model_clues,
        "conflicting_evidence": profile.invalid_metric_set,
        "missing_sector_specific_metrics": profile.data_deficit_flags,
        "template_contamination_risk": "LOW" if data.get("lifecycle_logic_status") == STATUS_PASS else "HIGH",
        "valuation_method_fit": profile.valuation_method_fit,
        "fallback_used_count": profile.fallback_used_count,
        "fallback_used_sections": profile.fallback_used_sections,
        "ai_patch_applied": data.get("patch_status") == "APPLIED",
        "what_this_report_can_judge": ["first-pass financial screen", "benchmark comparison", "data gaps"],
        "what_this_report_cannot_judge": ["target price", "buy/sell decision", "unprovided sector-specific facts"],
        "recommended_framework_extensions": [profile.suggested_framework_extension] if profile.suggested_framework_extension else [],
        "recommended_tests": [f"test_{profile.primary_profile.lower().replace(' ', '_')}_routing"],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.with_suffix(".json").write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    content = f"""# System Self Review

## Self Review Summary

- Asset profile confidence: {summary['profile_confidence']}
- Framework coverage: {summary['framework_coverage_level']}
- Company-specificity: {data.get('company_specificity_status')}
- Missing sector-specific data: {', '.join(profile.data_deficit_flags) if profile.data_deficit_flags else 'None detected automatically'}
- AI patch applied: {'yes' if summary['ai_patch_applied'] else 'no'}
- Fallback used: {profile.fallback_used_count} section(s)
- Manual verification required: {'yes' if profile.data_deficit_flags or profile.framework_coverage_level != 'FULL' else 'no'}
- Suggested future framework extension: {profile.suggested_framework_extension or 'None'}

## What This Report Can Judge

{chr(10).join(f"- {item}" for item in summary['what_this_report_can_judge'])}

## What This Report Cannot Judge

{chr(10).join(f"- {item}" for item in summary['what_this_report_cannot_judge'])}
"""
    path.write_text(content, encoding="utf-8")


def presentation_gate(run_dir: Path, language: str = "en") -> dict[str, Any]:
    report_dir = run_dir / "report"
    chart_dir = run_dir / "charts"
    required = [
        run_dir / "README.md",
        report_dir,
        chart_dir,
        run_dir / "data",
        run_dir / "audit",
        run_dir / "ai",
        run_dir / "metadata",
        run_dir / "self_review",
    ]
    missing = [str(path.relative_to(run_dir)) for path in required if not path.exists()]
    status = STATUS_FAIL if missing else STATUS_PASS
    return {"PRESENTATION_STATUS": status, "missing": missing}


def organize_report_pack(run_dir: Path, ticker: str) -> dict[str, Path]:
    folders = {
        "report": run_dir / "report",
        "charts": run_dir / "charts",
        "data": run_dir / "data",
        "audit": run_dir / "audit",
        "ai": run_dir / "ai",
        "dashboard": run_dir / "dashboard",
        "metadata": run_dir / "metadata",
        "self_review": run_dir / "self_review",
    }
    for folder in folders.values():
        folder.mkdir(parents=True, exist_ok=True)
    return folders


def write_run_readme(run_dir: Path, ticker: str, benchmark: str, status: dict[str, Any]) -> None:
    content = f"""# {ticker} Research Run

Start here:

1. `report/{ticker}_research_report.md`
2. `report/{ticker}_research_report_cn.md`
3. `metadata/report_status.json`
4. `self_review/system_self_review.md`
5. `ai/patch_diff_log.md`

## Status

- Overall: {status.get('OVERALL_REPORT_STATUS')}
- Asset Profile: {status.get('ASSET_PROFILE')}
- Framework Coverage: {status.get('FRAMEWORK_COVERAGE_LEVEL')}
- Company Specificity: {status.get('COMPANY_SPECIFICITY_STATUS')}
- Patch Status: {status.get('PATCH_STATUS')}
- Patch Materiality: {status.get('PATCH_MATERIALITY_STATUS')}
- Fallback Used: {status.get('FALLBACK_USED_COUNT')}

## Folders

- `report/`: final Markdown reports
- `charts/`: PNG chart outputs
- `data/`: CSV data exports
- `audit/`: data, method, language, lifecycle, and presentation checks
- `ai/`: correction patch and patched block logs
- `dashboard/`: interactive HTML dashboards
- `metadata/`: run metadata and status files
- `self_review/`: automatic system review and improvement suggestions
"""
    (run_dir / "README.md").write_text(content, encoding="utf-8")


def copy_pack_to_latest(run_dir: Path, latest_dir: Path) -> None:
    if latest_dir.exists():
        shutil.rmtree(latest_dir)
    shutil.copytree(run_dir, latest_dir)
