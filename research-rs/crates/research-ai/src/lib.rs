pub mod client;
pub mod company_understanding;
pub mod financial_interpretation;
pub mod prompts;
pub mod research_blueprint;
pub mod schemas;
pub mod self_review;

use research_core::types::*;

pub fn run_local_compact_analyst(
    payload: &ProviderPayload,
) -> (
    CompanyUnderstanding,
    FinancialInterpretation,
    ResearchBlueprint,
    AiSelfReview,
    usize,
    usize,
) {
    let understanding = company_understanding::understand_company(payload);
    let interpretation = financial_interpretation::interpret_financials(payload, &understanding);
    let blueprint = research_blueprint::build_blueprint(payload, &understanding, &interpretation);
    let review = self_review::review(payload, &understanding, &interpretation, &blueprint);
    (understanding, interpretation, blueprint, review, 0, 0)
}

#[cfg(test)]
mod tests;
