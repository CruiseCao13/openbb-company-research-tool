use research_core::types::*;

pub fn build_blueprint(
    _payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
) -> ResearchBlueprint {
    let frame = understanding.correct_research_frame.clone();
    let lower = frame.to_lowercase();
    let (must, must_not, questions, red_flags, valuation, gaps, next) = if lower
        .contains("semiconductor")
    {
        (
            vec!["data center revenue".into(), "gross margin sustainability".into(), "capacity / supply constraint".into(), "customer concentration".into(), "export control risk".into()],
            vec!["bank metrics".into(), "generic mature-compounder conclusion".into()],
            vec!["Is demand durable beyond the current AI/capex cycle?".into(), "Can margins hold as supply expands?".into(), "What data is missing for segment-level evidence?".into()],
            vec!["hyperscaler capex slowdown".into(), "export controls".into(), "margin compression".into()],
            "Use revenue growth, gross margin durability, capex intensity, and valuation premium risk; do not use a target price.".into(),
            vec!["segment revenue / margin".into(), "customer concentration".into(), "supply constraints".into()],
            vec!["Verify data center / AI revenue split.".into(), "Check gross margin bridge and capacity commentary.".into(), "Review export-control and customer concentration disclosures.".into()],
        )
    } else if lower.contains("financial") || lower.contains("bank") {
        (
            vec![
                "ROE".into(),
                "ROA".into(),
                "NIM".into(),
                "credit loss".into(),
                "deposit cost".into(),
                "capital ratio".into(),
            ],
            vec![
                "industrial FCF as core".into(),
                "net debt / EBITDA as core".into(),
            ],
            vec![
                "Is ROE supported by asset quality or leverage?".into(),
                "Are credit losses rising?".into(),
                "Is funding cost pressuring profitability?".into(),
            ],
            vec![
                "credit deterioration".into(),
                "deposit cost pressure".into(),
                "capital ratio weakness".into(),
            ],
            "Use P/B, ROE, NIM, credit quality, funding cost, and capital adequacy.".into(),
            vec![
                "NIM".into(),
                "credit losses".into(),
                "capital ratio".into(),
                "deposit cost".into(),
            ],
            vec![
                "Pull NIM and deposit-cost trend.".into(),
                "Check credit loss and provision coverage.".into(),
                "Review capital ratios and asset quality.".into(),
            ],
        )
    } else if lower.contains("biotech") || lower.contains("pharma") {
        (
            vec!["pipeline stage".into(), "clinical milestones".into(), "cash runway".into(), "R&D burn".into(), "dilution risk".into()],
            vec!["ordinary PE".into(), "ordinary SaaS growth logic".into()],
            vec!["Which pipeline assets drive value?".into(), "How long can current cash fund R&D?".into(), "What regulatory milestones can change the thesis?".into()],
            vec!["trial failure".into(), "financing dilution".into(), "regulatory delay".into()],
            "Use cash runway, R&D burn, partnership quality, regulatory milestones, and dilution risk.".into(),
            vec!["pipeline stage".into(), "trial status".into(), "FDA / EMA milestones".into()],
            vec!["Map candidates, indications, phase, and next milestone timing.".into(), "Check regulatory path and trial disclosure cadence.".into(), "Calculate cash runway and dilution pressure.".into()],
        )
    } else if lower.contains("shipping") || lower.contains("transport") {
        (
            vec!["freight rate / yield".into(), "fleet utilization".into(), "fuel cost".into(), "orderbook".into(), "leverage".into()],
            vec!["biotech pipeline".into(), "software margin logic".into()],
            vec!["Where is the company in the transport cycle?".into(), "Are rates and utilization normalizing?".into(), "Can leverage survive a downcycle?".into()],
            vec!["rate-cycle reversal".into(), "fuel cost shock".into(), "excess capacity".into()],
            "Use cycle-normalized earnings, utilization, rate/yield cycle, fuel cost, leverage, and fleet/orderbook.".into(),
            vec!["rate/yield trend".into(), "fleet utilization".into(), "orderbook".into()],
            vec!["Check freight/yield trend and utilization.".into(), "Review fuel-cost exposure and fleet age.".into(), "Compare leverage against cycle stress.".into()],
        )
    } else {
        (
            vec![
                "business model".into(),
                "money flow".into(),
                "industry-specific drivers".into(),
            ],
            vec!["confident complete industry conclusion".into()],
            vec![
                "What does this company actually sell?".into(),
                "Which metrics are industry-specific?".into(),
                "What data is missing?".into(),
            ],
            vec![
                "framework uncertainty".into(),
                "missing industry metrics".into(),
            ],
            "Screening-only until industry-specific valuation drivers are verified.".into(),
            interpretation.unsupported_due_to_missing_data.clone(),
            vec![
                "Read the latest filing for revenue engines.".into(),
                "Identify industry-specific KPIs.".into(),
                "Decide which valuation method is actually suitable.".into(),
            ],
        )
    };

    ResearchBlueprint {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        core_thesis: format!("The central research question is whether the {} frame is supported by locked data and company-specific evidence.", frame),
        asset_profile: frame,
        secondary_profile: understanding.key_growth_drivers.first().cloned().unwrap_or_else(|| "Secondary profile not verified".into()),
        must_analyze: must,
        must_not_analyze_as_core: must_not,
        key_questions: questions,
        red_flags,
        valuation_frame: valuation,
        data_gaps: gaps,
        next_checks: next,
        report_section_guidance: vec!["Start with identity, then money flow, then valuation fit, then unsupported claims.".into()],
        confidence: understanding.confidence.clone(),
        human_review_required: understanding.human_review_required,
    }
}
