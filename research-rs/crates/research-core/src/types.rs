use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderPayload {
    pub ticker: String,
    pub market: String,
    pub provider: String,
    pub fetched_at: String,
    pub company_profile: CompanyProfile,
    #[serde(default)]
    pub price_history: Vec<PricePoint>,
    #[serde(default)]
    pub income_statement: Vec<StatementRow>,
    #[serde(default)]
    pub balance_sheet: Vec<StatementRow>,
    #[serde(default)]
    pub cash_flow: Vec<StatementRow>,
    #[serde(default)]
    pub valuation_snapshot: serde_json::Value,
    #[serde(default)]
    pub segments: Vec<serde_json::Value>,
    #[serde(default)]
    pub metadata: ProviderMetadata,
    #[serde(default)]
    pub error: Option<ProviderError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompanyProfile {
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub description: String,
    pub exchange: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PricePoint {
    pub date: String,
    pub close: Option<f64>,
    pub volume: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatementRow {
    pub period: String,
    pub metric: String,
    pub value: Option<f64>,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderMetadata {
    #[serde(default)]
    pub data_quality_warnings: Vec<String>,
    pub source: String,
    pub provider_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderError {
    pub error_type: String,
    pub error_message: String,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyUnderstanding {
    pub company_identity: String,
    pub business_model: String,
    pub revenue_engines: Vec<String>,
    pub profit_pool: String,
    pub key_growth_drivers: Vec<String>,
    pub key_risks: Vec<String>,
    pub not_this: Vec<String>,
    pub correct_research_frame: String,
    pub wrong_frames_to_avoid: Vec<String>,
    pub confidence: Confidence,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialInterpretation {
    pub revenue_explanation: String,
    pub margin_explanation: String,
    pub cash_flow_explanation: String,
    pub where_money_comes_from: String,
    pub where_money_goes: String,
    pub capex_or_rnd_pressure: String,
    pub debt_and_financing: String,
    pub shareholder_return_quality: String,
    pub valuation_method_fit: String,
    pub unsupported_due_to_missing_data: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchBlueprint {
    pub core_thesis: String,
    pub asset_profile: String,
    pub secondary_profile: String,
    pub must_analyze: Vec<String>,
    pub must_not_analyze_as_core: Vec<String>,
    pub key_questions: Vec<String>,
    pub red_flags: Vec<String>,
    pub valuation_frame: String,
    pub data_gaps: Vec<String>,
    pub next_checks: Vec<String>,
    pub report_section_guidance: Vec<String>,
    pub confidence: Confidence,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSelfReview {
    pub company_understanding_check: CheckStatus,
    pub framework_fit_check: CheckStatus,
    pub numeric_consistency_check: CheckStatus,
    pub money_flow_check: CheckStatus,
    pub unsupported_claims: Vec<String>,
    pub wrong_framework_risk: Vec<String>,
    pub required_rewrite_sections: Vec<String>,
    pub final_confidence: Confidence,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Confidence {
    HIGH,
    MEDIUM,
    LOW,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    PASS,
    WARNING,
    FAIL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatus {
    pub overall_status: String,
    pub provider_payload_valid: String,
    pub company_understanding_present: String,
    pub financial_interpretation_present: String,
    pub research_blueprint_present: String,
    pub ai_self_review_present: String,
    pub money_flow_present: String,
    pub human_review_required: bool,
    pub ai_mode: String,
    pub ai_calls: usize,
    pub cache_hits: usize,
    pub provider_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContext {
    pub ticker: String,
    pub market: String,
    pub provider: String,
    pub ai_mode: String,
    pub run_id: String,
    pub root: String,
    pub force: bool,
    pub pack: bool,
}
