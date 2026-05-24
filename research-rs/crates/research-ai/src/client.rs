use research_core::types::ProviderPayload;

pub fn external_ai_available() -> bool {
    std::env::var("OPENAI_API_KEY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn compact_payload_size(payload: &ProviderPayload) -> usize {
    payload.company_profile.description.len()
        + payload.income_statement.len() * 32
        + payload.cash_flow.len() * 32
        + payload.balance_sheet.len() * 32
}
