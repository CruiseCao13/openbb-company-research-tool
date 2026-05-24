use research_core::types::ReportStatus;

#[derive(Debug, Clone)]
pub struct LintResult {
    pub failed_checks: Vec<String>,
    pub warnings: Vec<String>,
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
        let lhs = expected.to_lowercase();
        let rhs = frame.to_lowercase();
        let framework_conflict =
            (lhs.contains("financial") && !rhs.contains("financial") && !rhs.contains("bank"))
                || (lhs.contains("semiconductor") && !rhs.contains("semiconductor"))
                || (lhs.contains("shipping")
                    && !rhs.contains("shipping")
                    && !rhs.contains("transport"));
        if framework_conflict {
            failed.push("wrong_framework_conflict".to_string());
        } else if lhs.contains("biotech") && !rhs.contains("biotech") && !rhs.contains("pharma") {
            warnings.push("framework_limited".to_string());
        }
    }
    LintResult {
        failed_checks: failed,
        warnings,
    }
}
