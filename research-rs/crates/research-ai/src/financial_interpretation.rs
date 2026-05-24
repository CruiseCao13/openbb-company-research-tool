use research_core::types::*;

fn latest_metric(rows: &[StatementRow], needles: &[&str]) -> Option<f64> {
    rows.iter()
        .find(|r| needles.iter().any(|n| r.metric.to_lowercase().contains(n)))
        .and_then(|r| r.value)
}

pub fn interpret_financials(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
) -> FinancialInterpretation {
    let revenue = latest_metric(&payload.income_statement, &["revenue", "total revenue"]);
    let op_cf = latest_metric(
        &payload.cash_flow,
        &["operating cash flow", "cash from operations"],
    );
    let capex = latest_metric(&payload.cash_flow, &["capital expenditure", "capex"]);
    let rnd = latest_metric(&payload.income_statement, &["research", "r&d"]);
    let debt = latest_metric(&payload.balance_sheet, &["debt"]);

    let unsupported = if payload.error.is_some() {
        vec!["Provider payload has an error; numeric conclusions must stay screening-only.".into()]
    } else if revenue.is_none() {
        vec!["Revenue is missing from the compact provider payload.".into()]
    } else {
        Vec::new()
    };

    FinancialInterpretation {
        schema_version: SCHEMA_VERSION.to_string(),
        revenue_explanation: match revenue {
            Some(v) => format!("Locked data includes latest revenue around {:.1}. The report can discuss revenue direction only within provider coverage.", v),
            None => "Revenue is not available in locked provider data; revenue quality cannot be treated as verified evidence.".into(),
        },
        margin_explanation: format!("Margin interpretation must use the {} frame and avoid cross-industry shortcuts.", understanding.correct_research_frame),
        cash_flow_explanation: match (op_cf, capex) {
            (Some(cfo), Some(cx)) => format!("Operating cash flow is {:.1}; capital expenditure is {:.1}. Free cash flow quality depends on the gap between operating cash generation and reinvestment needs.", cfo, cx),
            (Some(cfo), None) => format!("Operating cash flow is {:.1}, but capex is not available. Free cash flow quality remains incomplete.", cfo),
            _ => "Cash flow data is incomplete; the report must flag cash generation limits instead of inferring quality.".into(),
        },
        where_money_comes_from: "Money comes from operating revenue when available, operating cash flow if positive, and financing when operating cash is insufficient.".into(),
        where_money_goes: format!(
            "Money goes to operating costs, reinvestment, {}{} and financing obligations when present.",
            if rnd.is_some() { "R&D, " } else { "" },
            if capex.is_some() { "capex," } else { "working capital," }
        ),
        capex_or_rnd_pressure: if understanding.correct_research_frame.to_lowercase().contains("biotech") {
            "R&D burn and runway matter more than ordinary PE.".into()
        } else if understanding.correct_research_frame.to_lowercase().contains("semiconductor") {
            "Capex, manufacturing capacity, and gross-margin recovery are central to the cash-flow story.".into()
        } else {
            "Reinvestment pressure should be judged against the company-specific frame, not a one-size-fits-all metric.".into()
        },
        debt_and_financing: match debt {
            Some(v) => format!("Debt-like obligations appear in locked data around {:.1}; financing risk should be reviewed in filings.", v),
            None => "Debt and financing pressure are not fully visible in the compact payload.".into(),
        },
        shareholder_return_quality: "Buybacks and dividends are interpretation topics only when the locked data and company frame support them.".into(),
        valuation_method_fit: format!("Valuation should fit {}. The report must not force PE, PS, or FCF yield when they do not explain the asset.", understanding.correct_research_frame),
        unsupported_due_to_missing_data: unsupported,
    }
}
