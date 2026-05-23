import importlib.util
import tempfile
import unittest
from pathlib import Path

import pandas as pd


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "scripts" / "company_research_tool.py"
SPEC = importlib.util.spec_from_file_location("company_research_tool", SCRIPT_PATH)
tool = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tool)


class MetricsTests(unittest.TestCase):
    def test_total_return_uses_first_and_last_close(self):
        close = pd.Series(
            [100.0, 125.0, 259.19],
            index=pd.to_datetime(["2024-01-01", "2024-01-02", "2024-01-03"]),
        )

        self.assertEqual(round(tool.total_return(close), 4), 1.5919)

    def test_output_dir_defaults_to_runs_folder(self):
        path = tool.output_dir_for_run(
            output=Path("reports"),
            symbol="AAPL",
            benchmark="SPY",
            start_date="2023-01-01",
            end_date=None,
            archive=False,
            run_id=None,
        )

        self.assertEqual(path.parts[:3], ("reports", "AAPL", "runs"))
        self.assertIn("AAPL_vs_SPY", path.name)

    def test_output_dir_run_id_implies_runs_folder(self):
        path = tool.output_dir_for_run(
            output=Path("reports"),
            symbol="AAPL",
            benchmark="SPY",
            start_date="2023-01-01",
            end_date=None,
            archive=False,
            run_id="test_2023_start",
        )

        self.assertEqual(path, Path("reports/AAPL/runs/test_2023_start"))

    def test_empty_warnings_section_is_human_readable(self):
        section = tool.warnings_section([])

        self.assertIn("No additional warning rule", section)
        self.assertNotIn("No automatic data warnings triggered", section)

    def test_category_classifier_marks_speculative_growth(self):
        summary = pd.DataFrame(
            [
                {"Metric": "Revenue CAGR", "Value": 0.35},
                {"Metric": "FCF Margin Latest", "Value": -0.20},
            ]
        )

        self.assertEqual(tool.classify_research_category({"quoteType": "EQUITY"}, summary), "Speculative Growth")

    def test_margin_stress_builds_personal_cushion_table(self):
        table = tool.build_margin_stress(100000, 25000, [0.5])

        self.assertEqual(table.iloc[0]["Portfolio Value"], 50000)
        self.assertEqual(table.iloc[0]["Equity Cushion"], 25000)
        self.assertEqual(table.iloc[0]["Loan / Value"], 0.5)

    def test_actual_close_price_chart_is_written(self):
        close = pd.DataFrame(
            {
                "AAPL": [100.0, 102.0, 101.0],
                "SPY": [400.0, 401.0, 403.0],
            },
            index=pd.to_datetime(["2024-01-01", "2024-01-02", "2024-01-03"]),
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "actual_close_price_chart.png"
            tool.plot_actual_close_price(close, "AAPL", "SPY", path)

            self.assertTrue(path.exists())
            self.assertGreater(path.stat().st_size, 0)

    def test_generate_report_data_dict_contains_required_fields(self):
        profile = pd.DataFrame([{"Field": "symbol", "Value": "AAPL"}])
        valuation = pd.DataFrame([{"Metric": "trailingPE", "Value": 30.0}])
        trends = pd.DataFrame([{"Revenue": 100.0, "Free Cash Flow": 20.0}])
        fundamental_summary = pd.DataFrame(
            [
                {"Metric": "Revenue CAGR", "Value": 0.05},
                {"Metric": "Gross Margin Latest", "Value": 0.45},
                {"Metric": "FCF Margin Latest", "Value": 0.20},
            ]
        )
        price_summary = pd.DataFrame(
            [
                {"Metric": "Total Return", "Target": 0.20, "Benchmark": 0.10, "Difference": 0.10},
                {"Metric": "Sharpe Ratio", "Target": 1.0, "Benchmark": 1.2, "Difference": -0.2},
                {"Metric": "Max Drawdown", "Target": -0.20, "Benchmark": -0.10, "Difference": -0.10},
            ]
        )
        score_table = pd.DataFrame(
            [
                {"Component": "Research Score", "Score": 61.0, "Weight": 1.0, "Profile": "Mature Compounder"},
                {"Component": "Growth Score", "Score": 55.0, "Weight": 0.2, "Profile": "Mature Compounder"},
                {"Component": "Profitability Score", "Score": 75.0, "Weight": 0.2, "Profile": "Mature Compounder"},
                {"Component": "Valuation Sanity Score", "Score": 35.0, "Weight": 0.2, "Profile": "Mature Compounder"},
                {"Component": "Risk Control Score", "Score": 60.0, "Weight": 0.2, "Profile": "Mature Compounder"},
            ]
        )
        target_price = pd.DataFrame({"close": [100.0, 110.0]})
        sanity_checks = pd.DataFrame([{"Severity": "INFO", "Check": "No triggered sanity failure"}])
        ruin_risk = pd.DataFrame([{"Metric": "Ruin Risk Score", "Value": 45.0}])

        with tempfile.TemporaryDirectory() as tmp_dir:
            report_data = tool.generate_report_data_dict(
                symbol="AAPL",
                benchmark="SPY",
                start_date="2023-01-01",
                end_date=None,
                out_dir=Path(tmp_dir),
                profile=profile,
                valuation=valuation,
                trends=trends,
                fundamental_summary=fundamental_summary,
                price_summary=price_summary,
                score_table=score_table,
                info={"quoteType": "EQUITY", "trailingPE": 30.0},
                target_price=target_price,
                warnings=[],
                sanity_checks=sanity_checks,
                ruin_risk=ruin_risk,
                margin_stress=pd.DataFrame(),
                actual_chart_name="actual.png",
                chart_name="normalized.png",
                drawdown_chart_name="drawdown.png",
                score_components_chart_name="score.png",
                growth_quality_chart_name="growth.png",
                ruin_risk_chart_name="ruin.png",
                interactive_chart_name="dashboard.html",
                radar_chart_name="radar.html",
            )

        for key in [
            "ticker",
            "benchmark",
            "version",
            "research_profile",
            "research_status",
            "one_line_verdict",
            "key_takeaways",
            "beginner_summary",
            "price_metrics",
            "growth_quality_metrics",
            "valuation_snapshot",
            "ruin_risk_metrics",
            "research_score",
            "score_components",
            "sanity_checks",
            "data_confidence",
            "manual_verification",
            "final_research_questions",
            "generated_files",
        ]:
            self.assertIn(key, report_data)


if __name__ == "__main__":
    unittest.main()
