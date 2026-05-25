use research_core::types::ReportStatus;

#[derive(Debug, Clone)]
pub struct LintResult {
    pub failed_checks: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn expected_family_conflict(expected: &str, frame: &str) -> bool {
    let lhs = expected.to_lowercase();
    let rhs = frame.to_lowercase();
    let rhs_has_any = |terms: &[&str]| terms.iter().any(|term| rhs.contains(term));

    if lhs.contains("medical devices") || lhs.contains("surgical robotics") {
        return rhs_has_any(&["biotech", "pharma", "drug pipeline", "clinical-stage"])
            || !rhs_has_any(&["medical", "device", "surgical", "robot", "medtech"]);
    }
    if lhs.contains("space") || lhs.contains("aerospace") || lhs.contains("launch") {
        return rhs_has_any(&["telecom", "wireless", "broadband", "bank", "insurance"])
            || !rhs_has_any(&[
                "space",
                "aerospace",
                "launch",
                "lunar",
                "project-based",
                "data-limited",
            ]);
    }
    if lhs.contains("financial") || lhs.contains("bank") {
        return !rhs_has_any(&["financial", "bank"]);
    }
    if lhs.contains("platform internet") || lhs.contains("digital ads") {
        return rhs_has_any(&["financial", "bank", "insurance"]);
    }
    if lhs.contains("industrial machinery") || lhs.contains("cyclical") {
        return rhs_has_any(&["insurance", "bank", "biotech", "pharma"]);
    }
    if lhs.contains("shipping") || lhs.contains("transport") {
        return rhs_has_any(&["biotech", "bank", "insurance"])
            || !rhs_has_any(&["shipping", "transport", "cycle"]);
    }
    if lhs.contains("consumer") || lhs.contains("baijiu") {
        return rhs_has_any(&["financial", "bank", "biotech", "semiconductor"]);
    }

    (lhs.contains("semiconductor") && !rhs.contains("semiconductor"))
        || (lhs.contains("insurance") && !rhs.contains("insurance"))
}

pub fn lint_status(
    status: &ReportStatus,
    expected_family: Option<&String>,
    frame: &str,
) -> LintResult {
    let mut failed = Vec::new();
    let mut warnings = Vec::new();
    if status.overall_status == "FAIL" {
        failed.push("report_status_fail".to_string());
    }
    if let Some(expected) = expected_family {
        if expected_family_conflict(expected, frame) {
            failed.push("wrong_framework_conflict".to_string());
        } else if expected.to_lowercase().contains("biotech")
            && !frame.to_lowercase().contains("biotech")
            && !frame.to_lowercase().contains("pharma")
        {
            warnings.push("framework_limited".to_string());
        }
    }
    LintResult {
        failed_checks: failed,
        warnings,
    }
}
