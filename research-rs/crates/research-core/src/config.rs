use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub prompt_version: String,
    pub model_name: String,
    pub provider_script: String,
    pub ai_budget: AiBudgetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBudgetConfig {
    pub max_calls_per_single_run: usize,
    pub max_calls_per_batch: usize,
    pub max_calls_per_ticker: usize,
    pub warn_after_calls: usize,
    pub fail_after_calls: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            prompt_version: "v5.0-local-compact-analyst".to_string(),
            model_name: "local-compact-analyst-fallback".to_string(),
            provider_script: "providers/provider_common.py".to_string(),
            ai_budget: AiBudgetConfig::default(),
        }
    }
}

impl Default for AiBudgetConfig {
    fn default() -> Self {
        Self {
            max_calls_per_single_run: 8,
            max_calls_per_batch: 100,
            max_calls_per_ticker: 6,
            warn_after_calls: 50,
            fail_after_calls: 200,
        }
    }
}
