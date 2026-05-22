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

    def test_output_dir_defaults_to_latest(self):
        path = tool.output_dir_for_run(
            output=Path("reports"),
            symbol="AAPL",
            benchmark="SPY",
            start_date="2023-01-01",
            end_date=None,
            archive=False,
            run_id=None,
        )

        self.assertEqual(path, Path("reports/AAPL/latest"))

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

        self.assertIn("No obvious data-quality warnings", section)
        self.assertNotIn("No automatic data warnings triggered", section)

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


if __name__ == "__main__":
    unittest.main()
