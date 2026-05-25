# v5 Money Flow Specificity Pass 01

Date: 2026-05-25

Scope: fixed local fallback Money Flow specificity and reran only the five warning samples from Core Data Accuracy Rerun Pass 01. No external OpenAI API, no provider rewrite, no UI/Tauri/D3/PDF work.

Final status: PASS

## Warning Samples Inspected

| Ticker | Before run | Before Money Flow | After run | After Money Flow | Report status |
|---|---|---|---|---|---|
| AAPL | `core_data_accuracy_rerun_01_aapl` | WARNING | `reports/AAPL/runs/money_flow_specificity_01_aapl` | PASS | PASS |
| GOOGL | `core_data_accuracy_rerun_01_googl` | WARNING | `reports/GOOGL/runs/money_flow_specificity_01_googl` | PASS | PASS |
| CAT | `core_data_accuracy_rerun_01_cat` | WARNING | `reports/CAT/runs/money_flow_specificity_01_cat` | PASS | PASS |
| ISRG | `core_data_accuracy_rerun_01_isrg` | WARNING | `reports/ISRG/runs/money_flow_specificity_01_isrg` | PASS | PASS |
| JPM | `core_data_accuracy_rerun_01_jpm` | WARNING | `reports/JPM/runs/money_flow_specificity_01_jpm` | PASS | PASS |

## Root Cause Per Sample

- AAPL: Generic local fallback phrasing did not tie hardware/services cash generation to OCF, capex, FCF, R&D, or shareholder-return rows.
- GOOGL: Generic local fallback phrasing did not tie ads/cloud/platform revenue to OCF, AI/cloud capex, R&D, and FCF.
- CAT: Generic local fallback phrasing did not tie industrial machinery cycle to working capital, inventory/receivables, capex, debt, and through-cycle cash conversion.
- ISRG: Generic local fallback phrasing did not tie medtech/procedure-linked revenue to R&D, SG&A, inventory/receivables, capex, and OCF.
- JPM: Generic local fallback phrasing used ordinary revenue/cash-flow language instead of bank economics and did not explicitly reject industrial FCF as core.

## Fixes Made

- Replaced generic local fallback Money Flow prose with frame-specific source/use narratives in `financial_interpretation.rs`.
- Local fallback now mentions locked OCF/capex/FCF values where available and names missing fields as data gaps.
- Bank frame now says industrial FCF yield is not the core test and points to net interest/fees/deposits/lending.
- Added tests for generic wording, OCF/capex/FCF coverage, bank-specific framing, space cash burn/financing gaps, A-share inventory/receivables, missing-field warnings, and Money Flow specificity report issue text.

## Before / After Details

### AAPL

- Before: WARNING
- After: PASS
- Rerun path: `reports/AAPL/runs/money_flow_specificity_01_aapl`
- Money source: Money comes from the hardware-plus-services ecosystem and operating cash conversion. Locked data shows revenue is 220960000000.0, operating cash flow is 111482000000.0, and free cash flow is 98767000000.0; buybacks or dividends should only be discussed where cash-flow and shareholder-return rows support them.
- Money uses: Money is absorbed by product costs, R&D, supply-chain working capital, capex, dividends/buybacks where supported, and financing obligations. Locked data shows capex is -12715000000.0 and R&D is 34550000000.0.

### GOOGL

- Before: WARNING
- After: PASS
- Rerun path: `reports/GOOGL/runs/money_flow_specificity_01_googl`
- Money source: Money comes from platform revenue streams such as ads, cloud, subscriptions, and operating cash conversion when supported by filings. Locked data shows revenue is 162535000000.0, operating cash flow is 164713000000.0, and free cash flow is 73266000000.0.
- Money uses: Money is absorbed by traffic acquisition/operating costs, R&D, AI/cloud infrastructure capex, working capital, taxes, and shareholder returns where supported. Locked data shows capex is -91447000000.0 and R&D is 61087000000.0.

### CAT

- Before: WARNING
- After: PASS
- Rerun path: `reports/CAT/runs/money_flow_specificity_01_cat`
- Money source: Money comes from cyclical equipment, parts, services, and dealer/end-market demand. Locked data shows revenue is 46111000000.0, operating cash flow is 11739000000.0, and free cash flow is 7453000000.0; through-cycle quality depends on working capital, inventory/receivables, and equipment-cycle cash conversion.
- Money uses: Money is absorbed by manufacturing costs, dealer/channel working capital, inventory and receivables, capex, debt service, and cycle-sensitive reinvestment. Locked data shows capex is -4286000000.0 and debt-like obligations is 33344000000.0.

### ISRG

- Before: WARNING
- After: PASS
- Rerun path: `reports/ISRG/runs/money_flow_specificity_01_isrg`
- Money source: Money comes from surgical systems, instruments/accessories, services, procedure-linked volume, and operating cash conversion. Locked data shows revenue is 3422400000.0, operating cash flow is 3030500000.0, and free cash flow is 2490700000.0; procedure mix, R&D, SG&A, inventory and receivables remain the next checks.
- Money uses: Money is absorbed by R&D, instruments/system manufacturing, SG&A, inventory/receivables, capex, and service infrastructure. Locked data shows R&D is 1311800000.0 and capex is -539800000.0.

### JPM

- Before: WARNING
- After: PASS
- Rerun path: `reports/JPM/runs/money_flow_specificity_01_jpm`
- Money source: Money comes from bank economics: net interest income, fees, cards, deposits and lending activity. Locked data shows revenue is 181847000000.0; industrial free-cash-flow yield is not the core money-source test for this company.
- Money uses: Money is absorbed by funding costs, credit losses/reserves, operating expense, capital requirements, dividends/buybacks where supported, and balance-sheet growth. Capex is not the core bank metric; locked data says capex is missing from locked data.

## Remaining Warnings

- None for the five affected samples.

## Hard Failures

- None.

## Files Changed

- `research-rs/crates/research-ai/src/financial_interpretation.rs`
- `research-rs/crates/research-ai/src/tests.rs`
- `research-rs/crates/research-report/src/tests.rs`
- `reports/release_checks/v5_0/money_flow_specificity_pass_01.md`

## Tests Added

- `money_flow_local_fallback_not_generic`
- `money_flow_mentions_ocf_capex_fcf_when_available`
- `money_flow_bank_avoids_industrial_fcf_core`
- `money_flow_space_company_mentions_cash_burn_or_financing_gap`
- `money_flow_a_share_consumer_mentions_inventory_or_receivables_when_available`
- `money_flow_warning_when_required_fields_missing`
- `money_flow_specificity_report_lists_issue`

## Validation

- `cargo test --manifest-path research-rs/Cargo.toml` passed before affected-sample rerun.
- Full required validation is run after this report before commit.

## Generated Run Folder Policy

The five rerun folders are validation artifacts and are intentionally not staged for commit. This report records their paths and before/after status.
