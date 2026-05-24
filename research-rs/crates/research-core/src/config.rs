use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub prompt_version: String,
    pub model_name: String,
    pub provider_script: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            prompt_version: "v5.0-local-compact-analyst".to_string(),
            model_name: "local-compact-analyst-fallback".to_string(),
            provider_script: "providers/provider_common.py".to_string(),
        }
    }
}
