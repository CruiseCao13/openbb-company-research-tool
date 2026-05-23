import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import pandas as pd


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

V4_PATH = SCRIPTS / "v4_workflow.py"
SPEC = importlib.util.spec_from_file_location("v4_workflow", V4_PATH)
v4 = importlib.util.module_from_spec(SPEC)
sys.modules["v4_workflow"] = v4
SPEC.loader.exec_module(v4)

TOOL_PATH = SCRIPTS / "company_research_tool.py"
TOOL_SPEC = importlib.util.spec_from_file_location("company_research_tool", TOOL_PATH)
tool = importlib.util.module_from_spec(TOOL_SPEC)
TOOL_SPEC.loader.exec_module(tool)


class V4WorkflowTests(unittest.TestCase):
    def test_payload_has_no_free_text_warnings(self):
        payload = {
            "fcf_confidence": "STAGE_2_UNVERIFIED",
            "source_type": "PROVIDER_DATA",
            "warning_enum": "PRIMARY_FILING_REQUIRED",
        }
        self.assertTrue(v4.no_free_text_warning_payload(payload))
        bad = {"warning_enum": "Medium, verify with 10-K"}
        self.assertFalse(v4.no_free_text_warning_payload(bad))

    def test_banned_phrases_lint(self):
        en = v4.lint_language("Investors should monitor this because further research is needed.", "en")
        zh = v4.lint_language("该公司显示出较强的盈利能力，未来动向需要进一步观察。", "zh")
        self.assertGreaterEqual(len(en["banned_phrase_hits"]), 1)
        self.assertGreaterEqual(len(zh["banned_phrase_hits"]), 1)

    def test_language_lint_report_generation(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            v4.write_language_lint_report(Path(tmp_dir), [v4.lint_language("Clean sentence.", "en")])
            self.assertTrue((Path(tmp_dir) / "language_lint_report.md").exists())

    def test_correction_log_schema(self):
        log = v4.build_ai_correction_log(self._report_data(), "en")
        self.assertEqual(v4.validate_ai_correction_log(log), "PASS")
        for correction in log["report_corrections"]:
            self.assertIn("section", correction)
            self.assertIn("original_issue", correction)
            self.assertIn("suggested_revision", correction)
            self.assertIn(correction["severity"], {"LOW", "MEDIUM", "HIGH"})
            self.assertIsInstance(correction["requires_data_verification"], bool)
            self.assertIn(correction["evidence_boundary"], {"FROM_PAYLOAD", "NEEDS_EXTERNAL_VERIFICATION", "NOT_ALLOWED"})

    def test_next_checks_and_stance_wording(self):
        en = v4.build_ai_correction_log(self._report_data(), "en")
        zh = v4.build_ai_correction_log(self._report_data(), "zh")
        self.assertEqual(len(en["next_3_checks"]), 3)
        self.assertIn("This is not a buy/sell recommendation. It is a research stance.", en["research_stance"])
        self.assertIn("这不是买卖建议，而是研究立场。", zh["research_stance"])

    def test_answerability_gate_honesty(self):
        result = v4.classify_answerability("Will AAPL double next year?", self._report_data())
        self.assertNotEqual(result["status"], "ANSWERABLE_FROM_REPORT")
        self.assertEqual(result["status"], "NOT_ANSWERABLE_FROM_CURRENT_DATA")
        self.assertIn("does not predict short-term returns", result["short_answer"])
        segment = v4.classify_answerability("Is Services carrying valuation?", self._report_data())
        self.assertEqual(segment["status"], "PARTIALLY_ANSWERABLE_NEEDS_VERIFICATION")
        self.assertEqual(segment["evidence_boundary"], "NEEDS_EXTERNAL_VERIFICATION")

    def test_ai_corrections_stay_within_payload_boundary(self):
        log = v4.build_ai_correction_log(self._report_data(), "en")
        joined = "\n".join(str(item) for item in log["report_corrections"])
        self.assertIn("[SEGMENT_DATA_MISSING]", joined)
        self.assertIn("Segment data is missing", joined)
        self.assertNotIn("Services revenue grew", joined)

    def test_no_self_correction_loop(self):
        log = v4.build_ai_correction_log(self._report_data(), "en")
        self.assertEqual(log["max_correction_passes"], 1)
        with tempfile.TemporaryDirectory() as tmp_dir:
            v4.write_language_lint_report(Path(tmp_dir), [v4.lint_language("Clean sentence.", "en")])
            text = (Path(tmp_dir) / "language_lint_report.md").read_text()
        self.assertIn("max_language_rewrite_attempts: 3", text)

    def test_unverified_auxiliary_log_naming(self):
        self.assertEqual(v4.artifact_filename("ai_correction_log.md", "UNVERIFIED"), "UNVERIFIED_ai_correction_log.md")
        self.assertEqual(v4.artifact_filename("language_lint_report.md", "UNVERIFIED"), "UNVERIFIED_language_lint_report.md")

    def test_placeholder_not_translated_for_chinese_pipeline(self):
        log = v4.build_ai_correction_log(self._report_data(), "zh")
        joined = "\n".join(str(item) for item in log["report_corrections"])
        self.assertIn("[SEGMENT_DATA_MISSING]", joined)
        self.assertNotIn("[业务线数据缺失]", joined)

    def test_battle_card_constraints(self):
        report_data = self._report_data()
        card = v4.render_battle_card(report_data, "en")
        for section in ["The Long Bet", "The Short Trigger", "Market Pricing"]:
            text = card.split(f"### {section}", 1)[1].split("###", 1)[0]
            sentences = [line for line in text.splitlines() if line.strip()]
            self.assertLessEqual(len(sentences), 3)
        verification = card.split("### Verification Priority", 1)[1]
        self.assertEqual(sum(1 for line in verification.splitlines() if line.strip().startswith(("1.", "2.", "3."))), 3)

    def test_risk_method_gate(self):
        ok = v4.RiskMethod(price_field="adj_close", annualization_days=252, risk_free_rate=0.0, benchmark="SPY")
        bad = v4.RiskMethod(price_field="", annualization_days=None, risk_free_rate=None, benchmark="SPY")
        self.assertEqual(v4.risk_method_status(ok), "PASS")
        self.assertEqual(v4.risk_method_status(bad), "FAIL")

    def test_data_audit_gate(self):
        report_data = self._report_data()
        audit = v4.build_data_audit(report_data, v4.RiskMethod())
        self.assertIn("Free Cash Flow", set(audit["metric_name"]))
        self.assertIn("Revenue CAGR", set(audit["metric_name"]))
        with tempfile.TemporaryDirectory() as tmp_dir:
            v4.write_data_audit(Path(tmp_dir), audit)
            self.assertTrue((Path(tmp_dir) / "data_audit.md").exists())
            self.assertTrue((Path(tmp_dir) / "data_audit.csv").exists())

    def test_price_label_sanity_check(self):
        price = pd.DataFrame(
            {"close": [100.0, 100.0], "adj_close": [100.0, 100.0]},
            index=pd.to_datetime(["2024-01-01", "2024-01-02"]),
        )
        check = v4.build_price_label_sanity_check("AAPL", price, chart_label_value=103.0, provider_latest_quote=100.0)
        self.assertEqual(check.iloc[0]["status"], "FAIL")

    def test_chinese_report_mode_argparse(self):
        with patch.object(sys, "argv", ["cresearch", "AAPL", "--cn"]):
            args = tool.parse_args()
        self.assertTrue(args.cn)
        with patch.object(sys, "argv", ["cresearch", "AAPL", "--chinese"]):
            args = tool.parse_args()
        self.assertTrue(args.cn)

    def test_bilingual_independence(self):
        with patch.object(sys, "argv", ["cresearch", "AAPL", "--cn", "--no-rich"]):
            args = tool.parse_args()
        self.assertTrue(args.cn)

    def test_valuation_sensitivity(self):
        text = v4.render_valuation_sensitivity(self._report_data(), "en")
        self.assertIn("Valuation Sensitivity", text)
        self.assertIn("This is not a price target", text)

    def test_score_direction_explanation(self):
        report_data = self._report_data()
        text = v4.render_battle_card(report_data, "en") + v4.render_battle_card(report_data, "zh")
        resilience = report_data["raw"]["ruin_risk"]
        self.assertIn("Balance Sheet Resilience Score", set(resilience["Metric"]))
        self.assertIn("买入的核心赌注", text)

    def test_missing_metric_placeholder_registry(self):
        self.assertEqual(v4.fmt_value(None), "[METRIC_MISSING_RAW]")
        payload = {"segment": "[SEGMENT_DATA_MISSING]", "method": "[METHOD_ASSUMPTION_MISSING]"}
        self.assertFalse(v4.payload_has_loose_missing_strings(payload))
        bad_payload = {"segment": "N/A"}
        self.assertTrue(v4.payload_has_loose_missing_strings(bad_payload))

    def test_ai_does_not_reason_from_missing_placeholder(self):
        text = v4.ai_response_for_missing_placeholder("[SEGMENT_DATA_MISSING]", topic="Segment data")
        self.assertIn("Segment data is missing", text)
        self.assertIn("NEEDS_EXTERNAL_VERIFICATION", text)
        self.assertNotIn("grew", text)

    def test_unverified_filename_on_gate_fail(self):
        name = v4.report_filename("AAPL", "research_report.md", "UNVERIFIED")
        self.assertEqual(name, "UNVERIFIED_AAPL_research_report.md")

    def test_overall_report_status_rules(self):
        self.assertEqual(v4.overall_report_status({"DATA_AUDIT_STATUS": "PASS", "RISK_METHOD_STATUS": "PASS"}), "VERIFIED")
        self.assertEqual(v4.overall_report_status({"DATA_AUDIT_STATUS": "WARNING", "RISK_METHOD_STATUS": "PASS"}), "WARNING")
        self.assertEqual(v4.overall_report_status({"DATA_AUDIT_STATUS": "FAIL", "RISK_METHOD_STATUS": "PASS"}), "UNVERIFIED")

    def test_cli_warning_on_gate_fail(self):
        self.assertEqual(v4.overall_report_status({"LANGUAGE_LINT_STATUS": "FAIL"}), "UNVERIFIED")

    def test_ai_layer_cannot_mutate_payload(self):
        payload = {"Revenue": 416.16, "FCF": 98.77, "PE": 37.43}
        suggestions = {"ai_correction_log": ["Check provider data."]}
        self.assertTrue(v4.ai_layer_can_only_suggest(payload, suggestions))

    def _report_data(self):
        fundamental = pd.DataFrame(
            [
                {"Metric": "Revenue CAGR", "Value": 0.018},
                {"Metric": "FCF Margin Latest", "Value": 0.23},
            ]
        )
        valuation = pd.DataFrame(
            [
                {"Metric": "trailingPE", "Value": 37.4},
                {"Metric": "priceToSalesTrailing12Months", "Value": 10.0},
            ]
        )
        price = pd.DataFrame(
            [
                {"Metric": "Total Return", "Target": 1.0, "Benchmark": 0.8, "Difference": 0.2},
                {"Metric": "Sharpe Ratio", "Target": 1.0, "Benchmark": 1.2, "Difference": -0.2},
            ]
        )
        score = pd.DataFrame([{"Component": "Research Score", "Score": 61.34, "Weight": 1.0}])
        ruin = pd.DataFrame([{"Metric": "Balance Sheet Resilience Score", "Value": 48.78}])
        return {
            "ticker": "AAPL",
            "benchmark": "SPY",
            "version": "4.0.0",
            "research_profile": "Mature Compounder",
            "research_status": "Watchlist",
            "start_date": "2023-01-01",
            "end_date": None,
            "raw": {
                "fundamental_summary": fundamental,
                "valuation": valuation,
                "price_summary": price,
                "score_table": score,
                "ruin_risk": ruin,
                "info": {"currency": "USD"},
            },
        }


if __name__ == "__main__":
    unittest.main()
