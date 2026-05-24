use research_core::types::*;

fn bullet(items: &[String]) -> String {
    if items.is_empty() {
        "- Not available from locked data.\n".to_string()
    } else {
        items.iter().map(|x| format!("- {x}\n")).collect()
    }
}

pub fn render_report(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    review: &AiSelfReview,
    status: &ReportStatus,
) -> String {
    let name = if payload.company_profile.name.is_empty() {
        payload.ticker.clone()
    } else {
        payload.company_profile.name.clone()
    };
    format!(
        r#"# {ticker} Company Research Report

> Version: v5.0  
> Company: {name}  
> Market: {market}  
> Provider: {provider}  
> Status: {status_value}  
> Note: This report is for first-pass research only. It is not investment advice.

## Table of Contents

1. Report Status
2. Company Identity
3. Business Model
4. Money Flow: Where Money Comes From and Where It Goes
5. Financial Statement Interpretation
6. AI Research Blueprint
7. Valuation Frame
8. Risks and Red Flags
9. Data Gaps and Unsupported Claims
10. AI Self Review
11. Next Checks
12. Appendix: Locked Data

## 1. Report Status

| Item | Value |
|---|---|
| Overall status | {status_value} |
| Provider status | {provider_status} |
| AI mode | {ai_mode} |
| AI calls | {ai_calls} |
| Cache hits | {cache_hits} |
| Human review required | {human_review} |

The status separates locked data availability from interpretation confidence. A warning means the report can be useful as a screening memo, but the unsupported sections need human review.

## 2. Company Identity

**Identity:** {identity}

**Correct research frame:** {frame}

**What this company is not:**  
{not_this}

## 3. Business Model

{business_model}

Revenue engines currently identified:

{revenue_engines}

Profit pool:

{profit_pool}

## 4. Money Flow: Where Money Comes From and Where It Goes

**Where money comes from:** {money_from}

**Where money goes:** {money_goes}

This matters because growth is not automatically valuable. The report needs to distinguish operating cash generation from financing, reinvestment, R&D, capex, working capital, buybacks, and debt service.

## 5. Financial Statement Interpretation

**Revenue:** {revenue_explanation}

**Margins:** {margin_explanation}

**Cash flow:** {cash_flow_explanation}

**Capex / R&D pressure:** {capex_rnd}

**Debt and financing:** {debt}

**Shareholder return quality:** {shareholder}

## 6. AI Research Blueprint

**Core thesis:** {core_thesis}

**Asset profile:** {asset_profile}

**Secondary profile:** {secondary_profile}

Must analyze:

{must_analyze}

Must not analyze as core:

{must_not}

Key questions:

{questions}

## 7. Valuation Frame

{valuation}

The report does not provide a target price, buy/sell recommendation, or short-term price prediction.

## 8. Risks and Red Flags

{red_flags}

## 9. Data Gaps and Unsupported Claims

Data gaps:

{data_gaps}

Unsupported claims flagged by AI self-review:

{unsupported}

## 10. AI Self Review

| Check | Status |
|---|---|
| Company understanding | {cu_check:?} |
| Framework fit | {ff_check:?} |
| Numeric consistency | {num_check:?} |
| Money flow | {money_check:?} |
| Final confidence | {confidence:?} |

Wrong-framework risks:

{wrong_risk}

## 11. Next Checks

{next_checks}

## 12. Appendix: Locked Data

| Field | Value |
|---|---|
| Ticker | {ticker} |
| Sector | {sector} |
| Industry | {industry} |
| Currency | {currency} |
| Price points | {price_count} |
| Income rows | {income_count} |
| Balance sheet rows | {balance_count} |
| Cash-flow rows | {cash_count} |

"#,
        ticker = payload.ticker,
        name = name,
        market = payload.market,
        provider = payload.provider,
        status_value = status.overall_status,
        provider_status = status.provider_status,
        ai_mode = status.ai_mode,
        ai_calls = status.ai_calls,
        cache_hits = status.cache_hits,
        human_review = status.human_review_required,
        identity = understanding.company_identity,
        frame = understanding.correct_research_frame,
        not_this = bullet(&understanding.not_this),
        business_model = understanding.business_model,
        revenue_engines = bullet(&understanding.revenue_engines),
        profit_pool = understanding.profit_pool,
        money_from = interpretation.where_money_comes_from,
        money_goes = interpretation.where_money_goes,
        revenue_explanation = interpretation.revenue_explanation,
        margin_explanation = interpretation.margin_explanation,
        cash_flow_explanation = interpretation.cash_flow_explanation,
        capex_rnd = interpretation.capex_or_rnd_pressure,
        debt = interpretation.debt_and_financing,
        shareholder = interpretation.shareholder_return_quality,
        core_thesis = blueprint.core_thesis,
        asset_profile = blueprint.asset_profile,
        secondary_profile = blueprint.secondary_profile,
        must_analyze = bullet(&blueprint.must_analyze),
        must_not = bullet(&blueprint.must_not_analyze_as_core),
        questions = bullet(&blueprint.key_questions),
        valuation = blueprint.valuation_frame,
        red_flags = bullet(&blueprint.red_flags),
        data_gaps = bullet(&blueprint.data_gaps),
        unsupported = bullet(&review.unsupported_claims),
        cu_check = review.company_understanding_check,
        ff_check = review.framework_fit_check,
        num_check = review.numeric_consistency_check,
        money_check = review.money_flow_check,
        confidence = review.final_confidence,
        wrong_risk = bullet(&review.wrong_framework_risk),
        next_checks = bullet(&blueprint.next_checks),
        sector = payload.company_profile.sector,
        industry = payload.company_profile.industry,
        currency = payload.company_profile.currency,
        price_count = payload.price_history.len(),
        income_count = payload.income_statement.len(),
        balance_count = payload.balance_sheet.len(),
        cash_count = payload.cash_flow.len(),
    )
}

pub fn render_self_review_md(review: &AiSelfReview) -> String {
    format!(
        "# AI Self Review\n\n| Check | Status |\n|---|---|\n| Company understanding | {:?} |\n| Framework fit | {:?} |\n| Numeric consistency | {:?} |\n| Money flow | {:?} |\n| Final confidence | {:?} |\n| Human review required | {} |\n\n## Unsupported Claims\n\n{}\n\n## Wrong-Framework Risk\n\n{}\n",
        review.company_understanding_check,
        review.framework_fit_check,
        review.numeric_consistency_check,
        review.money_flow_check,
        review.final_confidence,
        review.human_review_required,
        bullet(&review.unsupported_claims),
        bullet(&review.wrong_framework_risk),
    )
}
