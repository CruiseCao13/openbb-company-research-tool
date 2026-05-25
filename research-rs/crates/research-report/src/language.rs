use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageQualityScore {
    pub schema_version: String,
    pub language: String,
    pub language_quality_score: u8,
    pub grade: String,
    pub presentation_status: String,
    pub generic_phrase_detected: bool,
    pub translationese_detected: bool,
    pub repeated_sentence_pattern: bool,
    pub unsupported_soft_claim: bool,
    pub vague_risk_language: bool,
    pub vague_growth_language: bool,
    pub vague_next_check: bool,
    pub missing_specific_subject: bool,
    pub too_many_empty_transitions: bool,
    pub chinese_english_mixed_heading: bool,
    pub english_report_contains_chinese_heading: bool,
    pub chinese_report_contains_untranslated_heading: bool,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LanguageLintResult {
    pub score: LanguageQualityScore,
    pub polish_trace: Vec<LanguagePolishTraceEntry>,
}

#[derive(Debug, Clone)]
pub struct LanguagePolishTraceEntry {
    pub section: String,
    pub issue: String,
    pub before_excerpt: String,
    pub after_excerpt: String,
    pub validator_result: String,
}

const GENERIC_ENGLISH: &[&str] = &[
    "strong potential",
    "investors should pay attention",
    "it is important to note",
    "in conclusion",
    "overall, the company faces opportunities and challenges",
    "the future remains uncertain",
    "based on the data, we can see",
    "robust growth prospects",
    "significant opportunities",
    "comprehensive analysis",
    "dynamic market environment",
];

const GENERIC_CHINESE: &[&str] = &[
    "该公司具有较强发展潜力",
    "投资者应持续关注",
    "未来存在一定不确定性",
    "机遇与挑战并存",
    "综合来看",
    "值得注意的是",
    "进一步研究是必要的",
    "从数据可以看出",
    "具备良好的成长空间",
    "行业前景广阔",
];

const TRANSLATIONESE_CHINESE: &[&str] = &[
    "这表明了",
    "对于投资者来说是值得关注的",
    "可能会面临一些不确定性",
    "盈利能力是重要的",
];

const SPECIFIC_SUBJECTS: &[&str] = &[
    "revenue",
    "margin",
    "fcf",
    "free cash flow",
    "capex",
    "debt",
    "cash flow",
    "segment",
    "valuation",
    "multiple",
    "data gap",
    "营业收入",
    "利润率",
    "经营现金流",
    "自由现金流",
    "资本开支",
    "有息负债",
    "估值",
    "数据缺口",
    "货币资金",
    "应收账款",
    "存货",
];

pub fn language_lint(report: &str, language: &str) -> LanguageLintResult {
    let lower = report.to_lowercase();
    let generic_phrase_detected = GENERIC_ENGLISH.iter().any(|phrase| lower.contains(phrase))
        || GENERIC_CHINESE.iter().any(|phrase| report.contains(phrase));
    let translationese_detected = TRANSLATIONESE_CHINESE
        .iter()
        .any(|phrase| report.contains(phrase));
    let repeated_sentence_pattern = repeated_starter_count(report) > 4;
    let vague_next_check = report
        .lines()
        .filter(|line| {
            let l = line.to_lowercase();
            l.contains("next check")
                || l.contains("下一步")
                || l.contains("manual check")
                || l.contains("人工核查")
        })
        .any(|line| {
            let l = line.to_lowercase();
            l.contains("more research")
                || l.contains("further research")
                || line.contains("继续研究")
                || line.contains("进一步研究")
        });
    let vague_risk_language = contains_vague_phrase_without_subject(
        report,
        &["risk is high", "high risk", "风险较高", "风险较大"],
    );
    let vague_growth_language = contains_vague_phrase_without_subject(
        report,
        &[
            "growth potential",
            "growth prospects",
            "成长空间",
            "发展潜力",
        ],
    );
    let unsupported_soft_claim = contains_vague_phrase_without_subject(
        report,
        &["attractive", "promising", "值得关注", "前景较好"],
    );
    let missing_specific_subject = paragraphs(report)
        .iter()
        .filter(|p| p.len() > 80)
        .any(|p| !contains_specific_subject(p));
    let too_many_empty_transitions = empty_transition_count(report) > 4;
    let english_report_contains_chinese_heading = language == "en"
        && report
            .lines()
            .filter(|line| line.starts_with('#'))
            .any(contains_cjk);
    let chinese_report_contains_untranslated_heading = language == "zh"
        && report
            .lines()
            .filter(|line| line.starts_with('#'))
            .any(|line| {
                [
                    "Report Status",
                    "Company Identity",
                    "Business Model",
                    "Money Flow",
                    "Financial Statement",
                    "Research Blueprint",
                    "Valuation Frame",
                    "Risks and Red Flags",
                    "Data Gaps",
                    "Next Checks",
                    "Appendix",
                ]
                .iter()
                .any(|needle| line.contains(needle))
            });
    let chinese_english_mixed_heading =
        english_report_contains_chinese_heading || chinese_report_contains_untranslated_heading;

    let checks = [
        ("generic_phrase_detected", generic_phrase_detected, 12),
        ("translationese_detected", translationese_detected, 10),
        ("repeated_sentence_pattern", repeated_sentence_pattern, 8),
        ("unsupported_soft_claim", unsupported_soft_claim, 10),
        ("vague_risk_language", vague_risk_language, 10),
        ("vague_growth_language", vague_growth_language, 10),
        ("vague_next_check", vague_next_check, 10),
        ("missing_specific_subject", missing_specific_subject, 8),
        ("too_many_empty_transitions", too_many_empty_transitions, 6),
        (
            "chinese_english_mixed_heading",
            chinese_english_mixed_heading,
            12,
        ),
    ];
    let mut score = 100i32;
    let mut findings = Vec::new();
    for (name, failed, penalty) in checks {
        if failed {
            score -= penalty;
            findings.push(name.to_string());
        }
    }
    let score = score.clamp(0, 100) as u8;
    let grade = match score {
        90..=100 => "natural",
        80..=89 => "good",
        70..=79 => "acceptable",
        60..=69 => "weak",
        _ => "fail",
    }
    .to_string();
    let presentation_status = if score < 60 {
        "FAIL"
    } else if score < 70 {
        "WARNING"
    } else {
        "PASS"
    }
    .to_string();

    LanguageLintResult {
        score: LanguageQualityScore {
            schema_version: "v5.0.0".to_string(),
            language: language.to_string(),
            language_quality_score: score,
            grade,
            presentation_status,
            generic_phrase_detected,
            translationese_detected,
            repeated_sentence_pattern,
            unsupported_soft_claim,
            vague_risk_language,
            vague_growth_language,
            vague_next_check,
            missing_specific_subject,
            too_many_empty_transitions,
            chinese_english_mixed_heading,
            english_report_contains_chinese_heading,
            chinese_report_contains_untranslated_heading,
            findings,
        },
        polish_trace: Vec::new(),
    }
}

pub fn language_polish(report: &str, language: &str) -> (String, Vec<LanguagePolishTraceEntry>) {
    let mut polished = report.to_string();
    let mut trace = Vec::new();
    let replacements = if language == "zh" {
        vec![
            ("综合来看，", ""),
            ("值得注意的是，", ""),
            ("进一步研究是必要的", "下一步要核查具体数据缺口"),
            ("未来存在一定不确定性", "这份报告还不能验证相关风险"),
            ("机遇与挑战并存", "风险和机会需要分开核查"),
            ("从数据可以看出", "目前锁定数据支持"),
        ]
    } else {
        vec![
            ("Based on the data, we can see", "The locked data supports"),
            ("It is important to note that", "The key issue is"),
            ("In conclusion,", ""),
            (
                "overall, the company faces opportunities and challenges",
                "the report separates verified evidence from unresolved risks",
            ),
            (
                "the future remains uncertain",
                "this report cannot verify the future path",
            ),
            (
                "investors should pay attention",
                "the next manual check should focus",
            ),
        ]
    };
    for (before, after) in replacements {
        if polished.contains(before) {
            polished = polished.replace(before, after);
            trace.push(LanguagePolishTraceEntry {
                section: "language_polish".to_string(),
                issue: "generic_or_translationese_phrase".to_string(),
                before_excerpt: before.to_string(),
                after_excerpt: after.to_string(),
                validator_result: "PASS".to_string(),
            });
        }
    }
    (polished, trace)
}

pub fn language_naturalness_markdown(result: &LanguageLintResult) -> String {
    let findings = if result.score.findings.is_empty() {
        "- No generic or translationese language patterns detected.".to_string()
    } else {
        result
            .score
            .findings
            .iter()
            .map(|finding| format!("- {finding}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "# Language Naturalness Report\n\nStatus: {}\n\n| Metric | Value |\n|---|---:|\n| Language quality score | {} |\n| Grade | {} |\n| Language | {} |\n\n## Findings\n\n{}\n\n## Standard\n\nThe report should read like a research memo: specific subjects, concrete risks, executable next checks, no generic AI prose, and no mixed-language headings.\n",
        result.score.presentation_status,
        result.score.language_quality_score,
        result.score.grade,
        result.score.language,
        findings
    )
}

pub fn language_polish_trace_markdown(trace: &[LanguagePolishTraceEntry]) -> String {
    if trace.is_empty() {
        return "# Language Polish Trace\n\nStatus: PASS\n\nNo language polish rewrite was required.\n"
            .to_string();
    }
    let rows = trace
        .iter()
        .map(|entry| {
            format!(
                "| {} | {} | {} | {} | {} |",
                entry.section,
                entry.issue,
                entry.before_excerpt.replace('|', "/"),
                entry.after_excerpt.replace('|', "/"),
                entry.validator_result
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "# Language Polish Trace\n\nStatus: PASS\n\n| Section | Issue | Before | After | Validator |\n|---|---|---|---|---|\n{}\n",
        rows
    )
}

fn paragraphs(report: &str) -> Vec<String> {
    report
        .split("\n\n")
        .map(str::trim)
        .filter(|p| {
            !p.is_empty()
                && !p.starts_with('#')
                && !p.starts_with('|')
                && !p.starts_with('>')
                && !p.starts_with("- ")
        })
        .map(ToString::to_string)
        .collect()
}

fn contains_specific_subject(text: &str) -> bool {
    let lower = text.to_lowercase();
    SPECIFIC_SUBJECTS
        .iter()
        .any(|subject| lower.contains(&subject.to_lowercase()))
}

fn contains_vague_phrase_without_subject(report: &str, phrases: &[&str]) -> bool {
    paragraphs(report).iter().any(|paragraph| {
        let lower = paragraph.to_lowercase();
        phrases
            .iter()
            .any(|phrase| lower.contains(&phrase.to_lowercase()))
            && !contains_specific_subject(paragraph)
    })
}

fn repeated_starter_count(report: &str) -> usize {
    let starters = [
        "This means",
        "This matters",
        "Overall",
        "It is important",
        "值得注意的是",
        "综合来看",
        "这意味着",
    ];
    starters
        .iter()
        .map(|starter| {
            report
                .lines()
                .filter(|line| line.trim_start().starts_with(starter))
                .count()
        })
        .max()
        .unwrap_or(0)
}

fn empty_transition_count(report: &str) -> usize {
    [
        "overall",
        "in addition",
        "moreover",
        "值得注意的是",
        "综合来看",
        "此外",
    ]
    .iter()
    .map(|phrase| {
        report
            .to_lowercase()
            .matches(&phrase.to_lowercase())
            .count()
    })
    .sum()
}

fn contains_cjk(line: &str) -> bool {
    line.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}
