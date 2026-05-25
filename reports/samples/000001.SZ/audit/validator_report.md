# Validator Report

Overall status: WARNING

Human review required: true

## Compiler-style Validation Passes

| Pass | Status | Blocking | Suggested Fix |
|---|---|---:|---|
| ProviderPayloadPass | PASS | false | Inspect provider_status.json and retry or switch provider when warning. |
| AiJsonSchemaPass | PASS | true | Regenerate AI artifacts with the schema-constrained prompt compiler. |
| MoneyFlowPass | PASS | true | Regenerate money-flow interpretation from locked cash-flow and balance-sheet data. |
| EvidenceMapPass | PASS | true | Map claims to locked data, chart/table evidence, assumptions, or data gaps. |
| SectionSourceMapPass | PASS | true | Map every report section to AI artifacts or locked data; renderer must not invent thesis, valuation, risks, or next checks. |
| ChartTablePass | PASS | true | Regenerate chart/table plan and explanation blocks. |
| VisualLintPass | PASS | true | Fix report structure, chart links, dashboard, data coverage, or forbidden language. |
| PdfExportPass | PASS | false | Install or repair the PDF export helper; do not pretend PDF succeeded. |
