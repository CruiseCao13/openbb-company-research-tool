# Compact Payload Training Check

- Path: `training/cases/compact_payload_cases.jsonl`
- Exists: True
- Required markers present: True

```text
{"schema_version":"v5.0.0","ticker":"LUNR","issue_type":"compact_payload_identity_guard","why_bad":"A real external AI run previously selected a telecom frame for Intuitive Machines. The compact payload must provide enough company identity detail to prevent sector inference from generic financial metrics.","required_payload_fields":["company name","sector","industry","business description","provider summary","market","provider","data coverage","missing critical fields","space/lunar/NASA/aerospace keywords when provider has them"],"must_contain":["Intuitive Machines","space infrastructure","lunar","NASA","aerospace"],"must_not_contain":["telecom carrier inference without provider evidence","wireless service revenue","broadband revenue","subscriber churn"],"fix_target":"compact_payload","regression_status":"new"}

```

Final status: PASS
