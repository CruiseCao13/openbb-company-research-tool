import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

TERMINAL_UI_PATH = SCRIPTS / "terminal_ui.py"
SPEC = importlib.util.spec_from_file_location("terminal_ui", TERMINAL_UI_PATH)
terminal_ui = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(terminal_ui)


class TerminalUITests(unittest.TestCase):
    def test_plain_mode_does_not_crash(self):
        self.assertTrue(terminal_ui.plain_mode_does_not_crash())

    def test_public_functions_do_not_crash_in_plain_mode(self):
        terminal_ui.configure(enabled=False)
        terminal_ui.print_app_banner("3.0.0")
        terminal_ui.print_run_config("AAPL", "SPY", ai_review=True, archive_enabled=True, model="gpt-4o-mini")
        terminal_ui.step_done("[1/8] Fetching market data")
        terminal_ui.step_warn("[7/8] Running AI review", "OPENAI_API_KEY not found.")
        terminal_ui.step_error("[7/8] Running AI review", "test error")
        terminal_ui.print_ai_review_status("skipped", model="gpt-4o-mini", error="OPENAI_API_KEY not found.")


if __name__ == "__main__":
    unittest.main()
