use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchOutcome {
    PASS,
    WARNING,
    UNVERIFIED_EXPECTED,
    UNVERIFIED_UNEXPECTED,
    FAIL,
    REPORT_FAILED,
    FETCH_FAILED,
}
