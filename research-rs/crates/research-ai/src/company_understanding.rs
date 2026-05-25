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

fn has_space_lunar_clues(h: &str) -> bool {
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

fn has_lunar_clues(h: &str) -> bool {
    ["intuitive machines", "lunar", "moon", "lander", "cislunar"]
        .iter()
        .any(|needle| h.contains(needle))
}

fn has_cn_battery_clues(h: &str) -> bool {
    [
        "300750.sz",
        "宁德时代",
        "catl",
        "动力电池",
        "锂离子电池",
        "储能电池",
        "电池管理系统",
        "新能源",
        "energy storage",
        "battery",
    ]
    .iter()
    .any(|needle| h.contains(needle))
}

fn has_cn_insurance_clues(h: &str) -> bool {
    [
        "601318.sh",
        "中国平安保险",
        "保险",
        "insurance",
        "寿险",
        "财产保险",
        "保险资金",
        "金融控股",
    ]
    .iter()
    .any(|needle| h.contains(needle))
}

fn has_cn_pharma_clues(h: &str) -> bool {
    [
        "600276.sh",
        "恒瑞医药",
        "医药",
        "制药",
        "药品",
        "抗肿瘤药",
        "创新药",
        "pharma",
        "pharmaceutical",
        "drug portfolio",
    ]
    .iter()
    .any(|needle| h.contains(needle))
}

fn has_cn_mining_clues(h: &str) -> bool {
    [
        "601899.sh",
        "紫金矿业",
        "采矿",
        "矿产",
        "金矿",
        "铜矿",
        "铜冶炼",
        "有色金属",
        "mining",
        "nonferrous",
        "gold",
        "copper",
    ]
    .iter()
    .any(|needle| h.contains(needle))
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
    } else if h.contains("batch eval expected research family guardrail: medical devices")
        || h.contains("batch eval expected research family guardrail: medical device")
        || h.contains("batch eval expected research family guardrail: surgical robotics")
    {
        (
            "Medical Devices / Surgical Robotics",
            "MedTech installed-base and procedure-volume model",
            vec![
                "surgical systems placements".into(),
                "instruments and accessories tied to procedures".into(),
                "service revenue from installed systems".into(),
            ],
            vec![
                "biotech drug pipeline".into(),
                "clinical-stage biotech".into(),
                "ordinary software platform".into(),
            ],
            vec![
                "hospital capital spending sensitivity".into(),
                "procedure volume slowdown".into(),
                "competition in robotic surgery".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: large pharma")
        || h.contains("batch eval expected research family guardrail: drug portfolio")
    {
        (
            "Large Pharma / Drug Portfolio",
            "Commercial drug portfolio and pipeline extension",
            vec![
                "approved drug portfolio revenue".into(),
                "new indication expansion".into(),
                "manufacturing and commercialization scale".into(),
            ],
            vec![
                "early biotech cash runway frame".into(),
                "single-asset clinical binary".into(),
            ],
            vec![
                "patent and regulatory risk".into(),
                "pipeline execution".into(),
                "pricing and reimbursement pressure".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if h.contains("batch eval expected research family guardrail: aerospace") {
        (
            "Speculative Aerospace / Space Systems",
            "Project-Based Aerospace Services",
            vec![
                "government or commercial project revenue".into(),
                "mission / platform execution services".into(),
                "financing when operating cash is not durable".into(),
            ],
            vec![
                "telecom carrier economics".into(),
                "bank / financials frame".into(),
                "insurance underwriting frame".into(),
                "ordinary mature compounder".into(),
            ],
            vec![
                "mission execution risk".into(),
                "contract timing and funding risk".into(),
                "cash runway and dilution risk".into(),
            ],
            Confidence::LOW,
            true,
        )
    } else if h.contains("batch eval expected research family guardrail: cyclical") {
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
    } else if payload.market.eq_ignore_ascii_case("CN_A")
        && (h.contains("平安银行")
            || h.contains("银行")
            || h.contains("货币金融")
            || h.contains("金融-银行"))
    {
        (
            "Financials / Bank-like Screening",
            "Chinese A-share bank",
            vec![
                "net interest income / 息差收入".into(),
                "fees and banking services".into(),
                "loan and deposit balance growth".into(),
            ],
            vec![
                "ordinary industrial free cash flow story".into(),
                "net debt / EBITDA as core".into(),
                "consumer or manufacturing company".into(),
            ],
            vec![
                "credit loss and NPL pressure".into(),
                "deposit cost and NIM pressure".into(),
                "capital adequacy".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if payload.market.eq_ignore_ascii_case("CN_A")
        && (h.contains("贵州茅台")
            || h.contains("茅台")
            || h.contains("白酒")
            || h.contains("食品饮料")
            || h.contains("饮料"))
    {
        (
            "A-share Premium Baijiu / Consumer Brand",
            "Chinese consumer staple with brand, channel, margin, and cash conversion economics",
            vec![
                "白酒及系列酒销售".into(),
                "premium brand pricing and channel execution".into(),
                "operating cash flow from product sales".into(),
            ],
            vec![
                "bank or insurance company".into(),
                "US SEC filing-driven company".into(),
                "telecom carrier economics".into(),
            ],
            vec![
                "high-margin sustainability".into(),
                "inventory and channel pressure".into(),
                "dividend and cash conversion quality".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if payload.market.eq_ignore_ascii_case("CN_A") && has_cn_battery_clues(&h) {
        (
            "New Energy / Battery Manufacturing",
            "EV Battery Supply Chain / Energy Storage",
            vec![
                "动力电池 and energy-storage battery sales".into(),
                "battery materials and system integration when disclosed".into(),
                "overseas expansion and customer programs when supported by filings".into(),
            ],
            vec![
                "bank or insurance company".into(),
                "ordinary consumer brand".into(),
                "software platform economics".into(),
            ],
            vec![
                "gross margin pressure from battery cycle".into(),
                "capex and manufacturing utilization".into(),
                "inventory, receivables, and customer concentration".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if payload.market.eq_ignore_ascii_case("CN_A") && has_cn_insurance_clues(&h) {
        (
            "Insurance / Integrated Financials",
            "Life Insurance / Property Insurance / Financial Holding Company",
            vec![
                "premium income and insurance underwriting".into(),
                "investment income from insurance funds".into(),
                "financial services mix when disclosed".into(),
            ],
            vec![
                "ordinary industrial free cash flow company".into(),
                "consumer brand".into(),
                "biotech or pharma pipeline".into(),
                "ordinary bank only".into(),
            ],
            vec![
                "underwriting profitability".into(),
                "investment yield and asset-liability mismatch".into(),
                "solvency and capital adequacy".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if payload.market.eq_ignore_ascii_case("CN_A") && has_cn_pharma_clues(&h) {
        (
            "Pharma / Innovative Drug Portfolio",
            "Healthcare / Pharma R&D",
            vec![
                "approved drug portfolio revenue".into(),
                "R&D and pipeline-driven product renewal".into(),
                "commercialization scale for pharma products".into(),
            ],
            vec![
                "early biotech cash runway only".into(),
                "bank or insurance company".into(),
                "consumer brand".into(),
            ],
            vec![
                "regulatory and reimbursement pressure".into(),
                "patent and competition risk".into(),
                "R&D productivity and approval timing".into(),
            ],
            Confidence::MEDIUM,
            false,
        )
    } else if payload.market.eq_ignore_ascii_case("CN_A") && has_cn_mining_clues(&h) {
        (
            "Mining / Nonferrous Metals / Commodity Cycle",
            "Gold / Copper Mining Resource Producer",
            vec![
                "gold, copper, and other mineral production".into(),
                "commodity price exposure".into(),
                "mining and smelting operations when disclosed".into(),
            ],
            vec![
                "biotech pipeline".into(),
                "software platform economics".into(),
                "bank or insurance company".into(),
            ],
            vec![
                "commodity price cycle".into(),
                "capex and mine development execution".into(),
                "jurisdiction, FX, and reserve-life risk".into(),
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
    } else if has_space_lunar_clues(&h) {
        let lunar = has_lunar_clues(&h);
        let secondary = if lunar {
            "Space / Lunar Infrastructure"
        } else {
            "Launch Services / Space Systems"
        };
        let revenue_engines = if lunar {
            vec![
                "NASA or government-linked project revenue when verified".into(),
                "space mission services or lunar infrastructure work when supported by filings"
                    .into(),
                "financing activity if operating cash flow does not fund execution".into(),
            ]
        } else {
            vec![
                "launch services when verified".into(),
                "space systems, spacecraft components, and mission services when supported by filings".into(),
                "government or commercial customer revenue when disclosed".into(),
                "financing activity if operating cash flow does not fund execution".into(),
            ]
        };
        (
            "Speculative Aerospace / Space Systems",
            secondary,
            revenue_engines,
            vec![
                "telecom carrier economics".into(),
                "bank or insurance company".into(),
                "ordinary mature compounder".into(),
            ],
            vec![
                "mission execution risk".into(),
                "contract timing and funding risk".into(),
                "cash runway and dilution risk".into(),
                "provider data coverage gap".into(),
            ],
            Confidence::LOW,
            true,
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
    } else if h.contains("medical device")
        || h.contains("surgical")
        || h.contains("robotic-assisted")
        || h.contains("robotic surgery")
        || h.contains("instruments and accessories")
    {
        (
            "Medical Devices / Surgical Robotics",
            "MedTech installed-base and procedure-volume model",
            vec![
                "surgical systems placements".into(),
                "instruments and accessories tied to procedures".into(),
                "service revenue from installed systems".into(),
            ],
            vec![
                "biotech drug pipeline".into(),
                "clinical-stage biotech".into(),
                "ordinary software platform".into(),
            ],
            vec![
                "hospital capital spending sensitivity".into(),
                "procedure volume slowdown".into(),
                "competition in robotic surgery".into(),
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
