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
    let profile_text = format!(
        "{} {} {}",
        payload.company_profile.name,
        payload.company_profile.industry,
        payload.company_profile.description
    )
    .to_lowercase();
    let lunar_context = ["intuitive machines", "lunar", "moon", "lander", "cislunar"]
        .iter()
        .any(|needle| profile_text.contains(needle));
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

    let frame_lower = understanding.correct_research_frame.to_lowercase();
    if frame_lower.contains("aerospace")
        || frame_lower.contains("space")
        || frame_lower.contains("lunar")
    {
        return FinancialInterpretation {
            schema_version: SCHEMA_VERSION.to_string(),
            ai_provenance: AiProvenance::default(),
            revenue_explanation: match revenue {
                Some(v) => format!("Locked data includes latest revenue around {:.1}. For a space/aerospace project company, that number must be tied to contract delivery, mission milestones, and customer concentration before it can support a durable revenue claim.", v),
                None => "Revenue is not available in locked provider data; this report cannot verify contract concentration, mission cash timing, or project revenue quality.".into(),
            },
            margin_explanation: "Margin quality depends on project cost, mission execution, milestone timing, and contract terms. Do not read it like a carrier, bank, insurer, or mature compounder.".into(),
            cash_flow_explanation: match (op_cf, capex) {
                (Some(cfo), Some(cx)) => format!("Operating cash flow is {:.1}; capital expenditure is {:.1}. The key question is whether mission execution and engineering spend consume more cash than contracts produce.", cfo, cx),
                (Some(cfo), None) => format!("Operating cash flow is {:.1}, but capex/project spend is not fully visible. Cash runway and financing need remain manual checks.", cfo),
                _ => "Cash-flow detail is incomplete; this report cannot verify whether project execution is self-funded or dependent on external financing.".into(),
            },
            where_money_comes_from: if lunar_context {
                "Money may come from NASA or government-linked project revenue, mission services, lunar infrastructure work, and financing if operating cash flow does not cover execution spend; current locked data requires filing checks before contract timing can be verified.".into()
            } else {
                "Money may come from launch services, space systems, spacecraft components, mission services, government or commercial customer contracts, and financing if operating cash flow does not cover execution spend; current locked data requires filing checks before contract timing can be verified.".into()
            },
            where_money_goes: if lunar_context {
                "Money goes to mission execution, engineering work, payload/lander development, working capital, and financing obligations when present; exact contract margin and cash runway remain data gaps unless filings provide the split.".into()
            } else {
                "Money goes to launch vehicle development, space systems engineering, spacecraft components, mission operations, working capital, and financing obligations when present; exact contract margin and cash runway remain data gaps unless filings provide the split.".into()
            },
            capex_or_rnd_pressure: "Engineering, mission execution, and project delivery spend are central. Treat them as cash-runway and milestone-delivery questions, not generic capex.".into(),
            debt_and_financing: match debt {
                Some(v) => format!("Debt-like obligations appear in locked data around {:.1}; financing terms and dilution risk should be checked against cash runway.", v),
                None => "Debt and financing pressure are not fully visible in the compact payload; cash runway and possible dilution remain manual checks.".into(),
            },
            shareholder_return_quality: "Shareholder returns are not a core frame unless locked data shows them; financing quality, dilution, and runway matter first.".into(),
            valuation_method_fit: "Use a speculative aerospace/project-execution frame. Ordinary PE or telecom-style infrastructure multiples are not meaningful unless profitability and contract durability are verified.".into(),
            unsupported_due_to_missing_data: {
                let mut gaps = unsupported;
                gaps.push(if lunar_context {
                    "Contract backlog, NASA/customer concentration, mission milestone timing, and cash runway are not fully verified in the compact payload.".into()
                } else {
                    "Contract backlog, customer concentration, launch cadence, mission milestone timing, and cash runway are not fully verified in the compact payload.".into()
                });
                gaps
            },
        };
    }

    FinancialInterpretation {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
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
