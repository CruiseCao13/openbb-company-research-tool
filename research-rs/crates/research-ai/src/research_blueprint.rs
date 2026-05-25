use research_core::types::*;

pub fn build_blueprint(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
) -> ResearchBlueprint {
    let frame = understanding.correct_research_frame.clone();
    let lower = frame.to_lowercase();
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
    } else if lower.contains("aerospace") || lower.contains("space") || lower.contains("lunar") {
        (
            if lunar_context {
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
            },
            vec![
                "telecom carrier economics".into(),
                "bank / financials metrics".into(),
                "insurance underwriting metrics".into(),
                "ordinary mature-compounder shortcut".into(),
            ],
            vec![
                "Which missions, customers, and contracts are actually disclosed?".into(),
                "Does operating cash flow cover engineering and mission execution spend?".into(),
                "How much financing or dilution is needed if project receipts are delayed?".into(),
            ],
            vec![
                "mission delay or failure".into(),
                "contract funding gap".into(),
                "cash runway pressure".into(),
                "unsupported revenue-engine claims".into(),
            ],
            "Use project execution, backlog/contract quality when available, cash runway, dilution risk, and scenario framing. Do not force telecom, bank, insurance, or mature-compounder multiples.".into(),
            if lunar_context {
                vec![
                    "NASA/customer contract details".into(),
                    "backlog and milestone timing".into(),
                    "mission-level cost and margin".into(),
                    "cash runway and financing terms".into(),
                ]
            } else {
                vec![
                    "customer contract details".into(),
                    "launch backlog and cadence".into(),
                    "space systems margin and component mix".into(),
                    "cash runway and financing terms".into(),
                ]
            },
            vec![
                "Read the latest filing for contract revenue, backlog, and customer concentration.".into(),
                if lunar_context {
                    "Map mission milestones to expected cash receipts and execution spend.".into()
                } else {
                    "Map launch cadence, space systems orders, and mission milestones to expected cash receipts and execution spend.".into()
                },
                "Calculate cash runway from operating cash flow, engineering/project spend, and financing availability.".into(),
            ],
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
                "credit loss / provision expense trend".into(),
                "deposit cost trend".into(),
                "loan mix and collateral detail".into(),
            ],
            vec![
                "Pull NIM and deposit-cost trend.".into(),
                "Check credit loss and provision coverage.".into(),
                "Review capital ratios and asset quality.".into(),
            ],
        )
    } else if lower.contains("baijiu")
        || lower.contains("consumer brand")
        || lower.contains("consumer staple")
    {
        (
            vec![
                "营业收入 and volume/price mix".into(),
                "归母净利润 and 扣非归母净利润".into(),
                "毛利率 / 净利率 durability".into(),
                "经营现金流 and cash conversion".into(),
                "inventory, receivables, and dividend sustainability".into(),
            ],
            vec![
                "US SEC filing-driven 10-K / 10-Q frame".into(),
                "telecom carrier economics".into(),
                "ordinary speculative project company".into(),
            ],
            vec![
                "Is revenue supported by product/channel quality rather than one-off effects?".into(),
                "Do margins and cash conversion support the brand-quality story?".into(),
                "Are inventory, receivables, and dividends consistent with healthy cash flow?".into(),
            ],
            vec![
                "channel inventory pressure".into(),
                "margin normalization".into(),
                "dividend unsupported by cash conversion".into(),
            ],
            "Use A-share consumer-brand metrics: revenue, attributable net profit, non-recurring-adjusted net profit, margin, operating cash flow, inventory/receivables, ROE, and RMB valuation context. Do not use a US filing template.".into(),
            vec![
                "dividend detail".into(),
                "contract liabilities / advances".into(),
                "cash and interest-bearing debt detail".into(),
            ],
            vec![
                "Check annual report for product mix, channel inventory, and contract liabilities.".into(),
                "Compare operating cash flow with attributable net profit and dividend cash outflow.".into(),
                "Verify RMB units and A-share accounting labels before publishing.".into(),
            ],
        )
    } else if lower.contains("medical devices")
        || lower.contains("surgical robotics")
        || lower.contains("medtech")
    {
        (
            vec![
                "installed surgical robotics base".into(),
                "procedure volume".into(),
                "instruments and accessories revenue".into(),
                "system placements".into(),
                "hospital capital spending sensitivity".into(),
            ],
            vec![
                "biotech drug pipeline".into(),
                "clinical-stage biotech cash runway".into(),
                "ordinary SaaS growth logic".into(),
            ],
            vec![
                "Are procedure volumes supporting recurring instrument revenue?".into(),
                "Are system placements expanding the installed base?".into(),
                "Is hospital capital spending constraining growth?".into(),
            ],
            vec![
                "procedure slowdown".into(),
                "hospital capex pressure".into(),
                "robotic surgery competition".into(),
            ],
            "Use installed base, procedure volume, instruments/accessories mix, system placements, and hospital capex sensitivity. Do not use biotech pipeline or cash-runway framing as the core.".into(),
            vec![
                "procedure volume trend".into(),
                "installed base and utilization".into(),
                "system placement data".into(),
            ],
            vec![
                "Check procedure growth and instruments/accessories revenue mix.".into(),
                "Review system placement and installed-base disclosures.".into(),
                "Compare hospital capital spending pressure with system demand.".into(),
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
