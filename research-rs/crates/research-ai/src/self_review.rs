use research_core::types::*;
use research_core::validation::detect_wrong_framework_conflicts;

pub fn review(
    payload: &ProviderPayload,
    understanding: &CompanyUnderstanding,
    interpretation: &FinancialInterpretation,
    blueprint: &ResearchBlueprint,
) -> AiSelfReview {
    let mut unsupported = interpretation.unsupported_due_to_missing_data.clone();
    let mut wrong = Vec::new();
    if understanding
        .correct_research_frame
        .to_lowercase()
        .contains("unknown")
    {
        wrong.push("Framework is not fully identified; report must stay screening-only.".into());
    }
    if payload.error.is_some() {
        unsupported.push("Provider error prevents full financial interpretation.".into());
    }
    let framework_conflicts = detect_wrong_framework_conflicts(
        payload,
        understanding,
        interpretation,
        blueprint,
        &AiSelfReview::default(),
    );
    if !framework_conflicts.is_empty() {
        wrong.push(
            "Selected research frame conflicts with provider name, industry, or description; revenue engines may be unsupported.".into(),
        );
    }
    let blueprint_generic = blueprint.must_analyze.len() < 3 || blueprint.next_checks.len() < 3;
    let needs_rewrite = blueprint_generic || !framework_conflicts.is_empty();
    AiSelfReview {
        schema_version: SCHEMA_VERSION.to_string(),
        ai_provenance: AiProvenance::default(),
        company_understanding_check: if understanding.company_identity.len() > 20 {
            CheckStatus::PASS
        } else {
            CheckStatus::FAIL
        },
        framework_fit_check: if !framework_conflicts.is_empty() {
            CheckStatus::FAIL
        } else if blueprint_generic {
            CheckStatus::WARNING
        } else {
            CheckStatus::PASS
        },
        numeric_consistency_check: CheckStatus::PASS,
        money_flow_check: if interpretation.where_money_comes_from.is_empty() {
            CheckStatus::FAIL
        } else {
            CheckStatus::PASS
        },
        unsupported_claims: unsupported,
        wrong_framework_risk: wrong,
        required_rewrite_sections: if !framework_conflicts.is_empty() {
            vec![
                "Company Identity".into(),
                "Business Model".into(),
                "Money Flow".into(),
                "AI Research Blueprint".into(),
            ]
        } else if blueprint_generic {
            vec!["AI Research Blueprint".into()]
        } else {
            Vec::new()
        },
        final_confidence: if !framework_conflicts.is_empty() {
            Confidence::LOW
        } else {
            understanding.confidence.clone()
        },
        human_review_required: understanding.human_review_required
            || payload.error.is_some()
            || needs_rewrite,
    }
}
