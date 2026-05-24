use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResearchErrorKind {
    ProviderError,
    ProviderTimeout,
    ProviderSchemaInvalid,
    ProviderDataEmpty,
    ProviderFallbackUsed,
    AiUnavailable,
    AiJsonInvalid,
    AiSchemaInvalid,
    AiUnsupportedClaim,
    ValidationFailed,
    VisualLintFailed,
    ReportRenderFailed,
    ChartGenerationFailed,
    PdfExportFailed,
    BatchTickerFailed,
    CacheReadFailed,
    CacheWriteFailed,
    ConfigInvalid,
    IoFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchError {
    pub kind: ResearchErrorKind,
    pub stage: String,
    pub ticker: String,
    pub provider: Option<String>,
    pub recoverable: bool,
    pub fallback_attempted: bool,
    pub user_message: String,
    pub debug_message: String,
    pub suggested_next_action: String,
    pub log_path: Option<String>,
}

impl ResearchError {
    pub fn provider_failure(
        ticker: &str,
        provider: &str,
        stage: &str,
        debug_message: String,
    ) -> Self {
        Self {
            kind: ResearchErrorKind::ProviderError,
            stage: stage.to_string(),
            ticker: ticker.to_string(),
            provider: Some(provider.to_string()),
            recoverable: true,
            fallback_attempted: false,
            user_message: "Provider data fetch failed; the run should degrade or retry without crashing the whole batch.".to_string(),
            debug_message,
            suggested_next_action:
                "Retry with --force, try a fallback provider, or inspect audit/provider_validation.md."
                    .to_string(),
            log_path: None,
        }
    }
}
