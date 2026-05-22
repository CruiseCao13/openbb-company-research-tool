import importlib.util
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "scripts" / "company_research_tool.py"
SPEC = importlib.util.spec_from_file_location("company_research_tool", SCRIPT_PATH)
tool = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tool)


class FormattingTests(unittest.TestCase):
    def test_metric_registry_formats_ratios_as_ratios(self):
        self.assertEqual(tool.format_value_by_metric("Sharpe Ratio", 0.56234), "0.5623")
        self.assertEqual(tool.format_value_by_metric("Beta vs Benchmark", 1.23456), "1.2346")

    def test_metric_registry_formats_percent_metrics_as_percentages(self):
        self.assertEqual(tool.format_value_by_metric("Total Return", 1.5919), "159.19%")
        self.assertEqual(tool.format_value_by_metric("Gross Margin", 0.7674), "76.74%")

    def test_metric_registry_formats_count_metrics_as_integers(self):
        self.assertEqual(tool.format_value_by_metric("Positive FCF Years", 2.0), "2")

    def test_metric_registry_formats_share_counts_with_commas(self):
        self.assertEqual(tool.format_value_by_metric("sharesOutstanding", 1500000000), "1,500,000,000")


if __name__ == "__main__":
    unittest.main()
