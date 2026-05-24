use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "v5.0.0";

pub fn default_schema_version() -> String {
    SCHEMA_VERSION.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderPayload {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
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
pub struct ValuationSnapshot {
    pub pe: Option<f64>,
    pub ps: Option<f64>,
    pub pb: Option<f64>,
    pub ev_revenue: Option<f64>,
    pub ev_ebitda: Option<f64>,
    pub market_cap: Option<f64>,
    pub raw_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NormalizedFinancials {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub ticker: String,
    pub reporting_currency: String,
    pub income_statement: Vec<StatementRow>,
    pub balance_sheet: Vec<StatementRow>,
    pub cash_flow: Vec<StatementRow>,
    pub valuation_snapshot: ValuationSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NormalizedPriceHistory {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub ticker: String,
    pub price_currency: String,
    pub points: Vec<PricePoint>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompanyUnderstanding {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub ai_provenance: AiProvenance,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinancialInterpretation {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub ai_provenance: AiProvenance,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchBlueprint {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub ai_provenance: AiProvenance,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiSelfReview {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub ai_provenance: AiProvenance,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Confidence {
    HIGH,
    MEDIUM,
    #[default]
    LOW,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CheckStatus {
    PASS,
    #[default]
    WARNING,
    FAIL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatus {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
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
    pub visual_lint_status: String,
    pub pdf_export_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub ticker: String,
    pub provider: String,
    pub status: String,
    pub cache_hit: bool,
    pub attempts: usize,
    pub stdout_excerpt: String,
    pub stderr_excerpt: String,
    pub user_message: String,
    pub suggested_next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTrace {
    pub stage: String,
    pub status: String,
    pub duration_ms: u128,
    pub cache_hit: bool,
    pub provider_used: Option<String>,
    pub ai_calls: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub output_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTrace {
    pub ticker: String,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub total_ms: u128,
    pub provider_used: String,
    pub ai_mode: String,
    pub ai_calls: usize,
    pub cache_hits: usize,
    pub stages: Vec<StageTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQualityScore {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub content_quality_score: u8,
    pub visual_quality_score: u8,
    pub data_quality_score: u8,
    pub ai_provenance_score: u8,
    pub money_flow_score: u8,
    pub evidence_score: u8,
    pub ai_confidence_score: u8,
    pub reproducibility_score: u8,
    pub completeness_score: u8,
    pub overall_product_score: u8,
    pub grade: String,
    pub chart_table_score: u8,
    pub visual_lint_status: String,
    pub presentation_status: String,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPassResult {
    pub pass_name: String,
    pub status: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub evidence: Vec<String>,
    pub suggested_fix: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitPolicy {
    pub reporting_currency: String,
    pub price_currency: String,
    pub financial_statement_unit: String,
    pub percentage_format: String,
    pub multiple_format: String,
    pub share_count_unit: String,
    pub date_range: String,
    pub provider_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub ticker: String,
    pub run_id: String,
    pub generated_at: String,
    pub files: Vec<String>,
    pub report_status: String,
    pub ai_mode: String,
    pub provider: String,
    pub has_dashboard: bool,
    pub has_pdf: bool,
    pub has_charts: bool,
    pub has_self_review: bool,
    pub file_entries: Vec<PackFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
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
    pub lang: String,
    pub mode: String,
    pub require_external_ai: bool,
    pub no_ai_cache: bool,
    pub max_attempts: usize,
    pub auto_fix: bool,
    pub fail_fast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiTaskUsage {
    pub task: String,
    pub source: String,
    pub prompt_version: String,
    pub external_ai_used: bool,
    pub external_ai_attempted: bool,
    pub external_ai_used_from_cache: bool,
    pub local_mock_used: bool,
    pub cache_hit: bool,
    pub new_external_ai_call: bool,
    pub request_attempted: bool,
    pub request_success: bool,
    pub request_id: String,
    pub model: String,
    pub input_tokens_estimate: usize,
    pub output_tokens_estimate: usize,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiUsage {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub ai_provenance: AiProvenance,
    pub ai_mode: String,
    pub require_external_ai: bool,
    pub no_ai_cache: bool,
    pub external_ai_used: bool,
    pub external_ai_attempted: bool,
    pub external_ai_used_from_cache: bool,
    pub local_mock_used: bool,
    pub cache_used: bool,
    pub ai_calls: usize,
    pub new_external_ai_calls: usize,
    pub cache_hits: usize,
    pub model: String,
    pub provider: String,
    pub tasks: Vec<AiTaskUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvenance {
    pub source: String,
    pub external_ai_used: bool,
    pub local_mock_used: bool,
    pub cache_hit: bool,
    pub new_external_ai_call: bool,
    pub model: String,
    pub prompt_version: String,
    pub request_attempted: bool,
    pub request_success: bool,
    pub request_id: String,
    pub generated_at: String,
    pub input_digest: String,
    pub output_digest: String,
}

impl Default for AiProvenance {
    fn default() -> Self {
        Self {
            source: "local_mock".into(),
            external_ai_used: false,
            local_mock_used: true,
            cache_hit: false,
            new_external_ai_call: false,
            model: "local-compact-analyst-fallback".into(),
            prompt_version: "local_fallback".into(),
            request_attempted: false,
            request_success: false,
            request_id: String::new(),
            generated_at: String::new(),
            input_digest: String::new(),
            output_digest: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub run_id: String,
    pub total: usize,
    pub pass: usize,
    pub warning: usize,
    pub fail: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCase {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub ticker: String,
    pub issue_type: String,
    pub expected_behavior: String,
}
