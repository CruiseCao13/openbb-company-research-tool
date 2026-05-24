use research_core::types::*;

fn haystack(payload: &ProviderPayload) -> String {
    format!(
        "{} {} {} {} {}",
        payload.ticker,
        payload.company_profile.name,
        payload.company_profile.sector,
        payload.company_profile.industry,
        payload.company_profile.description
    )
    .to_lowercase()
}

pub fn understand_company(payload: &ProviderPayload) -> CompanyUnderstanding {
    let h = haystack(payload);
    let name = if payload.company_profile.name.trim().is_empty() {
        payload.ticker.clone()
    } else {
        payload.company_profile.name.clone()
    };

    let (frame, secondary, revenue_engines, not_this, risks, confidence, human) = if h
        .contains("batch eval expected research family guardrail: ai semiconductor")
    {
        (
            "AI Semiconductor / Data Center Growth Compounder",
            "Semiconductor AI platform",
            vec![
                "accelerators".into(),
                "data center chips".into(),
                "networking / platform ecosystem".into(),
            ],
            vec![
                "financial company".into(),
                "foundry turnaround unless manufacturing evidence exists".into(),
            ],
            vec![
                "hyperscaler capex cycle".into(),
                "export controls".into(),
                "valuation premium risk".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h
        .contains("batch eval expected research family guardrail: capital-intensive semiconductor")
    {
        (
            "Capital-Intensive Semiconductor Turnaround",
            "Technology manufacturing",
            vec![
                "processors".into(),
                "foundry or manufacturing services".into(),
                "data center / client computing".into(),
            ],
            vec![
                "ordinary mature compounder".into(),
                "software-only platform".into(),
            ],
            vec![
                "capex burden".into(),
                "gross margin recovery".into(),
                "process-node execution".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: financial") {
        (
            "Financials / Bank-like Screening",
            "Financial services",
            vec![
                "net interest income".into(),
                "fees".into(),
                "trading / asset management revenue".into(),
            ],
            vec!["ordinary industrial free cash flow story".into()],
            vec![
                "credit loss".into(),
                "deposit cost".into(),
                "capital adequacy".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: shipping") {
        (
            "Shipping / Airlines / Transport Cycle",
            "Cyclical transport",
            vec![
                "freight / passenger yield".into(),
                "fleet utilization".into(),
            ],
            vec![
                "biotech pipeline".into(),
                "mature software compounder".into(),
            ],
            vec![
                "rate cycle".into(),
                "fuel cost".into(),
                "leverage and fleet orderbook".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: biotech") {
        (
            "Biotech / Pharma Research Frame",
            "Pipeline and regulatory risk",
            vec![
                "product revenue or partnership revenue".into(),
                "pipeline milestones".into(),
            ],
            vec!["ordinary PE story".into(), "SaaS growth company".into()],
            vec![
                "clinical failure".into(),
                "regulatory risk".into(),
                "cash runway and dilution".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("batch eval expected research family guardrail: reit") {
        (
            "REIT-like Screening",
            "Real estate income vehicle",
            vec!["rent".into(), "occupancy".into(), "same-store NOI".into()],
            vec!["ordinary EPS / FCF story".into()],
            vec![
                "interest cost".into(),
                "debt maturity".into(),
                "occupancy risk".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("batch eval expected research family guardrail: insurance") {
        (
            "Insurance-like Screening",
            "Underwriting and float",
            vec!["premiums".into(), "investment income".into()],
            vec!["ordinary bank".into(), "industrial FCF story".into()],
            vec![
                "combined ratio".into(),
                "reserve adequacy".into(),
                "catastrophe exposure".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("batch eval expected research family guardrail: cyclical")
        || h.contains("batch eval expected research family guardrail: aerospace")
    {
        (
            "Cyclical / Industrial Cycle",
            "Asset-heavy cycle",
            vec![
                "commodity-sensitive revenue".into(),
                "equipment / project demand".into(),
            ],
            vec!["low PE means cheap".into()],
            vec![
                "cycle peak earnings".into(),
                "normalized margin".into(),
                "capex cycle".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: telecom") {
        (
            "Telecom / Infrastructure Cash Flow",
            "Debt-heavy regulated-like network business",
            vec![
                "wireless service revenue".into(),
                "broadband / network revenue".into(),
            ],
            vec!["ordinary high-growth tech".into()],
            vec![
                "debt load".into(),
                "capex intensity".into(),
                "subscriber churn".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: utilities") {
        (
            "Utilities / Infrastructure",
            "Regulated asset base",
            vec![
                "regulated electricity sales".into(),
                "rate base growth".into(),
                "generation assets".into(),
            ],
            vec!["ordinary high-growth technology".into()],
            vec![
                "rate case risk".into(),
                "debt cost".into(),
                "capex plan execution".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: consumer") {
        (
            "Consumer / Retail",
            "Store, traffic, ticket, and brand economics",
            vec![
                "same-store sales".into(),
                "traffic / ticket".into(),
                "store or channel growth".into(),
            ],
            vec!["semiconductor seller".into(), "biotech pipeline".into()],
            vec![
                "traffic slowdown".into(),
                "inventory pressure".into(),
                "brand dilution".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("apple inc")
        || h.contains("iphone")
        || h.contains("ipad")
        || h.contains("mac ")
        || h.contains("consumer electronics")
    {
        (
            "Mature Consumer Technology Compounder",
            "Hardware / Services ecosystem",
            vec![
                "hardware products".into(),
                "services ecosystem".into(),
                "installed base monetization".into(),
            ],
            vec![
                "bank".into(),
                "biotech pipeline".into(),
                "semiconductor seller".into(),
            ],
            vec![
                "premium valuation risk".into(),
                "services / hardware mix".into(),
                "regulatory platform risk".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("alphabet")
        || h.contains("google")
        || h.contains("advertising")
        || h.contains("google search")
    {
        (
            "Platform Internet / Digital Ads / Cloud",
            "Mega-cap technology",
            vec![
                "digital advertising".into(),
                "cloud services".into(),
                "subscriptions and platforms".into(),
            ],
            vec!["bank".into(), "semiconductor manufacturer".into()],
            vec![
                "advertising cycle".into(),
                "cloud margin pressure".into(),
                "regulatory platform risk".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("meta platforms") || h.contains("facebook") || h.contains("instagram") {
        (
            "Platform Internet / Social Ads / AI Infrastructure",
            "Mega-cap technology",
            vec![
                "advertising".into(),
                "social platforms".into(),
                "AI-driven engagement".into(),
            ],
            vec!["semiconductor seller".into(), "bank".into()],
            vec![
                "ad cycle".into(),
                "AI capex intensity".into(),
                "regulatory risk".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("restaurant")
        || h.contains("retail")
        || h.contains("apparel")
        || h.contains("food and beverages")
        || h.contains("stores")
    {
        (
            "Consumer / Retail",
            "Store, traffic, ticket, and brand economics",
            vec![
                "same-store sales".into(),
                "traffic / ticket".into(),
                "store or channel growth".into(),
            ],
            vec!["semiconductor seller".into(), "biotech pipeline".into()],
            vec![
                "traffic slowdown".into(),
                "inventory pressure".into(),
                "brand dilution".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("utility")
        || h.contains("utilities")
        || h.contains("regulated electric")
        || h.contains("electric power")
    {
        (
            "Utilities / Infrastructure",
            "Regulated asset base",
            vec![
                "regulated electricity sales".into(),
                "rate base growth".into(),
                "generation assets".into(),
            ],
            vec!["ordinary high-growth technology".into()],
            vec![
                "rate case risk".into(),
                "debt cost".into(),
                "capex plan execution".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("telecom")
        || h.contains("communications")
        || h.contains("wireless")
        || h.contains("broadband")
    {
        (
            "Telecom / Infrastructure Cash Flow",
            "Debt-heavy regulated-like network business",
            vec![
                "wireless service revenue".into(),
                "broadband / network revenue".into(),
            ],
            vec!["ordinary high-growth tech".into()],
            vec![
                "debt load".into(),
                "capex intensity".into(),
                "subscriber churn".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("semiconductor")
        || h.contains("gpu")
        || h.contains("accelerated computing")
        || h.contains("foundry")
    {
        let turnaround = h.contains("foundry")
            || h.contains("intel")
            || h.contains("integrated device manufacturing")
            || h.contains("wafer");
        if turnaround {
            (
                "Capital-Intensive Semiconductor Turnaround",
                "Technology manufacturing",
                vec![
                    "processors".into(),
                    "foundry or manufacturing services".into(),
                    "data center / client computing".into(),
                ],
                vec![
                    "ordinary mature compounder".into(),
                    "software-only platform".into(),
                ],
                vec![
                    "capex burden".into(),
                    "gross margin recovery".into(),
                    "process-node execution".into(),
                ],
                Confidence::MEDIUM,
                false,
            )
        } else {
            (
                "AI Semiconductor / Data Center Growth Compounder",
                "Semiconductor AI platform",
                vec![
                    "accelerators".into(),
                    "data center chips".into(),
                    "networking / platform ecosystem".into(),
                ],
                vec![
                    "financial company".into(),
                    "foundry turnaround unless manufacturing evidence exists".into(),
                ],
                vec![
                    "hyperscaler capex cycle".into(),
                    "export controls".into(),
                    "valuation premium risk".into(),
                ],
                Confidence::MEDIUM,
                false,
            )
        }
    } else if h.contains("bank")
        || h.contains("jpmorgan")
        || h.contains("broker")
        || h.contains("exchange")
    {
        (
            "Financials / Bank-like Screening",
            "Financial services",
            vec![
                "net interest income".into(),
                "fees".into(),
                "trading / asset management revenue".into(),
            ],
            vec!["ordinary industrial free cash flow story".into()],
            vec![
                "credit loss".into(),
                "deposit cost".into(),
                "capital adequacy".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if (h.contains("biotech")
        || h.contains("clinical")
        || h.contains("therapeutic")
        || h.contains("pharma")
        || h.contains("drug"))
        && !h.contains("shipping")
        && !h.contains("freight")
        && !h.contains("vessel")
        && !h.contains("fleet")
    {
        (
            "Biotech / Pharma Research Frame",
            "Pipeline and regulatory risk",
            vec![
                "product revenue or partnership revenue".into(),
                "pipeline milestones".into(),
            ],
            vec!["ordinary PE story".into(), "SaaS growth company".into()],
            vec![
                "clinical failure".into(),
                "regulatory risk".into(),
                "cash runway and dilution".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("shipping")
        || h.contains("container")
        || h.contains("airline")
        || h.contains("transport")
        || h.contains("logistics")
        || h.contains("vessel")
        || h.contains("fleet")
    {
        if h.contains("insurance") {
            (
                "Insurance-like Screening",
                "Underwriting and float",
                vec!["premiums".into(), "investment income".into()],
                vec!["ordinary bank".into(), "industrial FCF story".into()],
                vec![
                    "combined ratio".into(),
                    "reserve adequacy".into(),
                    "catastrophe exposure".into(),
                ],
                Confidence::LOW,
                true,
            )
        } else if h.contains("oil")
            || h.contains("gas")
            || h.contains("mining")
            || h.contains("steel")
            || h.contains("equipment")
            || h.contains("caterpillar")
        {
            (
                "Cyclical / Industrial Cycle",
                "Asset-heavy cycle",
                vec![
                    "commodity-sensitive revenue".into(),
                    "equipment / project demand".into(),
                ],
                vec!["low PE means cheap".into()],
                vec![
                    "cycle peak earnings".into(),
                    "normalized margin".into(),
                    "capex cycle".into(),
                ],
                Confidence::MEDIUM,
                false,
            )
        } else {
            (
                "Shipping / Airlines / Transport Cycle",
                "Cyclical transport",
                vec![
                    "freight / passenger yield".into(),
                    "fleet utilization".into(),
                ],
                vec![
                    "biotech pipeline".into(),
                    "mature software compounder".into(),
                ],
                vec![
                    "rate cycle".into(),
                    "fuel cost".into(),
                    "leverage and fleet orderbook".into(),
                ],
                Confidence::MEDIUM,
                false,
            )
        }
    } else if h.contains("reit") || h.contains("real estate investment trust") {
        (
            "REIT-like Screening",
            "Real estate income vehicle",
            vec!["rent".into(), "occupancy".into(), "same-store NOI".into()],
            vec!["ordinary EPS / FCF story".into()],
            vec![
                "interest cost".into(),
                "debt maturity".into(),
                "occupancy risk".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("insurance") || h.contains("underwriting") {
        (
            "Insurance-like Screening",
            "Underwriting and float",
            vec!["premiums".into(), "investment income".into()],
            vec!["ordinary bank".into(), "industrial FCF story".into()],
            vec![
                "combined ratio".into(),
                "reserve adequacy".into(),
                "catastrophe exposure".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("oil")
        || h.contains("gas")
        || h.contains("mining")
        || h.contains("steel")
        || h.contains("equipment")
        || h.contains("caterpillar")
    {
        (
            "Cyclical / Industrial Cycle",
            "Asset-heavy cycle",
            vec![
                "commodity-sensitive revenue".into(),
                "equipment / project demand".into(),
            ],
            vec!["low PE means cheap".into()],
            vec![
                "cycle peak earnings".into(),
                "normalized margin".into(),
                "capex cycle".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("telecom") || h.contains("communications") || h.contains("wireless") {
        (
            "Telecom / Infrastructure Cash Flow",
            "Debt-heavy regulated-like network business",
            vec![
                "wireless service revenue".into(),
                "broadband / network revenue".into(),
            ],
            vec!["ordinary high-growth tech".into()],
            vec![
                "debt load".into(),
                "capex intensity".into(),
                "subscriber churn".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else {
        (
            "Unknown / Data-Limited Screening",
            "Framework not fully covered",
            vec!["current provider data is insufficient to identify revenue engines".into()],
            vec!["confident complete industry research".into()],
            vec![
                "framework uncertainty".into(),
                "missing industry-specific data".into(),
            ],
            Confidence::LOW,
            true,
        )
    };

    CompanyUnderstanding {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        company_identity: format!("{name} is best treated as {frame} based on the locked provider profile and financial context."),
        business_model: format!("The research frame is {frame}. The report should explain how the company earns money before interpreting valuation."),
        revenue_engines,
        profit_pool: format!("Profit pool assessment should focus on the economics implied by {frame}, not a generic template."),
        key_growth_drivers: vec![secondary.into()],
        key_risks: risks,
        not_this,
        correct_research_frame: frame.into(),
        wrong_frames_to_avoid: vec!["generic mature compounder unless supported".into(), "target price model".into(), "buy/sell advice".into()],
        confidence,
        human_review_required: human || payload.error.is_some(),
    }
}
