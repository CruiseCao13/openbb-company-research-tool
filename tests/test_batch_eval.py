from __future__ import annotations

import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import batch_eval  # noqa: E402


def _write_run_folder(base: Path, ticker: str, run_id: str, *, outcome: str = "pass", patch: bool = True, profile_override: dict | None = None) -> Path:
    run_dir = base / ticker / "runs" / run_id
    for folder in ["report", "charts", "data", "audit", "ai", "dashboard", "metadata", "self_review"]:
        (run_dir / folder).mkdir(parents=True, exist_ok=True)
    (run_dir / "README.md").write_text("readme", encoding="utf-8")
    (run_dir / "charts" / "Figure_01_price_actual.png").write_bytes(b"png")
    profile = profile_override or {
        "primary_profile": "Mature Compounder" if outcome == "pass" else "Unknown / Data-Limited Screening",
        "secondary_profile": "Biotech-like Screening" if outcome != "pass" else "",
        "framework_coverage_level": "FULL" if outcome == "pass" else "SCREENING_ONLY",
        "dominant_metric_set": ["pipeline", "clinical trial", "cash runway", "dilution"] if outcome != "pass" else ["free cash flow", "margin durability"],
        "data_deficit_flags": ["pipeline stage"] if outcome != "pass" else [],
        "business_model_clues": ["biotech platform"] if outcome != "pass" else [],
        "suggested_framework_extension": "Biotech Research Profile" if outcome != "pass" else "",
    }
    status = {
        "DATA_VERIFICATION_STATUS": "PASS",
        "THESIS_VERIFICATION_STATUS": "PASS" if outcome == "pass" else "UNVERIFIED",
        "OVERALL_REPORT_STATUS": "PASS" if outcome == "pass" else "UNVERIFIED",
        "LIFECYCLE_LOGIC_STATUS": "PASS",
        "PATCH_STATUS": "APPLIED" if patch else "NOT_NEEDED",
        "PATCH_MATERIALITY_STATUS": "MATERIAL_PATCH_APPLIED" if patch else "NO_MATERIAL_CHANGE",
    }
    (run_dir / "metadata" / "asset_profile.json").write_text(json.dumps(profile), encoding="utf-8")
    (run_dir / "metadata" / "report_status.json").write_text(json.dumps(status), encoding="utf-8")
    (run_dir / "audit" / "lifecycle_logic_report.md").write_text("# Lifecycle Logic\n\nPASS\n", encoding="utf-8")
    (run_dir / "self_review" / "system_self_review.md").write_text("# Self Review\n", encoding="utf-8")
    gap_text = "# Framework Gap Analysis\n\nSuspected Industry: clinical-stage biotech.\n\nMissing Metrics: pipeline stage, FDA milestone, cash runway.\n"
    if outcome == "pass":
        gap_text = "# Framework Gap Analysis\n\nCoverage Level: FULL.\n"
    (run_dir / "self_review" / "framework_gap_analysis.md").write_text(gap_text, encoding="utf-8")
    if patch:
        patch_data = {
            "patch_records": [
                {
                    "block_id": "next_checks",
                    "old_text": "old",
                    "new_text": "new profile specific pipeline clinical cash runway dilution",
                    "materiality": "MATERIAL_PATCH_APPLIED",
                }
            ]
        }
        (run_dir / "ai" / "correction_patch.json").write_text(json.dumps(patch_data), encoding="utf-8")
        (run_dir / "ai" / "patch_diff_log.md").write_text("Old Text\nNew Text\nMateriality: MATERIAL_PATCH_APPLIED\n", encoding="utf-8")
    report = f"""# {ticker} Equity Research Report

## Table of Contents
- [1. Report Status Card](#1-report-status-card)
- [2. One-line Verdict](#2-one-line-verdict)
- [3. Next Research Steps](#3-next-research-steps)

## 1. Report Status Card

| Metric | Value |
|---|---|
| Price | 24.06 |

![Figure 1](../charts/Figure_01_price_actual.png)

## 2. One-line Verdict

This is a clean first-pass report.

## 3. Next Research Steps

- Check pipeline stage, clinical trial timing, cash runway, and dilution risk.
- Verify FDA milestone cadence and partnership revenue quality.
- Reconcile data deficits with latest filings.
"""
    (run_dir / "report" / f"{ticker}_research_report.md").write_text(report, encoding="utf-8")
    return run_dir


def _fake_runner_factory(base: Path):
    def fake_runner(**kwargs):
        ticker = kwargs["symbol"]
        run_id = kwargs["run_id"]
        if ticker == "BADFETCH":
            raise TimeoutError("provider timeout")
        outcome = "fail" if ticker in {"RXRX", "BAD"} else "pass"
        run_dir = _write_run_folder(base, ticker, run_id, outcome=outcome)
        return {"symbol": ticker, "folder": str(run_dir), "score": 50.0}
    return fake_runner


def _fake_packer(run_dir: Path) -> Path:
    zip_path = run_dir / f"{run_dir.parents[1].name}_research_pack.zip"
    with zipfile.ZipFile(zip_path, "w") as archive:
        archive.writestr("README.md", "readme")
        for folder in ["report", "charts", "data", "audit", "ai", "dashboard", "metadata", "self_review"]:
            archive.writestr(f"{folder}/.keep", "x")
    return zip_path


def test_smoke_eval_set_loads():
    tickers = batch_eval.load_eval_set(str(ROOT / "eval_sets" / "smoke_12.yaml"))
    assert len(tickers) == 12
    assert {"AAPL", "RKLB", "INTC", "RXRX"}.issubset(tickers)


def test_broad_200_eval_set_loads():
    tickers = batch_eval.load_eval_set(str(ROOT / "eval_sets" / "broad_200.yaml"))
    assert len(tickers) == 200
    assert {"AAPL", "RXRX", "INTC", "O", "PGR", "ZIM", "NEE"}.issubset(tickers)


def test_batch_runner_smoke_creates_summary(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - AAPL\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), both=True, pack=True, no_ai=True, runner=_fake_runner_factory(tmp_path / "reports"), packer=_fake_packer)
    out = Path(result.out_dir)
    assert (out / "batch_summary.md").exists()
    assert (out / "batch_summary.csv").exists()
    assert (out / "failures.md").exists()
    assert result.training_cases_generated >= 1


def test_batch_runner_respects_max_ai_reviews(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n  - BAD\n", encoding="utf-8")
    result = batch_eval.run_batch(
        str(eval_set),
        ai_review_failures=True,
        no_ai=False,
        max_ai_reviews=1,
        runner=_fake_runner_factory(tmp_path / "reports"),
    )
    assert result.ai_reviews_used == 1


def test_batch_uses_compact_payload_for_ai(tmp_path):
    run_dir = _write_run_folder(tmp_path / "reports", "RXRX", "run1", outcome="fail")
    lint = batch_eval.lint_run_folder(run_dir)
    payload = batch_eval.build_compact_ai_review_payload(run_dir, lint)
    assert "bad_sections" in payload
    assert "report/" not in json.dumps(payload)
    assert "clinical trial" in json.dumps(payload)


def test_ai_review_not_called_when_no_ai(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), no_ai=True, ai_review_failures=True, runner=_fake_runner_factory(tmp_path / "reports"))
    assert result.ai_reviews_used == 0


def test_batch_can_resume(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - AAPL\n", encoding="utf-8")
    first = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    second = batch_eval.run_batch(str(eval_set), resume=True, runner=_fake_runner_factory(tmp_path / "reports"))
    assert first.out_dir == second.out_dir


def test_ai_review_failures_resume_reuses_existing_runs(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n", encoding="utf-8")
    first = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))

    def exploding_runner(**kwargs):
        raise AssertionError("resume AI review should not rerun provider fetch")

    second = batch_eval.run_batch(
        str(eval_set),
        resume=True,
        ai_review_failures=True,
        no_ai=False,
        max_ai_reviews=1,
        runner=exploding_runner,
    )
    assert first.out_dir == second.out_dir
    assert second.ai_reviews_used == 1


def test_batch_generates_training_cases(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    assert result.training_cases_generated == 1
    assert (Path(result.out_dir) / "training_cases_generated.jsonl").read_text().strip()
    assert (tmp_path / "training_cases" / "corrections" / "correction_cases.jsonl").exists()


def test_training_case_schema_valid(tmp_path):
    ticker_result = batch_eval.TickerRunResult(
        ticker="RXRX",
        status="COMPLETED_WITH_WARNINGS",
        asset_profile={"primary_profile": "Biotech-like Screening", "dominant_metric_set": ["pipeline"]},
        report_status={"OVERALL_REPORT_STATUS": "UNVERIFIED"},
    )
    lint = batch_eval.LintResult(ticker="RXRX", status="FAIL", failed_checks=["next_checks_too_generic"])
    case = batch_eval.generate_training_case(ticker_result, lint, {"suggested_profile": "Biotech-like Screening"})
    for key in ["ticker", "detected_profile", "expected_profile", "failed_checks", "must_contain", "reason"]:
        assert key in case


def test_batch_summary_has_failure_distribution(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "failure_type_distribution.md").read_text()
    assert "Failure Type Distribution" in text


def test_batch_summary_md_has_dashboard_sections(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - AAPL\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "batch_summary.md").read_text()
    for heading in [
        "Batch Evaluation Report",
        "Executive Summary",
        "Status Dashboard",
        "What Worked",
        "Actual Failures",
        "Reports Requiring Review",
        "Expected Unverified",
        "Failure Type Distribution",
        "Profile Distribution",
        "Generated Training Cases",
        "Credit / AI Usage",
        "Known Limitations",
    ]:
        assert heading in text


def test_batch_profile_matches_single_run_profile_for_intc(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ngroups:\n  semiconductor_turnaround:\n    - INTC\n", encoding="utf-8")
    intc_profile = {
        "primary_profile": "Capital-Intensive Semiconductor Turnaround",
        "secondary_profile": "Hybrid / Technology Manufacturing",
        "framework_coverage_level": "PARTIAL",
        "dominant_metric_set": ["foundry execution", "capex intensity", "gross margin recovery"],
        "data_deficit_flags": ["foundry revenue / margin", "capex roadmap"],
        "business_model_clues": ["Semiconductor / manufacturing turnaround"],
        "suggested_framework_extension": "Semiconductor / Manufacturing Turnaround Research Profile",
    }

    def fake_runner(**kwargs):
        run_dir = _write_run_folder(tmp_path / "reports", "INTC", kwargs["run_id"], profile_override=intc_profile)
        return {"symbol": "INTC", "folder": str(run_dir), "score": 50.0}

    result = batch_eval.run_batch(str(eval_set), runner=fake_runner)
    assert result.results[0].asset_profile["primary_profile"] == "Capital-Intensive Semiconductor Turnaround"


def test_failures_md_has_failure_detail_sections(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - BADFETCH\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "failures.md").read_text()
    for heading in ["What happened", "Error Type", "Why it matters", "Suggested next action", "Training case"]:
        assert heading in text


def test_credit_usage_estimate_mentions_external_ai_calls(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), ai_review_failures=True, no_ai=False, max_ai_reviews=1, runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "credit_usage_estimate.md").read_text()
    assert "External AI calls: 0" in text
    assert "local deterministic/compact review only" in text
    assert "broad_200" in text


def test_profile_distribution_md_lists_profiles(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - AAPL\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "profile_distribution.md").read_text()
    assert "Why this matters" in text
    assert "Primary Profile" in text
    assert "Secondary Profile" in text
    assert "Framework Coverage" in text
    assert "Suggested Extension" in text


def test_batch_summary_does_not_label_expected_unverified_as_failed(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ngroups:\n  biotech_like:\n    - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "batch_summary.md").read_text()
    actual_failures = text.split("## 4. Actual Failures", 1)[1].split("## 5. Reports Requiring Review", 1)[0]
    assert "RXRX" not in actual_failures
    assert "RXRX" in text.split("## 5. Reports Requiring Review", 1)[1]


def test_training_case_does_not_use_low_confidence_detected_profile_as_expected():
    ticker_result = batch_eval.TickerRunResult(
        ticker="MYST",
        status="COMPLETED_WITH_WARNINGS",
        asset_profile={
            "primary_profile": "Unknown / Data-Limited Screening",
            "secondary_profile": "Biotech-like Screening",
            "sector_confidence": "LOW",
            "framework_coverage_level": "SCREENING_ONLY",
            "dominant_metric_set": ["pipeline stage"],
        },
        report_status={"OVERALL_REPORT_STATUS": "UNVERIFIED"},
    )
    lint = batch_eval.LintResult(ticker="MYST", status="WARNING")
    case = batch_eval.generate_training_case(ticker_result, lint, None)
    assert case["expected_profile"] == "HUMAN_REVIEW_REQUIRED"
    assert case["human_review_required"] is True

    ticker_result.expected_profile_family = "Shipping / Airlines / Transport"
    conflict_case = batch_eval.generate_training_case(ticker_result, lint, None)
    assert conflict_case["profile_conflict"] is True
    assert "freight rate / yield" in conflict_case["must_contain"]
    assert "pipeline" in conflict_case["must_not_contain"]


def test_training_cases_are_linked_in_summary(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - RXRX\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    text = (Path(result.out_dir) / "batch_summary.md").read_text()
    assert "training_cases/generated/RXRX_" in text


def test_unknown_with_business_clues_becomes_training_case(tmp_path):
    run_dir = _write_run_folder(tmp_path / "reports", "RXRX", "run1", outcome="fail")
    lint = batch_eval.lint_run_folder(run_dir)
    ticker_result = batch_eval.TickerRunResult(ticker="RXRX", status="COMPLETED_WITH_WARNINGS", asset_profile=batch_eval._read_json(run_dir / "metadata" / "asset_profile.json"), report_status=batch_eval._read_json(run_dir / "metadata" / "report_status.json"), lint=lint)
    assert batch_eval._result_needs_training(ticker_result)


def test_patch_materiality_failure_generates_case(tmp_path):
    run_dir = _write_run_folder(tmp_path / "reports", "RXRX", "run1", outcome="fail", patch=True)
    patch = {"patch_records": [{"old_text": "same", "new_text": "same", "materiality": "CONFIRMED_NO_CHANGE"}]}
    (run_dir / "ai" / "correction_patch.json").write_text(json.dumps(patch), encoding="utf-8")
    lint = batch_eval.lint_run_folder(run_dir)
    assert "patch_not_material" in lint.failed_checks


def test_next_checks_generic_generates_case(tmp_path):
    run_dir = _write_run_folder(tmp_path / "reports", "RXRX", "run1", outcome="fail")
    report = run_dir / "report" / "RXRX_research_report.md"
    report.write_text(report.read_text().replace("Check pipeline stage, clinical trial timing, cash runway, and dilution risk.", "Check latest filing and primary source."), encoding="utf-8")
    lint = batch_eval.lint_run_folder(run_dir)
    assert "next_checks_too_generic" in lint.failed_checks


def test_fetch_failure_does_not_crash_batch(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    eval_set = tmp_path / "mini.yaml"
    eval_set.write_text("name: mini\ntickers:\n  - BADFETCH\n", encoding="utf-8")
    result = batch_eval.run_batch(str(eval_set), runner=_fake_runner_factory(tmp_path / "reports"))
    assert result.results[0].status == "FETCH_FAILED"
    assert (Path(result.out_dir) / "batch_summary.md").exists()


def test_pack_missing_is_detected(tmp_path):
    run_dir = _write_run_folder(tmp_path / "reports", "AAPL", "run1", outcome="pass")
    lint = batch_eval.lint_run_folder(run_dir, pack_expected=True)
    assert "pack_missing" in lint.failed_checks
