import importlib.util
import os
import sys
import unittest
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

AI_REVIEW_PATH = SCRIPTS / "ai_review.py"
SPEC = importlib.util.spec_from_file_location("ai_review", AI_REVIEW_PATH)
ai_review = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ai_review)

TOOL_PATH = SCRIPTS / "company_research_tool.py"
TOOL_SPEC = importlib.util.spec_from_file_location("company_research_tool", TOOL_PATH)
tool = importlib.util.module_from_spec(TOOL_SPEC)
TOOL_SPEC.loader.exec_module(tool)


class AIReviewTests(unittest.TestCase):
    def test_build_ai_review_payload_includes_required_fields(self):
        report_data = {
            "ticker": "AAPL",
            "benchmark": "SPY",
            "version": "3.0.0",
            "research_profile": "Mature Compounder",
            "research_status": "Watchlist",
            "one_line_verdict": "AAPL needs valuation discipline.",
            "key_takeaways": ["AAPL outperformed but valuation is demanding."],
            "beginner_summary": [{"Area": "Valuation", "Status": "Expensive"}],
            "price_metrics": [{"Metric": "Total Return", "Target": 1.0}],
            "growth_quality_metrics": [{"Metric": "Revenue CAGR", "Value": 0.02}],
            "valuation_snapshot": [{"Metric": "trailingPE", "Value": 30}],
            "ruin_risk_metrics": [{"Metric": "Balance Sheet Resilience Score", "Value": 60}],
            "research_score": {"score": 61.0, "status": "Watchlist"},
            "score_components": [{"Component": "Research Score", "Score": 61.0}],
            "sanity_checks": [],
            "data_confidence": "Medium",
            "manual_verification": ["10-K"],
            "final_research_questions": ["Why not buy SPY?"],
        }

        payload = ai_review.build_ai_review_payload(report_data)

        for key in [
            "ticker",
            "benchmark",
            "research_status",
            "price_metrics",
            "research_score",
            "manual_verification",
            "final_research_questions",
        ]:
            self.assertIn(key, payload)

    def test_render_ai_review_markdown_renders_mocked_review(self):
        review = ai_review.AIReviewResult(
            analyst_summary="The report is cautious and evidence-based.",
            evidence_check=["Return beat benchmark, but Sharpe was weaker."],
            main_risks=["Valuation depends on continued execution."],
            possible_beginner_misreadings=["Score is not a buy signal."],
            what_to_verify_next=["Latest 10-K revenue mix."],
            beginner_translation="Good business evidence, but not a simple bargain.",
            confidence_note="Use primary filings before relying on the conclusion.",
        )

        markdown = ai_review.render_ai_review_markdown(review)

        self.assertIn("## AI Review", markdown)
        self.assertIn("Analyst Summary", markdown)
        self.assertIn("Score is not a buy signal", markdown)

    def test_render_ai_review_skipped(self):
        markdown = ai_review.render_ai_review_skipped("OPENAI_API_KEY not found.")

        self.assertIn("AI Review was requested but skipped", markdown)
        self.assertIn("OPENAI_API_KEY not found", markdown)

    def test_missing_api_key_does_not_crash(self):
        with patch.dict(os.environ, {}, clear=True):
            result = ai_review.call_ai_review({"ticker": "AAPL"})

        self.assertIsNone(result)
        self.assertIn("OPENAI_API_KEY", ai_review.call_ai_review.last_error)

    def test_argparse_recognizes_ai_flags(self):
        argv = [
            "cresearch",
            "AAPL",
            "--ai-review",
            "--ai-model",
            "gpt-4o-mini",
            "--ai-review-depth",
            "deep",
            "--ai-timeout",
            "30",
            "--ai-max-output-tokens",
            "800",
            "--no-rich",
        ]

        with patch.object(sys, "argv", argv):
            args = tool.parse_args()

        self.assertTrue(args.ai_review)
        self.assertEqual(args.ai_model, "gpt-4o-mini")
        self.assertEqual(args.ai_review_depth, "deep")
        self.assertEqual(args.ai_timeout, 30)
        self.assertEqual(args.ai_max_output_tokens, 800)
        self.assertTrue(args.no_rich)


if __name__ == "__main__":
    unittest.main()
