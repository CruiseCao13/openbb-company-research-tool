import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

import pandas as pd


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

SPEC = importlib.util.spec_from_file_location("asset_aware", SCRIPTS / "asset_aware.py")
asset_aware = importlib.util.module_from_spec(SPEC)
sys.modules["asset_aware"] = asset_aware
SPEC.loader.exec_module(asset_aware)

TOOL_SPEC = importlib.util.spec_from_file_location("company_research_tool", SCRIPTS / "company_research_tool.py")
company_tool = importlib.util.module_from_spec(TOOL_SPEC)
sys.modules["company_research_tool"] = company_tool
TOOL_SPEC.loader.exec_module(company_tool)


class AssetAwareRoutingTests(unittest.TestCase):
    def test_aapl_like_payload_routes_to_mature_compounder(self):
        profile = asset_aware.build_asset_profile(
            self._info("Technology", "Consumer Electronics"),
            self._fundamentals(revenue_cagr=0.018, latest_growth=0.064, operating_margin=0.32, fcf_margin=0.24, positive_ni=4, positive_fcf=4),
            self._valuation(pe=37.4, ps=10.0, ev_revenue=10.1),
            pd.DataFrame(),
            self._ruin(cash_runway=None),
        )
        self.assertEqual(profile.primary_profile, "Mature Compounder")
        self.assertIn("margin durability", profile.report_thesis_spine)
        blocks = asset_aware.build_report_blocks({"ticker": "AAPL", "benchmark": "SPY", "valuation_snapshot": self._valuation().to_dict("records")}, profile, "en")
        joined = "\n".join(block["content"] for block in blocks)
        self.assertIn("mature quality case", joined)
        self.assertNotIn("cash runway, financing access, and dilution", joined)

    def test_rklb_like_payload_routes_to_speculative_growth_without_mature_contamination(self):
        profile = asset_aware.build_asset_profile(
            self._info("Industrials", "Aerospace & Defense", "space launch satellite aerospace"),
            self._fundamentals(revenue_cagr=0.45, latest_growth=0.55, operating_margin=-0.35, fcf_margin=-0.50, positive_ni=0, positive_fcf=0),
            self._valuation(pe=float("nan"), ps=55.0, ev_revenue=52.0),
            pd.DataFrame(),
            self._ruin(cash_runway=2.5),
        )
        self.assertEqual(profile.primary_profile, "Speculative Growth")
        self.assertEqual(profile.secondary_profile, "Aerospace / Space Systems")
        report_data = {"ticker": "RKLB", "benchmark": "SPY", "valuation_snapshot": self._valuation(pe=float("nan"), ps=55.0, ev_revenue=52.0).to_dict("records")}
        blocks = asset_aware.build_report_blocks(report_data, profile, "en")
        joined = "\n".join(block["content"] for block in blocks)
        self.assertIn("execution-and-financing growth case", joined)
        self.assertIn("burn", joined)
        self.assertIn("dilution", joined)
        self.assertNotIn("Services thesis", joined)
        self.assertNotIn("buyback-supported EPS", joined)
        self.assertNotIn("PE compression is not the right core frame", joined)

    def test_speculative_growth_valuation_uses_sales_burn_dilution_frame(self):
        profile = asset_aware.AssetProfile(
            lifecycle_profile="Speculative Growth",
            primary_profile="Speculative Growth",
            secondary_profile="",
            profitability_status="Unprofitable / not established",
            sector_type="industrials",
            sector_confidence="MEDIUM",
            valuation_method_fit="PS / EV Revenue / burn / dilution",
            cash_flow_profile="Negative or unproven FCF",
            capital_intensity="High / execution dependent",
            dilution_risk="Elevated",
            cyclicality_risk="Industry-specific",
            financial_company_flag=False,
            dominant_metric_set=["revenue growth", "cash runway"],
            invalid_metric_set=["PE compression"],
            data_deficit_flags=["backlog/order conversion"],
            research_stance_anchor="Speculative Watchlist",
            report_thesis_spine="High-growth but unprofitable/speculative growth.",
            thesis_spine_confidence="MEDIUM",
            framework_coverage_level="PARTIAL",
        )
        text = asset_aware.build_valuation_section(
            profile,
            {"valuation_snapshot": self._valuation(pe=float("nan"), ps=55.0, ev_revenue=52.0).to_dict("records")},
            "en",
        )
        self.assertIn("PS, EV/Revenue, revenue growth, burn, runway, dilution", text)
        self.assertIn("This is not a price target", text)
        self.assertNotIn("| PE compresses", text)

    def test_unknown_company_degrades_and_generates_framework_gap(self):
        profile = asset_aware.build_asset_profile(
            self._info("", "", "special purpose obscure business with limited disclosure"),
            self._fundamentals(revenue_cagr=float("nan"), latest_growth=float("nan"), operating_margin=float("nan"), fcf_margin=float("nan"), positive_ni=0, positive_fcf=0),
            self._valuation(pe=float("nan"), ps=float("nan"), ev_revenue=float("nan")),
            pd.DataFrame(),
            self._ruin(cash_runway=float("nan")),
        )
        self.assertEqual(profile.primary_profile, "Unknown / Data-Limited Screening")
        self.assertIn(profile.framework_coverage_level, {"SCREENING_ONLY", "UNKNOWN"})
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "framework_gap_analysis.md"
            asset_aware.write_framework_gap_analysis(path, profile)
            text = path.read_text()
        self.assertIn("Framework Gap Analysis", text)
        self.assertIn("What This Report Cannot Judge", text)

    def test_intc_like_payload_routes_to_semiconductor_turnaround_not_unknown(self):
        profile = asset_aware.build_asset_profile(
            self._info(
                "Technology",
                "Semiconductors",
                "semiconductor processor foundry manufacturing data center client computing integrated device",
            ),
            self._fundamentals(revenue_cagr=-0.02, latest_growth=0.03, operating_margin=0.04, fcf_margin=-0.12, positive_ni=2, positive_fcf=0),
            self._valuation(pe=28.0, ps=3.1, ev_revenue=3.3),
            self._trends(revenue=54000.0, capex=-25000.0),
            self._ruin(cash_runway=3.5),
        )
        self.assertEqual(profile.primary_profile, "Capital-Intensive Semiconductor Turnaround")
        self.assertEqual(profile.framework_coverage_level, "PARTIAL")
        self.assertIn("foundry execution", profile.dominant_metric_set)
        self.assertIn("capex pressure", profile.research_stance_anchor)
        self.assertNotEqual(profile.primary_profile, "Unknown / Data-Limited Screening")

    def test_semiconductor_turnaround_mentions_capex_margin_foundry_fcf_pressure(self):
        profile = asset_aware.build_asset_profile(
            self._info("Technology", "Semiconductors", "semiconductor foundry manufacturing data center processor"),
            self._fundamentals(revenue_cagr=0.01, latest_growth=-0.03, operating_margin=0.03, fcf_margin=-0.08, positive_ni=2, positive_fcf=0),
            self._valuation(pe=30.0, ps=3.0, ev_revenue=3.2),
            self._trends(revenue=100.0, capex=-28.0),
            self._ruin(cash_runway=2.5),
        )
        report_data = {"ticker": "INTC", "benchmark": "SPY", "valuation_snapshot": self._valuation(pe=30.0, ps=3.0, ev_revenue=3.2).to_dict("records")}
        joined = "\n".join(block["content"] for block in asset_aware.build_report_blocks(report_data, profile, "en"))
        for term in ["foundry", "capex", "gross-margin", "process", "data center", "free-cash-flow"]:
            self.assertIn(term, joined)
        self.assertNotIn("mature quality case", joined)

    def test_semiconductor_turnaround_generates_sector_data_deficits(self):
        profile = asset_aware.build_asset_profile(
            self._info("Technology", "Semiconductors", "semiconductor foundry manufacturing data center processor"),
            self._fundamentals(revenue_cagr=0.01, latest_growth=-0.03, operating_margin=0.03, fcf_margin=-0.08, positive_ni=2, positive_fcf=0),
            self._valuation(pe=30.0, ps=3.0, ev_revenue=3.2),
            self._trends(revenue=100.0, capex=-28.0),
            self._ruin(cash_runway=2.5),
        )
        deficits = " ".join(profile.data_deficit_flags)
        self.assertIn("foundry revenue / margin", deficits)
        self.assertIn("capex roadmap", deficits)
        self.assertIn("process node progress", deficits)
        self.assertIn("gross margin bridge", deficits)

    def test_lifecycle_logic_report_exists_when_lifecycle_fails(self):
        profile = asset_aware.AssetProfile(
            lifecycle_profile="Capital-Intensive Semiconductor Turnaround",
            primary_profile="Capital-Intensive Semiconductor Turnaround",
            secondary_profile="Hybrid / Technology Manufacturing",
            profitability_status="Unprofitable / not established",
            sector_type="technology",
            sector_confidence="MEDIUM",
            valuation_method_fit="turnaround / margin recovery / capex pressure / segment verification",
            cash_flow_profile="Negative or unproven FCF",
            capital_intensity="High / execution dependent",
            dilution_risk="Normal / not primary",
            cyclicality_risk="Not primary",
            financial_company_flag=False,
            dominant_metric_set=["foundry execution"],
            invalid_metric_set=["simple mature compounder thesis"],
            data_deficit_flags=["foundry revenue / margin"],
            research_stance_anchor="Semiconductor Turnaround",
            report_thesis_spine="Capital-intensive semiconductor turnaround.",
            thesis_spine_confidence="MEDIUM",
            framework_coverage_level="PARTIAL",
        )
        report = asset_aware.lifecycle_logic_check(profile, {"bad": "This is a simple mature compounder."})
        self.assertEqual(report["status"], "FAIL")
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "lifecycle_logic_report.md"
            asset_aware.write_lifecycle_logic_report(path, report, profile)
            text = path.read_text()
        self.assertIn("Lifecycle Logic Report", text)
        self.assertIn("profile/body conflict", text)
        self.assertIn("Suggested Fix", text)

    def test_framework_gap_analysis_is_specific_not_generic(self):
        profile = asset_aware.build_asset_profile(
            self._info("Technology", "Semiconductors", "semiconductor foundry manufacturing data center processor"),
            self._fundamentals(revenue_cagr=0.01, latest_growth=-0.03, operating_margin=0.03, fcf_margin=-0.08, positive_ni=2, positive_fcf=0),
            self._valuation(pe=30.0, ps=3.0, ev_revenue=3.2),
            self._trends(revenue=100.0, capex=-28.0),
            self._ruin(cash_runway=2.5),
        )
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "framework_gap_analysis.md"
            asset_aware.write_framework_gap_analysis(path, profile)
            text = path.read_text()
        self.assertIn("Semiconductor / integrated device manufacturing / foundry turnaround", text)
        self.assertIn("process-node execution", text)
        self.assertNotIn("Company-specific industry framework", text)

    def test_ai_uses_business_clues_not_only_sector_label(self):
        profile = asset_aware.build_asset_profile(
            self._info("Technology", "Hardware", "foundry manufacturing semiconductor processor data center client computing"),
            self._fundamentals(revenue_cagr=0.00, latest_growth=-0.02, operating_margin=0.02, fcf_margin=-0.10, positive_ni=1, positive_fcf=0),
            self._valuation(pe=25.0, ps=2.5, ev_revenue=2.8),
            self._trends(revenue=100.0, capex=-30.0),
            self._ruin(cash_runway=2.5),
        )
        self.assertEqual(profile.primary_profile, "Capital-Intensive Semiconductor Turnaround")
        self.assertIn("Semiconductor / manufacturing turnaround", profile.business_model_clues)

    def test_biotech_reit_and_financial_unknown_gaps_are_specific(self):
        cases = [
            (self._info("Healthcare", "Biotechnology", "clinical therapeutic drug candidate"), "Biotech Research Profile", "pipeline stage"),
            (self._info("Real Estate", "REIT", "real estate investment trust properties occupancy"), "REIT Research Profile", "FFO"),
            (self._info("Financial Services", "Insurance", "insurance underwriting reserves premiums"), "Insurance Research Profile", "underwriting"),
        ]
        for info, extension, expected in cases:
            profile = asset_aware.build_asset_profile(
                info,
                self._fundamentals(revenue_cagr=float("nan"), latest_growth=float("nan"), operating_margin=float("nan"), fcf_margin=float("nan"), positive_ni=0, positive_fcf=0),
                self._valuation(pe=float("nan"), ps=float("nan"), ev_revenue=float("nan")),
                pd.DataFrame(),
                self._ruin(cash_runway=float("nan")),
            )
            with tempfile.TemporaryDirectory() as tmp_dir:
                path = Path(tmp_dir) / "framework_gap_analysis.md"
                asset_aware.write_framework_gap_analysis(path, profile)
                text = path.read_text()
            self.assertIn(extension, text)
            self.assertIn(expected, text)

    def test_financial_and_cyclical_profiles_reject_wrong_core_metrics(self):
        financial = asset_aware.build_asset_profile(
            self._info("Financial Services", "Banks"),
            self._fundamentals(revenue_cagr=0.02, latest_growth=0.01, operating_margin=0.20, fcf_margin=0.10, positive_ni=4, positive_fcf=4),
            self._valuation(pe=12.0, ps=3.0, ev_revenue=3.0),
            pd.DataFrame(),
            self._ruin(cash_runway=float("nan")),
        )
        cyclical = asset_aware.build_asset_profile(
            self._info("Energy", "Oil & Gas", "oil gas commodity"),
            self._fundamentals(revenue_cagr=0.05, latest_growth=-0.02, operating_margin=0.18, fcf_margin=0.12, positive_ni=4, positive_fcf=4),
            self._valuation(pe=6.0, ps=1.2, ev_revenue=1.4),
            pd.DataFrame(),
            self._ruin(cash_runway=float("nan")),
        )
        self.assertEqual(financial.primary_profile, "Financials")
        self.assertIn("ordinary FCF margin", financial.invalid_metric_set)
        self.assertEqual(cyclical.primary_profile, "Cyclical")
        self.assertIn("low PE means cheap", cyclical.invalid_metric_set)

    def test_patch_artifacts_and_diff_log_are_generated(self):
        profile = asset_aware.build_asset_profile(
            self._info("Industrials", "Aerospace & Defense", "space launch satellite aerospace"),
            self._fundamentals(revenue_cagr=0.45, latest_growth=0.55, operating_margin=-0.35, fcf_margin=-0.50, positive_ni=0, positive_fcf=0),
            self._valuation(pe=float("nan"), ps=55.0, ev_revenue=52.0),
            pd.DataFrame(),
            self._ruin(cash_runway=2.5),
        )
        blocks = asset_aware.build_report_blocks({"ticker": "RKLB", "benchmark": "SPY", "valuation_snapshot": self._valuation(ps=55.0, ev_revenue=52.0).to_dict("records")}, profile, "en")
        patched, patch_log = asset_aware.apply_interpretation_patch(blocks, profile)
        self.assertEqual(patch_log["patch_status"], "APPLIED")
        self.assertIn(patch_log["patch_materiality_status"], {"MATERIAL_PATCH_APPLIED", "FALLBACK_PATCH_APPLIED"})
        locked = [block for block in patched if block["block_type"].startswith("LOCKED")]
        self.assertTrue(all(not block["editable"] for block in locked))
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir)
            asset_aware.write_patch_artifacts(out, patch_log, patched)
            self.assertTrue((out / "correction_patch.json").exists())
            self.assertTrue((out / "patched_report_blocks.json").exists())
            diff = (out / "patch_diff_log.md").read_text()
            self.assertIn("Old Text", diff)
            self.assertIn("New Text", diff)
            self.assertIn("Materiality:", diff)

    def test_overall_status_and_fallback_degradation(self):
        self.assertEqual(asset_aware.overall_status_v43({"A": "PASS"}), "PASS")
        self.assertEqual(asset_aware.overall_status_v43({"A": "WARNING"}), "WARNING")
        self.assertEqual(asset_aware.overall_status_v43({"A": "FAIL"}), "UNVERIFIED")
        self.assertEqual(asset_aware.overall_status_v43({"A": "PASS"}, fallback_count=3), "WARNING_DEGRADED")

    def test_report_pack_helpers_create_expected_structure(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            run_dir = Path(tmp_dir)
            dirs = asset_aware.organize_report_pack(run_dir, "TEST")
            status = {"OVERALL_REPORT_STATUS": "PASS", "ASSET_PROFILE": "Unknown", "FRAMEWORK_COVERAGE_LEVEL": "SCREENING_ONLY", "COMPANY_SPECIFICITY_STATUS": "WARNING", "PATCH_STATUS": "APPLIED", "FALLBACK_USED_COUNT": 1}
            asset_aware.write_run_readme(run_dir, "TEST", "SPY", status)
            gate = asset_aware.presentation_gate(run_dir)
            self.assertEqual(gate["PRESENTATION_STATUS"], "PASS")
            self.assertTrue((run_dir / "README.md").exists())
            for key in ["report", "charts", "data", "audit", "ai", "metadata", "self_review"]:
                self.assertTrue(dirs[key].exists())

    def test_openbb_research_short_command_exists(self):
        pyproject = (ROOT / "pyproject.toml").read_text()
        self.assertIn('openbb-research = "openbb_company_research_tool.__main__:main"', pyproject)

    def test_pack_command_creates_zip_with_expected_folders(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            run_dir = Path(tmp_dir) / "reports" / "TEST" / "runs" / "run1"
            asset_aware.organize_report_pack(run_dir, "TEST")
            (run_dir / "README.md").write_text("readme")
            for folder in ["report", "charts", "data", "audit", "ai", "dashboard", "metadata", "self_review"]:
                (run_dir / folder / ".keep").write_text("x")
            zip_path = company_tool.pack_report_folder(run_dir)
            self.assertTrue(zip_path.exists())
            self.assertEqual(zip_path.name, "TEST_research_pack.zip")

    def test_help_includes_short_commands_and_pack_usage(self):
        original_argv = sys.argv[:]
        try:
            sys.argv = ["openbb-research", "--help"]
            with self.assertRaises(SystemExit):
                company_tool.parse_args()
        finally:
            sys.argv = original_argv
        examples = company_tool.parse_args.__code__.co_consts
        joined = "\n".join(str(item) for item in examples)
        self.assertIn("openbb-research TICKER --pack", joined)
        self.assertIn("openbb-research pack RUN_FOLDER", joined)

    def test_rust_cpp_tools_are_optional_but_documented_when_applicable(self):
        text = (ROOT / "docs" / "performance_notes.md").read_text()
        self.assertIn("Rust", text)
        self.assertIn("Python fallback", text)

    def test_general_profile_matrix_routes_to_specific_frameworks(self):
        cases = [
            (
                "insurance",
                self._info("Financial Services", "Insurance", "insurance underwriting reserves premiums catastrophe claims"),
                "Insurance-like Screening",
                ["combined ratio", "reserve adequacy", "float"],
            ),
            (
                "reit",
                self._info("Real Estate", "REIT", "real estate investment trust properties occupancy rent spread"),
                "REIT-like Screening",
                ["FFO", "AFFO", "occupancy"],
            ),
            (
                "consumer",
                self._info("Consumer Cyclical", "Retail", "retail same-store traffic ticket inventory store count"),
                "Consumer / Retail",
                ["same-store sales", "traffic", "inventory"],
            ),
            (
                "utility",
                self._info("Utilities", "Electric Utility", "regulated electric utility rate base allowed ROE"),
                "Utilities / Infrastructure",
                ["regulated asset base", "allowed ROE", "rate case"],
            ),
            (
                "transport",
                self._info("Industrials", "Airlines", "airline transport fleet load factor fuel cost"),
                "Shipping / Airlines / Transport",
                ["load factor / utilization", "fuel cost", "fleet age"],
            ),
        ]
        for _, info, expected_profile, expected_terms in cases:
            profile = asset_aware.build_asset_profile(
                info,
                self._fundamentals(revenue_cagr=0.02, latest_growth=0.01, operating_margin=0.08, fcf_margin=0.03, positive_ni=2, positive_fcf=1),
                self._valuation(pe=18.0, ps=2.0, ev_revenue=2.0),
                pd.DataFrame(),
                self._ruin(cash_runway=3.0),
            )
            self.assertEqual(profile.primary_profile, expected_profile)
            joined = " ".join(profile.dominant_metric_set + profile.data_deficit_flags)
            for term in expected_terms:
                self.assertIn(term, joined)
            report_data = {"ticker": "TEST", "benchmark": "SPY", "valuation_snapshot": self._valuation(pe=18.0, ps=2.0, ev_revenue=2.0).to_dict("records")}
            text = "\n".join(block["content"] for block in asset_aware.build_report_blocks(report_data, profile, "en"))
            self.assertTrue(any(term in text for term in expected_terms))

    def test_biotech_profile_uses_pipeline_cash_runway_not_generic_unknown(self):
        profile = asset_aware.build_asset_profile(
            self._info("Healthcare", "Biotechnology", "clinical therapeutic drug candidate AI drug discovery platform"),
            self._fundamentals(revenue_cagr=0.40, latest_growth=0.20, operating_margin=-1.20, fcf_margin=-1.40, positive_ni=0, positive_fcf=0),
            self._valuation(pe=-4.0, ps=20.0, ev_revenue=18.0),
            pd.DataFrame(),
            self._ruin(cash_runway=1.8),
        )
        self.assertEqual(profile.secondary_profile, "Biotech-like Screening")
        report_data = {"ticker": "RXRX", "benchmark": "SPY", "valuation_snapshot": self._valuation(pe=-4.0, ps=20.0, ev_revenue=18.0).to_dict("records")}
        text = "\n".join(block["content"] for block in asset_aware.build_report_blocks(report_data, profile, "en"))
        self.assertIn("pipeline", text)
        self.assertIn("R&D burn", text)
        self.assertIn("cash runway", text)
        self.assertNotIn("ordinary SaaS growth thesis", text)

    def test_negative_boundary_sentence_not_flagged_as_forbidden(self):
        profile = asset_aware.AssetProfile(
            lifecycle_profile="Unknown / Data-Limited Screening",
            primary_profile="Unknown / Data-Limited Screening",
            secondary_profile="",
            profitability_status="Unprofitable / not established",
            sector_type="unknown",
            sector_confidence="LOW",
            valuation_method_fit="screening-only",
            cash_flow_profile="Negative or unproven FCF",
            capital_intensity="Unknown",
            dilution_risk="Normal / not primary",
            cyclicality_risk="Not primary",
            financial_company_flag=False,
            dominant_metric_set=["data availability"],
            invalid_metric_set=["complete thesis"],
            data_deficit_flags=["business model verification"],
            research_stance_anchor="Screening-only",
            report_thesis_spine="Screening-only.",
            thesis_spine_confidence="LOW",
            framework_coverage_level="SCREENING_ONLY",
        )
        report = asset_aware.lifecycle_logic_check(profile, {"boundary": "This cannot be treated as a complete thesis or complete research conclusion."})
        self.assertEqual(report["status"], "PASS")

    def test_complete_research_conclusion_claim_is_flagged(self):
        profile = asset_aware.AssetProfile(
            lifecycle_profile="Unknown / Data-Limited Screening",
            primary_profile="Unknown / Data-Limited Screening",
            secondary_profile="",
            profitability_status="Unprofitable / not established",
            sector_type="unknown",
            sector_confidence="LOW",
            valuation_method_fit="screening-only",
            cash_flow_profile="Negative or unproven FCF",
            capital_intensity="Unknown",
            dilution_risk="Normal / not primary",
            cyclicality_risk="Not primary",
            financial_company_flag=False,
            dominant_metric_set=["data availability"],
            invalid_metric_set=["complete thesis"],
            data_deficit_flags=["business model verification"],
            research_stance_anchor="Screening-only",
            report_thesis_spine="Screening-only.",
            thesis_spine_confidence="LOW",
            framework_coverage_level="SCREENING_ONLY",
        )
        report = asset_aware.lifecycle_logic_check(profile, {"bad": "This can be treated as a complete research conclusion."})
        self.assertEqual(report["status"], "FAIL")

    def test_patch_applied_requires_material_profile_specific_change(self):
        profile = asset_aware.build_asset_profile(
            self._info("Healthcare", "Biotechnology", "clinical therapeutic drug candidate AI drug discovery platform"),
            self._fundamentals(revenue_cagr=0.40, latest_growth=0.20, operating_margin=-1.20, fcf_margin=-1.40, positive_ni=0, positive_fcf=0),
            self._valuation(pe=-4.0, ps=20.0, ev_revenue=18.0),
            pd.DataFrame(),
            self._ruin(cash_runway=1.8),
        )
        blocks = asset_aware.build_report_blocks({"ticker": "RXRX", "benchmark": "SPY", "valuation_snapshot": self._valuation(pe=-4.0, ps=20.0, ev_revenue=18.0).to_dict("records")}, profile, "en")
        patched, patch_log = asset_aware.apply_interpretation_patch(blocks, profile)
        diff_text = json.dumps(patch_log, ensure_ascii=False)
        self.assertEqual(patch_log["patch_status"], "APPLIED")
        self.assertTrue(any(record["old_text"].strip() != record["new_text"].strip() for record in patch_log["patch_records"]))
        self.assertIn("pipeline", diff_text)
        self.assertIn("R&D burn", diff_text)
        final_text = "\n".join(block["content"] for block in patched)
        self.assertIn("Patch note", final_text)
        self.assertIn("pipeline", final_text)

    def test_patch_applied_requires_old_new_difference(self):
        profile = asset_aware.build_asset_profile(
            self._info("Healthcare", "Biotechnology", "clinical therapeutic drug candidate AI drug discovery platform"),
            self._fundamentals(revenue_cagr=0.40, latest_growth=0.20, operating_margin=-1.20, fcf_margin=-1.40, positive_ni=0, positive_fcf=0),
            self._valuation(pe=-4.0, ps=20.0, ev_revenue=18.0),
            pd.DataFrame(),
            self._ruin(cash_runway=1.8),
        )
        blocks = asset_aware.build_report_blocks({"ticker": "RXRX", "benchmark": "SPY", "valuation_snapshot": self._valuation(pe=-4.0, ps=20.0, ev_revenue=18.0).to_dict("records")}, profile, "en")
        _, patch_log = asset_aware.apply_interpretation_patch(blocks, profile)
        material_records = [record for record in patch_log["patch_records"] if record["materiality"] != "CONFIRMED_NO_CHANGE"]
        self.assertGreater(len(material_records), 0)
        self.assertTrue(all(record["old_text"].strip() != record["new_text"].strip() for record in material_records))

    def test_no_change_patch_does_not_count_as_applied(self):
        profile = asset_aware.build_asset_profile(
            self._info("Technology", "Consumer Electronics"),
            self._fundamentals(revenue_cagr=0.018, latest_growth=0.064, operating_margin=0.32, fcf_margin=0.24, positive_ni=4, positive_fcf=4),
            self._valuation(pe=37.4, ps=10.0, ev_revenue=10.1),
            pd.DataFrame(),
            self._ruin(cash_runway=None),
        )
        blocks = asset_aware.build_report_blocks({"ticker": "AAPL", "benchmark": "SPY", "valuation_snapshot": self._valuation().to_dict("records")}, profile, "en")
        _, patch_log = asset_aware.apply_interpretation_patch(blocks, profile)
        self.assertEqual(patch_log["patch_status"], "NOT_NEEDED")
        self.assertEqual(patch_log["patch_materiality_status"], "NO_MATERIAL_CHANGE")

    def test_patch_materiality_status_is_reported(self):
        profile = asset_aware.build_asset_profile(
            self._info("Industrials", "Aerospace & Defense", "space launch satellite aerospace"),
            self._fundamentals(revenue_cagr=0.45, latest_growth=0.55, operating_margin=-0.35, fcf_margin=-0.50, positive_ni=0, positive_fcf=0),
            self._valuation(pe=float("nan"), ps=55.0, ev_revenue=52.0),
            pd.DataFrame(),
            self._ruin(cash_runway=2.5),
        )
        blocks = asset_aware.build_report_blocks({"ticker": "RKLB", "benchmark": "SPY", "valuation_snapshot": self._valuation(ps=55.0, ev_revenue=52.0).to_dict("records")}, profile, "en")
        _, patch_log = asset_aware.apply_interpretation_patch(blocks, profile)
        self.assertIn("patch_materiality_status", patch_log)
        self.assertIn(patch_log["patch_materiality_status"], {"MATERIAL_PATCH_APPLIED", "FALLBACK_PATCH_APPLIED", "NO_MATERIAL_CHANGE"})

    def test_verification_status_rollups_split_data_and_thesis(self):
        statuses = {
            "DATA_AUDIT_STATUS": "PASS",
            "RISK_METHOD_STATUS": "PASS",
            "PRICE_LABEL_CHECK_STATUS": "PASS",
            "LIFECYCLE_LOGIC_STATUS": "PASS",
            "COMPANY_SPECIFICITY_STATUS": "WARNING",
            "AI_ANALYST_REVIEW_STATUS": "PASS",
            "LANGUAGE_LINT_STATUS": "PASS",
            "FRAMEWORK_COVERAGE_LEVEL": "PARTIAL",
        }
        data_status = asset_aware.rollup_data_verification_status(statuses)
        thesis_status = asset_aware.rollup_thesis_verification_status(statuses)
        self.assertEqual(data_status, "PASS")
        self.assertEqual(thesis_status, "WARNING")
        self.assertEqual(asset_aware.overall_status_from_verification(data_status, thesis_status), "WARNING")

    def _info(self, sector, industry, summary="consumer products software services"):
        return {
            "sector": sector,
            "industry": industry,
            "longBusinessSummary": summary,
            "quoteType": "EQUITY",
        }

    def _fundamentals(self, revenue_cagr=0.02, latest_growth=0.03, operating_margin=0.25, fcf_margin=0.15, positive_ni=4, positive_fcf=4):
        return pd.DataFrame(
            [
                {"Metric": "Revenue CAGR", "Value": revenue_cagr},
                {"Metric": "Revenue Growth Latest", "Value": latest_growth},
                {"Metric": "Gross Margin Latest", "Value": 0.45},
                {"Metric": "Operating Margin Latest", "Value": operating_margin},
                {"Metric": "FCF Margin Latest", "Value": fcf_margin},
                {"Metric": "Positive Net Income Years", "Value": positive_ni},
                {"Metric": "Positive FCF Years", "Value": positive_fcf},
            ]
        )

    def _valuation(self, pe=30.0, ps=8.0, ev_revenue=8.0):
        return pd.DataFrame(
            [
                {"Metric": "trailingPE", "Value": pe},
                {"Metric": "priceToSalesTrailing12Months", "Value": ps},
                {"Metric": "enterpriseToRevenue", "Value": ev_revenue},
            ]
        )

    def _ruin(self, cash_runway=None):
        return pd.DataFrame(
            [
                {"Metric": "Cash Runway Years", "Value": cash_runway},
                {"Metric": "Balance Sheet Resilience Score", "Value": 55.0},
            ]
        )

    def _trends(self, revenue=100.0, capex=-20.0):
        return pd.DataFrame(
            [
                {"Metric": "Revenue", "Latest": revenue},
                {"Metric": "Capital Expenditure", "Latest": capex},
            ]
        )


if __name__ == "__main__":
    unittest.main()
