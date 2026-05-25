# Prompt Change Log

## 2026-05-25 - LUNR wrong-framework emergency guard

- Prompt: `company_understanding_v1.md`
- Related issue: `wrong_profile`, `hallucinated_revenue_engine`
- Case: `LUNR` / Intuitive Machines was previously allowed to drift into a telecom frame despite provider identity clues for space, lunar infrastructure, NASA-linked mission services, and aerospace execution.
- Change: the prompt now requires reading provider name, sector, industry, and business description before choosing a research frame. It forbids telecom frames unless provider data explicitly supports wireless carrier, broadband service, subscribers, or telecom network revenue.
- Before quality: real external run produced a wrong telecom frame and should be treated as a hard failure.
- After quality: deterministic local regression produces a space/aerospace or data-limited frame with human review when evidence is insufficient.
- Default status: retained as `company_understanding_v1.md` for v5 alpha compatibility; future prompt edits should be versioned to `v2` after broad regression comparison.
- Rollback risk: overfitting LUNR. The regression set must continue checking AAPL, GOOGL, CAT, AMD, JPM, ZIM, ASTS, 600519.SH, and 000001.SZ.
