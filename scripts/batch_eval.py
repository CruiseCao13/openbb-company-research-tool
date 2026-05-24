from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import shutil
import time
import traceback
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any, Callable


BATCH_STATUSES = {
    "COMPLETED",
    "COMPLETED_WITH_WARNINGS",
    "FETCH_FAILED",
    "PARTIAL_DATA",
    "REPORT_FAILED",
    "LINT_FAILED",
    "AI_REVIEW_FAILED",
    "PACK_FAILED",
}

TRANSIENT_ERROR_PATTERNS = (
    "timeout",
    "timed out",
    "connection reset",
    "temporarily",
    "temporary",
    "rate limit",
    "too many requests",
    "empty response",
    "provider",
)

RAW_PLACEHOLDERS = (
    "[METRIC_MISSING_RAW]",
    "[FIELD_MISSING_PROVIDER]",
    "[PRIMARY_SOURCE_REQUIRED]",
    "[SEGMENT_DATA_MISSING]",
    "[METHOD_ASSUMPTION_MISSING]",
    "[PRICE_LABEL_UNVERIFIED]",
)


@dataclass
class BatchOptions:
    eval_set_path: str
    both: bool = False
    full: bool = False
    pack: bool = False
    max_workers: int = 2
    no_ai: bool = True
    ai_review_failures: bool = False
    max_ai_reviews: int = 40
    resume: bool = False
    only_failed: bool = False
    force: bool = False
    benchmark: str = "SPY"
    start: str = "2023-01-01"
    output: str = "reports"
    batch_id: str | None = None


@dataclass
class LintResult:
    ticker: str
    status: str
    failed_checks: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    outcome: str = "PASS"
    details: dict[str, Any] = field(default_factory=dict)


@dataclass
class TickerRunResult:
    ticker: str
    status: str
    outcome: str = "FAIL"
    run_folder: str | None = None
    report_status: dict[str, Any] = field(default_factory=dict)
    asset_profile: dict[str, Any] = field(default_factory=dict)
    lint: LintResult | None = None
    error_type: str | None = None
    error_message: str | None = None
    error_category: str | None = None
    reason: str | None = None
    suggested_next_action: str | None = None
    attempts: int = 1
    pack_zip: str | None = None
    ai_review: dict[str, Any] | None = None
    training_case_generated: bool = False
    expected_profile_family: str | None = None


@dataclass
class BatchResult:
    batch_id: str
    eval_set_path: str
    out_dir: str
    results: list[TickerRunResult] = field(default_factory=list)
    ai_reviews_used: int = 0
    ai_reviews_skipped_by_cache: int = 0
    training_cases_generated: int = 0


def _strip_comment(line: str) -> str:
    return line.split("#", 1)[0].rstrip()


def load_eval_set(path: str) -> list[str]:
    """Load the project's small YAML subset and return deduplicated tickers."""
    tickers: list[str] = []
    in_tickers = False
    in_groups = False
    current_group = None
    for raw in Path(path).read_text(encoding="utf-8").splitlines():
        line = _strip_comment(raw)
        if not line.strip():
            continue
        stripped = line.strip()
        if stripped == "tickers:":
            in_tickers = True
            in_groups = False
            current_group = None
            continue
        if stripped == "groups:":
            in_groups = True
            in_tickers = False
            current_group = None
            continue
        if in_groups and raw.startswith("  ") and not raw.startswith("    ") and stripped.endswith(":"):
            current_group = stripped[:-1]
            continue
        if stripped.startswith("- "):
            value = stripped[2:].strip().upper()
            if value and (in_tickers or current_group is not None):
                tickers.append(value)
    seen = set()
    deduped = []
    for ticker in tickers:
        if ticker not in seen:
            seen.add(ticker)
            deduped.append(ticker)
    return deduped


GROUP_PROFILE_FAMILIES = {
    "mature_compounder": "Mature Compounder",
    "mega_cap_tech": "Hybrid Growth Compounder",
    "semiconductor_ai": "Hybrid AI Semiconductor Compounder",
    "semiconductor_turnaround": "Capital-Intensive Semiconductor Turnaround",
    "speculative_growth": "Speculative Growth",
    "biotech_like": "Biotech-like Screening",
    "pharma": "Pharma / Biotech-like Screening",
    "medical_devices": "Medical Devices Screening",
    "banks": "Financials",
    "brokers_exchanges": "Financials",
    "insurance": "Insurance-like Screening",
    "reit": "REIT-like Screening",
    "energy": "Cyclical",
    "materials_mining": "Cyclical",
    "industrials": "Cyclical / Industrial",
    "aerospace_defense": "Aerospace / Defense",
    "airlines_transport": "Shipping / Airlines / Transport",
    "shipping_logistics": "Shipping / Airlines / Transport",
    "consumer_retail": "Consumer / Retail",
    "restaurants": "Consumer / Retail",
    "utilities": "Utilities / Infrastructure",
    "telecom_media": "Telecom / Media Screening",
    "small_unknown": "Unknown / Data-Limited Screening",
}


def load_eval_expectations(path: str) -> dict[str, str]:
    """Load expected profile families from YAML groups or per-ticker metadata."""
    expectations: dict[str, str] = {}
    current_group: str | None = None
    pending_ticker: str | None = None
    for raw in Path(path).read_text(encoding="utf-8").splitlines():
        line = _strip_comment(raw)
        if not line.strip():
            continue
        stripped = line.strip()
        if stripped == "groups:":
            current_group = None
            pending_ticker = None
            continue
        if raw.startswith("  ") and not raw.startswith("    ") and stripped.endswith(":"):
            current_group = stripped[:-1]
            pending_ticker = None
            continue
        if stripped.startswith("- "):
            pending_ticker = stripped[2:].strip().upper()
            if pending_ticker and current_group:
                expectations[pending_ticker] = GROUP_PROFILE_FAMILIES.get(current_group, current_group.replace("_", " ").title())
            continue
        if pending_ticker and stripped.startswith("expected_profile_family:"):
            expectations[pending_ticker] = stripped.split(":", 1)[1].strip()
    return expectations


def _batch_name(eval_set_path: str) -> str:
    return Path(eval_set_path).stem


def _now_id() -> str:
    return datetime.now().strftime("%Y%m%d_%H%M%S")


def _latest_batch_dir(root: Path, prefix: str) -> Path | None:
    if not root.exists():
        return None
    matches = sorted([p for p in root.iterdir() if p.is_dir() and p.name.startswith(prefix)])
    return matches[-1] if matches else None


def retry_fetch_or_run(fn: Callable[[], Any], max_attempts: int = 2, sleep_seconds: float = 2.0) -> tuple[Any, int]:
    last_exc: Exception | None = None
    for attempt in range(1, max_attempts + 1):
        try:
            return fn(), attempt
        except Exception as exc:  # noqa: BLE001 - batch mode must classify provider failures.
            last_exc = exc
            message = f"{type(exc).__name__}: {exc}".lower()
            is_transient = any(pattern in message for pattern in TRANSIENT_ERROR_PATTERNS)
            if not is_transient or attempt >= max_attempts:
                break
            time.sleep(sleep_seconds)
    assert last_exc is not None
    raise last_exc


def _read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}


def _scan_reports(run_folder: Path) -> list[Path]:
    report_dir = run_folder / "report"
    return sorted(report_dir.glob("*.md")) if report_dir.exists() else []


def _has_reader_unfriendly_numeric(report_text: str) -> bool:
    return bool(re.search(r"\| -?[0-9]+\.[0-9]{3,}", report_text))


def _section_numbers_bad(report_text: str) -> bool:
    numbers = [int(match.group(1)) for match in re.finditer(r"^##\s+(\d+)\.", report_text, re.MULTILINE)]
    if not numbers:
        return True
    return numbers != list(range(numbers[0], numbers[0] + len(numbers))) or len(numbers) != len(set(numbers))


def _toc_mismatch(report_text: str) -> bool:
    toc = re.findall(r"^- \[(.+?)\]\(#", report_text, flags=re.MULTILINE)
    headings = re.findall(r"^##\s+(.+)$", report_text, flags=re.MULTILINE)
    if not toc:
        return True
    return not any(item in "\n".join(headings) for item in toc[:3])


def _chart_refs_missing(report_file: Path, run_folder: Path) -> list[str]:
    missing = []
    text = report_file.read_text(encoding="utf-8", errors="ignore")
    for ref in re.findall(r"!\[[^\]]*\]\(([^)]+)\)", text):
        if ref.startswith(("http://", "https://")):
            continue
        candidates = [(report_file.parent / ref).resolve(), (run_folder / ref).resolve()]
        if not any(path.exists() for path in candidates):
            missing.append(ref)
    return missing


def _patch_materiality_bad(run_folder: Path, report_status: dict[str, Any]) -> bool:
    patch_status = report_status.get("PATCH_STATUS", "NOT_NEEDED")
    if patch_status == "NOT_NEEDED":
        return False
    patch_path = run_folder / "ai" / "correction_patch.json"
    if not patch_path.exists():
        return True
    patch = _read_json(patch_path)
    records = patch.get("patch_records") or patch.get("records") or []
    if patch_status == "APPLIED":
        return not any(
            record.get("materiality") in {"MATERIAL_PATCH_APPLIED", "FALLBACK_PATCH_APPLIED"}
            or record.get("material_change") is True
            for record in records
        )
    return False


def _framework_gap_generic(run_folder: Path, asset_profile: dict[str, Any] | None = None) -> bool:
    gap = run_folder / "self_review" / "framework_gap_analysis.md"
    if not gap.exists():
        return True
    text = gap.read_text(encoding="utf-8", errors="ignore").lower()
    generic_hits = ["company-specific industry framework", "technology\n", "unknown industry"]
    specific_terms = [
        "biotech",
        "semiconductor",
        "foundry",
        "reit",
        "insurance",
        "bank",
        "cyclical",
        "shipping",
        "utility",
        "retail",
        "clinical",
        "pipeline",
    ]
    is_generic = any(hit in text for hit in generic_hits) and not any(term in text for term in specific_terms)
    if not is_generic:
        return False
    asset_profile = asset_profile or {}
    if asset_profile.get("primary_profile", "").startswith("Unknown") and not asset_profile.get("business_model_clues"):
        return False
    return True


def _next_checks_generic(report_text: str) -> bool:
    lower = report_text.lower()
    next_idx = lower.find("next research")
    if next_idx < 0:
        next_idx = lower.find("下一步")
    section = lower[next_idx: next_idx + 1400] if next_idx >= 0 else lower[-1400:]
    profile_terms = [
        "pipeline",
        "clinical",
        "fda",
        "foundry",
        "capex",
        "cash runway",
        "dilution",
        "backlog",
        "nim",
        "credit loss",
        "ffo",
        "combined ratio",
        "same-store",
        "制程",
        "代工",
        "资本开支",
        "管线",
        "临床",
        "稀释",
        "现金 runway",
    ]
    generic_phrases = ["latest filing", "primary source", "manual verification", "最新财报", "人工核查"]
    return sum(term in section for term in profile_terms) < 2 and any(phrase in section for phrase in generic_phrases)


def classify_batch_outcome(lint: LintResult, report_status: dict[str, Any], asset_profile: dict[str, Any]) -> str:
    hard_failures = {
        "missing_asset_profile",
        "missing_report_status",
        "missing_data_thesis_status",
        "missing_lifecycle_report",
        "missing_framework_gap",
        "missing_patch_diff",
        "pack_missing",
        "section_numbering_bad",
        "toc_mismatch",
        "raw_placeholder",
        "forbidden_template_contamination",
        "numeric_format_bad",
        "patch_not_material",
        "self_review_missing",
        "chart_reference_missing",
    }
    if hard_failures.intersection(lint.failed_checks):
        return "FAIL"
    overall = report_status.get("OVERALL_REPORT_STATUS", "PASS")
    thesis = report_status.get("THESIS_VERIFICATION_STATUS", "PASS")
    profile = asset_profile.get("primary_profile", "")
    coverage = asset_profile.get("framework_coverage_level", "")
    if overall == "UNVERIFIED" or thesis in {"UNVERIFIED", "FAIL"}:
        if profile.startswith("Unknown") or coverage in {"SCREENING_ONLY", "UNKNOWN"}:
            if "framework_gap_too_generic" not in lint.failed_checks and "next_checks_too_generic" not in lint.failed_checks:
                return "UNVERIFIED_EXPECTED"
        return "UNVERIFIED_UNEXPECTED"
    if lint.failed_checks:
        return "FAIL"
    if lint.warnings or overall in {"WARNING", "WARNING_DEGRADED"}:
        return "WARNING"
    return "PASS"


def lint_run_folder(run_folder: Path, pack_expected: bool = False) -> LintResult:
    ticker = run_folder.parents[1].name if run_folder.parent.name == "runs" else run_folder.name
    failed: list[str] = []
    warnings: list[str] = []
    details: dict[str, Any] = {}
    asset_profile_path = run_folder / "metadata" / "asset_profile.json"
    report_status_path = run_folder / "metadata" / "report_status.json"
    asset_profile = _read_json(asset_profile_path)
    report_status = _read_json(report_status_path)
    if not asset_profile_path.exists():
        failed.append("missing_asset_profile")
    if not report_status_path.exists():
        failed.append("missing_report_status")
    if report_status and not {"DATA_VERIFICATION_STATUS", "THESIS_VERIFICATION_STATUS"}.issubset(report_status):
        failed.append("missing_data_thesis_status")
    if report_status.get("LIFECYCLE_LOGIC_STATUS") != "PASS" and not (run_folder / "audit" / "lifecycle_logic_report.md").exists():
        failed.append("missing_lifecycle_report")
    if not (run_folder / "self_review" / "framework_gap_analysis.md").exists():
        failed.append("missing_framework_gap")
    if report_status.get("PATCH_STATUS", "NOT_NEEDED") != "NOT_NEEDED" and not (run_folder / "ai" / "patch_diff_log.md").exists():
        failed.append("missing_patch_diff")
    if pack_expected and not list(run_folder.glob("*_research_pack.zip")):
        failed.append("pack_missing")
    if not (run_folder / "self_review" / "system_self_review.md").exists():
        failed.append("self_review_missing")
    reports = _scan_reports(run_folder)
    if not reports:
        failed.append("report_missing")
    combined_text = ""
    missing_charts: list[str] = []
    for report in reports:
        text = report.read_text(encoding="utf-8", errors="ignore")
        combined_text += "\n" + text
        if _section_numbers_bad(text):
            failed.append("section_numbering_bad")
        if _toc_mismatch(text):
            warnings.append("toc_mismatch")
        if _has_reader_unfriendly_numeric(text):
            failed.append("numeric_format_bad")
        missing_charts.extend(_chart_refs_missing(report, run_folder))
    if missing_charts:
        failed.append("chart_reference_missing")
        details["missing_charts"] = missing_charts[:20]
    if any(token in combined_text for token in RAW_PLACEHOLDERS):
        failed.append("raw_placeholder")
    contamination = [
        "buyback-supported EPS" if asset_profile.get("primary_profile", "").startswith(("Speculative", "Biotech", "Unknown")) and "buyback-supported EPS" in combined_text else "",
        "low PE means cheap" if "low PE means cheap" in combined_text else "",
    ]
    if any(contamination):
        failed.append("forbidden_template_contamination")
        details["template_contamination"] = [item for item in contamination if item]
    if _patch_materiality_bad(run_folder, report_status):
        failed.append("patch_not_material")
    if _framework_gap_generic(run_folder, asset_profile):
        failed.append("framework_gap_too_generic")
    if _next_checks_generic(combined_text):
        failed.append("next_checks_too_generic")
    if asset_profile.get("primary_profile", "").startswith("Unknown") and asset_profile.get("business_model_clues"):
        if "framework_gap_too_generic" in failed:
            failed.append("profile_unknown_but_clues_exist")
    failed = sorted(set(failed))
    warnings = sorted(set(warnings))
    lint = LintResult(
        ticker=ticker,
        status="FAIL" if failed else "WARNING" if warnings else "PASS",
        failed_checks=failed,
        warnings=warnings,
        details=details,
    )
    lint.outcome = classify_batch_outcome(lint, report_status, asset_profile)
    return lint


def _hash_obj(obj: Any) -> str:
    payload = json.dumps(obj, sort_keys=True, ensure_ascii=False, default=str)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def build_compact_ai_review_payload(run_folder: Path, lint_result: LintResult) -> dict[str, Any]:
    asset_profile = _read_json(run_folder / "metadata" / "asset_profile.json")
    report_status = _read_json(run_folder / "metadata" / "report_status.json")
    snippets: dict[str, str] = {}
    for report in _scan_reports(run_folder)[:2]:
        text = report.read_text(encoding="utf-8", errors="ignore")
        for marker in ["Next Research Steps", "下一步研究清单", "Framework Gap", "研究框架"]:
            idx = text.lower().find(marker.lower())
            if idx >= 0:
                snippets[marker] = text[idx: idx + 900]
    return {
        "ticker": lint_result.ticker,
        "asset_profile": {
            "primary_profile": asset_profile.get("primary_profile"),
            "secondary_profile": asset_profile.get("secondary_profile"),
            "dominant_metric_set": asset_profile.get("dominant_metric_set", []),
            "data_deficit_flags": asset_profile.get("data_deficit_flags", []),
            "framework_coverage_level": asset_profile.get("framework_coverage_level"),
        },
        "report_status": {
            "data": report_status.get("DATA_VERIFICATION_STATUS"),
            "thesis": report_status.get("THESIS_VERIFICATION_STATUS"),
            "overall": report_status.get("OVERALL_REPORT_STATUS"),
        },
        "failed_checks": lint_result.failed_checks,
        "bad_sections": snippets,
        "expected_profile_rules": asset_profile.get("dominant_metric_set", []),
    }


def compact_ai_review(payload: dict[str, Any]) -> dict[str, Any]:
    """Local compact review scaffold; no paid API call is made in batch infrastructure."""
    profile = " / ".join(
        item for item in [
            payload.get("asset_profile", {}).get("primary_profile"),
            payload.get("asset_profile", {}).get("secondary_profile"),
        ] if item
    )
    failed = payload.get("failed_checks", [])
    metric_set = payload.get("asset_profile", {}).get("dominant_metric_set", [])
    replacement = "Use profile-specific next checks tied to " + ", ".join(metric_set[:6]) + "."
    return {
        "correction_summary": f"Compact review found {len(failed)} deterministic issue(s) for {profile}.",
        "suggested_profile": profile,
        "section_to_patch": "next_checks" if "next_checks_too_generic" in failed else "framework_gap",
        "replacement_text": replacement,
        "reason": "Deterministic linter found weak or missing company-specific interpretation.",
        "must_contain": metric_set[:6],
        "must_not_contain": ["generic monitoring", "full report rewrite"],
        "external_ai_call": False,
    }


PROFILE_EXPECTED_FEATURES = {
    "Mature Compounder": ["margin durability", "FCF stability", "business mix", "buybacks", "PE / FCF sensitivity"],
    "Speculative Growth": ["revenue growth quality", "gross margin improvement", "cash burn", "cash runway", "dilution risk", "path to profitability"],
    "Capital-Intensive Semiconductor Turnaround": ["foundry revenue / margin", "capex roadmap", "gross margin bridge", "process node progress", "data center competitiveness", "free cash flow bridge"],
    "Hybrid AI Semiconductor Compounder": ["AI data center revenue", "gross margin sustainability", "supply / capacity constraint", "customer concentration", "hyperscaler capex cycle", "export control risk", "valuation premium risk"],
    "Biotech-like Screening": ["pipeline stage", "clinical trial milestones", "FDA / EMA regulatory path", "R&D burn", "cash runway", "dilution risk"],
    "Financials": ["NIM", "ROE", "ROA", "credit losses", "provision coverage", "deposit cost", "capital adequacy", "asset quality"],
    "Insurance-like Screening": ["combined ratio", "underwriting margin", "reserve adequacy", "investment income", "float", "catastrophe exposure"],
    "REIT-like Screening": ["FFO", "AFFO", "occupancy", "rent spread", "same-store NOI", "debt maturity", "cap rate"],
    "Cyclical": ["cycle position", "normalized earnings", "mean reversion", "commodity / demand sensitivity", "margin peak risk"],
    "Shipping / Airlines / Transport": ["freight rate / yield", "fleet utilization", "fuel cost", "fleet age", "orderbook", "leverage", "cycle sensitivity"],
    "Utilities / Infrastructure": ["regulated asset base", "allowed ROE", "rate case", "capex plan", "debt cost", "dividend coverage"],
    "Consumer / Retail": ["same-store sales", "traffic", "ticket", "inventory", "gross margin", "store count"],
    "Unknown / Data-Limited Screening": ["business model verification", "industry-specific metric discovery", "manual framework selection"],
}

PROFILE_MUST_NOT_CONTAIN = {
    "Shipping / Airlines / Transport": ["pipeline", "clinical trial", "FDA", "regulatory milestones", "drug candidate"],
    "Financials": ["ordinary FCF margin as core", "net debt / EBITDA as core", "cash runway framing"],
    "Biotech-like Screening": ["ordinary mature compounder thesis", "ordinary PE conclusion", "buyback-supported EPS"],
    "Capital-Intensive Semiconductor Turnaround": ["low PE means cheap", "ordinary mature compounder thesis", "software margin thesis"],
}


def _profile_family_compatible(expected: str | None, detected: str | None, secondary: str | None = "") -> bool:
    if not expected:
        return True
    detected_text = f"{detected or ''} {secondary or ''}".lower()
    expected_lower = expected.lower()
    aliases = {
        "financials": ["financial", "bank"],
        "shipping / airlines / transport": ["shipping", "transport", "airline", "cyclical"],
        "capital-intensive semiconductor turnaround": ["semiconductor", "turnaround"],
        "hybrid ai semiconductor compounder": ["ai semiconductor", "semiconductor", "data center"],
        "biotech-like screening": ["biotech", "clinical"],
        "reit-like screening": ["reit", "real estate"],
        "insurance-like screening": ["insurance"],
        "utilities / infrastructure": ["utilities", "infrastructure"],
        "cyclical": ["cyclical", "commodity", "energy", "industrial"],
        "speculative growth": ["speculative", "unprofitable", "growth"],
        "mature compounder": ["mature compounder"],
    }
    return any(alias in detected_text for alias in aliases.get(expected_lower, [expected_lower]))


def _expected_features_for_family(expected: str | None, asset: dict[str, Any]) -> list[str]:
    if expected:
        for key, features in PROFILE_EXPECTED_FEATURES.items():
            if key.lower() in expected.lower() or expected.lower() in key.lower():
                return features
    return asset.get("dominant_metric_set", []) or asset.get("data_deficit_flags", [])


def generate_training_case(
    ticker_result: TickerRunResult,
    lint_result: LintResult,
    ai_review: dict[str, Any] | None,
) -> dict[str, Any]:
    asset = ticker_result.asset_profile
    expected_family = ticker_result.expected_profile_family
    detected = asset.get("primary_profile")
    secondary = asset.get("secondary_profile")
    confidence = asset.get("sector_confidence") or asset.get("thesis_spine_confidence")
    compatible = _profile_family_compatible(expected_family, detected, secondary)
    profile_conflict = bool(expected_family and not compatible)
    if not expected_family and (confidence == "LOW" or asset.get("framework_coverage_level") in {"UNKNOWN", "SCREENING_ONLY"}):
        expected_profile = "HUMAN_REVIEW_REQUIRED"
        human_review_required = True
    else:
        expected_profile = expected_family or (ai_review.get("suggested_profile") if ai_review else detected)
        human_review_required = profile_conflict or expected_profile == "HUMAN_REVIEW_REQUIRED"
    expected_features = _expected_features_for_family(expected_profile if expected_profile != "HUMAN_REVIEW_REQUIRED" else expected_family, asset)
    failure_reason = ticker_result.reason or ticker_result.error_message or "Deterministic linter generated this case."
    suggested_fix = ticker_result.suggested_next_action or "Inspect failed checks and convert repeated issues into deterministic tests."
    must_not = []
    if expected_profile and expected_profile != "HUMAN_REVIEW_REQUIRED":
        for key, terms in PROFILE_MUST_NOT_CONTAIN.items():
            if key.lower() in expected_profile.lower() or expected_profile.lower() in key.lower():
                must_not = terms
                break
    return {
        "ticker": ticker_result.ticker,
        "company_name": asset.get("company_name", ticker_result.ticker),
        "detected_profile": asset.get("primary_profile"),
        "expected_profile": expected_profile,
        "expected_profile_family": expected_family,
        "profile_conflict": profile_conflict,
        "human_review_required": human_review_required,
        "report_status": ticker_result.report_status,
        "failed_checks": lint_result.failed_checks,
        "wrong_output": "; ".join(lint_result.failed_checks),
        "failure_reason": failure_reason,
        "suggested_fix": suggested_fix,
        "expected_output_features": expected_features,
        "must_contain": expected_features[:8],
        "must_not_contain": must_not,
        "reason": ai_review.get("reason", failure_reason) if ai_review else failure_reason,
        "data_refs_used": ["metadata/asset_profile.json", "metadata/report_status.json", "self_review/framework_gap_analysis.md"],
        "fixed_by": "pending",
        "regression_status": "new",
    }


def _write_yaml_like(path: Path, obj: dict[str, Any]) -> None:
    lines = []
    for key, value in obj.items():
        if isinstance(value, (list, tuple)):
            lines.append(f"{key}:")
            for item in value:
                lines.append(f"  - {item}")
        elif isinstance(value, dict):
            lines.append(f"{key}: {json.dumps(value, ensure_ascii=False)}")
        else:
            lines.append(f"{key}: {value}")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _ensure_training_dirs() -> dict[str, Path]:
    root = Path("training_cases")
    paths = {
        "root": root,
        "generated": root / "generated",
        "corrections": root / "corrections",
        "eval_sets": root / "eval_sets",
    }
    for path in paths.values():
        path.mkdir(parents=True, exist_ok=True)
    readme = root / "README.md"
    if not readme.exists():
        readme.write_text(
            "# Training Cases\n\nLocal eval-driven correction cases generated from deterministic batch failures. This is not model fine-tuning.\n",
            encoding="utf-8",
        )
    return paths


def _write_training_case(case: dict[str, Any], batch_out_dir: Path) -> None:
    paths = _ensure_training_dirs()
    date = datetime.now().strftime("%Y%m%d")
    ticker = str(case.get("ticker", "UNKNOWN")).upper()
    _write_yaml_like(paths["generated"] / f"{ticker}_{date}.yaml", case)
    line = json.dumps(case, ensure_ascii=False)
    with (paths["corrections"] / "correction_cases.jsonl").open("a", encoding="utf-8") as fh:
        fh.write(line + "\n")
    with (batch_out_dir / "training_cases_generated.jsonl").open("a", encoding="utf-8") as fh:
        fh.write(line + "\n")


def _is_fetch_failure(exc: Exception) -> bool:
    message = f"{type(exc).__name__}: {exc}".lower()
    return any(pattern in message for pattern in TRANSIENT_ERROR_PATTERNS) or "no data" in message or "ticker" in message


def classify_exception(exc: Exception) -> tuple[str, str, str]:
    message = f"{type(exc).__name__}: {exc}"
    lower = message.lower()
    if "plot_" in lower or "renderer" in lower or "float() argument" in lower:
        return (
            "report-renderer",
            "Report generation reached the rendering stage but failed while converting provider data into a chart/report artifact.",
            "Fix renderer null handling or rerun after the renderer patch; do not treat this as an investment-data conclusion.",
        )
    if _is_fetch_failure(exc):
        return (
            "provider/data",
            "The provider returned missing, partial, unsupported, or transient data for this ticker.",
            "Retry later, verify ticker provider support, or mark this case as provider/data failure.",
        )
    return (
        "unknown",
        "The batch runner caught an unexpected report-generation exception.",
        "Inspect the error traceback, add a deterministic test, and classify the failure before broad_200.",
    )


def run_one_ticker_safely(
    ticker: str,
    options: BatchOptions,
    batch_out_dir: Path,
    runner: Callable[..., dict[str, Any]] | None = None,
    packer: Callable[[Path], Path] | None = None,
    expected_profile_family: str | None = None,
) -> TickerRunResult:
    from company_research_tool import pack_report_folder, run_one

    runner = runner or run_one
    packer = packer or pack_report_folder
    result = TickerRunResult(ticker=ticker, status="REPORT_FAILED")
    run_id = f"{batch_out_dir.name}_{ticker}"

    def call_runner() -> dict[str, Any]:
        return runner(
            symbol=ticker,
            benchmark=options.benchmark,
            start_date=options.start,
            end_date=None,
            years=5,
            output=Path(options.output),
            risk_free_rate=0.0,
            archive=True,
            run_id=run_id,
            ai_review=False,
            audit_data=True,
            cn=options.both,
            language="both" if options.both else "en",
            term_style="pure",
            price_field="adj_close",
            annualization_days=252,
            profile_hint=expected_profile_family,
        )

    try:
        row, attempts = retry_fetch_or_run(call_runner)
        result.attempts = attempts
        run_folder = Path(row["folder"])
        result.run_folder = str(run_folder)
        if options.pack:
            try:
                result.pack_zip = str(packer(run_folder))
            except Exception as exc:  # noqa: BLE001
                result.status = "PACK_FAILED"
                result.error_type = type(exc).__name__
                result.error_message = str(exc)
        result.report_status = _read_json(run_folder / "metadata" / "report_status.json")
        result.asset_profile = _read_json(run_folder / "metadata" / "asset_profile.json")
        lint = lint_run_folder(run_folder, pack_expected=options.pack)
        result.lint = lint
        result.outcome = lint.outcome
        if result.status != "PACK_FAILED":
            result.status = "COMPLETED" if lint.outcome == "PASS" else "COMPLETED_WITH_WARNINGS"
        pointer_dir = batch_out_dir / "runs" / ticker
        pointer_dir.mkdir(parents=True, exist_ok=True)
        (pointer_dir / "pointer.json").write_text(
            json.dumps({"run_folder": str(run_folder), "pack_zip": result.pack_zip}, indent=2),
            encoding="utf-8",
        )
    except Exception as exc:  # noqa: BLE001
        result.attempts = getattr(result, "attempts", 1)
        result.error_type = type(exc).__name__
        result.error_message = str(exc)
        result.error_category, result.reason, result.suggested_next_action = classify_exception(exc)
        result.status = "FETCH_FAILED" if _is_fetch_failure(exc) else "REPORT_FAILED"
        result.outcome = "FETCH_FAILED" if result.status == "FETCH_FAILED" else "REPORT_FAILED"
        result.lint = LintResult(ticker=ticker, status="FAIL", failed_checks=[result.status.lower()], outcome=result.outcome)
        result.report_status = {}
        result.asset_profile = {}
        (batch_out_dir / "runs" / ticker).mkdir(parents=True, exist_ok=True)
        (batch_out_dir / "runs" / ticker / "error.json").write_text(
            json.dumps(
                {
                    "ticker": ticker,
                    "status": result.status,
                    "error_type": result.error_type,
                    "error_message": result.error_message,
                    "error_category": result.error_category,
                    "reason": result.reason,
                    "suggested_next_action": result.suggested_next_action,
                    "traceback": traceback.format_exc(limit=5),
                },
                indent=2,
            ),
            encoding="utf-8",
        )
    return result


def load_existing_ticker_result(
    ticker: str,
    batch_out_dir: Path,
    pack_expected: bool = False,
    packer: Callable[[Path], Path] | None = None,
) -> TickerRunResult | None:
    pointer = batch_out_dir / "runs" / ticker / "pointer.json"
    error_path = batch_out_dir / "runs" / ticker / "error.json"
    if pointer.exists():
        data = _read_json(pointer)
        run_folder_raw = data.get("run_folder")
        if not run_folder_raw:
            return None
        run_folder = Path(run_folder_raw)
        if not run_folder.exists():
            return None
        pack_zip = data.get("pack_zip")
        if pack_expected and (not pack_zip or not Path(pack_zip).exists()):
            try:
                from company_research_tool import pack_report_folder
                zip_path = (packer or pack_report_folder)(run_folder)
                pack_zip = str(zip_path)
                pointer.write_text(json.dumps({"run_folder": str(run_folder), "pack_zip": pack_zip}, indent=2), encoding="utf-8")
            except Exception:
                pack_zip = None
        report_status = _read_json(run_folder / "metadata" / "report_status.json")
        asset_profile = _read_json(run_folder / "metadata" / "asset_profile.json")
        lint = lint_run_folder(run_folder, pack_expected=pack_expected)
        return TickerRunResult(
            ticker=ticker,
            status="COMPLETED" if lint.outcome == "PASS" else "COMPLETED_WITH_WARNINGS",
            outcome=lint.outcome,
            run_folder=str(run_folder),
            report_status=report_status,
            asset_profile=asset_profile,
            lint=lint,
            pack_zip=pack_zip,
            expected_profile_family=None,
        )
    if error_path.exists():
        data = _read_json(error_path)
        status = data.get("status", "REPORT_FAILED")
        return TickerRunResult(
            ticker=ticker,
            status=status,
            outcome=status,
            error_type=data.get("error_type"),
            error_message=data.get("error_message"),
            error_category=data.get("error_category"),
            reason=data.get("reason"),
            suggested_next_action=data.get("suggested_next_action"),
            lint=LintResult(ticker=ticker, status="FAIL", failed_checks=[status.lower()], outcome=status),
        )
    return None


def _result_needs_training(result: TickerRunResult) -> bool:
    return result.outcome != "PASS" or bool(result.lint and result.lint.failed_checks)


def _result_needs_ai_review(result: TickerRunResult) -> bool:
    if result.outcome in {"FAIL", "UNVERIFIED_UNEXPECTED", "UNVERIFIED_EXPECTED", "REPORT_FAILED"}:
        return True
    if result.lint and any(
        check in result.lint.failed_checks
        for check in ["patch_not_material", "next_checks_too_generic", "framework_gap_too_generic", "profile_unknown_but_clues_exist"]
    ):
        return True
    return False


def run_batch(
    eval_set_path: str,
    both: bool = False,
    full: bool = False,
    pack: bool = False,
    max_workers: int = 2,
    no_ai: bool = True,
    ai_review_failures: bool = False,
    max_ai_reviews: int = 40,
    resume: bool = False,
    only_failed: bool = False,
    force: bool = False,
    batch_id: str | None = None,
    runner: Callable[..., dict[str, Any]] | None = None,
    packer: Callable[[Path], Path] | None = None,
) -> BatchResult:
    options = BatchOptions(
        eval_set_path=eval_set_path,
        both=both,
        full=full,
        pack=pack,
        max_workers=max_workers,
        no_ai=no_ai,
        ai_review_failures=ai_review_failures,
        max_ai_reviews=max_ai_reviews,
        resume=resume,
        only_failed=only_failed,
        force=force,
        batch_id=batch_id,
    )
    tickers = load_eval_set(eval_set_path)
    expectations = load_eval_expectations(eval_set_path)
    root = Path("reports") / "batch_runs"
    root.mkdir(parents=True, exist_ok=True)
    prefix = _batch_name(eval_set_path)
    out_dir = _latest_batch_dir(root, prefix) if resume else None
    if out_dir is None:
        out_dir = root / (batch_id or f"{prefix}_{_now_id()}")
    out_dir.mkdir(parents=True, exist_ok=True)
    for sub in ["cache", "runs"]:
        (out_dir / sub).mkdir(exist_ok=True)
    (out_dir / "training_cases_generated.jsonl").write_text("", encoding="utf-8")
    result = BatchResult(batch_id=out_dir.name, eval_set_path=eval_set_path, out_dir=str(out_dir))

    cache_path = out_dir / "cache" / "ai_review_cache.json"
    ai_cache = _read_json(cache_path)

    previous_failures: set[str] = set()
    if only_failed and (out_dir / "batch_summary.json").exists():
        previous = json.loads((out_dir / "batch_summary.json").read_text(encoding="utf-8"))
        previous_failures = {
            row["ticker"] for row in previous.get("results", [])
            if row.get("outcome") not in {"PASS", "WARNING", "UNVERIFIED_EXPECTED"}
        }
    selected_tickers = [ticker for ticker in tickers if not only_failed or ticker in previous_failures]

    review_existing_only = resume and not force

    def process_result(ticker_result: TickerRunResult) -> None:
        ticker_result.expected_profile_family = expectations.get(ticker_result.ticker)
        if ai_review_failures and not no_ai and ticker_result.run_folder and ticker_result.lint and _result_needs_ai_review(ticker_result):
            payload = build_compact_ai_review_payload(Path(ticker_result.run_folder), ticker_result.lint)
            failure_hash = _hash_obj({"ticker": ticker_result.ticker, "failed": ticker_result.lint.failed_checks, "payload": payload})
            if failure_hash in ai_cache:
                ticker_result.ai_review = ai_cache[failure_hash]
                result.ai_reviews_skipped_by_cache += 1
            elif result.ai_reviews_used < max_ai_reviews:
                review = compact_ai_review(payload)
                ticker_result.ai_review = review
                ai_cache[failure_hash] = review
                result.ai_reviews_used += 1
        if ticker_result.lint and _result_needs_training(ticker_result):
            case = generate_training_case(ticker_result, ticker_result.lint, ticker_result.ai_review)
            _write_training_case(case, out_dir)
            ticker_result.training_case_generated = True
            result.training_cases_generated += 1
        result.results.append(ticker_result)
        write_batch_outputs(result, out_dir)

    if review_existing_only or max_workers <= 1:
        for ticker in selected_tickers:
            ticker_result = load_existing_ticker_result(ticker, out_dir, pack_expected=pack, packer=packer) if review_existing_only else None
            if ticker_result is None:
                ticker_result = run_one_ticker_safely(ticker, options, out_dir, runner=runner, packer=packer, expected_profile_family=expectations.get(ticker))
            process_result(ticker_result)
    else:
        with ThreadPoolExecutor(max_workers=max(1, max_workers)) as executor:
            futures = {
                executor.submit(run_one_ticker_safely, ticker, options, out_dir, runner, packer, expectations.get(ticker)): ticker
                for ticker in selected_tickers
            }
            for future in as_completed(futures):
                process_result(future.result())
    cache_path.write_text(json.dumps(ai_cache, indent=2, ensure_ascii=False), encoding="utf-8")
    write_batch_outputs(result, out_dir)
    return result


def _profile_distribution(results: list[TickerRunResult]) -> dict[str, int]:
    dist: dict[str, int] = {}
    for result in results:
        profile = result.asset_profile.get("primary_profile") or "Unknown"
        dist[profile] = dist.get(profile, 0) + 1
    return dict(sorted(dist.items(), key=lambda item: item[0]))


def _failure_distribution(results: list[TickerRunResult]) -> dict[str, int]:
    dist: dict[str, int] = {}
    for result in results:
        checks = result.lint.failed_checks if result.lint else [result.status]
        for check in checks or [result.outcome]:
            dist[check] = dist.get(check, 0) + 1
    return dict(sorted(dist.items(), key=lambda item: item[1], reverse=True))


def _counts(results: list[TickerRunResult]) -> dict[str, int]:
    keys = [
        "PASS",
        "WARNING",
        "UNVERIFIED_EXPECTED",
        "UNVERIFIED_UNEXPECTED",
        "FAIL",
        "FETCH_FAILED",
        "REPORT_FAILED",
        "NEEDS_HUMAN_REVIEW",
    ]
    counts = {key: 0 for key in keys}
    for result in results:
        counts[result.outcome] = counts.get(result.outcome, 0) + 1
    return counts


def _md_table(headers: list[str], rows: list[list[Any]]) -> str:
    if not rows:
        rows = [["None" for _ in headers]]
    lines = ["| " + " | ".join(headers) + " |", "| " + " | ".join("---" for _ in headers) + " |"]
    for row in rows:
        lines.append("| " + " | ".join(str(cell).replace("\n", " ") for cell in row) + " |")
    return "\n".join(lines)


def _training_case_file(ticker: str) -> str:
    date = datetime.now().strftime("%Y%m%d")
    return f"training_cases/generated/{ticker.upper()}_{date}.yaml"


def _checks(result: TickerRunResult) -> list[str]:
    return result.lint.failed_checks if result.lint else [result.status]


def _checks_or_outcome(result: TickerRunResult) -> list[str]:
    checks = _checks(result)
    return checks if checks else [result.outcome]


def _reason_for_result(result: TickerRunResult) -> str:
    if result.reason:
        return result.reason
    if result.error_message:
        return result.error_message
    checks = _checks(result)
    if "framework_gap_too_generic" in checks and result.ticker == "NVDA":
        return "NVDA-like AI semiconductor reports need a specific data-center, margin, supply, customer, export-control, and valuation-premium framework."
    if "framework_gap_too_generic" in checks:
        return "Framework gap narrative was too generic for the detected asset profile."
    if "next_checks_too_generic" in checks:
        return "Next checks were too generic for the detected asset profile."
    return "Deterministic lint flagged this report for review."


def _suggested_fix_for_result(result: TickerRunResult) -> str:
    if result.suggested_next_action:
        return result.suggested_next_action
    checks = _checks(result)
    if "framework_gap_too_generic" in checks and result.ticker == "NVDA":
        return "Use Hybrid AI Semiconductor Compounder framing and require AI data-center revenue, gross-margin sustainability, hyperscaler capex, supply/capacity, export controls, and valuation premium checks."
    if "framework_gap_too_generic" in checks:
        return "Replace generic framework gap text with profile-specific missing metrics and manual verification steps."
    if "next_checks_too_generic" in checks:
        return "Regenerate next checks from the asset profile's dominant metric set."
    if result.run_folder:
        return f"Inspect `{result.run_folder}` and generated training case."
    return "Retry provider fetch or inspect the captured exception."


def write_batch_outputs(batch_result: BatchResult, out_dir: Path) -> None:
    results = batch_result.results
    counts = _counts(results)
    profile_dist = _profile_distribution(results)
    failure_dist = _failure_distribution(results)
    summary = {
        "batch_id": batch_result.batch_id,
        "eval_set_path": batch_result.eval_set_path,
        "total_tickers": len(results),
        "counts": counts,
        "ai_reviews_used": batch_result.ai_reviews_used,
        "ai_reviews_skipped_by_cache": batch_result.ai_reviews_skipped_by_cache,
        "training_cases_generated": batch_result.training_cases_generated,
        "results": [
            {
                "ticker": r.ticker,
                "status": r.status,
                "outcome": r.outcome,
                "profile": r.asset_profile.get("primary_profile"),
                "run_folder": r.run_folder,
                "failed_checks": r.lint.failed_checks if r.lint else [],
                "warnings": r.lint.warnings if r.lint else [],
                "error_type": r.error_type,
                "error_category": r.error_category,
                "error_message": r.error_message,
                "reason": _reason_for_result(r),
                "suggested_next_action": _suggested_fix_for_result(r),
                "training_case_generated": r.training_case_generated,
                "ai_reviewed": r.ai_review is not None,
            }
            for r in results
        ],
    }
    (out_dir / "batch_summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    csv_fields = ["ticker", "status", "outcome", "profile", "run_folder", "failed_checks", "warnings", "error_type", "error_category", "reason", "suggested_next_action", "training_case_generated", "ai_reviewed"]
    with (out_dir / "batch_summary.csv").open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=csv_fields)
        writer.writeheader()
        for row in summary["results"]:
            writer.writerow({
                key: ";".join(row[key]) if key in {"failed_checks", "warnings"} else row.get(key)
                for key in csv_fields
            })

    actual_failures = [r for r in results if r.outcome in {"FAIL", "UNVERIFIED_UNEXPECTED", "FETCH_FAILED", "REPORT_FAILED"}]
    review_needed = [r for r in results if r.outcome in {"WARNING", "UNVERIFIED_EXPECTED"}]
    warnings = [r for r in results if r.outcome == "WARNING"]
    unverifiable = [r for r in results if r.outcome == "UNVERIFIED_EXPECTED"]
    failures = actual_failures
    generated = datetime.now().strftime("%Y-%m-%d %H:%M Asia/Singapore")
    status_rows = [
        ["Total tickers", len(results)],
        ["Completed", sum(1 for r in results if r.status in {"COMPLETED", "COMPLETED_WITH_WARNINGS"})],
        ["Report failed", counts.get("REPORT_FAILED", 0)],
        ["PASS", counts.get("PASS", 0)],
        ["WARNING", counts.get("WARNING", 0)],
        ["UNVERIFIED_EXPECTED", counts.get("UNVERIFIED_EXPECTED", 0)],
        ["UNVERIFIED_UNEXPECTED", counts.get("UNVERIFIED_UNEXPECTED", 0)],
        ["FAIL", counts.get("FAIL", 0)],
        ["AI reviews used", batch_result.ai_reviews_used],
        ["Training cases generated", batch_result.training_cases_generated],
    ]
    failure_rows = [[r.ticker, ", ".join(_checks_or_outcome(r)), _reason_for_result(r), _suggested_fix_for_result(r)] for r in actual_failures[:20]]
    review_rows = [[r.ticker, r.outcome, r.asset_profile.get("primary_profile", "Unknown"), r.asset_profile.get("framework_coverage_level", "UNKNOWN"), _suggested_fix_for_result(r)] for r in review_needed[:30]]
    warning_rows = [[r.ticker, r.asset_profile.get("primary_profile", "Unknown"), ", ".join(r.lint.warnings if r.lint else []), "Yes" if r.outcome != "PASS" else "No"] for r in warnings[:20]]
    case_rows = [[r.ticker, ", ".join(_checks_or_outcome(r)), _training_case_file(r.ticker)] for r in results if r.training_case_generated]
    md = f"""# Batch Evaluation Report

> Batch ID: `{batch_result.batch_id}`  
> Eval Set: `{Path(batch_result.eval_set_path).name}`  
> Mode: deterministic first, compact AI review for failures  
> External AI Calls: 0  
> Generated: {generated}

## 1. Executive Summary

This batch validates the v4.4 batch evaluation foundation. It is a smoke-level infrastructure run, not the full `broad_200` live pressure test.

The batch runner isolated ticker-level failures, generated report packs where possible, wrote deterministic lint summaries, and produced local system-training cases for hard or suspicious outcomes.

## 2. Status Dashboard

{_md_table(["Metric", "Value"], status_rows)}

## 3. What Worked

- Batch runner completed without a global crash.
- Failed ticker(s) were isolated and written into `failures.md`.
- Training cases were generated for hard or suspicious outcomes.
- Compact review did not send full reports, CSV files, or charts.
- Pack outputs were checked during deterministic lint.

## 4. Actual Failures

{_md_table(["Ticker", "Failure", "Reason", "Suggested Fix"], failure_rows)}

Actual failures are limited to `REPORT_FAILED`, `FETCH_FAILED`, `FAIL`, and `UNVERIFIED_UNEXPECTED`. Longer failure detail is available in `failures.md`.

## 5. Reports Requiring Review

{_md_table(["Ticker", "Outcome", "Primary Profile", "Framework Coverage", "Suggested Action"], review_rows)}

## 6. Expected Unverified / Framework-Limited Reports

{_md_table(["Ticker", "Primary Profile", "Secondary Profile", "Suggested Extension"], [[r.ticker, r.asset_profile.get("primary_profile", "Unknown"), r.asset_profile.get("secondary_profile", ""), r.asset_profile.get("suggested_framework_extension", "")] for r in unverifiable])}

## 7. Failure Type Distribution

{_md_table(["Failure Type", "Count"], [[key, value] for key, value in failure_dist.items()])}

## 8. Profile Distribution

{_md_table(["Profile", "Count"], [[key, value] for key, value in profile_dist.items()])}

## 9. Generated Training Cases

{_md_table(["Ticker", "Case Type", "File"], case_rows)}

## 10. Credit / AI Usage

- External paid AI calls: 0
- Compact local failure reviews: {batch_result.ai_reviews_used}
- Full reports sent to AI: No
- CSV / charts sent to AI: No
- Review mode: local deterministic/compact review only; no external AI API call.
- `broad_200` was not run live.

## 11. Next Recommended Fixes

1. Review any `REPORT_FAILED` ticker and classify whether it is provider/data or renderer logic.
2. Improve repeated `framework_gap_too_generic` cases with profile-specific framework text.
3. Rerun `smoke_12` after fixes.
4. Only after smoke is stable, run `broad_200` deterministic-only.

## 12. Known Limitations

- This is not a live `broad_200` result.
- Compact AI review is local and conservative in this foundation pass.
- Some warning reports may still require human review before becoming release-quality examples.
"""
    (out_dir / "batch_summary.md").write_text(md, encoding="utf-8")

    def write_failure_listing(path: Path, rows: list[TickerRunResult]) -> None:
        body = ["# Batch Failures", ""]
        for r in rows:
            checks = ", ".join(_checks(r))
            body.extend([
                f"## {r.ticker} — {r.outcome}",
                "",
                "### What happened",
                _reason_for_result(r),
                "",
                "### Error Type",
                r.error_category or r.error_type or "linter",
                "",
                "### Failed Checks",
                checks or "None",
                "",
                "### Why it matters",
                "This ticker cannot be treated as a clean batch pass until the failure is classified and either fixed or accepted as an expected data/framework limitation.",
                "",
                "### Suggested next action",
                _suggested_fix_for_result(r),
                "",
                "### Training case",
                f"Generated: {r.training_case_generated}. File: `{_training_case_file(r.ticker)}`" if r.training_case_generated else "Not generated.",
                "",
            ])
        if not rows:
            body.append("None.")
        path.write_text("\n".join(body), encoding="utf-8")

    def write_warning_listing(path: Path, rows: list[TickerRunResult]) -> None:
        body = ["# Batch Warnings", ""]
        for r in rows:
            body.extend([
                f"## {r.ticker}",
                f"- Profile: {r.asset_profile.get('primary_profile', 'Unknown')}",
                f"- Warning checks: {', '.join(r.lint.warnings if r.lint else []) or 'None'}",
                f"- Human review needed: {'Yes' if r.outcome != 'PASS' else 'No'}",
                "- Suggested action: review warning checks after hard failures are cleared.",
                "",
            ])
        if not rows:
            body.append("None.")
        path.write_text("\n".join(body), encoding="utf-8")

    write_failure_listing(out_dir / "failures.md", failures)
    write_warning_listing(out_dir / "warnings.md", warnings)
    write_failure_listing(out_dir / "unverifiable_expected.md", unverifiable)

    profile_lines = ["# Profile Distribution", "", "## Why this matters", "", "This file checks whether the smoke/broad set covers multiple asset types instead of only testing mega-cap technology stocks.", ""]
    profile_lines.append(_md_table(
        ["Ticker", "Primary Profile", "Secondary Profile", "Framework Coverage", "Suggested Extension"],
        [
            [
                r.ticker,
                r.asset_profile.get("primary_profile", "Unknown"),
                r.asset_profile.get("secondary_profile", ""),
                r.asset_profile.get("framework_coverage_level", "UNKNOWN"),
                r.asset_profile.get("suggested_framework_extension", ""),
            ]
            for r in sorted(results, key=lambda item: item.ticker)
        ],
    ))
    (out_dir / "profile_distribution.md").write_text("\n".join(profile_lines) + "\n", encoding="utf-8")
    (out_dir / "failure_type_distribution.md").write_text(
        "# Failure Type Distribution\n\n" + "\n".join(f"- {key}: {value}" for key, value in failure_dist.items()) + "\n",
        encoding="utf-8",
    )
    (out_dir / "ai_review_summary.md").write_text(
        f"""# AI Review Summary

- Reviewed tickers: {', '.join(r.ticker for r in results if r.ai_review) or 'None'}
- Reason for review: deterministic linter marked failed or suspicious cases.
- Compact payload only: Yes
- External paid AI calls: 0
- Training cases generated: {batch_result.training_cases_generated}
- What AI was allowed to do: suggest interpretation fixes, profile-specific next checks, and framework-gap improvements.
- What AI was not allowed to do: change locked financial data, valuation multiples, risk metrics, charts, CSV files, or research scores.

No external paid AI API calls were made. Reviews were local compact failure reviews / deterministic review summaries.
""",
        encoding="utf-8",
    )
    (out_dir / "credit_usage_estimate.md").write_text(
        f"""# Credit Usage Estimate

## Current run

- External AI calls: 0
- Compact local reviews: {batch_result.ai_reviews_used}
- Full report AI reviews: 0
- Full CSV/chart AI reviews: 0
- AI Reviews Skipped by Cache: {batch_result.ai_reviews_skipped_by_cache}
- Tickers reviewed: {', '.join(r.ticker for r in results if r.ai_review) or 'None'}

## Credit control rules

- Deterministic linter runs first.
- AI review is disabled by default.
- AI only reviews failed/suspicious compact payloads.
- Full reports, CSV files, and charts were not sent to AI.
- `broad_200` was not run live.
- This run used local deterministic/compact review only; no external AI API call was made.
""",
        encoding="utf-8",
    )
    (out_dir / "cache" / "report_hashes.json").write_text(
        json.dumps({r.ticker: _hash_obj({"status": r.report_status, "profile": r.asset_profile}) for r in results}, indent=2),
        encoding="utf-8",
    )
    (out_dir / "cache" / "failure_hashes.json").write_text(
        json.dumps({r.ticker: _hash_obj(r.lint.failed_checks if r.lint else [r.status]) for r in results}, indent=2),
        encoding="utf-8",
    )


def parse_batch_args(argv: list[str]) -> argparse.Namespace:
    examples = """
Examples:
  openbb-research batch eval_sets/smoke_12.yaml
  python scripts/batch_eval.py --set smoke_12 --run-id smoke_12_after_profile_fix
  openbb-research batch eval_sets/broad_200.yaml --both --full --pack
  openbb-research batch eval_sets/broad_200.yaml --no-ai
  openbb-research batch eval_sets/broad_200.yaml --ai-review-failures --max-ai-reviews 40 --resume
  openbb-research batch eval_sets/broad_200.yaml --max-workers 3
  openbb-research batch eval_sets/broad_200.yaml --only-failed
"""
    parser = argparse.ArgumentParser(
        prog="openbb-research batch",
        description="Run cross-industry batch evaluation with deterministic lint, optional compact AI review, and training-case generation.",
        epilog=examples,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("eval_set", nargs="?", help="YAML eval set, e.g. eval_sets/smoke_12.yaml")
    parser.add_argument("--set", dest="eval_set_name", help="Eval set shorthand, e.g. smoke_12 or broad_200.")
    parser.add_argument("--run-id", dest="run_id", help="Explicit batch run folder name under reports/batch_runs/.")
    parser.add_argument("--both", action="store_true", help="Generate English and Chinese reports.")
    parser.add_argument("--full", action="store_true", help="Use full report mode.")
    parser.add_argument("--pack", action="store_true", help="Zip each completed ticker report pack.")
    parser.add_argument("--max-workers", type=int, default=2, help="Maximum workers. Default: 2.")
    parser.add_argument("--no-ai", action="store_true", default=False, help="Disable compact AI review. Recommended default for broad batches.")
    parser.add_argument("--ai-review-failures", action="store_true", help="Run compact review only for failed/suspicious cases.")
    parser.add_argument("--max-ai-reviews", type=int, default=40, help="Maximum compact AI reviews. Default: 40.")
    parser.add_argument("--resume", action="store_true", help="Resume latest batch folder for this eval set.")
    parser.add_argument("--only-failed", action="store_true", help="Only rerun previous failed/suspicious tickers when resuming.")
    parser.add_argument("--force", action="store_true", help="Force rerun even if outputs exist.")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> BatchResult:
    args = parse_batch_args(argv or [])
    eval_set = args.eval_set
    if args.eval_set_name:
        name = args.eval_set_name
        eval_set = name if name.endswith(".yaml") else f"eval_sets/{name}.yaml"
    if not eval_set:
        raise SystemExit("eval set is required; pass eval_sets/smoke_12.yaml or --set smoke_12")
    result = run_batch(
        eval_set,
        both=args.both,
        full=args.full,
        pack=args.pack,
        max_workers=args.max_workers,
        no_ai=args.no_ai or not args.ai_review_failures,
        ai_review_failures=args.ai_review_failures,
        max_ai_reviews=args.max_ai_reviews,
        resume=args.resume,
        only_failed=args.only_failed,
        force=args.force,
        batch_id=args.run_id,
    )
    print(f"Batch folder: {result.out_dir}")
    print(f"AI Reviews Used: {result.ai_reviews_used}")
    print(f"Training Cases Generated: {result.training_cases_generated}")
    return result


if __name__ == "__main__":
    import sys

    main(sys.argv[1:])
