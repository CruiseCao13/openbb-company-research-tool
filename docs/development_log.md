# Development Log & Engineering Rules

## v4.3 — Asset-Aware Research Report System

### Problem

RKLB-style stress testing showed that a report could detect a speculative-growth profile but still reuse mature-compounder interpretation language.

That is a product-level failure: the data label was correct, but the research logic in the body did not follow the label.

### Design Response

v4.3 introduced:

- asset profile routing
- multi-metric thesis spine
- profile-specific valuation and Q&A blocks
- AI interpretation patch artifacts
- lifecycle logic and company-specificity checks
- fallback usage tracking
- organized report pack folders
- automatic system self-review

### Lesson Learned

The report shell can be shared, but the research conclusion cannot be templated blindly.

Unknown companies may be downgraded. They must not be forced into a confident research frame.

### INTC Pressure-Test Correction

An INTC-like stress test exposed a second failure mode: the system could downgrade too broadly and call a company `Unknown / Data-Limited Screening` even when business clues pointed to a more specific framework.

The fix was not ticker-specific. v4.3 now includes a generalized `Capital-Intensive Semiconductor Turnaround` route based on sector, industry, business-summary clues, capex intensity, margin pressure, weak or negative FCF, and unstable profitability.

The report must treat these companies as manufacturing-turnaround cases, not generic technology screens. The required verification frame includes foundry revenue / margin, data-center segment trend, client computing trend, capex roadmap, process-node progress, manufacturing utilization, inventory pressure, gross-margin bridge, and free-cash-flow pressure.

Rule established:

- lifecycle failures must produce `audit/lifecycle_logic_report.md`
- framework gap analysis must be specific, not generic
- `screening-only` must not claim that no sector-specific data is missing
- pack commands and help text must be treated as part of the product surface, not optional polish

This document records real engineering problems, troubleshooting steps, fixes, verification commands, and design rules discovered during the development of `openbb-company-research-tool`.

It is not a release note. For version-level changes, see `CHANGELOG.md`.

---

## Core Principle

> Data calculates. AI reviews. Human decides.

The project must remain a data-first research workflow. Python owns deterministic calculations. AI may review and explain the generated data, but it must not invent numbers, modify financial metrics, provide buy/sell instructions, or replace human judgment.

---

## 1. v2.0 — From Data Pack to Research Workflow

### Problem

The early tool produced useful outputs, but it was still closer to a data pack than a research workflow.

It generated numbers, charts, and reports, but the structure did not fully support repeatable review, historical comparison, or clear user interpretation.

### Why It Mattered

A user could run a ticker once and get data, but there was no strong research workflow:

- old outputs could be overwritten
- static charts were hard to inspect
- risk was too price-history focused
- scoring was too one-size-fits-all
- data warnings were too passive
- reports lacked a strong presentation surface

### Design Response

v2.0 introduced:

- archive-by-default output structure
- `latest/` refreshed as newest copy
- Plotly interactive HTML dashboard
- score radar HTML
- sanity checks with severity / finding / action
- ruin risk metrics
- sector/lifecycle-aware scoring
- personal margin stress table

### Lesson Learned

A useful financial tool must not only calculate metrics. It must organize outputs into a repeatable research workflow.

---

## 2. v2.1 — Beginner Clarity Layer

### Problem

The v2.0 report became more powerful, but it still risked feeling like a professional metric dump.

For beginners, terms such as Sharpe Ratio, FCF, EBITDA, drawdown, valuation multiple, and Net Debt / EBITDA are not self-explanatory.

### Why It Mattered

The target user is not a professional analyst. A beginner may misread:

- historical outperformance as a buy signal
- low drawdown as low business risk
- high score as investment approval
- good cash flow as valuation safety
- high revenue growth as business quality

### Design Response

v2.1 added:

- How to Read This Report
- Beginner Summary
- Plain-English Meaning under major sections
- chart-reading notes
- Research Score beginner warning
- `docs/metric_guide.md`

### Lesson Learned

A beginner-friendly report should not remove professional metrics. It should add an interpretation layer that explains what the metrics mean, what they do not mean, and how they can be misread.

---

## 3. v3.0 — Optional AI Review Layer

### Problem

The report had deterministic metrics and beginner explanations, but it still lacked a second-pass reasoning layer.

The tool needed a way to check whether the evidence supported the conclusion, identify weak assumptions, and explain possible beginner misreadings.

### Why It Mattered

A deterministic report can calculate numbers, but it cannot always synthesize:

- what the central tension is
- where the report may be overconfident
- which assumptions are weakest
- what a beginner is most likely to misunderstand

### Design Response

v3.0 added:

- `scripts/ai_review.py`
- OpenAI Chat Completions structured outputs
- Pydantic schema
- compact AI payload derived from deterministic report data
- AI Review Markdown renderer
- graceful fallback when AI fails
- Rich terminal UI via `scripts/terminal_ui.py`

### Core Boundary

AI Review must not:

- give buy / sell / hold recommendations
- provide price targets
- predict short-term price movement
- invent numbers
- modify deterministic calculations

### Lesson Learned

AI is useful as a reviewer and explainer, not as the owner of financial truth.

---

## 4. Python Environment Issue

### Problem

Running a direct Python command produced:

```text
ModuleNotFoundError: No module named 'openai'
```

### Root Cause

The terminal used the global / pyenv Python environment rather than the project-local virtual environment.

The `openai` package was installed inside `.venv`, but bare `python` did not point to `.venv/bin/python`.

### Fix

Use project-local Python explicitly:

```bash
.venv/bin/python
```

### Verification

```bash
.venv/bin/python - <<'PY'
import openai
print("openai installed:", openai.__version__)
PY
```

Expected result:

```text
openai installed: 2.38.0
```

### Rule

Inside this project, use `.venv/bin/python` for validation, testing, and script execution. Do not rely on the shell's bare `python` alias.

---

## 5. OpenAI API Quota Issue

### Problem

A real API call failed with:

```text
RateLimitError: Error code: 429
insufficient_quota
```

### Root Cause

The API key was readable and valid, but the OpenAI API account had no available credits.

This was not a code issue and not an invalid API key issue.

### Fix

Add API billing credits to the OpenAI Platform account.

### Verification

```bash
.venv/bin/python - <<'PY'
import os
from openai import OpenAI

client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))

resp = client.chat.completions.create(
    model=os.getenv("OPENAI_MODEL", "gpt-4o-mini"),
    messages=[
        {"role": "user", "content": "Reply with exactly: API_OK"}
    ],
    max_tokens=10,
)

print(resp.choices[0].message.content)
PY
```

Expected result:

```text
API_OK
```

### Lesson Learned

API key validity and API billing quota are separate problems. A key can be valid but still unable to call the API because the account has no credits.

---

## 6. AI Review Fallback

### Problem

The AI layer depends on an external API. Network errors, missing keys, quota errors, model errors, and malformed outputs are all possible.

If the AI layer fails, the deterministic report must still generate.

### Design Rule

AI Review is optional. Core report generation must never depend on AI success.

### Fix

Implemented graceful fallback:

- missing API key does not crash
- API failure does not crash
- quota failure does not crash
- failed AI Review renders a skipped / failed section
- deterministic report still generates

### Verification

```bash
.venv/bin/python scripts/company_research_tool.py AAPL --benchmark SPY --start 2023-01-01 --ai-review --no-rich
```

Expected result:

- normal report generated
- AI Review skipped or failed clearly
- no crash

### Lesson Learned

Optional AI features should degrade gracefully. External API failure must be treated as expected runtime behavior, not an exceptional collapse.

---

## 7. Git Tag and Release Hygiene

### Problem

Remote tag push failed with:

```text
! [rejected] v2.1.0 -> v2.1.0 (already exists)
```

### Root Cause

A tag already existed on the remote and pointed to an earlier commit.

### Fix

Delete the stale remote tag and push the corrected tag:

```bash
git push origin :refs/tags/v2.1.0
git push origin v2.1.0
```

### Correct Release Order

```bash
git add -A
git commit -m "release vX.X ..."
git tag vX.X.X
git push
git push origin vX.X.X
```

### Verification

```bash
git status
git log --oneline --decorate -5
```

Expected result:

```text
nothing to commit, working tree clean
HEAD -> main, tag: vX.X.X, origin/main
```

### Lesson Learned

A release is not complete until commit, tag, remote branch, and remote tag all point to the intended state.

---

## 8. Report Quality Issues Found After v3.0

### Problem

The v3.0 report is usable and presentable, but several quality issues remain:

- duplicate horizontal rule before AI Review
- provider summary can truncate mid-word
- raw provider field names remain in some tables
- AI Review can sound generic
- AI Review verification list can duplicate Manual Verification
- some wording approaches portfolio management language
- charts can be more visually unified

### Why It Matters

Small presentation issues reduce trust. In financial tools, trust is not only about calculations. It is also affected by formatting, wording, labels, and visual consistency.

### Lesson Learned

A report can be technically correct but still feel unfinished if the presentation layer contains rough edges.

---

## 9. Engineering Rules Established

1. Python calculates. AI reviews. Human decides.
2. Financial metrics must be deterministic and reproducible.
3. AI must not invent numbers or modify calculated results.
4. External API failures must not break report generation.
5. Use `.venv/bin/python` for all local validation.
6. Report language should be clear, direct, and non-promotional.
7. Research Score is a heuristic screening tool, not a prediction model.
8. Changelog records release changes; development log records engineering problem-solving.
9. New features should not be added before known issues are recorded and prioritized.
10. Avoid turning the tool into a stock-picking or trading signal system.
