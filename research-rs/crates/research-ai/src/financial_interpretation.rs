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
    let revenue = latest_metric(
        &payload.income_statement,
        &["revenue", "total revenue", "营业收入"],
    );
    let op_cf = latest_metric(
        &payload.cash_flow,
        &["operating cash flow", "cash from operations", "经营现金"],
    );
    let capex = latest_metric(&payload.cash_flow, &["capital expenditure", "capex"]);
    let rnd = latest_metric(&payload.income_statement, &["research", "r&d"]);
    let debt = latest_metric(&payload.balance_sheet, &["debt", "有息负债", "负债合计"]);
    let parent_net_profit = latest_metric(&payload.income_statement, &["归母净利润"]);
    let gross_margin = latest_metric(&payload.income_statement, &["毛利率"]);
    let net_margin = latest_metric(&payload.income_statement, &["净利率"]);
    let nim = latest_metric(&payload.balance_sheet, &["净息差"]);
    let npl = latest_metric(&payload.balance_sheet, &["不良贷款率"]);
    let capital_ratio = latest_metric(&payload.balance_sheet, &["资本充足率"]);

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

    if payload.market.eq_ignore_ascii_case("CN_A") && frame_lower.contains("bank") {
        return FinancialInterpretation {
            schema_version: SCHEMA_VERSION.to_string(),
            ai_provenance: AiProvenance::default(),
            revenue_explanation: match revenue {
                Some(v) => format!("锁定数据给出最近一期营业收入约 {:.1} CNY。银行收入需要继续拆成净利息收入、手续费和其他非息收入，不能按普通工业收入看。", v),
                None => "当前 provider 没有给出银行营业收入；不能验证收入结构。".into(),
            },
            margin_explanation: match (nim, parent_net_profit) {
                (Some(n), Some(p)) => format!("净息差约 {:.2}%，归母净利润约 {:.1} CNY。银行盈利质量要和资产质量、拨备、资本充足率一起看。", n, p),
                (Some(n), None) => format!("净息差约 {:.2}%，但归母净利润缺失；盈利质量仍需要年报口径核查。", n),
                _ => "银行净息差或归母净利润缺失；不能用普通毛利率/FCF 框架替代。".into(),
            },
            cash_flow_explanation: match op_cf {
                Some(v) => format!("provider 给出每股经营现金流/经营现金流相关指标约 {:.2}。银行经营现金流口径和工商企业不同，本报告只把它作为资金流辅助信号，核心仍看 NIM、资产质量、资本充足率和存贷款结构。", v),
                None => "银行经营现金流口径不适合作为核心 FCF 指标；当前 provider 也没有足够现金流明细。".into(),
            },
            where_money_comes_from: "钱主要来自存贷款利差、手续费及其他银行业务收入；当前 locked data 已提供营业收入、净息差、贷款和存款线索，但仍需年报拆分净利息收入与非息收入。".into(),
            where_money_goes: "钱主要消耗在资金成本、信用减值/拨备、运营费用和资本占用上；如果不良贷款率或资本充足率恶化，利润不能只看表面增长。".into(),
            capex_or_rnd_pressure: "银行不是工业 capex 模型；真正要查的是资产质量、拨备覆盖、资本充足率和存款成本。".into(),
            debt_and_financing: match (npl, capital_ratio) {
                (Some(n), Some(c)) => format!("不良贷款率约 {:.2}%，资本充足率约 {:.2}%。这比净债务/EBITDA 更适合作为银行风险入口。", n, c),
                _ => "不良贷款率或资本充足率缺失；银行风险需要人工核查。".into(),
            },
            shareholder_return_quality: "分红质量只有在监管资本、拨备和盈利稳定性可验证时才有意义；当前报告不把分红当成核心结论。".into(),
            valuation_method_fit: "银行更适合用 P/B、ROE、NIM、资产质量和资本充足率框架；不要使用普通工业 FCF 或净债务/EBITDA 作为核心。".into(),
            unsupported_due_to_missing_data: unsupported,
        };
    }

    if payload.market.eq_ignore_ascii_case("CN_A") {
        return FinancialInterpretation {
            schema_version: SCHEMA_VERSION.to_string(),
            ai_provenance: AiProvenance::default(),
            revenue_explanation: match revenue {
                Some(v) => format!("锁定数据给出最近一期营业收入约 {:.1} CNY。A 股消费公司要继续看收入增长是否来自价格、销量、渠道还是产品结构。", v),
                None => "当前 provider 没有给出营业收入；不能验证增长来源。".into(),
            },
            margin_explanation: match (gross_margin, net_margin) {
                (Some(g), Some(n)) => format!("毛利率约 {:.2}%，净利率约 {:.2}%。这能支持高毛利/强品牌的初步判断，但仍要结合费用率、渠道和产品结构。", g, n),
                _ => "毛利率或净利率缺失；不能只用收入判断品牌质量。".into(),
            },
            cash_flow_explanation: match op_cf {
                Some(v) => format!("provider 给出经营现金流相关指标约 {:.2}。消费品牌的关键是销售回款、预收/合同负债、库存和分红是否共同支持现金转化。", v),
                None => "经营现金流缺失；不能验证销售是否真正变成现金。".into(),
            },
            where_money_comes_from: "钱主要来自主营产品销售和经营现金流；当前 locked data 支持营业收入、归母净利润、毛利率/净利率和经营现金流相关指标的初步检查。".into(),
            where_money_goes: "钱主要花在生产成本、渠道/运营、库存占用、税费和分红上；如果库存或应收周转恶化，利润质量需要降级。".into(),
            capex_or_rnd_pressure: "这类 A 股消费公司通常不应先套重资产 capex 或研发烧钱框架；更重要的是品牌、渠道、库存、回款和现金分红。".into(),
            debt_and_financing: match debt {
                Some(v) => format!("负债相关指标约 {:.1}。这只是资产负债表入口，还需要结合货币资金、合同负债和有息负债明细。", v),
                None => "货币资金和有息负债明细不足；资金安全边界需要人工核查。".into(),
            },
            shareholder_return_quality: "分红只有在经营现金流、库存和货币资金同时支持时才健康；当前 provider 未完整提供分红明细，不能把高分红写成已验证事实。".into(),
            valuation_method_fit: "A 股消费龙头更适合看收入/利润韧性、毛利率、现金转化、ROE 和估值分位；不要使用美股 10-K/10-Q 叙事或普通项目型公司框架。".into(),
            unsupported_due_to_missing_data: {
                let mut gaps = unsupported;
                gaps.push("分红明细、货币资金、有息负债、合同负债和更细的库存/应收余额仍需年报核查。".into());
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
