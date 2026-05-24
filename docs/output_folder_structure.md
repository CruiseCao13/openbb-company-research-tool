# Output Folder Structure

v4.3 run folders are organized as report packs.

```text
reports/
  TICKER/
    latest/
    runs/
      RUN_ID/
        README.md
        report/
        charts/
        data/
        audit/
        ai/
        dashboard/
        metadata/
        self_review/
```

## Folder Meaning

`report/` contains final Markdown reports.

`charts/` contains static PNG figures.

`data/` contains CSV exports.

`audit/` contains data, method, language, lifecycle, company-specificity, and presentation checks.

`ai/` contains correction logs and interpretation patch artifacts.

`dashboard/` contains interactive HTML dashboards.

`metadata/` contains machine-readable run state such as `asset_profile.json` and `report_status.json`.

`self_review/` contains automatic system self-review, framework gap analysis, improvement suggestions, and regression test suggestions.

