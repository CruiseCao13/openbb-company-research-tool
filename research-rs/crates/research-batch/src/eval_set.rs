use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EvalSet {
    pub name: String,
    pub tickers: Vec<String>,
    pub expected_family: BTreeMap<String, String>,
}

fn group_family(group: &str) -> String {
    match group {
        "mature_compounder" => "Mature Compounder",
        "mega_cap_tech" => "Platform Internet / Mega-cap Tech",
        "semiconductor_ai" => "AI Semiconductor / Data Center Growth Compounder",
        "semiconductor_turnaround" => "Capital-Intensive Semiconductor Turnaround",
        "speculative_growth" => "Speculative Growth",
        "biotech_like" => "Biotech / Pharma Research Frame",
        "pharma" => "Biotech / Pharma Research Frame",
        "medical_devices" => "Medical Devices",
        "banks" | "brokers_exchanges" => "Financials / Bank-like Screening",
        "insurance" => "Insurance-like Screening",
        "reit" => "REIT-like Screening",
        "energy" | "materials_mining" | "industrials" => "Cyclical / Industrial Cycle",
        "aerospace_defense" => "Aerospace / Defense",
        "airlines_transport" | "shipping_logistics" => "Shipping / Airlines / Transport Cycle",
        "consumer_retail" | "restaurants" | "luxury_apparel" => "Consumer / Retail",
        "utilities" => "Utilities / Infrastructure",
        "telecom_media" => "Telecom / Infrastructure Cash Flow",
        "payments_fintech" => "Payments / Fintech",
        _ => "Unknown / Data-Limited Screening",
    }
    .to_string()
}

pub fn load_eval_set(path: &Path) -> Result<EvalSet> {
    let raw = fs::read_to_string(path)?;
    let mut name = path.file_stem().unwrap().to_string_lossy().to_string();
    let mut tickers = Vec::new();
    let mut expected_family = BTreeMap::new();
    let mut current_group: Option<String> = None;
    for line in raw.lines() {
        let stripped = line.split('#').next().unwrap_or("").trim();
        if stripped.is_empty() {
            continue;
        }
        if stripped.starts_with("name:") {
            name = stripped.split_once(':').unwrap().1.trim().to_string();
        } else if line.starts_with("  ") && !line.starts_with("    ") && stripped.ends_with(':') {
            current_group = Some(stripped.trim_end_matches(':').to_string());
        } else if stripped.starts_with("- ") {
            let ticker = stripped.trim_start_matches("- ").trim().to_uppercase();
            if !tickers.contains(&ticker) {
                tickers.push(ticker.clone());
            }
            if let Some(group) = &current_group {
                expected_family.insert(ticker, group_family(group));
            }
        }
    }
    Ok(EvalSet {
        name,
        tickers,
        expected_family,
    })
}
