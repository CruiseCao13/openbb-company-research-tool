use crate::types::*;

fn identity_haystack(payload: &ProviderPayload) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        payload.ticker,
        payload.market,
        payload.company_profile.name,
        payload.company_profile.sector,
        payload.company_profile.industry,
        payload.company_profile.description,
        payload.metadata.source
    )
    .to_lowercase()
}

fn ai_haystack(
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    _review: &AiSelfReview,
) -> String {
    [
        understanding.company_identity.clone(),
        understanding.business_model.clone(),
        understanding.revenue_engines.join(" "),
        understanding.profit_pool.clone(),
        understanding.key_growth_drivers.join(" "),
        understanding.key_risks.join(" "),
        understanding.correct_research_frame.clone(),
        interpretation.revenue_explanation.clone(),
        interpretation.cash_flow_explanation.clone(),
        interpretation.where_money_comes_from.clone(),
        interpretation.where_money_goes.clone(),
        blueprint.core_thesis.clone(),
        blueprint.asset_profile.clone(),
        blueprint.must_analyze.join(" "),
    ]
    .join(" ")
    .to_lowercase()
}

pub fn has_space_lunar_identity(payload: &ProviderPayload) -> bool {
    let h = identity_haystack(payload);
    [
        "intuitive machines",
        " space",
        "lunar",
        "moon",
        "nasa",
        "aerospace",
        "defense",
        "mission",
        "lander",
        "exploration",
        "satellite",
        "cislunar",
        "launch",
        "spacecraft",
    ]
    .iter()
    .any(|needle| h.contains(needle))
}

pub fn has_explicit_telecom_carrier_identity(payload: &ProviderPayload) -> bool {
    let h = identity_haystack(payload);
    (h.contains("telecom")
        || h.contains("telecommunications")
        || h.contains("wireless carrier")
        || h.contains("broadband")
        || h.contains("subscriber")
        || h.contains("communications services"))
        && !has_space_lunar_identity(payload)
}

fn has_lunar_identity(payload: &ProviderPayload) -> bool {
    let h = identity_haystack(payload);
    ["intuitive machines", "lunar", "moon", "lander", "cislunar"]
        .iter()
        .any(|needle| h.contains(needle))
}

pub fn detect_wrong_framework_conflicts(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    review: &AiSelfReview,
) -> Vec<String> {
    let mut failures = Vec::new();
    let ai = ai_haystack(understanding, interpretation, blueprint, review);
    if has_space_lunar_identity(payload) && !has_explicit_telecom_carrier_identity(payload) {
        let forbidden_terms = [
            "telecom / infrastructure cash flow",
            "wireless service revenue",
            "broadband / network revenue",
            "subscriber churn",
            "regulated telecom",
            "bank-like screening",
            "insurance-like screening",
            "mature compounder",
        ];
        for term in forbidden_terms {
            if ai.contains(term) {
                failures.push(format!(
                    "wrong_framework_conflict:space_lunar_identity_contains_{}",
                    term.replace([' ', '/', '-'], "_")
                ));
            }
        }
    }
    failures
}

pub fn apply_framework_challenge_guard(
    payload: &ProviderPayload,
    understanding: &mut CompanyUnderstanding,
    interpretation: &mut FinancialInterpretation,
    blueprint: &mut ResearchBlueprint,
    review: &mut AiSelfReview,
) -> Vec<String> {
    let conflicts =
        detect_wrong_framework_conflicts(payload, understanding, interpretation, blueprint, review);
    if conflicts.is_empty() {
        return conflicts;
    }

    let company_name = if payload.company_profile.name.trim().is_empty() {
        payload.ticker.clone()
    } else {
        payload.company_profile.name.clone()
    };
    let lunar_context = has_lunar_identity(payload);

    understanding.company_identity = if lunar_context {
        format!("{company_name} requires a space / lunar infrastructure or aerospace-services frame based on the locked provider profile. The current payload is not enough for a full financial conclusion.")
    } else {
        format!("{company_name} requires a launch services / space systems / aerospace-services frame based on the locked provider profile. The current payload is not enough for a full financial conclusion.")
    };
    understanding.business_model = "The safe starting point is project-based aerospace services, mission execution, launch/space-systems work, and contract funding. The report must avoid carrier-style access-service economics unless filings prove that model.".into();
    understanding.revenue_engines = if lunar_context {
        vec![
            "NASA or government-linked project revenue when verified".into(),
            "space mission services or lunar infrastructure work when supported by filings".into(),
            "financing activity if operating cash flow does not fund execution".into(),
        ]
    } else {
        vec![
            "launch services when verified".into(),
            "space systems, spacecraft components, and mission services when supported by filings"
                .into(),
            "government or commercial customer revenue when disclosed".into(),
            "financing activity if operating cash flow does not fund execution".into(),
        ]
    };
    understanding.profit_pool = "Profitability cannot be treated as verified from the compact payload; the key questions are contract margin, mission execution cost, cash runway, and financing need.".into();
    understanding.key_growth_drivers = vec![
        "NASA / government contract execution".into(),
        if lunar_context {
            "lunar mission milestones".into()
        } else {
            "launch cadence and mission milestones".into()
        },
        "space systems backlog if verified".into(),
    ];
    understanding.key_risks = vec![
        "mission execution risk".into(),
        "contract timing and funding risk".into(),
        "cash runway and dilution risk".into(),
        "provider data coverage gap".into(),
    ];
    understanding.not_this = vec![
        "telecom carrier economics".into(),
        "bank or insurance company".into(),
        "ordinary mature compounder".into(),
    ];
    understanding.correct_research_frame =
        "Unknown / Data-Limited Screening with Aerospace extension".into();
    understanding.wrong_frames_to_avoid = vec![
        "telecom carrier frame".into(),
        "bank / financials frame".into(),
        "insurance frame".into(),
        "ordinary mature compounder frame".into(),
    ];
    understanding.confidence = Confidence::LOW;
    understanding.human_review_required = true;

    interpretation.revenue_explanation = "Current provider data should be read as a data-limited aerospace/project-execution case. Revenue cannot be described as recurring carrier access economics unless filings explicitly support that.".into();
    interpretation.margin_explanation = "Margin quality depends on project cost, milestone execution, and contract terms. The compact payload does not verify durable aerospace margins.".into();
    interpretation.cash_flow_explanation = "Cash-flow interpretation must separate operating cash generation from mission spending and financing. If cash-flow rows are incomplete, the report must stop at a data gap.".into();
    interpretation.where_money_comes_from = if lunar_context {
        "Money may come from verified project or government-linked contract revenue and from financing if operations are not self-funding; the current compact payload requires manual filing checks before stronger claims.".into()
    } else {
        "Money may come from launch services, space systems, spacecraft components, mission services, government or commercial customer contracts, and financing if operations are not self-funding; the current compact payload requires manual filing checks before stronger claims.".into()
    };
    interpretation.where_money_goes = if lunar_context {
        "Money likely goes to mission execution, engineering, launch or lander-related costs, working capital, and financing obligations when present; exact amounts require locked cash-flow data or filings.".into()
    } else {
        "Money likely goes to launch vehicle development, space systems engineering, spacecraft components, mission operations, working capital, and financing obligations when present; exact amounts require locked cash-flow data or filings.".into()
    };
    interpretation.capex_or_rnd_pressure = "Engineering and mission execution spend are central checks. Treat them as cash-consumption and milestone-delivery questions, not as a generic network capex story.".into();
    interpretation.debt_and_financing = "Financing risk must be checked directly because project-based aerospace companies can depend on new funding before operating cash flow is durable.".into();
    interpretation.shareholder_return_quality = "Buybacks or dividends are not a core frame unless locked data shows them; the default check is dilution and funding quality.".into();
    interpretation.valuation_method_fit = "Use a speculative aerospace/project execution frame. Ordinary telecom, bank, insurance, or mature-compounder valuation shortcuts are not suitable without supporting provider evidence.".into();
    if !interpretation
        .unsupported_due_to_missing_data
        .iter()
        .any(|x| x.contains("aerospace"))
    {
        interpretation.unsupported_due_to_missing_data.push(
            if lunar_context {
                "Space/lunar project backlog, NASA contract terms, mission milestones, and cash runway are not fully verified in the compact payload.".into()
            } else {
                "Launch backlog, customer contract terms, mission milestones, space systems margins, and cash runway are not fully verified in the compact payload.".into()
            },
        );
    }

    blueprint.core_thesis = if lunar_context {
        "The central research question is whether the company can convert space/lunar mission execution and contract funding into durable cash generation before financing risk becomes the main story.".into()
    } else {
        "The central research question is whether the company can convert launch services, space systems work, and mission execution into durable cash generation before financing risk becomes the main story.".into()
    };
    blueprint.asset_profile = "Unknown / Data-Limited Screening with Aerospace extension".into();
    blueprint.secondary_profile = if lunar_context {
        "Space / Lunar Infrastructure".into()
    } else {
        "Launch Services / Space Systems".into()
    };
    blueprint.must_analyze = if lunar_context {
        vec![
            "NASA or government-linked contract evidence".into(),
            "mission milestone execution".into(),
            "project margin and cost overrun risk".into(),
            "cash burn and financing runway".into(),
            "customer or contract concentration".into(),
        ]
    } else {
        vec![
            "launch services and space systems revenue split".into(),
            "spacecraft components and mission services evidence".into(),
            "launch cadence and mission execution".into(),
            "cash burn and financing runway".into(),
            "customer or contract concentration".into(),
        ]
    };
    blueprint.must_not_analyze_as_core = vec![
        "telecom carrier economics".into(),
        "bank / financials metrics".into(),
        "insurance underwriting metrics".into(),
        "ordinary mature-compounder shortcut".into(),
    ];
    blueprint.key_questions = vec![
        "Which contracts, milestones, and missions are actually disclosed?".into(),
        "Does operating cash flow cover mission execution and engineering spend?".into(),
        "How much financing or dilution is needed if project cash flow is delayed?".into(),
    ];
    blueprint.red_flags = vec![
        "mission delay or failure".into(),
        "contract funding gap".into(),
        "cash runway pressure".into(),
        "unsupported revenue-engine claims".into(),
    ];
    blueprint.valuation_frame = "Use project execution, backlog/contract quality when available, cash runway, dilution risk, and scenario framing. Do not force telecom or mature-compounder multiples.".into();
    blueprint.data_gaps = vec![
        if lunar_context {
            "NASA / customer contract details".into()
        } else {
            "customer contract details".into()
        },
        if lunar_context {
            "backlog and milestone timing".into()
        } else {
            "launch backlog and cadence".into()
        },
        if lunar_context {
            "mission-level cost and margin".into()
        } else {
            "space systems margin and component mix".into()
        },
        "cash runway and financing terms".into(),
    ];
    blueprint.next_checks = vec![
        "Read the latest filing for contract revenue, backlog, and customer concentration.".into(),
        "Map mission milestones to expected cash receipts and execution spend.".into(),
        "Calculate cash runway from operating cash flow, capex/engineering spend, and financing availability.".into(),
    ];
    blueprint.report_section_guidance = vec![
        "Lead with identity and data limits, then explain mission/project economics, money flow, and manual filing checks.".into(),
    ];
    blueprint.confidence = Confidence::LOW;
    blueprint.human_review_required = true;

    review.company_understanding_check = CheckStatus::FAIL;
    review.framework_fit_check = CheckStatus::FAIL;
    review.final_confidence = Confidence::LOW;
    review.human_review_required = true;
    if !review
        .wrong_framework_risk
        .iter()
        .any(|x| x.contains("space/lunar"))
    {
        review.wrong_framework_risk.push(
            "Provider identity has space/lunar aerospace clues, so carrier-style infrastructure framing is blocked and requires human review.".into(),
        );
    }
    for section in [
        "Company Identity",
        "Business Model",
        "Money Flow",
        "AI Research Blueprint",
    ] {
        if !review
            .required_rewrite_sections
            .iter()
            .any(|existing| existing == section)
        {
            review.required_rewrite_sections.push(section.into());
        }
    }

    conflicts
}

pub fn validate_provider_payload(payload: &ProviderPayload) -> Vec<String> {
    let mut failures = Vec::new();
    if payload.ticker.trim().is_empty() {
        failures.push("invalid_provider_payload:ticker_missing".to_string());
    }
    if payload.company_profile.name.trim().is_empty() && payload.error.is_none() {
        failures.push("weak_company_identity:name_missing".to_string());
    }
    if payload.market.eq_ignore_ascii_case("CN_A") && payload.error.is_none() {
        if !matches!(payload.company_profile.currency.as_str(), "CNY" | "RMB") {
            failures.push("cn_a_provider_payload:currency_missing_or_not_cny".to_string());
        }
        if payload
            .price_history
            .iter()
            .all(|point| point.close.is_none())
        {
            failures.push("cn_a_provider_payload:price_history_missing".to_string());
        }
        if payload
            .income_statement
            .iter()
            .all(|row| row.value.is_none())
        {
            failures.push("cn_a_provider_payload:income_statement_missing".to_string());
        }
        if payload.balance_sheet.iter().all(|row| row.value.is_none()) {
            failures.push("cn_a_provider_payload:balance_sheet_missing".to_string());
        }
        if payload.cash_flow.iter().all(|row| row.value.is_none()) {
            failures.push("cn_a_provider_payload:cash_flow_missing".to_string());
        }
    }
    failures
}

pub fn validate_ai_json(
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
    _review: &AiSelfReview,
) -> Vec<String> {
    let mut failures = Vec::new();
    if understanding.company_identity.trim().is_empty() {
        failures.push("missing_company_understanding".to_string());
    }
    if understanding.not_this.is_empty() {
        failures.push("missing_not_this_boundary".to_string());
    }
    if interpretation.where_money_comes_from.trim().is_empty()
        || interpretation.where_money_goes.trim().is_empty()
    {
        failures.push("missing_money_flow".to_string());
    }
    let money_flow = format!(
        "{} {}",
        interpretation.where_money_comes_from, interpretation.where_money_goes
    )
    .to_lowercase();
    let generic_money_flow = [
        "money comes from operating revenue when available",
        "money goes to operating costs",
        "costs and reinvestment",
        "pay attention to cash flow",
        "cash flow is important",
    ]
    .iter()
    .any(|phrase| money_flow.contains(phrase));
    if generic_money_flow {
        failures.push("generic_money_flow".to_string());
    }
    if blueprint.core_thesis.trim().len() < 40 {
        failures.push("blueprint_too_generic".to_string());
    }
    if blueprint.asset_profile.trim().is_empty() || blueprint.must_analyze.is_empty() {
        failures.push("missing_research_blueprint".to_string());
    }
    if understanding.ai_provenance.source.is_empty()
        || interpretation.ai_provenance.source.is_empty()
        || blueprint.ai_provenance.source.is_empty()
    {
        failures.push("missing_ai_provenance".to_string());
    }
    failures
}

pub fn detect_forbidden_advice(text: &str) -> bool {
    let lower = text.to_lowercase();
    let boundary_phrases = [
        "does not provide a target price",
        "not provide a target price",
        "not investment advice",
        "do not use a target price",
        "does not provide a target price, buy/sell recommendation",
        "not provide buy/sell",
        "not a buy/sell",
    ];
    let sanitized = boundary_phrases
        .iter()
        .fold(lower, |acc, phrase| acc.replace(phrase, ""));
    [
        "buy rating",
        "sell rating",
        "target price",
        "price target",
        "should buy",
        "should sell",
    ]
    .iter()
    .any(|needle| sanitized.contains(needle))
}

pub fn report_status(
    payload_failures: &[String],
    ai_failures: &[String],
    review: &AiSelfReview,
    provider_status: String,
    ai_mode: String,
    ai_calls: usize,
    cache_hits: usize,
) -> ReportStatus {
    let human_review_required = review.human_review_required
        || !payload_failures.is_empty()
        || !ai_failures.is_empty()
        || provider_status != "PASS"
        || review.final_confidence == Confidence::LOW
        || !review.required_rewrite_sections.is_empty();
    let overall_status = if ai_failures.iter().any(|f| {
        f.contains("wrong_framework")
            || f.contains("unsupported")
            || f.contains("hallucinated_revenue_engine")
    }) {
        "FAIL"
    } else if human_review_required {
        "WARNING"
    } else {
        "PASS"
    };
    ReportStatus {
        schema_version: SCHEMA_VERSION.to_string(),
        overall_status: overall_status.to_string(),
        provider_payload_valid: if payload_failures.is_empty() {
            "PASS"
        } else {
            "WARNING"
        }
        .to_string(),
        company_understanding_present: "PASS".to_string(),
        financial_interpretation_present: "PASS".to_string(),
        research_blueprint_present: "PASS".to_string(),
        ai_self_review_present: "PASS".to_string(),
        money_flow_present: if ai_failures.iter().any(|f| f == "missing_money_flow") {
            "FAIL"
        } else {
            "PASS"
        }
        .to_string(),
        human_review_required,
        ai_mode,
        ai_calls,
        cache_hits,
        provider_status,
        visual_lint_status: "PASS".to_string(),
        pdf_export_status: "PASS".to_string(),
    }
}

pub fn visual_lint(
    report: &str,
    dashboard_exists: bool,
    chart_manifest_exists: bool,
    data_coverage_exists: bool,
    chart_table_quality_exists: bool,
    pdf_export_report_exists: bool,
) -> (String, Vec<String>) {
    let mut failures = Vec::new();
    if !report.starts_with('#') || !(report.contains("> Status:") || report.contains("> 状态："))
    {
        failures.push("report_has_status_block".to_string());
    }
    if !(report.contains("## Table of Contents") || report.contains("## 目录")) {
        failures.push("report_has_toc".to_string());
    }
    if !(report.contains("What to look at:")
        || report.contains("Company-specific observation:")
        || report.contains("怎么看："))
    {
        failures.push("chart_explanations_present".to_string());
    }
    if report.contains("NaN") || report.contains("null") || report.contains("[METRIC_MISSING_RAW]")
    {
        failures.push("no_raw_nan_or_placeholder".to_string());
    }
    if !dashboard_exists {
        failures.push("dashboard_exists".to_string());
    }
    if !chart_manifest_exists {
        failures.push("chart_manifest_exists".to_string());
    }
    if !data_coverage_exists {
        failures.push("data_usage_coverage_report_exists".to_string());
    }
    if !chart_table_quality_exists {
        failures.push("chart_table_quality_report_exists".to_string());
    }
    if !pdf_export_report_exists {
        failures.push("pdf_export_report_exists".to_string());
    }
    if detect_forbidden_advice(report) {
        failures.push("no_forbidden_advice".to_string());
    }
    let status = if failures.is_empty() { "PASS" } else { "FAIL" }.to_string();
    (status, failures)
}
